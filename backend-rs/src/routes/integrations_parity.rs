//! Real-data parity handlers for the integration surface.
//!
//! Why this file exists: `/integrations/trackers/*`, `/settings` and
//! `/settings/whatsapp/*` are already registered in `main.rs`, but the handlers
//! behind several of those routes answer from literals -- `verified: true` with
//! no Graph call, two canned WhatsApp templates, `Authentication verified` with
//! no HTTP request, an issue key minted from a row count. The functions here do
//! the work for real (or refuse, naming what is missing) so the routes can be
//! re-pointed at them. They are deliberately self-contained: only `AppState`,
//! SQL and the shared `reqwest` client.
//!
//! Rules applied throughout:
//! * a masked value (`••••1234`) is never sent and never persisted;
//! * missing credentials produce 503 naming the exact field and where to set it;
//! * a provider rejection is returned as the provider's own message, not a
//!   generic failure;
//! * nothing is written to the database as "success" unless the provider said
//!   so on the wire.

use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::time::Instant;
use uuid::Uuid;

const PROVIDER_TIMEOUT_SECS: u64 = 8;
const GRAPH_TIMEOUT_SECS: u64 = 15;
const TRACKER_KEYS: &[&str] = &["linear", "github", "jira", "slack"];

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn is_masked(value: &str) -> bool {
    value.contains('•') || value.contains('\u{2022}')
}

/// Safe to transmit: present, non-empty, and not the display mask the UI shows
/// back on the next page load.
fn usable(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !is_masked(trimmed)
}

fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 10 {
        return "••••••••".to_string();
    }
    let head: String = chars[..4].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}••••••••{tail}")
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn parse_object(raw: Option<&str>) -> Value {
    parse_object_checked(raw).0
}

// "column empty" and "column present but unreadable" are different states.
fn parse_object_checked(raw: Option<&str>) -> (Value, Option<String>) {
    let text = match raw {
        Some(text) if !text.trim().is_empty() => text.trim(),
        _ => return (json!({}), None),
    };
    match serde_json::from_str::<Value>(text) {
        Ok(value) if value.is_object() => (value, None),
        Ok(_) => (json!({}), Some("stored value is JSON but not an object".to_string())),
        Err(err) => (json!({}), Some(format!("stored value is not valid JSON: {err}"))),
    }
}

/// First usable string among `keys`, looked up in the credentials object and
/// then in the config object. Returns the key that matched so a refusal can
/// name it.
fn credential(creds: &Value, cfg: &Value, keys: &[&str]) -> Option<(String, String)> {
    for key in keys {
        for source in [creds, cfg] {
            if let Some(raw) = source.get(*key).and_then(|v| v.as_str()) {
                if usable(raw) {
                    return Some(((*key).to_string(), raw.trim().to_string()));
                }
            }
        }
    }
    None
}

/// Pulls the provider's own explanation out of the shapes these APIs actually
/// return: Meta `{error:{message}}`, Slack `{error}`, GitHub `{message}`,
/// Jira `{errorMessages[]}`, Linear `{errors:[{message}]}`.
fn provider_error(body: &Value) -> Option<String> {
    if let Some(message) = body.pointer("/error/message").and_then(|v| v.as_str()) {
        let mut text = message.to_string();
        if let Some(code) = body.pointer("/error/code").and_then(|v| v.as_i64()) {
            text.push_str(&format!(" (code {code})"));
        }
        if let Some(subcode) = body.pointer("/error/error_subcode").and_then(|v| v.as_i64()) {
            text.push_str(&format!(", subcode {subcode}"));
        }
        return Some(text);
    }
    if let Some(message) = body.get("error").and_then(|v| v.as_str()) {
        return Some(message.to_string());
    }
    if let Some(message) = body.get("message").and_then(|v| v.as_str()) {
        return Some(message.to_string());
    }
    if let Some(errors) = body.get("errors").and_then(|v| v.as_array()) {
        let text: Vec<String> = errors
            .iter()
            .filter_map(|e| {
                e.get("message")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| e.as_str().map(|s| s.to_string()))
            })
            .collect();
        if !text.is_empty() {
            return Some(text.join("; "));
        }
    }
    if let Some(messages) = body.get("errorMessages").and_then(|v| v.as_array()) {
        let text: Vec<String> = messages.iter().filter_map(|m| m.as_str()).map(|s| s.to_string()).collect();
        if !text.is_empty() {
            return Some(text.join("; "));
        }
    }
    None
}

fn truncate(input: &str, max_chars: usize) -> String {
    let count = input.chars().count();
    if count <= max_chars {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max_chars).collect();
    out.push('…');
    out
}

/// The single place a live HTTP result becomes JSON, so no handler can claim a
/// status that the transport did not produce.
struct LiveCall {
    ok: bool,
    status: Option<u16>,
    latency_ms: i64,
    body: Value,
    raw: String,
    transport_error: Option<String>,
    body_error: Option<String>,
}

impl LiveCall {
    fn transport(endpoint: &str, err: &reqwest::Error) -> LiveCall {
        let reason = if err.is_timeout() {
            format!("timed out after {PROVIDER_TIMEOUT_SECS}s")
        } else if err.is_connect() {
            "could not connect".to_string()
        } else {
            err.to_string()
        };
        LiveCall {
            ok: false,
            status: None,
            latency_ms: 0,
            body: json!({}),
            raw: String::new(),
            transport_error: Some(format!("{endpoint}: {reason}")),
            body_error: None,
        }
    }

    fn message(&self) -> String {
        if let Some(err) = &self.transport_error {
            return err.clone();
        }
        if let Some(provider) = provider_error(&self.body) {
            return provider;
        }
        match self.status {
            Some(code) if self.ok => match &self.body_error {
                Some(err) => err.clone(),
                None => format!("provider accepted the request with HTTP {code}"),
            },
            Some(code) => {
                let detail = if self.raw.trim().is_empty() {
                    "no body".to_string()
                } else {
                    truncate(&self.raw, 300)
                };
                format!("provider returned HTTP {code}: {detail}")
            }
            None => "no response".to_string(),
        }
    }

    fn json(&self, extra: Value) -> Value {
        let mut base = json!({
            "liveCallMade": self.status.is_some(),
            "callAttempted": true,
            "httpStatus": self.status,
            "latencyMs": self.latency_ms,
        });
        if let Some(obj) = base.as_object_mut() {
            if let Some(error) = &self.transport_error {
                obj.insert("transportError".into(), json!(error));
            }
            if let Some(error) = &self.body_error {
                obj.insert("bodyError".into(), json!(error));
            }
            if let Value::Object(map) = extra {
                for (k, v) in map {
                    obj.insert(k, v);
                }
            }
        }
        base
    }
}

async fn execute(client: &reqwest::Client, builder: reqwest::RequestBuilder, endpoint: String) -> LiveCall {
    let started = Instant::now();
    let request = builder
        .timeout(std::time::Duration::from_secs(PROVIDER_TIMEOUT_SECS))
        .build();
    // `.build()` only fails when a header value is invalid; report that as a
    // transport error naming the endpoint rather than unwrapping or silently
    // skipping the call.
    let request = match request {
        Ok(request) => request,
        Err(err) => return LiveCall::transport(&endpoint, &err),
    };
    match client.execute(request).await {
        Ok(response) => {
            let status = response.status().as_u16();
            let (text, body_error) = match response.text().await {
                Ok(text) => (text, None),
                Err(err) => (
                    String::new(),
                    Some(format!("HTTP {status} reached {endpoint}, but the response body could not be read: {err}")),
                ),
            };
            let body = match serde_json::from_str::<Value>(&text) {
                Ok(value) => value,
                Err(_) => json!({}),
            };
            let body_error = body_error.or_else(|| match text.trim().is_empty() {
                true => Some("the provider returned an empty body".to_string()),
                false if body.is_null() || body.as_object().map(|m| m.is_empty()).unwrap_or(false) => {
                    Some(format!("the provider body is not JSON: {}", truncate(&text, 200)))
                }
                false => None,
            });
            LiveCall {
                ok: (200..300).contains(&status),
                status: Some(status),
                latency_ms: started.elapsed().as_millis() as i64,
                body,
                raw: text,
                transport_error: None,
                body_error,
            }
        }
        Err(err) => LiveCall::transport(&endpoint, &err),
    }
}

// `Ok(0)` is "nothing matched", which is not the same statement as success.
fn persistence_json(result: Result<u64, sqlx::Error>, what: &str) -> Value {
    match result {
        Ok(0) => json!({
            "ok": false,
            "rowsAffected": 0,
            "reason": format!("no row was matched, so {what} was not written"),
        }),
        Ok(rows) => json!({ "ok": true, "rowsAffected": rows }),
        Err(err) => json!({
            "ok": false,
            "reason": format!("database failure while writing {what}: {err}"),
        }),
    }
}

fn db_failure(what: &str, err: &sqlx::Error) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("could not read {what}"),
            "detail": err.to_string(),
            "liveCallMade": false,
        })),
    )
}

fn refuse(endpoint: &str, reason: &str) -> (StatusCode, Value) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        json!({
            "status": "unconfigured",
            "connected": false,
            "liveCallMade": false,
            "endpoint": endpoint,
            "refusal": reason,
            "checkedAt": Utc::now().to_rfc3339(),
        }),
    )
}

// ---------------------------------------------------------------------------
// Issue tracker configurations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
struct TrackerRow {
    id: String,
    provider: String,
    account_name: Option<String>,
    is_enabled: Option<bool>,
    test_mode: Option<bool>,
    credentials: Option<String>,
    config: Option<String>,
    last_tested_at: Option<String>,
    last_test_status: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

const SELECT_TRACKER: &str = "SELECT id, provider, account_name, is_enabled, test_mode, credentials, config, last_tested_at, last_test_status, created_at, updated_at FROM issue_tracker_configs WHERE provider = $1";

impl TrackerRow {
    fn creds(&self) -> Value {
        parse_object(self.credentials.as_deref())
    }

    fn cfg(&self) -> Value {
        parse_object(self.config.as_deref())
    }

