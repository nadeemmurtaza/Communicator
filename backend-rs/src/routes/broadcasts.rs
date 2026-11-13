use crate::db::AppState;
use crate::routes::auth::Tenant;
use crate::services::{channels, messaging};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

/// `models::Broadcast` carries only the columns the list view had; dispatch
/// additionally needs the template payload and the owning tenant.
const BROADCAST_FIELDS: &str = "id, name, template_name, template_language, body_preview, \
    tag_name, status, scheduled_at, created_at, audience_count, total_recipients, sent_count, \
    delivered_count, read_count, failed_count, skipped_count, template_params, tenant_key";

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastRow {
    pub id: String,
    pub name: String,
    pub template_name: Option<String>,
    pub template_language: Option<String>,
    pub body_preview: Option<String>,
    pub tag_name: Option<String>,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub created_at: String,
    pub audience_count: Option<i32>,
    pub total_recipients: Option<i32>,
    pub sent_count: Option<i32>,
    pub delivered_count: Option<i32>,
    pub read_count: Option<i32>,
    pub failed_count: Option<i32>,
    pub skipped_count: Option<i32>,
    #[serde(serialize_with = "json_or_raw")]
    pub template_params: Option<String>,
    pub tenant_key: Option<String>,
}

/// The column is TEXT holding JSON. Emit it parsed so the client sees the object
/// it wrote; when an operator pasted something that is not JSON, emit the raw
/// string instead of failing the whole read or silently returning null.
fn json_or_raw<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value.as_deref().and_then(|raw| serde_json::from_str::<Value>(raw).ok()) {
        Some(parsed) => parsed.serialize(serializer),
        None => value.serialize(serializer),
    }
}

fn db_error(context: &str, err: sqlx::Error) -> Response {
    tracing::warn!("[BROADCASTS] {context}: {err}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("{context}: {err}"),
            "databaseError": true,
        })),
    )
        .into_response()
}

fn refuse(status: StatusCode, message: &str, extra: Value) -> Response {
    let mut body = json!({ "error": message, "sent": false });
    if let (Some(target), Some(source)) = (body.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    (status, Json(body)).into_response()
}

/// The caller may pass a schedule; the stored value must be a timestamp the
/// scheduler can compare, so anything unparseable is refused instead of stored
/// and silently never firing.
fn normalize_timestamp(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Utc).to_rfc3339());
    }
    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(Utc.from_utc_datetime(&naive).to_rfc3339());
        }
    }
    None
}

/// A stored `Option<String>` is only a value when it holds text; empty and
/// whitespace-only columns are the absence of a setting, not a setting of "".
fn clean_text(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|text| !text.is_empty())
}

fn parse_tags(tag_str: &str) -> Vec<String> {    tag_str
        .split(|c| c == ',' || c == '|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Operator-supplied template variables. A map names its own placeholders; an
/// array is positional. Anything else is refused rather than guessed at.
#[derive(Debug, Clone, Default)]
struct TemplateParams {
    named: HashMap<String, String>,
    positional: Vec<String>,
}

fn scalar_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Null => Some(String::new()),
        _ => None,
    }
}

fn parse_params(value: &Value) -> Result<TemplateParams, String> {
    match value {
        Value::Null => Ok(TemplateParams::default()),
        Value::Object(map) => {
            let mut named = HashMap::new();
            for (key, entry) in map {
                let text = scalar_to_string(entry).ok_or_else(|| {
                    format!("template_params.{key} must be a string, number or boolean")
                })?;
                named.insert(key.trim().to_lowercase(), text);
            }
            Ok(TemplateParams {
                named,
                positional: Vec::new(),
            })
        }
        Value::Array(items) => {
            let mut positional = Vec::new();
            for (index, entry) in items.iter().enumerate() {
                let text = scalar_to_string(entry).ok_or_else(|| {
                    format!("template_params[{index}] must be a string, not a nested object")
                })?;
                positional.push(text);
            }
            Ok(TemplateParams {
                named: HashMap::new(),
                positional,
            })
        }
        other => Err(format!(
            "template_params must be an object of named values or an ordered array, got {}",
            match other {
                Value::Bool(_) => "a boolean",
                Value::Number(_) => "a number",
                _ => "an unknown shape",
            }
        )),
    }
}

fn params_from_column(raw: Option<&str>) -> Result<TemplateParams, String> {
    let Some(text) = raw.map(str::trim).filter(|value| !value.is_empty() && *value != "{}") else {
        return Ok(TemplateParams::default());
    };
    let parsed: Value =
        serde_json::from_str(text).map_err(|err| format!("stored template_params is not JSON: {err}"))?;
    parse_params(&parsed)
}

#[derive(sqlx::FromRow)]
struct AudienceRow {
    id: String,
    value: String,
    channel: String,
    person_name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    consent: Option<String>,
}

impl AudienceRow {
    fn token(&self, name: &str) -> Option<String> {
        let non_empty = |value: Option<&String>| {
            value
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        match name {
            "name" | "full_name" | "customer_name" => non_empty(self.person_name.as_ref()),
            "first_name" => non_empty(self.first_name.as_ref()).or_else(|| {
                non_empty(self.person_name.as_ref()).map(|full| {
                    full.split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string()
                })
            }),
            "last_name" => non_empty(self.last_name.as_ref()).or_else(|| {
                non_empty(self.person_name.as_ref()).and_then(|full| {
                    full.split_whitespace()
                        .skip(1)
                        .next()
                        .map(str::to_string)
                })
            }),
            "phone" | "msisdn" => Some(self.value.clone()).filter(|v| !v.trim().is_empty()),
            "contact_id" | "id" => Some(self.id.clone()),
            _ => None,
        }
    }
}

/// Placeholders in operator-authored text, in first-appearance order.
/// `Err` marks an unclosed `{{`, which must not reach a customer.
fn extract_tokens(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            return Err(format!("unclosed `{{{{` in: {}", &rest[start..]));
        };
        let token = after[..end].trim().to_lowercase();
        if token.is_empty() {
            return Err("empty placeholder `{{}}`".to_string());
        }
        if !tokens.contains(&token) {
            tokens.push(token);
        }
        rest = &after[end + 2..];
    }
    Ok(tokens)
}

