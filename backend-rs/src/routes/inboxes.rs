#![allow(dead_code)]

use crate::{
    db::{AppState, DbPool},
    models::ChannelHealth,
    services::inbound,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde_json::{json, Map, Value};
use sqlx::Row;
use tracing::warn;
use uuid::Uuid;

// ---------------------------------------------------------
// Honesty helpers
//
// Nothing below this block may invent a value. A field is only ever reported
// when it was actually read from the inbox credentials JSON, the
// `integration_settings` row, the process environment, or a live provider
// response. Otherwise it is `null` and the missing keys are named.
// ---------------------------------------------------------

/// A masked secret (contains the bullet character) was never a usable
/// credential; treat it as absent, exactly like `meta_sync` does.
fn usable(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() || trimmed.contains('\u{2022}') {
        return None;
    }
    Some(trimmed.to_string())
}

fn json_str(value: Option<&Value>) -> Option<String> {
    usable(value.and_then(|v| v.as_str()))
}

fn cred(map: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| json_str(map.get(*k)))
}

fn cred_bool(map: &Map<String, Value>, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|k| map.get(*k).and_then(|v| v.as_bool()))
}

fn cred_i64(map: &Map<String, Value>, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|k| map.get(*k).and_then(|v| v.as_i64()))
}

fn cred_arr(map: &Map<String, Value>, keys: &[&str]) -> Option<Value> {
    keys
        .iter()
        .find_map(|k| map.get(*k))
        .and_then(|v| v.as_array().cloned())
        .map(Value::Array)
}

fn config_val(raw: String) -> Option<String> {
    usable(Some(&raw))
}

/// TikTok outbound calls need an access token. `tiktok_app_secret` only signs
/// webhooks and is not a token, so the config field — which already folds in
/// the `TIKTOK_ACCESS_TOKEN` env var and any vault secret applied at startup —
/// is the only real source.
fn tiktok_access_token(state: &AppState) -> Option<String> {
    usable(Some(state.config.tiktok_access_token.as_str()))
}

fn value_or_null(raw: Option<String>) -> Value {
    match raw {
        Some(v) => Value::String(v),
        None => Value::Null,
    }
}

/// Only these columns may be interpolated into the settings read; anything
/// else is refused so a caller cannot steer the SQL.
const SETTING_COLUMNS: &[&str] = &[
    "meta_access_token",
    "meta_phone_number_id",
    "meta_waba_id",
    "meta_app_secret",
    "meta_webhook_verify_token",
    "whatsapp_verified_at",
    "whatsapp_verified_name",
    "whatsapp_display_number",
    "business_name",
];

/// `Ok(Some(v))` means the column was read and held a usable value.
/// `Ok(None)` means the operator has not configured it.
/// `Err` means the row or column could not be read at all, which is a different
/// fact and must never be reported as "not configured".
async fn setting_str(state: &AppState, column: &str) -> Result<Option<String>, sqlx::Error> {
    if !SETTING_COLUMNS.contains(&column) {
        return Err(sqlx::Error::Protocol(format!(
            "refused to read integration_settings column {column}: it is not on the read whitelist"
        )));
    }
    let sql = format!("SELECT CAST({column} AS TEXT) AS value FROM integration_settings LIMIT 1");
    let Some(row) = sqlx::query(&sql).fetch_optional(&state.db).await? else {
        return Ok(None);
    };
    let value = row.try_get::<Option<String>, _>("value")?;
    Ok(value.and_then(|v| usable(Some(v.as_str()))))
}

/// Webhook/embed URLs must be configured; there is no sane default domain.
fn public_base_url(state: &AppState) -> Option<String> {
    usable(Some(&state.config.public_url))
        .or_else(|| {
            ["PUBLIC_BASE_URL", "APP_BASE_URL", "FRONTEND_BASE_URL"]
                .iter()
                .find_map(|key| std::env::var(key).ok())
                .and_then(|v| usable(Some(&v)))
        })
        .map(|mut url| {
            while url.ends_with('/') {
                url.pop();
            }
            url
        })
}

fn missing_text(channel: &str, keys: &[&str]) -> String {
    format!(
        "{channel} is not configured: no real value found for [{}]. Set it on the inbox credentials, integration_settings, or the process environment.",
        keys.join(", ")
    )
}

/// A refusal that would otherwise read "nothing is stored" when the stored blob
/// is what could not be read.
fn with_cred_notes(text: String, warnings: &[String]) -> String {
    if warnings.is_empty() {
        text
    } else {
        format!("{text} Note: {}", warnings.join("; "))
    }
}

/// `is_active` only tells us the row is enabled. Without a live provider read
/// the most we can honestly say is that credentials are present.
fn honest_label(is_active: bool, configured: bool) -> &'static str {
    match (is_active, configured) {
        (false, _) => "Inbox disabled",
        (true, false) => "Not configured",
        (true, true) => "Credentials present - not verified against the provider",
    }
}


fn err_response(status: StatusCode, message: &str) -> Response {
    warn!("{status} {message}");
    (status, Json(json!({ "error": message }))).into_response()
}

/// The SELECT that `query_inbox_by_channels_checked` runs, quoted in failures so
/// an unreadable table names the query that broke.
const INBOX_LOOKUP_QUERY: &str =
    "SELECT ... FROM inboxes WHERE channel_type IN (?) ORDER BY is_active DESC, updated_at DESC LIMIT 1";

const MESSAGE_EVENTS_QUERY: &str =
    "SELECT ... FROM messages WHERE channel IN (?) ORDER BY sent_at DESC LIMIT 50";

const GATEWAY_LOOKUP_QUERY: &str =
    "SELECT ... FROM channel_gateway_connections WHERE gateway_type = ? AND is_active = 1 LIMIT 1";

/// A read that failed is a different fact from a read that returned no rows.
/// This is the wording the operator gets: the table, the query and the driver's
/// own text, with no implied "empty".
fn read_failure(table: &str, query: &str, err: &sqlx::Error) -> String {
    format!("Could not read {table}: the query `{query}` failed ({err}). This is a database failure, not an empty result set, so the real state is unknown.")
}

fn read_failure_response(table: &str, query: &str, err: &sqlx::Error) -> Response {
    err_response(StatusCode::INTERNAL_SERVER_ERROR, &read_failure(table, query, err))
}

/// Names the `integration_settings` column that could not be read, for responses
/// that keep serving the rest of their data.
fn settings_read_failure(column: &str, err: &sqlx::Error) -> String {
    format!(
        "The integration_settings column {column} could not be read ({err}); values that fall back to it are unknown, not unset."
    )
}

/// The integration_settings read, in the shape a view handler that keeps serving
/// partial data can use: the value plus a warning naming the column when the
/// read itself failed, so `None` from a broken table never reads as "unset".
async fn setting_opt(state: &AppState, column: &str, warnings: &mut Vec<String>) -> Option<String> {
    match setting_str(state, column).await {
        Ok(value) => value,
        Err(err) => {
            warnings.push(settings_read_failure(column, &err));
            None
        }
    }
}

/// `resolve_one` in warning form: same short-circuit order, but a failed
/// settings read becomes a warning plus an unresolved value rather than a
/// silent "not configured".
async fn resolve_opt(
    state: &AppState,
    creds: &Map<String, Value>,
    cred_keys: &[&str],
    setting_column: Option<&str>,
    config_value: Option<String>,
    warnings: &mut Vec<String>,
) -> Option<String> {
    match resolve_one(state, creds, cred_keys, setting_column, config_value).await {
        Ok(value) => value,
        Err(err) => {
            warnings.push(settings_read_failure(
                setting_column.unwrap_or("integration_settings"),
                &err,
            ));
            None
        }
    }
}

/// The inbox row read for a view handler that keeps serving. `Ok`-shaped: the
/// row (or `None` when `inboxes` genuinely has no such row) plus a warning that
/// names the table and query when the read failed, so a broken table is never
/// reported as "channel not configured".
async fn inbox_row_for_view(
    pool: &DbPool,
    channel_types: &[&str],
) -> (Option<InboxRow>, Option<String>) {
    match query_inbox_by_channels_checked(pool, channel_types).await {
        Ok(row) => (row, None),
        Err(err) => (None, Some(read_failure("inboxes", INBOX_LOOKUP_QUERY, &err))),
    }
}

/// The active gateway row read for a view handler, in the same warning form.
async fn gateway_row_for_view(
    pool: &DbPool,
    gateway_type: &str,
) -> (Option<GatewayRow>, Option<String>) {
    match sqlx::query_as::<_, GatewayRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE gateway_type = ? AND is_active = 1 LIMIT 1",
    )
    .bind(gateway_type)
    .fetch_optional(pool)
    .await
    {
        Ok(row) => (row, None),
        Err(err) => (None, Some(read_failure("channel_gateway_connections", GATEWAY_LOOKUP_QUERY, &err))),
    }
}

fn request_str(payload: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| json_str(payload.get(*k)))
}

fn request_i64(payload: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|k| payload.get(*k).and_then(|v| v.as_i64()))
}

fn require_recipient(payload: &Value, keys: &[&str], channel: &str) -> Result<String, Response> {
    request_str(payload, keys)
        .map(|raw| raw.split_whitespace().collect::<Vec<_>>().join(""))
        .ok_or_else(|| {
            err_response(
                StatusCode::BAD_REQUEST,
                &format!("Refusing to send a {channel} test message: payload must include a real recipient (one of {}). No placeholder number is used.", keys.join(", ")),
            )
        })
}

fn require_body(payload: &Value, keys: &[&str]) -> Result<String, Response> {
    request_str(payload, keys).ok_or_else(|| {
        err_response(
            StatusCode::BAD_REQUEST,
            &format!("Refusing to send: payload must include a real message body (one of {}). No placeholder text is used.", keys.join(", ")),
        )
    })
}

/// `InboundError` does not implement `Display`, so render its own variants the
/// same way `routes/contacts.rs` and `services/widget_support.rs` do: the
/// contact-resolution message verbatim, or the driver's own error text.
/// Nothing here invents a reason.
fn inbound_error_detail(err: &inbound::InboundError) -> String {
    match err {
        inbound::InboundError::UnknownContact(msg) => msg.clone(),
        inbound::InboundError::Db(e) => e.to_string(),
    }
}

/// Outbound test messages are only meaningful if they are persisted; the
/// conversation and contact come from the same helpers the inbound pipeline
/// uses so IDs are never invented.
async fn ensure_outbound_conversation(
    pool: &DbPool,
    channel: &str,
    recipient: &str,
    tenant_key: &str,
) -> Result<String, Response> {
    let contact_id = inbound::get_or_create_contact(pool, channel, recipient, "", tenant_key)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!(
                    "Could not resolve a contact for {recipient} on {channel}: {}",
                    inbound_error_detail(&e)
                ),
            )
        })?;
    // A missing inbox row is tolerable (the conversation can still be created
    // without one), but an unreadable `inboxes` table is not: reported as "no
    // inbox row" it would tell the operator the channel is unconfigured.
    let inbox_id: Option<String> = match sqlx::query("SELECT id FROM inboxes WHERE channel_type = ? ORDER BY updated_at DESC LIMIT 1")
        .bind(channel)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(row)) => Some(
            row.try_get::<String, _>("id")
                .map_err(|err| read_failure_response("inboxes", INBOX_LOOKUP_QUERY, &err))?,
        ),
        Ok(None) => None,
        Err(err) => return Err(read_failure_response("inboxes", INBOX_LOOKUP_QUERY, &err)),
    };
    inbound::get_or_create_inbox_conversation(pool, &contact_id, channel, inbox_id.as_deref(), None)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!(
                    "Could not open a {channel} conversation for the test send: {}",
                    inbound_error_detail(&e)
                ),
            )
        })
}

/// What the provider really did, in the same three shapes `SendResult` uses in
/// `services/messaging.rs`. A caller hands this to `record_outbound_message`,
/// which derives `messages.status` from it, so no call site can assert `sent`
/// by typing a string.
#[derive(Clone, Copy)]
enum SendEvidence<'a> {
    /// The provider answered with `http_status`; `refusal` is its own error
    /// text when the code says no.
    Answered {
        http_status: u16,
        refusal: Option<&'a str>,
    },
    /// There is no answer at all -- a credential was missing or the transport
    /// failed before the provider replied. `reason` names the cause.
    NoAnswer { reason: &'a str },
}

impl<'a> SendEvidence<'a> {
    /// Built only from a status code a provider actually returned.
    fn accepted(http_status: u16) -> Self {
        Self::Answered { http_status, refusal: None }
    }

    fn rejected(http_status: u16, refusal: &'a str) -> Self {
        Self::Answered { http_status, refusal: Some(refusal) }
    }

    fn not_sent(reason: &'a str) -> Self {
        Self::NoAnswer { reason }
    }

    fn http_status(self) -> Option<u16> {
        match self {
            Self::Answered { http_status, .. } => Some(http_status),
            Self::NoAnswer { .. } => None,
        }
    }

    /// The provider's own message for a refusal, or the reason nothing left.
    fn detail(self) -> Option<&'a str> {
        match self {
            Self::Answered { refusal, .. } => refusal,
            Self::NoAnswer { reason } => Some(reason),
        }
    }

    /// The `messages.status` this evidence proves, derived exactly like
    /// `SendResult::message_status()`: a real 2xx is the only proof of a send,
    /// a non-2xx answer is a refusal, and the absence of any HTTP status means
    /// nothing was taken.
    fn message_status(self) -> &'static str {
        match self.http_status() {
            Some(code) if (200..300).contains(&code) => "sent",
            Some(_) => "failed",
            None => "not_sent",
        }
    }
}

/// Records an outbound test message and returns the audit block every response
/// echoes as `messagePersistence`. The stored status comes from `evidence`, and
/// `sent_at` is only written when that evidence is a real 2xx, so no send
/// response can imply a message the provider never accepted.
async fn record_outbound_message(
    pool: &DbPool,
    conversation_id: &str,
    channel: &str,
    provider: &str,
    body: &str,
    provider_message_id: Option<&str>,
    evidence: SendEvidence<'_>,
    metadata: Option<&Value>,
) -> Value {
    let id = Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();
    let status = evidence.message_status();
    let http_status = evidence.http_status();
    let refusal = evidence.detail();
    let sent_at: Option<String> = if status == "sent" { Some(now) } else { None };

    let mut meta_map = match metadata {
        Some(Value::Object(map)) => map.clone(),
        Some(other) => {
            let mut map = Map::new();
            map.insert("providerResponse".to_string(), other.clone());
            map
        }
        None => Map::new(),
    };
    meta_map.insert("messageStatus".to_string(), Value::String(status.to_string()));
    if let Some(code) = http_status {
        meta_map.insert("httpStatus".to_string(), Value::Number(code.into()));
    }
    if let Some(reason) = refusal {
        // A provider that already sent its own `error` envelope keeps it; the
        // derived text only fills the gap when the body carried none.
        if !meta_map.contains_key("error") {
            meta_map.insert("error".to_string(), Value::String(reason.to_string()));
        }
    }
    let meta_raw = match serde_json::to_string(&Value::Object(meta_map)) {
        Ok(raw) => Some(raw),
        Err(err) => {
            warn!("Could not serialise the {channel} provider metadata for the audit row: {err}");
            None
        }
    };

    let result = sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, direction, channel, body, provider,
            provider_message_id, status, provider_metadata, sent_at, sender_type
        ) VALUES (?, ?, 'outbound', ?, ?, ?, ?, ?, ?, ?, 'agent')
        "#,
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(channel)
    .bind(body)
    .bind(provider)
    .bind(provider_message_id)
    .bind(status)
    .bind(&meta_raw)
    .bind(sent_at.as_deref())
    .execute(pool)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => json!({
            "ok": true,
            "saved": true,
            "messageId": id,
            "status": status,
            "httpStatus": http_status,
            "sentAt": value_or_null(sent_at),
        }),
        Ok(res) => {
            let reason = format!(
                "The outbound {channel} message was not stored: the INSERT affected {} rows.",
                res.rows_affected()
            );
            warn!("{reason}");
            json!({ "ok": false, "saved": false, "messageId": id, "status": status, "reason": reason })
        }
        Err(err) => {
            warn!("Could not persist outbound {channel} message: {err}");
            json!({
                "ok": false,
                "saved": false,
                "messageId": id,
                "status": status,
                "reason": format!("The outbound {channel} message row (status `{status}`) was not written to `messages`, so nothing about this send is auditable."),
                "detail": err.to_string(),
            })
        }
    }
}

/// Suffix naming a failed audit-row write for the plain-text error responses.
fn audit_suffix(audit: &Value) -> String {
    match audit {
        Value::Null => String::new(),
        other if other.get("ok").and_then(|v| v.as_bool()) == Some(true) => String::new(),
        other => format!(" Note: {} {}", other.get("reason").and_then(|v| v.as_str()).unwrap_or("the message could not be stored"), other.get("detail").and_then(|v| v.as_str()).map(|d| format!("({d})")).unwrap_or_default()),
    }
}

// ---- provider HTTP ----

