//! Typed access to the `integration_settings` table -- the Connections page's
//! backing store.
//!
//! Before this module every caller re-implemented the same SQL and the same
//! "DB value wins, else fall back to the process environment" rule, each with
//! slightly different column lists. The rules live here once:
//!
//! * one row per tenant (`tenant_key`), default tenant is the one the product
//!   ships with and is *not* invented here -- it comes from
//!   `Config::default_tenant_key` when set, otherwise the caller must pass one;
//! * a stored value is only usable when it is non-empty and not a display mask
//!   (`••••1234` is not a credential -- see `meta_sync.rs`);
//! * writes go through a column whitelist, so a webhook payload can never make
//!   `integration_settings` the target of a `SQL`-shaped field name.

use crate::config::Config;
use crate::db::DbPool;
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;

/// Columns this module is allowed to write, mapped to the SQL type each binding
/// needs. Anything else in a patch is reported as rejected instead of ignored
/// silently.
enum ColumnType {
    Text,
    Int,
    Bool,
}

fn writable_columns() -> HashMap<&'static str, ColumnType> {
    let mut m = HashMap::new();
    for (name, kind) in [
        ("whatsapp_provider", ColumnType::Text),
        ("meta_access_token", ColumnType::Text),
        ("meta_phone_number_id", ColumnType::Text),
        ("meta_waba_id", ColumnType::Text),
        ("meta_app_secret", ColumnType::Text),
        ("meta_webhook_verify_token", ColumnType::Text),
        ("whatsapp_verified_at", ColumnType::Text),
        ("whatsapp_verified_name", ColumnType::Text),
        ("whatsapp_display_number", ColumnType::Text),
        ("business_name", ColumnType::Text),
        ("business_description", ColumnType::Text),
        ("agent_guidelines", ColumnType::Text),
        ("auto_reply_enabled", ColumnType::Bool),
        ("business_hours_enabled", ColumnType::Bool),
        ("business_hours_start", ColumnType::Int),
        ("business_hours_end", ColumnType::Int),
        ("business_timezone", ColumnType::Text),
        ("away_message", ColumnType::Text),
    ] {
        m.insert(name, kind);
    }
    m
}

/// A stored credential/display value is only real when it is non-empty and not
/// the masked form the UI shows back.
pub fn usable(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !trimmed.chars().any(|c| c == '•') && !trimmed.starts_with("•")
}