fn resolve_token(
    token: &str,
    contact: &AudienceRow,
    params: &TemplateParams,
) -> Result<String, String> {
    if let Some(value) = params.named.get(token) {
        return Ok(value.clone());
    }
    if let Some(value) = contact.token(token) {
        return Ok(value);
    }
    if let Ok(index) = token.parse::<usize>() {
        if index >= 1 {
            return params
                .positional
                .get(index - 1)
                .cloned()
                .ok_or_else(|| format!("no value supplied for positional parameter {index}"));
        }
    }
    Err(format!(
        "`{{{{{token}}}}}` has no value: not in template_params and not a contact field \
         (name, first_name, last_name, phone, contact_id)"
    ))
}

/// Fills placeholders using the operator's params first, then real contact
/// fields. Unresolvable placeholders abort the send for that recipient -- a
/// half-substituted message is worse than an unsent one.
fn fill_placeholders(
    input: &str,
    contact: &AudienceRow,
    params: &TemplateParams,
) -> Result<String, String> {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    let mut missing: Vec<String> = Vec::new();
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            return Err(format!("the message text has an unclosed placeholder: {}", &rest[start..]));
        };
        let token = after[..end].trim().to_lowercase();
        match resolve_token(&token, contact, params) {
            Ok(value) => out.push_str(&value),
            Err(reason) => missing.push(reason),
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    if missing.is_empty() {
        Ok(out)
    } else {
        Err(format!(
            "contact {} cannot be sent: {}",
            contact.id,
            missing.join("; ")
        ))
    }
}

/// Meta takes template variables as an ordered list. The order is derived from
/// the operator's own preview text (first appearance of each placeholder); when
/// the preview has no placeholders, an explicitly ordered array is used.
fn build_body_params(
    preview: &str,
    contact: &AudienceRow,
    params: &TemplateParams,
) -> Result<Vec<String>, String> {
    let tokens = extract_tokens(preview)?;
    if !tokens.is_empty() {
        let mut values = Vec::with_capacity(tokens.len());
        for token in &tokens {
            values.push(resolve_token(token, contact, params)?);
        }
        return Ok(values);
    }
    let mut values = Vec::with_capacity(params.positional.len());
    for entry in &params.positional {
        values.push(fill_placeholders(entry, contact, params)?);
    }
    Ok(values)
}

async fn tenant_scoped_contacts(state: &AppState) -> bool {
    crate::routes::contacts::column_exists(&state.db, "contacts", "tenant_key").await
}

/// The audience of a campaign is the set of contacts carrying its tags, inside
/// the requesting tenant. Consent is reported, never assumed.
async fn load_audience(
    state: &AppState,
    tags_json: &str,
    tenant_key: &str,
) -> Result<Vec<AudienceRow>, String> {
    let scoped = tenant_scoped_contacts(state).await;
    let sql = format!(
        r#"
        SELECT DISTINCT
            ct.id,
            ct.value,
            ct.channel,
            ct.person_name,
            ct.first_name,
            ct.last_name,
            (SELECT cr.status FROM consent_records cr
              WHERE cr.contact_id = ct.id
              ORDER BY cr.recorded_at DESC LIMIT 1) AS consent
        FROM contacts ct
        JOIN contact_tags ctags ON ct.id = ctags.contact_id
        JOIN tags t ON ctags.tag_id = t.id
        WHERE t.name IN (SELECT value FROM json_each($1))
        {}
        ORDER BY ct.id
        "#,
        if scoped {
            "AND COALESCE(ct.tenant_key, '') = $2"
        } else {
            ""
        }
    );
    let mut query = sqlx::query_as::<_, AudienceRow>(&sql).bind(tags_json);
    if scoped {
        query = query.bind(tenant_key);
    }
    query
        .fetch_all(&state.db)
        .await
        .map_err(|err| format!("the audience could not be resolved: {err}"))
}

fn tags_payload(tags: &[String]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
}

// ------------------------------------------------------------------ handlers

pub async fn list_broadcasts(State(state): State<AppState>, Tenant(tenant_key): Tenant) -> Response {
    let rows = match sqlx::query_as::<_, BroadcastRow>(&format!(
        "SELECT {BROADCAST_FIELDS} FROM broadcasts WHERE COALESCE(tenant_key, '') = $1 ORDER BY created_at DESC"
    ))
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("the campaign list could not be read", err),
    };

    Json(rows).into_response()
}

#[derive(Deserialize)]
pub struct CreateBroadcastPayload {
    pub name: String,
    pub template_name: Option<String>,
    pub template_language: Option<String>,
    pub body_preview: Option<String>,
    pub template_params: Option<Value>,
    pub tag_name: Option<String>,
    pub scheduled_at: Option<String>,
    /// Only ever a claim: the campaign is stored under the authenticated tenant,
    /// and a body that names a different one is refused instead of ignored.
    pub tenant_key: Option<String>,
}