async fn http_get_json(state: &AppState, url: &str, bearer: Option<&str>) -> Result<Value, String> {
    let mut req = state.http_client.get(url);
    if let Some(token) = bearer {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.map_err(|e| format!("transport failure: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("could not read response body: {e}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {} - {}", status.as_u16(), truncate(&text)));
    }
    serde_json::from_str::<Value>(&text).map_err(|e| format!("non-JSON response: {e} - {}", truncate(&text)))
}

/// Like `http_get_json`, but for providers (Discord) whose Authorization
/// header isn't the `Bearer <token>` scheme `bearer_auth` sends.
async fn http_get_json_auth(state: &AppState, url: &str, authorization: &str) -> Result<Value, String> {
    let resp = state
        .http_client
        .get(url)
        .header("Authorization", authorization)
        .send()
        .await
        .map_err(|e| format!("transport failure: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("could not read response body: {e}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {} - {}", status.as_u16(), truncate(&text)));
    }
    serde_json::from_str::<Value>(&text).map_err(|e| format!("non-JSON response: {e} - {}", truncate(&text)))
}

async fn http_get_json_basic(
    state: &AppState,
    url: &str,
    user: &str,
    password: &str,
) -> Result<Value, String> {
    let resp = state
        .http_client
        .get(url)
        .basic_auth(user, Some(password))
        .send()
        .await
        .map_err(|e| format!("transport failure: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("could not read response body: {e}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {} - {}", status.as_u16(), truncate(&text)));
    }
    serde_json::from_str::<Value>(&text).map_err(|e| format!("non-JSON response: {e} - {}", truncate(&text)))
}

async fn http_post_json(
    state: &AppState,
    url: &str,
    body: &Value,
    bearer: Option<&str>,
) -> Result<(u16, Value, String), String> {
    let mut req = state.http_client.post(url).json(body);
    if let Some(token) = bearer {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.map_err(|e| format!("transport failure: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("could not read response body: {e}"))?;
    let parsed = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }));
    Ok((status.as_u16(), parsed, truncate(&text)))
}

async fn http_post_form(
    state: &AppState,
    url: &str,
    pairs: &[(&str, String)],
    auth: Option<(&str, &str)>,
) -> Result<(u16, Value, String), String> {
    let mut req = state.http_client.post(url);
    if let Some((user, pass)) = auth {
        req = req.basic_auth(user, Some(pass));
    }
    let resp = req
        .form(pairs)
        .send()
        .await
        .map_err(|e| format!("transport failure: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("could not read response body: {e}"))?;
    let parsed = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }));
    Ok((status.as_u16(), parsed, truncate(&text)))
}

fn truncate(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.len() > 400 {
        format!("{}—[truncated]", &trimmed[..400])
    } else {
        trimmed.to_string()
    }
}

/// Live read of the WhatsApp Business phone-number node. Every field is
/// whatever Meta actually returned for this exact token; nothing is inferred.
async fn fetch_wa_phone_node(
    state: &AppState,
    token: &str,
    phone_id: &str,
) -> Result<Value, String> {
    let version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        state.config.meta_whatsapp_api_version.as_str()
    };
    let url = format!(
        "https://graph.facebook.com/{version}/{phone_id}?fields=verified_name,display_phone_number,quality_rating,platform_type,code_verification_status,messaging_limit_tier,name_status&access_token={token}"
    );
    http_get_json(state, &url, None).await
}

/// Turns Meta's `quality_rating` into a human label. Unknown values are
/// echoed raw rather than smoothed over.
fn quality_rating_label(raw: Option<&str>) -> Option<String> {
    let value = raw?;
    Some(match value {
        "GREEN" => "Green (no restriction)".to_string(),
        "YELLOW" => "Yellow (restricted messaging)".to_string(),
        "RED" => "Red (at risk of disabling)".to_string(),
        other => other.to_string(),
    })
}

/// Meta returns either a string tier (`TIER_250`) or an object
/// `{ "tier": 2, "limit": 1000 }`. Anything unrecognised is reported as-is.
fn limit_tier_label(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => {
            let tier = map.get("tier").and_then(|v| v.as_i64());
            let limit = map.get("limit").and_then(|v| v.as_i64());
            match (tier, limit) {
                (Some(t), Some(l)) => Some(format!("Tier {t} - {l} unique customers / 24h")),
                (Some(t), None) => Some(format!("Tier {t}")),
                (None, Some(l)) => Some(format!("{l} unique customers / 24h")),
                (None, None) => None,
            }
        }
        _ => None,
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InboxRow {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub phone_number: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub credentials: Option<String>,
    pub greeting_enabled: bool,
    pub greeting_message: Option<String>,
    pub out_of_office_enabled: bool,
    pub out_of_office_message: Option<String>,
    pub csat_enabled: bool,
    pub auto_read_receipts: bool,
    pub working_hours_enabled: bool,
    pub working_hours_start: Option<i64>,
    pub working_hours_end: Option<i64>,
    pub timezone: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GatewayRow {
    pub id: String,
    pub gateway_type: String,
    pub provider: String,
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub credentials: Option<String>,
    pub config: Option<String>,
    pub test_status: Option<String>,
    pub latency_ms: Option<i64>,
    pub last_tested_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub direction: String,
    pub channel: String,
    pub body: String,
    pub provider: Option<String>,
    pub provider_message_id: Option<String>,
    pub status: String,
    pub provider_metadata: Option<String>,
    pub sent_at: String,
    pub intent: Option<String>,
    pub is_private: Option<bool>,
    pub sender_type: Option<String>,
    pub sender_name: Option<String>,
}

fn parse_json_obj(raw: &Option<String>) -> serde_json::Map<String, serde_json::Value> {
    raw.as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

/// Splits "nothing is stored here" from "something is stored but could not be
/// read": a credentials blob that fails to parse is a data problem, not an empty
/// configuration, and the second element says which.
fn parse_json_obj_note(raw: &Option<String>, label: &str) -> (Map<String, Value>, Option<String>) {
    let Some(text) = raw.as_deref() else {
        return (Map::new(), None);
    };
    match serde_json::from_str::<Value>(text) {
        Ok(value) => match value.as_object().cloned() {
            Some(map) => (map, None),
            None => (
                Map::new(),
                Some(format!(
                    "{label} is stored but is not a JSON object, so none of its keys could be read; they are unknown, not absent."
                )),
            ),
        },
        Err(err) => (
            Map::new(),
            Some(format!(
                "{label} is stored but could not be parsed as JSON ({err}); the keys inside it are unknown, not absent."
            )),
        ),
    }
}

/// The credentials map of an inbox row, with an unreadable blob reported instead
/// of collapsing into "no credentials configured".
fn inbox_credentials(inbox: Option<&InboxRow>, warnings: &mut Vec<String>) -> Map<String, Value> {
    match inbox {
        Some(row) => {
            let (map, note) = parse_json_obj_note(&row.credentials, "inboxes.credentials");
            if let Some(note) = note {
                warnings.push(note);
            }
            map
        }
        None => Map::new(),
    }
}

/// The credentials map of a gateway row, with an unreadable blob reported
/// instead of collapsing into "no gateway credentials".
fn gateway_credentials(gateway: Option<&GatewayRow>, warnings: &mut Vec<String>) -> Map<String, Value> {
    match gateway {
        Some(row) => {
            let (map, note) = parse_json_obj_note(&row.credentials, "channel_gateway_connections.credentials");
            if let Some(note) = note {
                warnings.push(note);
            }
            map
        }
        None => Map::new(),
    }
}

fn serialize_inbox(row: &InboxRow) -> serde_json::Value {
    let (creds_map, creds_note) = parse_json_obj_note(&row.credentials, "inboxes.credentials");
    let creds_val = serde_json::Value::Object(creds_map);

    let phone = row.phone_number.clone().and_then(|v| usable(Some(v.as_str())));
    let avatar = row.avatar_url.clone().and_then(|v| usable(Some(v.as_str())));
    let greeting_msg = row
        .greeting_message
        .clone()
        .and_then(|v| usable(Some(v.as_str())));
    let ooo_msg = row
        .out_of_office_message
        .clone()
        .and_then(|v| usable(Some(v.as_str())));
    let tz = row.timezone.clone().and_then(|v| usable(Some(v.as_str())));

    json!({
        "id": row.id,
        "name": row.name,
        "channel": row.channel_type,
        "channelType": row.channel_type,
        "channel_type": row.channel_type,
        "phoneNumber": value_or_null(phone.clone()),
        "phone_number": value_or_null(phone),
        "avatarUrl": value_or_null(avatar.clone()),
        "avatar_url": value_or_null(avatar),
        "isActive": row.is_active,
        "is_active": row.is_active,
        // There is no default-inbox column in the schema, so nothing claims it.
        "isDefault": Value::Null,
        // Only the enabled flag is known here; provider connectivity is a
        // separate, live question and is never implied by this row.
        "status": if row.is_active { "enabled" } else { "disabled" },
        // No read-receipt state exists per inbox, so the count is unknown.
        "unreadCount": Value::Null,
        "unreadCountNote": "Not tracked: no read/unread state is persisted per inbox.",
        "greetingEnabled": row.greeting_enabled,
        "greeting_enabled": row.greeting_enabled,
        "greetingMessage": value_or_null(greeting_msg.clone()),
        "greeting_message": value_or_null(greeting_msg),
        "outOfOfficeEnabled": row.out_of_office_enabled,
        "out_of_office_enabled": row.out_of_office_enabled,
        "outOfOfficeMessage": value_or_null(ooo_msg.clone()),
        "out_of_office_message": value_or_null(ooo_msg),
        "csatEnabled": row.csat_enabled,
        "csat_enabled": row.csat_enabled,
        "autoReadReceipts": row.auto_read_receipts,
        "auto_read_receipts": row.auto_read_receipts,
        "workingHoursEnabled": row.working_hours_enabled,
        "working_hours_enabled": row.working_hours_enabled,
        "workingHoursStart": row.working_hours_start,
        "working_hours_start": row.working_hours_start,
        "workingHoursEnd": row.working_hours_end,
        "working_hours_end": row.working_hours_end,
        "timezone": value_or_null(tz),
        "credentials": creds_val,
        "credentialsParseError": value_or_null(creds_note.clone()),
        "credentials_parse_error": value_or_null(creds_note),
        "createdAt": value_or_null(row.created_at.clone()),
        "updatedAt": value_or_null(row.updated_at.clone()),
    })
}

/// Reads the channel's inbox row. `Ok(None)` proves the row does not exist;
/// `Err` means `inboxes` could not be read, so the channel's state is unknown
/// rather than unconfigured.
async fn query_inbox_by_channels_checked(
    pool: &DbPool,
    channel_types: &[&str],
) -> Result<Option<InboxRow>, sqlx::Error> {
    let placeholders = channel_types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query_str = format!(
        "SELECT id, name, channel_type, phone_number, avatar_url, is_active, credentials, greeting_enabled, greeting_message, out_of_office_enabled, out_of_office_message, csat_enabled, auto_read_receipts, working_hours_enabled, working_hours_start, working_hours_end, timezone, created_at, updated_at FROM inboxes WHERE channel_type IN ({}) ORDER BY is_active DESC, updated_at DESC LIMIT 1",
        placeholders
    );
    let mut q = sqlx::query_as::<_, InboxRow>(&query_str);
    for ct in channel_types {
        q = q.bind(ct);
    }
    q.fetch_optional(pool).await
}

/// The same lookup with the database failure already turned into the response
/// that names the table and the query, for handlers whose whole payload depends
/// on this row.
async fn inbox_row_checked(
    pool: &DbPool,
    channel_types: &[&str],
) -> Result<Option<InboxRow>, Response> {
    query_inbox_by_channels_checked(pool, channel_types)
        .await
        .map_err(|err| read_failure_response("inboxes", INBOX_LOOKUP_QUERY, &err))
}

async fn query_channel_events(pool: &DbPool, channels: &[&str]) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let placeholders = channels.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query_str = format!(
        "SELECT id, conversation_id, direction, channel, body, provider, provider_message_id, status, provider_metadata, sent_at, intent, is_private, sender_type, sender_name FROM messages WHERE channel IN ({}) ORDER BY sent_at DESC LIMIT 50",
        placeholders
    );
    let mut q = sqlx::query_as::<_, MessageRow>(&query_str);
    for ch in channels {
        q = q.bind(ch);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter()
        .map(|m| {
            json!({
                "id": m.id,
                "conversationId": m.conversation_id,
                "direction": m.direction.to_lowercase(),
                "status": m.status,
                "body": m.body,
                "provider": usable(m.provider.as_deref()).unwrap_or_else(|| m.channel.clone()),
                "providerMessageId": m.provider_message_id,
                "senderName": m.sender_name,
                "senderType": m.sender_type,
                "sentAt": m.sent_at,
            })
        })
        .collect())
}

/// Resolves one logical requirement from, in order: the inbox credentials JSON,
/// the `integration_settings` row, then the process environment. Returns
/// `Ok(None)` when no real value exists anywhere - never a placeholder - and
/// `Err` when the settings column itself could not be read, which is a database
/// failure rather than proof of an unset value.
async fn resolve_one(
    state: &AppState,
    creds: &Map<String, Value>,
    cred_keys: &[&str],
    setting_column: Option<&str>,
    config_value: Option<String>,
) -> Result<Option<String>, sqlx::Error> {
    if let Some(v) = cred(creds, cred_keys) {
        return Ok(Some(v));
    }
    if let Some(column) = setting_column {
        if let Some(v) = setting_str(state, column).await? {
            return Ok(Some(v));
        }
    }
    Ok(config_value.and_then(|v| usable(Some(v.as_str()))))
}

/// Returns the labels that could not be resolved for real. A settings column
/// that cannot be read is a database failure, not a missing credential, so it is
/// propagated as a 500 rather than being reported as "not configured".
async fn missing_required(
    state: &AppState,
    creds: &Map<String, Value>,
    reqs: &[(&str, &[&str], Option<&str>, Option<String>)],
) -> Result<Vec<String>, Response> {
    let mut missing = Vec::new();
    for (label, keys, column, config) in reqs {
        match resolve_one(state, creds, keys, *column, config.clone()).await {
            Ok(Some(_)) => {}
            Ok(None) => missing.push((*label).to_string()),
            Err(err) => {
                let column_name = column.unwrap_or("integration_settings");
                return Err(err_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &settings_read_failure(column_name, &err),
                ));
            }
        }
    }
    Ok(missing)
}

/// Gateway credentials can supply keys the inbox row does not have; the inbox
/// always wins so a per-inbox override is respected.
fn merged_creds(primary: &Map<String, Value>, secondary: Option<&Map<String, Value>>) -> Map<String, Value> {
    let mut out = primary.clone();
    if let Some(extra) = secondary {
        for (k, v) in extra {
            if json_str(Some(v)).is_some() && json_str(out.get(k)).is_none() {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    out
}

/// Builds the health row the dashboard renders. `isConnected` is only true
/// when the inbox is enabled *and* every required credential really resolved.
/// When `real_connection` is `Some(true)`, the provider actually answered a
/// live API probe; when `Some(false)`, the probe failed and `connection_error`
/// explains why.
fn summarize(
    channel: &'static str,
    exists: bool,
    active: bool,
    missing: &[String],
    real_connection: Option<bool>,
    connection_error: Option<String>,
    provider_info: Option<serde_json::Value>,
) -> ChannelHealth {
    let configured = exists && active && missing.is_empty();
    let inbound_only = channel == "live-chat" || channel == "website";
    let is_connected = configured && real_connection.unwrap_or(false);
    let status = match (exists, active, configured, real_connection) {
        (false, _, _, _) => "NO_INBOX",
        (true, false, _, _) => "DISABLED",
        (true, true, true, Some(true)) if !inbound_only => "CONNECTED",
        (true, true, true, Some(false)) => "PROBE_FAILED",
        (true, true, true, None) => "CONFIGURED_NOT_VERIFIED",
        (true, true, true, Some(true)) => "CONFIGURED_INBOUND_ONLY",
        (true, true, false, _) => "NOT_CONFIGURED",
    };
    let status_label = match (exists, active, configured, real_connection) {
        (false, _, _, _) => "No inbox registered for this channel",
        (true, false, _, _) => "Inbox disabled",
        (true, true, true, Some(true)) if !inbound_only => {
            "Verified by a live API probe to the provider"
        }
        (true, true, true, Some(false)) => {
            "Credentials present but the provider rejected the live probe"
        }
        (true, true, true, None) => {
            "Credentials complete - not verified against the provider"
        }
        (true, true, true, Some(true)) => {
            "Widget token complete - inbound only; no outbound widget transport in this build"
        }
        (true, true, false, _) => "Not configured",
    };
    let warning = if configured && inbound_only {
        Some("No outbound transport for live-chat: a stored agent reply reaches the visitor only when the widget polls GET /widget/conversations/<id>/messages/since, so nothing sent on this channel is confirmed delivered.".to_string())
    } else if is_connected {
        None
    } else if real_connection == Some(false) {
        connection_error.clone()
    } else if !exists {
        Some(format!("No {channel} row exists in `inboxes`."))
    } else if !active {
        Some(format!("The {channel} inbox is disabled (is_active = false)."))
    } else {
        Some(missing_text(channel, &missing.iter().map(|s| s.as_str()).collect::<Vec<_>>()))
    };

    let last_verified_at = real_connection.and_then(|ok| {
        if ok {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        }
    });

    ChannelHealth {
        channel: channel.to_string(),
        is_connected,
        status: status.to_string(),
        status_label: status_label.to_string(),
        warning,
        real_connection,
        connection_error,
        last_verified_at,
        provider_info,
    }
}

/// Writes (or clears, when `value` is `Value::Null`) one key inside the inbox
/// credentials JSON. Reports what the database really did - a failed or
/// zero-row UPDATE is never returned as success - and hands back the value read
/// out of the stored row rather than the value the client asked for.
async fn set_credential_key(
    pool: &DbPool,
    channel_types: &[&str],
    key: &str,
    value: Value,
) -> Result<(WriteReport, Option<Value>), sqlx::Error> {
    let Some(inbox) = query_inbox_by_channels_checked(pool, channel_types).await? else {
        return Ok((
            WriteReport { existed: false, rows_affected: 0 },
            None,
        ));
    };
    let mut creds = parse_json_obj(&inbox.credentials);
    if value.is_null() {
        creds.remove(key);
    } else {
        creds.insert(key.to_string(), value);
    }
    let raw = serde_json::to_string(&Value::Object(creds)).unwrap_or_else(|_| "{}".to_string());
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE inboxes SET credentials = ?, updated_at = ? WHERE id = ?")
        .bind(&raw)
        .bind(&now)
        .bind(&inbox.id)
        .execute(pool)
        .await;

    match result {
        Ok(res) => {
            let stored = serde_json::from_str::<Value>(&raw)
                .ok()
                .and_then(|v| v.get(key).cloned());
            Ok((
                WriteReport { existed: true, rows_affected: res.rows_affected() },
                stored,
            ))
        }
        Err(err) => Err(err),
    }
}

struct TwilioCreds {
    account_sid: String,
    auth_token: String,
    from_number: Option<String>,
    messaging_service_sid: Option<String>,
}

/// Reads Twilio credentials from the channel inbox, its active gateway row, or
/// the process environment - in that order. Refuses with an honest error that
/// names what is missing rather than substituting a demo account.
async fn twilio_creds(state: &AppState, channel_types: &[&str], gateway_type: &str) -> Result<TwilioCreds, Response> {
    // A read that failed is not "no credentials": both the inbox and the active
    // gateway rows are propagated as database failures so the operator is never
    // told the channel is unconfigured because a table could not be read.
    let inbox = query_inbox_by_channels_checked(&state.db, channel_types)
        .await
        .map_err(|err| read_failure_response("inboxes", INBOX_LOOKUP_QUERY, &err))?;
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();
    let gateway = sqlx::query_as::<_, GatewayRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE gateway_type = ? AND is_active = 1 LIMIT 1",
    )
    .bind(gateway_type)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| read_failure_response("channel_gateway_connections", GATEWAY_LOOKUP_QUERY, &err))?;
    let gateway_creds = gateway
        .as_ref()
        .map(|g| parse_json_obj(&g.credentials))
        .unwrap_or_default();
    let creds = merged_creds(&inbox_creds, Some(&gateway_creds));

    let account_sid = cred(&creds, &["accountSid", "account_sid"])
        .or_else(|| config_val(state.config.twilio_account_sid.clone()));
    let auth_token = cred(&creds, &["authToken", "auth_token"])
        .or_else(|| config_val(state.config.twilio_auth_token.clone()));
    let from_number = inbox
        .as_ref()
        .and_then(|i| usable(i.phone_number.as_deref()))
        .or_else(|| cred(&creds, &["fromNumber", "from_number", "callerId", "voiceNumber"]))
        .or_else(|| config_val(state.config.twilio_sms_from.clone()));
    let messaging_service_sid = cred(&creds, &["messagingServiceSid", "messaging_service_sid"])
        .or_else(|| config_val(state.config.twilio_messaging_service_sid.clone()));

    let mut missing: Vec<&str> = Vec::new();
    if account_sid.is_none() {
        missing.push("twilio accountSid (inbox/gateway credentials or TWILIO_ACCOUNT_SID)");
    }
    if auth_token.is_none() {
        missing.push("twilio authToken (inbox/gateway credentials or TWILIO_AUTH_TOKEN)");
    }
    if from_number.is_none() && messaging_service_sid.is_none() {
        missing.push("a sender number (inbox phone_number / fromNumber / TWILIO_SMS_FROM) or messagingServiceSid");
    }
    if !missing.is_empty() {
        return Err(err_response(
            StatusCode::SERVICE_UNAVAILABLE,
            &missing_text("twilio", &missing),
        ));
    }

    let (Some(account_sid), Some(auth_token)) = (account_sid, auth_token) else {
        return Err(err_response(
            StatusCode::SERVICE_UNAVAILABLE,
            &missing_text("twilio", &["twilio accountSid", "twilio authToken"]),
        ));
    };

    Ok(TwilioCreds {
        account_sid,
        auth_token,
        from_number,
        messaging_service_sid,
    })
}

fn twilio_url(account_sid: &str, path: &str) -> String {
    format!("https://api.twilio.com/2010-04-01/Accounts/{account_sid}/{path}")
}

/// Signs a real Twilio Video/Voice access token (HS256 JWT) with the stored
/// auth token. Nothing is returned unless the signature was produced.
fn twilio_access_token(
    account_sid: &str,
    auth_token: &str,
    identity: &str,
    twiml_app_sid: &str,
    ttl: i64,
) -> Result<String, String> {
    use base64::Engine as _;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let now = Utc::now().timestamp();

    let header = json!({ "alg": "HS256", "typ": "JWT", "cty": "twilio-fpa;json+1.1" });
    let claims = json!({
        "iss": account_sid,
        "sub": account_sid,
        "iat": now,
        "nbf": now,
        "exp": now + ttl,
        "jti": Uuid::new_v4().simple().to_string(),
        "grants": {
            "identity": identity,
            "grant": {
                "key": "category",
                "value": "communications",
                "filter": {
                    "voice": {
                        "outgoing": { "appSid": twiml_app_sid },
                        "incoming": { "allow": true }
                    }
                }
            }
        }
    });

    let signing_input = format!(
        "{}.{}",
        engine.encode(serde_json::to_vec(&header).map_err(|e| e.to_string())?),
        engine.encode(serde_json::to_vec(&claims).map_err(|e| e.to_string())?)
    );

    let mut mac = Hmac::<Sha256>::new_from_slice(auth_token.as_bytes())
        .map_err(|e| format!("HMAC key rejected: {e}"))?;
    mac.update(signing_input.as_bytes());
    let signature = engine.encode(mac.finalize().into_bytes());

    Ok(format!("{signing_input}.{signature}"))
}

fn graph_version(state: &AppState) -> String {
    usable(Some(state.config.meta_whatsapp_api_version.as_str())).unwrap_or_else(|| "v21.0".to_string())
}

/// Sends a text message through the Messenger/Instagram Send API and returns
/// the status code Meta actually answered with plus its body. `endpoint` is
/// `me/messages` for a page token and also works for an IG token. The code
/// travels with the body because it is the only proof a caller has that the
/// message was taken.
///
/// `Err((Some(code), text))` is a provider that answered and refused;
/// `Err((None, text))` is a request that got no answer at all.
async fn messenger_send(
    state: &AppState,
    token: &str,
    recipient_key: &str,
    recipient: &str,
    text: &str,
    messaging_type: &str,
) -> Result<(u16, Value), (Option<u16>, String)> {
    let version = graph_version(state);
    let url = format!("https://graph.facebook.com/{version}/me/messages");
    let body = json!({
        "recipient": { recipient_key: recipient },
        "messaging_type": messaging_type,
        "message": { "text": text }
    });
    let (status_code, parsed, raw) = match http_post_json(state, &url, &body, Some(token)).await {
        Ok(reply) => reply,
        Err(err) => return Err((None, err)),
    };
    if !(200..300).contains(&status_code) {
        return Err((Some(status_code), format!("HTTP {status_code} - {raw}")));
    }
    Ok((status_code, parsed))
}

/// What a write to the `inboxes` table actually did. `existed` distinguishes
/// "no inbox row for this channel" from a database failure, and `rows_affected`
/// makes a matched-but-zero-row UPDATE impossible to report as a save.
pub struct WriteReport {
    pub existed: bool,
    pub rows_affected: u64,
}

impl WriteReport {
    fn saved(&self) -> bool {
        self.existed && self.rows_affected > 0
    }
}

/// Attaches the honest `persistence` block to the freshly-read channel state
/// and gives the response the status code the write really earned.
fn write_response(
    channel: &str,
    write: Result<WriteReport, sqlx::Error>,
    mut body: Value,
) -> Response {
    let (status, ok, reason, detail) = match &write {
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            false,
            format!("The {channel} inbox row could not be written; the state in this response is what the database still holds."),
            Some(err.to_string()),
        ),
        Ok(report) if !report.existed => (
            StatusCode::NOT_FOUND,
            false,
            format!("No {channel} inbox row exists in `inboxes`, so nothing was saved. Connect the channel first."),
            None,
        ),
        Ok(report) if report.rows_affected == 0 => (
            StatusCode::INTERNAL_SERVER_ERROR,
            false,
            format!("The {channel} inbox row was not updated: the UPDATE matched 0 rows."),
            None,
        ),
        Ok(report) => (
            StatusCode::OK,
            true,
            format!("{channel} inbox row updated ({} row(s) affected).", report.rows_affected),
            None,
        ),
    };
    let rows_affected = match &write {
        Ok(report) => json!(report.rows_affected),
        Err(_) => Value::Null,
    };

    body["persistence"] = json!({
        "ok": ok,
        "saved": ok,
        "rowsAffected": rows_affected,
        "reason": reason,
        "detail": value_or_null(detail),
    });
    if !ok {
        body["error"] = Value::String(reason);
    }
    (status, Json(body)).into_response()
}

async fn update_inbox_record(
    pool: &DbPool,
    channel_types: &[&str],
    payload: &serde_json::Value,
    extra_keys: &[(&str, &str)],
) -> Result<WriteReport, sqlx::Error> {
    let Some(existing) = query_inbox_by_channels_checked(pool, channel_types).await? else {
        return Ok(WriteReport { existed: false, rows_affected: 0 });
    };

    let mut creds = parse_json_obj(&existing.credentials);
    for &(pkey, ckey) in extra_keys {
        if let Some(val) = payload.get(pkey) {
            if !val.is_null() {
                creds.insert(ckey.to_string(), val.clone());
            }
        }
    }

    let new_name = payload.get("name").and_then(|v| v.as_str()).unwrap_or(&existing.name);
    let new_phone = payload
        .get("phone_number")
        .or_else(|| payload.get("phoneNumber"))
        .or_else(|| payload.get("voice_number"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or(existing.phone_number.clone());

    let greeting_en = payload.get("greeting_enabled").or_else(|| payload.get("greetingEnabled")).and_then(|v| v.as_bool()).unwrap_or(existing.greeting_enabled);
    let greeting_msg = payload.get("greeting_message").or_else(|| payload.get("greetingMessage")).and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.greeting_message.clone());
    let ooo_en = payload.get("out_of_office_enabled").or_else(|| payload.get("outOfOfficeEnabled")).and_then(|v| v.as_bool()).unwrap_or(existing.out_of_office_enabled);
    let ooo_msg = payload.get("out_of_office_message").or_else(|| payload.get("outOfOfficeMessage")).and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.out_of_office_message.clone());
    let csat_en = payload.get("csat_enabled").or_else(|| payload.get("csatEnabled")).and_then(|v| v.as_bool()).unwrap_or(existing.csat_enabled);
    let wh_en = payload.get("working_hours_enabled").or_else(|| payload.get("workingHoursEnabled")).and_then(|v| v.as_bool()).unwrap_or(existing.working_hours_enabled);
    let wh_start = payload.get("working_hours_start").or_else(|| payload.get("workingHoursStart")).and_then(|v| v.as_i64()).or(existing.working_hours_start);
    let wh_end = payload.get("working_hours_end").or_else(|| payload.get("workingHoursEnd")).and_then(|v| v.as_i64()).or(existing.working_hours_end);
    let tz = payload.get("timezone").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.timezone.clone());

    let creds_str = serde_json::to_string(&creds).unwrap_or_else(|_| "{}".to_string());
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        UPDATE inboxes
        SET name = ?, phone_number = ?, credentials = ?, greeting_enabled = ?,
            greeting_message = ?, out_of_office_enabled = ?, out_of_office_message = ?,
            csat_enabled = ?, working_hours_enabled = ?, working_hours_start = ?,
            working_hours_end = ?, timezone = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(new_name)
    .bind(&new_phone)
    .bind(&creds_str)
    .bind(greeting_en)
    .bind(&greeting_msg)
    .bind(ooo_en)
    .bind(&ooo_msg)
    .bind(csat_en)
    .bind(wh_en)
    .bind(wh_start)
    .bind(wh_end)
    .bind(&tz)
    .bind(&now)
    .bind(&existing.id)
    .execute(pool)
    .await?;

    Ok(WriteReport { existed: true, rows_affected: result.rows_affected() })
}

// ---------------------------------------------------------
// 1. Inboxes & Channels Overview
// ---------------------------------------------------------

pub async fn list_inboxes(State(state): State<AppState>) -> Result<impl IntoResponse, Response> {
    let inboxes = sqlx::query_as::<_, InboxRow>(
        "SELECT id, name, channel_type, phone_number, avatar_url, is_active, credentials, greeting_enabled, greeting_message, out_of_office_enabled, out_of_office_message, csat_enabled, auto_read_receipts, working_hours_enabled, working_hours_start, working_hours_end, timezone, created_at, updated_at FROM inboxes ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| read_failure_response("inboxes", "SELECT ... FROM inboxes ORDER BY name ASC", &err))?;

    let gateways = sqlx::query_as::<_, GatewayRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| read_failure_response("channel_gateway_connections", "SELECT ... FROM channel_gateway_connections ORDER BY name ASC", &err))?;

    let mut result: Vec<serde_json::Value> = inboxes.into_iter().map(|i| serialize_inbox(&i)).collect();

    for gw in gateways {
        let creds_map = parse_json_obj(&gw.credentials);
        let creds_val = serde_json::Value::Object(creds_map);

        // The gateway table records an actual probe result; use it instead of
        // inferring connectivity from `is_active`.
        let tested = usable(gw.test_status.as_deref());
        let last_tested = usable(gw.last_tested_at.as_deref());

        result.push(json!({
            "id": format!("gw-{}", gw.id),
            "name": gw.name,
            "channel": gw.gateway_type,
            "channelType": gw.gateway_type,
            "channel_type": gw.gateway_type,
            "kind": "gateway",
            "provider": gw.provider,
            "phoneNumber": Value::Null,
            "phone_number": Value::Null,
            "avatarUrl": Value::Null,
            "avatar_url": Value::Null,
            "isActive": gw.is_active,
            "is_active": gw.is_active,
            "isDefault": gw.is_default,
            "status": tested.as_deref()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|| "untested".to_string()),
            "testStatus": value_or_null(tested.clone()),
            "lastTestedAt": value_or_null(last_tested.clone()),
            "latencyMs": gw.latency_ms,
            "unreadCount": Value::Null,
            "credentials": creds_val,
            "config": gw.config.as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .unwrap_or(Value::Null),
            "createdAt": value_or_null(usable(gw.created_at.as_deref())),
            "updatedAt": value_or_null(usable(gw.updated_at.as_deref())),
        }));
    }

    Ok(Json(result))
}

pub async fn get_channels_health(State(state): State<AppState>) -> Result<impl IntoResponse, Response> {
    let inboxes = sqlx::query_as::<_, InboxRow>(
        "SELECT id, name, channel_type, phone_number, avatar_url, is_active, credentials, greeting_enabled, greeting_message, out_of_office_enabled, out_of_office_message, csat_enabled, auto_read_receipts, working_hours_enabled, working_hours_start, working_hours_end, timezone, created_at, updated_at FROM inboxes",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| read_failure_response("inboxes", "SELECT ... FROM inboxes", &err))?;

    let gateways = sqlx::query_as::<_, GatewayRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| read_failure_response("channel_gateway_connections", "SELECT ... FROM channel_gateway_connections", &err))?;

    let find_inbox = |types: &[&str]| inboxes.iter().find(|i| types.contains(&i.channel_type.as_str()));
    let find_gw = |gt: &str| gateways.iter().find(|g| g.gateway_type == gt && g.is_active);

    let creds_of = |inbox: Option<&InboxRow>, gateway_type: Option<&str>| -> Map<String, Value> {
        let base = inbox.map(|i| parse_json_obj(&i.credentials)).unwrap_or_default();
        match gateway_type.and_then(find_gw) {
            Some(g) => merged_creds(&base, Some(&parse_json_obj(&g.credentials))),
            None => base,
        }
    };
    let active_of = |inbox: Option<&InboxRow>| inbox.map(|i| i.is_active).unwrap_or(false);

    // 1. WhatsApp
    let wa_inbox = find_inbox(&["whatsapp"]);
    let wa_creds = creds_of(wa_inbox, None);
    let wa_missing = missing_required(
        &state,
        &wa_creds,
        &[
            (
                "meta access token",
                &["accessToken", "access_token", "whatsappAccessToken"],
                Some("meta_access_token"),
                config_val(state.config.meta_whatsapp_access_token.clone()),
            ),
            (
                "phone number id",
                &["phoneNumberId", "phone_number_id", "waPhoneNumberId"],
                Some("meta_phone_number_id"),
                config_val(state.config.meta_whatsapp_phone_number_id.clone()),
            ),
        ],
    )
    .await?;

    // 2. Live Chat
    let lc_inbox = find_inbox(&["website", "live-chat", "live_chat"]);
    let lc_creds = creds_of(lc_inbox, None);
    let lc_missing = missing_required(
        &state,
        &lc_creds,
        &[("website token", &["websiteToken", "siteToken", "live_token"], None, None)],
    )
    .await?;

    // 3. SMS
    let sms_inbox = find_inbox(&["sms"]);
    let sms_creds = creds_of(sms_inbox, Some("sms"));
    let sms_missing = missing_required(
        &state,
        &sms_creds,
        &[
            (
                "twilio account sid",
                &["accountSid", "account_sid"],
                None,
                config_val(state.config.twilio_account_sid.clone()),
            ),
            (
                "twilio auth token",
                &["authToken", "auth_token"],
                None,
                config_val(state.config.twilio_auth_token.clone()),
            ),
            (
                "sender number or messaging service sid",
                &["fromNumber", "from_number", "messagingServiceSid", "messaging_service_sid"],
                None,
                config_val(state.config.twilio_sms_from.clone()),
            ),
        ],
    )
    .await?;

    // 4. Email
    let em_inbox = find_inbox(&["email"]);
    let em_creds = creds_of(em_inbox, Some("email"));
    let em_missing = missing_required(
        &state,
        &em_creds,
        &[
            ("smtp host", &["smtpHost", "smtp_host"], None, None),
            ("from address", &["fromAddress", "from_address", "fromEmail"], None, None),
            ("smtp username", &["smtpUsername", "smtp_user", "username"], None, None),
            ("smtp password", &["smtpPassword", "smtp_password", "password"], None, None),
        ],
    )
    .await?;

    // 5. Facebook
    let fb_inbox = find_inbox(&["facebook"]);
    let fb_creds = creds_of(fb_inbox, None);
    let fb_missing = missing_required(
        &state,
        &fb_creds,
        &[
            ("page id", &["pageId", "page_id"], None, None),
            ("page access token", &["pageAccessToken", "page_access_token", "accessToken"], None, None),
        ],
    )
    .await?;

    // 6. Instagram
    let ig_inbox = find_inbox(&["instagram"]);
    let ig_creds = creds_of(ig_inbox, None);
    let ig_missing = missing_required(
        &state,
        &ig_creds,
        &[
            ("account id", &["igId", "ig_id", "instagramUserId", "userId"], None, None),
            ("access token", &["accessToken", "access_token"], None, None),
        ],
    )
    .await?;

    // 7. Telegram
    let tg_inbox = find_inbox(&["telegram"]);
    let tg_creds = creds_of(tg_inbox, None);
    let tg_missing = missing_required(
        &state,
        &tg_creds,
        &[("bot token", &["botToken", "bot_token", "accessToken"], None, None)],
    )
    .await?;

    // 8. LINE
    let line_inbox = find_inbox(&["line"]);
    let line_creds = creds_of(line_inbox, None);
    let line_missing = missing_required(
        &state,
        &line_creds,
        &[
            ("channel id", &["channelId", "channel_id"], None, None),
            ("channel access token", &["channelAccessToken", "channel_access_token"], None, None),
        ],
    )
    .await?;

    // 9. Voice
    let voice_inbox = find_inbox(&["voice"]);
    let voice_creds = creds_of(voice_inbox, Some("voice_sip"));
    let voice_missing = missing_required(
        &state,
        &voice_creds,
        &[
            (
                "twilio account sid",
                &["accountSid", "account_sid"],
                None,
                config_val(state.config.twilio_account_sid.clone()),
            ),
            (
                "twilio auth token",
                &["authToken", "auth_token"],
                None,
                config_val(state.config.twilio_auth_token.clone()),
            ),
            (
                "voice number",
                &["voiceNumber", "fromNumber", "callerId", "from_number"],
                None,
                config_val(state.config.twilio_sms_from.clone()),
            ),
        ],
    )
    .await?;

    // 10. TikTok
    let tt_inbox = find_inbox(&["tiktok"]);
    let tt_creds = creds_of(tt_inbox, None);
    let tt_missing = missing_required(
        &state,
        &tt_creds,
        &[
            ("app id", &["appId", "app_id", "clientKey"], None, None),
            (
                "access token (inbox credentials accessToken or env TIKTOK_ACCESS_TOKEN)",
                &["accessToken", "access_token"],
                None,
                tiktok_access_token(&state),
            ),
        ],
    )
    .await?;

    // --- Live API probes for real connection status ---

    // 1. WhatsApp: probe Meta Graph API
    let (wa_real, wa_err, wa_info) = if wa_missing.is_empty() {
        let (token, phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
        let t = usable(Some(token.as_str()));
        let p = usable(Some(phone_id.as_str()));
        match (t, p) {
            (Some(tok), Some(pid)) => {
                match fetch_wa_phone_node(&state, &tok, &pid).await {
                    Ok(node) => (Some(true), None, Some(node)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 2. SMS (Twilio): probe Twilio Account API
    let (sms_real, sms_err, sms_info) = if sms_missing.is_empty() {
        let sid = resolve_one(&state, &sms_creds, &["accountSid", "account_sid"], None, config_val(state.config.twilio_account_sid.clone())).await.ok().flatten();
        let tok = resolve_one(&state, &sms_creds, &["authToken", "auth_token"], None, config_val(state.config.twilio_auth_token.clone())).await.ok().flatten();
        match (sid, tok) {
            (Some(s), Some(t)) if s.starts_with("AC") => {
                let url = format!("https://api.twilio.com/2010-04-01/Accounts/{s}.json");
                match http_get_json_basic(&state, &url, &s, &t).await {
                    Ok(acc) => (Some(true), None, Some(acc)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 3. Voice (Twilio): same probe as SMS
    let (voice_real, voice_err, voice_info) = if voice_missing.is_empty() {
        let sid = resolve_one(&state, &voice_creds, &["accountSid", "account_sid"], None, config_val(state.config.twilio_account_sid.clone())).await.ok().flatten();
        let tok = resolve_one(&state, &voice_creds, &["authToken", "auth_token"], None, config_val(state.config.twilio_auth_token.clone())).await.ok().flatten();
        match (sid, tok) {
            (Some(s), Some(t)) if s.starts_with("AC") => {
                let url = format!("https://api.twilio.com/2010-04-01/Accounts/{s}.json");
                match http_get_json_basic(&state, &url, &s, &t).await {
                    Ok(acc) => (Some(true), None, Some(acc)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 4. Telegram: probe getMe API
    let (tg_real, tg_err, tg_info) = if tg_missing.is_empty() {
        let bot_token = resolve_one(&state, &tg_creds, &["botToken", "bot_token", "accessToken"], None, None).await.ok().flatten();
        match bot_token {
            Some(tok) if !tok.is_empty() => {
                let url = format!("https://api.telegram.org/bot{tok}/getMe");
                match http_get_json(&state, &url, None).await {
                    Ok(me) => (Some(true), None, Some(me)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 5. Facebook (Messenger): probe Graph API page
    let (fb_real, fb_err, fb_info) = if fb_missing.is_empty() {
        let page_token = resolve_one(&state, &fb_creds, &["pageAccessToken", "page_access_token", "accessToken"], None, None).await.ok().flatten();
        let page_id = resolve_one(&state, &fb_creds, &["pageId", "page_id"], None, None).await.ok().flatten();
        match (page_token, page_id) {
            (Some(tok), Some(pid)) => {
                let version = if state.config.meta_whatsapp_api_version.is_empty() { "v21.0" } else { &state.config.meta_whatsapp_api_version };
                let url = format!("https://graph.facebook.com/{version}/{pid}?fields=name,picture");
                match http_get_json(&state, &url, Some(&tok)).await {
                    Ok(page) => (Some(true), None, Some(page)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 6. Instagram: probe Graph API
    let (ig_real, ig_err, ig_info) = if ig_missing.is_empty() {
        let ig_id = resolve_one(&state, &ig_creds, &["igId", "ig_id", "instagramUserId", "userId"], None, None).await.ok().flatten();
        let tok = resolve_one(&state, &ig_creds, &["accessToken", "access_token"], None, None).await.ok().flatten();
        match (ig_id, tok) {
            (Some(id), Some(t)) => {
                let version = if state.config.meta_whatsapp_api_version.is_empty() { "v21.0" } else { &state.config.meta_whatsapp_api_version };
                let url = format!("https://graph.instagram.com/{version}/{id}?fields=id,username");
                match http_get_json(&state, &url, Some(&t)).await {
                    Ok(info) => (Some(true), None, Some(info)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 7. LINE: probe profile API
    let (line_real, line_err, line_info) = if line_missing.is_empty() {
        let channel_tok = resolve_one(&state, &line_creds, &["channelAccessToken", "channel_access_token"], None, None).await.ok().flatten();
        match channel_tok {
            Some(tok) if !tok.is_empty() => {
                let url = "https://api.line.me/v2/bot/profile";
                let mut req = state.http_client.get(url);
                req = req.header("Authorization", format!("Bearer {tok}"));
                match req.send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match resp.json::<serde_json::Value>().await {
                            Ok(profile) => (Some(true), None, Some(profile)),
                            Err(e) => (Some(false), Some(format!("LINE profile parse error: {e}")), None),
                        }
                    }
                    Ok(resp) => (Some(false), Some(format!("LINE API returned HTTP {}", resp.status())), None),
                    Err(e) => (Some(false), Some(format!("Could not reach LINE API: {e}")), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // 8. TikTok: probe user info API
    let (tt_real, tt_err, tt_info) = if tt_missing.is_empty() {
        let tok = resolve_one(&state, &tt_creds, &["accessToken", "access_token"], None, tiktok_access_token(&state)).await.ok().flatten();
        match tok {
            Some(t) if !t.is_empty() => {
                let url = "https://open.tiktokapis.com/v2/user/info/";
                match http_get_json(&state, url, Some(&t)).await {
                    Ok(info) => (Some(true), None, Some(info)),
                    Err(e) => (Some(false), Some(e), None),
                }
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };

    // Live-chat and email have no outbound provider to probe
    let health = vec![
        summarize("whatsapp", wa_inbox.is_some(), active_of(wa_inbox), &wa_missing, wa_real, wa_err, wa_info),
        summarize("live-chat", lc_inbox.is_some(), active_of(lc_inbox), &lc_missing, None, None, None),
        summarize("sms", sms_inbox.is_some(), active_of(sms_inbox), &sms_missing, sms_real, sms_err, sms_info),
        summarize("email", em_inbox.is_some(), active_of(em_inbox), &em_missing, None, None, None),
        summarize("facebook", fb_inbox.is_some(), active_of(fb_inbox), &fb_missing, fb_real, fb_err, fb_info),
        summarize("instagram", ig_inbox.is_some(), active_of(ig_inbox), &ig_missing, ig_real, ig_err, ig_info),
        summarize("telegram", tg_inbox.is_some(), active_of(tg_inbox), &tg_missing, tg_real, tg_err, tg_info),
        summarize("line", line_inbox.is_some(), active_of(line_inbox), &line_missing, line_real, line_err, line_info),
        summarize("voice", voice_inbox.is_some(), active_of(voice_inbox), &voice_missing, voice_real, voice_err, voice_info),
        summarize("tiktok", tt_inbox.is_some(), active_of(tt_inbox), &tt_missing, tt_real, tt_err, tt_info),
    ];

    Ok(Json(health))
}

// ---------------------------------------------------------
// 2. WhatsApp Channel
// ---------------------------------------------------------

pub async fn get_whatsapp_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["whatsapp"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let serialized_inbox = inbox
        .as_ref()
        .map(|i| serialize_inbox(i))
        .unwrap_or(Value::Null);
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);

    let (raw_token, raw_phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let token = usable(Some(raw_token.as_str()));
    let phone_id = usable(Some(raw_phone_id.as_str()));

    let mut missing: Vec<String> = Vec::new();
    if token.is_none() {
        missing.push("meta access token".to_string());
    }
    if phone_id.is_none() {
        missing.push("phone number id".to_string());
    }

    // The only way to know the real display name, quality, tier and
    // verification state is to ask Meta for this exact phone-number node.
    let live = match (&token, &phone_id) {
        (Some(t), Some(p)) => Some(fetch_wa_phone_node(&state, t, p).await),
        _ => None,
    };
    let node: Option<Value> = match &live {
        Some(Ok(v)) => Some(v.clone()),
        _ => None,
    };
    let live_error: Option<String> = match &live {
        Some(Err(e)) => Some(e.clone()),
        _ => None,
    };

    let field = |key: &str| -> Option<String> {
        node.as_ref()
            .and_then(|n| usable(n.get(key).and_then(|v| v.as_str())))
    };

    let is_connected = node.is_some();
    // `setting_str` is async, so the fallbacks are inline matches rather than
    // `or_else` closures; the short-circuit order is unchanged.
    let display_name = match field("verified_name") {
        Some(v) => Some(v),
        None => setting_opt(&state, "whatsapp_verified_name", &mut warnings).await,
    };
    let display_phone = match field("display_phone_number") {
        Some(v) => Some(v),
        None => match setting_opt(&state, "whatsapp_display_number", &mut warnings).await {
            Some(v) => Some(v),
            None => inbox.as_ref().and_then(|i| usable(i.phone_number.as_deref())),
        },
    };
    let quality = quality_rating_label(field("quality_rating").as_deref());
    let tier = limit_tier_label(node.as_ref().and_then(|n| n.get("messaging_limit_tier")));
    let code_status = field("code_verification_status");
    let verified_at = setting_opt(&state, "whatsapp_verified_at", &mut warnings).await;

    let callback_base = public_base_url(&state);
    let callback_url = callback_base
        .as_ref()
        .map(|base| format!("{base}/api/v1/webhooks/meta/whatsapp"));
    let verify_token = resolve_opt(
        &state,
        &creds,
        &["webhookVerifyToken", "webhook_verify_token"],
        Some("meta_webhook_verify_token"),
        config_val(state.config.meta_webhook_verify_token.clone()),
        &mut warnings,
    )
    .await;

    let mut callback_note = Value::Null;
    if callback_url.is_none() {
        warn!("WhatsApp webhook callback URL cannot be built: PUBLIC_URL / PUBLIC_BASE_URL is not configured");
        callback_note = Value::String(
            "callbackUrl is null because no public base URL is configured (set PUBLIC_URL or PUBLIC_BASE_URL)."
                .to_string(),
        );
    }
    if let Some(err) = &live_error {
        warn!("WhatsApp live profile read failed: {err}");
    }

    Json(json!({
        "channel": "whatsapp",
        "inbox": serialized_inbox,
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url),
            "callbackUrlNote": callback_note,
            "verifyToken": value_or_null(verify_token),
            "verifyTokenSource": "inbox credentials / integration_settings / META_WEBHOOK_VERIFY_TOKEN",
            "subscribedFields": ["messages", "message_echoes", "message_reactions"]
        },
        "health": {
            "isConnected": is_connected,
            "configured": token.is_some() && phone_id.is_some(),
            "status": if is_connected { "CONNECTED" } else if token.is_some() && phone_id.is_some() { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if is_connected {
                "Verified by a live Graph API read"
            } else if token.is_some() && phone_id.is_some() {
                "Credentials present but Meta rejected the live read"
            } else {
                "Not configured"
            },
            "verifiedAt": value_or_null(verified_at),
            "displayName": value_or_null(display_name),
            "displayPhoneNumber": value_or_null(display_phone),
            "qualityRating": value_or_null(quality),
            "messagingLimitTier": value_or_null(tier),
            "codeVerificationStatus": value_or_null(code_status),
            "nameStatus": value_or_null(field("name_status")),
            "phoneNumberId": value_or_null(phone_id),
            "providerFields": node.unwrap_or(Value::Null),
            "connectionError": value_or_null(live_error),
            "missingCredentials": missing
        }
    }))
}

pub async fn update_whatsapp_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["whatsapp"],
        &payload,
        &[
            ("phone_number_id", "phoneNumberId"),
            ("waba_id", "wabaId"),
            ("access_token", "accessToken"),
            ("app_secret", "appSecret"),
            ("webhook_verify_token", "webhookVerifyToken"),
            ("catalog_id", "catalogId"),
        ],
    )
    .await;

    write_response("whatsapp", write, get_whatsapp_channel(State(state)).await.0)
}

pub async fn verify_whatsapp_connection(State(state): State<AppState>) -> impl IntoResponse {
    let (raw_token, raw_phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let token = match usable(Some(raw_token.as_str())) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("whatsapp", &["meta access token (inbox credentials, integration_settings.meta_access_token or META_WHATSAPP_ACCESS_TOKEN)"]),
            )
        }
    };
    let phone_id = match usable(Some(raw_phone_id.as_str())) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("whatsapp", &["phone number id (integration_settings.meta_phone_number_id or META_PHONE_NUMBER_ID)"]),
            )
        }
    };

    let started = std::time::Instant::now();
    let node = fetch_wa_phone_node(&state, &token, &phone_id).await;
    let reachable = node.is_ok();
    if let Err(err) = &node {
        warn!("WhatsApp verification probe failed for phone number {phone_id}: {err}");
    }

    // Whether the number actually subscribes to the webhook fields is itself a
    // live Graph read; failures are reported, never assumed.
    let version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        state.config.meta_whatsapp_api_version.as_str()
    };
    let subscription = http_get_json(
        &state,
        &format!("https://graph.facebook.com/{version}/{phone_id}/subscribed_apps"),
        Some(&token),
    )
    .await;

    let node_value = node.as_ref().ok().cloned().unwrap_or(Value::Null);
    // The probe above is a live Graph read of this exact phone-number node, so
    // the only timestamp this endpoint may report is when that read completed.
    let verified_at = reachable.then(Utc::now).map(|t| t.to_rfc3339());
    let status = if reachable {
        StatusCode::OK
    } else {
        StatusCode::BAD_GATEWAY
    };

    (
        status,
        Json(json!({
            "status": if reachable { "ok" } else { "failed" },
            "verified": reachable,
            "health": {
                "isConnected": reachable,
                "graphApiReachable": reachable,
                "tokenValid": reachable,
                "roundTripMs": started.elapsed().as_millis() as i64,
                "verifiedAt": value_or_null(verified_at),
                "verificationState": if reachable { "verified-by-live-graph-read" } else { "not verified" },
                "verificationReason": if reachable {
                    Value::String(format!("GET graph.facebook.com/{version}/{phone_id} returned the phone-number node."))
                } else {
                    node.as_ref().err().map(|e| Value::String(format!("Meta did not answer the live phone-number read: {e}"))).unwrap_or(Value::Null)
                },
                "webhookSubscribedApps": subscription.as_ref().ok().cloned().unwrap_or(Value::Null),
                "webhookSubscribedAppsError": value_or_null(subscription.as_ref().err().cloned()),
                "providerResponse": node_value,
                "connectionError": node.err().map(Value::String).unwrap_or(Value::Null),
            }
        })),
    )
        .into_response()
}

pub async fn list_channel_whatsapp_templates(State(state): State<AppState>) -> impl IntoResponse {
    let (raw_token, _) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let token = match usable(Some(raw_token.as_str())) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("whatsapp templates", &["meta access token"]),
            )
        }
    };

    let inbox = match inbox_row_checked(&state.db, &["whatsapp"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let waba_id = match resolve_one(
        &state,
        &creds,
        &["wabaId", "waba_id", "businessId"],
        Some("meta_waba_id"),
        None,
    )
    .await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("whatsapp templates", &["WABA id (inbox credentials wabaId or integration_settings.meta_waba_id)"]), &warnings),
            )
        }
        Err(err) => {
            return err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &settings_read_failure("meta_waba_id", &err),
            )
        }
    };

    let version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        state.config.meta_whatsapp_api_version.as_str()
    };
    let url = format!(
        "https://graph.facebook.com/{version}/{waba_id}/message_templates?limit=100"
    );

    match http_get_json(&state, &url, Some(&token)).await {
        Ok(response) => {
            let data = response
                .get("data")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let templates: Vec<Value> = data
                .iter()
                .map(|tpl| {
                    let components = tpl
                        .get("structure")
                        .and_then(|s| s.get("components"))
                        .and_then(|c| c.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let body_text = components
                        .iter()
                        .find(|c| c.get("type").and_then(|v| v.as_str()) == Some("BODY"))
                        .and_then(|c| c.get("text"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    json!({
                        "id": value_or_null(tpl.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())),
                        "name": value_or_null(tpl.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())),
                        "category": value_or_null(tpl.get("category").and_then(|v| v.as_str()).map(|s| s.to_string())),
                        "language": value_or_null(tpl.get("language").and_then(|v| v.as_str()).map(|s| s.to_string())),
                        "status": value_or_null(tpl.get("status").and_then(|v| v.as_str()).map(|s| s.to_string())),
                        "bodyText": value_or_null(body_text),
                        "components": components,
                        "lastUpdated": Value::Null,
                    })
                })
                .collect();
            Json(templates).into_response()
        }
        Err(err) => {
            warn!("Could not load WhatsApp templates from Meta: {err}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": format!("Meta did not return the template list: {err}"),
                    "degraded": true,
                })),
            )
                .into_response()
        }
    }
}

pub async fn sync_whatsapp_templates() -> impl IntoResponse {
    err_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Not implemented for Meta WhatsApp: sync-templates would need a local whatsapp_templates table (not part of the Rust schema) before Meta templates could be stored; templates are read live from /inboxes/channels/whatsapp/templates instead.",
    )
}

pub async fn send_whatsapp_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let to = match require_recipient(&payload, &["recipient_phone", "recipient", "to"], "WhatsApp") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let (raw_token, raw_phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let token = match usable(Some(raw_token.as_str())) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("WhatsApp test send", &["meta access token"]),
            )
        }
    };
    let phone_id = match usable(Some(raw_phone_id.as_str())) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("WhatsApp test send", &["phone number id"]),
            )
        }
    };

    let version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        state.config.meta_whatsapp_api_version.as_str()
    };
    let meta_url = format!("https://graph.facebook.com/{version}/{phone_id}/messages");
    let meta_body = json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": to,
        "type": "text",
        "text": { "preview_url": false, "body": msg }
    });

    let conversation_id =
        match ensure_outbound_conversation(&state.db, "whatsapp", &to, &state.config.default_tenant_key).await {
            Ok(id) => id,
            Err(resp) => return resp,
        };

    let response = match http_post_json(&state, &meta_url, &meta_body, Some(&token)).await {
        Ok(v) => v,
        Err(err) => {
            let meta = json!({ "error": err });
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "whatsapp",
                "whatsapp",
                &msg,
                None,
                SendEvidence::not_sent(&err),
                Some(&meta),
            )
            .await;
            return err_response(
                StatusCode::BAD_GATEWAY,
                &format!("WhatsApp send failed before reaching Meta: {err}{}", audit_suffix(&audit)),
            );
        }
    };
    let (status_code, body, raw) = response;

    let first = body.get("messages").and_then(|v| v.as_array()).and_then(|a| a.first());
    let provider_message_id = first.and_then(|m| usable(m.get("id").and_then(|v| v.as_str())));
    // Meta's send response carries no per-message status; delivery only ever
    // arrives on the webhook. That reported status stays separate from the
    // recorded one, which only Meta's HTTP answer can decide.
    let status_from_provider = first.and_then(|m| usable(m.get("status").and_then(|v| v.as_str())));
    let provider_status = status_from_provider
        .clone()
        .unwrap_or_else(|| "accepted".to_string());

    if !(200..300).contains(&status_code) {
        warn!("Meta rejected the WhatsApp test send (HTTP {status_code}): {raw}");
        let refusal = format!("Meta returned HTTP {status_code}: {raw}");
        let evidence = SendEvidence::rejected(status_code, &refusal);
        let audit = record_outbound_message(
            &state.db,
            &conversation_id,
            "whatsapp",
            "whatsapp",
            &msg,
            provider_message_id.as_deref(),
            evidence,
            Some(&body),
        )
        .await;
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "success": false,
                "status": evidence.message_status(),
                "recipient": to,
                "provider": "whatsapp",
                "httpStatus": status_code,
                "providerResponse": body,
                "error": refusal,
                "messagePersistence": audit,
            })),
        )
            .into_response();
    }

    let evidence = SendEvidence::accepted(status_code);
    let audit = record_outbound_message(
        &state.db,
        &conversation_id,
        "whatsapp",
        "whatsapp",
        &msg,
        provider_message_id.as_deref(),
        evidence,
        Some(&body),
    )
    .await;

    Json(json!({
        "success": true,
        "recipient": to,
        "message": msg,
        "messageId": value_or_null(provider_message_id),
        "conversationId": conversation_id,
        "provider": "whatsapp",
        "status": provider_status,
        "recordedStatus": evidence.message_status(),
        "statusSource": status_from_provider.map(|_| "reported by Meta").unwrap_or("inferred from the HTTP status; Meta returned no per-message status"),
        "messagePersistence": audit,
        "httpStatus": status_code,
        "providerResponse": body,
        "sentAt": Utc::now().to_rfc3339()
    }))
    .into_response()
}

pub async fn list_whatsapp_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["whatsapp"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

pub async fn get_whatsapp_profile(State(state): State<AppState>) -> impl IntoResponse {
    let (wa_token, wa_phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let profile = crate::services::meta_sync::fetch_meta_business_profile(
        &state,
        &wa_token,
        &wa_phone_id,
        &state.config.meta_whatsapp_api_version,
    )
    .await;
    Json(profile)
}

pub async fn update_whatsapp_profile(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let (raw_token, raw_phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let token = match usable(Some(raw_token.as_str())) {
        Some(v) => v,
        None => return err_response(StatusCode::SERVICE_UNAVAILABLE, &missing_text("WhatsApp business profile update", &["meta access token"])),
    };
    let phone_id = match usable(Some(raw_phone_id.as_str())) {
        Some(v) => v,
        None => return err_response(StatusCode::SERVICE_UNAVAILABLE, &missing_text("WhatsApp business profile update", &["phone number id"])),
    };

    // Forward only the fields Meta's business_profile endpoint accepts, and
    // only those the caller actually sent.
    let mut body = json!({ "messaging_product": "whatsapp" });
    let mut forwarded: Vec<&str> = Vec::new();
    for key in ["about", "address", "description", "email", "vertical"] {
        if let Some(value) = json_str(payload.get(key)) {
            body[key] = Value::String(value);
            forwarded.push(key);
        }
    }
    if let Some(sites) = payload.get("websites").and_then(|v| v.as_array()) {
        if !sites.is_empty() {
            body["websites"] = Value::Array(sites.clone());
            forwarded.push("websites");
        }
    }

    if forwarded.is_empty() {
        return err_response(
            StatusCode::BAD_REQUEST,
            "No WhatsApp business-profile field supplied. Send at least one of about, address, description, email, vertical, websites.",
        );
    }

    let version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        state.config.meta_whatsapp_api_version.as_str()
    };
    let url = format!("https://graph.facebook.com/{version}/{phone_id}/business_profile");

    match http_post_json(&state, &url, &body, Some(&token)).await {
        Ok((status_code, response, _raw)) if status_code < 300 => Json(json!({
            "status": "ok",
            "profile": payload,
            "forwardedFields": forwarded,
            "metaSynced": true,
            "httpStatus": status_code,
            "providerResponse": response,
        }))
        .into_response(),
        Ok((status_code, response, raw)) => {
            warn!("Meta rejected the business profile update (HTTP {status_code}): {raw}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "failed",
                    "profile": payload,
                    "forwardedFields": forwarded,
                    "metaSynced": false,
                    "httpStatus": status_code,
                    "error": format!("Meta returned HTTP {status_code}: {raw}"),
                    "providerResponse": response,
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Meta for the business profile update: {err}"),
        ),
    }
}

pub async fn register_whatsapp_phone(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pin = match request_str(&payload, &["pin"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Refusing to register: payload must carry the real 4-digit `pin` Meta sent to the number. No default is used.",
            )
        }
    };

    let (raw_token, raw_phone_id) = crate::services::meta_sync::resolve_meta_credentials(&state).await;
    let token = match usable(Some(raw_token.as_str())) {
        Some(v) => v,
        None => return err_response(StatusCode::SERVICE_UNAVAILABLE, &missing_text("WhatsApp number registration", &["meta access token"])),
    };
    let phone_id = match usable(Some(raw_phone_id.as_str())) {
        Some(v) => v,
        None => return err_response(StatusCode::SERVICE_UNAVAILABLE, &missing_text("WhatsApp number registration", &["phone number id"])),
    };

    let version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        state.config.meta_whatsapp_api_version.as_str()
    };
    let method = request_str(&payload, &["method"]).unwrap_or_else(|| "sms".to_string());
    let body = json!({
        "messaging_product": "whatsapp",
        "pin": pin,
        "method": method
    });
    let url = format!("https://graph.facebook.com/{version}/{phone_id}/register");

    match http_post_json(&state, &url, &body, Some(&token)).await {
        Ok((status_code, response, _raw)) if status_code < 300 => {
            Json(json!({
                "status": "registered",
                "httpStatus": status_code,
                "providerResponse": response,
                "message": "Meta accepted the registration request; verification status must be read back from the phone-number node."
            }))
            .into_response()
        }
        Ok((status_code, response, raw)) => {
            warn!("Meta rejected the number registration (HTTP {status_code}): {raw}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "failed",
                    "httpStatus": status_code,
                    "providerResponse": response,
                    "error": format!("Meta returned HTTP {status_code}: {raw}"),
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Meta to register the number: {err}"),
        ),
    }
}

// ---------------------------------------------------------
// 3. Live Chat Channel & Pre-Chat
// ---------------------------------------------------------

pub async fn get_livechat_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["website", "live-chat", "live_chat"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let serialized_inbox = inbox
        .as_ref()
        .map(|i| serialize_inbox(i))
        .unwrap_or(Value::Null);
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);

    let website_token = cred(&creds, &["websiteToken", "website_token", "siteToken"]);
    let domain = cred(&creds, &["websiteDomain", "website_domain", "domain"]);
    let widget_color = cred(&creds, &["widgetColor", "widget_color"]);
    let greeting_title = cred(&creds, &["greetingTitle", "greeting_title"]);
    let greeting_tagline = cred(&creds, &["greetingTagline", "greeting_tagline"]);
    let widget_position = cred(&creds, &["widgetPosition", "widget_position"]);
    let avatar_url = inbox
        .as_ref()
        .and_then(|i| usable(i.avatar_url.as_deref()))
        .or_else(|| cred(&creds, &["avatarUrl", "avatar_url"]));

    let base = public_base_url(&state);
    let embed_script = match (&base, &website_token) {
        (Some(host), Some(token)) => Some(format!(
            "<script defer src=\"{host}/assets/widget.js\" data-token=\"{token}\"></script>"
        )),
        _ => None,
    };
    if embed_script.is_none() {
        warn!(
            "Live-chat embed code unavailable: public base URL ({}) and website token ({}) must both be configured",
            if base.is_some() { "set" } else { "unset" },
            if website_token.is_some() { "set" } else { "unset" }
        );
    }

    let stored_prechat = creds.get("prechat").cloned().unwrap_or(Value::Null);
    let prechat_enabled = stored_prechat
        .get("isEnabled")
        .or_else(|| stored_prechat.get("enabled"))
        .and_then(|v| v.as_bool());
    let has_embed = embed_script.is_some();

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = website_token.is_some();
    let missing: Vec<&str> = if !configured { vec!["websiteToken"] } else { vec![] };

    Json(json!({
        "channel": "live-chat",
        "inbox": serialized_inbox,
        "warnings": warnings,
        "websiteToken": value_or_null(website_token),
        "websiteDomain": value_or_null(domain),
        "embedScript": value_or_null(embed_script),
        "embedScriptNote": if has_embed { "" } else {
            "embedScript is null: PUBLIC_URL (or PUBLIC_BASE_URL) and the widget token must both be configured."
        },
        "branding": {
            "widgetColor": value_or_null(widget_color),
            "greetingTitle": value_or_null(greeting_title),
            "greetingTagline": value_or_null(greeting_tagline),
            "widgetPosition": value_or_null(widget_position),
            "avatarUrl": value_or_null(avatar_url)
        },
        "preChatForm": stored_prechat,
        "preChatEnabled": prechat_enabled,
        "health": {
            "isConnected": active && configured,
            "configured": configured,
            "inboxActive": active,
            // A website inbox has no provider to hold a connection open with:
            // the two facts below are the inbox row's `is_active` flag and a
            // widget token read from its credentials. `CONNECTED` would claim an
            // outbound link that this build does not have, so the label states
            // the observed condition instead.
            "status": if !active { "DISABLED" } else if configured { "CONFIGURED_INBOUND_ONLY" } else { "NOT_CONFIGURED" },
            "statusLabel": if !active {
                "Inbox disabled".to_string()
            } else if !configured {
                honest_label(active, configured).to_string()
            } else {
                "Widget token present and the inbox is enabled; no provider was contacted".to_string()
            },
            "outboundTransport": "none",
            "transportWarning": if active && configured {
                "No outbound transport for live-chat: no websocket hub is mounted and a website inbox row carries no push endpoint, so a stored agent reply reaches the visitor only when the widget polls GET /widget/conversations/<id>/messages/since. Nothing sent on this channel is confirmed delivered."
            } else {
                ""
            },
            "verifiedByLiveProbe": false,
            "probeNote": "There is no live-chat endpoint to probe: the widget is browser-side, so no provider can confirm this channel from the server.",
            "missingCredentials": missing,
            "connectionError": if active && configured { Value::Null } else { Value::String(missing_text("live-chat", &missing)) }
        }
    }))
}

pub async fn update_livechat_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["website", "live-chat", "live_chat"],
        &payload,
        &[
            ("website_token", "websiteToken"),
            ("website_domain", "websiteDomain"),
            ("widget_color", "widgetColor"),
            ("greeting_title", "greetingTitle"),
            ("greeting_tagline", "greetingTagline"),
            ("widget_position", "widgetPosition"),
        ],
    )
    .await;

    write_response("live-chat", write, get_livechat_channel(State(state)).await.0)
}

pub async fn regenerate_livechat_token(State(state): State<AppState>) -> impl IntoResponse {
    let existing = match inbox_row_checked(&state.db, &["website", "live-chat", "live_chat"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if existing.is_none() {
        return err_response(
            StatusCode::NOT_FOUND,
            "No live-chat inbox exists, so there is nothing to attach a new widget token to.",
        );
    }

    let new_tok = format!("tok_{}", Uuid::new_v4().simple());
    let (report, stored) = match set_credential_key(
        &state.db,
        &["website", "live-chat", "live_chat"],
        "websiteToken",
        Value::String(new_tok.clone()),
    )
    .await
    {
        Ok(pair) => pair,
        Err(err) => {
            return err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("The new widget token could not be written to the live-chat inbox credentials: database error: {err}"),
            )
        }
    };
    if !report.saved() {
        return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The new widget token was not stored: the live-chat inbox UPDATE matched {} row(s).",
                report.rows_affected
            ),
        );
    }

    let persisted = json_str(stored.as_ref()).filter(|v| *v == new_tok);
    if persisted.is_none() {
        return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The write reported success but the stored credentials do not contain the new token, so it was not persisted.",
        );
    }

    let base = public_base_url(&state);
    let embed_script = base
        .as_ref()
        .map(|host| format!("<script defer src=\"{host}/assets/widget.js\" data-token=\"{new_tok}\"></script>"));

    Json(json!({
        "websiteToken": new_tok,
        "persisted": true,
        "embedScript": value_or_null(embed_script.clone()),
        "embedScriptNote": if embed_script.is_some() { "" } else { "PUBLIC_URL is not configured, so no embed snippet can be built." },
        "message": "Token regenerated and stored on the live-chat inbox."
    }))
    .into_response()
}

