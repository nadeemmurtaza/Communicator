#![allow(dead_code)]

use crate::db::{AppState, DbPool};
use crate::routes::auth::Tenant;
use crate::services::inbound;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// The channel vocabulary the rest of the backend actually persists
/// (`inboxes.channel_type`, `contacts.channel`, the webhook routes in main.rs).
/// Anything outside this list is rejected rather than silently defaulted to
/// WhatsApp the way the legacy CSV importer did.
pub const SUPPORTED_CHANNELS: [&str; 10] = [
    "whatsapp",
    "sms",
    "email",
    "facebook",
    "instagram",
    "telegram",
    "line",
    "voice",
    "tiktok",
    "website",
];

pub fn is_supported_channel(value: &str) -> bool {
    SUPPORTED_CHANNELS.contains(&value.trim().to_lowercase().as_str())
}

fn channel_title(channel: &str) -> String {
    let mut chars = channel.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Channels whose identity is a phone number and therefore has to be E.164.
pub fn is_phone_channel(channel: &str) -> bool {
    matches!(channel, "whatsapp" | "sms" | "voice")
}

/// Strips separators and a leading `tel:` and returns `+<digits>` when the
/// result is a real E.164 subscriber number (7-15 digits, first digit non-zero).
pub fn normalize_e164(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches("tel:").trim();
    if trimmed.is_empty() {
        return None;
    }
    let digits: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let has_plus = trimmed.starts_with('+');
    if digits.len() < 7 || digits.len() > 15 {
        return None;
    }
    if digits.starts_with('0') {
        return None;
    }
    Some(if has_plus {
        format!("+{digits}")
    } else {
        format!("+{digits}")
    })
}

pub fn looks_like_email(raw: &str) -> bool {
    let value = raw.trim();
    match value.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !value.contains(' ')
        }
        None => false,
    }
}

/// True when `table` exists in this PostgreSQL database. Used instead of assuming a
/// schema the deployment may not have wired yet, so a route can name the exact
/// missing dependency instead of returning fabricated rows.
pub async fn table_exists(db: &DbPool, table: &str) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM information_schema.tables WHERE table_schema='public' AND table_name = $1")
        .bind(table)
        .fetch_one(db)
        .await
        .unwrap_or(0)
        > 0
}

pub async fn column_exists(db: &DbPool, table: &str, column: &str) -> bool {
    let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(db)
        .await
        .unwrap_or_default();
    rows.iter()
        .any(|r| r.try_get::<String, _>("name").unwrap_or_default() == column)
}

/// Renders a database failure as a real error body instead of an empty 200.
pub fn db_error_response(context: &str, err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("{context}: {err}"),
            "databaseError": true
        })),
    )
        .into_response()
}

pub fn missing_dependency_response(dependency: &str, detail: &str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "error": format!("Cannot serve this request: {dependency} is not available."),
            "missingDependency": dependency,
            "detail": detail,
            "retryable": false
        })),
    )
        .into_response()
}

/// One parsed CSV record: header name -> cell value (lowercased headers).
#[derive(Clone, Debug)]
pub struct CsvRecord {
    pub row_number: usize,
    pub values: HashMap<String, String>,
}

/// Minimal RFC-4180 reader: quoted fields, doubled quotes, CR/LF handling and a
/// UTF-8 BOM. Written here because the crate set has no CSV dependency.
pub fn parse_csv_records(text: &str) -> Vec<CsvRecord> {
    parse_csv_table(text).1
}

/// Same reader, but also hands back the header row in file order, which the
/// import preview has to show as `detectedColumns`.
pub fn parse_csv_table(text: &str) -> (Vec<String>, Vec<CsvRecord>) {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    let delimiter = {
        let first_line = body.lines().next().unwrap_or("");
        let commas = first_line.matches(',').count();
        let semicolons = first_line.matches(';').count();
        if semicolons > commas {
            ';'
        } else {
            ','
        }
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = body.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == delimiter {
            current.push(std::mem::take(&mut field));
        } else if c == '\n' || c == '\r' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            current.push(std::mem::take(&mut field));
            if !current.iter().any(|v| !v.trim().is_empty()) {
                current.clear();
                continue;
            }
            rows.push(std::mem::take(&mut current));
        } else {
            field.push(c);
        }
    }
    if !field.is_empty() || !current.is_empty() {
        current.push(field);
        if current.iter().any(|v| !v.trim().is_empty()) {
            rows.push(current);
        }
    }

    if rows.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // Normalized names key the records; the raw spelling is what the preview
    // reports back as `detectedColumns`.
    let headers: Vec<String> = rows[0]
        .iter()
        .map(|h| h.trim().to_lowercase().replace(' ', "_"))
        .collect();

    let records = rows
        .iter()
        .skip(1)
        .enumerate()
        .map(|(idx, cells)| {
            let mut values = HashMap::new();
            for (col, cell) in cells.iter().enumerate() {
                let key = headers
                    .get(col)
                    .cloned()
                    .unwrap_or_else(|| format!("column_{}", col + 1));
                values.insert(key, cell.trim().to_string());
            }
            // +1 for the header row, +1 for 1-based row numbers shown to humans.
            CsvRecord {
                row_number: idx + 2,
                values,
            }
        })
        .collect();

    let raw_headers: Vec<String> = rows[0].iter().map(|h| h.trim().to_string()).collect();
    (raw_headers, records)
}

fn first_field(row: &CsvRecord, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| row.values.get(*k))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Mirrors the legacy column mapping, but without the WhatsApp fallback: an
/// unrecognised channel is reported per row instead of being guessed at.
pub struct ResolvedRow {
    pub channel: Option<String>,
    pub value: Option<String>,
    pub name: String,
    /// None when no language column existed: an unrecorded language must never
    /// become a stored "en".
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Map<String, serde_json::Value>,
    pub errors: Vec<String>,
}

fn split_tags(raw: &str) -> Vec<String> {
    raw.split([',', '|', ';'])
        .map(|t| t.trim().trim_start_matches('#').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

pub fn resolve_csv_row(row: &CsvRecord, default_channel: Option<&str>) -> ResolvedRow {
    let mut resolved = ResolvedRow {
        channel: None,
        value: None,
        name: first_field(row, &["name", "person_name", "full_name", "display_name"])
            .unwrap_or_default(),
        language: first_field(row, &["language", "lang"]),
        tags: split_tags(&first_field(row, &["tags", "tag"]).unwrap_or_default()),
        custom_fields: serde_json::Map::new(),
        errors: Vec::new(),
    };

    let raw_value = first_field(
        row,
        &[
            "value", "phone", "whatsapp", "email", "number", "mobile", "handle", "open_id",
            "openid", "chat_id", "id",
        ],
    );
    let Some(raw_value) = raw_value else {
        resolved.errors.push(
            "no identity column (one of value, phone, whatsapp, email, number, mobile, handle, open_id, chat_id)".to_string(),
        );
        return resolved;
    };

    let raw_channel = first_field(row, &["channel", "channel_type", "type"])
        .or_else(|| default_channel.map(|c| c.trim().to_lowercase()))
        .map(|c| c.trim().to_lowercase());

    let channel = match raw_channel {
        Some(ref c) if is_supported_channel(c) => Some(c.clone()),
        Some(c) => {
            resolved
                .errors.push(format!("unknown channel '{c}' (supported: {SUPPORTED_CHANNELS:?})"));
            None
        }
        // No explicit channel: infer only from unambiguous evidence.
        None => {
            if looks_like_email(&raw_value) {
                Some("email".to_string())
            } else if raw_value.starts_with('@') {
                Some("instagram".to_string())
            } else if raw_value.starts_with("tt_") {
                Some("tiktok".to_string())
            } else if raw_value.starts_with("tg_") {
                Some("telegram".to_string())
            } else if raw_value.starts_with('U') && raw_value.len() >= 10 {
                Some("line".to_string())
            } else if normalize_e164(&raw_value).is_some() {
                resolved.errors.push(
                    "channel not stated and a phone number is not enough to pick one (whatsapp, sms and voice all take E.164)".to_string(),
                );
                None
            } else {
                resolved
                    .errors.push("channel not stated and cannot be inferred from the value".to_string());
                None
            }
        }
    };

    if let Some(ref ch) = channel {
        let normalized = if is_phone_channel(ch) {
            match normalize_e164(&raw_value) {
                Some(v) => Some(v),
                None => {
                    resolved.errors.push(format!(
                        "'{raw_value}' is not a valid E.164 number for the {ch} channel"
                    ));
                    None
                }
            }
        } else if ch == "email" {
            if looks_like_email(&raw_value) {
                Some(raw_value.to_lowercase())
            } else {
                resolved
                    .errors.push(format!("'{raw_value}' is not a valid email address"));
                None
            }
        } else {
            Some(raw_value.clone())
        };
        resolved.value = normalized;
    }

    let known_keys = [
        "value", "phone", "whatsapp", "email", "number", "mobile", "handle", "open_id", "openid",
        "chat_id", "id", "name", "person_name", "full_name", "display_name", "channel",
        "channel_type", "type", "tags", "tag", "language", "lang",
    ];
    for (key, value) in &row.values {
        if !known_keys.contains(&key.as_str()) && !value.is_empty() {
            resolved
                .custom_fields
                .insert(key.clone(), serde_json::Value::String(value.clone()));
        }
    }

    resolved
}

/// Natural key the schema deduplicates contacts on: `ix_contacts_channel_value`.
pub async fn existing_contact_id(
    db: &DbPool,
    channel: &str,
    value: &str,
) -> Result<Option<String>, sqlx::Error> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT id FROM contacts WHERE channel = $1 AND value = $2 LIMIT 1")
            .bind(channel)
            .bind(value)
            .fetch_optional(db)
            .await?,
    )
}

/// Find-or-create a tag row and link it to a contact, honouring the
/// `contact_tags` composite primary key so a re-import cannot double-link.
/// Returns whether this call actually wrote (created the tag or added a link),
/// so callers can distinguish a real change from an already-current state.
pub async fn attach_tag(
    db: &DbPool,
    contact_id: &str,
    tag_name: &str,
    tenant_key: &str,
) -> Result<bool, sqlx::Error> {
    let clean = tag_name.trim().trim_start_matches('#');
    if clean.is_empty() {
        return Ok(false);
    }

    let mut wrote = false;
    let tag_id: Option<String> = sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
        .bind(clean)
        .fetch_optional(db)
        .await?;

    let tag_id = match tag_id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO tags (id, name, created_at, tenant_key) VALUES ($1, $2, $3, $4)")
                .bind(&id)
                .bind(clean)
                .bind(Utc::now().to_rfc3339())
                .bind(tenant_key)
                .execute(db)
                .await?;
            wrote = true;
            id
        }
    };

    let linked = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM contact_tags WHERE contact_id = $1 AND tag_id = $2",
    )
    .bind(contact_id)
    .bind(&tag_id)
    .fetch_one(db)
    .await?;

    if linked == 0 {
        sqlx::query("INSERT INTO contact_tags (contact_id, tag_id) VALUES ($1, $2)")
            .bind(contact_id)
            .bind(&tag_id)
            .execute(db)
            .await?;
        wrote = true;
    }
    Ok(wrote)
}

#[derive(Deserialize, Debug)]
pub struct ContactQueryParams {
    pub tag: Option<String>,
    pub channel: Option<String>,
    pub search: Option<String>,
    pub segment_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactRecord {
    pub id: String,
    pub value: String,
    pub channel: String,
    pub person_name: String,
    pub role: String,
    pub created_at: String,
    /// Null when the platform never told us; "en" would be a guess the UI reads
    /// as a recorded preference.
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    /// Null when no consent row exists — absent consent is not a consent state.
    pub opt_in_status: Option<String>,
    pub quality_rating: Option<String>,
}

/// Dynamic CRM segment filter matching against PostgreSQL contact and tags.
/// `Ok(None)` means the segment id is not in `contact_segments` at all, which
/// must not be reported as "matches everything"; `Err` is a database failure.
async fn contact_matches_segment(
    db: &sqlx::PgPool,
    segment_id: &str,
    channel: &str,
    tags: &[String],
) -> Result<Option<bool>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct SegFilters {
        filters: Option<String>,
    }

    let row = sqlx::query_as::<_, SegFilters>(
        "SELECT filters FROM contact_segments WHERE id = $1",
    )
    .bind(segment_id)
    .fetch_optional(db)
    .await?;