    /// Secrets are returned masked, with a `has_*` companion so the page can
    /// still show "configured, ending in …1234".
    fn json(&self) -> Value {
        let (creds, credentials_problem) = parse_object_checked(self.credentials.as_deref());
        let mut masked = Map::new();
        if let Some(obj) = creds.as_object() {
            for (key, value) in obj {
                let is_secret = ["token", "key", "secret", "password"]
                    .iter()
                    .any(|term| key.to_lowercase().contains(term));
                match (is_secret, value.as_str()) {
                    (true, Some(text)) => {
                        masked.insert(key.clone(), json!(mask_secret(text)));
                        masked.insert(format!("has_{key}"), json!(usable(text)));
                        masked.insert(format!("has{}", to_camel_key(key)), json!(usable(text)));
                    }
                    _ => {
                        masked.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        let (config, config_problem) = parse_object_checked(self.config.as_deref());
        json!({
            "id": self.id,
            "provider": self.provider,
            "account_name": self.account_name,
            "accountName": self.account_name,
            "is_enabled": self.is_enabled,
            "isEnabled": self.is_enabled,
            "test_mode": self.test_mode,
            "testMode": self.test_mode,
            "credentials": Value::Object(masked),
            "credentialsUnreadable": credentials_problem,
            "config": config,
            "configUnreadable": config_problem,
            "last_tested_at": self.last_tested_at,
            "lastTestedAt": self.last_tested_at,
            "last_test_status": self.last_test_status,
            "lastTestStatus": self.last_test_status,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        })
    }

    // NULL flags are unknown flags, so live writes stay refused.
    fn is_sandbox(&self) -> bool {
        self.test_mode != Some(false)
    }

    fn is_disabled(&self) -> bool {
        self.is_enabled == Some(false)
    }
}

fn to_camel_key(key: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for ch in key.chars() {
        if ch == '_' {
            upper = true;
            continue;
        }
        out.push(if upper { ch.to_ascii_uppercase() } else { ch });
        upper = false;
    }
    out
}

async fn load_tracker(state: &AppState, provider: &str) -> Result<Option<TrackerRow>, sqlx::Error> {
    sqlx::query_as::<_, TrackerRow>(SELECT_TRACKER)
        .bind(provider)
        .fetch_optional(&state.db)
        .await
}

async fn write_tracker_test_result(
    state: &AppState,
    provider: &str,
    status: &str,
) -> Value {
    let result = sqlx::query("UPDATE issue_tracker_configs SET last_tested_at = $1, last_test_status = $2 WHERE provider = $3")
        .bind(Utc::now().to_rfc3339())
        .bind(status)
        .bind(provider)
        .execute(&state.db)
        .await
        .map(|result| result.rows_affected());
    persistence_json(result, "issue_tracker_configs.last_test_status")
}

/// GET /integrations/trackers/configs
pub async fn list_tracker_configs(State(state): State<AppState>) -> Response {
    let rows = match sqlx::query_as::<_, TrackerRow>(
        "SELECT id, provider, account_name, is_enabled, test_mode, credentials, config, last_tested_at, last_test_status, created_at, updated_at FROM issue_tracker_configs ORDER BY provider ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_failure("issue_tracker_configs", &err).into_response(),
    };

    // Strict parity with the Python route: a bare array of masked configs. The
    // sandbox state is carried per provider in `test_mode`, and
    // POST /test/{provider} answers 503 while it is true.
    Json(rows.iter().map(|r| r.json()).collect::<Vec<Value>>()).into_response()
}

/// GET /integrations/trackers/configs/{provider}
pub async fn get_tracker_config(State(state): State<AppState>, Path(provider): Path<String>) -> Response {
    let row = match load_tracker(&state, &provider).await {
        Ok(row) => row,
        Err(err) => return db_failure("issue_tracker_configs", &err).into_response(),
    };
    match row {
        Some(row) => Json(row.json()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("no configuration stored for provider {provider}"),
                "knownProviders": TRACKER_KEYS,
                "hint": format!("save credentials with PUT /integrations/trackers/configs/{provider}"),
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct TrackerConfigPayload {
    #[serde(default, alias = "accountName")]
    pub account_name: Option<String>,
    #[serde(default, alias = "isEnabled")]
    pub is_enabled: Option<bool>,
    #[serde(default, alias = "testMode")]
    pub test_mode: Option<bool>,
    #[serde(default)]
    pub credentials: Option<Value>,
    #[serde(default)]
    pub config: Option<Value>,
}

/// POST|PUT /integrations/trackers/configs/{provider}
///
/// Credentials merge field-by-field. A value that is still masked -- which is
/// what the page has in its inputs until the operator retypes -- is skipped and
/// reported, never written: persisting `lin_api_••••••abcd` would silently break
/// every later live call.
pub async fn update_tracker_config(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(payload): Json<TrackerConfigPayload>,
) -> Response {
    if provider.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "provider is required" })),
        )
            .into_response();
    }
    let provider = provider.trim().to_ascii_lowercase();
    let existing = match load_tracker(&state, &provider).await {
        Ok(row) => row,
        Err(err) => return db_failure("issue_tracker_configs", &err).into_response(),
    };
    let now = Utc::now().to_rfc3339();

    let mut creds = match existing.as_ref() {
        Some(row) => row.creds(),
        None => json!({}),
    };
    let mut cfg = match existing.as_ref() {
        Some(row) => row.cfg(),
        None => json!({}),
    };

    let mut skipped_masked: Vec<String> = Vec::new();
    if let Some(incoming) = payload.credentials.as_ref().and_then(|v| v.as_object()) {
        if !creds.is_object() {
            creds = json!({});
        }
        if let Some(target) = creds.as_object_mut() {
            for (key, value) in incoming {
                match value.as_str() {
                    Some(text) if is_masked(text) => skipped_masked.push(key.clone()),
                    Some(text) if text.trim().is_empty() => {
                        // An explicit empty string is the UI's "clear this field".
                        target.insert(key.clone(), json!(""));
                    }
                    _ => {
                        target.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }
    if let Some(incoming) = payload.config.as_ref().and_then(|v| v.as_object()) {
        if !cfg.is_object() {
            cfg = json!({});
        }
        if let Some(target) = cfg.as_object_mut() {
            for (key, value) in incoming {
                target.insert(key.clone(), value.clone());
            }
        }
    }

    let account_name = payload
        .account_name
        .clone()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| existing.as_ref().and_then(|r| r.account_name.clone()));
    let mut defaulted_fields: Vec<&str> = Vec::new();
    let is_enabled = match payload
        .is_enabled
        .or_else(|| existing.as_ref().and_then(|r| r.is_enabled))
    {
        Some(value) => value,
        // The column is NOT NULL, so an unread column keeps the storage default
        // and the response names it as a default rather than an operator choice.
        None => {
            defaulted_fields.push("is_enabled");
            true
        }
    };
    let test_mode = match payload
        .test_mode
        .or_else(|| existing.as_ref().and_then(|r| r.test_mode))
    {
        Some(value) => value,
        None => {
            defaulted_fields.push("test_mode");
            true
        }
    };
    let created = match existing.as_ref().and_then(|r| r.created_at.clone()) {
        Some(value) => value,
        None => {
            defaulted_fields.push("created_at");
            now.clone()
        }
    };
    let id = match existing.as_ref().map(|r| r.id.clone()) {
        Some(value) => value,
        None => Uuid::new_v4().simple().to_string(),
    };

    let result = sqlx::query(
        r#"
        INSERT INTO issue_tracker_configs (id, provider, account_name, is_enabled, test_mode, credentials, config, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT(provider) DO UPDATE SET
            account_name = excluded.account_name,
            is_enabled = excluded.is_enabled,
            test_mode = excluded.test_mode,
            credentials = excluded.credentials,
            config = excluded.config,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&id)
    .bind(&provider)
    .bind(&account_name)
    .bind(is_enabled)
    .bind(test_mode)
    .bind(creds.to_string())
    .bind(cfg.to_string())
    .bind(&created)
    .bind(&now)
    .execute(&state.db)
    .await
    .map(|result| result.rows_affected());

    let rows_affected = match result {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "could not persist the tracker configuration", "detail": err.to_string() })),
            )
                .into_response();
        }
    };

    let mut body = match load_tracker(&state, &provider).await {
        Ok(Some(row)) => row.json(),
        Ok(None) => json!({}),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "saved": true,
                    "rereadFailed": true,
                    "error": format!("the configuration was written but could not be read back: {err}"),
                })),
            )
                .into_response();
        }
    };
    let read_back_ok = body.get("id").is_some();
    if let Some(obj) = body.as_object_mut() {
        obj.insert(
            "saved".into(),
            json!({
                "rowsAffected": rows_affected,
                "skippedMaskedCredentialFields": skipped_masked.clone(),
                "defaultsApplied": defaulted_fields,
                "detail": if skipped_masked.is_empty() {
                    "all supplied credential fields were stored".to_string()
                } else {
                    format!(
                        "{} field(s) held a masked value from the previous page load and were NOT overwritten: {}",
                        skipped_masked.len(),
                        skipped_masked.join(", ")
                    )
                },
                "testMode": test_mode,
            }),
        );
        if !read_back_ok {
            obj.insert(
                "note".into(),
                json!("the write reported success but the row could not be read back; the fields above are the values sent, not a read of stored state"),
            );
        }
    }
    Json(body).into_response()
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Live tracker authentication
// ---------------------------------------------------------------------------

/// A refusal that also reports whether the "unconfigured" stamp could be saved.
async fn refuse_unconfigured(state: &AppState, provider: &str, endpoint: &str, reason: &str) -> Response {
    let persistence = write_tracker_test_result(state, provider, "unconfigured").await;
    let (code, mut body) = refuse(endpoint, reason);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("lastTestStatusStored".into(), persistence);
    }
    (code, Json(body)).into_response()
}