pub async fn send_livechat_test_visitor(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let msg = match require_body(&payload, &["initial_message", "message", "body"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // A synthetic visitor still needs a real identity; nothing is invented.
    let identity = match request_str(&payload, &["email", "visitorId", "externalId", "name"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Refusing to create a test visitor: payload must carry a real identity (email, visitorId, externalId or name).",
            )
        }
    };
    let display_name = request_str(&payload, &["name"]).unwrap_or_else(|| identity.clone());

    let contact_id = match inbound::get_or_create_contact(
        &state.db,
        "website",
        &identity,
        &display_name,
        &state.config.default_tenant_key,
    )
    .await {
        Ok(id) => id,
        Err(err) => return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "Could not persist the test visitor contact: {}",
                inbound_error_detail(&err)
            ),
        ),
    };

    // A missing inbox row still lets the conversation be created, but an
    // unreadable `inboxes` table is a database failure, not "no inbox row":
    // reported as such rather than silently linking nothing.
    let inbox_id: Option<String> = match sqlx::query("SELECT id FROM inboxes WHERE channel_type IN ('website','live-chat','live_chat') ORDER BY updated_at DESC LIMIT 1")
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => match row.try_get::<String, _>("id") {
            Ok(id) => Some(id),
            Err(err) => return read_failure_response("inboxes", INBOX_LOOKUP_QUERY, &err),
        },
        Ok(None) => None,
        Err(err) => return read_failure_response("inboxes", INBOX_LOOKUP_QUERY, &err),
    };

    let conversation_id = match inbound::get_or_create_inbox_conversation(
        &state.db,
        &contact_id,
        "website",
        inbox_id.as_deref(),
        None,
    )
    .await {
        Ok(id) => id,
        Err(err) => return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "Could not open a conversation for the test visitor: {}",
                inbound_error_detail(&err)
            ),
        ),
    };

    let message_id = match inbound::record_raw_inbound_message(
        &state.db,
        &conversation_id,
        "website",
        &msg,
        "live-chat",
        "",
        &display_name,
        Some(json!({ "source": "test_visitor_intake" })),
        None,
    )
    .await {
        Ok(id) => id,
        Err(err) => return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The visitor message could not be stored: {}",
                inbound_error_detail(&err)
            ),
        ),
    };

    Json(json!({
        "status": "created",
        "conversationId": conversation_id,
        "contactId": contact_id,
        "messageId": message_id,
        "message": msg,
        "visitor": display_name,
        "persisted": true
    }))
    .into_response()
}