fn text(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn flag(row: &sqlx::sqlite::SqliteRow, column: &str) -> bool {
    row.try_get::<Option<bool>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(false)
        || row
            .try_get::<Option<i64>, _>(column)
            .ok()
            .flatten()
            .unwrap_or(0)
            != 0
}

fn number(row: &sqlx::sqlite::SqliteRow, column: &str, default: i64) -> i64 {
    row.try_get::<Option<i64>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(default)
}

const SELECT_COLUMNS: &str = "id, tenant_key, whatsapp_provider, meta_access_token, \
     meta_phone_number_id, meta_waba_id, meta_app_secret, meta_webhook_verify_token, \
     whatsapp_verified_at, whatsapp_verified_name, whatsapp_display_number, business_name, \
     business_description, agent_guidelines, auto_reply_enabled, business_hours_enabled, \
     business_hours_start, business_hours_end, business_timezone, away_message, updated_at";

#[derive(Debug, Clone)]
pub struct IntegrationSettings {
    pub id: String,
    pub tenant_key: String,
    pub whatsapp_provider: String,
    pub meta_access_token: String,
    pub meta_phone_number_id: String,
    pub meta_waba_id: String,
    pub meta_app_secret: String,
    pub meta_webhook_verify_token: String,
    pub whatsapp_verified_at: String,
    pub whatsapp_verified_name: String,
    pub whatsapp_display_number: String,
    pub business_name: String,
    pub business_description: String,
    pub agent_guidelines: String,
    pub auto_reply_enabled: bool,
    pub business_hours_enabled: bool,
    pub business_hours_start: i64,
    pub business_hours_end: i64,
    pub business_timezone: String,
    pub away_message: String,
    pub updated_at: String,
}

impl IntegrationSettings {
    /// Row with nothing in it: used when the table has no row for the tenant
    /// and creating one would be a write on a read path. `is_default` lets
    /// callers tell "the admin never opened the Connections page" apart from
    /// "the admin configured it", which the reply engine and the business-hours
    /// gate both need.
    pub fn empty(tenant_key: &str) -> Self {
        Self {
            id: String::new(),
            tenant_key: tenant_key.to_string(),
            whatsapp_provider: String::new(),
            meta_access_token: String::new(),
            meta_phone_number_id: String::new(),
            meta_waba_id: String::new(),
            meta_app_secret: String::new(),
            meta_webhook_verify_token: String::new(),
            whatsapp_verified_at: String::new(),
            whatsapp_verified_name: String::new(),
            whatsapp_display_number: String::new(),
            business_name: String::new(),
            business_description: String::new(),
            agent_guidelines: String::new(),
            auto_reply_enabled: false,
            business_hours_enabled: false,
            business_hours_start: -1,
            business_hours_end: -1,
            business_timezone: String::new(),
            away_message: String::new(),
            updated_at: String::new(),
        }
    }

    pub fn is_default(&self) -> bool {
        self.id.is_empty()
    }

    /// WhatsApp credentials resolved field-by-field: the row wins when it holds
    /// a usable value, otherwise the process environment is used. Never a
    /// compiled-in token.
    pub fn whatsapp_credentials(&self, config: &Config) -> WhatsAppCredentials {
        WhatsAppCredentials {
            provider: first_usable(&[
                &self.whatsapp_provider,
                "meta", // transport default, not a credential
            ]),
            access_token: first_usable(&[&self.meta_access_token, &config.meta_whatsapp_access_token]),
            phone_number_id: first_usable(&[
                &self.meta_phone_number_id,
                &config.meta_whatsapp_phone_number_id,
            ]),
            waba_id: self.meta_waba_id.clone(),
            app_secret: first_usable(&[&self.meta_app_secret, &config.meta_app_secret]),
            webhook_verify_token: first_usable(&[
                &self.meta_webhook_verify_token,
                &config.meta_webhook_verify_token,
            ]),
            api_version: first_usable(&[&config.meta_whatsapp_api_version]),
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.id,
            "tenantKey": self.tenant_key,
            "whatsappProvider": self.whatsapp_provider,
            "metaAccessTokenPresent": usable(&self.meta_access_token),
            "metaPhoneNumberId": self.meta_phone_number_id,
            "metaWabaId": self.meta_waba_id,
            "whatsappDisplayNumber": self.whatsapp_display_number,
            "whatsappVerifiedName": self.whatsapp_verified_name,
            "whatsappVerifiedAt": self.whatsapp_verified_at,
            "businessName": self.business_name,
            "businessDescription": self.business_description,
            "agentGuidelines": self.agent_guidelines,
            "autoReplyEnabled": self.auto_reply_enabled,
            "businessHoursEnabled": self.business_hours_enabled,
            "businessHoursStart": self.business_hours_start,
            "businessHoursEnd": self.business_hours_end,
            "businessTimezone": self.business_timezone,
            "awayMessage": self.away_message,
            "updatedAt": self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WhatsAppCredentials {
    pub provider: String,
    pub access_token: String,
    pub phone_number_id: String,
    pub waba_id: String,
    pub app_secret: String,
    pub webhook_verify_token: String,
    pub api_version: String,
}

fn first_usable(values: &[&str]) -> String {
    for value in values {
        if usable(value) {
            return value.trim().to_string();
        }
    }
    String::new()
}

fn row_to_settings(row: &sqlx::sqlite::SqliteRow) -> IntegrationSettings {
    IntegrationSettings {
        id: text(row, "id"),
        tenant_key: text(row, "tenant_key"),
        whatsapp_provider: text(row, "whatsapp_provider"),
        meta_access_token: text(row, "meta_access_token"),
        meta_phone_number_id: text(row, "meta_phone_number_id"),
        meta_waba_id: text(row, "meta_waba_id"),
        meta_app_secret: text(row, "meta_app_secret"),
        meta_webhook_verify_token: text(row, "meta_webhook_verify_token"),
        whatsapp_verified_at: text(row, "whatsapp_verified_at"),
        whatsapp_verified_name: text(row, "whatsapp_verified_name"),
        whatsapp_display_number: text(row, "whatsapp_display_number"),
        business_name: text(row, "business_name"),
        business_description: text(row, "business_description"),
        agent_guidelines: text(row, "agent_guidelines"),
        auto_reply_enabled: flag(row, "auto_reply_enabled"),
        business_hours_enabled: flag(row, "business_hours_enabled"),
        business_hours_start: number(row, "business_hours_start", -1),
        business_hours_end: number(row, "business_hours_end", -1),
        business_timezone: text(row, "business_timezone"),
        away_message: text(row, "away_message"),
        updated_at: text(row, "updated_at"),
    }
}

/// Read one tenant's row. `Ok(None)` means the Connections page has never been
/// saved for this tenant -- callers that must not write use this instead of
/// `get_or_create`.
pub async fn load(pool: &DbPool, tenant_key: &str) -> Result<Option<IntegrationSettings>, sqlx::Error> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM integration_settings WHERE tenant_key = ? LIMIT 1");
    let row = sqlx::query(&sql)
        .bind(tenant_key)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(row_to_settings))
}

/// Read the only row a single-tenant deployment has, when the caller does not
/// know (or has not configured) a tenant key. Deliberately not a `load`
/// fallback that fabricates anything: `Ok(None)` still means "the Connections
/// page was never saved", which is exactly what the outbound channels report.
pub async fn load_any(pool: &DbPool) -> Result<Option<IntegrationSettings>, sqlx::Error> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM integration_settings ORDER BY updated_at DESC LIMIT 1");
    let row = sqlx::query(&sql).fetch_optional(pool).await?;
    Ok(row.as_ref().map(row_to_settings))
}

/// Read the tenant's row, creating it from the process environment when absent
/// (the behaviour the Python `get_settings_row` has). Created rows carry only
/// credentials the environment actually provides: an unset env var stays empty
/// so a later send reports it as missing rather than as configured.
pub async fn get_or_create(
    pool: &DbPool,
    config: &Config,
    tenant_key: &str,
) -> Result<IntegrationSettings, sqlx::Error> {
    if let Some(existing) = load(pool, tenant_key).await? {
        return Ok(existing);
    }

    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO integration_settings (id, tenant_key, whatsapp_provider, meta_access_token, \
         meta_phone_number_id, meta_app_secret, meta_webhook_verify_token, business_timezone, \
         updated_at) VALUES (?, ?, 'meta', ?, ?, ?, ?, 'UTC', ?)",
    )
    .bind(&id)
    .bind(tenant_key)
    .bind(&config.meta_whatsapp_access_token)
    .bind(&config.meta_whatsapp_phone_number_id)
    .bind(&config.meta_app_secret)
    .bind(&config.meta_webhook_verify_token)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(load(pool, tenant_key)
        .await?
        .unwrap_or_else(|| IntegrationSettings::empty(tenant_key)))
}