pub async fn create_broadcast(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(payload): Json<CreateBroadcastPayload>,
) -> Response {
    if let Some(claimed) = payload
        .tenant_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if claimed != tenant_key {
            return refuse(
                StatusCode::FORBIDDEN,
                &format!(
                    "the request body names tenant '{claimed}' but the credentials belong to \
                     '{tenant_key}'; the campaign was not created."
                ),
                json!({ "field": "tenant_key", "requestTenant": claimed }),
            );
        }
    }
    let name = payload.name.trim();
    if name.is_empty() {
        return refuse(StatusCode::BAD_REQUEST, "`name` is required.", json!({ "field": "name" }));
    }
    let tags = parse_tags(payload.tag_name.as_deref().unwrap_or_default());
    if tags.is_empty() {
        return refuse(
            StatusCode::BAD_REQUEST,
            "`tag_name` must list at least one audience tag. Campaigns are never sent to the \
             whole contact table by default.",
            json!({ "field": "tag_name" }),
        );
    }
    let body_preview = payload
        .body_preview
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(preview) = body_preview else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "`body_preview` is required: it is the text recorded on the thread and the source \
             of the template variable order.",
            json!({ "field": "body_preview" }),
        );
    };

    let template_name = payload
        .template_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let template_language = payload
        .template_language
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if template_name.is_none() && template_language.is_some() {
        return refuse(
            StatusCode::BAD_REQUEST,
            "`template_language` was given without `template_name`.",
            json!({ "field": "template_name" }),
        );
    }
    if let Err(reason) = extract_tokens(preview) {
        return refuse(
            StatusCode::BAD_REQUEST,
            &format!("the message text is malformed: {reason}"),
            json!({ "field": "body_preview" }),
        );
    }

    // Validated here so an unusable shape never reaches the database; the raw
    // JSON is what gets stored, and dispatch re-reads it with the same parser.
    if let Some(raw) = payload.template_params.as_ref() {
        if let Err(reason) = parse_params(raw) {
            return refuse(
                StatusCode::BAD_REQUEST,
                &format!("`template_params` is unusable: {reason}"),
                json!({ "field": "template_params" }),
            );
        }
    }
    let params_json = payload
        .template_params
        .filter(|value| !value.is_null())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let scheduled_at = match payload.scheduled_at.as_deref() {
        None => None,
        Some(raw) => match normalize_timestamp(raw) {
            Some(stamp) => Some(stamp),
            None => {
                return refuse(
                    StatusCode::BAD_REQUEST,
                    &format!("`scheduled_at` \"{raw}\" is not an ISO-8601 timestamp."),
                    json!({ "field": "scheduled_at" }),
                )
            }
        },
    };

    let audience = match load_audience(&state, &tags_payload(&tags), &tenant_key).await {
        Ok(rows) => rows,
        Err(reason) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": reason })),
            )
                .into_response()
        }
    };
    let audience_count = audience.len() as i32;

    let id = Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();
    let status = if scheduled_at.is_some() { "scheduled" } else { "draft" };

    let insert = sqlx::query(
        r#"
        INSERT INTO broadcasts (
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at, audience_count, total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count,
            template_params, tenant_key
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 0, 0, 0, 0, 0, $12, $13)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(template_name.as_deref())
    .bind(template_language.as_deref())
    .bind(preview)
    .bind(payload.tag_name.as_deref().unwrap_or_default())
    .bind(status)
    .bind(scheduled_at.as_deref())
    .bind(&now)
    .bind(audience_count)
    .bind(audience_count)
    .bind(&params_json)
    .bind(&tenant_key)
    .execute(&state.db)
    .await;

    let insert = match insert {
        Ok(report) => report,
        Err(err) => return db_error("the campaign could not be stored", err),
    };

    if insert.rows_affected() != 1 {
        return db_error(
            "the campaign could not be stored",
            sqlx::Error::Protocol(format!(
                "INSERT broadcasts reported {} rows affected, expected 1",
                insert.rows_affected()
            )),
        );
    }

    let mut warnings = Vec::new();
    if audience_count == 0 {
        warnings.push(format!(
            "no contacts in tenant '{tenant_key}' carry the tag(s) {tags:?}; this campaign has an \
             empty audience."
        ));
    }
    if let Some(stamp) = &scheduled_at {
        let in_past = DateTime::parse_from_rfc3339(stamp)
            .map(|when| when.with_timezone(&Utc) <= Utc::now())
            .unwrap_or(false);
        if in_past {
            warnings.push("scheduled_at is already in the past, so the next scheduler run will \
                           dispatch this campaign immediately."
                .to_string());
        }
    }

    Json(json!({
        "id": id,
        "name": name,
        "templateName": template_name,
        "template_name": template_name,
        "templateLanguage": template_language,
        "template_language": template_language,
        "bodyPreview": preview,
        "body_preview": preview,
        "templateParams": payload_template_params_json(&params_json),
        "tagName": payload.tag_name.unwrap_or_default(),
        "tags": tags,
        "status": status,
        "scheduledAt": scheduled_at,
        "createdAt": now,
        "tenantKey": tenant_key,
        "audienceCount": audience_count,
        "audience_count": audience_count,
        "totalRecipients": audience_count,
        "sentCount": 0,
        "deliveredCount": 0,
        "readCount": 0,
        "failedCount": 0,
        "skippedCount": 0,
        "warnings": warnings,
    }))
    .into_response()
}

fn payload_template_params_json(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or(Value::Null)
}

#[derive(Deserialize)]
pub struct AudienceQuery {
    pub tags: Option<String>,
}