/// POST /integrations/trackers/test/{provider}
///
/// Every branch below is an actual request to the provider. Nothing sleeps and
/// nothing claims "authenticated" without a 2xx and an identity in the body.
pub async fn test_tracker_connection(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Response {
    let provider = provider.trim().to_ascii_lowercase();
    if !TRACKER_KEYS.contains(&provider.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "unsupported",
                "connected": false,
                "liveCallMade": false,
                "error": format!("no live authentication path is implemented for {provider}"),
                "supported": TRACKER_KEYS,
            })),
        )
            .into_response();
    }

    let row = match load_tracker(&state, &provider).await {
        Ok(row) => row,
        Err(err) => return db_failure("issue_tracker_configs", &err).into_response(),
    };
    let Some(row) = row else {
        let (code, mut body) = refuse(
            &format!("{provider} (not contacted)"),
            &format!(
                "no configuration row for {provider}: POST /integrations/trackers/configs/{provider} with credentials, then test again"
            ),
        );
        if let Some(obj) = body.as_object_mut() {
            obj.insert("storedTestResult".into(), json!(null));
            obj.insert(
                "note".into(),
                json!("last_test_status was not written: there is no configuration row to stamp"),
            );
        }
        return (code, Json(body)).into_response();
    };

    if row.is_sandbox() {
        let persistence = write_tracker_test_result(&state, &provider, "unverified").await;
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "unverified",
                "connected": false,
                "mode": "sandbox",
                "liveCallMade": false,
                "provider": provider,
                "testMode": row.test_mode,
                "detail": format!(
                    "{provider} is stored as sandbox (test_mode = {}), so no request was sent to the provider. \
                     A sandbox cannot confirm credentials; set test_mode = false to run a live check.",
                    match row.test_mode {
                        Some(true) => "true".to_string(),
                        Some(false) => "false".to_string(),
                        None => "unset, treated as sandbox".to_string(),
                    }
                ),
                "lastTestStatusStored": persistence,
                "checkedAt": Utc::now().to_rfc3339(),
            })),
        )
            .into_response();
    }

    let creds = row.creds();
    let cfg = row.cfg();
    let client = &state.http_client;

    let call: LiveCall = match provider.as_str() {
        "linear" => match credential(&creds, &cfg, &["api_key", "apiKey", "personal_access_token"]) {
            None => {
                return refuse_unconfigured(
                    &state,
                    &provider,
                    "https://api.linear.app/graphql",
                    "no usable linear api_key (expected lin_api_…). Masked values saved from a previous page load do not count.",
                )
                .await
            }
            Some((_, key)) => {
                let endpoint = "https://api.linear.app/graphql".to_string();
                    execute(
                    client,
                    client
                        .post(&endpoint)
                        .header("Authorization", key)
                        .header("Content-Type", "application/json")
                        .json(&json!({ "query": "{ viewer { id name email } }" })),
                    endpoint,
                )
                .await
            }
        },
        "github" => match credential(&creds, &cfg, &["access_token", "accessToken", "token", "personal_access_token"]) {
            None => {
                return refuse_unconfigured(
                    &state,
                    &provider,
                    "https://api.github.com/user",
                    "no usable github access_token (expected ghp_… or github_pat_…).",
                )
                .await
            }
            Some((_, token)) => {
                let endpoint = "https://api.github.com/user".to_string();
                    execute(
                    client,
                    client
                        .get(&endpoint)
                        .header("Authorization", format!("Bearer {token}"))
                        .header("User-Agent", "Communicator-Rust-Backend")
                        .header("Accept", "application/vnd.github+json"),
                    endpoint,
                )
                .await
            }
        },
        "jira" => {
            let domain = credential(&creds, &cfg, &["domain", "site", "site_url", "baseUrl"])
                .map(|(_, v)| v.replace("https://", "").replace("http://", "").trim_end_matches('/').to_string());
            let email = credential(&creds, &cfg, &["email", "user", "account_email"]).map(|(_, v)| v);
            let token = credential(&creds, &cfg, &["api_token", "apiToken", "personal_access_token"]).map(|(_, v)| v);
            let (Some(domain), Some(token)) = (domain.clone(), token) else {
                let endpoint_text = match &domain {
                    Some(domain) => format!("https://{domain}/rest/api/3/myself (not contacted)"),
                    None => "https://<domain unset>/rest/api/3/myself (not contacted)".to_string(),
                };
                return refuse_unconfigured(
                    &state,
                    &provider,
                    &endpoint_text,
                    "jira needs credentials.domain (e.g. acme.atlassian.net) and credentials.api_token; email is optional for basic auth.",
                )
                .await;
            };
            let endpoint = format!("https://{domain}/rest/api/3/myself");
            let mut request = client.get(&endpoint).header("Accept", "application/json");
            if let Some(email) = email {
                request = request.basic_auth(email, Some(token));
            } else {
                // Atlassian also accepts a bearer API token for some deployments.
                request = request.header("Authorization", format!("Bearer {token}"));
            }
            execute(client, request, endpoint).await
        }
        "slack" => match credential(&creds, &cfg, &["bot_token", "botToken", "access_token", "token"]) {
            None => {
                return refuse_unconfigured(
                    &state,
                    &provider,
                    "https://slack.com/api/auth.test",
                    "no usable slack bot_token (expected xoxb-…).",
                )
                .await
            }
            Some((_, token)) => {
                let endpoint = "https://slack.com/api/auth.test".to_string();
                    execute(
                    client,
                    client
                        .post(&endpoint)
                        .header("Authorization", format!("Bearer {token}"))
                        .header("Content-Type", "application/json;charset=utf-8"),
                    endpoint,
                )
                .await
            }
        },
        _ => unreachable!("provider validated above"),
    };

    // The HTTP status alone is not proof: Slack answers 200 with `ok:false` and
    // Linear answers 200 with an `errors` array. Require a real identity.
    let identity = extract_identity(&provider, &call.body);
    let confirmed = call.ok && identity.is_some();

    let status = if confirmed {
        "connected"
    } else if call.transport_error.is_some() {
        "unreachable"
    } else if call.ok {
        "unconfirmed"
    } else {
        "error"
    };
    let persistence = write_tracker_test_result(&state, &provider, status).await;

    let message = match (&identity, confirmed, call.status) {
        (Some(id), true, _) => format!("{} authenticated as {id}", capitalize(&provider)),
        (None, false, Some(code)) => format!(
            "{} answered HTTP {code} but the body carried no {} identity field, so the connection is not confirmed",
            capitalize(&provider),
            provider
        ),
        _ => call.message(),
    };

    let body = call.json(json!({
        "provider": provider,
        "mode": "live",
        "status": status,
        "connected": confirmed,
        "identity": identity,
        "message": message,
        "accountName": row.account_name,
        "lastTestStatusStored": persistence,
        "checkedAt": Utc::now().to_rfc3339(),
    }));
    let code = if confirmed {
        StatusCode::OK
    } else if call.transport_error.is_some() {
        StatusCode::BAD_GATEWAY
    } else if call.ok {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::UNAUTHORIZED
    };
    (code, Json(body)).into_response()
}