pub async fn list_livechat_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["website", "live-chat", "live_chat"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

async fn stored_prechat(state: &AppState) -> Result<Option<Value>, Response> {
    let Some(inbox) = inbox_row_checked(&state.db, &["website", "live-chat", "live_chat"]).await? else {
        return Ok(None);
    };
    let creds = parse_json_obj(&inbox.credentials);
    Ok(creds.get("prechat").cloned().filter(|v| v.is_object()))
}

pub async fn get_prechat_config(State(state): State<AppState>) -> impl IntoResponse {
    match stored_prechat(&state).await {
        Ok(Some(config)) => Json(config).into_response(),
        Ok(None) => {
            warn!("No pre-chat configuration has ever been stored on the live-chat inbox");
            Json(json!({
                "isEnabled": false,
                "enabled": false,
                "title": Value::Null,
                "description": Value::Null,
                "submit_button_text": Value::Null,
                "require_email": Value::Null,
                "fields": [],
                "routing_rules": [],
                "note": "Nothing has been saved through PUT /inboxes/prechat-config yet, so no form exists. Empty is the real state."
            }))
            .into_response()
        }
        Err(resp) => resp,
    }
}

pub async fn update_prechat_config(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if !payload.is_object() {
        return err_response(
            StatusCode::BAD_REQUEST,
            "The pre-chat configuration must be a JSON object.",
        );
    }
    match inbox_row_checked(&state.db, &["website", "live-chat", "live_chat"]).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                "There is no live-chat inbox to attach a pre-chat form to.",
            );
        }
        Err(resp) => return resp,
    }

    match set_credential_key(&state.db, &["website", "live-chat", "live_chat"], "prechat", payload).await {
        Ok((report, Some(stored))) if report.saved() => Json(stored).into_response(),
        Ok((report, _)) => err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The pre-chat configuration was not stored: the live-chat inbox UPDATE matched {} row(s).",
                report.rows_affected
            ),
        ),
        Err(err) => err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("The pre-chat configuration could not be written to the live-chat inbox credentials: database error: {err}"),
        ),
    }
}