    let Some(SegFilters { filters }) = row else {
        return Ok(None);
    };
    let Some(f_str) = filters.filter(|f| !f.trim().is_empty()) else {
        return Ok(Some(true));
    };

    let Ok(filter_json) = serde_json::from_str::<serde_json::Value>(&f_str) else {
        return Ok(Some(true));
    };

    // 1. Channel filter
    if let Some(ch_arr) = filter_json.get("channels").and_then(|c| c.as_array()) {
        if !ch_arr.is_empty() {
            let matches_ch = ch_arr.iter().any(|v| v.as_str() == Some(channel));
            if !matches_ch {
                return Ok(Some(false));
            }
        }
    }

    // 2. Tags filter
    if let Some(tag_arr) = filter_json.get("tags").and_then(|t| t.as_array()) {
        if !tag_arr.is_empty() {
            let mode = filter_json
                .get("tag_match_mode")
                .and_then(|m| m.as_str())
                .unwrap_or("any");
            let req_tags: Vec<&str> = tag_arr.iter().filter_map(|t| t.as_str()).collect();
            if mode == "all" {
                let has_all = req_tags.iter().all(|rt| tags.iter().any(|t| t == rt));
                if !has_all {
                    return Ok(Some(false));
                }
            } else {
                let has_any = req_tags.iter().any(|rt| tags.iter().any(|t| t == rt));
                if !has_any {
                    return Ok(Some(false));
                }
            }
        }
    }

    Ok(Some(true))
}

/// Renders the "cannot evaluate this segment" outcomes for the two list paths.
fn segment_lookup_failed_response(err: sqlx::Error) -> Response {
    db_error_response("could not read the contact segment filters", err)
}

fn unknown_segment_response(segment_id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": format!("contact segment '{segment_id}' does not exist"),
            "segmentId": segment_id
        })),
    )
        .into_response()
}

/// Which columns this deployment actually created on `contacts`. The Rust
/// schema grows through `ALTER TABLE` statements owned by another file, so the
/// read paths build their projection from what is really present instead of
/// naming columns that may not exist and 500-ing at runtime.
pub async fn contact_column_names(db: &DbPool) -> HashSet<String> {
    let rows = sqlx::query("PRAGMA table_info(contacts)")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    rows.iter()
        .filter_map(|r| r.try_get::<String, _>("name").ok())
        .map(|n| n.to_lowercase())
        .collect()
}

/// Columns the CRM routes want on `contacts` that the base `CREATE TABLE` does
/// not guarantee; each is added by an `ALTER TABLE` owned by another file.
pub const CONTACT_OPTIONAL_COLUMNS: [&str; 7] = [
    "role",
    "language",
    "quality_rating",
    "custom_fields",
    "lead_id",
    "customer_id",
    "tenant_key",
];

/// Splits the desired contact projection into what this database really has and
/// what is still missing, so reads never name an absent column.
pub async fn contact_projection(db: &DbPool) -> (Vec<&'static str>, Vec<&'static str>) {
    let cols = contact_column_names(db).await;
    let mut present: Vec<&'static str> = vec!["id", "value", "channel", "person_name", "created_at"];
    let mut missing: Vec<&'static str> = Vec::new();
    for candidate in CONTACT_OPTIONAL_COLUMNS {
        if cols.contains(candidate) {
            present.push(candidate);
        } else {
            missing.push(candidate);
        }
    }
    (present, missing)
}

/// A contact read through the dynamic projection: columns the database does not
/// have come back as `None` instead of failing the whole query.
pub struct LoadedContact {
    pub id: String,
    pub value: String,
    pub channel: String,
    pub person_name: Option<String>,
    pub role: Option<String>,
    pub created_at: String,
    pub language: Option<String>,
    pub quality_rating: Option<String>,
    pub custom_fields: Option<String>,
    pub lead_id: Option<String>,
    pub customer_id: Option<String>,
    pub tenant_key: Option<String>,
}

impl LoadedContact {
    pub fn from_row(row: &sqlx::postgres::PgRow) -> Self {
        let text = |name: &str| -> Option<String> {
            row.try_get::<Option<String>, _>(name)
                .ok()
                .flatten()
                .or_else(|| row.try_get::<String, _>(name).ok())
        };
        Self {
            id: row.try_get("id").unwrap_or_default(),
            value: row.try_get("value").unwrap_or_default(),
            channel: row.try_get("channel").unwrap_or_default(),
            person_name: text("person_name"),
            role: text("role"),
            created_at: text("created_at").unwrap_or_default(),
            language: text("language"),
            quality_rating: text("quality_rating"),
            custom_fields: text("custom_fields"),
            lead_id: text("lead_id"),
            customer_id: text("customer_id"),
            tenant_key: text("tenant_key"),
        }
    }

    pub fn custom_fields_json(&self) -> serde_json::Value {
        match self.custom_fields.as_deref() {
            Some(raw) if !raw.trim().is_empty() => serde_json::from_str(raw)
                .unwrap_or_else(|_| json!({ "_unparseable": raw.to_string() })),
            _ => json!({}),
        }
    }
}

/// `contacts.custom_fields` is the only place the ported schema can keep Meta
/// telemetry, so read it back defensively rather than inventing a rating.
pub fn field_string(fields: &serde_json::Value, key: &str) -> Option<String> {
    fields.get(key).and_then(|v| match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

/// DATETIME columns in this schema are written as RFC-3339 text;
/// normalize before comparing.
pub fn parse_db_timestamp(raw: &str) -> Option<chrono::DateTime<Utc>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Utc));
    }
    let naive = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&trimmed.replace(' ', "T"), "%Y-%m-%dT%H:%M:%S%.f"))
        .ok()?;
    Some(Utc.from_utc_datetime(&naive))
}

pub fn max_timestamp(a: Option<&str>, b: Option<&str>) -> Option<String> {
    let pa = a.and_then(parse_db_timestamp);
    let pb = b.and_then(parse_db_timestamp);
    match (pa, pb) {
        (Some(x), Some(y)) => Some(if x >= y {
            a.unwrap().to_string()
        } else {
            b.unwrap().to_string()
        }),
        (Some(_), None) => a.map(|s| s.to_string()),
        (None, Some(_)) => b.map(|s| s.to_string()),
        (None, None) => None,
    }
}

pub async fn list_contacts(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(params): Query<ContactQueryParams>,
) -> impl IntoResponse {
    let (projection, missing) = contact_projection(&state.db).await;
    let scoped = projection.contains(&"tenant_key");
    let sql = format!(
        "SELECT {} FROM contacts {} ORDER BY created_at DESC",
        projection.join(", "),
        if scoped {
            "WHERE COALESCE(tenant_key, '') = $1"
        } else {
            ""
        }
    );

    let mut query = sqlx::query(&sql);
    if scoped {
        query = query.bind(&tenant_key);
    }
    let rows = match query.fetch_all(&state.db).await {
        Ok(rows) => rows,
        Err(err) => return db_error_response("could not read contacts", err),
    };

    let mut result = Vec::new();
    for row in rows {
        let id: String = match row.try_get("id") {
            Ok(v) => v,
            Err(err) => return db_error_response("contacts row has no readable id", err),
        };
        let value: String = row.try_get("value").unwrap_or_default();
        let channel: String = row.try_get("channel").unwrap_or_default();
        let person_name: Option<String> = row.try_get("person_name").ok().flatten();
        let created_at: Option<String> = row.try_get("created_at").ok().flatten();
        let role: Option<String> = row.try_get("role").ok().flatten();
        let language: Option<String> = row.try_get("language").ok().flatten();
        let quality_rating: Option<String> = row.try_get("quality_rating").ok().flatten();
        let raw_custom_fields: Option<String> = row.try_get("custom_fields").ok().flatten();

        // Query tags for contact
        let tags: Vec<String> = match sqlx::query_scalar::<_, String>(
            r#"
            SELECT t.name 
            FROM contact_tags ct 
            JOIN tags t ON ct.tag_id = t.id 
            WHERE ct.contact_id = $1
            ORDER BY t.name ASC
            "#,
        )
        .bind(&id)
        .fetch_all(&state.db)
        .await
        {
            Ok(tags) => tags,
            Err(err) => return db_error_response("could not read tags for a contact", err),
        };

        // Query consent
        let opt_in: Option<String> = match sqlx::query_scalar::<_, String>(
            "SELECT status FROM consent_records WHERE contact_id = $1 ORDER BY recorded_at DESC LIMIT 1",
        )
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(status) => status,
            Err(err) => return db_error_response("could not read the consent record", err),
        };

        // Filter by segment if requested
        if let Some(ref seg_id) = params.segment_id {
            if seg_id != "all" {
                match contact_matches_segment(&state.db, seg_id, &channel, &tags).await {
                    Err(err) => return segment_lookup_failed_response(err),
                    Ok(None) => return unknown_segment_response(seg_id),
                    Ok(Some(false)) => continue,
                    Ok(Some(true)) => {}
                }
            }
        }

        // Filter by tag if requested
        if let Some(ref t) = params.tag {
            if t != "all" && !tags.contains(t) {
                continue;
            }
        }
        // Filter by channel if requested
        if let Some(ref ch) = params.channel {
            if ch != "all" && channel != *ch {
                continue;
            }
        }
        // Filter by search query if requested
        if let Some(ref q) = params.search {
            let q_lower = q.to_lowercase();
            let name_match = person_name.as_deref().unwrap_or("").to_lowercase().contains(&q_lower);
            let val_match = value.to_lowercase().contains(&q_lower);
            if !name_match && !val_match {
                continue;
            }
        }

        let custom_fields: serde_json::Value = match raw_custom_fields.as_deref() {
            Some(raw) if !raw.trim().is_empty() => serde_json::from_str(raw)
                .unwrap_or_else(|_| json!({ "_unparseable": raw.to_string() })),
            _ => json!({}),
        };

        result.push(ContactRecord {
            id,
            value,
            channel,
            person_name: person_name.unwrap_or_default(),
            role: role.unwrap_or_default(),
            created_at: created_at.unwrap_or_default(),
            language,
            tags,
            custom_fields,
            // No consent row means no recorded consent; it is not the same thing
            // as being opted in, and compliance.rs treats unknown as unsendable.
            opt_in_status: opt_in,
            quality_rating,
        });
    }

    if missing.is_empty() {
        return Json(result).into_response();
    }
    // The directory still renders, but the operator is told which columns are
    // absent rather than being shown fabricated defaults as if they were data.
    // The body stays an array because that is the shape the frontend parses, so
    // the warning travels in a header.
    let mut response = Json(result).into_response();
    if let Ok(header) = HeaderValue::from_str(&format!(
        "contacts table is missing column(s): {}; those fields are reported as null or empty",
        missing.join(", ")
    )) {
        response.headers_mut().insert("x-warnings", header);
    }
    response
}