fn extract_identity(provider: &str, body: &Value) -> Option<String> {
    match provider {
        "linear" => body
            .pointer("/data/viewer/name")
            .and_then(|v| v.as_str())
            .map(|name| {
                body.pointer("/data/viewer/email")
                    .and_then(|e| e.as_str())
                    .map(|email| format!("{name} <{email}>"))
                    .unwrap_or_else(|| name.to_string())
            }),
        "github" => body.get("login").and_then(|v| v.as_str()).map(String::from),
        "jira" => body
            .get("displayName")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| body.get("emailAddress").and_then(|v| v.as_str()).map(String::from)),
        "slack" => {
            if body.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                let user = body.get("user").and_then(|v| v.as_str())?;
                let team = body.get("team").and_then(|v| v.as_str());
                Some(match team {
                    Some(team) => format!("{user}@{team}"),
                    None => user.to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Linked issues: real creation and real status refresh
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
pub struct CreateIssuePayload {
    pub provider: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default, alias = "issueType")]
    pub issue_type: Option<String>,
    #[serde(default, alias = "conversationId")]
    pub conversation_id: Option<String>,
    #[serde(default, alias = "ticketId")]
    pub ticket_id: Option<String>,
    #[serde(default, alias = "actorEmail")]
    pub actor_email: Option<String>,
    #[serde(default, alias = "assignedTo")]
    pub assigned_to: Option<String>,
}

struct CreatedIssue {
    external_id: Option<String>,
    key: Option<String>,
    url: Option<String>,
    status: Option<String>,
}

/// Maps a human priority label to Linear's integer priority code.
///
/// Linear's GraphQL `IssueCreateInput.priority` is an Int, not a free-text
/// label: 0 = none, 1 = urgent, 2 = high, 3 = medium, 4 = low. The rest of this
/// codebase speaks in labels (`urgent | high | medium | low`),
/// so this is the single translation point. An empty or unknown label returns
/// `None` so the field is omitted from the mutation rather than sent as a value
/// the API would reject -- it is never silently coerced to a made-up priority.
fn linear_priority(label: Option<&str>) -> Option<i64> {
    let normalized = label?.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "none" => Some(0),
        "urgent" => Some(1),
        "high" => Some(2),
        "medium" => Some(3),
        "low" => Some(4),
        _ => None,
    }
}

/// POST /integrations/trackers/create-issue
///
/// The issue key and URL come back from the provider. If the provider is not
/// reachable or not configured the response is a refusal and no `linked_issues`
/// row is written, because a link to an issue that does not exist is worse than
/// no link.
pub async fn create_linked_issue(
    State(state): State<AppState>,
    Json(payload): Json<CreateIssuePayload>,
) -> Response {
    let provider = payload.provider.trim().to_ascii_lowercase();
    if !["linear", "github", "jira"].contains(&provider.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("unsupported tracker provider {provider}"),
                "supported": ["linear", "github", "jira"],
            })),
        )
            .into_response();
    }
    if payload.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "title is required" })),
        )
            .into_response();
    }

    let row = match load_tracker(&state, &provider).await {
        Ok(row) => row,
        Err(err) => return db_failure("issue_tracker_configs", &err).into_response(),
    };
    let Some(row) = row else {
        let (code, body) = refuse(
            &provider,
            &format!("no {provider} configuration stored; save credentials via PUT /integrations/trackers/configs/{provider}"),
        );
        return (code, Json(body)).into_response();
    };
    if row.is_sandbox() || row.is_disabled() {
        let (code, mut body) = refuse(
            &provider,
            &format!(
                "{} is stored with test_mode/enabled that forbids live writes; an issue would be recorded \
                 in this database without existing in {}. Turn test_mode off to create it for real.",
                capitalize(&provider),
                provider
            ),
        );
        if let Some(obj) = body.as_object_mut() {
            obj.insert("notCreated".into(), json!(true));
            obj.insert("testMode".into(), json!(row.test_mode));
            obj.insert("isEnabled".into(), json!(row.is_enabled));
        }
        return (code, Json(body)).into_response();
    }

    let creds = row.creds();
    let cfg = row.cfg();
    let client = &state.http_client;
    let title = payload.title.trim().to_string();
    let description = payload
        .description
        .clone()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let issue_type = payload
        .issue_type
        .clone()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());

    let call: LiveCall = match provider.as_str() {
        "linear" => {
            let Some((_, key)) = credential(&creds, &cfg, &["api_key", "apiKey", "personal_access_token"]) else {
                let (code, body) = refuse("https://api.linear.app/graphql", "no usable linear api_key");
                return (code, Json(body)).into_response();
            };
            let Some((_, team_id)) = credential(&creds, &cfg, &["team_id", "teamId", "default_team_id", "team_key", "teamKey"]) else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "unconfigured",
                        "created": false,
                        "liveCallMade": false,
                        "refusal": "Linear needs config.team_id (the team the issue belongs to); it is not guessed from a counter",
                    })),
                )
                    .into_response();
            };
            let mutation = r#"mutation IssueCreate($input: IssueCreateInput!) {
                issueCreate(input: $input) { success issue { id identifier key url title state { name } } }
            }"#;
            // Linear models priority as an integer (0 none, 1 urgent .. 4 low),
            // not a free-text label, so an unmapped label is left out rather
            // than sent as something the API will reject.
            let mut input = Map::new();
            input.insert("title".into(), json!(title.clone()));
            if let Some(text) = &description {
                input.insert("description".into(), json!(text.clone()));
            }
            input.insert("teamId".into(), json!(team_id));
            if let Some(code) = linear_priority(payload.priority.as_deref()) {
                input.insert("priority".into(), json!(code));
            }
            let endpoint = "https://api.linear.app/graphql".to_string();
            execute(
                client,
                client
                    .post(&endpoint)
                    .header("Authorization", key)
                    .header("Content-Type", "application/json")
                    .json(&json!({
                        "query": mutation,
                        "variables": { "input": input }
                    })),
                endpoint,
            )
            .await
        }
        "github" => {
            let Some((_, token)) = credential(&creds, &cfg, &["access_token", "accessToken", "token", "personal_access_token"]) else {
                let (code, body) = refuse("https://api.github.com", "no usable github access_token");
                return (code, Json(body)).into_response();
            };
            let owner = credential(&creds, &cfg, &["owner", "organization", "repo_owner"]).map(|(_, v)| v);
            let repo = credential(&creds, &cfg, &["repo", "repository", "default_repo"]).map(|(_, v)| v);
            let (Some(owner), Some(repo)) = (owner, repo) else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "unconfigured",
                        "created": false,
                        "liveCallMade": false,
                        "refusal": "GitHub needs config.owner and config.repo; no repository is substituted",
                    })),
                )
                    .into_response();
            };
            let endpoint = format!("https://api.github.com/repos/{owner}/{repo}/issues");
            let mut github_input = Map::new();
            github_input.insert("title".into(), json!(title.clone()));
            if let Some(text) = &description {
                github_input.insert("body".into(), json!(text.clone()));
            }
            if let Some(label) = &issue_type {
                github_input.insert("labels".into(), json!([label]));
            }
                execute(
                client,
                client
                    .post(&endpoint)
                    .header("Authorization", format!("Bearer {token}"))
                    .header("User-Agent", "Communicator-Rust-Backend")
                    .header("Accept", "application/vnd.github+json")
                    .json(&github_input),
                endpoint,
            )
            .await
        }
        _ => {
            let domain = credential(&creds, &cfg, &["domain", "site", "baseUrl"])
                .map(|(_, v)| v.replace("https://", "").trim_end_matches('/').to_string());
            let token = credential(&creds, &cfg, &["api_token", "apiToken"]).map(|(_, v)| v);
            let email = credential(&creds, &cfg, &["email", "user"]).map(|(_, v)| v);
            let project_key = credential(&creds, &cfg, &["project_key", "projectKey", "default_project"]).map(|(_, v)| v);
            let (Some(domain), Some(token)) = (domain, token) else {
                let (code, body) = refuse("https://<domain>/rest/api/3/issue", "jira needs credentials.domain and credentials.api_token");
                return (code, Json(body)).into_response();
            };
            let Some(project_key) = project_key else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "unconfigured",
                        "created": false,
                        "liveCallMade": false,
                        "refusal": "Jira needs config.project_key (the project the issue is created in)",
                    })),
                )
                    .into_response();
            };
            // Jira rejects an issue with no issuetype, so the choice is required
            // rather than invented here.
            let Some(issue_type) = issue_type.clone() else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "unconfigured",
                        "created": false,
                        "liveCallMade": false,
                        "missing": ["issueType"],
                        "refusal": "Jira requires an issue type; it is not defaulted to Bug or Task",
                    })),
                )
                    .into_response();
            };
            let endpoint = format!("https://{domain}/rest/api/3/issue");
            let mut request = client
                .post(&endpoint)
                .header("Accept", "application/json")
                .header("Content-Type", "application/json");
            request = match email {
                Some(email) => request.basic_auth(email, Some(token)),
                None => request.header("Authorization", format!("Bearer {token}")),
            };
            let mut fields = Map::new();
            fields.insert("project".into(), json!({ "key": project_key }));
            fields.insert("summary".into(), json!(title.clone()));
            // Jira Cloud REST v3 wants Atlassian Document Format, not a bare
            // string, so the paragraph is built here.
            fields.insert(
                "description".into(),
                match description.as_deref().filter(|text| !text.trim().is_empty()) {
                    Some(text) => json!({
                        "type": "doc",
                        "version": 1,
                        "content": [{
                            "type": "paragraph",
                            "content": [{ "type": "text", "text": text }]
                        }]
                    }),
                    None => json!({ "type": "doc", "version": 1, "content": [{ "type": "paragraph" }] }),
                },
            );
            fields.insert("issuetype".into(), json!({ "name": capitalize(&issue_type) }));
            // Only sent when the operator chose one: Jira rejects a null
            // priority outright.
            if let Some(priority) = payload.priority.as_deref().filter(|p| !p.trim().is_empty()) {
                fields.insert("priority".into(), json!({ "name": capitalize(priority) }));
            }

            execute(
                client,
                request.json(&json!({ "fields": fields })),
                endpoint,
            )
            .await
        }
    };

    let created: Option<CreatedIssue> = if provider == "linear" {
        let issue = call.body.pointer("/data/issueCreate/issue");
        let success = call
            .body
            .pointer("/data/issueCreate/success")
            .and_then(|v| v.as_bool())
            == Some(true);
        issue.map(|issue| CreatedIssue {
            external_id: issue.get("id").and_then(|v| v.as_str()).map(String::from),
            key: issue
                .get("identifier")
                .or_else(|| issue.get("key"))
                .and_then(|v| v.as_str())
                .map(String::from),
            url: issue.get("url").and_then(|v| v.as_str()).map(String::from),
            status: issue
                .pointer("/state/name")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
        .filter(|_| success || call.ok)
    } else if provider == "github" {
        call.body.get("number").and_then(|v| v.as_i64()).map(|number| {
            CreatedIssue {
                external_id: Some(number.to_string()),
                key: Some(format!("#{number}")),
                url: call
                    .body
                    .get("html_url")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                status: call
                    .body
                    .get("state")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            }
        })
    } else {
        call.body.get("key").and_then(|v| v.as_str()).map(|key| {
            let domain = credential(&creds, &cfg, &["domain", "site", "baseUrl"])
                .map(|(_, v)| {
                    v.replace("https://", "")
                        .replace("http://", "")
                        .trim_end_matches('/')
                        .to_string()
                });
            CreatedIssue {
                external_id: call.body.get("id").and_then(|v| v.as_str()).map(String::from),
                key: Some(key.to_string()),
                url: domain.map(|domain| format!("https://{domain}/browse/{key}")),
                // Jira's create response does not carry a workflow status.
                status: call
                    .body
                    .pointer("/fields/status/name")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            }
        })
    };

    let Some(created) = created.filter(|_| call.ok) else {
        let stamp = if call.transport_error.is_some() || matches!(call.status, Some(401) | Some(403)) {
            write_tracker_test_result(&state, &provider, "error").await
        } else {
            json!({
                "ok": false,
                "skipped": true,
                "reason": "the provider rejected the create request; that is not a connection result, so last_test_status was left unchanged",
            })
        };
        let status = if call.transport_error.is_some() {
            StatusCode::BAD_GATEWAY
        } else {
            StatusCode::UNAUTHORIZED
        };
        return (
            status,
            Json(json!({
                "created": false,
                "liveCallMade": call.status.is_some(),
                "callAttempted": true,
                "httpStatus": call.status,
                "latencyMs": call.latency_ms,
                "error": call.message(),
                "detail": format!("the {} issue was NOT created, so no link was stored", provider),
                "providerResponse": truncate(&call.raw, 600),
                "lastTestStatusStored": stamp,
            })),
        )
            .into_response();
    };

    let mut missing_from_provider: Vec<&str> = Vec::new();
    if created.key.is_none() {
        missing_from_provider.push("issue key");
    }
    if created.url.is_none() {
        missing_from_provider.push("issue url");
    }
    let (Some(issue_key), Some(issue_url)) = (created.key.clone(), created.url.clone()) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "created": true,
                "stored": false,
                "liveCallMade": true,
                "httpStatus": call.status,
                "missing": missing_from_provider,
                "providerResponse": truncate(&call.raw, 600),
                "error": "the provider accepted the create request but its body carried no issue key/url, so storing a link would record an address we do not have",
            })),
        )
            .into_response();
    };

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().simple().to_string();
    let status_from_provider = created.status.clone();
    // The column is NOT NULL: an unknown status is stored empty and the row is
    // marked pending so nothing claims a workflow state the provider never sent.
    let stored_status = match &status_from_provider {
        Some(status) => status.clone(),
        None => String::new(),
    };
    let stored_sync_status = if status_from_provider.is_some() { "synced" } else { "pending" };
    let metadata = json!({
        "description": description,
        "issueType": issue_type,
        "priority": payload.priority,
        "createdVia": "integrations_parity",
        "httpStatus": call.status,
        "latencyMs": call.latency_ms,
        "statusSource": if status_from_provider.is_some() { "providerCreateResponse" } else { "notReturnedByProvider" },
    })
    .to_string();

    let inserted = sqlx::query(
        r#"
        INSERT INTO linked_issues (
            id, conversation_id, ticket_id, provider, external_issue_id, issue_key, issue_url,
            title, status, priority, assigned_to, created_by_email, sync_status, metadata_json,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(&id)
    .bind(payload.conversation_id.as_deref().filter(|v| !v.trim().is_empty()))
    .bind(payload.ticket_id.as_deref().filter(|v| !v.trim().is_empty()))
    .bind(&provider)
    .bind(created.external_id.clone())
    .bind(&issue_key)
    .bind(&issue_url)
    .bind(payload.title.trim())
    .bind(&stored_status)
    .bind(payload.priority.as_deref().filter(|v| !v.trim().is_empty()))
    .bind(payload.assigned_to.as_deref().filter(|v| !v.trim().is_empty()))
    .bind(payload.actor_email.as_deref().filter(|v| !v.trim().is_empty()))
    .bind(stored_sync_status)
    .bind(&metadata)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map(|result| result.rows_affected());

    let stored_rows = match inserted {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "created": true,
                    "stored": false,
                    "providerIssue": issue_url,
                    "error": format!("the provider created the issue but the link could not be stored: {err}"),
                })),
            )
                .into_response();
        }
    };

    let activity = match payload.ticket_id.as_deref().filter(|v| !v.trim().is_empty()) {
        Some(ticket_id) => persistence_json(
            sqlx::query(
                "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES ($1, $2, NULL, $3, 'external_issue', $4, $5, $6)",
            )
            .bind(Uuid::new_v4().simple().to_string())
            .bind(ticket_id)
            .bind(row.account_name.clone())
            .bind(format!("Created external issue [{issue_key}] {} at {issue_url}", payload.title))
            .bind(metadata.clone())
            .bind(&now)
            .execute(&state.db)
            .await
            .map(|result| result.rows_affected()),
            "ticket_activities",
        ),
        None => json!({ "ok": false, "skipped": true, "reason": "no ticketId was supplied, so no ticket activity was written" }),
    };

    let status_note = match &status_from_provider {
        Some(_) => Value::Null,
        None => json!(format!(
            "{provider} did not return a workflow status for the new issue, so it is stored as unknown; \
             POST /integrations/trackers/linked/{id}/sync reads the real one"
        )),
    };

    (
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "provider": provider,
            "externalIssueId": created.external_id,
            "issueKey": issue_key,
            "issueUrl": issue_url,
            "title": payload.title,
            "status": status_from_provider,
            "syncStatus": stored_sync_status,
            "statusNote": status_note,
            "createdAt": now,
            "created": true,
            "stored": stored_rows > 0,
            "rowsAffected": stored_rows,
            "ticketActivity": activity,
            "liveCallMade": call.status.is_some(),
            "httpStatus": call.status,
            "latencyMs": call.latency_ms,
        })),
    )
        .into_response()
}