pub async fn reset_prechat_config(State(state): State<AppState>) -> impl IntoResponse {
    match inbox_row_checked(&state.db, &["website", "live-chat", "live_chat"]).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                "There is no live-chat inbox whose pre-chat form could be cleared.",
            );
        }
        Err(resp) => return resp,
    }
    let clear = set_credential_key(
        &state.db,
        &["website", "live-chat", "live_chat"],
        "prechat",
        Value::Null,
    )
    .await;
    // A cleared key reads back as absent, so only the row count proves the
    // removal landed; say so instead of showing the form as reset for free.
    let (report, _cleared) = match clear {
        Ok(pair) => pair,
        Err(err) => {
            return err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("The pre-chat configuration could not be cleared: database error: {err}"),
            )
        }
    };
    if !report.saved() {
        return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The pre-chat configuration was not cleared: the live-chat inbox UPDATE matched {} row(s).",
                report.rows_affected
            ),
        );
    }
    get_prechat_config(State(state)).await.into_response()
}

pub async fn test_prechat_intake(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let answers = payload.get("answers").cloned().unwrap_or(Value::Null);
    let config = match stored_prechat(&state).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "No pre-chat form is stored, so there are no routing rules to evaluate. Configure it via PUT /inboxes/prechat-config first.",
            )
        }
        Err(resp) => return resp,
    };

    let rules = config
        .get("routing_rules")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if rules.is_empty() {
        return err_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "The stored pre-chat configuration contains no routing_rules, so no department can be assigned.",
        );
    }

    let matched = rules.iter().find(|rule| {
        let field = rule.get("field").and_then(|v| v.as_str()).unwrap_or_default();
        let expected = rule.get("equals").or_else(|| rule.get("value")).and_then(|v| v.as_str());
        let actual = answers.get(field).and_then(|v| v.as_str());
        match (expected, actual) {
            (Some(want), Some(got)) => want.eq_ignore_ascii_case(got),
            (None, Some(_)) => true,
            _ => false,
        }
    });

    match matched {
        Some(rule) => Json(json!({
            "status": "intake_success",
            "extractedAttributes": answers,
            "assignedDepartment": value_or_null(
                rule.get("department").and_then(|v| v.as_str()).map(|s| s.to_string())
            ),
            "matchedRule": rule,
            "evaluatedRuleCount": rules.len()
        }))
        .into_response(),
        None => Json(json!({
            "status": "no_rule_matched",
            "extractedAttributes": answers,
            "assignedDepartment": Value::Null,
            "evaluatedRuleCount": rules.len()
        }))
        .into_response(),
    }
}

// ---------------------------------------------------------
// 4. SMS Channel
// ---------------------------------------------------------

pub async fn get_sms_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["sms"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let serialized_inbox = inbox
        .as_ref()
        .map(|i| serialize_inbox(i))
        .unwrap_or(Value::Null);
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();
    let (gateway, gateway_warning) = gateway_row_for_view(&state.db, "sms").await;
    if let Some(w) = gateway_warning {
        warnings.push(w);
    }
    let gateway_creds = gateway
        .as_ref()
        .map(|g| parse_json_obj(&g.credentials))
        .unwrap_or_default();
    let creds = merged_creds(&inbox_creds, Some(&gateway_creds));

    let sid = cred(&creds, &["accountSid", "account_sid"])
        .or_else(|| config_val(state.config.twilio_account_sid.clone()));
    let has_token = cred(&creds, &["authToken", "auth_token"])
        .or_else(|| config_val(state.config.twilio_auth_token.clone()))
        .is_some();
    let phone = inbox
        .as_ref()
        .and_then(|i| usable(i.phone_number.as_deref()))
        .or_else(|| cred(&creds, &["fromNumber", "from_number", "smsFallback"]))
        .or_else(|| config_val(state.config.twilio_sms_from.clone()));
    let messaging_service = cred(&creds, &["messagingServiceSid", "messaging_service_sid"])
        .or_else(|| config_val(state.config.twilio_messaging_service_sid.clone()));
    let a2p_status = cred(&creds, &["a2p10dlcStatus", "a2p_10dlc_status"]);
    let brand_name = cred(&creds, &["brandName", "brand_name"]);
    let brand_tier = cred(&creds, &["brandTier", "brand_tier"]);
    let campaign_id = cred(&creds, &["campaignId", "campaign_id"]);
    let opt_out_message = cred(&creds, &["optOutMessage", "opt_out_message"]);
    let opt_in_message = cred(&creds, &["optInMessage", "opt_in_message"]);
    // Keyword lists are JSON arrays in the credentials (the Python adapter reads
    // `creds.get("optOutKeywords", [...])`), so they resolve through `cred_arr`,
    // which only accepts a real stored array.
    let opt_keywords = cred_arr(&creds, &["optOutKeywords", "opt_out_keywords"]);
    let opt_in_keywords = cred_arr(&creds, &["optInKeywords", "opt_in_keywords"]);

    let base = public_base_url(&state);
    let callback_url = base
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/twilio/sms"));

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = sid.is_some() && has_token && (phone.is_some() || messaging_service.is_some());
    let mut missing: Vec<&str> = Vec::new();
    if sid.is_none() {
        missing.push("twilio accountSid");
    }
    if !has_token {
        missing.push("twilio authToken");
    }
    if phone.is_none() && messaging_service.is_none() {
        missing.push("sender number or messagingServiceSid");
    }

    // Opt-out keywords are a compliance requirement of the carrier rules, not a
    // per-inbox setting; report them only as a documented default and make the
    // distinction explicit instead of pretending they were read from Twilio.
    Json(json!({
        "channel": "sms",
        "inbox": serialized_inbox,
        "warnings": warnings,
        "gateway": gateway.as_ref().map(|g| json!({
            "id": g.id,
            "provider": g.provider,
            "name": g.name,
            "testStatus": value_or_null(usable(g.test_status.as_deref())),
            "lastTestedAt": value_or_null(usable(g.last_tested_at.as_deref())),
            "latencyMs": g.latency_ms,
        })),
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured, so the Twilio webhook URL cannot be reported." },
            "httpMethod": "POST",
            "signatureValidation": true,
            "fallbackUrl": Value::Null
        },
        "compliance10dlc": {
            "status": value_or_null(a2p_status.clone()),
            "statusSource": "stored on the inbox/gateway credentials only if it was entered there - it is not read from Twilio",
            "campaignId": value_or_null(campaign_id),
            "brandName": value_or_null(brand_name),
            "brandTier": value_or_null(brand_tier),
            "useCase": Value::Null,
            "optOutKeywords": opt_keywords.clone().unwrap_or_else(|| Value::Array(vec![
                Value::String("STOP".into()), Value::String("CANCEL".into()),
                Value::String("UNSUBSCRIBE".into()), Value::String("QUIT".into()),
                Value::String("END".into())])),
            "optOutKeywordsSource": if opt_keywords.is_some() { "stored on the inbox/gateway credentials" } else { "documented US carrier default - never read from Twilio by this backend" },
            "optInKeywords": opt_in_keywords.clone().unwrap_or_else(|| Value::Array(vec![
                Value::String("START".into()), Value::String("UNSTOP".into())])),
            "optInKeywordsSource": if opt_in_keywords.is_some() { "stored on the inbox/gateway credentials" } else { "documented US carrier default - never read from Twilio by this backend" },
            "optOutMessage": value_or_null(opt_out_message),
            "optInMessage": value_or_null(opt_in_message)
        },
        "health": {
            "isConnected": configured,
            "configured": configured,
            "inboxActive": active,
            "accountSid": value_or_null(sid),
            "phoneNumber": value_or_null(phone),
            "messagingServiceSid": value_or_null(messaging_service),
            "a2p10dlcStatus": value_or_null(a2p_status),
            "status": if !active && inbox.is_some() { "DISABLED" } else if configured { "CONNECTED" } else { "NOT_CONFIGURED" },
            "statusLabel": honest_label(active, configured),
            "verifiedByLiveProbe": false,
            "missingCredentials": missing,
            "connectionError": if configured { Value::Null } else { Value::String(missing_text("sms", &missing)) }
        }
    }))
}

pub async fn update_sms_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["sms"],
        &payload,
        &[
            ("account_sid", "accountSid"),
            ("auth_token", "authToken"),
            ("messaging_service_sid", "messagingServiceSid"),
            ("a2p_10dlc_status", "a2p10dlcStatus"),
            ("brand_name", "brandName"),
            ("brand_tier", "brandTier"),
            ("campaign_id", "campaignId"),
            ("opt_out_message", "optOutMessage"),
            ("opt_in_message", "optInMessage"),
        ],
    )
    .await;

    write_response("sms", write, get_sms_channel(State(state)).await.0)
}

pub async fn verify_sms_connection(State(state): State<AppState>) -> impl IntoResponse {
    let creds = match twilio_creds(&state, &["sms"], "sms").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // Twilio needs HTTP basic auth: a successful answer is proof that the
    // stored sid/token pair really works.
    let account = http_get_json_basic(
        &state,
        &twilio_url(&creds.account_sid, "Account.json"),
        &creds.account_sid,
        &creds.auth_token,
    )
    .await;

    let numbers = http_get_json_basic(
        &state,
        &twilio_url(&creds.account_sid, "IncomingPhoneNumbers.json"),
        &creds.account_sid,
        &creds.auth_token,
    )
    .await;

    let account_ok = account.is_ok();
    let owned: Vec<String> = numbers
        .as_ref()
        .ok()
        .and_then(|v| v.get("incoming_phone_numbers")?.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|n| usable(n.get("phone_number").and_then(|v| v.as_str())))
        .collect();
    let error = account.as_ref().err().or_else(|| numbers.as_ref().err()).cloned();

    if !account_ok {
        warn!("Twilio verification failed for account {}: {:?}", creds.account_sid, error);
    }

    let status = if account_ok { StatusCode::OK } else { StatusCode::BAD_GATEWAY };
    (
        status,
        Json(json!({
            "status": if account_ok { "connected" } else { "failed" },
            "verified": account_ok,
            "accountSid": creds.account_sid,
            "account": account.unwrap_or(Value::Null),
            "ownedNumbers": owned,
            "configuredSender": value_or_null(creds.from_number.clone()),
            "senderIsOwned": creds.from_number.as_ref().map(|f| owned.iter().any(|n| n == f)),
            "messagingServiceSid": value_or_null(creds.messaging_service_sid),
            "a2p10dlcStatus": "Not read from Twilio by this endpoint",
            "error": value_or_null(error),
            "message": if account_ok {
                "Twilio accepted the stored credentials for this account."
            } else {
                "Twilio rejected the stored credentials."
            }
        })),
    )
        .into_response()
}

pub async fn send_sms_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let to = match require_recipient(&payload, &["recipient_phone", "recipient", "to"], "SMS") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let creds = match twilio_creds(&state, &["sms"], "sms").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let mut pairs: Vec<(&str, String)> = vec![("To", to.clone()), ("Body", msg.clone())];
    if let Some(service) = &creds.messaging_service_sid {
        pairs.push(("MessagingServiceSid", service.clone()));
    } else if let Some(from) = &creds.from_number {
        pairs.push(("From", from.clone()));
    }

    let conversation_id =
        match ensure_outbound_conversation(&state.db, "sms", &to, &state.config.default_tenant_key).await {
            Ok(id) => id,
            Err(resp) => return resp,
        };

    match http_post_form(
        &state,
        &twilio_url(&creds.account_sid, "Messages.json"),
        &pairs,
        Some((&creds.account_sid, &creds.auth_token)),
    )
    .await
    {
        Ok((status_code, body, _)) if (200..300).contains(&status_code) => {
            let sid = usable(body.get("sid").and_then(|v| v.as_str()));
            // Twilio normally reports a status in the body; without one the
            // delivery state is unknown, so it is not invented here.
            let status_from_provider = usable(body.get("status").and_then(|v| v.as_str()));
            let provider_status = status_from_provider
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let evidence = SendEvidence::accepted(status_code);
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "sms",
                "twilio",
                &msg,
                sid.as_deref(),
                evidence,
                Some(&body),
            )
            .await;
            Json(json!({
                "status": provider_status,
                "statusSource": status_from_provider.map(|_| "reported by Twilio").unwrap_or("not in the Twilio response; status unknown"),
                "recordedStatus": evidence.message_status(),
                "sid": value_or_null(sid),
                "recipient": to,
                "message": msg,
                "conversationId": conversation_id,
                "messagePersistence": audit,
                "httpStatus": status_code,
                "providerResponse": body
            }))
            .into_response()
        }
        Ok((status_code, body, raw)) => {
            warn!("Twilio rejected the SMS test send (HTTP {status_code}): {raw}");
            let refusal = format!("Twilio returned HTTP {status_code}: {raw}");
            let evidence = SendEvidence::rejected(status_code, &refusal);
            let audit = record_outbound_message(&state.db, &conversation_id, "sms", "twilio", &msg, None, evidence, Some(&body)).await;
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": evidence.message_status(),
                    "recipient": to,
                    "httpStatus": status_code,
                    "error": refusal,
                    "messagePersistence": audit,
                    "providerResponse": body
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Twilio: {err}"),
        ),
    }
}

pub async fn list_sms_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["sms"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 5. Facebook & Instagram
// ---------------------------------------------------------

pub async fn get_facebook_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["facebook"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let serialized_inbox = inbox
        .as_ref()
        .map(|i| serialize_inbox(i))
        .unwrap_or(Value::Null);
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);

    let stored_page_id = cred(&creds, &["pageId", "page_id"]);
    let token = cred(&creds, &["pageAccessToken", "page_access_token", "accessToken"]);
    let verify_token = resolve_opt(
        &state,
        &creds,
        &["webhookVerifyToken", "webhook_verify_token"],
        Some("meta_webhook_verify_token"),
        config_val(state.config.meta_webhook_verify_token.clone()),
        &mut warnings,
    )
    .await;

    // Ask Graph for whatever identity the stored token really grants.
    let probe_target = stored_page_id.clone().unwrap_or_else(|| "me".to_string());
    let live = match &token {
        Some(t) => Some(
            http_get_json(
                &state,
                &format!(
                    "https://graph.facebook.com/{}/{}?fields=id,name,category,verification_status",
                    graph_version(&state),
                    probe_target
                ),
                Some(t),
            )
            .await,
        ),
        None => None,
    };
    let node = live.as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let live_error = live.as_ref().and_then(|r| r.as_ref().err()).cloned();
    if let Some(err) = &live_error {
        warn!("Facebook page read failed: {err}");
    }

    let graph_field = |key: &str| -> Option<String> {
        node.as_ref()
            .and_then(|n| usable(n.get(key).and_then(|v| v.as_str())))
    };
    let page_id = graph_field("id").or(stored_page_id.clone());
    let page_name = cred(&creds, &["pageName", "page_name"]).or_else(|| graph_field("name"));

    let stored_icebreakers = cred_arr(&creds, &["icebreakers", "icebreakerQuestions"]);
    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = page_id.is_some() && token.is_some();
    let mut missing: Vec<&str> = Vec::new();
    if page_id.is_none() {
        missing.push("pageId");
    }
    if token.is_none() {
        missing.push("pageAccessToken");
    }
    let base = public_base_url(&state);
    let callback_url = base
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/meta/facebook"));

    Json(json!({
        "channel": "facebook",
        "inbox": serialized_inbox,
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured, so the webhook URL cannot be reported." },
            "verifyToken": value_or_null(verify_token),
            "signatureAlgorithm": "sha256",
            "fields": ["messages", "messaging_postbacks", "message_reactions"]
        },
        "pageProfile": {
            "pageId": value_or_null(page_id.clone()),
            "pageName": value_or_null(page_name.clone()),
            "category": value_or_null(graph_field("category")),
            "verificationTier": value_or_null(graph_field("verification_status")),
            "source": if node.is_some() { "live Graph API read" } else { "stored credentials only" }
        },
        "icebreakers": stored_icebreakers.unwrap_or_else(|| Value::Array(vec![])),
        "icebreakersNote": "Empty until page-level icebreakers are stored on the inbox credentials; they are not invented.",
        "health": {
            "isConnected": node.is_some(),
            "configured": configured,
            "inboxActive": active,
            "pageId": value_or_null(page_id),
            "pageName": value_or_null(page_name),
            "status": if node.is_some() { "CONNECTED" } else if configured { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if node.is_some() {
                "Page identity confirmed by a live Graph read"
            } else {
                honest_label(active, configured)
            },
            "missingCredentials": missing,
            "connectionError": value_or_null(live_error),
            "providerFields": node.unwrap_or(Value::Null)
        }
    }))
}

pub async fn update_facebook_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["facebook"],
        &payload,
        &[
            ("page_id", "pageId"),
            ("page_name", "pageName"),
            ("page_access_token", "pageAccessToken"),
            ("app_secret", "appSecret"),
            ("webhook_verify_token", "webhookVerifyToken"),
        ],
    )
    .await;

    write_response("facebook", write, get_facebook_channel(State(state)).await.0)
}

pub async fn verify_facebook_connection(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["facebook"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["pageAccessToken", "page_access_token", "accessToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("facebook", &["page access token (inbox credentials pageAccessToken)"]), &warnings),
            )
        }
    };

    let version = graph_version(&state);
    match http_get_json(
        &state,
        &format!("https://graph.facebook.com/{version}/me?fields=id,name"),
        Some(&token),
    )
    .await
    {
        Ok(identity) => Json(json!({
            "status": "connected",
            "verified": true,
            "pageId": value_or_null(usable(identity.get("id").and_then(|v| v.as_str()))),
            "pageName": value_or_null(usable(identity.get("name").and_then(|v| v.as_str()))),
            "graphApiVersion": version,
            "providerResponse": identity,
            "message": "Graph API accepted this page token."
        }))
        .into_response(),
        Err(err) => {
            warn!("Facebook verification failed: {err}");
            err_response(
                StatusCode::BAD_GATEWAY,
                &format!("Facebook rejected the stored page token: {err}"),
            )
        }
    }
}

pub async fn send_facebook_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let psid = match require_recipient(&payload, &["recipient_psid", "recipient", "psid"], "Facebook Messenger") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox = match inbox_row_checked(&state.db, &["facebook"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["pageAccessToken", "page_access_token", "accessToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("facebook", &["page access token (inbox credentials pageAccessToken)"]), &warnings),
            )
        }
    };
    let messaging_type =
        request_str(&payload, &["messaging_type"]).unwrap_or_else(|| "RESPONSE".to_string());

    let conversation_id = match ensure_outbound_conversation(
        &state.db,
        "facebook",
        &psid,
        &state.config.default_tenant_key,
    )
    .await
    {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match messenger_send(&state, &token, "psid", &psid, &msg, &messaging_type).await {
        Ok((status_code, response)) => {
            let mid = usable(response.get("message_id").and_then(|v| v.as_str()));
            let evidence = SendEvidence::accepted(status_code);
            let recorded = evidence.message_status();
            let mid_note = if mid.is_none() {
                format!("Meta answered HTTP {status_code}, so the send is recorded as `{recorded}`, but its reply carried no message_id: the provider message id is unknown.")
            } else {
                String::new()
            };
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "facebook",
                "facebook",
                &msg,
                mid.as_deref(),
                evidence,
                Some(&response),
            )
            .await;
            Json(json!({
                "status": recorded,
                "httpStatus": status_code,
                "mid": value_or_null(mid),
                "midNote": mid_note,
                "recipient": psid,
                "message": msg,
                "conversationId": conversation_id,
                "messagePersistence": audit,
                "providerResponse": response
            }))
            .into_response()
        }
        Err((code, err)) => {
            warn!("Messenger test send failed: {err}");
            let evidence = match code {
                Some(status_code) => SendEvidence::rejected(status_code, &err),
                None => SendEvidence::not_sent(&err),
            };
            let audit = record_outbound_message(&state.db, &conversation_id, "facebook", "facebook", &msg, None, evidence, None)
                .await;
            let prefix = if code.is_some() { "Meta rejected the Messenger send" } else { "The Messenger send never reached Meta" };
            err_response(
                StatusCode::BAD_GATEWAY,
                &format!("{prefix}: {err}{}", audit_suffix(&audit)),
            )
        }
    }
}