pub async fn list_contact_tags(State(state): State<AppState>) -> Response {
    #[derive(Serialize, sqlx::FromRow)]
    struct TagRow {
        id: String,
        name: String,
    }

    // An empty list has to mean "there are no tags", never "the read failed".
    let tags = match sqlx::query_as::<_, TagRow>("SELECT id, name FROM tags ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
    {
        Ok(tags) => tags,
        Err(err) => return db_error_response("could not read tags", err),
    };

    Json(tags).into_response()
}

pub async fn get_contact(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (projection, missing) = contact_projection(&state.db).await;
    let scoped = projection.contains(&"tenant_key");
    let sql = format!(
        "SELECT {} FROM contacts WHERE id = $1{}",
        projection.join(", "),
        if scoped {
            " AND COALESCE(tenant_key, '') = $2"
        } else {
            ""
        }
    );
    let mut query = sqlx::query(&sql).bind(&id);
    if scoped {
        query = query.bind(&tenant_key);
    }

    let row = match query.fetch_optional(&state.db).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Contact {id} does not exist for tenant '{tenant_key}'"), "id": id })),
            )
                .into_response()
        }
        Err(err) => return db_error_response("could not read contact", err),
    };

    let c = LoadedContact::from_row(&row);
    let id = c.id.clone();

    let tags: Vec<String> = match sqlx::query_scalar::<_, String>(
        r#"
        SELECT t.name 
        FROM contact_tags ct 
        JOIN tags t ON ct.tag_id = t.id
        WHERE ct.contact_id = $1
        ORDER BY t.name ASC
        "#,
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    {
        Ok(tags) => tags,
        Err(err) => return db_error_response("could not read tags for the contact", err),
    };

    #[derive(sqlx::FromRow)]
    struct RawConsent {
        status: String,
        recorded_at: Option<String>,
    }

    // A failed consent read must not be reported as "no consent recorded": the
    // profile UI would show an absent opt-in for somebody who did opt in.
    let consent = match sqlx::query_as::<_, RawConsent>(
        "SELECT status, recorded_at FROM consent_records WHERE contact_id = $1 ORDER BY recorded_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(consent) => consent,
        Err(err) => return db_error_response("could not read the consent record", err),
    };

    let opt_in_status = consent.as_ref().map(|cr| cr.status.clone());
    // No consent row, no consent timestamp. The contact's creation time is not
    // evidence of when somebody opted in.
    let recorded_at = consent.and_then(|cr| cr.recorded_at);

    // Custom fields directly from contacts table
    let custom_fields: serde_json::Value = match c.custom_fields.as_deref() {
        Some(raw) if !raw.trim().is_empty() => serde_json::from_str(raw)
            .unwrap_or_else(|_| json!({ "_unparseable": raw.to_string() })),
        _ => json!({}),
    };

    // Latest Conversation Telemetry directly from conversations table
    #[derive(sqlx::FromRow)]
    struct RawConv {
        id: String,
        state: String,
        priority: Option<String>,
        resolution_type: Option<String>,
        csat_score: Option<i64>,
        last_inbound_at: Option<String>,
        last_outbound_at: Option<String>,
    }

    let conv = match sqlx::query_as::<_, RawConv>(
        "SELECT id, state, priority, resolution_type, csat_score, last_inbound_at, last_outbound_at FROM conversations WHERE contact_id = $1 ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(conv) => conv,
        Err(err) => return db_error_response("could not read the latest conversation", err),
    };

    let mut conversation_json = serde_json::Value::Null;
    let mut recent_messages_json = Vec::new();

    if let Some(ref cv) = conv {
        let total_msgs: i64 = match sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = $1")
            .bind(&cv.id)
            .fetch_one(&state.db)
            .await
        {
            Ok(n) => n,
            Err(err) => return db_error_response("could not count the messages of the conversation", err),
        };

        let inbound_count: i64 = match sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = $1 AND UPPER(direction) = 'INBOUND'")
            .bind(&cv.id)
            .fetch_one(&state.db)
            .await
        {
            Ok(n) => n,
            Err(err) => return db_error_response("could not count the inbound messages", err),
        };

        let outbound_count: i64 = match sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = $1 AND UPPER(direction) = 'OUTBOUND'")
            .bind(&cv.id)
            .fetch_one(&state.db)
            .await
        {
            Ok(n) => n,
            Err(err) => return db_error_response("could not count the outbound messages", err),
        };

        #[derive(sqlx::FromRow)]
        struct MsgSnippet {
            body: String,
            sent_at: String,
        }

        let last_inbound = match sqlx::query_as::<_, MsgSnippet>(
            "SELECT body, sent_at FROM messages WHERE conversation_id = $1 AND UPPER(direction) = 'INBOUND' ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(&cv.id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(snippet) => snippet,
            Err(err) => return db_error_response("could not read the last inbound message", err),
        };

        #[derive(sqlx::FromRow)]
        struct OutboundSnippet {
            body: String,
            sent_at: String,
            status: Option<String>,
        }

        let last_outbound = match sqlx::query_as::<_, OutboundSnippet>(
            "SELECT body, sent_at, status FROM messages WHERE conversation_id = $1 AND UPPER(direction) = 'OUTBOUND' ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(&cv.id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(snippet) => snippet,
            Err(err) => return db_error_response("could not read the last outbound message", err),
        };

        conversation_json = json!({
            "id": cv.id,
            "state": cv.state.to_lowercase(),
            // Null when nobody set a priority; "medium" is not a recorded value.
            "priority": cv.priority.clone(),
            "resolutionType": cv.resolution_type,
            "totalMessages": total_msgs,
            "inboundCount": inbound_count,
            "outboundCount": outbound_count,
            "lastInboundText": last_inbound.as_ref().map(|m| m.body.clone()),
            "lastInboundAt": last_inbound.as_ref().map(|m| m.sent_at.clone()).or_else(|| cv.last_inbound_at.clone()),
            "lastOutboundText": last_outbound.as_ref().map(|m| m.body.clone()),
            "lastOutboundAt": last_outbound.as_ref().map(|m| m.sent_at.clone()).or_else(|| cv.last_outbound_at.clone()),
            "lastOutboundStatus": last_outbound.as_ref().and_then(|m| m.status.clone()),
            "csatScore": cv.csat_score
        });

        #[derive(sqlx::FromRow)]
        struct RecentMsg {
            id: String,
            direction: String,
            body: String,
            sent_at: String,
            status: Option<String>,
        }

        let recents = match sqlx::query_as::<_, RecentMsg>(
            "SELECT id, direction, body, sent_at, status FROM messages WHERE conversation_id = $1 ORDER BY sent_at DESC LIMIT 15",
        )
        .bind(&cv.id)
        .fetch_all(&state.db)
        .await
        {
            Ok(recents) => recents,
            Err(err) => return db_error_response("could not read the recent messages", err),
        };

        for rm in recents {
            recent_messages_json.push(json!({
                "id": rm.id,
                "direction": rm.direction.to_lowercase(),
                "body": rm.body,
                "sentAt": rm.sent_at,
                "status": rm.status
            }));
        }
    }

    // Memories directly from contact_memories table
    #[derive(sqlx::FromRow)]
    struct MemoryItem {
        id: String,
        fact: String,
        category: Option<String>,
        confidence: Option<String>,
        noted_at: Option<String>,
        created_at: Option<String>,
    }

    let mut profile_warnings: Vec<String> = Vec::new();
    for column in &missing {
        profile_warnings.push(format!("contacts.{column} is not part of this schema; its field is reported as null or empty"));
    }

    let memories: Vec<MemoryItem> = if table_exists(&state.db, "contact_memories").await {
        match sqlx::query_as::<_, MemoryItem>(
            "SELECT id, fact, category, confidence, noted_at, created_at FROM contact_memories WHERE contact_id = $1 ORDER BY COALESCE(noted_at, created_at) DESC",
        )
        .bind(&id)
        .fetch_all(&state.db)
        .await
        {
            Ok(rows) => rows,
            Err(err) => return db_error_response("could not read contact memories", err),
        }
    } else {
        profile_warnings.push(
            "contact_memories table is not wired; remembered facts cannot be read".to_string(),
        );
        Vec::new()
    };

    let memories_json: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            let conf: Option<f64> = m.confidence.as_deref().and_then(|c| c.parse().ok());
            if conf.is_none() {
                profile_warnings.push(format!("memory {} has no parseable confidence value", m.id));
            }
            json!({
                "id": m.id,
                "fact": m.fact,
                "category": m.category,
                "confidence": conf,
                "notedAt": m.noted_at.or(m.created_at.clone()),
                "createdAt": m.created_at,
                "source": "contact_memory"
            })
        })
        .collect();

    // Lead Info from leads table if contact has a lead_id
    let mut lead_json = serde_json::Value::Null;
    if let Some(ref lid) = c.lead_id {
        #[derive(sqlx::FromRow)]
        struct RawLead {
            id: String,
            name: Option<String>,
            qualification_rationale: Option<String>,
            harvester_institution_id: Option<String>,
            institution_type: Option<String>,
            city: Option<String>,
        }

        match sqlx::query_as::<_, RawLead>(
            "SELECT id, name, qualification_rationale, harvester_institution_id, institution_type, city FROM leads WHERE id = $1",
        )
        .bind(lid)
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some(lead)) => {
                lead_json = json!({
                    "id": lead.id,
                    "name": lead.name,
                    "qualificationRationale": lead.qualification_rationale,
                    "institutionId": lead.harvester_institution_id,
                    "institutionType": lead.institution_type,
                    "city": lead.city
                });
            }
            Ok(None) => profile_warnings.push(format!(
                "lead {lid} is referenced by the contact but is not in the leads table"
            )),
            Err(err) => return db_error_response("could not read the lead of the contact", err),
        }
    }

    // Meta's wa_id is the WhatsApp identifier for a number; an email, Telegram
    // or Instagram identity has no wa_id, so deriving one from `value` would
    // publish a plausible-looking but invented provider id.
    let derived_wa_id = if c.channel == "whatsapp" {
        Some(c.value.replace('+', ""))
    } else {
        None
    };

    // Meta telemetry is only real when a sync wrote it onto the record; the
    // quality rating, name status and verification flags are read from
    // custom_fields and reported as null when no provider sync has run.
    let cf_get = |key: &str| custom_fields.get(key).cloned();
    let response_data = json!({
        "id": c.id,
        "value": c.value,
        "personName": c.person_name,
        "channel": c.channel,
        "role": c.role.unwrap_or_default(),
        "createdAt": c.created_at,
        "language": c.language,
        "tags": tags,
        "customFields": custom_fields,
        "meta": {
            "waId": cf_get("wa_id").and_then(|v| v.as_str().map(String::from)).or(derived_wa_id),
            "isWhatsAppRegistered": c.channel == "whatsapp",
            "qualityRating": cf_get("quality_rating").and_then(|v| v.as_str().map(String::from)).or(c.quality_rating.clone()),
            "optInStatus": opt_in_status.as_deref().map(|s| s.to_lowercase()),
            "optInTimestamp": recorded_at,
            "verified": cf_get("meta_verified"),
            "syncedAt": cf_get("meta_synced_at"),
            "nameStatus": cf_get("name_status"),
            "codeVerificationStatus": cf_get("code_verification_status")
        },
        "conversation": conversation_json,
        "memories": memories_json,
        "recentMessages": recent_messages_json,
        "lead": lead_json,
        "warnings": profile_warnings
    });

    (StatusCode::OK, Json(response_data)).into_response()
}

#[derive(Deserialize)]
pub struct TagPayload {
    pub tag: String,
}

pub async fn add_contact_tag(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(contact_id): Path<String>,
    Json(payload): Json<TagPayload>,
) -> impl IntoResponse {
    // A failed existence read must not surface as "Contact not found".
    let contact_exists: bool = match sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM contacts WHERE id = $1",
    )
    .bind(&contact_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(count) => count > 0,
        Err(err) => return db_error_response("could not check whether the contact exists", err),
    };

    if !contact_exists {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))).into_response();
    }

    let clean_tag = payload.tag.trim().trim_start_matches('#').to_string();
    if clean_tag.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Tag cannot be empty" }))).into_response();
    }

    if let Err(err) = attach_tag(&state.db, &contact_id, &clean_tag, &tenant_key).await {
        return db_error_response("could not apply tag", err);
    }

    let tags: Vec<String> = match sqlx::query_scalar::<_, String>(
        r#"
        SELECT t.name FROM contact_tags ct JOIN tags t ON ct.tag_id = t.id
        WHERE ct.contact_id = $1 ORDER BY t.name ASC
        "#,
    )
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(tags) => tags,
        // The tag is attached; saying so truthfully needs the read-back to work.
        Err(err) => return db_error_response("tag was attached but the tag list could not be read back", err),
    };

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "contactId": contact_id, "tag": clean_tag, "tags": tags })),
    )
        .into_response()
}

pub async fn remove_contact_tag(
    State(state): State<AppState>,
    Path((contact_id, tag)): Path<(String, String)>,
) -> impl IntoResponse {
    let clean_tag = tag.trim().trim_start_matches('#');
    let result = sqlx::query(
        "DELETE FROM contact_tags WHERE contact_id = $1 AND tag_id IN (SELECT id FROM tags WHERE name = $2)",
    )
    .bind(&contact_id)
    .bind(clean_tag)
    .execute(&state.db)
    .await;

    let removed = match result {
        Ok(report) => report.rows_affected(),
        Err(err) => return db_error_response("could not remove tag", err),
    };

    if removed == 0 {
        let contact_exists = match sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM contacts WHERE id = $1",
        )
        .bind(&contact_id)
        .fetch_one(&state.db)
        .await
        {
            Ok(count) => count > 0,
            Err(err) => return db_error_response("could not check whether the contact exists", err),
        };
        if !contact_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Contact not found", "contactId": contact_id })),
            )
                .into_response();
        }
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("Contact {contact_id} does not carry the tag '{clean_tag}'"),
                "contactId": contact_id,
                "tag": clean_tag
            })),
        )
            .into_response();
    }

    let tags: Vec<String> = match sqlx::query_scalar::<_, String>(
        r#"
        SELECT t.name FROM contact_tags ct JOIN tags t ON ct.tag_id = t.id
        WHERE ct.contact_id = $1 ORDER BY t.name ASC
        "#,
    )
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(tags) => tags,
        Err(err) => return db_error_response("tag was removed but the tag list could not be read back", err),
    };

    Json(json!({ "status": "ok", "removed": true, "contactId": contact_id, "tag": clean_tag, "tags": tags }))
        .into_response()
}