pub async fn get_audience_preview(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(query): Query<AudienceQuery>,
) -> Response {
    let tags = parse_tags(query.tags.as_deref().unwrap_or_default());
    if tags.is_empty() {
        return Json(json!({
            "tags": Vec::<String>::new(),
            "totalSelected": 0,
            "eligibleCount": 0,
            "optedInCount": 0,
            "optedOutCount": 0,
            "unknownConsentCount": 0,
            "sampleContacts": Vec::<Value>::new(),
            "dailyLimit": Value::Null,
            "dailyLimitSource": Value::Null,
            "warnings": ["no tags were supplied, so no audience was resolved."],
        }))
        .into_response();
    }

    let audience = match load_audience(&state, &tags_payload(&tags), &tenant_key).await {
        Ok(rows) => rows,
        Err(reason) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": reason }))).into_response()
        }
    };

    let mut opted_in = 0_i64;
    let mut opted_out = 0_i64;
    let mut unknown = 0_i64;
    for contact in &audience {
        match contact.consent.as_deref().map(|s| s.trim().to_lowercase()) {
            Some(status) if status == "opted_in" => opted_in += 1,
            Some(status) if status == "opted_out" => opted_out += 1,
            _ => unknown += 1,
        }
    }

    let sample_contacts: Vec<Value> = audience
        .iter()
        .take(5)
        .map(|contact| {
            json!({
                "id": contact.id,
                "name": contact.person_name,
                "phone": contact.value,
                "channel": contact.channel,
                "consent": contact.consent,
            })
        })
        .collect();

    // Meta's per-number daily messaging cap is real account state, so it is read
    // from the connected phone number instead of assuming a number.
    let (access_token, phone_number_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let profile = crate::services::meta_sync::fetch_meta_business_profile(
        &state,
        &access_token,
        &phone_number_id,
        &state.config.meta_whatsapp_api_version,
    )
    .await;

    let mut warnings = Vec::new();
    let (daily_limit, limit_source) = if profile.messaging_limit_tier.trim().is_empty() {
        (
            Value::Null,
            Value::Null,
        )
    } else {
        (
            Value::String(profile.messaging_limit_tier.clone()),
            Value::String(format!(
                "Meta business profile{}",
                if profile.display_phone_number.trim().is_empty() {
                    " for the connected number".to_string()
                } else {
                    format!(" for {}", profile.display_phone_number)
                }
            )),
        )
    };
    if daily_limit.is_null() {
        warnings.push(match profile.error {
            Some(reason) => format!("the Meta daily cap is unknown: {reason}"),
            None => "Meta returned no messaging_limit_tier for the connected number, so the daily \
                    cap is unknown rather than assumed."
                .to_string(),
        });
    }
    if unknown > 0 {
        warnings.push(format!(
            "{unknown} contact(s) have no OPTED_IN consent record; the send pipeline refuses them, \
             so only {opted_in} are actually deliverable."
        ));
    }

    Json(json!({
        "tags": tags,
        "totalSelected": audience.len(),
        "eligibleCount": opted_in,
        "optedInCount": opted_in,
        "optedOutCount": opted_out,
        "unknownConsentCount": unknown,
        "sampleContacts": sample_contacts,
        "dailyLimit": daily_limit,
        "dailyLimitSource": limit_source,
        "tenantKey": tenant_key,
        "warnings": warnings,
    }))
    .into_response()
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipientDetailRow {
    pub id: String,
    pub contact_id: String,
    pub message_id: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
}

const RECIPIENT_SELECT: &str = r#"
    SELECT
        br.id,
        br.contact_id,
        br.message_id,
        ct.person_name AS contact_name,
        ct.value AS contact_phone,
        br.status,
        br.error,
        br.sent_at,
        br.delivered_at,
        br.read_at
    FROM broadcast_recipients br
    LEFT JOIN contacts ct ON br.contact_id = ct.id
    WHERE br.broadcast_id = $1
    ORDER BY br.sent_at DESC
"#;

pub async fn get_campaign_analytics(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    let campaign = match sqlx::query_as::<_, BroadcastRow>(&format!(
        "SELECT {BROADCAST_FIELDS} FROM broadcasts WHERE id = $1 AND COALESCE(tenant_key, '') = $2"
    ))
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the campaign could not be read", err),
    };
    let Some(campaign) = campaign else {
        return refuse(
            StatusCode::NOT_FOUND,
            &format!("campaign {id} does not exist for tenant '{tenant_key}'."),
            json!({ "campaignId": id }),
        );
    };

    let recipients = match sqlx::query_as::<_, RecipientDetailRow>(RECIPIENT_SELECT)
        .bind(&id)
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("the recipient ledger could not be read", err),
    };

    // Recomputed from the ledger, because the stored counters are only as fresh
    // as the last writer and receipts arrive after the send.
    let actual_sent = recipients.iter().filter(|r| r.status == "sent").count();
    let actual_delivered = recipients.iter().filter(|r| r.delivered_at.is_some()).count();
    let actual_read = recipients.iter().filter(|r| r.read_at.is_some()).count();
    let actual_failed = recipients.iter().filter(|r| r.status == "failed").count();
    let actual_skipped = recipients.iter().filter(|r| r.status == "not_sent").count();

    let audience = recipients.len().max(campaign.audience_count.unwrap_or(0) as usize);
    let stored_sent = campaign.sent_count.unwrap_or(0) as usize;
    let stored_failed = campaign.failed_count.unwrap_or(0) as usize;
    let stored_skipped = campaign.skipped_count.unwrap_or(0) as usize;

    let mut warnings = Vec::new();
    if stored_sent != actual_sent || stored_failed != actual_failed || stored_skipped != actual_skipped {
        warnings.push(format!(
            "stored counters (sent {stored_sent}, failed {stored_failed}, skipped {stored_skipped}) \
             disagree with the recipient ledger (sent {actual_sent}, failed {actual_failed}, \
             skipped {actual_skipped}); the ledger is reported."
        ));
    }
    if actual_delivered == 0 && actual_sent > 0 {
        warnings.push("no delivery receipts yet: Meta reports delivered/read asynchronously \
                       through the webhook.".to_string());
    }

    let pct = |value: usize| -> i64 {
        if audience == 0 {
            0
        } else {
            ((value as f64 / audience as f64) * 100.0).round() as i64
        }
    };

    Json(json!({
        "campaign": campaign,
        "metrics": {
            "audience": audience,
            "submitted": actual_sent,
            "submittedPct": pct(actual_sent),
            "delivered": actual_delivered,
            "deliveredPct": pct(actual_delivered),
            "read": actual_read,
            "readPct": pct(actual_read),
            "failed": actual_failed,
            "failedPct": pct(actual_failed),
            "skipped": actual_skipped,
            "skippedPct": pct(actual_skipped),
        },
        "deliveryBreakdown": {
            "deliveredRate": pct(actual_delivered),
            "readRate": pct(actual_read),
            "failureRate": pct(actual_failed),
        },
        "recipients": recipients,
        "campaignId": id,
        "totalRecipients": audience,
        "sent": actual_sent,
        "delivered": actual_delivered,
        "read": actual_read,
        "failed": actual_failed,
        "skipped": actual_skipped,
        "deliveryRate": pct(actual_delivered),
        "readRate": pct(actual_read),
        "warnings": warnings,
    }))
    .into_response()
}

#[derive(Deserialize)]
pub struct ReschedulePayload {
    pub scheduled_at: Option<String>,
}