/// POST /integrations/trackers/linked/{id}/sync -- status is read back from the
/// provider; a failed read leaves the stored status untouched and says so.
pub async fn sync_issue_status(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let row = match sqlx::query_as::<_, (String, Option<String>, String, String, Option<String>)>(
        "SELECT provider, external_issue_id, issue_key, issue_url, ticket_id FROM linked_issues WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_failure("linked_issues", &err).into_response(),
    };

    let Some((provider, external_id, issue_key, issue_url, ticket_id)) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "linked issue not found", "id": id })),
        )
            .into_response();
    };

    let provider = provider.to_ascii_lowercase();
    let config_row = match load_tracker(&state, &provider).await {
        Ok(row) => row,
        Err(err) => return db_failure("issue_tracker_configs", &err).into_response(),
    };
    let Some(config_row) = config_row else {
        let (code, body) = refuse(&provider, &format!("no {provider} configuration to authenticate against"));
        return (code, Json(body)).into_response();
    };
    let creds = config_row.creds();
    let cfg = config_row.cfg();
    let client = &state.http_client;

    let call: LiveCall = match provider.as_str() {
        "linear" => {
            let Some((_, key)) = credential(&creds, &cfg, &["api_key", "apiKey", "personal_access_token"]) else {
                let (code, body) = refuse("https://api.linear.app/graphql", "no usable linear api_key");
                return (code, Json(body)).into_response();
            };
            let lookup_id = match &external_id {
                Some(id) if usable(id) => id.clone(),
                _ => issue_key.clone(),
            };
            let endpoint = "https://api.linear.app/graphql".to_string();
                execute(
                client,
                client
                    .post(&endpoint)
                    .header("Authorization", key)
                    .header("Content-Type", "application/json")
                    .json(&json!({
                        "query": "query($id: ID!) { issue(id: $id) { id identifier state { name } assignee { name } } }",
                        "variables": { "id": lookup_id }
                    })),
                endpoint,
            )
            .await
        }
        "github" => {
            let Some((_, token)) = credential(&creds, &cfg, &["access_token", "accessToken", "token"]) else {
                let (code, body) = refuse("https://api.github.com", "no usable github access_token");
                return (code, Json(body)).into_response();
            };
            // The stored URL is the only reliable source of owner/repo here.
            let path = match issue_url.strip_prefix("https://github.com/") {
                Some(path) => path.trim_end_matches('/').to_string(),
                None => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({
                            "error": "the stored issue_url is not a github issue URL, so the repository cannot be resolved",
                            "issueUrl": issue_url,
                            "liveCallMade": false,
                        })),
                    )
                        .into_response();
                }
            };
            let mut parts = path.split('/');
            let (Some(owner), Some(repo)) = (parts.next(), parts.next()) else {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "error": "the stored issue_url is not a github issue URL, so the repository cannot be resolved",
                        "issueUrl": issue_url,
                        "liveCallMade": false,
                    })),
                )
                    .into_response();
            };
            let number = match external_id
                .as_deref()
                .and_then(|id| id.rsplit(|c| c == '#' || c == '/').next())
            {
                Some(candidate) if !candidate.is_empty() && candidate.chars().all(|c| c.is_ascii_digit()) => {
                    candidate.to_string()
                }
                _ => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({
                            "error": "the stored external_issue_id holds no github issue number, so there is nothing to fetch",
                            "externalIssueId": external_id.clone(),
                            "liveCallMade": false,
                        })),
                    )
                        .into_response();
                }
            };
            let endpoint = format!("https://api.github.com/repos/{owner}/{repo}/issues/{number}");
                execute(
                client,
                client
                    .get(&endpoint)
                    .header("Authorization", format!("Bearer {token}"))
                    .header("User-Agent", "Communicator-Rust-Backend")
                    .header("Accept", "application/vnd.github+json"),
                endpoint,
            )
            .await
        }
        "jira" => {
            let domain = credential(&creds, &cfg, &["domain", "site", "baseUrl"])
                .map(|(_, v)| v.replace("https://", "").trim_end_matches('/').to_string());
            let token = credential(&creds, &cfg, &["api_token", "apiToken"]).map(|(_, v)| v);
            let email = credential(&creds, &cfg, &["email", "user"]).map(|(_, v)| v);
            let (Some(domain), Some(token)) = (domain, token) else {
                let (code, body) = refuse("https://<domain>/rest/api/3/issue/<key>", "jira needs credentials.domain and credentials.api_token");
                return (code, Json(body)).into_response();
            };
            let endpoint = format!("https://{domain}/rest/api/3/issue/{issue_key}?fields=status,assignedTo");
            let mut request = client.get(&endpoint).header("Accept", "application/json");
            request = match email {
                Some(email) => request.basic_auth(email, Some(token)),
                None => request.header("Authorization", format!("Bearer {token}")),
            };
            execute(client, request, endpoint).await
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("no live status read is implemented for provider {provider}"),
                    "supported": ["linear", "github", "jira"],
                })),
            )
                .into_response();
        }
    };

    let fetched_status: Option<String> = if !call.ok {
        None
    } else {
        match provider.as_str() {
            "linear" => call
                .body
                .pointer("/data/issue/state/name")
                .and_then(|v| v.as_str())
                .map(String::from),
            "github" => call.body.get("state").and_then(|v| v.as_str()).map(|s| match s {
                "open" => "Open".to_string(),
                "closed" => "Closed".to_string(),
                other => capitalize(other),
            }),
            _ => call
                .body
                .pointer("/fields/status/name")
                .and_then(|v| v.as_str())
                .map(String::from),
        }
    };

    let now = Utc::now().to_rfc3339();
    match fetched_status {
        Some(status) => {
            let stored = persistence_json(
                sqlx::query("UPDATE linked_issues SET status = $1, sync_status = 'synced', updated_at = $2 WHERE id = $3")
                    .bind(&status)
                    .bind(&now)
                    .bind(&id)
                    .execute(&state.db)
                    .await
                    .map(|result| result.rows_affected()),
                "linked_issues.status and sync_status",
            );
            let synced = stored.get("ok").and_then(|v| v.as_bool()) == Some(true);
            let activity = match ticket_id.as_deref().filter(|v| !v.trim().is_empty()) {
                Some(ticket_id) => persistence_json(
                    sqlx::query(
                        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES ($1, $2, NULL, $3, 'external_issue', $4, '{}', $5)",
                    )
                    .bind(Uuid::new_v4().simple().to_string())
                    .bind(ticket_id)
                    .bind(config_row.account_name.clone())
                    .bind(format!("[{issue_key}] status read from provider: {status}"))
                    .bind(&now)
                    .execute(&state.db)
                    .await
                    .map(|result| result.rows_affected()),
                    "ticket_activities",
                ),
                None => json!({ "ok": false, "skipped": true, "reason": "this linked issue is not attached to a ticket" }),
            };
            let body = json!({
                "id": id,
                "provider": provider,
                "issueKey": issue_key,
                "issueUrl": issue_url,
                "status": status,
                "syncStatus": if synced { json!("synced") } else { Value::Null },
                "statusStored": synced,
                "storage": stored,
                "ticketActivity": activity,
                "updatedAt": now,
                "liveCallMade": call.status.is_some(),
                "httpStatus": call.status,
                "latencyMs": call.latency_ms,
                "note": if synced {
                    Value::Null
                } else {
                    json!("the provider returned this status but it was not written to linked_issues; see storage.reason")
                },
            });
            let code = if synced { StatusCode::OK } else { StatusCode::INTERNAL_SERVER_ERROR };
            (code, Json(body)).into_response()
        }
        None => {
            let stored = persistence_json(
                sqlx::query("UPDATE linked_issues SET sync_status = 'error', updated_at = $1 WHERE id = $2")
                    .bind(&now)
                    .bind(&id)
                    .execute(&state.db)
                    .await
                    .map(|result| result.rows_affected()),
                "linked_issues.sync_status",
            );
            let marked = stored.get("ok").and_then(|v| v.as_bool()) == Some(true);
            let code = if call.transport_error.is_some() {
                StatusCode::BAD_GATEWAY
            } else if call.status == Some(404) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::UNAUTHORIZED
            };
            (
                code,
                Json(json!({
                    "id": id,
                    "provider": provider,
                    "issueKey": issue_key,
                    "syncStatus": if marked { json!("error") } else { Value::Null },
                    "statusUnchanged": true,
                    "liveCallMade": call.status.is_some(),
                    "callAttempted": true,
                    "httpStatus": call.status,
                    "error": call.message(),
                    "storage": stored,
                    "detail": "the stored status was not modified because the provider did not return one",
                })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// /settings -- the Connections page, backed by integration_settings
// ---------------------------------------------------------------------------

const SELECT_SETTINGS: &str = "SELECT id, tenant_key, whatsapp_provider, meta_access_token, meta_phone_number_id, meta_waba_id, meta_app_secret, meta_webhook_verify_token, whatsapp_verified_at, whatsapp_verified_name, whatsapp_display_number, business_name, business_description, agent_guidelines, auto_reply_enabled, business_hours_enabled, business_hours_start, business_hours_end, business_timezone, away_message, updated_at FROM integration_settings ORDER BY updated_at DESC LIMIT 1";

#[derive(Debug, Clone, sqlx::FromRow)]
struct SettingsRow {
    id: String,
    tenant_key: Option<String>,
    whatsapp_provider: Option<String>,
    meta_access_token: Option<String>,
    meta_phone_number_id: Option<String>,
    meta_waba_id: Option<String>,
    meta_app_secret: Option<String>,
    meta_webhook_verify_token: Option<String>,
    whatsapp_verified_at: Option<String>,
    whatsapp_verified_name: Option<String>,
    whatsapp_display_number: Option<String>,
    business_name: Option<String>,
    business_description: Option<String>,
    agent_guidelines: Option<String>,
    auto_reply_enabled: Option<bool>,
    business_hours_enabled: Option<bool>,
    business_hours_start: Option<i64>,
    business_hours_end: Option<i64>,
    business_timezone: Option<String>,
    away_message: Option<String>,
    updated_at: Option<String>,
}

impl SettingsRow {
    fn field(&self, name: &str) -> Option<&str> {
        match name {
            "meta_access_token" => self.meta_access_token.as_deref(),
            "meta_phone_number_id" => self.meta_phone_number_id.as_deref(),
            "meta_waba_id" => self.meta_waba_id.as_deref(),
            "meta_app_secret" => self.meta_app_secret.as_deref(),
            "meta_webhook_verify_token" => self.meta_webhook_verify_token.as_deref(),
            _ => None,
        }
    }
}

async fn load_settings(state: &AppState) -> Result<Option<SettingsRow>, sqlx::Error> {
    sqlx::query_as::<_, SettingsRow>(SELECT_SETTINGS)
        .fetch_optional(&state.db)
        .await
}

// The Graph API version we address is only known when it was actually set.
fn meta_api_version(state: &AppState) -> Option<String> {
    let from_config = state.config.meta_whatsapp_api_version.trim().to_string();
    if usable(&from_config) {
        return Some(from_config);
    }
    env_value("META_WHATSAPP_API_VERSION")
}

/// The Connections page's row. Reading never writes: no row is reported as
/// absent, and the empty INSERT lives only in the endpoints that save input.
async fn create_settings_row(state: &AppState) -> Result<SettingsRow, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().simple().to_string();
    let result = sqlx::query(
        "INSERT INTO integration_settings (id, tenant_key, whatsapp_provider, updated_at) VALUES ($1, $2, 'meta', $3)",
    )
    .bind(&id)
    .bind(env_value("DEFAULT_TENANT_KEY"))
    .bind(&now)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    match load_settings(state).await? {
        Some(row) => Ok(row),
        None => Err(sqlx::Error::RowNotFound),
    }
}

/// GET /settings -- masked, like the Python route. Nothing is echoed in full.
pub async fn get_integration_settings(State(state): State<AppState>) -> Response {
    let row = match load_settings(&state).await {
        Ok(row) => row,
        Err(err) => return db_failure("integration_settings", &err).into_response(),
    };
    let settings = row.as_ref();
    let exists = settings.is_some();

    fn present(value: Option<&str>) -> Option<&str> {
        match value.map(str::trim) {
            Some(text) if !text.is_empty() => Some(text),
            _ => None,
        }
    }

    let mask = |name: &str| -> Option<String> {
        let value = present(settings.and_then(|row| row.field(name)));
        match value {
            None => None,
            Some(value) if is_masked(value) => Some("…set(masked)".to_string()),
            Some(value) => {
                let chars: Vec<char> = value.chars().collect();
                Some(if chars.len() > 4 {
                    format!("...{}", chars[chars.len() - 4..].iter().collect::<String>())
                } else {
                    "...set".to_string()
                })
            }
        }
    };

    let configured = |name: &str| -> Option<bool> {
        settings.map(|row| match row.field(name) {
            Some(value) => usable(value),
            None => false,
        })
    };

    Json(json!({
        "whatsapp": {
            "provider": present(settings.and_then(|row| row.whatsapp_provider.as_deref())),
            "apiVersion": meta_api_version(&state),
            "phoneNumberId": present(settings.and_then(|row| row.meta_phone_number_id.as_deref())),
            "wabaId": present(settings.and_then(|row| row.meta_waba_id.as_deref())),
            "accessTokenMasked": mask("meta_access_token"),
            "appSecretMasked": mask("meta_app_secret"),
            "webhookVerifyTokenMasked": mask("meta_webhook_verify_token"),
            "webhookCallbackPath": "/api/v1/webhooks/meta/whatsapp",
            "hasAccessToken": configured("meta_access_token"),
            "verifiedAt": present(settings.and_then(|row| row.whatsapp_verified_at.as_deref())),
            "verifiedName": present(settings.and_then(|row| row.whatsapp_verified_name.as_deref())),
            "displayNumber": present(settings.and_then(|row| row.whatsapp_display_number.as_deref())),
            "verifiedNameNote": if exists {
                "reflects the last successful POST /settings/whatsapp/verify; it is not re-read from Meta on this request"
            } else {
                "no integration_settings row exists, so nothing has ever been verified"
            },
        },
        "business": {
            "name": present(settings.and_then(|row| row.business_name.as_deref())),
            "description": present(settings.and_then(|row| row.business_description.as_deref())),
            "agentGuidelines": present(settings.and_then(|row| row.agent_guidelines.as_deref())),
            "autoReplyEnabled": settings.and_then(|row| row.auto_reply_enabled),
            "businessHoursEnabled": settings.and_then(|row| row.business_hours_enabled),
            "businessHoursStart": settings.and_then(|row| row.business_hours_start),
            "businessHoursEnd": settings.and_then(|row| row.business_hours_end),
            "businessTimezone": present(settings.and_then(|row| row.business_timezone.as_deref())),
            "awayMessage": present(settings.and_then(|row| row.away_message.as_deref())),
        },
        "row": {
            "id": settings.map(|row| row.id.clone()),
            "tenantKey": settings.and_then(|row| row.tenant_key.clone()),
            "exists": exists,
            "updatedAt": settings.and_then(|row| row.updated_at.clone()),
            "note": if exists { "" } else { "no integration_settings row has been saved yet; every field below is null because nothing was configured, not because it was turned off" },
        },
    }))
    .into_response()
}

/// PUT /settings -- whitelisted columns only, so a stray payload key cannot
/// become an SQL column name. Masked values are skipped and reported.
pub async fn update_integration_settings(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let mut patch: Map<String, Value> = Map::new();
    let mut skipped_masked: Vec<String> = Vec::new();

    let mut absorb = |source: Option<&Value>, prefix: &str| {
        let Some(source) = source else { return };
        let Some(obj) = source.as_object() else { return };
        for (key, value) in obj {
            let mapped = match (prefix, key.as_str()) {
                ("whatsapp", "access_token") | ("whatsapp", "accessToken") => "meta_access_token",
                ("whatsapp", "phone_number_id") | ("whatsapp", "phoneNumberId") => "meta_phone_number_id",
                ("whatsapp", "waba_id") | ("whatsapp", "wabaId") => "meta_waba_id",
                ("whatsapp", "app_secret") | ("whatsapp", "appSecret") => "meta_app_secret",
                ("whatsapp", "webhook_verify_token") | ("whatsapp", "webhookVerifyToken") => "meta_webhook_verify_token",
                ("whatsapp", "provider") => "whatsapp_provider",
                ("business", "name") => "business_name",
                ("business", "description") => "business_description",
                ("business", "agent_guidelines") | ("business", "agentGuidelines") => "agent_guidelines",
                ("business", "auto_reply_enabled") | ("business", "autoReplyEnabled") => "auto_reply_enabled",
                ("business", "business_hours_enabled") | ("business", "businessHoursEnabled") => "business_hours_enabled",
                ("business", "business_hours_start") | ("business", "businessHoursStart") => "business_hours_start",
                ("business", "business_hours_end") | ("business", "businessHoursEnd") => "business_hours_end",
                ("business", "business_timezone") | ("business", "businessTimezone") => "business_timezone",
                ("business", "away_message") | ("business", "awayMessage") => "away_message",
                _ => {
                    // Flat snake_case keys are accepted as-is.
                    if prefix.is_empty() {
                        key.as_str()
                    } else {
                        continue;
                    }
                }
            };
            if let Some(text) = value.as_str() {
                if is_masked(text) {
                    skipped_masked.push(mapped.to_string());
                    continue;
                }
            }
            patch.insert(mapped.to_string(), value.clone());
        }
    };

    absorb(payload.get("whatsapp"), "whatsapp");
    absorb(payload.get("business"), "business");
    if payload.is_object() {
        absorb(Some(&payload), "");
    }

    let mut row_created = false;
    let row = match load_settings(&state).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            row_created = true;
            match create_settings_row(&state).await {
                Ok(row) => row,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "saved": false,
                            "error": "no integration_settings row existed and a new one could not be created",
                            "detail": err.to_string(),
                        })),
                    )
                        .into_response();
                }
            }
        }
        Err(err) => return db_failure("integration_settings", &err).into_response(),
    };

    let allowed: Vec<&str> = vec![
        "whatsapp_provider",
        "meta_access_token",
        "meta_phone_number_id",
        "meta_waba_id",
        "meta_app_secret",
        "meta_webhook_verify_token",
        "whatsapp_verified_name",
        "whatsapp_display_number",
        "business_name",
        "business_description",
        "agent_guidelines",
        "auto_reply_enabled",
        "business_hours_enabled",
        "business_hours_start",
        "business_hours_end",
        "business_timezone",
        "away_message",
    ];

    let mut applied: Vec<String> = Vec::new();
    let mut cleared: Vec<String> = Vec::new();
    let mut rejected: Vec<String> = Vec::new();
    for key in allowed.iter().copied() {
        let Some(value) = patch.get(key) else { continue };
        let result = match key {
            "auto_reply_enabled" | "business_hours_enabled" => {
                let flag = match value.as_bool() {
                    Some(flag) => Some(flag),
                    None => match value.as_str().map(|text| text.to_ascii_lowercase()) {
                        Some(text) => match text.as_str() {
                            "1" | "true" | "yes" | "on" => Some(true),
                            "0" | "false" | "no" | "off" => Some(false),
                            _ => None,
                        },
                        None => None,
                    },
                };
                let Some(flag) = flag else {
                    rejected.push(format!("{key} (not a boolean)"));
                    continue;
                };
                sqlx::query(&format!("UPDATE integration_settings SET {key} = $1, updated_at = $2 WHERE id = $3"))
                    .bind(flag)
                    .bind(Utc::now().to_rfc3339())
                    .bind(&row.id)
                    .execute(&state.db)
                    .await
                    .map(|result| result.rows_affected())
            }
            "business_hours_start" | "business_hours_end" => {
                let parsed = value.as_i64().or_else(|| value.as_str().and_then(|s| s.parse().ok()));
                let Some(number) = parsed else {
                    rejected.push(format!("{key} (not a number)"));
                    continue;
                };
                let valid = match key {
                    "business_hours_start" => (0..=23).contains(&number),
                    _ => (0..=24).contains(&number),
                };
                if !valid {
                    rejected.push(format!("{key} = {number} is outside 0-23/0-24"));
                    continue;
                }
                sqlx::query(&format!("UPDATE integration_settings SET {key} = $1, updated_at = $2 WHERE id = $3"))
                    .bind(number)
                    .bind(Utc::now().to_rfc3339())
                    .bind(&row.id)
                    .execute(&state.db)
                    .await
                    .map(|result| result.rows_affected())
            }
            _ => {
                let text = match value {
                    Value::Null => {
                        cleared.push(key.to_string());
                        String::new()
                    }
                    Value::String(text) => text.clone(),
                    _ => {
                        rejected.push(format!("{key} (not text)"));
                        continue;
                    }
                };
                sqlx::query(&format!("UPDATE integration_settings SET {key} = $1, updated_at = $2 WHERE id = $3"))
                    .bind(&text)
                    .bind(Utc::now().to_rfc3339())
                    .bind(&row.id)
                    .execute(&state.db)
                    .await
                    .map(|result| result.rows_affected())
            }
        };
        match result {
            Ok(1) => applied.push(key.to_string()),
            Ok(rows) => rejected.push(format!("{key} (the row was not updated; {rows} rows affected)")),
            Err(err) => rejected.push(format!("{key}: {err}")),
        }
    }

    for key in patch.keys() {
        if !allowed.contains(&key.as_str()) {
            rejected.push(format!("{key} (not a writable column)"));
        }
    }

    Json(json!({
        "status": if applied.is_empty() { "noop" } else { "saved" },
        "applied": applied,
        "cleared": cleared,
        "rejected": rejected,
        "skippedMaskedFields": skipped_masked,
        "rowCreated": row_created,
        "createdRowNote": if row_created {
            json!("this request created the integration_settings row; columns the request did not carry hold the storage defaults declared in db/mod.rs")
        } else {
            Value::Null
        },
        "detail": if skipped_masked.is_empty() {
            String::new()
        } else {
            format!(
                "{} field(s) arrived masked from the page and were left unchanged; retype them to overwrite",
                skipped_masked.len()
            )
        },
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// WhatsApp: real Graph API calls
// ---------------------------------------------------------------------------

struct GraphCredentials {
    token: String,
    phone_number_id: String,
    waba_id: String,
    api_version: String,
    sources: Vec<String>,
}

struct CredentialRefusal {
    missing: Vec<String>,
    database_errors: Vec<String>,
}

fn graph_base(version: &str) -> String {
    format!("https://graph.facebook.com/{version}")
}

// The first active WhatsApp inbox's credential blob, with DB failures kept
// distinct from "no inbox configured".
async fn inbox_credentials(state: &AppState) -> Result<Option<Value>, sqlx::Error> {
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT credentials FROM inboxes WHERE channel_type = 'whatsapp' AND is_active = 1 LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;
    Ok(match raw {
        Some(raw) => {
            let parsed = parse_object(Some(&raw));
            parsed.is_object().then_some(parsed)
        }
        None => None,
    })
}

/// DB row first, then the WhatsApp inbox credentials, then the process
/// environment. Refusals name the exact fields the operator must set.
async fn graph_credentials(
    state: &AppState,
    need_phone: bool,
    need_waba: bool,
) -> Result<GraphCredentials, CredentialRefusal> {
    let mut missing = Vec::new();
    let mut database_errors = Vec::new();

    let row = match load_settings(state).await {
        Ok(row) => row,
        Err(err) => {
            database_errors.push(format!("integration_settings: {err}"));
            None
        }
    };

    let mut token = match row.as_ref().and_then(|r| r.meta_access_token.clone()) {
        Some(token) => token,
        None => String::new(),
    };
    let mut token_source = "integration_settings.meta_access_token".to_string();
    if !usable(&token) {
        // A WhatsApp inbox may carry its own token.
        match inbox_credentials(state).await {
            Ok(Some(parsed)) => {
                if let Some((key, value)) = credential(&parsed, &parsed, &["access_token", "accessToken", "meta_access_token"]) {
                    token = value;
                    token_source = format!("whatsapp inbox credentials.{key}");
                }
            }
            Ok(None) => {}
            Err(err) => database_errors.push(format!("inboxes.credentials: {err}")),
        }
    }
    if !usable(&token) {
        if let Some(from_env) = env_value("META_WHATSAPP_ACCESS_TOKEN").or_else(|| env_value("META_ACCESS_TOKEN")) {
            token = from_env;
            token_source = "META_WHATSAPP_ACCESS_TOKEN".to_string();
        }
    }
    if !usable(&token) {
        missing.push("access token (integration_settings.meta_access_token, the whatsapp inbox credentials, or META_WHATSAPP_ACCESS_TOKEN)".to_string());
    }

    let mut phone_number_id = match row.as_ref().and_then(|r| r.meta_phone_number_id.clone()) {
        Some(value) => value,
        None => String::new(),
    };
    let mut phone_source = "integration_settings.meta_phone_number_id".to_string();
    if !usable(&phone_number_id) {
        if let Some(from_env) = env_value("META_PHONE_NUMBER_ID") {
            phone_number_id = from_env;
            phone_source = "META_PHONE_NUMBER_ID".to_string();
        }
    }
    if need_phone && !usable(&phone_number_id) {
        missing.push("phone number id (integration_settings.meta_phone_number_id or META_PHONE_NUMBER_ID)".to_string());
    }

    let mut waba_id = match row.as_ref().and_then(|r| r.meta_waba_id.clone()) {
        Some(value) => value,
        None => String::new(),
    };
    let mut waba_source = "integration_settings.meta_waba_id".to_string();
    if !usable(&waba_id) {
        match inbox_credentials(state).await {
            Ok(Some(parsed)) => {
                if let Some((key, value)) = credential(&parsed, &parsed, &["waba_id", "wabaId", "businessId"]) {
                    waba_id = value;
                    waba_source = format!("whatsapp inbox credentials.{key}");
                }
            }
            Ok(None) => {}
            Err(err) => database_errors.push(format!("inboxes.credentials: {err}")),
        }
    }
    if !usable(&waba_id) {
        if let Some(from_env) = env_value("META_WABA_ID") {
            waba_id = from_env;
            waba_source = "META_WABA_ID".to_string();
        }
    }
    if need_waba && !usable(&waba_id) {
        missing.push("WABA id (integration_settings.meta_waba_id, the whatsapp inbox credentials wabaId, or META_WABA_ID)".to_string());
    }

    // No Graph request can be built without a version, and inventing one would
    // report a platform version nobody configured.
    let api_version = match meta_api_version(state) {
        Some(version) => version,
        None => {
            missing.push("Meta WhatsApp API version (config.meta_whatsapp_api_version or META_WHATSAPP_API_VERSION)".to_string());
            String::new()
        }
    };

    if !missing.is_empty() || !database_errors.is_empty() {
        return Err(CredentialRefusal {
            missing,
            database_errors,
        });
    }

    Ok(GraphCredentials {
        token: token.trim().to_string(),
        phone_number_id: phone_number_id.trim().to_string(),
        waba_id: waba_id.trim().to_string(),
        api_version,
        sources: vec![token_source, phone_source, waba_source],
    })
}

/// The shared shape for "we did not ask Meta anything, and here is why".
fn credential_refusal(
    refusal: &CredentialRefusal,
    endpoint: &str,
    reason: &str,
) -> (StatusCode, Json<Value>) {
    let (code, mut body) = refuse(endpoint, reason);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("missingFields".into(), json!(refusal.missing));
        obj.insert("databaseErrors".into(), json!(refusal.database_errors));
        obj.insert(
            "status".into(),
            json!(if refusal.database_errors.is_empty() { "unconfigured" } else { "database_error" }),
        );
    }
    let code = if refusal.database_errors.is_empty() {
        code
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (code, Json(body))
}

#[derive(Debug, Default, Deserialize)]
pub struct TemplateQuery {
    pub limit: Option<i64>,
    pub status: Option<String>,
}

// Meta's own words, or its HTTP status, never a generic failure string.
fn meta_error(body: &Value, status: Option<u16>, raw: &str) -> String {
    if let Some(provider) = provider_error(body) {
        return provider;
    }
    match status {
        Some(code) => format!(
            "Meta returned HTTP {code}: {}",
            if raw.trim().is_empty() { "no body".to_string() } else { truncate(raw, 300) }
        ),
        None => "Meta returned no HTTP status".to_string(),
    }
}

struct MetaReply {
    status: u16,
    body: Value,
    raw: String,
    body_error: Option<String>,
}

async fn read_reply(response: reqwest::Response, endpoint: &str) -> MetaReply {
    let status = response.status().as_u16();
    let (raw, body_error) = match response.text().await {
        Ok(raw) => (raw, None),
        Err(err) => (
            String::new(),
            Some(format!("Meta answered HTTP {status} at {endpoint}, but the body could not be read: {err}")),
        ),
    };
    let body = match serde_json::from_str::<Value>(&raw) {
        Ok(body) => body,
        Err(_) => json!({}),
    };
    let body_error = body_error.or_else(|| {
        if raw.trim().is_empty() {
            Some(format!("Meta answered HTTP {status} with an empty body"))
        } else {
            None
        }
    });
    MetaReply {
        status,
        body,
        raw,
        body_error,
    }
}

/// POST /settings/whatsapp/verify -- asks Meta for the phone number's own
/// record and persists exactly what it answers.
pub async fn verify_whatsapp_settings(State(state): State<AppState>) -> Response {
    let creds = match graph_credentials(&state, true, false).await {
        Ok(creds) => creds,
        Err(refusal) => {
            return credential_refusal(
                &refusal,
                &format!("{} (not contacted)", graph_base("<version unset>")),
                "WhatsApp credentials or the Graph API version are not configured, so no request was sent to Meta.",
            )
            .into_response()
        }
    };

    let endpoint = format!("{}/{}/", graph_base(&creds.api_version), creds.phone_number_id);
    let started = Instant::now();
    let client = &state.http_client;
    let request = client
        .get(&endpoint)
        .timeout(std::time::Duration::from_secs(GRAPH_TIMEOUT_SECS))
        .header("Authorization", format!("Bearer {}", creds.token))
        .query(&[("fields", "verified_name,display_phone_number,quality_rating,platform_type")])
        .build();

    let response = match request {
        Ok(request) => client.execute(request).await,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "verified": false,
                    "status": "requestInvalid",
                    "liveCallMade": false,
                    "endpoint": endpoint,
                    "error": format!("could not build the Meta request: {err}"),
                })),
            )
                .into_response();
        }
    };

    let reply = match response {
        Ok(response) => read_reply(response, &endpoint).await,
        Err(err) => {
            let reason = if err.is_timeout() {
                format!("Meta did not answer within {GRAPH_TIMEOUT_SECS}s")
            } else if err.is_connect() {
                "could not connect to graph.facebook.com".to_string()
            } else {
                err.to_string()
            };
            let cleared = clear_verification(&state).await;
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "verified": false,
                    "status": "unreachable",
                    "liveCallMade": false,
                    "endpoint": endpoint,
                    "error": reason,
                    "verifiedRecordCleared": cleared,
                })),
            )
                .into_response();
        }
    };
    let latency_ms = started.elapsed().as_millis() as i64;

    if reply.status != 200 {
        let cleared = clear_verification(&state).await;
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "verified": false,
                "status": "rejected",
                "liveCallMade": true,
                "httpStatus": reply.status,
                "latencyMs": latency_ms,
                "endpoint": endpoint,
                "error": meta_error(&reply.body, Some(reply.status), &reply.raw),
                "verifiedRecordCleared": cleared,
                "credentialSources": creds.sources,
            })),
        )
            .into_response();
    }

    let verified_name = match reply.body.get("verified_name").and_then(|v| v.as_str()) {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => {
            let cleared = clear_verification(&state).await;
            return (
                StatusCode::OK,
                Json(json!({
                    "verified": false,
                    "status": "unverified",
                    "liveCallMade": true,
                    "httpStatus": reply.status,
                    "latencyMs": latency_ms,
                    "endpoint": endpoint,
                    "verifiedName": Value::Null,
                    "error": reply.body_error.unwrap_or_else(|| {
                        "Meta answered HTTP 200 without a verified_name for this phone number id, which means the number is not business-verified".to_string()
                    }),
                    "providerResponse": truncate(&reply.raw, 300),
                    "verifiedRecordCleared": cleared,
                    "credentialSources": creds.sources,
                })),
            )
                .into_response();
        }
    };

    let display_number = match reply.body.get("display_phone_number").and_then(|v| v.as_str()) {
        Some(number) if !number.trim().is_empty() => json!(number.trim()),
        _ => Value::Null,
    };
    let stored = persist_verification(
        &state,
        &verified_name,
        match &display_number {
            Value::String(number) => number.as_str(),
            _ => "",
        },
    )
    .await;

    Json(json!({
        "status": "connected",
        "verified": true,
        "liveCallMade": true,
        "httpStatus": reply.status,
        "latencyMs": latency_ms,
        "endpoint": endpoint,
        "verifiedName": verified_name,
        "displayNumber": display_number,
        "qualityRating": reply.body.get("quality_rating"),
        "platformType": reply.body.get("platform_type"),
        "verifiedAt": Utc::now().to_rfc3339(),
        "persisted": stored,
        "credentialSources": creds.sources,
    }))
    .into_response()
}