pub async fn toggle_contact_consent(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // `Err` here must not become 404: a database failure is not "no such
    // contact", and the handler writes consent right after this check.
    let contact_tenant: Option<String> = match sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(tenant_key, '') FROM contacts WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(owner) => owner,
        Err(err) => return db_error_response("could not read the owner of the contact", err),
    };

    match contact_tenant {
        Some(owner) if owner == tenant_key => {}
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Contact not found", "contactId": id })),
            )
                .into_response()
        }
    }

    if !table_exists(&state.db, "consent_records").await {
        return missing_dependency_response(
            "consent_records table",
            "run CREATE TABLE IF NOT EXISTS consent_records (see db/mod.rs) before toggling consent",
        );
    }

    let now = Utc::now().to_rfc3339();
    // This read decides which way the toggle goes, so a failure cannot be
    // silently treated as "no consent recorded" (that would opt people in).
    let current_status: Option<String> = match sqlx::query_scalar::<_, String>(
        "SELECT status FROM consent_records WHERE contact_id = $1 ORDER BY recorded_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(status) => status,
        Err(err) => return db_error_response("could not read the current consent status", err),
    };

    let is_currently_opted_in =
        current_status.as_deref().map(|s| s.to_lowercase()) == Some("opted_in".to_string());
    let new_db_status = if is_currently_opted_in { "opted_out" } else { "opted_in" };

    // Append-only: a consent ledger that overwrites its own history cannot
    // answer "when did this person ask us to stop?".
    let inserted = sqlx::query(
        "INSERT INTO consent_records (id, contact_id, status, source, note, recorded_at) VALUES ($1, $2, $3, 'admin_profile_action', 'Toggled via Contact Profile UI', $4)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&id)
    .bind(new_db_status)
    .bind(&now)
    .execute(&state.db)
    .await;

    let report = match inserted {
        Ok(report) => report,
        Err(err) => return db_error_response("could not record consent", err),
    };
    if report.rows_affected() == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "the consent INSERT reported no rows written; consent was not changed",
                "contactId": id,
                "databaseError": true
            })),
        )
            .into_response();
    }

    Json(json!({
        "status": "success",
        "contactId": id,
        "optInStatus": new_db_status,
        "optInTimestamp": now
    }))
    .into_response()
}

pub async fn sync_contact_meta(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct ContactInfo {
        channel: String,
        value: String,
        custom_fields: Option<String>,
    }

    let contact = sqlx::query_as::<_, ContactInfo>(
        "SELECT channel, value, custom_fields FROM contacts WHERE id = $1 AND COALESCE(tenant_key, '') = $2",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await;

    let c = match contact {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Contact not found", "contactId": id })),
            )
                .into_response()
        }
        Err(err) => return db_error_response("could not read contact for sync", err),
    };

    // A per-contact Meta profile lookup needs the phone_number_id + access token
    // the inboxes/integration_settings rows carry. Without them there is not
    // even a local refresh to attribute to a provider connection.
    let (access_token, phone_number_id) =
        crate::services::meta_sync::resolve_meta_credentials(&state).await;
    if access_token.is_empty() || phone_number_id.is_empty() {
        return missing_dependency_response(
            "Meta Cloud API credentials",
            "no meta_access_token / meta_phone_number_id configured (env META_WHATSAPP_ACCESS_TOKEN, META_PHONE_NUMBER_ID, or the whatsapp inbox / integration_settings row); no provider lookup can run, and nothing is written",
        );
    }

    let mut fields = c
        .custom_fields
        .as_deref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let now = Utc::now().to_rfc3339();
    // No graph.facebook.com call runs in this handler, so it must not stamp
    // `meta_synced_at` (which the profile publishes as Meta's sync time). The
    // key below names the event that really happened: a local identity refresh.
    fields.insert("local_identity_refresh_at".to_string(), json!(now));
    if c.channel == "whatsapp" {
        fields.insert("wa_id".to_string(), json!(c.value.replace('+', "")));
    }
    fields.insert("synced_channel".to_string(), json!(c.channel));

    let updated = sqlx::query("UPDATE contacts SET custom_fields = $1, updated_at = $2 WHERE id = $3")
        .bind(serde_json::Value::Object(fields).to_string())
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    let report = match updated {
        Ok(report) => report,
        Err(err) => return db_error_response("could not persist the refreshed identity fields", err),
    };
    if report.rows_affected() == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "the contact row disappeared before the refresh could be saved; nothing was written",
                "contactId": id
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            // Null: no provider sync has run. Kept in the body so the profile
            // shows "never synced" instead of a timestamp we invented.
            "syncedAt": serde_json::Value::Null,
            "identityRefreshedAt": now,
            "contactId": id,
            "channel": c.channel,
            "value": c.value,
            "note": "identity fields refreshed from the stored contact; no Meta call was made and the provider quality rating requires the Meta display-profile endpoint, which is not ported"
        })),
    )
        .into_response()
}

pub async fn sync_meta_contacts(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
) -> impl IntoResponse {
    // The legacy route paged Meta's contact list end to end. services::meta_sync
    // only resolves credentials and refreshes the business profile today, so
    // there is nothing real to write here and inventing a count would be a fake.
    let (access_token, phone_number_id) =
        crate::services::meta_sync::resolve_meta_credentials(&state).await;
    // A failed count must not be reported as "0 WhatsApp contacts"; the number
    // below is quoted back to the operator as fact.
    let whatsapp_contacts: i64 = match sqlx::query_scalar(
        "SELECT count(*) FROM contacts WHERE channel = 'whatsapp' AND COALESCE(tenant_key, '') = $1",
    )
    .bind(&tenant_key)
    .fetch_one(&state.db)
    .await
    {
        Ok(count) => count,
        Err(err) => return db_error_response("could not count the WhatsApp contacts", err),
    };

    if access_token.is_empty() || phone_number_id.is_empty() {
        return missing_dependency_response(
            "Meta Cloud API contact-list sync",
            "no Meta credentials configured, and routes::contacts has no graph.facebook.com contacts page-fetch implementation; refusing to report contacts as synchronized",
        );
    }

    missing_dependency_response(
        "Meta Cloud API contact-list sync service",
        &format!(
            "credentials are configured and {whatsapp_contacts} WhatsApp contacts exist for tenant '{tenant_key}', but the paginated Meta contacts endpoint has not been ported to services::meta_sync; refusing to report a synchronisation that did not run"
        ),
    )
}

fn inbound_error_detail(err: &inbound::InboundError) -> String {
    match err {
        inbound::InboundError::UnknownContact(msg) => msg.clone(),
        inbound::InboundError::Db(e) => e.to_string(),
    }
}

pub async fn get_or_create_contact_conversation(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    #[derive(sqlx::FromRow)]
    struct ContactRow {
        channel: String,
        tenant_key: Option<String>,
    }

    let contact = match sqlx::query_as::<_, ContactRow>(
        "SELECT channel, tenant_key FROM contacts WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Contact {id} does not exist") })),
            )
                .into_response()
        }
        Err(err) => return db_error_response("could not read contact", err),
    };

    if contact.tenant_key.as_deref().unwrap_or(&tenant_key) != tenant_key.as_str() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("Contact {id} belongs to tenant '{}', not '{tenant_key}'", contact.tenant_key.clone().unwrap_or_default())
            })),
        )
            .into_response();
    }

    // `get_or_create_inbox_conversation` owns the reopen semantics: an existing
    // thread is returned unless it was already won/lost, and an opted-out
    // contact is not silently re-engaged.
    let conversation_id =
        match inbound::get_or_create_inbox_conversation(&state.db, &id, &contact.channel, None, None)
            .await
        {
            Ok(cid) => cid,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("could not open a conversation: {}", inbound_error_detail(&err)) })),
                )
                    .into_response()
            }
        };

    let state_value: Option<String> =
        match sqlx::query_scalar::<_, String>("SELECT state FROM conversations WHERE id = $1")
            .bind(&conversation_id)
            .fetch_optional(&state.db)
            .await
        {
            Ok(s) => s,
            Err(err) => return db_error_response("conversation created but could not be read back", err),
        };

    (
        StatusCode::OK,
        Json(json!({
            "id": conversation_id,
            // Null means the conversation we were handed is not in
            // `conversations`; "unknown" would look like a real lifecycle state.
            "state": state_value.map(|s| s.to_lowercase())
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct CreateContactPayload {
    pub value: String,
    pub channel: String,
    #[serde(default, alias = "personName", alias = "name")]
    pub person_name: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default, alias = "customFields")]
    pub custom_fields: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default, alias = "customerId")]
    pub customer_id: Option<String>,
}