pub async fn reschedule_campaign(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<ReschedulePayload>,
) -> Response {
    let scheduled_at = match payload.scheduled_at.as_deref() {
        None => None,
        Some(raw) => match normalize_timestamp(raw) {
            Some(stamp) => Some(stamp),
            None => {
                return refuse(
                    StatusCode::BAD_REQUEST,
                    &format!("`scheduled_at` \"{raw}\" is not an ISO-8601 timestamp."),
                    json!({ "field": "scheduled_at" }),
                )
            }
        },
    };

    let existing = match sqlx::query("SELECT status FROM broadcasts WHERE id = $1 AND COALESCE(tenant_key, '') = $2")
        .bind(&id)
        .bind(&tenant_key)
        .fetch_optional(&state.db)
        .await
    {
        Ok(row) => row.and_then(|r| r.try_get::<String, _>("status").ok()),
        Err(err) => return db_error("the campaign state could not be read", err),
    };
    let Some(current_status) = existing else {
        return refuse(
            StatusCode::NOT_FOUND,
            &format!("campaign {id} does not exist for tenant '{tenant_key}'."),
            json!({ "campaignId": id }),
        );
    };
    if current_status == "sending" || current_status == "completed" {
        return refuse(
            StatusCode::CONFLICT,
            &format!(
                "a campaign in status '{current_status}' has already been dispatched and cannot \
                 be rescheduled."
            ),
            json!({ "campaignId": id, "status": current_status }),
        );
    }

    let status = if scheduled_at.is_some() { "scheduled" } else { "draft" };
    let updated = match sqlx::query("UPDATE broadcasts SET scheduled_at = $1, status = $2 WHERE id = $3")
        .bind(scheduled_at.as_deref())
        .bind(status)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the schedule could not be stored", err),
    };
    if updated.rows_affected() != 1 {
        return db_error(
            "the schedule could not be stored",
            sqlx::Error::Protocol(format!(
                "UPDATE broadcasts affected {} rows, expected 1",
                updated.rows_affected()
            )),
        );
    }

    Json(json!({
        "id": id,
        "status": status,
        "scheduledAt": scheduled_at,
        "previousStatus": current_status,
    }))
    .into_response()
}