/// Column-whitelisted patch write. Returns the columns actually updated plus
/// the keys that were refused, so the route can tell the operator which fields
/// it ignored instead of dropping them silently.
pub async fn update(
    pool: &DbPool,
    tenant_key: &str,
    patch: &Value,
) -> Result<UpdateReport, sqlx::Error> {
    let columns = writable_columns();
    let mut sets: Vec<String> = Vec::new();
    let mut applied: Vec<String> = Vec::new();
    let mut rejected: Vec<String> = Vec::new();
    let mut object: HashMap<&str, &Value> = HashMap::new();

    if let Some(map) = patch.as_object() {
        for (key, value) in map {
            object.insert(key.as_str(), value);
        }
    }

    // Sorted so the built SQL (and its binding order) is deterministic.
    let mut keys: Vec<&str> = object.keys().copied().collect();
    keys.sort_unstable();

    let mut binds_text: Vec<String> = Vec::new();
    let mut binds_int: Vec<i64> = Vec::new();
    let mut binds_bool: Vec<bool> = Vec::new();

    for key in keys {
        match columns.get(key) {
            Some(ColumnType::Text) => {
                let value = object[key].as_str().unwrap_or_default().to_string();
                sets.push(format!("{key} = ?"));
                binds_text.push(value);
                applied.push(key.to_string());
            }
            Some(ColumnType::Int) => {
                let value = object[key]
                    .as_i64()
                    .or_else(|| object[key].as_str().and_then(|s| s.parse().ok()))
                    .unwrap_or(0);
                sets.push(format!("{key} = ?"));
                binds_int.push(value);
                applied.push(key.to_string());
            }
            Some(ColumnType::Bool) => {
                let raw = object[key];
                let value = match raw {
                    Value::Bool(flag) => *flag,
                    Value::Number(num) => num.as_i64().map(|v| v != 0).unwrap_or(false),
                    Value::String(text) => matches!(
                        text.to_ascii_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    ),
                    _ => false,
                };
                sets.push(format!("{key} = ?"));
                binds_bool.push(value);
                applied.push(key.to_string());
            }
            None => rejected.push(key.to_string()),
        }
    }

    if sets.is_empty() {
        return Ok(UpdateReport {
            rows_affected: 0,
            applied,
            rejected,
        });
    }

    sets.push("updated_at = ?".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let sql = format!(
        "UPDATE integration_settings SET {} WHERE tenant_key = ?",
        sets.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for value in binds_text {
        query = query.bind(value);
    }
    for value in binds_int {
        query = query.bind(value);
    }
    for value in binds_bool {
        query = query.bind(value);
    }
    query = query.bind(now).bind(tenant_key);

    let result = query.execute(pool).await?;
    Ok(UpdateReport {
        rows_affected: result.rows_affected(),
        applied,
        rejected,
    })
}

#[derive(Debug, Clone)]
pub struct UpdateReport {
    pub rows_affected: u64,
    pub applied: Vec<String>,
    pub rejected: Vec<String>,
}

/// Business-hours / away-message inputs are integers and strings the operator
/// typed; validate them here so every write path shares the same rules.
pub fn validate_business_hours(patch: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let get = |key: &str| patch.get(key).and_then(|v| v.as_i64());
    if let Some(start) = get("business_hours_start") {
        if !(0..=23).contains(&start) {
            problems.push(format!("business_hours_start must be 0-23, got {start}"));
        }
    }
    if let Some(end) = get("business_hours_end") {
        if !(0..=24).contains(&end) {
            problems.push(format!("business_hours_end must be 0-24, got {end}"));
        }
    }
    if let (Some(start), Some(end)) = (get("business_hours_start"), get("business_hours_end")) {
        if start == end {
            problems.push(format!(
                "business hours start and end are both {start}: the window is empty, \
                 every message would be treated as out-of-hours"
            ));
        }
    }
    problems
}