/// Stores exactly what Meta said about this phone number, reporting the write.
async fn persist_verification(state: &AppState, verified_name: &str, display_number: &str) -> Value {
    let now = Utc::now().to_rfc3339();
    let (mode, result) = match load_settings(state).await {
        Err(err) => {
            return json!({
                "ok": false,
                "reason": format!("could not read integration_settings: {err}"),
            })
        }
        Ok(Some(row)) => (
            "updated",
            sqlx::query(
                "UPDATE integration_settings SET whatsapp_verified_at = $1, whatsapp_verified_name = $2, whatsapp_display_number = $3, updated_at = $4 WHERE id = $5",
            )
            .bind(&now)
            .bind(verified_name)
            .bind(display_number)
            .bind(&now)
            .bind(&row.id)
            .execute(&state.db)
            .await
            .map(|result| result.rows_affected()),
        ),
        Ok(None) => (
            "inserted",
            sqlx::query(
                "INSERT INTO integration_settings (id, tenant_key, whatsapp_provider, whatsapp_verified_at, whatsapp_verified_name, whatsapp_display_number, updated_at) VALUES ($1, $2, 'meta', $3, $4, $5, $6)",
            )
            .bind(Uuid::new_v4().simple().to_string())
            .bind(env_value("DEFAULT_TENANT_KEY"))
            .bind(&now)
            .bind(verified_name)
            .bind(display_number)
            .bind(&now)
            .execute(&state.db)
            .await
            .map(|result| result.rows_affected()),
        ),
    };
    let mut value = persistence_json(result, "Meta's verification answer");
    if let Some(obj) = value.as_object_mut() {
        obj.insert("mode".into(), json!(mode));
        if mode == "inserted" {
            obj.insert(
                "note".into(),
                json!("no integration_settings row existed, so one was created containing only what Meta reported"),
            );
        }
    }
    value
}