/// Shared by the dispatch route, the `/broadcasts/process-scheduled` endpoint
/// and `routes/scheduler.rs`, so every entry point gets the same truth.
pub(crate) async fn run_dispatch(
    state: &AppState,
    tenant_key: &str,
    id: &str,
) -> Result<Value, (StatusCode, Value)> {
    let campaign = sqlx::query_as::<_, BroadcastRow>(&format!(
        "SELECT {BROADCAST_FIELDS} FROM broadcasts WHERE id = $1 AND COALESCE(tenant_key, '') = $2"
    ))
    .bind(id)
    .bind(tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": format!("the campaign could not be read: {err}") })))?;
    let Some(campaign) = campaign else {
        return Err((
            StatusCode::NOT_FOUND,
            json!({ "error": format!("campaign {id} does not exist for tenant '{tenant_key}'.") }),
        ));
    };

    if matches!(campaign.status.as_str(), "sending" | "completed" | "cancelled") {
        return Err((
            StatusCode::CONFLICT,
            json!({
                "error": format!("campaign {id} is already in status '{}'; refusing to send it twice.", campaign.status),
                "status": campaign.status,
            }),
        ));
    }

    let tags = parse_tags(campaign.tag_name.as_deref().unwrap_or_default());
    if tags.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({
                "error": "this campaign has no audience tags; sending it would mean messaging \
                          contacts that were never selected, so nothing was sent.",
                "field": "tag_name",
            }),
        ));
    }

    let preview = campaign
        .body_preview
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "body_preview is empty; there is no operator-authored message to send.",
                    "field": "body_preview",
                }),
            )
        })?;

    let params = params_from_column(campaign.template_params.as_deref()).map_err(|reason| {
        (
            StatusCode::BAD_REQUEST,
            json!({ "error": format!("stored template_params is unusable: {reason}"), "field": "template_params" }),
        )
    })?;

    // A template send needs both halves of the identity; one without the other
    // is not a template, so it is treated as no template and reported.
    let template_spec = match (clean_text(&campaign.template_name), clean_text(&campaign.template_language)) {
        (Some(name), Some(language)) => Some((name.to_string(), language.to_string())),
        _ => None,
    };
    let template_ready = template_spec.is_some();

    let audience = load_audience(state, &tags_payload(&tags), tenant_key)
        .await
        .map_err(|reason| (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": reason })))?;
    if audience.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({
                "error": format!("no contacts in tenant '{tenant_key}' carry the tag(s) {tags:?}."),
                "audienceCount": 0,
            }),
        ));
    }

    // Credential truth first: a campaign where nothing can leave the process is
    // refused before any row is written, rather than recorded as sent.
    let mut channels_needed: Vec<String> = audience
        .iter()
        .map(|contact| contact.channel.trim().to_lowercase())
        .filter(|channel| !channel.is_empty())
        .collect();
    channels_needed.sort();
    channels_needed.dedup();
    if template_ready {
        channels_needed.retain(|channel| channel == "whatsapp");
    }
    if channels_needed.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({
                "error": "none of the audience contacts record a usable channel, so there is \
                          nowhere to send this campaign.",
            }),
        ));
    }
    let mut blocked: Vec<String> = Vec::new();
    for channel in &channels_needed {
        let report = channels::missing_credentials_report(state, channel, None).await;
        let credential_ok = report
            .error
            .as_deref()
            .unwrap_or_default()
            .starts_with("Credential pre-check only");
        if !credential_ok {
            blocked.push(format!("{channel}: {}", report.reason()));
        }
    }
    if !blocked.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            json!({
                "error": "no campaign was dispatched: the outbound credentials are not configured.",
                "channels": blocked,
                "audienceCount": audience.len(),
            }),
        ));
    }

    let mark_sending = sqlx::query("UPDATE broadcasts SET status = 'sending' WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": format!("the campaign status could not be updated: {err}") })))?;
    if mark_sending.rows_affected() != 1 {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": format!("UPDATE broadcasts SET status='sending' affected {} rows", mark_sending.rows_affected()) }),
        ));
    }

    let mut warnings: Vec<String> = Vec::new();
    let mut sent = 0_i64;
    let mut failed = 0_i64;
    let mut skipped = 0_i64;
    let mut outcomes: Vec<Value> = Vec::new();

    for contact in &audience {
        let recipient_id = Uuid::new_v4().simple().to_string();
        let channel = contact.channel.trim().to_lowercase();

        if contact.value.trim().is_empty() {
            let reason = "the contact has no destination address on record".to_string();
            record_recipient(state, &recipient_id, id, contact, None, "not_sent", None, &reason).await;
            skipped += 1;
            outcomes.push(json!({ "contactId": contact.id, "status": "not_sent", "reason": reason }));
            continue;
        }
        if template_ready && channel != "whatsapp" {
            let reason = format!(
                "this campaign is a WhatsApp provider template; contact {} is on channel '{channel}'",
                contact.id
            );
            record_recipient(state, &recipient_id, id, contact, None, "not_sent", None, &reason).await;
            skipped += 1;
            outcomes.push(json!({ "contactId": contact.id, "status": "not_sent", "reason": reason }));
            continue;
        }
        if !channels_needed.contains(&channel) {
            let reason = if template_ready {
                format!("skipped: channel '{channel}' cannot carry a WhatsApp template")
            } else {
                format!("skipped: no outbound adapter is configured for channel '{channel}'")
            };
            record_recipient(state, &recipient_id, id, contact, None, "not_sent", None, &reason).await;
            skipped += 1;
            outcomes.push(json!({ "contactId": contact.id, "status": "not_sent", "reason": reason }));
            continue;
        }

        let body = match fill_placeholders(preview, contact, &params) {
            Ok(text) => text,
            Err(reason) => {
                record_recipient(state, &recipient_id, id, contact, None, "not_sent", None, &reason).await;
                skipped += 1;
                outcomes.push(json!({ "contactId": contact.id, "status": "not_sent", "reason": reason }));
                continue;
            }
        };
        let spec = if let Some((name, language)) = &template_spec {
            match build_body_params(preview, contact, &params) {
                Ok(body_params) => Some(channels::TemplateSpec {
                    name: name.clone(),
                    language: language.clone(),
                    body_params,
                }),
                Err(reason) => {
                    record_recipient(state, &recipient_id, id, contact, None, "not_sent", None, &reason).await;
                    skipped += 1;
                    outcomes.push(json!({ "contactId": contact.id, "status": "not_sent", "reason": reason }));
                    continue;
                }
            }
        } else {
            None
        };

        let conversation_id = match ensure_conversation(state, id, contact, &channel).await {
            Ok(conv_id) => conv_id,
            Err(reason) => {
                warnings.push(reason.clone());
                record_recipient(state, &recipient_id, id, contact, None, "not_sent", None, &reason).await;
                skipped += 1;
                outcomes.push(json!({ "contactId": contact.id, "status": "not_sent", "reason": reason }));
                continue;
            }
        };

        let request = messaging::OutboundRequest::new(
            &conversation_id,
            &contact.id,
            &contact.value,
            &channel,
            &body,
        )
        .tenant_key(Some(tenant_key))
        .sender("Campaign Broadcast", "system")
        .template(spec.as_ref());

        match messaging::send_outbound(state, &request).await {
            Ok(outcome) => {
                let result = &outcome.result;
                let (status, error) = match result.status {
                    messaging::SendStatus::Accepted => ("sent", None),
                    messaging::SendStatus::Rejected => ("failed", Some(result.reason())),
                    messaging::SendStatus::NotSent => ("not_sent", Some(result.reason())),
                };
                let provider_id = if result.provider_message_id.is_empty() {
                    None
                } else {
                    Some(result.provider_message_id.clone())
                };
                record_recipient(
                    state,
                    &recipient_id,
                    id,
                    contact,
                    Some(outcome.message_id.clone()),
                    status,
                    provider_id.as_deref(),
                    error.as_deref().unwrap_or_default(),
                )
                .await;
                match status {
                    "sent" => sent += 1,
                    "failed" => failed += 1,
                    _ => skipped += 1,
                }
                outcomes.push(json!({
                    "contactId": contact.id,
                    "messageId": outcome.message_id,
                    "status": status,
                    "provider": result.provider,
                    "httpStatus": result.http_status,
                    "reason": error,
                }));
            }
            Err(err) => {
                failed += 1;
                let reason = err.to_string();
                record_recipient(state, &recipient_id, id, contact, None, "failed", None, &reason).await;
                outcomes.push(json!({ "contactId": contact.id, "status": "failed", "reason": reason }));
            }
        }
    }

    // Delivered and read are whatever the ledger already holds; only a webhook
    // receipt can move them, never this function.
    let delivered = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM broadcast_recipients WHERE broadcast_id = $1 AND delivered_at IS NOT NULL",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await;
    let read = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM broadcast_recipients WHERE broadcast_id = $1 AND read_at IS NOT NULL",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await;

    // Nothing went out at all, so the campaign is not "completed" in any sense
    // an operator would accept.
    let final_status = if sent > 0 { "completed" } else { "failed" };
    let audience_count = audience.len() as i32;

    let counters = match (delivered, read) {
        (Ok(delivered), Ok(read)) => Some((delivered as i32, read as i32)),
        (delivered, read) => {
            for (label, outcome) in [("delivered", delivered), ("read", read)] {
                if let Err(err) = outcome {
                    warnings.push(format!(
                        "the {label} count could not be read from the recipient ledger: {err}"
                    ));
                }
            }
            None
        }
    };

    let update = match counters {
        Some((delivered, read)) => sqlx::query(
            "UPDATE broadcasts SET status = $1, audience_count = $2, total_recipients = $3, \
             sent_count = $4, delivered_count = $5, read_count = $6, failed_count = $7, skipped_count = $8 \
             WHERE id = $9",
        )
        .bind(final_status)
        .bind(audience_count)
        .bind(audience_count)
        .bind(sent as i32)
        .bind(delivered)
        .bind(read)
        .bind(failed as i32)
        .bind(skipped as i32)
        .bind(id)
        .execute(&state.db)
        .await
        .map(|report| (report.rows_affected(), Some(delivered), Some(read))),
        None => sqlx::query(
            "UPDATE broadcasts SET status = $1, audience_count = $2, total_recipients = $3, \
             sent_count = $4, failed_count = $5, skipped_count = $6 WHERE id = $7",
        )
        .bind(final_status)
        .bind(audience_count)
        .bind(audience_count)
        .bind(sent as i32)
        .bind(failed as i32)
        .bind(skipped as i32)
        .bind(id)
        .execute(&state.db)
        .await
        .map(|report| (report.rows_affected(), None::<i32>, None::<i32>)),
    };

    let (rows, delivered_stored, read_stored) = match update {
        Ok(values) => values,
        Err(err) => {
            warnings.push(format!("the campaign counters could not be stored: {err}"));
            (0, None, None)
        }
    };
    if rows != 1 {
        warnings.push(format!(
            "the campaign counters were NOT stored: the UPDATE affected {rows} rows."
        ));
    }

    warnings.push(
        "delivered and read come from webhook receipts already stored; Meta reports them after \
         the send."
            .to_string(),
    );
    if sent == 0 {
        warnings.push(format!(
            "no message reached a provider for this campaign ({skipped} skipped, {failed} refused); \
             status was set to 'failed'."
        ));
    }

    Ok(json!({
        "id": id,
        "status": final_status,
        "audienceCount": audience_count,
        "audience_count": audience_count,
        "totalRecipients": audience_count,
        "sentCount": sent,
        "sent_count": sent,
        "deliveredCount": delivered_stored,
        "readCount": read_stored,
        "failedCount": failed,
        "failed_count": failed,
        "skippedCount": skipped,
        "skipped_count": skipped,
        "templateUsed": template_ready,
        "recipients": outcomes,
        "warnings": warnings,
    }))
}