pub async fn list_facebook_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["facebook"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

pub async fn get_instagram_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["instagram"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let serialized_inbox = inbox
        .as_ref()
        .map(|i| serialize_inbox(i))
        .unwrap_or(Value::Null);
    let creds = inbox.as_ref().map(|i| parse_json_obj(&i.credentials)).unwrap_or_default();

    let stored_ig_id = cred(&creds, &["igId", "ig_id", "instagramUserId", "userId"]);
    let token = cred(&creds, &["accessToken", "access_token", "igAccessToken"]);
    let verify_token = resolve_opt(
        &state,
        &creds,
        &["webhookVerifyToken", "webhook_verify_token"],
        Some("meta_webhook_verify_token"),
        config_val(state.config.meta_webhook_verify_token.clone()),
        &mut warnings,
    )
    .await;

    let probe_target = stored_ig_id.clone().unwrap_or_else(|| "me".to_string());
    let live = match &token {
        Some(t) => Some(
            http_get_json(
                &state,
                &format!(
                    "https://graph.facebook.com/{}/{}?fields=id,username,account_type,verification_status",
                    graph_version(&state),
                    probe_target
                ),
                Some(t),
            )
            .await,
        ),
        None => None,
    };
    let node = live.as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let live_error = live.as_ref().and_then(|r| r.as_ref().err()).cloned();
    if let Some(err) = &live_error {
        warn!("Instagram account read failed: {err}");
    }
    let field = |key: &str| -> Option<String> {
        node.as_ref()
            .and_then(|n| usable(n.get(key).and_then(|v| v.as_str())))
    };

    let ig_id = field("id").or(stored_ig_id.clone());
    let ig_username = cred(&creds, &["igUsername", "ig_username", "username"]).or_else(|| field("username"));
    let account_type = cred(&creds, &["accountType"]).or_else(|| field("account_type"));
    let connected_page = cred(&creds, &["connectedFacebookPage", "connected_facebook_page"]);
    let story_reply = cred(&creds, &["storyReplyMessage", "story_reply_message"]);
    let story_enabled = cred_bool(&creds, &["storyMentionsEnabled", "story_mentions_enabled"]);
    let stored_icebreakers = cred_arr(&creds, &["icebreakers", "icebreakerQuestions"]);
    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = ig_id.is_some() && token.is_some();
    let mut missing: Vec<&str> = Vec::new();
    if ig_id.is_none() {
        missing.push("igId");
    }
    if token.is_none() {
        missing.push("accessToken");
    }
    let callback_url = public_base_url(&state)
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/meta/instagram"));

    Json(json!({
        "channel": "instagram",
        "inbox": serialized_inbox,
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured, so the Instagram webhook URL cannot be reported." },
            "verifyToken": value_or_null(verify_token),
            "signatureAlgorithm": "sha256",
            "fields": ["messages", "messaging_postbacks", "story_mentions"]
        },
        "accountProfile": {
            "igUsername": value_or_null(ig_username.clone()),
            "igId": value_or_null(ig_id.clone()),
            "connectedFacebookPage": value_or_null(connected_page),
            "accountType": value_or_null(account_type),
            "source": if node.is_some() { "live Graph API read" } else { "stored credentials only" }
        },
        "storyMentions": {
            "enabled": story_enabled,
            "enabledNote": if story_enabled.is_none() { "No storyMentionsEnabled value is stored on this inbox." } else { "" },
            "replyMessage": value_or_null(story_reply)
        },
        "icebreakers": stored_icebreakers.unwrap_or_else(|| Value::Array(vec![])),
        "icebreakersNote": "Empty until icebreakers are stored on the inbox credentials; they are not invented.",
        "health": {
            "isConnected": node.is_some(),
            "configured": configured,
            "inboxActive": active,
            "igId": value_or_null(ig_id),
            "igUsername": value_or_null(ig_username),
            "status": if node.is_some() { "CONNECTED" } else if configured { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if node.is_some() {
                "Account identity confirmed by a live Graph read"
            } else {
                honest_label(active, configured)
            },
            "missingCredentials": missing,
            "connectionError": value_or_null(live_error),
            "providerFields": node.unwrap_or(Value::Null)
        }
    }))
}

pub async fn update_instagram_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["instagram"],
        &payload,
        &[
            ("ig_id", "igId"),
            ("ig_username", "igUsername"),
            ("access_token", "accessToken"),
            ("app_secret", "appSecret"),
            ("webhook_verify_token", "webhookVerifyToken"),
        ],
    )
    .await;

    write_response("instagram", write, get_instagram_channel(State(state)).await.0)
}

pub async fn verify_instagram_connection(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["instagram"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["accessToken", "access_token", "igAccessToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("instagram", &["access token (inbox credentials accessToken)"]), &warnings),
            )
        }
    };

    let version = graph_version(&state);
    match http_get_json(
        &state,
        &format!("https://graph.facebook.com/{version}/me?fields=id,username,account_type"),
        Some(&token),
    )
    .await
    {
        Ok(identity) => Json(json!({
            "status": "connected",
            "verified": true,
            "igId": value_or_null(usable(identity.get("id").and_then(|v| v.as_str()))),
            "igUsername": value_or_null(usable(identity.get("username").and_then(|v| v.as_str()))),
            "accountType": value_or_null(usable(identity.get("account_type").and_then(|v| v.as_str()))),
            "graphApiVersion": version,
            "providerResponse": identity,
            "message": "Graph API accepted this Instagram token."
        }))
        .into_response(),
        Err(err) => {
            warn!("Instagram verification failed: {err}");
            err_response(
                StatusCode::BAD_GATEWAY,
                &format!("Instagram rejected the stored token: {err}"),
            )
        }
    }
}

pub async fn send_instagram_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let recipient = match require_recipient(
        &payload,
        &["recipient_igsid", "recipient", "recipient_username", "igsid"],
        "Instagram Direct",
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox = match inbox_row_checked(&state.db, &["instagram"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["accessToken", "access_token", "igAccessToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("instagram", &["access token (inbox credentials accessToken)"]), &warnings),
            )
        }
    };

    // Instagram identifies recipients either by IGSID or by username; the
    // payload decides which recipient key Meta receives.
    let by_username = request_str(&payload, &["recipient_username"]).is_some()
        && request_str(&payload, &["recipient_igsid", "igsid"]).is_none();
    let recipient_key = if by_username { "username" } else { "id" };

    let conversation_id = match ensure_outbound_conversation(
        &state.db,
        "instagram",
        &recipient,
        &state.config.default_tenant_key,
    )
    .await
    {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let messaging_type =
        request_str(&payload, &["messaging_type"]).unwrap_or_else(|| "RESPONSE".to_string());
    match messenger_send(&state, &token, recipient_key, &recipient, &msg, &messaging_type).await {
        Ok((status_code, response)) => {
            let mid = usable(response.get("message_id").and_then(|v| v.as_str()));
            let evidence = SendEvidence::accepted(status_code);
            let recorded = evidence.message_status();
            let mid_note = if mid.is_none() {
                format!("Meta answered HTTP {status_code}, so the send is recorded as `{recorded}`, but its reply carried no message_id: the provider message id is unknown.")
            } else {
                String::new()
            };
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "instagram",
                "instagram",
                &msg,
                mid.as_deref(),
                evidence,
                Some(&response),
            )
            .await;
            Json(json!({
                "status": recorded,
                "httpStatus": status_code,
                "mid": value_or_null(mid),
                "midNote": mid_note,
                "recipient": recipient,
                "message": msg,
                "conversationId": conversation_id,
                "messagePersistence": audit,
                "providerResponse": response
            }))
            .into_response()
        }
        Err((code, err)) => {
            warn!("Instagram test send failed: {err}");
            let evidence = match code {
                Some(status_code) => SendEvidence::rejected(status_code, &err),
                None => SendEvidence::not_sent(&err),
            };
            let audit = record_outbound_message(&state.db, &conversation_id, "instagram", "instagram", &msg, None, evidence, None)
                .await;
            let prefix = if code.is_some() { "Meta rejected the Instagram send" } else { "The Instagram send never reached Meta" };
            err_response(
                StatusCode::BAD_GATEWAY,
                &format!("{prefix}: {err}{}", audit_suffix(&audit)),
            )
        }
    }
}

pub async fn list_instagram_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["instagram"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 6. Telegram & LINE
// ---------------------------------------------------------

pub async fn get_telegram_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["telegram"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let creds = inbox.as_ref().map(|i| parse_json_obj(&i.credentials)).unwrap_or_default();
    let bot_token = cred(&creds, &["botToken", "bot_token", "accessToken"]);
    let secret_token = cred(&creds, &["secretToken", "secret_token"]);

    // getMe is the only real source for the bot's username, display name and
    // group privacy setting.
    let live = match &bot_token {
        Some(token) => Some(
            http_get_json(
                &state,
                &format!("https://api.telegram.org/bot{token}/getMe"),
                None,
            )
            .await,
        ),
        None => None,
    };
    let result = live.as_ref().and_then(|r| r.as_ref().ok());
    let live_error = live.as_ref().and_then(|r| r.as_ref().err()).cloned();
    if let Some(err) = &live_error {
        warn!("Telegram getMe failed: {err}");
    }
    let bot = result.and_then(|v| v.get("result"));
    let tg_field = |key: &str| -> Option<String> {
        usable(bot?.get(key).and_then(|v| v.as_str()))
    };

    let username = tg_field("username").map(|u| format!("@{u}"));
    let bot_username = cred(&creds, &["botUsername", "bot_username"]).or(username);
    let display_name = tg_field("first_name");
    let can_join_groups = bot.and_then(|b| b.get("can_join_groups")).and_then(|v| v.as_bool());
    let can_read = bot
        .and_then(|b| b.get("can_read_all_group_messages"))
        .and_then(|v| v.as_bool());
    let supports_inline = bot
        .and_then(|b| b.get("supports_inline_queries"))
        .and_then(|v| v.as_bool());
    let stored_commands = cred_arr(&creds, &["commands", "botCommands"]);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = bot_token.is_some();
    let callback_url = public_base_url(&state)
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/telegram"));

    Json(json!({
        "channel": "telegram",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured, so the Telegram webhook URL cannot be reported." },
            "secretToken": value_or_null(secret_token),
            "security": "HMAC Secret Header"
        },
        "botProfile": {
            "botUsername": value_or_null(bot_username.clone()),
            "botName": value_or_null(display_name),
            "canJoinGroups": can_join_groups,
            "canReadGroupMessages": can_read,
            "supportsInlineQueries": supports_inline,
            "commands": stored_commands.unwrap_or_else(|| Value::Array(vec![])),
            "commandsNote": "Empty unless commands were stored on the inbox credentials; getMe does not return them.",
            "source": if bot.is_some() { "live getMe" } else { "stored credentials only" }
        },
        "health": {
            "isConnected": bot.is_some(),
            "configured": configured,
            "inboxActive": active,
            "botUsername": value_or_null(bot_username),
            "telegramApi": "https://api.telegram.org",
            "status": if bot.is_some() { "CONNECTED" } else if configured { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if bot.is_some() {
                "Bot identity confirmed by getMe"
            } else {
                honest_label(active, configured)
            },
            "missingCredentials": if configured { vec![] } else { vec!["botToken"] },
            "connectionError": value_or_null(live_error),
            "providerFields": result.cloned().unwrap_or(Value::Null)
        }
    }))
}

pub async fn update_telegram_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["telegram"],
        &payload,
        &[
            ("bot_token", "botToken"),
            ("bot_username", "botUsername"),
            ("secret_token", "secretToken"),
        ],
    )
    .await;

    write_response("telegram", write, get_telegram_channel(State(state)).await.0)
}

pub async fn verify_telegram_connection(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["telegram"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["botToken", "bot_token", "accessToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("telegram", &["bot token (inbox credentials botToken)"]), &warnings),
            )
        }
    };

    match http_get_json(&state, &format!("https://api.telegram.org/bot{token}/getMe"), None).await {
        Ok(response) => {
            // Only a `result` object is proof of an identity; a 2xx without one
            // is reported as unverified instead of becoming "connected".
            let bot = response.get("result").filter(|v| v.is_object()).cloned();
            let api_ok = response.get("ok").and_then(|v| v.as_bool());
            let verified = api_ok == Some(true) && bot.is_some();
            let username = bot
                .as_ref()
                .and_then(|b| usable(b.get("username").and_then(|v| v.as_str())))
                .map(|u| format!("@{u}"));
            if !verified {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({
                        "status": "unverified",
                        "verified": false,
                        "telegramOk": value_or_null(api_ok.map(|v| v.to_string())),
                        "botUsername": Value::Null,
                        "telegramApi": "https://api.telegram.org",
                        "providerResponse": response,
                        "error": "Telegram answered without `ok: true` and a bot `result` object, so the stored bot token is not proven valid.",
                    })),
                )
                    .into_response();
            }
            let bot = bot.unwrap_or(Value::Null);
            Json(json!({
                "status": "connected",
                "verified": true,
                "botUsername": value_or_null(username),
                "telegramApi": "https://api.telegram.org",
                "providerResponse": bot,
                "message": "Telegram getMe accepted the stored bot token."
            }))
            .into_response()
        }
        Err(err) => {
            warn!("Telegram verification failed: {err}");
            err_response(
                StatusCode::BAD_GATEWAY,
                &format!("Telegram rejected the stored bot token: {err}"),
            )
        }
    }
}

pub async fn send_telegram_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let chat_id = match require_recipient(&payload, &["recipient_chat_id", "chat_id", "recipient"], "Telegram") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox = match inbox_row_checked(&state.db, &["telegram"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["botToken", "bot_token", "accessToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("telegram", &["bot token (inbox credentials botToken)"]), &warnings),
            )
        }
    };

    let conversation_id = match ensure_outbound_conversation(
        &state.db,
        "telegram",
        &chat_id,
        &state.config.default_tenant_key,
    )
    .await
    {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let body = json!({ "chat_id": chat_id, "text": msg });
    match http_post_json(
        &state,
        &format!("https://api.telegram.org/bot{token}/sendMessage"),
        &body,
        None,
    )
    .await
    {
        Ok((status_code, response, raw)) if (200..300).contains(&status_code) => {
            let result = response.get("result").filter(|v| v.is_object()).cloned();
            let mid = result
                .as_ref()
                .and_then(|v| v.get("message_id"))
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string());
            // Telegram answers HTTP 200 with `ok: false` for most refusals, so
            // the flag -- not the status code -- decides whether the message was
            // taken, exactly like the telegram adapter in services/channels.rs.
            let refused = response.get("ok").and_then(Value::as_bool) == Some(false);
            let refusal_text = if refused {
                match response.get("description").and_then(Value::as_str) {
                    Some(description) => format!("Telegram returned HTTP {status_code} with ok=false: {description}"),
                    None => format!("Telegram returned HTTP {status_code} with ok=false and no description: {raw}"),
                }
            } else {
                String::new()
            };
            let evidence = if refused {
                SendEvidence::rejected(status_code, &refusal_text)
            } else {
                SendEvidence::accepted(status_code)
            };
            let recorded = evidence.message_status();
            let mid_note = if !refused && mid.is_none() {
                format!("Telegram answered HTTP {status_code}, so the send is recorded as `{recorded}`, but its reply carried no result.message_id: provider_message_id is null and the delivered message's own id is unknown.")
            } else {
                String::new()
            };
            let payload = if refused { &response } else { result.as_ref().unwrap_or(&response) };
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "telegram",
                "telegram",
                &msg,
                mid.as_deref(),
                evidence,
                Some(payload),
            )
            .await;
            if refused {
                warn!("Telegram refused the test send inside HTTP {status_code}: {refusal_text}");
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({
                        "status": recorded,
                        "recipient": chat_id,
                        "httpStatus": status_code,
                        "error": refusal_text,
                        "messagePersistence": audit,
                        "providerResponse": response,
                    })),
                )
                    .into_response();
            }
            Json(json!({
                "status": recorded,
                "statusNote": mid_note,
                "messageId": value_or_null(mid),
                "recipient": chat_id,
                "message": msg,
                "conversationId": conversation_id,
                "messagePersistence": audit,
                "providerResponse": result.unwrap_or(Value::Null)
            }))
            .into_response()
        }
        Ok((status_code, response, raw)) => {
            warn!("Telegram rejected the test send (HTTP {status_code}): {raw}");
            let refusal = format!("Telegram returned HTTP {status_code}: {raw}");
            let evidence = SendEvidence::rejected(status_code, &refusal);
            let audit = record_outbound_message(&state.db, &conversation_id, "telegram", "telegram", &msg, None, evidence, Some(&response))
                .await;
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": evidence.message_status(),
                    "recipient": chat_id,
                    "httpStatus": status_code,
                    "error": refusal,
                    "messagePersistence": audit,
                    "providerResponse": response
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach the Telegram Bot API: {err}"),
        ),
    }
}

pub async fn list_telegram_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["telegram"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

pub async fn get_line_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["line"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let creds = inbox.as_ref().map(|i| parse_json_obj(&i.credentials)).unwrap_or_default();
    let token = cred(&creds, &["channelAccessToken", "channel_access_token"]);

    // LINE's bot info endpoint is the only real source for display name,
    // primary status and the account id.
    let live = match &token {
        Some(t) => Some(http_get_json(&state, "https://api.line.me/v2/bot/info", Some(t)).await),
        None => None,
    };
    let bot = live.as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let live_error = live.as_ref().and_then(|r| r.as_ref().err()).cloned();
    if let Some(err) = &live_error {
        warn!("LINE bot info read failed: {err}");
    }
    let bot_field = |key: &str| -> Option<String> {
        bot.as_ref().and_then(|b| usable(b.get(key).and_then(|v| v.as_str())))
    };

    let stored_channel_id = cred(&creds, &["channelId", "channel_id"]);
    let channel_id = bot_field("userId").or(stored_channel_id.clone());
    let line_account_id = cred(&creds, &["lineAccountId", "line_account_id"]).or_else(|| bot_field("userId"));
    let display_name = bot_field("displayName");
    let verified = bot.as_ref().and_then(|b| b.get("verified")).and_then(|v| v.as_bool());
    let account_type = cred(&creds, &["accountType"]);
    let quick_replies = cred_arr(&creds, &["quickReplies", "quick_replies"]);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = token.is_some();
    let callback_url = public_base_url(&state)
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/line"));

    Json(json!({
        "channel": "line",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured, so the LINE webhook URL cannot be reported." },
            "signatureValidation": true,
            "signatureHeader": "x-line-signature"
        },
        "accountProfile": {
            "channelId": value_or_null(channel_id.clone()),
            "lineAccountId": value_or_null(line_account_id.clone()),
            "displayName": value_or_null(display_name),
            "verifiedByLine": verified,
            "accountType": value_or_null(account_type),
            "accountTypeNote": "Only reported when it was stored on the inbox credentials; LINE does not expose a free-form account type.",
            "quickReplies": quick_replies.unwrap_or_else(|| Value::Array(vec![])),
            "quickRepliesNote": "Empty until quick replies are stored on the inbox credentials.",
            "source": if bot.is_some() { "live /v2/bot/info" } else { "stored credentials only" }
        },
        "health": {
            "isConnected": bot.is_some(),
            "configured": configured,
            "inboxActive": active,
            "channelId": value_or_null(channel_id),
            "lineAccountId": value_or_null(line_account_id),
            "status": if bot.is_some() { "CONNECTED" } else if configured { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if bot.is_some() {
                "Bot identity confirmed by LINE"
            } else {
                honest_label(active, configured)
            },
            "missingCredentials": if configured { vec![] } else { vec!["channelAccessToken"] },
            "connectionError": value_or_null(live_error),
            "providerFields": bot.unwrap_or(Value::Null)
        }
    }))
}

pub async fn update_line_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["line"],
        &payload,
        &[
            ("channel_id", "channelId"),
            ("channel_secret", "channelSecret"),
            ("channel_access_token", "channelAccessToken"),
            ("line_account_id", "lineAccountId"),
        ],
    )
    .await;

    write_response("line", write, get_line_channel(State(state)).await.0)
}

pub async fn verify_line_connection(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["line"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["channelAccessToken", "channel_access_token"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("line", &["channel access token (inbox credentials channelAccessToken)"]), &warnings),
            )
        }
    };

    match http_get_json(&state, "https://api.line.me/v2/bot/info", Some(&token)).await {
        Ok(info) => Json(json!({
            "status": "connected",
            "verified": true,
            "channelId": value_or_null(usable(info.get("userId").and_then(|v| v.as_str()))),
            "displayName": value_or_null(usable(info.get("displayName").and_then(|v| v.as_str()))),
            "lineApi": "https://api.line.me/v2/bot",
            "providerResponse": info,
            "message": "LINE accepted the stored channel access token."
        }))
        .into_response(),
        Err(err) => {
            warn!("LINE verification failed: {err}");
            err_response(
                StatusCode::BAD_GATEWAY,
                &format!("LINE rejected the stored channel token: {err}"),
            )
        }
    }
}

pub async fn send_line_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = match require_recipient(&payload, &["recipient_user_id", "recipient", "userId"], "LINE") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox = match inbox_row_checked(&state.db, &["line"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["channelAccessToken", "channel_access_token"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes(missing_text("line", &["channel access token (inbox credentials channelAccessToken)"]), &warnings),
            )
        }
    };

    let conversation_id = match ensure_outbound_conversation(
        &state.db,
        "line",
        &user_id,
        &state.config.default_tenant_key,
    )
    .await
    {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let body = json!({
        "to": user_id,
        "messages": [{ "type": "text", "text": msg }]
    });
    match http_post_json(&state, "https://api.line.me/v2/bot/message/push", &body, Some(&token)).await {
        Ok((status_code, response, _)) if (200..300).contains(&status_code) => {
            // LINE returns its delivery id only in the `x-line-request-id`
            // response header, and `http_post_json` hands back the status code,
            // the parsed body and the body text -- not headers -- so the id
            // stays null instead of being invented.
            let evidence = SendEvidence::accepted(status_code);
            let audit = record_outbound_message(&state.db, &conversation_id, "line", "line", &msg, None, evidence, None).await;
            Json(json!({
                "status": evidence.message_status(),
                "messageId": Value::Null,
                "messageIdNote": "LINE does not return a message id in the push response body; the delivery id arrives in the x-line-request-id response header, which this route's HTTP helper does not expose, so no id is stored and the LINE request id is unknown.",
                "recipient": user_id,
                "message": msg,
                "conversationId": conversation_id,
                "messagePersistence": audit,
                "httpStatus": status_code,
                "providerResponse": response
            }))
            .into_response()
        }
        Ok((status_code, response, raw)) => {
            warn!("LINE rejected the test send (HTTP {status_code}): {raw}");
            let refusal = format!("LINE returned HTTP {status_code}: {raw}");
            let evidence = SendEvidence::rejected(status_code, &refusal);
            let audit = record_outbound_message(&state.db, &conversation_id, "line", "line", &msg, None, evidence, Some(&response)).await;
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": evidence.message_status(),
                    "recipient": user_id,
                    "httpStatus": status_code,
                    "error": refusal,
                    "messagePersistence": audit,
                    "providerResponse": response
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach the LINE Messaging API: {err}"),
        ),
    }
}