/// POST /contacts — the write path the directory UI and the CRM use to add a
/// channel identity by hand. Provisioning is delegated to
/// `services::inbound::get_or_create_contact` so the lead row, the
/// `(channel, value)` dedupe and the consent record are exactly the same ones
/// inbound traffic gets, never a second, cheaper copy.
pub async fn create_contact(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(payload): Json<CreateContactPayload>,
) -> Response {
    let channel = payload.channel.trim().to_lowercase();
    if payload.value.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "`value` is required: the channel identity (E.164 number, email address or platform id) of the contact"
            })),
        )
            .into_response();
    }
    if channel.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "`channel` is required",
                "supportedChannels": SUPPORTED_CHANNELS
            })),
        )
            .into_response();
    }
    if !is_supported_channel(&channel) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("unsupported channel '{channel}'"),
                "supportedChannels": SUPPORTED_CHANNELS
            })),
        )
            .into_response();
    }

    let value = if is_phone_channel(&channel) {
        match normalize_e164(&payload.value) {
            Some(v) => v,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": format!("'{}' is not a valid E.164 number for the {channel} channel (7-15 digits, country code first)", payload.value.trim())
                    })),
                )
                    .into_response()
            }
        }
    } else if channel == "email" {
        if !looks_like_email(&payload.value) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("'{}' is not a valid email address", payload.value.trim()) })),
            )
                .into_response();
        }
        payload.value.trim().to_lowercase()
    } else {
        payload.value.trim().to_string()
    };

    let already_present = existing_contact_id(&state.db, &channel, &value)
        .await
        .map(|r| r.is_some());

    let mut warnings: Vec<String> = Vec::new();
    match already_present {
        Ok(true) => warnings.push(format!(
            "a contact with channel '{channel}' and value '{value}' already existed; its row was reused, not duplicated"
        )),
        Ok(false) => {}
        Err(err) => return db_error_response("could not check for an existing contact", err),
    }

    let name = payload.person_name.unwrap_or_default();
    let contact_id = match inbound::get_or_create_contact(&state.db, &channel, &value, &name, &tenant_key)
        .await
    {
        Ok(id) => id,
        Err(inbound::InboundError::Db(err)) => {
            return db_error_response("could not create the contact", err)
        }
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": inbound_error_detail(&err) })),
            )
                .into_response()
        }
    };

    // `get_or_create_contact` dedupes on (channel, value) across the whole table,
    // so a reused id may belong to another tenant. Read the stored row back before
    // any write: it is the only honest source for `created_at` / `person_name`
    // (provisioning stamps its own clock and may keep a fallback or pre-existing
    // name), and it lets us refuse to mutate a foreign tenant's contact.
    let (read_projection, read_missing) = contact_projection(&state.db).await;
    let read_sql = format!("SELECT {} FROM contacts WHERE id = $1", read_projection.join(", "));
    let stored = match sqlx::query(&read_sql).bind(&contact_id).fetch_optional(&state.db).await {
        Ok(Some(row)) => LoadedContact::from_row(&row),
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "the provisioned contact could not be read back; nothing was created for this tenant",
                    "contactId": contact_id,
                    "databaseError": true
                })),
            )
                .into_response()
        }
        Err(err) => return db_error_response("the provisioned contact could not be read back", err),
    };
    if stored.tenant_key.as_deref().map(|owner| owner != tenant_key) == Some(true) {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": format!(
                    "a contact for channel '{channel}' value '{value}' already exists and belongs to another tenant; nothing was created or modified for tenant '{tenant_key}'"
                ),
                "contactId": contact_id,
                "alreadyPresent": true
            })),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    // Only a language the caller actually stated is stored; defaulting to "en"
    // would put an invented preference on a real contact record (and overwrite
    // the stored one when the identity already existed).
    let language = payload.language.filter(|l| !l.trim().is_empty());

    let mut fields = payload.custom_fields.unwrap_or_default();
    if let Some(language) = language.as_deref() {
        fields.insert("language".to_string(), json!(language));
    }
    fields.entry("source".to_string()).or_insert(json!("manual_create"));

    if column_exists(&state.db, "contacts", "custom_fields").await {
        let serialized_fields = match serde_json::to_string(&fields) {
            Ok(text) => text,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("the custom fields could not be serialized: {err}") })),
                )
                    .into_response()
            }
        };
        let sql = if language.is_some() {
            "UPDATE contacts SET custom_fields = $1, language = $2, updated_at = $3 WHERE id = $4"
        } else {
            "UPDATE contacts SET custom_fields = $1, updated_at = $2 WHERE id = $3"
        };
        let mut query = sqlx::query(sql).bind(serialized_fields);
        if let Some(language) = language.as_deref() {
            query = query.bind(language);
        }
        let saved = query.bind(&now).bind(&contact_id).execute(&state.db).await;
        match saved {
            Ok(report) if report.rows_affected() == 0 => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "the contact row could not be updated after creation; its custom fields were not saved",
                        "contactId": contact_id
                    })),
                )
                    .into_response()
            }
            Ok(_) => {}
            Err(err) => {
                return db_error_response(
                    "contact row exists but its language/custom fields could not be saved",
                    err,
                )
            }
        }
    } else if let Some(language) = language.as_deref() {
        let saved =
            sqlx::query("UPDATE contacts SET language = $1, updated_at = $2 WHERE id = $3")
                .bind(language)
                .bind(&now)
                .bind(&contact_id)
                .execute(&state.db)
                .await;
        match saved {
            Ok(report) if report.rows_affected() == 0 => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "the contact row could not be updated after creation; its language was not saved",
                        "contactId": contact_id
                    })),
                )
                    .into_response()
            }
            Ok(_) => {}
            Err(err) => {
                return db_error_response("contact row exists but its language could not be saved", err)
            }
        }
    } else {
        warnings.push(
            "no language was supplied and contacts.custom_fields is absent, so nothing beyond the contact row itself was written"
                .to_string(),
        );
    }

    let mut applied_tags: Vec<String> = Vec::new();
    for tag in payload.tags.unwrap_or_default() {
        match attach_tag(&state.db, &contact_id, &tag, &tenant_key).await {
            Ok(_) => applied_tags.push(tag.trim().to_string()),
            Err(err) => warnings.push(format!("tag '{tag}' could not be attached: {err}")),
        }
    }

    if let Some(customer_id) = payload.customer_id.filter(|c| !c.trim().is_empty()) {
        match link_customer_row(&state.db, &contact_id, &customer_id, &tenant_key).await {
            Ok(LinkOutcome::Linked(AuditWrite::NotLogged(reason))) => warnings.push(format!(
                "contact was linked to customer '{customer_id}' but the audit row was not written: {reason}"
            )),
            Ok(LinkOutcome::Linked(AuditWrite::Logged)) => {}
            Ok(LinkOutcome::AlreadyLinked) => warnings.push(format!(
                "contact was already linked to customer '{customer_id}'; no second link row was written"
            )),
            Ok(LinkOutcome::NoSchema) => warnings.push(format!(
                "customer_id '{customer_id}' was not stored: the customers table / contacts.customer_id column is not wired yet"
            )),
            Err(err) => return err,
        }
    }

    // The reply claims only what is actually on the row. custom_fields is echoed
    // solely when the column exists (and the UPDATE above persisted `fields`);
    // otherwise those keys were never written, so null + a warning is reported.
    let custom_fields_persisted = !read_missing.iter().any(|c| *c == "custom_fields");
    if !custom_fields_persisted {
        warnings.push(
            "contacts.custom_fields is not part of this schema; the submitted custom fields were not persisted"
                .to_string(),
        );
    }
    let response_custom_fields = if custom_fields_persisted {
        json!(fields)
    } else {
        serde_json::Value::Null
    };

    (
        StatusCode::CREATED,
        Json(json!({
            "id": contact_id,
            "value": stored.value,
            "channel": stored.channel,
            "personName": stored.person_name,
            "language": language.or_else(|| stored.language.clone()),
            "tags": applied_tags,
            "customFields": response_custom_fields,
            "tenantKey": tenant_key,
            "createdAt": stored.created_at,
            "warnings": warnings
        })),
    )
        .into_response()
}

/// Returns `None` when the contact does not exist, and the owning tenant key so
/// callers can refuse cross-tenant access the way the legacy routes did.
async fn contact_owner(
    db: &DbPool,
    contact_id: &str,
) -> Result<Option<(String, Option<String>)>, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT channel, tenant_key FROM contacts WHERE id = $1",
        )
        .bind(contact_id)
        .fetch_optional(db)
        .await?,
    )
}

pub async fn list_contact_memories(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(contact_id): Path<String>,
) -> Response {
    #[derive(sqlx::FromRow)]
    struct MemoryRow {
        id: String,
        contact_id: String,
        fact: String,
        category: Option<String>,
        confidence: Option<String>,
        noted_at: Option<String>,
        created_at: Option<String>,
    }

    if let Some((_, owner)) = match contact_owner(&state.db, &contact_id).await {
        Ok(row) => row,
        Err(err) => return db_error_response("could not read contact", err),
    } {
        if owner.as_deref().unwrap_or(&tenant_key) != tenant_key.as_str() {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Contact {contact_id} does not belong to tenant '{tenant_key}'") })),
            )
                .into_response();
        }
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Contact {contact_id} does not exist") })),
        )
            .into_response();
    }

    if !table_exists(&state.db, "contact_memories").await {
        return missing_dependency_response(
            "contact_memories table",
            "remembered facts are read from `contact_memories`, which backend-rs/src/db/mod.rs does not create yet; returning [] would look like a contact with no history rather than a schema gap",
        );
    }

    let memories = match sqlx::query_as::<_, MemoryRow>(
        r#"SELECT id, contact_id, fact, category, confidence, noted_at, created_at
           FROM contact_memories
           WHERE contact_id = $1
           ORDER BY COALESCE(noted_at, created_at) DESC"#,
    )
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error_response("could not read contact memories", err),
    };

    let mut warnings = Vec::new();
    let list: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            let conf: Option<f64> = m.confidence.as_deref().and_then(|c| c.parse().ok());
            if conf.is_none() {
                warnings.push(format!("memory {} has no parseable confidence value", m.id));
            }
            json!({
                "id": m.id,
                "contactId": m.contact_id,
                "fact": m.fact,
                "category": m.category,
                "confidence": conf,
                "notedAt": m.noted_at.or(m.created_at.clone()),
                "createdAt": m.created_at,
                "source": "contact_memory"
            })
        })
        .collect();

    // Array body kept for frontend compatibility; provenance problems travel in
    // the `x-warnings` header instead of being hidden inside the payload.
    let mut response = Json(list).into_response();
    if !warnings.is_empty() {
        if let Ok(header) = HeaderValue::from_str(&warnings.join("; ")) {
            response.headers_mut().insert("x-warnings", header);
        }
    }
    response
}

#[derive(Deserialize)]
pub struct CreateMemoryPayload {
    pub fact: String,
    pub category: Option<String>,
}

pub async fn create_contact_memory(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(contact_id): Path<String>,
    Json(payload): Json<CreateMemoryPayload>,
) -> Response {
    if payload.fact.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "`fact` is required and must not be empty" })),
        )
            .into_response();
    }

    match contact_owner(&state.db, &contact_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Contact {contact_id} does not exist") })),
            )
                .into_response()
        }
        Err(err) => return db_error_response("could not read contact", err),
    }

    if !table_exists(&state.db, "contact_memories").await {
        return missing_dependency_response(
            "contact_memories table",
            "there is nowhere to persist a remembered fact until the DDL in WIRING NEEDED is applied; nothing was written",
        );
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let category = payload
        .category
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| "preference".to_string());

    if let Err(err) = sqlx::query(
        "INSERT INTO contact_memories (id, contact_id, fact, category, confidence, noted_at, created_at) VALUES ($1, $2, $3, $4, '1.0', $5, $6)",
    )
    .bind(&id)
    .bind(&contact_id)
    .bind(payload.fact.trim())
    .bind(&category)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        return db_error_response("could not store the memory", err);
    }

    let audit = write_audit(
        &state.db,
        "contact_memory",
        &id,
        "created",
        &tenant_key,
        &format!("Manual memory added for contact {contact_id}"),
    )
    .await;

    (
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "contactId": contact_id,
            "fact": payload.fact.trim(),
            "category": category,
            "confidence": 1.0,
            "notedAt": now,
            "createdAt": now,
            "source": "contact_memory",
            "audit": audit.value()
        })),
    )
        .into_response()
}

pub async fn delete_contact_memory(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path((contact_id, memory_id)): Path<(String, String)>,
) -> Response {
    if !table_exists(&state.db, "contact_memories").await {
        return missing_dependency_response(
            "contact_memories table",
            "no memory can be deleted because the table does not exist in this database",
        );
    }

    let result = match sqlx::query("DELETE FROM contact_memories WHERE id = $1 AND contact_id = $2")
        .bind(&memory_id)
        .bind(&contact_id)
        .execute(&state.db)
        .await
    {
        Ok(r) => r,
        Err(err) => return db_error_response("could not delete the memory", err),
    };

    if result.rows_affected() == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("memory {memory_id} is not attached to contact {contact_id}")
            })),
        )
            .into_response();
    }

    let audit = write_audit(
        &state.db,
        "contact_memory",
        &memory_id,
        "deleted",
        &tenant_key,
        &format!("Memory {memory_id} removed from contact {contact_id}"),
    )
    .await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "deleted",
            "id": memory_id,
            "contactId": contact_id,
            "audit": audit.value()
        })),
    )
        .into_response()
}

/// Result of an audit-log write, reported to the caller instead of dropped.
/// `NotLogged` always carries the reason, so a response can never imply that
/// provenance exists when it does not.
#[derive(Debug, Clone, PartialEq)]
enum AuditWrite {
    Logged,
    NotLogged(String),
}

impl AuditWrite {
    fn value(&self) -> serde_json::Value {
        match self {
            AuditWrite::Logged => json!({ "logged": true }),
            AuditWrite::NotLogged(reason) => json!({ "logged": false, "reason": reason }),
        }
    }
}

/// A no-op write with an explicit reason (e.g. there was nothing to record).
fn audit_not_needed(reason: &str) -> AuditWrite {
    AuditWrite::NotLogged(reason.to_string())
}