async fn record_recipient(
    state: &AppState,
    recipient_id: &str,
    broadcast_id: &str,
    contact: &AudienceRow,
    message_id: Option<String>,
    status: &str,
    provider_message_id: Option<&str>,
    error: &str,
) {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO broadcast_recipients (id, broadcast_id, contact_id, status, \
         provider_message_id, error, sent_at, delivered_at, read_at, message_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8)",
    )
    .bind(recipient_id)
    .bind(broadcast_id)
    .bind(&contact.id)
    .bind(status)
    .bind(provider_message_id)
    .bind(if error.is_empty() { None } else { Some(error) })
    .bind(if status == "sent" { Some(now) } else { None })
    .bind(message_id)
    .execute(&state.db)
    .await;
    match result {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => tracing::error!(
            "[BROADCASTS] recipient row for contact {} wrote {} rows (expected 1)",
            contact.id,
            report.rows_affected()
        ),
        Err(err) => tracing::error!("[BROADCASTS] recipient row for contact {} failed: {err}", contact.id),
    }
}

async fn ensure_conversation(
    state: &AppState,
    broadcast_id: &str,
    contact: &AudienceRow,
    channel: &str,
) -> Result<String, String> {
    let now = Utc::now().to_rfc3339();
    let existing = sqlx::query("SELECT id FROM conversations WHERE contact_id = $1 ORDER BY updated_at DESC LIMIT 1")
        .bind(&contact.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| format!("the conversation for contact {} could not be read: {err}", contact.id))?;

    if let Some(row) = existing {
        let conversation_id: String = row
            .try_get("id")
            .map_err(|err| format!("the conversation id for contact {} is unreadable: {err}", contact.id))?;
        let updated = sqlx::query("UPDATE conversations SET broadcast_id = $1, updated_at = $2 WHERE id = $3")
            .bind(broadcast_id)
            .bind(&now)
            .bind(&conversation_id)
            .execute(&state.db)
            .await
            .map_err(|err| format!("the conversation could not be linked to the campaign: {err}"))?;
        if updated.rows_affected() != 1 {
            return Err(format!(
                "conversation {conversation_id} was not linked to campaign {broadcast_id} ({} rows affected)",
                updated.rows_affected()
            ));
        }
        return Ok(conversation_id);
    }

    let conversation_id = Uuid::new_v4().simple().to_string();
    let inserted = sqlx::query(
        "INSERT INTO conversations (id, contact_id, contact_name, channel, state, broadcast_id, \
         human_takeover, created_at, updated_at) VALUES ($1, $2, $3, $4, 'ENGAGED', $5, 0, $6, $7)",
    )
    .bind(&conversation_id)
    .bind(&contact.id)
    .bind(contact.person_name.as_deref())
    .bind(channel)
    .bind(broadcast_id)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| format!("no conversation could be opened for contact {}: {err}", contact.id))?;
    if inserted.rows_affected() != 1 {
        return Err(format!(
            "the conversation for contact {} was not created ({} rows affected)",
            contact.id,
            inserted.rows_affected()
        ));
    }
    Ok(conversation_id)
}

pub async fn dispatch_broadcast(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    match run_dispatch(&state, &tenant_key, &id).await {
        Ok(body) => (StatusCode::OK, Json(body)).into_response(),
        Err((status, body)) => (status, Json(body)).into_response(),
    }
}

/// Cron entry point: there is no request header to carry a tenant, so it runs
/// the tenant the deployment configured. Nothing is assumed when it is unset.
pub async fn process_scheduled_campaigns(State(state): State<AppState>) -> Response {
    let tenant_key = state.config.default_tenant_key.trim().to_string();
    if tenant_key.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "DEFAULT_TENANT_KEY is not configured, so the scheduler does not know \
                          which tenant's campaigns to run. Nothing was dispatched.",
                "processed": 0,
            })),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    let due = match sqlx::query_as::<_, BroadcastRow>(&format!(
        "SELECT {BROADCAST_FIELDS} FROM broadcasts \
         WHERE status = 'scheduled' AND COALESCE(tenant_key, '') = $1 \
         AND scheduled_at IS NOT NULL AND scheduled_at <> '' AND scheduled_at <= $2"
    ))
    .bind(&tenant_key)
    .bind(&now)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("the scheduled campaigns could not be read", err),
    };

    let mut results = Vec::new();
    let mut dispatched = 0_usize;
    for campaign in &due {
        match run_dispatch(&state, &tenant_key, &campaign.id).await {
            Ok(body) => {
                dispatched += 1;
                results.push(body);
            }
            Err((status, body)) => results.push(json!({
                "id": campaign.id,
                "status": "not_dispatched",
                "httpStatus": status.as_u16(),
                "error": body.get("error").cloned().unwrap_or(Value::Null),
            })),
        }
    }

    Json(json!({
        "processed": dispatched,
        "due": due.len(),
        "tenantKey": tenant_key,
        "campaigns": results,
    }))
    .into_response()
}