pub async fn list_line_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["line"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 6b. Discord
// ---------------------------------------------------------

pub async fn get_discord_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["discord"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let creds = inbox.as_ref().map(|i| parse_json_obj(&i.credentials)).unwrap_or_default();
    let token = cred(&creds, &["botToken", "bot_token"]);
    let client_id = cred(&creds, &["clientId", "client_id", "applicationId", "application_id"]);

    // Fetch bot profile from Discord API
    let live = match &token {
        Some(t) => Some(http_get_json_auth(&state, "https://discord.com/api/v10/@me", &format!("Bot {t}")).await),
        None => None,
    };
    let bot_profile = live.as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let live_error = live.as_ref().and_then(|r| r.as_ref().err()).cloned();

    let bot_username = bot_profile
        .as_ref()
        .and_then(|b| b.get("username").and_then(|v| v.as_str()))
        .map(String::from);
    let bot_discriminator = bot_profile
        .as_ref()
        .and_then(|b| b.get("discriminator").and_then(|v| v.as_str()))
        .map(String::from);
    let bot_avatar = bot_profile
        .as_ref()
        .and_then(|b| b.get("avatar").and_then(|v| v.as_str()))
        .map(String::from);
    let bot_id = bot_profile
        .as_ref()
        .and_then(|b| b.get("id").and_then(|v| v.as_str()))
        .map(String::from);

    // Load commands
    let commands = sqlx::query(
        "SELECT id, name, description, is_active FROM discord_commands WHERE organization_id = 'org-default' ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let commands_json: Vec<serde_json::Value> = commands
        .iter()
        .map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "name": row.try_get::<String, _>(1).unwrap_or_default(),
                "description": row.try_get::<String, _>(2).unwrap_or_default(),
                "isActive": row.try_get::<bool, _>(3).unwrap_or(true),
            })
        })
        .collect();

    // Load settings
    let settings_row = sqlx::query(
        "SELECT auto_respond, greeting_message, away_message, allowed_channels, blocked_words, log_events \
         FROM discord_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let auto_respond = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<bool, _>(0).ok())
        .unwrap_or(true);
    let greeting_message = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(1).ok())
        .unwrap_or_else(|| "Hello! How can we help you today?".to_string());
    let away_message = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(2).ok())
        .unwrap_or_else(|| "We are currently away. We will respond when we return.".to_string());
    let allowed_channels = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(3).ok())
        .unwrap_or_else(|| "[]".to_string());
    let blocked_words = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(4).ok())
        .unwrap_or_else(|| "[]".to_string());
    let log_events = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<bool, _>(5).ok())
        .unwrap_or(true);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = token.is_some() && client_id.is_some();

    let callback_url = public_base_url(&state)
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/discord"));

    Json(json!({
        "channel": "discord",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured." },
            "interactionSecret": value_or_null(cred(&creds, &["interactionSecret", "interaction_secret"])),
        },
        "botProfile": {
            "botToken": if token.is_some() { Value::String("————".to_string()) } else { Value::Null },
            "clientId": value_or_null(client_id.clone()),
            "botUsername": value_or_null(bot_username),
            "botDiscriminator": value_or_null(bot_discriminator),
            "botAvatar": value_or_null(bot_avatar),
            "botId": value_or_null(bot_id.clone()),
            "source": if bot_profile.is_some() { "live /@me" } else { "stored credentials only" }
        },
        "commands": commands_json,
        "settings": {
            "autoRespond": auto_respond,
            "greetingMessage": greeting_message,
            "awayMessage": away_message,
            "allowedChannels": allowed_channels,
            "blockedWords": blocked_words,
            "logEvents": log_events,
        },
        "health": {
            "isConnected": bot_profile.is_some(),
            "configured": configured,
            "inboxActive": active,
            "botId": value_or_null(bot_id),
            "status": if bot_profile.is_some() { "CONNECTED" } else if configured { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if bot_profile.is_some() {
                "Bot identity confirmed by Discord"
            } else {
                honest_label(active, configured)
            },
            "missingCredentials": if configured { vec![] } else { vec!["botToken", "clientId"] },
            "connectionError": value_or_null(live_error),
        }
    }))
}

pub async fn update_discord_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["discord"],
        &payload,
        &[
            ("bot_token", "botToken"),
            ("client_id", "clientId"),
            ("application_id", "applicationId"),
            ("interaction_secret", "interactionSecret"),
        ],
    )
    .await;

    // Update settings if provided
    if let Some(settings) = payload.get("settings") {
        if let Some(greeting) = settings.get("greetingMessage").and_then(|v| v.as_str()) {
            let safe = greeting.replace('\'', "''");
            let _ = sqlx::query(&format!(
                "UPDATE discord_settings SET greeting_message = '{}', updated_at = (NOW()) WHERE organization_id = 'org-default'",
                safe
            ))
            .execute(&state.db)
            .await;
        }
        if let Some(away) = settings.get("awayMessage").and_then(|v| v.as_str()) {
            let safe = away.replace('\'', "''");
            let _ = sqlx::query(&format!(
                "UPDATE discord_settings SET away_message = '{}', updated_at = (NOW()) WHERE organization_id = 'org-default'",
                safe
            ))
            .execute(&state.db)
            .await;
        }
        if let Some(auto) = settings.get("autoRespond").and_then(|v| v.as_bool()) {
            let _ = sqlx::query(&format!(
                "UPDATE discord_settings SET auto_respond = {}, updated_at = (NOW()) WHERE organization_id = 'org-default'",
                if auto { 1 } else { 0 }
            ))
            .execute(&state.db)
            .await;
        }
    }

    write_response("discord", write, get_discord_channel(State(state)).await.0)
}

pub async fn verify_discord_connection(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["discord"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["botToken", "bot_token"]) {
        Some(t) => t,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Discord bot token is not configured. Set botToken in inbox credentials.",
            )
        }
    };

    match reqwest::Client::new()
        .get("https://discord.com/api/v10/@me")
        .header("Authorization", format!("Bot {token}"))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let body = response.json::<serde_json::Value>().await.unwrap_or(Value::Null);
                Json(json!({
                    "verified": true,
                    "status": "CONNECTED",
                    "bot": body,
                    "message": "Discord bot connection verified successfully."
                }))
                .into_response()
            } else {
                let body = response.text().await.unwrap_or_default();
                err_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("Discord API returned HTTP {}: {}", status, body),
                )
            }
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Discord API: {}", err),
        ),
    }
}

pub async fn send_discord_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["discord"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["botToken", "bot_token"]) {
        Some(t) => t,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Discord bot token is not configured.",
            )
        }
    };
    let channel_id = payload
        .get("channelId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Test message from Communicator Discord integration.");

    if channel_id.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "channelId is required.");
    }

    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    match reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Bot {token}"))
        .json(&json!({ "content": message }))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let body = response.json::<serde_json::Value>().await.unwrap_or(Value::Null);
                Json(json!({
                    "sent": true,
                    "status": "SENT",
                    "messageId": body.get("id").and_then(|v| v.as_str()),
                    "channelId": channel_id,
                    "providerResponse": body
                }))
                .into_response()
            } else {
                let body = response.text().await.unwrap_or_default();
                err_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("Discord API returned HTTP {}: {}", status, body),
                )
            }
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Discord API: {}", err),
        ),
    }
}

pub async fn list_discord_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["discord"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 6c. Slack
// ---------------------------------------------------------

pub async fn get_slack_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["slack"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let creds = inbox.as_ref().map(|i| parse_json_obj(&i.credentials)).unwrap_or_default();
    let bot_token = cred(&creds, &["botToken", "bot_token", "xoxb_token"]);
    let app_token = cred(&creds, &["appToken", "app_token", "xapp_token"]);
    let signing_secret = cred(&creds, &["signingSecret", "signing_secret"]);
    let app_id = cred(&creds, &["appId", "app_id"]);

    // Fetch bot info from Slack API
    let live = match &bot_token {
        Some(t) => Some(http_get_json(&state, "https://slack.com/api/api.test", Some(t)).await),
        None => None,
    };
    let api_test = live.as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let live_error = live
        .as_ref()
        .and_then(|r| r.as_ref().err())
        .map(|e| e.to_string());

    let bot_connected = api_test.as_ref().and_then(|v| v.get("ok")).and_then(|v| v.as_bool()).unwrap_or(false);
    let bot_user_id = api_test
        .as_ref()
        .and_then(|v| v.get("user_id").and_then(|v| v.as_str()))
        .map(String::from);

    // Load commands
    let commands = sqlx::query(
        "SELECT id, name, description, is_active FROM slack_commands WHERE organization_id = 'org-default' ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let commands_json: Vec<serde_json::Value> = commands
        .iter()
        .map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "name": row.try_get::<String, _>(1).unwrap_or_default(),
                "description": row.try_get::<String, _>(2).unwrap_or_default(),
                "isActive": row.try_get::<bool, _>(3).unwrap_or(true),
            })
        })
        .collect();

    // Load settings
    let settings_row = sqlx::query(
        "SELECT auto_respond, greeting_message, away_message, allowed_channels, blocked_words, log_events \
         FROM slack_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let auto_respond = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<bool, _>(0).ok())
        .unwrap_or(true);
    let greeting_message = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(1).ok())
        .unwrap_or_else(|| "Hello! How can we help you today?".to_string());
    let away_message = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(2).ok())
        .unwrap_or_else(|| "We are currently away. We will respond when we return.".to_string());
    let allowed_channels = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(3).ok())
        .unwrap_or_else(|| "[]".to_string());
    let blocked_words = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>(4).ok())
        .unwrap_or_else(|| "[]".to_string());
    let log_events = settings_row
        .as_ref()
        .and_then(|r| r.try_get::<bool, _>(5).ok())
        .unwrap_or(true);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let configured = bot_token.is_some() && signing_secret.is_some();

    let callback_url = public_base_url(&state)
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/slack"));

    Json(json!({
        "channel": "slack",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured." },
            "signingSecret": value_or_null(signing_secret.clone()),
        },
        "botProfile": {
            "botToken": if bot_token.is_some() { Value::String("••••".to_string()) } else { Value::Null },
            "appToken": value_or_null(app_token.clone()),
            "appId": value_or_null(app_id.clone()),
            "botUserId": value_or_null(bot_user_id.clone()),
            "source": if bot_connected { "live api.test" } else { "stored credentials only" }
        },
        "commands": commands_json,
        "settings": {
            "autoRespond": auto_respond,
            "greetingMessage": greeting_message,
            "awayMessage": away_message,
            "allowedChannels": allowed_channels,
            "blockedWords": blocked_words,
            "logEvents": log_events,
        },
        "health": {
            "isConnected": bot_connected,
            "configured": configured,
            "inboxActive": active,
            "botUserId": value_or_null(bot_user_id),
            "status": if bot_connected { "CONNECTED" } else if configured { "UNVERIFIED" } else { "NOT_CONFIGURED" },
            "statusLabel": if bot_connected {
                "Bot connection confirmed by Slack API"
            } else {
                honest_label(active, configured)
            },
            "missingCredentials": if configured { vec![] } else { vec!["botToken", "signingSecret"] },
            "connectionError": value_or_null(live_error),
        }
    }))
}

pub async fn update_slack_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["slack"],
        &payload,
        &[
            ("bot_token", "botToken"),
            ("app_token", "appToken"),
            ("signing_secret", "signingSecret"),
            ("app_id", "appId"),
        ],
    )
    .await;

    // Update settings if provided
    if let Some(settings) = payload.get("settings") {
        if let Some(greeting) = settings.get("greetingMessage").and_then(|v| v.as_str()) {
            let safe = greeting.replace('\'', "''");
            let _ = sqlx::query(&format!(
                "UPDATE slack_settings SET greeting_message = '{}', updated_at = (NOW()) WHERE organization_id = 'org-default'",
                safe
            ))
            .execute(&state.db)
            .await;
        }
        if let Some(away) = settings.get("awayMessage").and_then(|v| v.as_str()) {
            let safe = away.replace('\'', "''");
            let _ = sqlx::query(&format!(
                "UPDATE slack_settings SET away_message = '{}', updated_at = (NOW()) WHERE organization_id = 'org-default'",
                safe
            ))
            .execute(&state.db)
            .await;
        }
        if let Some(auto) = settings.get("autoRespond").and_then(|v| v.as_bool()) {
            let _ = sqlx::query(&format!(
                "UPDATE slack_settings SET auto_respond = {}, updated_at = (NOW()) WHERE organization_id = 'org-default'",
                if auto { 1 } else { 0 }
            ))
            .execute(&state.db)
            .await;
        }
    }

    write_response("slack", write, get_slack_channel(State(state)).await.0)
}

pub async fn verify_slack_connection(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["slack"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let bot_token = match cred(&creds, &["botToken", "bot_token", "xoxb_token"]) {
        Some(t) => t,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Slack bot token is not configured. Set botToken in inbox credentials.",
            )
        }
    };

    match reqwest::Client::new()
        .get("https://slack.com/api/api.test")
        .header("Authorization", format!("Bearer {bot_token}"))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let body = response.json::<serde_json::Value>().await.unwrap_or(Value::Null);
                let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if ok {
                    Json(json!({
                        "verified": true,
                        "status": "CONNECTED",
                        "bot": body,
                        "message": "Slack bot connection verified successfully."
                    }))
                    .into_response()
                } else {
                    err_response(StatusCode::BAD_GATEWAY, "Slack API returned ok=false.")
                }
            } else {
                let body = response.text().await.unwrap_or_default();
                err_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("Slack API returned HTTP {}: {}", status, body),
                )
            }
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Slack API: {}", err),
        ),
    }
}

pub async fn send_slack_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["slack"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let bot_token = match cred(&creds, &["botToken", "bot_token", "xoxb_token"]) {
        Some(t) => t,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Slack bot token is not configured.",
            )
        }
    };
    let channel_id = payload
        .get("channelId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Test message from Communicator Slack integration.");

    if channel_id.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "channelId is required.");
    }

    let form = serde_urlencoded::to_string([("channel", channel_id), ("text", message)])
        .unwrap_or_default();
    match reqwest::Client::new()
        .post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {bot_token}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let body = response.json::<serde_json::Value>().await.unwrap_or(Value::Null);
                let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if ok {
                    Json(json!({
                        "sent": true,
                        "status": "SENT",
                        "timestamp": body.get("ts").and_then(|v| v.as_str()),
                        "channelId": channel_id,
                        "providerResponse": body
                    }))
                    .into_response()
                } else {
                    let err = body.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                    err_response(StatusCode::BAD_GATEWAY, &format!("Slack API error: {}", err))
                }
            } else {
                let body = response.text().await.unwrap_or_default();
                err_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("Slack API returned HTTP {}: {}", status, body),
                )
            }
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Slack API: {}", err),
        ),
    }
}

pub async fn list_slack_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["slack"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 7. Voice & WebRTC
// ---------------------------------------------------------

pub async fn get_voice_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["voice"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();
    let (gateway, gateway_warning) = gateway_row_for_view(&state.db, "voice_sip").await;
    if let Some(w) = gateway_warning {
        warnings.push(w);
    }
    let gateway_creds = gateway
        .as_ref()
        .map(|g| parse_json_obj(&g.credentials))
        .unwrap_or_default();
    let creds = merged_creds(&inbox_creds, Some(&gateway_creds));

    let sip_domain = cred(&creds, &["sipDomain", "sip_domain"]);
    let caller_id = cred(&creds, &["callerId", "caller_id"])
        .or_else(|| inbox.as_ref().and_then(|i| usable(i.phone_number.as_deref())));
    let twiml_app_sid = cred(&creds, &["twimlAppSid", "twiml_app_sid"]);
    let voice_number = cred(&creds, &["voiceNumber", "voice_number"])
        .or_else(|| inbox.as_ref().and_then(|i| usable(i.phone_number.as_deref())));
    let fallback_number = cred(&creds, &["fallbackNumber", "fallback_number"]);
    let voice_url = cred(&creds, &["voiceUrl", "voice_url", "statusCallback"]);
    let webrtc_enabled = cred_bool(&creds, &["webrtcEnabled", "webrtc_enabled"]);
    let ivr_prompt = cred(&creds, &["ivrPrompt", "ivr_prompt"]);
    let ivr_menu = cred_arr(&creds, &["ivrMenu", "ivr_menu"]);
    let supported_codecs = cred_arr(&creds, &["supportedCodecs", "supported_codecs"]);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let mut missing: Vec<&str> = Vec::new();
    if voice_number.is_none() {
        missing.push("voiceNumber (inbox phone_number or credentials.voiceNumber)");
    }
    if twiml_app_sid.is_none() && voice_url.is_none() {
        missing.push("twimlAppSid or voiceUrl");
    }
    let configured = missing.is_empty() && caller_id.is_some();

    Json(json!({
        "channel": "voice",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "gateway": gateway.as_ref().map(|g| json!({
            "id": g.id,
            "provider": g.provider,
            "name": g.name,
            "testStatus": value_or_null(usable(g.test_status.as_deref())),
            "lastTestedAt": value_or_null(usable(g.last_tested_at.as_deref())),
            "latencyMs": g.latency_ms
        })),
        "sipConfig": {
            "sipDomain": value_or_null(sip_domain),
            "twimlAppSid": value_or_null(twiml_app_sid),
            "voiceUrl": value_or_null(voice_url),
            "voiceNumber": value_or_null(voice_number),
            "callerId": value_or_null(caller_id.clone()),
            "fallbackNumber": value_or_null(fallback_number),
            "status": if configured { "configured" } else { "incomplete" },
            "statusNote": "Configuration state only - no SIP trunk was dialled by this endpoint."
        },
        "webrtcConfig": {
            "enabled": webrtc_enabled,
            "enabledNote": if webrtc_enabled.is_none() { "No stored webrtcEnabled value." } else { "" },
            "tokenEndpoint": "/api/v1/inboxes/channels/voice/token",
            "supportedCodecs": supported_codecs.unwrap_or_else(|| Value::Array(vec![]))
        },
        "ivrConfig": {
            "enabled": ivr_prompt.is_some() || ivr_menu.is_some(),
            "prompt": value_or_null(ivr_prompt),
            "menu": ivr_menu.unwrap_or_else(|| Value::Array(vec![])),
            "note": "Only what is stored on the inbox credentials; no menu is assumed."
        },
        "health": {
            "isConnected": configured,
            "configured": configured,
            "inboxActive": active,
            "callerId": value_or_null(caller_id),
            "status": if !active { "DISABLED" } else if configured { "CONNECTED" } else { "NOT_CONFIGURED" },
            "statusLabel": honest_label(active, configured),
            "verifiedByLiveProbe": false,
            "missingCredentials": missing,
            "connectionError": if configured { Value::Null } else { Value::String(missing_text("voice", &missing)) }
        }
    }))
}

pub async fn update_voice_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["voice"],
        &payload,
        &[
            ("account_sid", "accountSid"),
            ("auth_token", "authToken"),
            ("voice_number", "voiceNumber"),
            ("caller_id", "callerId"),
            ("twiml_app_sid", "twimlAppSid"),
            ("sip_domain", "sipDomain"),
            ("ivr_prompt", "ivrPrompt"),
            ("ivr_menu", "ivrMenu"),
            ("webrtc_enabled", "webrtcEnabled"),
        ],
    )
    .await;

    write_response("voice", write, get_voice_channel(State(state)).await.0)
}

pub async fn verify_voice_channel(State(state): State<AppState>) -> impl IntoResponse {
    let creds = match twilio_creds(&state, &["voice"], "voice_sip").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox = match inbox_row_checked(&state.db, &["voice"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();

    let account = http_get_json_basic(
        &state,
        &twilio_url(&creds.account_sid, "Account.json"),
        &creds.account_sid,
        &creds.auth_token,
    )
    .await;
    let caller_ids = http_get_json_basic(
        &state,
        &twilio_url(&creds.account_sid, "OutgoingCallerId.json"),
        &creds.account_sid,
        &creds.auth_token,
    )
    .await;
    let twiml_app_sid = cred(&inbox_creds, &["twimlAppSid", "twiml_app_sid"]);
    let twiml_app = match &twiml_app_sid {
        Some(sid) => Some(http_get_json_basic(
            &state,
            &twilio_url(&creds.account_sid, &format!("Applications/{sid}.json")),
            &creds.account_sid,
            &creds.auth_token,
        )
        .await),
        None => None,
    };

    let confirmed_numbers: Vec<String> = caller_ids
        .as_ref()
        .ok()
        .and_then(|v| v.get("outgoing_caller_ids")?.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|c| usable(c.get("phone_number").and_then(|v| v.as_str())))
        .collect();

    let account_ok = account.is_ok();
    let caller_id_ok = creds
        .from_number
        .as_ref()
        .map(|n| confirmed_numbers.iter().any(|c| c == n));
    let twiml_ok = twiml_app
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false);
    let error = account
        .as_ref()
        .err()
        .or_else(|| caller_ids.as_ref().err())
        .or_else(|| twiml_app.as_ref().and_then(|r| r.as_ref().err()))
        .cloned();

    if let Some(err) = &error {
        warn!("Twilio voice verification failed: {err}");
    }

    let status = if account_ok { StatusCode::OK } else { StatusCode::BAD_GATEWAY };
    (
        status,
        Json(json!({
            "status": if account_ok { "connected" } else { "failed" },
            "verified": account_ok,
            "accountSid": creds.account_sid,
            "account": account.unwrap_or(Value::Null),
            "confirmedCallerIds": confirmed_numbers,
            "configuredSender": value_or_null(creds.from_number.clone()),
            "callerIdVerified": caller_id_ok,
            "twimlAppSid": value_or_null(twiml_app_sid),
            "twimlAppConfigured": if twiml_app.is_some() { Value::Bool(twiml_ok) } else { Value::Null },
            "twimlApp": twiml_app.and_then(|r| r.ok()).unwrap_or(Value::Null),
            "sipDomainReachable": Value::Null,
            "sipDomainNote": "Not probed: this endpoint has no SIP client, so reachability of a SIP domain cannot be claimed.",
            "error": value_or_null(error),
        })),
    )
        .into_response()
}

pub async fn get_voice_webrtc_token(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let creds = match twilio_creds(&state, &["voice"], "voice_sip").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let identity = match request_str(&payload, &["identity", "agentId", "clientId"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Refusing to mint an access token: the request must name the real participant via `identity`. No placeholder identity is signed.",
            )
        }
    };
    let inbox = match inbox_row_checked(&state.db, &["voice"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();
    let twiml_app_sid = match cred(&inbox_creds, &["twimlAppSid", "twiml_app_sid"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("voice webrtc token", &["twimlAppSid (inbox credentials)"]),
            )
        }
    };

    let ttl = request_i64(&payload, &["ttl"]).unwrap_or(3600);
    match twilio_access_token(&creds.account_sid, &creds.auth_token, &identity, &twiml_app_sid, ttl) {
        Ok(token) => Json(json!({
            "token": token,
            "identity": identity,
            "twimlAppSid": twiml_app_sid,
            "expiresIn": ttl,
            "algorithm": "HS256"
        }))
        .into_response(),
        Err(err) => err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Could not sign a Twilio access token: {err}"),
        ),
    }
}