/// The audit table is part of the shipped schema, but a failure to write it
/// must not roll back a user-visible action that already succeeded — so callers
/// get their result plus an honest `audit` field rather than a 500 for a
/// bookkeeping table.
///
/// `actor` stays unattributed: these handlers authenticate with `X-Tenant-Key`
/// only, so there is no verified person to record. Attributing this to "admin"
/// would invent an identity in the compliance trail.
async fn write_audit(
    db: &DbPool,
    entity_type: &str,
    entity_id: &str,
    action: &str,
    tenant_key: &str,
    description: &str,
) -> AuditWrite {
    if !table_exists(db, "audit_logs").await {
        return AuditWrite::NotLogged(
            "the audit_logs table does not exist in this database, so no provenance row was written"
                .to_string(),
        );
    }
    match sqlx::query(
        r#"INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, actor_name, severity, description, details, created_at)
           VALUES ($1, $2, $3, $4, 'unattributed_api_call', NULL, 'info', $5, $6)"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(description)
    .bind(json!({ "tenantKey": tenant_key }).to_string())
    .bind(Utc::now().to_rfc3339())
    .execute(db)
    .await
    {
        Ok(result) if result.rows_affected() == 0 => AuditWrite::NotLogged(
            "the audit_logs INSERT reported success but affected 0 rows, so no row was stored"
                .to_string(),
        ),
        Ok(_) => AuditWrite::Logged,
        Err(err) => {
            AuditWrite::NotLogged(format!("the audit_logs INSERT failed: {err}"))
        }
    }
}

pub async fn export_contacts(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(params): Query<ContactQueryParams>,
) -> Response {
    let (projection, missing) = contact_projection(&state.db).await;
    let scoped = projection.contains(&"tenant_key");
    let sql = format!(
        "SELECT {} FROM contacts {} ORDER BY created_at DESC",
        projection.join(", "),
        if scoped {
            "WHERE COALESCE(tenant_key, '') = $1"
        } else {
            ""
        }
    );

    let mut query = sqlx::query(&sql);
    if scoped {
        query = query.bind(&tenant_key);
    }
    let rows = match query.fetch_all(&state.db).await {
        Ok(rows) => rows,
        Err(err) => return db_error_response("could not read contacts for export", err),
    };

    let mut csv = String::from("Name,Phone/Value,Channel,Role,Language,Tags\n");
    let mut exported = 0usize;
    for row in rows {
        let c = LoadedContact::from_row(&row);

        let tags: Vec<String> = match sqlx::query_scalar::<_, String>(
            r#"
            SELECT t.name 
            FROM contact_tags ct 
            JOIN tags t ON ct.tag_id = t.id 
            WHERE ct.contact_id = $1
            ORDER BY t.name ASC
            "#,
        )
        .bind(&c.id)
        .fetch_all(&state.db)
        .await
        {
            Ok(tags) => tags,
            Err(err) => return db_error_response("could not read tags for export", err),
        };

        // Filter by segment if requested
        if let Some(ref seg_id) = params.segment_id {
            if seg_id != "all" {
                match contact_matches_segment(&state.db, seg_id, &c.channel, &tags).await {
                    Err(err) => return segment_lookup_failed_response(err),
                    Ok(None) => return unknown_segment_response(seg_id),
                    Ok(Some(false)) => continue,
                    Ok(Some(true)) => {}
                }
            }
        }
        // Filter by tag if requested
        if let Some(ref t) = params.tag {
            if t != "all" && !tags.contains(t) {
                continue;
            }
        }
        // Filter by channel if requested
        if let Some(ref ch) = params.channel {
            if ch != "all" && c.channel != *ch {
                continue;
            }
        }
        // Filter by search query if requested
        if let Some(ref q) = params.search {
            let q_lower = q.to_lowercase();
            let name_match = c.person_name.as_deref().unwrap_or("").to_lowercase().contains(&q_lower);
            let val_match = c.value.to_lowercase().contains(&q_lower);
            if !name_match && !val_match {
                continue;
            }
        }

        let quote = |v: String| format!("\"{}\"", v.replace('"', "\"\""));
        // A column the schema does not have exports as empty rather than as a
        // made-up "en"/"agent", so a downloaded sheet cannot claim data that
        // was never stored.
        let language = if missing.contains(&"language") {
            String::new()
        } else {
            c.language.clone().unwrap_or_default()
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            quote(c.person_name.clone().unwrap_or_default()),
            quote(c.value.clone()),
            quote(c.channel.clone()),
            quote(c.role.clone().unwrap_or_default()),
            quote(language),
            quote(tags.join("; "))
        ));
        exported += 1;
    }

    if missing.iter().any(|c| *c == "tenant_key") {
        // Refuse rather than export every tenant's contacts into one file.
        return missing_dependency_response(
            "contacts.tenant_key",
            "the export cannot be scoped to an organization because the tenant_key column is absent; nothing was exported",
        );
    }

    let filename = format!("contacts_export_{tenant_key}.csv").to_lowercase();
    let mut res = Response::new(axum::body::Body::from(csv));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        res.headers_mut().insert(header::CONTENT_DISPOSITION, value);
    }
    if let Ok(value) = HeaderValue::from_str(&format!("rows={exported}; missing_columns={}", missing.join("|"))) {
        res.headers_mut().insert("x-warnings", value);
    }
    res
}

// ---------------------------------------------------------------------------
// Multi-channel CSV import
// ---------------------------------------------------------------------------

fn bad_multipart<E: std::fmt::Display>(err: &E) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": format!("the multipart upload could not be read: {err}"),
            "hint": "send a form field named 'file' holding the CSV, optionally 'default_channel' and 'default_tags'"
        })),
    )
        .into_response()
}

#[derive(Debug)]
pub struct ImportForm {
    pub filename: Option<String>,
    pub csv_text: String,
    pub default_channel: Option<String>,
    pub default_tags: Vec<String>,
    pub unknown_fields: Vec<String>,
}

/// Reads the upload field by field. The bytes really come off the wire here —
/// nothing is sampled, stubbed or assumed.
async fn read_import_form(multipart: &mut Multipart) -> Result<ImportForm, Response> {
    let mut raw: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut default_channel: Option<String> = None;
    let mut default_tags: Vec<String> = Vec::new();
    let mut unknown_fields: Vec<String> = Vec::new();

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(err) => return Err(bad_multipart(&err)),
        };
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" | "csv" | "contacts" => {
                filename = field.file_name().map(|value| value.to_string());
                let bytes = match field.bytes().await {
                    Ok(bytes) => bytes,
                    Err(err) => return Err(bad_multipart(&err)),
                };
                raw = Some(bytes.to_vec());
            }
            "default_channel" => {
                let text = match field.text().await {
                    Ok(text) => text,
                    Err(err) => return Err(bad_multipart(&err)),
                };
                let trimmed = text.trim().to_lowercase();
                if !trimmed.is_empty() {
                    default_channel = Some(trimmed);
                }
            }
            "default_tags" => {
                let text = match field.text().await {
                    Ok(text) => text,
                    Err(err) => return Err(bad_multipart(&err)),
                };
                default_tags.extend(split_tags(&text));
            }
            other => {
                // Unknown fields are drained so the stream stays parseable, but a
                // part whose bytes fail to read is a transport error, not an empty
                // part we may dismiss as "ignored".
                if let Err(err) = field.bytes().await {
                    return Err(bad_multipart(&err));
                }
                unknown_fields.push(other.to_string());
            }
        }
    }

    let Some(bytes) = raw else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "multipart field 'file' containing the CSV is required",
                "fields": ["file", "default_channel", "default_tags"]
            })),
        )
            .into_response());
    };
    if bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "the uploaded file has no bytes in it" })),
        )
            .into_response());
    }
    let csv_text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "the uploaded file is not UTF-8 text; re-save it as a UTF-8 CSV"
                })),
            )
                .into_response())
        }
    };

    Ok(ImportForm {
        filename,
        csv_text,
        default_channel,
        default_tags,
        unknown_fields,
    })
}

/// `(channel, value)` -> `(contact_id, tenant_key)` for every row already in the
/// directory. `ix_contacts_channel_value` is a plain index, not a UNIQUE one,
/// so the application, not the database, owns the dedupe.
async fn contact_directory(
    db: &DbPool,
) -> Result<HashMap<(String, String), (String, Option<String>)>, sqlx::Error> {
    let has_tenant = column_exists(db, "contacts", "tenant_key").await;
    let sql = if has_tenant {
        "SELECT id, channel, value, tenant_key FROM contacts"
    } else {
        "SELECT id, channel, value FROM contacts"
    };
    let rows = sqlx::query(sql).fetch_all(db).await?;
    let mut map = HashMap::new();
    for row in rows {
        // Propagate instead of defaulting: a blank id entering this map would be
        // handed back as an "existing contact" and every update against it would
        // match zero rows while still being counted as a success.
        let id: String = row.try_get("id")?;
        let channel: String = row.try_get("channel")?;
        let value: String = row.try_get("value")?;
        if id.trim().is_empty() {
            return Err(sqlx::Error::Decode(
                "contacts row has an empty primary key".into(),
            ));
        }
        let tenant: Option<String> = if has_tenant {
            row.try_get("tenant_key").ok().flatten()
        } else {
            None
        };
        map.insert((channel.to_lowercase(), value), (id, tenant));
    }
    Ok(map)
}

#[derive(Debug)]
pub struct AnalyzedRow {
    pub row_number: usize,
    pub channel: Option<String>,
    pub value: Option<String>,
    pub name: String,
    /// None when the CSV had no language column for this row.
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Map<String, serde_json::Value>,
    pub errors: Vec<String>,
    pub existing_contact_id: Option<String>,
    pub duplicate_of_row: Option<usize>,
}

impl AnalyzedRow {
    fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.channel.is_some() && self.value.is_some()
    }
}

/// Pure validation pass shared by the preview and the real import: every row is
/// checked against the resolved channel vocabulary and against the rows already
/// in the database, so the preview cannot promise an import that would fail.
fn analyze_rows(
    records: &[CsvRecord],
    default_channel: Option<&str>,
    directory: &HashMap<(String, String), (String, Option<String>)>,
) -> Vec<AnalyzedRow> {
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    let mut analyzed = Vec::with_capacity(records.len());

    for record in records {
        let resolved = resolve_csv_row(record, default_channel);
        let mut errors = resolved.errors;
        let mut duplicate_of_row = None;
        let mut existing_contact_id = None;

        if let (Some(channel), Some(value)) = (resolved.channel.clone(), resolved.value.clone()) {
            let key = (channel.to_lowercase(), value.clone());
            match seen.get(&key) {
                Some(first_row) => {
                    duplicate_of_row = Some(*first_row);
                    errors.push(format!(
                        "same identity as CSV row {first_row}; the directory keys contacts on (channel, value)"
                    ));
                }
                None => {
                    seen.insert(key.clone(), record.row_number);
                    if let Some((id, _owner)) = directory.get(&key) {
                        existing_contact_id = Some(id.clone());
                    }
                }
            }
        }

        analyzed.push(AnalyzedRow {
            row_number: record.row_number,
            channel: resolved.channel,
            value: resolved.value,
            name: resolved.name,
            language: resolved.language,
            tags: resolved.tags,
            custom_fields: resolved.custom_fields,
            errors,
            existing_contact_id,
            duplicate_of_row,
        });
    }

    analyzed
}