/// A failed check must not leave a stale "verified" stamp behind.
async fn clear_verification(state: &AppState) -> Value {
    let row = match load_settings(state).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return json!({
                "ok": true,
                "rowsAffected": 0,
                "reason": "no integration_settings row existed, so there was nothing to clear",
            })
        }
        Err(err) => {
            return json!({
                "ok": false,
                "reason": format!("could not read integration_settings: {err}"),
            })
        }
    };
    persistence_json(
        sqlx::query(
            "UPDATE integration_settings SET whatsapp_verified_at = NULL, whatsapp_verified_name = '', whatsapp_display_number = '', updated_at = $1 WHERE id = $2",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(&row.id)
        .execute(&state.db)
        .await
        .map(|result| result.rows_affected()),
        "the stored WhatsApp verification stamp",
    )
}

/// GET /settings/whatsapp/templates -- Meta's real template list.
pub async fn list_whatsapp_templates(
    State(state): State<AppState>,
    Query(query): Query<TemplateQuery>,
) -> Response {
    let creds = match graph_credentials(&state, false, true).await {
        Ok(creds) => creds,
        Err(refusal) => {
            return credential_refusal(
                &refusal,
                "https://graph.facebook.com/<version>/<waba_id>/message_templates (not contacted)",
                "no usable WABA id or access token, so Meta was not asked for templates",
            )
            .into_response()
        }
    };

    let endpoint = format!("{}/{}/message_templates", graph_base(&creds.api_version), creds.waba_id);
    let started = Instant::now();
    let mut params: Vec<(String, String)> = vec![(
        "fields".to_string(),
        "id,name,status,category,language,components,quality_score".to_string(),
    )];
    params.push(("limit".to_string(), query.limit.unwrap_or(100).clamp(1, 250).to_string()));
    if let Some(status) = query.status.as_deref().filter(|s| usable(s)) {
        params.push(("status".to_string(), status.to_string()));
    }

    let response = state
        .http_client
        .get(&endpoint)
        .timeout(std::time::Duration::from_secs(GRAPH_TIMEOUT_SECS))
        .header("Authorization", format!("Bearer {}", creds.token))
        .query(&params)
        .send()
        .await;

    let reply = match response {
        Ok(response) => read_reply(response, &endpoint).await,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "templates": [],
                    "liveCallMade": false,
                    "endpoint": endpoint,
                    "error": if err.is_timeout() {
                        format!("Meta did not answer within {GRAPH_TIMEOUT_SECS}s")
                    } else {
                        err.to_string()
                    },
                })),
            )
                .into_response();
        }
    };

    if reply.status != 200 {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "templates": [],
                "liveCallMade": true,
                "httpStatus": reply.status,
                "endpoint": endpoint,
                "error": meta_error(&reply.body, Some(reply.status), &reply.raw),
                "providerResponse": reply.body_error,
            })),
        )
            .into_response();
    }

    // A body without a data array is not "zero templates" -- it is unknown.
    let templates = match reply.body.get("data") {
        Some(Value::Array(rows)) => rows
            .iter()
            .map(|template| {
                let body_text = template
                    .get("components")
                    .and_then(|v| v.as_array())
                    .and_then(|components| {
                        components
                            .iter()
                            .find(|c| c.get("type").and_then(|t| t.as_str()) == Some("BODY"))
                            .and_then(|c| c.get("text").and_then(|t| t.as_str()))
                    });
                json!({
                    "id": template.get("id"),
                    "name": template.get("name"),
                    "status": template.get("status"),
                    "category": template.get("category"),
                    "language": template.get("language"),
                    "qualityScore": template.get("quality_score"),
                    "bodyText": match body_text {
                        Some(text) => json!(text),
                        None => Value::Null,
                    },
                })
            })
            .collect::<Vec<Value>>(),
        other => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "templates": Value::Null,
                    "count": Value::Null,
                    "liveCallMade": true,
                    "httpStatus": reply.status,
                    "endpoint": endpoint,
                    "error": format!(
                        "Meta answered HTTP 200 but its body carried no data array ({}), so templates cannot be reported",
                        match other {
                            Some(_) => "data was not an array",
                            None => if reply.raw.trim().is_empty() { "no body" } else { "key \"data\" absent" },
                        }
                    ),
                    "rawResponse": truncate(&reply.raw, 300),
                    "providerResponse": reply.body_error,
                })),
            )
                .into_response();
        }
    };

    Json(json!({
        "templates": templates,
        "count": templates.len(),
        "liveCallMade": true,
        "httpStatus": reply.status,
        "latencyMs": started.elapsed().as_millis() as i64,
        "paging": reply.body.get("paging"),
        "wabaId": creds.waba_id,
        "apiVersion": creds.api_version,
    }))
    .into_response()
}