pub async fn initiate_voice_call(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let to = match require_recipient(&payload, &["recipient_phone", "to", "recipient"], "voice") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let creds = match twilio_creds(&state, &["voice"], "voice_sip").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let from = match (&creds.from_number, &creds.messaging_service_sid) {
        (Some(number), _) => number.clone(),
        _ => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("voice call", &["an outbound caller-id number (inbox phone_number / callerId)"]),
            )
        }
    };

    let inbox = match inbox_row_checked(&state.db, &["voice"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();
    let voice_url = cred(&inbox_creds, &["voiceUrl", "voice_url"]);
    // Calls.json takes an `ApplicationSid` (or a TwiML `Url`), never a signed
    // capability; `twilio_access_token` is only used by /voice/token.
    let application_sid = cred(&inbox_creds, &["applicationSid", "twimlAppSid"]);

    let conversation_id = match ensure_outbound_conversation(
        &state.db,
        "voice",
        &to,
        &state.config.default_tenant_key,
    )
    .await
    {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // Twilio needs either a TwiML Application SID or a Voice Url; both must be
    // configured, never guessed.
    let mut pairs: Vec<(&str, String)> = vec![("To", to.clone()), ("From", from.clone())];
    if let Some(app) = application_sid {
        pairs.push(("ApplicationSid", app));
    } else if let Some(url) = voice_url {
        pairs.push(("Url", url));
    } else {
        return err_response(
            StatusCode::SERVICE_UNAVAILABLE,
            &missing_text("voice call", &["twimlAppSid or voiceUrl on the voice inbox credentials"]),
        );
    }
    if let Some(trunk) = cred(&inbox_creds, &["trunkSid", "trunk_sid"]) {
        pairs.push(("TrunkSid", trunk));
    }

    match http_post_form(
        &state,
        &twilio_url(&creds.account_sid, "Calls.json"),
        &pairs,
        Some((&creds.account_sid, &creds.auth_token)),
    )
    .await
    {
        Ok((status_code, response, _)) if (200..300).contains(&status_code) => {
            let call_sid = usable(response.get("call_sid").and_then(|v| v.as_str()));
            let provider_status = usable(response.get("status").and_then(|v| v.as_str()))
                .unwrap_or_else(|| "unknown".to_string());
            let body_text = format!("Voice call initiated to {to} (Twilio status: {provider_status})");
            let evidence = SendEvidence::accepted(status_code);
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "voice",
                "twilio",
                &body_text,
                call_sid.as_deref(),
                evidence,
                Some(&response),
            )
            .await;
            Json(json!({
                "status": provider_status,
                "recordedStatus": evidence.message_status(),
                "callSid": value_or_null(call_sid),
                "conversationId": conversation_id,
                "to": to,
                "callerId": from,
                "messagePersistence": audit,
                "httpStatus": status_code,
                "providerResponse": response
            }))
            .into_response()
        }
        Ok((status_code, response, raw)) => {
            warn!("Twilio rejected the outbound call (HTTP {status_code}): {raw}");
            let refusal = format!("Twilio returned HTTP {status_code}: {raw}");
            let evidence = SendEvidence::rejected(status_code, &refusal);
            let audit = record_outbound_message(&state.db, &conversation_id, "voice", "twilio", &format!("Voice call to {to} failed"), None, evidence, Some(&response))
                .await;
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": evidence.message_status(),
                    "to": to,
                    "httpStatus": status_code,
                    "error": refusal,
                    "messagePersistence": audit,
                    "providerResponse": response
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Twilio to place the call: {err}"),
        ),
    }
}

pub async fn hangup_voice_call(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let call_sid = match request_str(&payload, &["callSid", "call_sid", "sid"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::BAD_REQUEST,
                "Refusing to hang up: payload must carry the real Twilio `callSid` returned by the initiate call response.",
            )
        }
    };
    let creds = match twilio_creds(&state, &["voice"], "voice_sip").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let pairs: Vec<(&str, String)> = vec![("Status", "completed".to_string())];
    match http_post_form(
        &state,
        &twilio_url(&creds.account_sid, &format!("Calls/{call_sid}.json")),
        &pairs,
        Some((&creds.account_sid, &creds.auth_token)),
    )
    .await
    {
        Ok((status_code, response, _)) if status_code < 300 => {
            let duration = response.get("duration").and_then(|v| v.as_i64());
            let provider_status = usable(response.get("status").and_then(|v| v.as_str()));
            Json(json!({
                "status": value_or_null(provider_status),
                "callSid": call_sid,
                "httpStatus": status_code,
                "result": {
                    "duration": duration,
                    "durationNote": if duration.is_none() { "Twilio has not reported a duration yet (the call may still be ringing)." } else { "" },
                    "recordingUrl": Value::Null,
                    "recordingNote": "No recording URL is reported unless a recording was requested for this call."
                },
                "providerResponse": response
            }))
            .into_response()
        }
        Ok((status_code, response, raw)) => {
            warn!("Twilio rejected the hangup for {call_sid} (HTTP {status_code}): {raw}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "failed",
                    "callSid": call_sid,
                    "httpStatus": status_code,
                    "error": format!("Twilio returned HTTP {status_code}: {raw}"),
                    "providerResponse": response
                })),
            )
                .into_response()
        }
        Err(err) => err_response(
            StatusCode::BAD_GATEWAY,
            &format!("Could not reach Twilio to end the call: {err}"),
        ),
    }
}

pub async fn get_voice_call_logs(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["voice"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

pub async fn list_voice_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["voice"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 8. Support Email
// ---------------------------------------------------------

pub async fn get_email_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["email"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let inbox_creds = inbox
        .as_ref()
        .map(|i| parse_json_obj(&i.credentials))
        .unwrap_or_default();
    let (gateway, gateway_warning) = gateway_row_for_view(&state.db, "email").await;
    if let Some(w) = gateway_warning {
        warnings.push(w);
    }
    let creds = merged_creds(
        &inbox_creds,
        gateway.as_ref().map(|g| parse_json_obj(&g.credentials)).as_ref(),
    );

    let email_addr = cred(&creds, &["emailFromAddress", "fromAddress", "from_address"]);
    let smtp_host = cred(&creds, &["smtpHost", "smtp_host"]);
    let imap_host = cred(&creds, &["imapHost", "imap_host"]);
    let from_name = cred(&creds, &["emailFromName", "fromName"]);
    let reply_to = cred(&creds, &["emailReplyTo", "replyTo"]);
    let smtp_port = cred_i64(&creds, &["smtpPort", "smtp_port"]);
    let smtp_username = cred(&creds, &["smtpUsername", "smtp_user", "username"]);
    let smtp_password = cred(&creds, &["smtpPassword", "smtp_password", "password"]).is_some();
    let smtp_encryption = cred(&creds, &["smtpEncryption", "smtp_encryption"]);
    let imap_port = cred_i64(&creds, &["imapPort", "imap_port"]);
    let imap_username = cred(&creds, &["imapUsername", "imap_user"]);
    let imap_password = cred(&creds, &["imapPassword", "imap_password"]).is_some();
    let imap_ssl = cred_bool(&creds, &["imapSsl", "imap_ssl"]);
    let imap_folder = cred(&creds, &["imapFolder", "imap_folder"]);
    let postmark_token = cred(&creds, &["postmarkServerToken", "postmark_server_token", "serverToken"]);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let mut missing: Vec<&str> = Vec::new();
    if email_addr.is_none() {
        missing.push("emailFromAddress");
    }
    if smtp_host.is_none() && postmark_token.is_none() {
        missing.push("smtpHost (or postmarkServerToken)");
    }
    if !smtp_password {
        missing.push("smtpPassword");
    }
    if smtp_username.is_none() && postmark_token.is_none() {
        missing.push("smtpUsername");
    }
    let configured = missing.is_empty();

    Json(json!({
        "channel": "email",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "gateway": gateway.as_ref().map(|g| json!({
            "id": g.id,
            "provider": g.provider,
            "name": g.name,
            "testStatus": value_or_null(usable(g.test_status.as_deref())),
            "lastTestedAt": value_or_null(usable(g.last_tested_at.as_deref()))
        })),
        "smtpConfig": {
            "host": value_or_null(smtp_host.clone()),
            "port": smtp_port,
            "portNote": "null means no port was ever stored; nothing is assumed.",
            "username": value_or_null(smtp_username),
            "hasPassword": smtp_password,
            "encryption": value_or_null(smtp_encryption),
            "fromAddress": value_or_null(email_addr.clone()),
            "fromName": value_or_null(from_name),
            "replyTo": value_or_null(reply_to),
            "status": if configured { "configured" } else { "incomplete" },
            "statusNote": "Configuration only - no SMTP handshake was performed by this endpoint."
        },
        "imapConfig": {
            "host": value_or_null(imap_host.clone()),
            "port": imap_port,
            "username": value_or_null(imap_username),
            "hasPassword": imap_password,
            "ssl": imap_ssl,
            "folder": value_or_null(imap_folder),
            "status": if imap_host.is_some() && imap_password { "configured" } else { "incomplete" },
            "statusNote": "Configuration only - this backend has no IMAP client, so mailbox state cannot be read."
        },
        "health": {
            "isConnected": configured,
            "configured": configured,
            "inboxActive": active,
            "emailAddress": value_or_null(email_addr),
            "smtpHost": value_or_null(smtp_host),
            "imapHost": value_or_null(imap_host),
            "postmarkServerTokenPresent": postmark_token.is_some(),
            "status": if !active { "DISABLED" } else if configured { "CONNECTED" } else { "NOT_CONFIGURED" },
            "statusLabel": honest_label(active, configured),
            "verifiedByLiveProbe": false,
            "missingCredentials": missing,
            "connectionError": if configured { Value::Null } else { Value::String(missing_text("email", &missing)) }
        }
    }))
}

pub async fn update_email_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["email"],
        &payload,
        &[
            ("email_from_address", "emailFromAddress"),
            ("email_from_name", "emailFromName"),
            ("email_reply_to", "emailReplyTo"),
            ("smtp_host", "smtpHost"),
            ("smtp_port", "smtpPort"),
            ("smtp_username", "smtpUsername"),
            ("smtp_password", "smtpPassword"),
            ("smtp_encryption", "smtpEncryption"),
            ("imap_host", "imapHost"),
            ("imap_port", "imapPort"),
            ("imap_username", "imapUsername"),
            ("imap_password", "imapPassword"),
            ("imap_ssl", "imapSsl"),
            ("imap_folder", "imapFolder"),
            ("email_signature", "emailSignature"),
            ("email_footer", "emailFooter"),
            ("postmark_server_token", "postmarkServerToken"),
        ],
    )
    .await;

    write_response("email", write, get_email_channel(State(state)).await.0)
}

pub async fn verify_email_channel(State(state): State<AppState>) -> impl IntoResponse {
    let inbox = match inbox_row_checked(&state.db, &["email"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);

    // Postmark is the only provider reachable without an SMTP/IMAP client, and
    // only when a real server token is stored.
    let token = match cred(&creds, &["postmarkServerToken", "postmark_server_token", "serverToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes("Not implemented for generic SMTP/IMAP: this backend has no SMTP or IMAP client, so raw host/port credentials cannot be verified. Store a `postmarkServerToken` on the email inbox to verify through the Postmark HTTP API.".to_string(), &warnings),
            )
        }
    };

    let req = state
        .http_client
        .get("https://api.postmarkapp.com/server")
        .header("X-Postmark-Server-Token", token.as_str());
    let resp = match req.send().await {
        Ok(v) => v,
        Err(err) => {
            warn!("Postmark could not be reached: {err}");
            return err_response(
                StatusCode::BAD_GATEWAY,
                &format!("Could not reach Postmark: {err}"),
            );
        }
    };
    let status_code = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_else(|err| format!("(Postmark response body could not be read: {err})"));
    let parsed: Value = serde_json::from_str(&text).unwrap_or(json!({ "raw": text }));

    if status_code >= 300 {
        warn!("Postmark rejected the stored server token (HTTP {status_code})");
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "status": "failed",
                "verified": false,
                "httpStatus": status_code,
                "smtpVerified": false,
                "imapVerified": false,
                "spfDkimVerified": Value::Null,
                "error": format!("Postmark returned HTTP {status_code}: {}", truncate(&text)),
                "providerResponse": parsed,
            })),
        )
            .into_response();
    }

    let server_name = usable(parsed.get("Name").and_then(|v| v.as_str()));
    Json(json!({
        "status": "connected",
        "verified": true,
        "httpStatus": status_code,
        "smtpVerified": true,
        "smtpVerifiedNote": "Postmark's server API accepted the stored token; this backend performed no direct SMTP handshake.",
        "imapVerified": false,
        "imapNote": "The Postmark server endpoint authenticates the sending token only; inbound IMAP is not polled by this backend.",
        "spfDkimVerified": Value::Null,
        "spfDkimNote": "SPF/DKIM belong to the sending domain, not to this credential pair; run a deliverability check to read them.",
        "smtpHost": value_or_null(usable(parsed.get("SMTPApiRequirements").and_then(|v| v.as_str())).or_else(|| Some("api.postmarkapp.com".to_string()))),
        "serverName": value_or_null(server_name),
        "providerResponse": parsed,
    }))
    .into_response()
}

pub async fn send_email_test_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let recipient = match require_recipient(
        &payload,
        &["recipient_email", "recipient", "to"],
        "email",
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let subject = match require_body(&payload, &["subject"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let msg = match require_body(&payload, &["message", "body", "text"]) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let inbox = match inbox_row_checked(&state.db, &["email"]).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut warnings: Vec<String> = Vec::new();
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);
    let token = match cred(&creds, &["postmarkServerToken", "postmark_server_token", "serverToken"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &with_cred_notes("Not implemented for generic SMTP: no SMTP client exists in this backend. Store a `postmarkServerToken` on the email inbox to send through Postmark.".to_string(), &warnings),
            )
        }
    };
    let from_address = match cred(&creds, &["emailFromAddress", "fromAddress"]) {
        Some(v) => v,
        None => {
            return err_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &missing_text("email send", &["emailFromAddress on the email inbox credentials"]),
            )
        }
    };

    let body = json!({
        "From": from_address,
        "To": recipient,
        "Subject": subject,
        "TextBody": msg,
        "MessageStream": "outbound"
    });

    let conversation_id = match ensure_outbound_conversation(
        &state.db,
        "email",
        &recipient,
        &state.config.default_tenant_key,
    )
    .await
    {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let resp = match state
        .http_client
        .post("https://api.postmarkapp.com/email")
        .header("X-Postmark-Server-Token", token.as_str())
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(v) => v,
        Err(err) => {
            let audit = record_outbound_message(
                &state.db,
                &conversation_id,
                "email",
                "postmark",
                &msg,
                None,
                SendEvidence::not_sent(&format!("No HTTP response from Postmark: {err}")),
                None,
            )
            .await;
            return err_response(
                StatusCode::BAD_GATEWAY,
                &format!("Could not reach Postmark: {err}{}", audit_suffix(&audit)),
            );
        }
    };
    let status_code = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_else(|err| format!("(Postmark response body could not be read: {err})"));
    let parsed: Value = serde_json::from_str(&text).unwrap_or(json!({ "raw": text }));

    if !(200..300).contains(&status_code) {
        warn!("Postmark rejected the test email (HTTP {status_code}): {}", truncate(&text));
        let refusal = format!("Postmark returned HTTP {status_code}: {}", truncate(&text));
        let evidence = SendEvidence::rejected(status_code, &refusal);
        let audit = record_outbound_message(&state.db, &conversation_id, "email", "postmark", &msg, None, evidence, Some(&parsed)).await;
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "status": evidence.message_status(),
                "recipient": recipient,
                "httpStatus": status_code,
                "error": refusal,
                "messagePersistence": audit,
                "providerResponse": parsed,
            })),
        )
            .into_response();
    }

    let provider_message_id = usable(parsed.get("MessageID").and_then(|v| v.as_str()));
    let evidence = SendEvidence::accepted(status_code);
    let audit = record_outbound_message(
        &state.db,
        &conversation_id,
        "email",
        "postmark",
        &format!("{subject}: {msg}"),
        provider_message_id.as_deref(),
        evidence,
        Some(&parsed),
    )
    .await;

    Json(json!({
        "status": evidence.message_status(),
        "statusNote": format!(
            "Postmark answered HTTP {status_code}{}; that proves it took the message, not that it reached an inbox -- delivery is only known from its webhook.",
            match &provider_message_id {
                Some(_) => ", and returned a MessageID",
                None => " without a MessageID, so provider_message_id is null and the provider message id is unknown",
            }
        ),
        "messageId": value_or_null(provider_message_id),
        "conversationId": conversation_id,
        "recipient": recipient,
        "subject": subject,
        "messagePersistence": audit,
        "httpStatus": status_code,
        "providerResponse": parsed
    }))
    .into_response()
}

pub async fn sync_email_imap(State(_state): State<AppState>) -> impl IntoResponse {
    err_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Not implemented for IMAP: this backend has no IMAP client, so a mailbox cannot be polled. Inbound email arrives through the webhook routes instead; reporting a successful sync here would be false.",
    )
}

pub async fn list_email_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["email"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}

// ---------------------------------------------------------
// 9. TikTok Channel
// ---------------------------------------------------------

pub async fn get_tiktok_channel(State(state): State<AppState>) -> Json<Value> {
    let mut warnings: Vec<String> = Vec::new();
    let (inbox, inbox_warning) = inbox_row_for_view(&state.db, &["tiktok"]).await;
    if let Some(w) = inbox_warning {
        warnings.push(w);
    }
    let creds = inbox_credentials(inbox.as_ref(), &mut warnings);

    let app_id = cred(&creds, &["appId", "app_id", "clientKey"]);
    let biz_acc = cred(&creds, &["businessAccountId", "business_account_id"]);
    // Never `tiktok_app_secret`: an app secret is not an access token.
    let access_token = cred(&creds, &["accessToken", "access_token"])
        .or_else(|| tiktok_access_token(&state));
    let access_token_source = if cred(&creds, &["accessToken", "access_token"]).is_some() {
        "inbox credentials"
    } else if access_token.is_some() {
        "config tiktok_access_token (TIKTOK_ACCESS_TOKEN env or vault secret)"
    } else {
        "not set"
    };
    let verify_token = resolve_opt(
        &state,
        &creds,
        &["webhookVerifyToken", "webhook_verify_token"],
        None,
        config_val(state.config.tiktok_webhook_verify_token.clone()),
        &mut warnings,
    )
    .await;
    let account_handle = cred(&creds, &["accountHandle", "account_handle", "username"]);
    let account_type = cred(&creds, &["accountType"]);
    let reply_enabled = cred_bool(&creds, &["commentAutoReplyEnabled", "comment_auto_reply_enabled"]);
    let reply_template = cred(&creds, &["commentAutoReplyText", "comment_auto_reply_text"]);

    let active = inbox.as_ref().map(|i| i.is_active).unwrap_or(false);
    let mut missing: Vec<&str> = Vec::new();
    if app_id.is_none() {
        missing.push("appId");
    }
    if access_token.is_none() {
        missing.push("accessToken (inbox credentials or env TIKTOK_ACCESS_TOKEN)");
    }
    let configured = missing.is_empty();
    let callback_url = public_base_url(&state)
        .as_ref()
        .map(|host| format!("{host}/api/v1/webhooks/tiktok"));

    Json(json!({
        "channel": "tiktok",
        "inbox": inbox.as_ref().map(|i| serialize_inbox(i)).unwrap_or(Value::Null),
        "warnings": warnings,
        "webhookConfig": {
            "callbackUrl": value_or_null(callback_url.clone()),
            "callbackUrlNote": if callback_url.is_some() { "" } else { "PUBLIC_URL is not configured, so the TikTok webhook URL cannot be reported." },
            "verifyToken": value_or_null(verify_token),
            "signatureHeader": "x-tiktok-signature",
            "supportedEvents": ["im.direct_message", "video.comment"]
        },
        "businessProfile": {
            "appId": value_or_null(app_id.clone()),
            "businessAccountId": value_or_null(biz_acc.clone()),
            "accountHandle": value_or_null(account_handle),
            "accountType": value_or_null(account_type),
            "note": "Only values that were actually stored on the inbox credentials are reported; TikTok identity requires an OAuth call this endpoint does not fake."
        },
        "commentAutoReply": {
            "enabled": reply_enabled,
            "enabledNote": if reply_enabled.is_none() { "No stored commentAutoReplyEnabled value." } else { "" },
            "template": value_or_null(reply_template)
        },
        "health": {
            "isConnected": configured,
            "configured": configured,
            "inboxActive": active,
            "appId": value_or_null(app_id),
            "businessAccountId": value_or_null(biz_acc),
            "accessTokenSource": access_token_source,
            "status": if !active { "DISABLED" } else if configured { "CONNECTED" } else { "NOT_CONFIGURED" },
            "statusLabel": honest_label(active, configured),
            "verifiedByLiveProbe": false,
            "verificationNote": "POST /inboxes/channels/tiktok/verify refuses: no TikTok OAuth token check is implemented, so no token here is ever reported as valid.",
            "missingCredentials": missing,
            "connectionError": if configured { Value::Null } else { Value::String(missing_text("tiktok", &missing)) }
        }
    }))
}

pub async fn update_tiktok_channel(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let write = update_inbox_record(
        &state.db,
        &["tiktok"],
        &payload,
        &[
            ("app_id", "appId"),
            ("app_secret", "appSecret"),
            ("access_token", "accessToken"),
            ("business_account_id", "businessAccountId"),
            ("webhook_verify_token", "webhookVerifyToken"),
            ("account_handle", "accountHandle"),
            ("comment_auto_reply_text", "commentAutoReplyText"),
            ("comment_auto_reply_enabled", "commentAutoReplyEnabled"),
        ],
    )
    .await;

    write_response("tiktok", write, get_tiktok_channel(State(state)).await.0)
}

pub async fn verify_tiktok_connection(State(_state): State<AppState>) -> impl IntoResponse {
    err_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Not implemented for tiktok: token verification. TikTok's business/DM APIs require an OAuth2 access-token refresh flow plus a per-business-account id lookup that this backend does not implement; a stored token cannot be proven valid here, so no verification is claimed.",
    )
}

pub async fn send_tiktok_test_message(
    State(_state): State<AppState>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    err_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Not implemented for tiktok: direct-message send. The backend has no working TikTok Direct Messages integration (conversation_id / open_id pairing plus the business message endpoint), so a message cannot actually leave this system. Reporting `delivered` would be false.",
    )
}

pub async fn send_tiktok_test_comment_reply(
    State(_state): State<AppState>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    err_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Not implemented for tiktok: comment reply. Replying to a video comment needs the TikTok Display/Comment API and a valid creator access token; neither the call nor a real reply id exists here.",
    )
}

pub async fn list_tiktok_events(State(state): State<AppState>) -> impl IntoResponse {
    match query_channel_events(&state.db, &["tiktok"]).await {
        Ok(events) => Json(events).into_response(),
        Err(err) => read_failure_response("messages", MESSAGE_EVENTS_QUERY, &err),
    }
}