/// Writes the attributes a CSV row carries onto an existing or freshly created
/// contact. Anything the current schema cannot hold is returned as a warning
/// naming the row and the column instead of being dropped in silence.
async fn write_row_attributes(
    db: &DbPool,
    contact_id: &str,
    row: &AnalyzedRow,
    missing: &[&'static str],
    fill_empty_name: bool,
    now: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let mut warnings = Vec::new();
    let has_language = !missing.iter().any(|c| *c == "language");
    let has_custom_fields = !missing.iter().any(|c| *c == "custom_fields");

    // Legacy rule kept: a stored display name is only ever filled in, never
    // overwritten by whatever the next file happens to claim.
    if fill_empty_name && !row.name.trim().is_empty() {
        sqlx::query(
            "UPDATE contacts SET person_name = $1, updated_at = $2 WHERE id = $3 AND (person_name IS NULL OR TRIM(person_name) = '')",
        )
        .bind(row.name.trim())
        .bind(now)
        .bind(contact_id)
        .execute(db)
        .await?;
    }

    // Only a language the CSV really carried is written: an absent column must
    // not stamp "en" onto a contact, and must not wipe a stored language either.
    match (row.language.as_deref(), has_language) {
        (Some(language), true) => {
            sqlx::query("UPDATE contacts SET language = $1, updated_at = $2 WHERE id = $3")
                .bind(language)
                .bind(now)
                .bind(contact_id)
                .execute(db)
                .await?;
        }
        (Some(language), false) => warnings.push(format!(
            "row {}: language '{language}' could not be stored because contacts.language is missing",
            row.row_number
        )),
        (None, _) => {}
    }

    if !row.custom_fields.is_empty() {
        if has_custom_fields {
            let current: Option<String> =
                sqlx::query_scalar("SELECT custom_fields FROM contacts WHERE id = $1")
                    .bind(contact_id)
                    .fetch_optional(db)
                    .await?;
            let mut merged = current
                .as_deref()
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                .and_then(|value| match value {
                    serde_json::Value::Object(map) => Some(map),
                    _ => None,
                })
                .unwrap_or_default();
            let mut changed = false;
            for (key, value) in &row.custom_fields {
                if !merged.contains_key(key) {
                    merged.insert(key.clone(), value.clone());
                    changed = true;
                }
            }
            if changed {
                match serde_json::to_string(&merged) {
                    Ok(serialized) => {
                        sqlx::query("UPDATE contacts SET custom_fields = $1, updated_at = $2 WHERE id = $3")
                            .bind(serialized)
                            .bind(now)
                            .bind(contact_id)
                            .execute(db)
                            .await?;
                    }
                    Err(err) => warnings.push(format!(
                        "row {}: merged custom fields could not be serialized ({err}); the stored value was left unchanged",
                        row.row_number
                    )),
                }
            }
        } else {
            warnings.push(format!(
                "row {}: extra CSV column(s) {} were not stored because contacts.custom_fields is missing",
                row.row_number,
                row.custom_fields.keys().cloned().collect::<Vec<String>>().join(", ")
            ));
        }
    }

    Ok(warnings)
}

pub async fn preview_import(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    let form = match read_import_form(&mut multipart).await {
        Ok(form) => form,
        Err(response) => return response,
    };
    let (headers, records) = parse_csv_table(&form.csv_text);
    if records.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "the CSV has a header row but no data rows",
                "detectedColumns": headers
            })),
        )
            .into_response();
    }

    let directory = match contact_directory(&state.db).await {
        Ok(map) => map,
        Err(err) => return db_error_response("could not load the existing contact directory", err),
    };

    let analyzed = analyze_rows(&records, form.default_channel.as_deref(), &directory);

    let mut detected_channels: Vec<String> = Vec::new();
    let mut invalid = 0usize;
    let mut would_update = 0usize;
    for row in &analyzed {
        if let Some(channel) = &row.channel {
            if !detected_channels.contains(channel) {
                detected_channels.push(channel.clone());
            }
        }
        if row.is_valid() {
            if row.existing_contact_id.is_some() {
                would_update += 1;
            }
        } else {
            invalid += 1;
        }
    }

    let row_errors: Vec<serde_json::Value> = analyzed
        .iter()
        .filter(|row| !row.errors.is_empty())
        .map(|row| {
            json!({
                "row": row.row_number,
                "value": row.value.clone().unwrap_or_default(),
                "channel": row.channel.clone(),
                "errors": row.errors,
                "existingContactId": row.existing_contact_id.clone(),
                "duplicateOfRow": row.duplicate_of_row
            })
        })
        .collect();

    let sample_rows: Vec<serde_json::Value> = analyzed
        .iter()
        .take(10)
        .map(|row| {
            json!({
                "row": row.row_number,
                "name": row.name,
                // Null channel = not resolved from the file; `valid` plus
                // `errors` carry why, so no stand-in bucket is needed here.
                "channel": row.channel.clone(),
                "value": row.value.clone().unwrap_or_default(),
                "tags": row.tags,
                "valid": row.is_valid(),
                "existing": row.existing_contact_id.is_some(),
                "errors": row.errors
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "totalRows": analyzed.len(),
            "detectedColumns": headers,
            "detectedChannels": detected_channels,
            "sampleRows": sample_rows,
            "rowErrors": row_errors,
            "invalidRows": invalid,
            "validRows": analyzed.len() - invalid,
            "existingContacts": would_update,
            "newContacts": analyzed.len() - invalid - would_update,
            "defaultChannel": form.default_channel,
            "defaultTags": form.default_tags,
            "unknownFields": form.unknown_fields,
            "filename": form.filename,
            "verdict": if invalid == 0 {
                format!("{} rows will import cleanly; {} of them match an existing (channel, value) identity and will be updated instead of duplicated", analyzed.len() - would_update, would_update)
            } else {
                format!("{invalid} of {} rows will be rejected; fix them before importing", analyzed.len())
            },
            "note": "the preview reads the real contact directory; nothing is written by this call"
        })),
    )
        .into_response()
}

pub async fn import_contacts(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    mut multipart: Multipart,
) -> Response {
    let form = match read_import_form(&mut multipart).await {
        Ok(form) => form,
        Err(response) => return response,
    };

    if let Some(channel) = &form.default_channel {
        if !is_supported_channel(channel) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("default_channel '{channel}' is not a channel this backend can store"),
                    "supportedChannels": SUPPORTED_CHANNELS
                })),
            )
                .into_response();
        }
    }

    let (headers, records) = parse_csv_table(&form.csv_text);
    if records.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "the CSV has a header row but no data rows; nothing was imported",
                "detectedColumns": headers
            })),
        )
            .into_response();
    }

    let directory = match contact_directory(&state.db).await {
        Ok(map) => map,
        Err(err) => return db_error_response("could not load the existing contact directory", err),
    };

    let analyzed = analyze_rows(&records, form.default_channel.as_deref(), &directory);
    let (_projection, missing) = contact_projection(&state.db).await;
    let mut warnings: Vec<String> = Vec::new();
    for column in ["language", "custom_fields", "customer_id"] {
        if missing.iter().any(|c| *c == column) {
            warnings.push(format!(
                "contacts.{column} is not in this schema (see WIRING NEEDED); values supplied for it were not persisted"
            ));
        }
    }
    if missing.iter().any(|c| *c == "tenant_key") {
        warnings.push("contacts.tenant_key is absent; new rows cannot be attributed to this tenant".to_string());
    }
    if !form.unknown_fields.is_empty() {
        warnings.push(format!(
            "ignored multipart field(s): {}",
            form.unknown_fields.join(", ")
        ));
    }

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut channel_counts: HashMap<String, usize> = HashMap::new();
    let mut row_errors: Vec<serde_json::Value> = Vec::new();
    let mut created_ids: Vec<String> = Vec::new();

    for row in &analyzed {
        if !row.is_valid() {
            skipped += 1;
            row_errors.push(json!({
                "row": row.row_number,
                "value": row.value.clone().unwrap_or_default(),
                "channel": row.channel.clone(),
                "errors": row.errors,
                "action": "skipped"
            }));
            continue;
        }

        let (Some(channel), Some(value)) = (row.channel.clone(), row.value.clone()) else {
            skipped += 1;
            continue;
        };
        *channel_counts.entry(channel.clone()).or_insert(0) += 1;

        let mut all_tags: Vec<String> = Vec::new();
        for tag in row.tags.iter().chain(form.default_tags.iter()) {
            let tag = tag.trim();
            if !tag.is_empty() && !all_tags.iter().any(|existing| existing == tag) {
                all_tags.push(tag.to_string());
            }
        }

        let now = Utc::now().to_rfc3339();
        let mut row_warnings: Vec<String> = Vec::new();
        let is_new = row.existing_contact_id.is_none();

        // Create or reuse on the same natural key the directory already uses.
        let contact_id = match row.existing_contact_id.clone() {
            Some(id) => id,
            None => match inbound::get_or_create_contact(
                &state.db,
                &channel,
                &value,
                &row.name,
                &tenant_key,
            )
            .await
            {
                Ok(id) => {
                    created += 1;
                    created_ids.push(id.clone());
                    id
                }
                Err(inbound::InboundError::Db(err)) => {
                    return db_error_response(
                        &format!("row {} could not be created after {} rows were already written", row.row_number, created),
                        err,
                    )
                }
                Err(err) => {
                    skipped += 1;
                    row_errors.push(json!({
                        "row": row.row_number,
                        "value": value,
                        "channel": channel,
                        "errors": [inbound_error_detail(&err)],
                        "action": "skipped"
                    }));
                    continue;
                }
            },
        };

        if !is_new {
            updated += 1;
        }

        // Name, language and any extra CSV columns, written for real where the
        // schema can hold them and reported as dropped where it cannot.
        match write_row_attributes(
            &state.db,
            &contact_id,
            row,
            &missing,
            !is_new,
            &now,
        )
        .await
        {
            Ok(row_warnings_now) => row_warnings.extend(row_warnings_now),
            Err(err) => {
                return db_error_response(
                    &format!(
                        "contact {} exists but its CSV attributes could not be saved",
                        contact_id
                    ),
                    err,
                )
            }
        }

        for tag in &all_tags {
            match attach_tag(&state.db, &contact_id, tag, &tenant_key).await {
                // `false` means the tag was already attached, which is the desired
                // end state, not a failure worth warning about.
                Ok(_) => {}
                Err(err) => row_warnings.push(format!("tag '{tag}' could not be attached: {err}")),
            }
        }

        if !row_warnings.is_empty() {
            row_errors.push(json!({
                "row": row.row_number,
                "value": value,
                "channel": channel,
                "errors": row_warnings,
                "action": if is_new { "created" } else { "updated" },
                "contactId": contact_id
            }));
        }
    }

    let audit = if created > 0 || updated > 0 {
        write_audit(
            &state.db,
            "contact",
            &tenant_key,
            "imported",
            &tenant_key,
            &format!(
                "CSV import '{}' created {created} contacts ({updated} updated, {skipped} rejected)",
                form.filename.as_deref().unwrap_or("(unnamed)")
            ),
        )
        .await
    } else {
        audit_not_needed("nothing was created or updated, so there is no import to record")
    };

    (
        StatusCode::OK,
        Json(json!({
            "status": if skipped == 0 { "ok" } else { "partial" },
            "created": created,
            "updated": updated,
            "skipped": skipped,
            "total": analyzed.len(),
            "channels": channel_counts,
            "errors": row_errors,
            "warnings": warnings,
            "unknownFields": form.unknown_fields,
            "filename": form.filename,
            "tenantKey": tenant_key,
            "createdIds": created_ids,
            "audit": audit.value()
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Cross-channel identity linking and the relationship profile
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum LinkOutcome {
    /// The link row was written; carries whether the provenance row landed too.
    Linked(AuditWrite),
    AlreadyLinked,
    NoSchema,
}

async fn link_customer_row(
    db: &DbPool,
    contact_id: &str,
    customer_id: &str,
    tenant_key: &str,
) -> Result<LinkOutcome, Response> {
    if !table_exists(db, "customers").await || !column_exists(db, "contacts", "customer_id").await {
        return Ok(LinkOutcome::NoSchema);
    }

    let customer: Option<(String, Option<String>)> =
        match sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT id, tenant_key FROM customers WHERE id = $1",
        )
        .bind(customer_id)
        .fetch_optional(db)
        .await
        {
            Ok(row) => row,
            Err(err) => {
                return Err(db_error_response(
                    "the customers table exists but the customer could not be read from it",
                    err,
                ))
            }
        };

    let Some((customer_id, customer_tenant)) = customer else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Customer {customer_id} does not exist") })),
        )
            .into_response());
    };

    if let Some(owner) = customer_tenant.as_deref() {
        if owner != tenant_key {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("contact belongs to tenant '{tenant_key}' but customer belongs to '{owner}'")
                })),
            )
                .into_response());
        }
    }

    let already: Option<String> =
        match sqlx::query_scalar::<_, String>("SELECT customer_id FROM contacts WHERE id = $1")
            .bind(contact_id)
            .fetch_optional(db)
            .await
        {
            Ok(value) => value,
            Err(err) => return Err(db_error_response("could not read the current customer link", err)),
        };
    if already.as_deref() == Some(customer_id.as_str()) {
        return Ok(LinkOutcome::AlreadyLinked);
    }

    let linked = sqlx::query("UPDATE contacts SET customer_id = $1, updated_at = $2 WHERE id = $3")
        .bind(&customer_id)
        .bind(Utc::now().to_rfc3339())
        .bind(contact_id)
        .execute(db)
        .await;
    // The `Linked` result and its audit row both claim a write happened, so a
    // 0-row UPDATE (the contact vanished between the reads) must not be reported
    // as a link.
    match linked {
        Ok(report) if report.rows_affected() == 0 => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "the contact row could not be updated with the customer link; nothing was linked",
                    "contactId": contact_id,
                    "databaseError": true
                })),
            )
                .into_response());
        }
        Ok(_) => {}
        Err(err) => {
            return Err(db_error_response(
                "the customer exists but the link could not be saved",
                err,
            ));
        }
    }

    let audit = write_audit(
        db,
        "contact",
        contact_id,
        "linked_customer",
        tenant_key,
        &format!("Contact {contact_id} linked to customer {customer_id}"),
    )
    .await;

    Ok(LinkOutcome::Linked(audit))
}

#[derive(Deserialize)]
pub struct LinkCustomerPayload {
    #[serde(alias = "customerId")]
    pub customer_id: String,
}