#[derive(Debug, Default, Deserialize)]
pub struct TemplatePayload {
    pub name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default, alias = "bodyText")]
    pub body_text: Option<String>,
    #[serde(default)]
    pub components: Option<Vec<Value>>,
}

/// POST /settings/whatsapp/templates -- Meta validates the name/category and
/// returns the created template id; nothing is reported as created until it
/// answers.
pub async fn create_whatsapp_template(
    State(state): State<AppState>,
    Json(payload): Json<TemplatePayload>,
) -> Response {
    let name = match payload.name.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "template name is required", "missing": ["name"] })),
            )
                .into_response();
        }
    };
    let valid_name = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid_name || name.len() > 50 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Meta rejects this name up front",
                "detail": "template names must be lowercase letters, digits and underscores, at most 50 characters",
                "name": name,
            })),
        )
            .into_response();
    }

    // Meta bills and reviews by category, so guessing one would act on the
    // owner's behalf.
    let category = match payload.category.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        Some(category) => category.to_ascii_uppercase(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "category is required; defaulting it would pick a Meta billing class the owner never chose",
                    "missing": ["category"],
                    "supported": ["MARKETING", "UTILITY", "AUTHENTICATION", "ISSUE_RESOLUTION"],
                })),
            )
                .into_response();
        }
    };
    if !["MARKETING", "UTILITY", "AUTHENTICATION", "ISSUE_RESOLUTION"].contains(&category.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("unsupported template category {category}"),
                "supported": ["MARKETING", "UTILITY", "AUTHENTICATION", "ISSUE_RESOLUTION"],
            })),
        )
            .into_response();
    }

    let language = match payload.language.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        Some(language) => language.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "language is required; defaulting it would publish a template in a language the owner did not pick",
                    "missing": ["language"],
                    "detail": "Meta expects a BCP-47 code such as en_US or pt_BR",
                })),
            )
                .into_response();
        }
    };

    let body_text = match payload.body_text.as_deref().map(str::trim) {
        Some(text) => text.to_string(),
        None => String::new(),
    };
    if body_text.is_empty() && payload.components.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "body_text is required (Meta will not accept a template with no BODY component)",
                "missing": ["body_text"],
            })),
        )
            .into_response();
    }

    let creds = match graph_credentials(&state, false, true).await {
        Ok(creds) => creds,
        Err(refusal) => {
            return credential_refusal(
                &refusal,
                "https://graph.facebook.com/<version>/<waba_id>/message_templates (not contacted)",
                "Meta was not asked to create the template because credentials are missing",
            )
            .into_response()
        }
    };

    let components: Value = match payload.components.as_ref() {
        Some(existing) if !existing.is_empty() => json!(existing),
        _ => json!([{ "type": "BODY", "text": body_text }]),
    };

    let endpoint = format!("{}/{}/message_templates", graph_base(&creds.api_version), creds.waba_id);
    let response = state
        .http_client
        .post(&endpoint)
        .timeout(std::time::Duration::from_secs(GRAPH_TIMEOUT_SECS))
        .header("Authorization", format!("Bearer {}", creds.token))
        .json(&json!({
            "name": name,
            "category": category,
            "language": language,
            "components": components,
        }))
        .send()
        .await;

    let reply = match response {
        Ok(response) => read_reply(response, &endpoint).await,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "created": false,
                    "liveCallMade": false,
                    "endpoint": endpoint,
                    "error": if err.is_timeout() {
                        format!("Meta did not answer within {GRAPH_TIMEOUT_SECS}s")
                    } else {
                        err.to_string()
                    },
                })),
            )
                .into_response();
        }
    };

    let accepted = matches!(reply.status, 200 | 201);
    if !accepted {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "created": false,
                "liveCallMade": true,
                "httpStatus": reply.status,
                "endpoint": endpoint,
                "error": meta_error(&reply.body, Some(reply.status), &reply.raw),
                "providerResponse": reply.body_error,
            })),
        )
            .into_response();
    }

    let template_id = reply.body.get("id").cloned();
    let id_reported = match &template_id {
        Some(Value::String(id)) => !id.trim().is_empty(),
        Some(other) => !other.is_null(),
        None => false,
    };

    (
        StatusCode::CREATED,
        Json(json!({
            "created": true,
            "liveCallMade": true,
            "httpStatus": reply.status,
            "endpoint": endpoint,
            "response": reply.body,
            "id": template_id,
            "name": name,
            "category": category,
            "language": language,
            "note": if id_reported {
                "Meta reviews templates asynchronously; poll GET /settings/whatsapp/templates for the real status"
            } else {
                "Meta accepted the request but its body carried no usable template id, so the id is reported as unknown"
            },
            "bodyError": reply.body_error,
        })),
    )
        .into_response()
}

/// DELETE /settings/whatsapp/templates/{name}
pub async fn delete_whatsapp_template(State(state): State<AppState>, Path(name): Path<String>) -> Response {
    let name = name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "template name is required" })),
        )
            .into_response();
    }

    let creds = match graph_credentials(&state, false, true).await {
        Ok(creds) => creds,
        Err(refusal) => {
            return credential_refusal(
                &refusal,
                "https://graph.facebook.com/<version>/<waba_id>/message_templates (not contacted)",
                "Meta was not asked to delete the template because credentials are missing",
            )
            .into_response()
        }
    };

    let endpoint = format!("{}/{}/message_templates", graph_base(&creds.api_version), creds.waba_id);
    let response = state
        .http_client
        .delete(&endpoint)
        .timeout(std::time::Duration::from_secs(GRAPH_TIMEOUT_SECS))
        .header("Authorization", format!("Bearer {}", creds.token))
        .query(&[("name", name.as_str())])
        .send()
        .await;

    let reply = match response {
        Ok(response) => read_reply(response, &endpoint).await,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "deleted": false,
                    "liveCallMade": false,
                    "templateName": name,
                    "endpoint": endpoint,
                    "error": if err.is_timeout() {
                        format!("Meta did not answer within {GRAPH_TIMEOUT_SECS}s")
                    } else {
                        err.to_string()
                    },
                })),
            )
                .into_response();
        }
    };

    let success = match reply.body.get("success") {
        Some(Value::Bool(flag)) => Some(*flag),
        _ => None,
    };
    let deleted = reply.status == 200 && success == Some(true);
    let error = if deleted {
        None
    } else if reply.status != 200 {
        Some(meta_error(&reply.body, Some(reply.status), &reply.raw))
    } else {
        Some(match success {
            Some(false) => "Meta reported success:false for the delete request".to_string(),
            _ => format!(
                "Meta answered HTTP 200 without a success flag, so the delete is unconfirmed: {}",
                if reply.raw.trim().is_empty() { "no body".to_string() } else { truncate(&reply.raw, 200) }
            ),
        })
    };

    (
        if deleted { StatusCode::OK } else { StatusCode::BAD_GATEWAY },
        Json(json!({
            "status": if deleted { "deleted" } else { "rejected" },
            "deleted": deleted,
            "templateName": name,
            "liveCallMade": true,
            "httpStatus": reply.status,
            "endpoint": endpoint,
            "providerResponse": reply.body,
            "rawResponse": truncate(&reply.raw, 300),
            "bodyError": reply.body_error,
            "error": error,
        })),
    )
        .into_response()
}