pub async fn list_broadcast_recipients(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    let owned = sqlx::query("SELECT 1 FROM broadcasts WHERE id = $1 AND COALESCE(tenant_key, '') = $2")
        .bind(&id)
        .bind(&tenant_key)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| db_error("the campaign could not be read", err));
    match owned {
        Err(response) => return response,
        Ok(None) => {
            return refuse(
                StatusCode::NOT_FOUND,
                &format!("campaign {id} does not exist for tenant '{tenant_key}'."),
                json!({ "campaignId": id }),
            )
        }
        Ok(Some(_)) => {}
    }

    let recipients = match sqlx::query_as::<_, RecipientDetailRow>(RECIPIENT_SELECT)
        .bind(&id)
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("the recipient ledger could not be read", err),
    };

    Json(recipients).into_response()
}

#[derive(Deserialize)]
pub struct PricingQuery {
    pub region: Option<String>,
    pub category: Option<String>,
    pub volume: Option<i64>,
}

/// Meta does not expose per-conversation rates through the messages API, so the
/// rate has to be the operator's own billing figure. It is read from
/// `whatsapp_pricing_rates`; when nothing is configured the endpoint refuses
/// instead of quoting a made-up number or an invented ROI multiple.
pub async fn get_whatsapp_pricing(State(state): State<AppState>, Query(query): Query<PricingQuery>) -> Response {
    let region = match query
        .region
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
    {
        Some(value) => value,
        None => {
            return refuse(
                StatusCode::BAD_REQUEST,
                "`region` is required; there is no default destination country.",
                json!({ "field": "region" }),
            )
        }
    };
    let category = match query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
    {
        Some(value) => value,
        None => {
            return refuse(
                StatusCode::BAD_REQUEST,
                "`category` is required (marketing, utility, authentication or service).",
                json!({ "field": "category" }),
            )
        }
    };
    let volume = match query.volume {
        Some(value) if value > 0 => value,
        _ => {
            return refuse(
                StatusCode::BAD_REQUEST,
                "`volume` must be a positive number of conversations.",
                json!({ "field": "volume" }),
            )
        }
    };

    let rate = sqlx::query(
        "SELECT rate_per_conversation, currency, source, updated_at FROM whatsapp_pricing_rates \
         WHERE region = $1 AND category = $2",
    )
    .bind(&region)
    .bind(&category)
    .fetch_optional(&state.db)
    .await;

    let row = match rate {
        Ok(Some(row)) => row,
        Ok(None) => {
            let configured = sqlx::query("SELECT region, category FROM whatsapp_pricing_rates")
                .fetch_all(&state.db)
                .await
                .unwrap_or_default();
            let available: Vec<Value> = configured
                .iter()
                .filter_map(|r| {
                    let region: String = r.try_get("region").ok()?;
                    let category: String = r.try_get("category").ok()?;
                    Some(json!({ "region": region, "category": category }))
                })
                .collect();
            return refuse(
                StatusCode::SERVICE_UNAVAILABLE,
                &format!("no rate is configured for region '{region}' and category '{category}'."),
                json!({
                    "configuredRates": available,
                    "howToSet": "INSERT INTO whatsapp_pricing_rates (region, category, currency, \
                                 rate_per_conversation, source, updated_at) VALUES (...); use the \
                                 figure on your Meta WhatsApp Business Platform invoice.",
                }),
            );
        }
        Err(err) => return db_error("the pricing table could not be read", err),
    };

    let rate_per_conversation: f64 = match row.try_get("rate_per_conversation") {
        Ok(value) => value,
        Err(err) => return db_error("the stored rate is unreadable", sqlx::Error::Decode(Box::new(err))),
    };
    let currency: String = match row.try_get("currency") {
        Ok(value) => value,
        Err(err) => return db_error("the stored currency is unreadable", sqlx::Error::Decode(Box::new(err))),
    };
    let source: String = match row.try_get("source") {
        Ok(value) => value,
        Err(err) => return db_error("the stored rate source is unreadable", sqlx::Error::Decode(Box::new(err))),
    };
    let updated_at: Option<String> = match row.try_get("updated_at") {
        Ok(value) => value,
        Err(err) => return db_error("the stored rate timestamp is unreadable", sqlx::Error::Decode(Box::new(err))),
    };

    let estimated_cost = (volume as f64 * rate_per_conversation * 100.0).round() / 100.0;
    // The cost is only expressible in USD when the row says it is USD. Anything
    // else is reported as the raw figure plus the reason it was not converted.
    let mut warnings = Vec::new();
    if currency != "USD" {
        warnings.push(format!(
            "the stored rate is denominated in '{currency}', so no USD total is shown; the figure \
             in `estimatedCost` is {currency}."
        ));
    }
    if source.trim().is_empty() {
        warnings.push("`source` is blank on the stored rate row, so this figure cannot be traced \
                       back to a Meta invoice."
            .to_string());
    }

    Json(json!({
        "region": region,
        "category": category,
        "volume": volume,
        "ratePerConversation": rate_per_conversation,
        "rate_per_conversation": rate_per_conversation,
        "ratePerMessage": Value::Null,
        "estimatedCostUsd": if currency == "USD" { json!(estimated_cost) } else { Value::Null },
        "estimatedCost": json!(estimated_cost),
        "currency": currency,
        "rateSource": source,
        "rateUpdatedAt": updated_at,
        "estimatedRoiUsd": Value::Null,
        "roiNote": "ROI is not computed: it needs conversion and revenue data this deployment \
                    has not measured.",
        "warnings": warnings,
        "supportedRegions": configured_regions(&state).await,
    }))
    .into_response()
}

async fn configured_regions(state: &AppState) -> Vec<Value> {
    let rows = sqlx::query("SELECT DISTINCT region FROM whatsapp_pricing_rates ORDER BY region")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    rows.iter()
        .filter_map(|r| r.try_get::<String, _>("region").ok())
        .map(|region| json!({ "code": region, "name": region }))
        .collect()
}