/// Explicit, agent-confirmed cross-channel identity merge. Never automatic: the
/// relationship is only read as one person once a human says so.
pub async fn link_customer(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(contact_id): Path<String>,
    Json(payload): Json<LinkCustomerPayload>,
) -> Response {
    if payload.customer_id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "`customer_id` is required: the existing Customer row this channel identity belongs to" })),
        )
            .into_response();
    }

    let contact = match contact_owner(&state.db, &contact_id).await {
        Ok(Some((_, owner))) => Some(owner),
        Ok(None) => None,
        Err(err) => return db_error_response("could not read contact", err),
    };

    let Some(owner) = contact else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Contact {contact_id} does not exist") })),
        )
            .into_response();
    };
    if let Some(owner) = owner.as_deref() {
        if owner != tenant_key {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Contact {contact_id} does not belong to tenant '{tenant_key}'") })),
            )
                .into_response();
        }
    }

    match link_customer_row(&state.db, &contact_id, payload.customer_id.trim(), &tenant_key).await {
        Ok(LinkOutcome::Linked(audit)) => (
            StatusCode::OK,
            Json(json!({
                "status": "linked",
                "contactId": contact_id,
                "customerId": payload.customer_id.trim(),
                "audit": audit.value()
            })),
        )
            .into_response(),
        Ok(LinkOutcome::AlreadyLinked) => (
            StatusCode::OK,
            Json(json!({
                "status": "already_linked",
                "contactId": contact_id,
                "customerId": payload.customer_id.trim()
            })),
        )
            .into_response(),
        Ok(LinkOutcome::NoSchema) => missing_dependency_response(
            "customers table / contacts.customer_id column",
            "cross-channel identity links are stored in `customers` plus `contacts.customer_id`; neither exists in this database yet, so nothing was written. Apply the DDL listed under WIRING NEEDED.",
        ),
        Err(response) => response,
    }
}

/// Channel identities that belong to the same Customer.
/// `Ok(None)` means this contact is not linked (or `customers` is absent), so
/// the profile covers this identity only; `Ok(Some(vec![]))` is a real gap in
/// the link and must be reported, not folded into "no link".
async fn sibling_contact_ids(
    db: &DbPool,
    contact: &LoadedContact,
) -> Result<Option<Vec<String>>, sqlx::Error> {
    let Some(customer_id) = contact.customer_id.clone().filter(|c| !c.trim().is_empty()) else {
        return Ok(None);
    };
    if !table_exists(db, "customers").await {
        return Ok(None);
    }
    Ok(Some(
        sqlx::query_scalar::<_, String>("SELECT id FROM contacts WHERE customer_id = $1")
            .bind(&customer_id)
            .fetch_all(db)
            .await?,
    ))
}

/// GET /contacts/{id}/relationship-profile — the pre-response briefing an agent
/// (human or AI) loads. Each fact carries its source, and a section the schema
/// cannot supply is reported as missing rather than quietly omitted.
pub async fn relationship_profile(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(contact_id): Path<String>,
) -> Response {
    let (projection, _missing) = contact_projection(&state.db).await;
    let scoped = projection.contains(&"tenant_key");
    let sql = format!(
        "SELECT {} FROM contacts WHERE id = $1{}",
        projection.join(", "),
        if scoped { " AND COALESCE(tenant_key, '') = $2" } else { "" }
    );
    let mut query = sqlx::query(&sql).bind(&contact_id);
    if scoped {
        query = query.bind(&tenant_key);
    }
    let row = match query.fetch_optional(&state.db).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Contact {contact_id} does not exist for tenant '{tenant_key}'") })),
            )
                .into_response()
        }
        Err(err) => return db_error_response("could not read contact", err),
    };
    let contact = LoadedContact::from_row(&row);

    let mut warnings: Vec<String> = Vec::new();
    let contact_ids = match sibling_contact_ids(&state.db, &contact).await {
        Ok(Some(ids)) if !ids.is_empty() => ids,
        Ok(Some(_)) => {
            // The contact claims a Customer that no contact row points back to:
            // a broken link, not an unlinked contact.
            warnings.push(format!(
                "customer '{}' is stored on this contact but no contact row references it; the profile covers this identity only",
                contact.customer_id.clone().unwrap_or_default()
            ));
            vec![contact.id.clone()]
        }
        Ok(None) => vec![contact.id.clone()],
        Err(err) => return db_error_response("could not read the sibling channel identities", err),
    };

    if !table_exists(&state.db, "customers").await {
        warnings.push(
            "customers table absent: the profile covers this channel identity only, not the person across channels"
                .to_string(),
        );
    }

    // channels_seen: real channel/value pairs for every linked identity.
    let mut channels_seen: Vec<serde_json::Value> = Vec::new();
    let placeholders = (1..=contact_ids.len())
        .map(|i| format!("${i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let channel_sql = format!("SELECT channel, value FROM contacts WHERE id IN ({placeholders})");
    let mut channel_query = sqlx::query(&channel_sql);
    for id in &contact_ids {
        channel_query = channel_query.bind(id);
    }
    match channel_query.fetch_all(&state.db).await {
        Ok(rows) => {
            for row in rows {
                // A column that cannot be read is reported, not renamed to an
                // "unknown" channel that looks like a stored identity.
                let channel: Option<String> = match row.try_get::<Option<String>, _>("channel") {
                    Ok(value) => value,
                    Err(err) => {
                        warnings.push(format!("a linked identity's channel could not be read: {err}"));
                        None
                    }
                };
                let value: Option<String> = match row.try_get::<Option<String>, _>("value") {
                    Ok(value) => value,
                    Err(err) => {
                        warnings.push(format!("a linked identity's value could not be read: {err}"));
                        None
                    }
                };
                channels_seen.push(json!({
                    "channel": channel,
                    "value": value,
                    "source": "contact_record"
                }));
            }
        }
        Err(err) => return db_error_response("could not read the linked channel identities", err),
    }

    // Conversations across every identity, newest first.
    #[derive(sqlx::FromRow)]
    struct ConvRow {
        id: String,
        state: Option<String>,
        resolution_type: Option<String>,
        updated_at: Option<String>,
        created_at: Option<String>,
    }
    let conv_sql = format!(
        "SELECT id, state, resolution_type, updated_at, created_at FROM conversations WHERE contact_id IN ({placeholders}) ORDER BY updated_at DESC"
    );
    let mut conv_query = sqlx::query_as::<_, ConvRow>(&conv_sql);
    for id in &contact_ids {
        conv_query = conv_query.bind(id);
    }
    let conversations = match conv_query.fetch_all(&state.db).await {
        Ok(rows) => rows,
        Err(err) => return db_error_response("could not read the conversation history", err),
    };

    let active_states = ["new", "awaiting_reply", "engaged"];
    let stage = if conversations.is_empty() {
        "new_prospect"
    } else if conversations
        .iter()
        .any(|c| c.state.as_deref().map(|s| s.eq_ignore_ascii_case("closed_won")).unwrap_or(false))
    {
        "client"
    } else if conversations.iter().any(|c| {
        let state = c.state.clone().unwrap_or_default().to_lowercase();
        active_states.contains(&state.as_str()) || state == "hot_lead"
    }) {
        "active_prospect"
    } else {
        "dormant"
    };

    let prior_closed: Vec<&ConvRow> = conversations
        .iter()
        .filter(|c| {
            let state = c.state.clone().unwrap_or_default().to_lowercase();
            state == "closed_won" || state == "closed_lost"
        })
        .collect();

    let prior_closed_outcomes: Vec<serde_json::Value> = prior_closed
        .iter()
        .take(3)
        .map(|c| {
            json!({
                "id": c.id.clone(),
                "state": c.state.clone(),
                "resolution_type": c.resolution_type.clone(),
                "closed_at": c.updated_at.clone(),
                "source": "conversation_record"
            })
        })
        .collect();

    let last_conversation = conversations.first();

    // Inquiries are the commercial memory; absent table => null + warning.
    let mut latest_inquiry: Option<serde_json::Value> = None;
    let mut total_inquiries = 0usize;
    if table_exists(&state.db, "inquiries").await {
        let count_sql = format!("SELECT count(*) FROM inquiries WHERE contact_id IN ({placeholders})");
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for id in &contact_ids {
            count_query = count_query.bind(id);
        }
        match count_query.fetch_optional(&state.db).await {
            Ok(Some(n)) => total_inquiries = n.max(0) as usize,
            Ok(None) => {}
            Err(err) => warnings.push(format!("inquiries count could not be read: {err}")),
        }

        let inquiry_sql = format!(
            "SELECT * FROM inquiries WHERE contact_id IN ({placeholders}) ORDER BY created_at DESC LIMIT 1"
        );
        let mut inquiry_query = sqlx::query(&inquiry_sql);
        for id in &contact_ids {
            inquiry_query = inquiry_query.bind(id);
        }
        match inquiry_query.fetch_optional(&state.db).await {
            Ok(Some(row)) => {
                let text = |key: &str| -> Option<String> {
                    row.try_get::<Option<String>, _>(key).ok().flatten()
                };
                let score: Option<f64> = match row.try_get::<Option<f64>, _>("qualification_score") {
                    Ok(value) => value,
                    Err(_) => row.try_get::<Option<i64>, _>("qualification_score").ok().flatten().map(|n| n as f64),
                };
                latest_inquiry = Some(json!({
                    "organization_name": text("organization_name"),
                    "industry": text("industry"),
                    "problem_description": text("problem_description"),
                    "capability_mapping": text("capability_mapping").and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok()),
                    "status": text("status"),
                    "qualification_score": score,
                    "source": "inquiry_record"
                }));
            }
            Ok(None) => {}
            Err(err) => {
                warnings.push(format!("inquiries table exists but could not be read: {err}"));
            }
        }
    } else {
        warnings.push(
            "inquiries table is not part of this schema: latest_inquiry is null rather than guessed".to_string(),
        );
    }

    // Remembered facts.
    let mut memories: Vec<serde_json::Value> = Vec::new();
    if table_exists(&state.db, "contact_memories").await {
        let memories_sql = format!(
            "SELECT id, contact_id, fact, category, confidence, noted_at, created_at FROM contact_memories WHERE contact_id IN ({placeholders}) ORDER BY COALESCE(noted_at, created_at) DESC LIMIT 10"
        );
        let mut q = sqlx::query(&memories_sql);
        for id in &contact_ids {
            q = q.bind(id);
        }
        match q.fetch_all(&state.db).await {
            Ok(rows) => {
                for row in rows {
                    let text = |key: &str| -> Option<String> {
                        row.try_get::<Option<String>, _>(key).ok().flatten()
                    };
                    memories.push(json!({
                        "id": text("id"),
                        "contactId": text("contact_id"),
                        "fact": text("fact").unwrap_or_default(),
                        "category": text("category"),
                        "confidence": row.try_get::<Option<f64>, _>("confidence").ok().flatten(),
                        "noted_at": text("noted_at").or_else(|| text("created_at")),
                        "notedAt": text("noted_at").or_else(|| text("created_at")),
                        "source": "contact_memory"
                    }));
                }
            }
            Err(err) => {
                warnings.push(format!("contact_memories exists but could not be read: {err}"));
            }
        }
    } else {
        warnings.push(
            "contact_memories table is not wired: remembered facts are unavailable".to_string(),
        );
    }

    // Lead record, when the provisioning path attached one.
    let mut lead_json = serde_json::Value::Null;
    if let Some(lead_id) = contact.lead_id.clone() {
        if table_exists(&state.db, "leads").await {
            match sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>, Option<String>)>(
                "SELECT id, name, harvester_institution_id, qualification_rationale FROM leads WHERE id = $1",
            )
            .bind(&lead_id)
            .fetch_optional(&state.db)
            .await
            {
                Ok(Some((id, name, institution, rationale))) => {
                    lead_json = json!({
                        "id": id,
                        "name": name,
                        "harvester_institution_id": institution,
                        "qualification_rationale": rationale
                    });
                }
                Ok(None) => warnings.push(format!("lead {lead_id} is referenced by the contact but is not in leads")),
                Err(err) => return db_error_response("could not read the lead", err),
            }
        }
    }

    let payload = json!({
        "contact": {
            "id": contact.id.clone(),
            "name": contact.person_name.clone(),
            "value": contact.value.clone(),
            "channel": contact.channel.clone(),
            "language": contact.language.clone(),
            "firstSeenAt": contact.created_at.clone(),
            "customerId": contact.customer_id.clone()
        },
        "contact_id": contact.id.clone(),
        "channels_seen": channels_seen,
        "relationship_stage": stage,
        "total_conversations": conversations.len(),
        "total_inquiries": total_inquiries,
        "last_interaction_at": last_conversation.and_then(|c| c.updated_at.clone()),
        "last_interaction_outcome": last_conversation.and_then(|c| c.resolution_type.clone()),
        "latest_inquiry": latest_inquiry,
        "prior_closed_outcomes": prior_closed_outcomes,
        "memories": memories.clone(),
        // `remembered_facts` is the key the profile page falls back to; it is
        // the same provenance-tagged list, not a second source of truth.
        "remembered_facts": memories,
        "lead": lead_json,
        "linked_contact_ids": contact_ids,
        "warnings": warnings
    });

    (StatusCode::OK, Json(payload)).into_response()
}

