use crate::db::AppState;
use crate::services::inbound::{self, InboundMessage};
use crate::services::signatures::{
    secrets_match, validate_line_signature, validate_meta_signature, validate_tiktok_signature,
    validate_twilio_signature,
};
use crate::services::whatsapp::WhatsAppService;
use crate::services::whatsapp_automations::WhatsAppAutomationEngine;
use axum::body::Bytes;
use axum::extract::{OriginalUri, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use sqlx::Row;

fn rejected(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// Reconstructs the absolute URL the provider signed. Twilio signs the exact
/// callback URL, so the proxy's forwarded scheme has to win over the local one.
fn absolute_url(headers: &HeaderMap, uri: &OriginalUri) -> String {
    let scheme = header(headers, "x-forwarded-proto").unwrap_or("http");
    let host = header(headers, "x-forwarded-host")
        .or_else(|| header(headers, "host"))
        .unwrap_or("localhost");
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    format!("{scheme}://{host}{path}{query}")
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    // Invalid UTF-8 in a form field is treated as opaque bytes: keep the input
    // rather than discarding a message body we could still store.
    String::from_utf8(out).unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned())
}

fn parse_form(body: &[u8]) -> Vec<(String, String)> {
    std::str::from_utf8(body)
        .unwrap_or("")
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(key), percent_decode(value))
        })
        .collect()
}

fn form_value(params: &[(String, String)], key: &str) -> String {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

/// Which tenant owns a channel-originated contact. Provider webhooks carry no
/// session, so ownership comes from the inbox the tenant connected, falling back
/// to the configured default. With neither set the request is refused rather
/// than filed under an invented tenant.
async fn resolve_tenant(state: &AppState, channel: &str) -> Result<String, String> {
    let from_inbox = match sqlx::query("SELECT tenant_key FROM inboxes WHERE channel_type = $1 AND is_active = 1 LIMIT 1")
        .bind(channel)
        .fetch_optional(&state.db)
        .await
    {
        Ok(row) => row
            .and_then(|row| row.try_get::<Option<String>, _>("tenant_key").ok().flatten())
            .filter(|value| !value.is_empty()),
        // WHY: an unreadable inboxes table is not "no inbox row": falling through
        // to DEFAULT_TENANT_KEY would file the contact under a tenant that may not
        // own this channel at all.
        Err(err) => {
            tracing::error!("The {channel} inbox owner could not be read: {err}");
            return Err(format!(
                "The tenant owning the {channel} inbox could not be determined: {err}"
            ));
        }
    };

    from_inbox
        .or_else(|| Some(state.config.default_tenant_key.clone()).filter(|value| !value.is_empty()))
        .ok_or_else(|| {
            format!("No tenant is configured for the {channel} inbox: set tenant_key on the inbox or DEFAULT_TENANT_KEY.")
        })
}

// ---------------------------------------------------------------------------
// Meta WhatsApp Cloud API
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct HubVerifyParams {
    #[serde(rename = "hub.mode")]
    hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    hub_challenge: Option<String>,
}

/// Meta's subscription handshake: the platform GETs the callback URL once with
/// `hub.mode=subscribe`, a token we chose, and a challenge it expects echoed
/// back verbatim. An unconfigured token refuses rather than echoing anything.
fn meta_handshake(params: HubVerifyParams, expected: String, unset_hint: &str) -> Response {
    if expected.is_empty() {
        return rejected(StatusCode::SERVICE_UNAVAILABLE, unset_hint);
    }
    let provided = params.hub_verify_token.unwrap_or_default();
    if params.hub_mode.as_deref() != Some("subscribe")
        || !secrets_match(&expected, Some(provided.as_str()))
    {
        return rejected(StatusCode::FORBIDDEN, "Verification token mismatch");
    }
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        params.hub_challenge.unwrap_or_default(),
    )
        .into_response()
}

/// Per-inbox provider credentials, with values the UI has masked back for
/// display (`••••1234`) discarded -- they are not secrets, and accepting them
/// would let a stored display string authenticate a webhook.
async fn inbox_credentials(state: &AppState, channel: &str) -> Value {
    let row = sqlx::query("SELECT credentials FROM inboxes WHERE channel_type = $1 AND is_active = 1 LIMIT 1")
        .bind(channel)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
    match row.and_then(|r| r.try_get::<Option<String>, _>("credentials").ok().flatten()) {
        Some(raw) => serde_json::from_str(&raw).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

fn credential(creds: &Value, key: &str) -> String {
    creds
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty() && !v.chars().any(|c| c == '•'))
        .unwrap_or_default()
        .to_string()
}

pub async fn meta_whatsapp_verify(
    State(state): State<AppState>,
    Query(params): Query<HubVerifyParams>,
) -> Response {
    let hint = "Meta webhook not configured: set META_WEBHOOK_VERIFY_TOKEN.";
    meta_handshake(params, state.config.meta_webhook_verify_token.clone(), hint)
}

fn normalize_meta_sender(sender_id: &str) -> String {
    if sender_id.is_empty() {
        return String::new();
    }
    format!("+{}", sender_id.trim_start_matches('+'))
}

fn extract_meta_message_text(msg: &Value) -> String {
    let obj = |keys: &[&str]| -> Option<String> {
        keys.iter()
            .fold(Some(msg), |acc, key| acc.and_then(|v| v.get(*key)))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
    };

    match msg.get("type").and_then(|v| v.as_str()).unwrap_or("") {
        "text" => obj(&["text", "body"]).unwrap_or_default(),
        "interactive" => {
            let kind = msg
                .pointer("/interactive/type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match kind {
                "button_reply" => obj(&["interactive", "button_reply", "title"]).unwrap_or_default(),
                "list_reply" => obj(&["interactive", "list_reply", "title"]).unwrap_or_default(),
                _ => String::new(),
            }
        }
        "button" => obj(&["button", "text"]).unwrap_or_default(),
        "image" => match obj(&["image", "caption"]) {
            Some(caption) if !caption.is_empty() => caption,
            _ => "[User sent an image]".to_string(),
        },
        "document" => match obj(&["document", "caption"]) {
            Some(caption) if !caption.is_empty() => caption,
            _ => format!(
                "[User sent document: {}]",
                obj(&["document", "filename"]).unwrap_or_else(|| "document".to_string())
            ),
        },
        "audio" => "[User sent an audio message]".to_string(),
        "video" => match obj(&["video", "caption"]) {
            Some(caption) if !caption.is_empty() => caption,
            _ => "[User sent a video]".to_string(),
        },
        "location" => {
            let name = obj(&["location", "name"])
                .filter(|v| !v.is_empty())
                .or_else(|| obj(&["location", "address"]));
            match name {
                Some(name) => format!("[User shared location: {name}]"),
                None => "[User shared location]".to_string(),
            }
        }
        _ => String::new(),
    }
}

fn extract_meta_payload_id(msg: &Value) -> Option<String> {
    match msg.get("type").and_then(|v| v.as_str()).unwrap_or("") {
        "interactive" => {
            let kind = msg
                .pointer("/interactive/type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match kind {
                "button_reply" => msg
                    .pointer("/interactive/button_reply/id")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                "list_reply" => msg
                    .pointer("/interactive/list_reply/id")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                _ => None,
            }
        }
        "button" => msg
            .pointer("/button/payload")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        _ => None,
    }
}

fn meta_status_to_message_status(meta_status: &str) -> Option<&'static str> {
    match meta_status {
        "sent" => Some("sent"),
        "delivered" => Some("delivered"),
        "read" => Some("read"),
        "failed" => Some("failed"),
        _ => None,
    }
}

fn meta_status_to_recipient_status(meta_status: &str) -> Option<&'static str> {
    match meta_status {
        "sent" => Some("sent"),
        "delivered" => Some("delivered"),
        "read" => Some("read"),
        "failed" => Some("failed"),
        _ => None,
    }
}

fn meta_failure_reason(status_event: &Value) -> String {
    let Some(errors) = status_event.get("errors").and_then(|v| v.as_array()) else {
        return "Message delivery failed".to_string();
    };
    let Some(err) = errors.first() else {
        return "Message delivery failed".to_string();
    };

    let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or_default();
    let title = err
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let details = err
        .get("error_data")
        .and_then(|data| data.get("details"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|v| !v.is_empty())
        .or_else(|| err.get("message").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();

    if code == 131026
        || title.to_ascii_lowercase().contains("not on whatsapp")
        || details.to_ascii_lowercase().contains("not on whatsapp")
    {
        return "Number is not on WhatsApp".to_string();
    }
    if !details.is_empty() {
        return format!("{title}: {details}");
    }
    if !title.is_empty() {
        return title;
    }
    format!("Meta error {code}")
}

/// WHY: a receipt can fail for a reason `sqlx::Error` cannot express -- the
/// statement ran but stored nothing -- and the webhook response must be able to
/// say "this receipt was not applied" rather than counting it as one.
enum ReceiptFailure {
    Db(sqlx::Error),
    NotWritten(String),
}

impl From<sqlx::Error> for ReceiptFailure {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err)
    }
}

impl std::fmt::Display for ReceiptFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(err) => write!(f, "{err}"),
            Self::NotWritten(note) => f.write_str(note),
        }
    }
}

/// Delivery/read receipts for messages we sent. Updates the 1:1 Message row
/// and, for campaign sends, the BroadcastRecipient plus the broadcast counters.
/// The bool says whether at least one row was genuinely written: `receipts` in
/// the webhook response may only count applied receipts.
async fn apply_meta_status(state: &AppState, status_event: &Value) -> Result<bool, ReceiptFailure> {
    let provider_message_id = status_event.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let meta_status = status_event.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let Some(message_status) = meta_status_to_message_status(meta_status) else {
        tracing::warn!("Meta status event '{meta_status}' for provider message {provider_message_id} has no message-status mapping; nothing was written");
        return Ok(false);
    };
    if provider_message_id.is_empty() {
        tracing::warn!("Meta {meta_status} event carried no message id; nothing was written");
        return Ok(false);
    }

    let event_at = status_event
        .get("timestamp")
        .and_then(|v| {
            v.as_str()
                .map(str::to_string)
                .or_else(|| v.as_i64().map(|n| n.to_string()))
        })
        .and_then(|raw| raw.parse::<i64>().ok())
        .and_then(chrono::DateTime::from_timestamp_secs)
        .unwrap_or_else(chrono::Utc::now);
    let event_at_text = event_at.to_rfc3339();

    let failure_reason = if meta_status == "failed" {
        Some(meta_failure_reason(status_event))
    } else {
        None
    };

    let mut applied = false;
    match inbound::apply_delivery_status(
        &state.db,
        provider_message_id,
        message_status,
        Some(status_event.clone()),
    )
    .await
    {
        Ok(true) => applied = true,
        Ok(false) => {
            tracing::warn!(
                "Meta {meta_status} receipt for provider message {provider_message_id} matched no \
                 outbound message row (campaign-only sends have one)"
            );
        }
        // WHY: the receipt write is the whole point of this handler; reporting it
        // as applied when it errored would leave the message stuck at its old
        // status while the provider was told the receipt landed.
        Err(err) => {
            tracing::error!(
                "Meta {meta_status} receipt for provider message {provider_message_id} could not be \
                 written to the message row: {err:?}"
            );
            return Err(ReceiptFailure::NotWritten(format!(
                "message receipt for {provider_message_id} was NOT stored: {err:?}"
            )));
        }
    }

    let recipient = sqlx::query(
        "SELECT id, broadcast_id, status FROM broadcast_recipients WHERE provider_message_id = $1 LIMIT 1",
    )
    .bind(provider_message_id)
    .fetch_optional(&state.db)
    .await?;
    let Some(recipient) = recipient else {
        return Ok(applied);
    };

    let recipient_id: String = recipient.try_get("id")?;
    let broadcast_id: Option<String> = recipient.try_get("broadcast_id").ok().flatten();
    let old_status: Option<String> = recipient.try_get("status").ok().flatten();
    let Some(new_status) = meta_status_to_recipient_status(meta_status) else {
        return Ok(applied);
    };

    let report = match meta_status {
        "sent" => sqlx::query(
            "UPDATE broadcast_recipients SET status = $1, sent_at = COALESCE(sent_at, $2) WHERE id = $3",
        )
        .bind(new_status)
        .bind(&event_at_text)
        .bind(&recipient_id)
        .execute(&state.db)
        .await?,
        "delivered" => sqlx::query(
            "UPDATE broadcast_recipients SET status = $1, delivered_at = COALESCE(delivered_at, $2) WHERE id = $3",
        )
        .bind(new_status)
        .bind(&event_at_text)
        .bind(&recipient_id)
        .execute(&state.db)
        .await?,
        "read" => sqlx::query(
            "UPDATE broadcast_recipients SET status = $1, read_at = COALESCE(read_at, $2) WHERE id = $3",
        )
        .bind(new_status)
        .bind(&event_at_text)
        .bind(&recipient_id)
        .execute(&state.db)
        .await?,
        "failed" => {
            let reason = failure_reason.unwrap_or_else(|| "Message delivery failed".to_string());
            sqlx::query(
                "UPDATE broadcast_recipients SET status = $1, error = $2 WHERE id = $3",
            )
            .bind(new_status)
            .bind(&reason)
            .bind(&recipient_id)
            .execute(&state.db)
            .await?
        }
        _ => {
            return Ok(applied);
        }
    };
    match report.rows_affected() {
        1 => applied = true,
        rows => {
            tracing::error!(
                "Meta {meta_status} receipt was NOT applied to broadcast recipient {recipient_id}: \
                 the UPDATE affected {rows} rows (expected 1)"
            );
            return Err(ReceiptFailure::NotWritten(format!(
                "recipient receipt for {recipient_id} was NOT stored ({rows} rows affected)"
            )));
        }
    }

    // Auxiliary: the broadcast counters. The receipt above is stored and reported
    // regardless, so a counter that cannot be written is logged, not hidden.
    match (broadcast_id, &old_status) {
        (Some(broadcast_id), Some(old)) => {
            let mut counter: Option<&str> = None;
            let mut delivered_increment = false;
            if meta_status == "delivered" && old != "delivered" && old != "read" {
                counter = Some("delivered_count");
            } else if meta_status == "read" {
                counter = Some("read_count");
                if old != "delivered" && old != "read" {
                    delivered_increment = true;
                }
            } else if meta_status == "failed" && old != "failed" {
                counter = Some("failed_count");
            }

            if let Some(column) = counter {
                // Column name comes from a closed match (delivered_count, read_count, failed_count).
                let query_str = format!(
                    "UPDATE broadcasts SET {column} = COALESCE({column}, 0) + 1 WHERE id = $1"
                );
                crate::db::report_write(
                    sqlx::query(&query_str)
                        .bind(&broadcast_id)
                        .execute(&state.db)
                        .await
                        .map(|report| report.rows_affected()),
                    &format!("broadcasts.{column} after a Meta {meta_status} receipt"),
                );
            }
            if delivered_increment {
                crate::db::report_write(
                    sqlx::query(
                        "UPDATE broadcasts SET delivered_count = COALESCE(delivered_count, 0) + 1 WHERE id = $1",
                    )
                    .bind(&broadcast_id)
                    .execute(&state.db)
                    .await
                    .map(|report| report.rows_affected()),
                    &format!("broadcasts.delivered_count after a Meta read receipt"),
                );
            }
        }
        (Some(broadcast_id), None) => tracing::warn!(
            "Broadcast {broadcast_id} counters were left alone for recipient {recipient_id}: its \
             stored status could not be read, so whether a delivered/read increment is due is unknown"
        ),
        (None, _) => {}
    }

    Ok(applied)
}

/// Says whether the profile `refresh_channel_info` fetched from Meta is actually
/// the row now stored in `integration_settings`, naming what is not.
async fn verify_channel_info(state: &AppState, profile: &Value) -> Result<String, String> {
    if profile.get("isConnected").and_then(|v| v.as_bool()) != Some(true) {
        let reason = profile
            .get("error")
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .unwrap_or("Meta returned no error text");
        return Err(format!(
            "Meta did not return a live profile ({reason}); the stored channel info is unchanged"
        ));
    }

    let mut expected: Vec<(&str, &str)> = Vec::new();
    for (column, key) in [
        ("whatsapp_display_number", "displayPhoneNumber"),
        ("whatsapp_verified_name", "verifiedName"),
    ] {
        if let Some(value) = profile.get(key).and_then(|v| v.as_str()).filter(|v| !v.is_empty()) {
            expected.push((column, value));
        }
    }
    if expected.is_empty() {
        return Err(
            "Meta reported a connected profile but no display number or verified name, so there was \
             nothing to store"
            .to_string(),
        );
    }

    // The same row `refresh_channel_info` writes: targets the first row in
    // integration_settings (PostgreSQL-compatible — no SQLite rowid).
    let row = sqlx::query(
        "SELECT whatsapp_display_number, whatsapp_verified_name FROM integration_settings LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|err| format!("the stored channel info could not be read back to verify it: {err}"))?;
    let Some(row) = row else {
        return Err(format!(
            "there is no integration_settings row, so the fetched values ({}) were not stored",
            expected.iter().map(|(column, _)| *column).collect::<Vec<_>>().join(", ")
        ));
    };

    let mut stale = Vec::new();
    for (column, want) in &expected {
        let stored: Option<String> = row.try_get(*column).ok().flatten();
        if stored.as_deref() != Some(want) {
            stale.push(format!("{column} is still '{}' while Meta says '{want}'", stored.unwrap_or_default()));
        }
    }
    if !stale.is_empty() {
        return Err(format!("the profile was fetched from Meta but not stored: {}", stale.join("; ")));
    }
    Ok(format!("{} stored channel value(s) match what Meta reported", expected.len()))
}

// ---------------------------------------------------------------------------
// WhatsApp brand resolution and automation helpers
// ---------------------------------------------------------------------------

/// Resolve brand_id from a WhatsApp sender phone number.
/// Looks up the inbox for this channel, then gets the brand_id.
async fn resolve_whatsapp_brand(pool: &sqlx::PgPool, from_num: &str) -> Option<String> {
    // Try to find the conversation for this sender, which may have a brand_id
    let conv = sqlx::query(r#"
        SELECT c.brand_id FROM conversations c
        JOIN contacts ct ON ct.id = c.contact_id
        WHERE ct.channel = 'whatsapp' AND ct.value = $1
        AND c.brand_id IS NOT NULL AND c.brand_id != ''
        ORDER BY c.updated_at DESC LIMIT 1
    "#)
    .bind(from_num)
    .fetch_optional(pool)
    .await;

    if let Ok(Some(row)) = conv {
        if let Ok(Some(brand_id)) = row.try_get::<Option<String>, _>("brand_id") {
            if !brand_id.is_empty() {
                return Some(brand_id);
            }
        }
    }

    // Fallback: find the first active WhatsApp inbox with a brand_id
    let inbox = sqlx::query(r#"
        SELECT brand_id FROM inboxes
        WHERE channel_type = 'whatsapp' AND is_active = 1
        AND brand_id IS NOT NULL AND brand_id != ''
        LIMIT 1
    "#)
    .fetch_optional(pool)
    .await;

    if let Ok(Some(row)) = inbox {
        if let Ok(Some(brand_id)) = row.try_get::<Option<String>, _>("brand_id") {
            if !brand_id.is_empty() {
                return Some(brand_id);
            }
        }
    }

    None
}

/// Sync customer profile data from Meta contacts array to whatsapp_customer_profiles.
/// Uses SELECT-first pattern: if a profile for this wa_id already exists, update it;
/// otherwise insert a new row. This avoids SQLite-only `INSERT OR REPLACE` syntax.
async fn sync_whatsapp_customer_profile(
    pool: &sqlx::PgPool,
    brand_id: &str,
    wa_id: &str,
    profile_name: &str,
    profile_picture_url: &str,
    about: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();

    // Check if a profile for this wa_id already exists
    let existing = sqlx::query(
        "SELECT id FROM whatsapp_customer_profiles WHERE wa_id = $1 LIMIT 1",
    )
    .bind(wa_id)
    .fetch_optional(pool)
    .await;

    match existing {
        Ok(Some(row)) => {
            // Update the existing profile
            let existing_id: String = row.try_get("id").unwrap_or_default();
            let _ = sqlx::query(
                "UPDATE whatsapp_customer_profiles \
                 SET brand_id = $1, profile_name = $2, profile_picture_url = $3, about = $4, \
                     last_fetched_at = $5, updated_at = $6 \
                 WHERE id = $7",
            )
            .bind(brand_id)
            .bind(profile_name)
            .bind(profile_picture_url)
            .bind(about)
            .bind(&now)
            .bind(&now)
            .bind(&existing_id)
            .execute(pool)
            .await;
        }
        _ => {
            // Insert a new profile
            let id = uuid::Uuid::new_v4().simple().to_string();
            let _ = sqlx::query(
                "INSERT INTO whatsapp_customer_profiles \
                 (id, wa_id, brand_id, profile_name, profile_picture_url, about, last_fetched_at, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(&id)
            .bind(wa_id)
            .bind(brand_id)
            .bind(profile_name)
            .bind(profile_picture_url)
            .bind(about)
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await;
        }
    }
}

/// Process WhatsApp automation for an inbound message
async fn process_whatsapp_automation(
    state: &AppState,
    brand_id: &str,
    contact_wa_id: &str,
    conversation_id: &str,
    message_text: &str,
    msg_value: &serde_json::Value,
) {
    let cfg = state.config.as_ref().clone();
    let engine = WhatsAppAutomationEngine::new(state.db.clone(), cfg.clone());
    let wa_service = WhatsAppService::new(state.db.clone(), cfg);

    // 1. Update session window (24h tracking)
    let _ = wa_service.update_session_window(brand_id, contact_wa_id).await;

    // 2. Parse interactive message details
    let interactive_type = msg_value.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let button_id = extract_meta_payload_id(msg_value);

    // 3. Handle agent commands (#bot, #resolve, #csat, #tag) — highest priority
    let agent_reply = engine.handle_agent_commands(brand_id, conversation_id, contact_wa_id, message_text).await;
    if let Some(reply) = agent_reply {
        tracing::info!("Agent command reply: {reply}");
        // Send the reply back to the customer
        let _ = wa_service.send_text(brand_id, contact_wa_id, &reply, false).await;
    }

    // 4. Handle CSAT response (only for interactive/button replies)
    if interactive_type == "interactive" {
        let csat_reply = engine.handle_csat_response(brand_id, conversation_id, contact_wa_id, message_text).await;
        if let Some(reply) = csat_reply {
            tracing::info!("CSAT reply: {reply}");
            let _ = wa_service.send_text(brand_id, contact_wa_id, &reply, false).await;
        }
    }

    // 5. Handle navigation menu (button clicks, list selections)
    if interactive_type == "interactive" {
        if let Some(ref bid) = button_id {
            let menu_reply = engine.handle_navigation_menu(brand_id, contact_wa_id, bid).await;
            if menu_reply.is_some() {
                tracing::info!("Navigation menu handled for payload: {bid}");
            }
        }
    }

    // 6. Match automation rules (keyword/regex/intent)
    match engine.match_advanced_rule(brand_id, message_text).await {
        Ok(Some((rule_id, action_type, _action_config))) => {
            tracing::info!("Automation rule matched: {rule_id}, action: {action_type}");
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!("Automation rule matching failed: {e}");
        }
    }

    // 7. Send welcome greeting if first touch
    let should_greet = engine.should_send_welcome_greeting(brand_id, conversation_id).await.unwrap_or(false);
    if should_greet {
        match engine.send_welcome_greeting(brand_id, conversation_id, contact_wa_id).await {
            Ok(_) => tracing::info!("Welcome greeting sent to {contact_wa_id}"),
            Err(e) => tracing::warn!("Welcome greeting failed: {e}"),
        }
    }

    // 8. Mark as read if auto_read_receipts is enabled for the brand
    let msg_id = msg_value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if !msg_id.is_empty() {
        let auto_read: bool = sqlx::query_scalar(
            "SELECT COALESCE((SELECT json_extract(credentials, '$.auto_read_receipts') FROM brand_credentials WHERE brand_id = $1 AND channel_type = 'whatsapp' AND is_active = 1 LIMIT 1), '0') = '1'"
        )
        .bind(brand_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(Some(false))
        .unwrap_or(false);

        if auto_read {
            let _ = wa_service.mark_as_read(brand_id, msg_id).await;
        }
    }

    // 9. Brand-specific AI auto-reply via reply engine
    // Only call if no automation already responded and auto-reply is enabled
    let auto_reply_enabled: bool = sqlx::query_scalar(
        "SELECT COALESCE((SELECT json_extract(credentials, '$.auto_reply_enabled') FROM brand_credentials WHERE brand_id = $1 AND channel_type = 'whatsapp' AND is_active = 1 LIMIT 1), '0') = '1'"
    )
    .bind(brand_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if auto_reply_enabled {
        // Check if this is a session message (within 24h window)
        let session_active = wa_service.check_session_active(brand_id, contact_wa_id).await.unwrap_or(false);
        if session_active {
            match crate::services::reply_engine::generate_reply(state, conversation_id, None).await {
                Ok(draft) => {
                    if !draft.body.is_empty() {
                        tracing::info!("AI auto-reply generated for conversation {conversation_id}");
                        let _ = wa_service.send_text(brand_id, contact_wa_id, &draft.body, false).await;
                    }
                }
                Err(_e) => {
                    tracing::debug!("Reply engine skipped for conversation {conversation_id}");
                }
            }
        }
    }
}

pub async fn meta_whatsapp_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if state.config.meta_app_secret.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Meta webhook not configured: set META_APP_SECRET.",
        );
    }
    if !validate_meta_signature(
        &state.config.meta_app_secret,
        &body,
        header(&headers, "x-hub-signature-256"),
    ) {
        return rejected(StatusCode::FORBIDDEN, "Invalid Meta webhook signature.");
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return rejected(StatusCode::BAD_REQUEST, "Unreadable webhook payload.");
    };
    if payload.get("object").and_then(|v| v.as_str()) != Some("whatsapp_business_account") {
        return Json(json!({ "status": "ignored" })).into_response();
    }

    let mut ingested = 0usize;
    let mut duplicates = 0usize;
    let mut receipts = 0usize;
    let mut receipts_unmatched = 0usize;
    let mut unknown_contacts = 0usize;
    let mut missing_sender = 0usize;
    let mut syncs = 0usize;
    let mut errors = Vec::new();

    for entry in payload.get("entry").and_then(|v| v.as_array()).into_iter().flatten() {
        for change in entry
            .get("changes")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            // Meta notifies us when the number's rating, display name or WABA
            // account changes. Re-read the live values instead of keeping a
            // stale copy that the UI would present as current.
            let field = change.get("field").and_then(|v| v.as_str()).unwrap_or("");
            if matches!(
                field,
                "phone_number_quality_update" | "phone_number_name_update" | "account_update"
            ) {
                let profile = crate::services::meta_sync::refresh_channel_info(&state).await;
                // WHY: refresh_channel_info reports what Meta returned, not what it
                // stored, so the stored row is re-read and a re-sync is claimed only
                // when the values on file actually match the fetched profile.
                match verify_channel_info(&state, &profile).await {
                    Ok(detail) => {
                        tracing::info!("Meta field update {field}: re-synced live channel info -- {detail}");
                        syncs += 1;
                    }
                    Err(detail) => {
                        tracing::error!("Meta field update {field}: NOT re-synced -- {detail}");
                    }
                }
            }

            let value = change.get("value").cloned().unwrap_or(Value::Null);

            let mut contacts_map = serde_json::Map::new();
            // Collect full contact data for profile sync
            let mut contact_profiles: Vec<(String, String, String, String)> = Vec::new();
            for ct in value.get("contacts").and_then(|v| v.as_array()).into_iter().flatten() {
                let wa_id = ct.get("wa_id").and_then(|v| v.as_str()).unwrap_or("");
                let profile_name = ct
                    .pointer("/profile/name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let profile_picture = ct
                    .pointer("/profile/picture")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let about = ct
                    .pointer("/profile/about")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !wa_id.is_empty() && !profile_name.is_empty() {
                    contacts_map.insert(wa_id.to_string(), json!(profile_name));
                    contacts_map.insert(normalize_meta_sender(wa_id), json!(profile_name));
                }
                if !wa_id.is_empty() {
                    contact_profiles.push((wa_id.to_string(), profile_name.to_string(), profile_picture.to_string(), about.to_string()));
                }
            }

            for msg in value.get("messages").and_then(|v| v.as_array()).into_iter().flatten() {
                let msg_id = msg.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let text = extract_meta_message_text(msg);
                let payload_id = extract_meta_payload_id(msg);
                let mut text = text;
                if text.is_empty() && payload_id.is_none() {
                    continue;
                }
                if text.is_empty() {
                    text = payload_id.clone().unwrap_or_default();
                }

                let from_num = msg.get("from").and_then(|v| v.as_str()).unwrap_or("");
                // WHY: an event with no `from` has no sender to file it under, and
                // looking up an empty sender id could attach the message to whatever
                // contact row happens to carry an empty value -- refused, never invented.
                if from_num.trim().is_empty() {
                    tracing::warn!(
                        "WhatsApp message '{msg_id}' arrived with no 'from' sender; refused instead \
                         of filed under an invented recipient"
                    );
                    missing_sender += 1;
                    continue;
                }
                let sender_name = contacts_map
                    .get(from_num)
                    .or_else(|| contacts_map.get(&normalize_meta_sender(from_num)))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let normalized_from = normalize_meta_sender(from_num);

                let inbound = InboundMessage {
                    channel: "whatsapp",
                    from_value: &normalized_from,
                    body: &text,
                    provider: "meta",
                    provider_message_id: msg_id,
                    sender_name,
                };
                match inbound::process_inbound_message(&state.db, &inbound).await {
                    Ok((message_id, true)) => {
                        duplicates += 1;
                        // Even duplicates need session window updates and automation
                        let conversation_id = sqlx::query_scalar::<_, String>(
                            "SELECT conversation_id FROM messages WHERE id = $1 LIMIT 1"
                        )
                        .bind(&message_id)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                        // Sync customer profile from Meta
                        for (wa_id, name, pic, about) in &contact_profiles {
                            if wa_id == from_num || normalize_meta_sender(wa_id) == normalized_from {
                                let _ = sync_whatsapp_customer_profile(
                                    &state.db, "", wa_id, name, pic, about
                                ).await;
                            }
                        }

                        // Resolve brand and run automation
                        if let Some(brand_id) = resolve_whatsapp_brand(&state.db, &normalized_from).await {
                            if !conversation_id.is_empty() {
                                let _ = process_whatsapp_automation(
                                    &state, &brand_id, &normalized_from, &conversation_id, &text, msg
                                ).await;
                            }
                        }
                    }
                    Ok((message_id, false)) => {
                        ingested += 1;
                        // Get conversation_id for automation
                        let conversation_id = sqlx::query_scalar::<_, String>(
                            "SELECT conversation_id FROM messages WHERE id = $1 LIMIT 1"
                        )
                        .bind(&message_id)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                        // Sync customer profile from Meta
                        for (wa_id, name, pic, about) in &contact_profiles {
                            if wa_id == from_num || normalize_meta_sender(wa_id) == normalized_from {
                                let _ = sync_whatsapp_customer_profile(
                                    &state.db, "", wa_id, name, pic, about
                                ).await;
                            }
                        }

                        // Resolve brand and run automation
                        if let Some(brand_id) = resolve_whatsapp_brand(&state.db, &normalized_from).await {
                            // Update conversation with brand_id
                            if !conversation_id.is_empty() {
                                let _ = sqlx::query(
                                    "UPDATE conversations SET brand_id = $1 WHERE id = $2"
                                )
                                .bind(&brand_id)
                                .bind(&conversation_id)
                                .execute(&state.db)
                                .await;

                                let _ = process_whatsapp_automation(
                                    &state, &brand_id, &normalized_from, &conversation_id, &text, msg
                                ).await;
                            }
                        }
                    }
                    Err(inbound::InboundError::UnknownContact(_)) => unknown_contacts += 1,
                    Err(other) => errors.push(format!("{other:?}")),
                }
            }

            for status_event in
                value.get("statuses").and_then(|v| v.as_array()).into_iter().flatten()
            {
                match apply_meta_status(&state, status_event).await {
                    Ok(true) => receipts += 1,
                    Ok(false) => receipts_unmatched += 1,
                    Err(err) => errors.push(format!("status receipt: {err}")),
                }
            }
        }
    }

    if !errors.is_empty() {
        tracing::error!("WhatsApp webhook processing errors: {:?}", errors);
    }
    let body = json!({
        "status": if errors.is_empty() { "ok" } else { "incomplete" },
        "ingested": ingested,
        "duplicates": duplicates,
        "receipts": receipts,
        "receipts_unmatched": receipts_unmatched,
        "channel_syncs": syncs,
        "unknown_contacts": unknown_contacts,
        "refused_missing_sender": missing_sender,
        "errors": errors,
    });
    // WHY: an `ok` for events this server could not write is how a provider stops
    // retrying a delivery it never got stored. Re-delivery is safe here because
    // already-stored messages come back as duplicates, so a write failure returns
    // 5xx and Meta retries instead of the event being lost.
    if errors.is_empty() {
        Json(body).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Twilio (SMS + WhatsApp)
// ---------------------------------------------------------------------------

async fn twilio_inbound(
    state: &AppState,
    headers: &HeaderMap,
    uri: &OriginalUri,
    body: &[u8],
    channel: &str,
) -> Response {
    if state.config.twilio_auth_token.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Twilio webhook not configured: set TWILIO_AUTH_TOKEN.",
        );
    }
    let params = parse_form(body);
    if !validate_twilio_signature(
        &state.config.twilio_auth_token,
        &absolute_url(headers, uri),
        &params,
        header(headers, "x-twilio-signature"),
    ) {
        return rejected(StatusCode::FORBIDDEN, "Invalid Twilio signature.");
    }

    let raw_from = form_value(&params, "From");
    let from_value = raw_from.strip_prefix("whatsapp:").unwrap_or(&raw_from).to_string();
    let message_body = form_value(&params, "Body");
    let message_sid = form_value(&params, "MessageSid");

    let inbound = InboundMessage {
        channel,
        from_value: &from_value,
        body: &message_body,
        provider: "twilio",
        provider_message_id: &message_sid,
        sender_name: "",
    };

    match inbound::process_inbound_message(&state.db, &inbound).await {
        Ok((_, duplicate)) => {
            Json(json!({ "status": "ok", "duplicate": duplicate })).into_response()
        }
        Err(inbound::InboundError::UnknownContact(reason)) => {
            tracing::info!("Ignoring Twilio inbound for unknown contact: {reason}");
            Json(json!({ "status": "ok", "ignored": "unknown_contact" })).into_response()
        }
        Err(other) => {
            tracing::error!("Twilio inbound failed: {other:?}");
            rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to record inbound message.",
            )
        }
    }
}

pub async fn twilio_sms_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    twilio_inbound(&state, &headers, &uri, &body, "sms").await
}

pub async fn twilio_whatsapp_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    twilio_inbound(&state, &headers, &uri, &body, "whatsapp").await
}

// ---------------------------------------------------------------------------
// Meta Messenger / Instagram (same `entry[].messaging[]` envelope)
// ---------------------------------------------------------------------------

/// One sender event shared by the Messenger and Instagram Direct webhooks.
/// Instagram's `story_mention` and Messenger's `attachments` both arrive with
/// no text, so the caller supplies the label it wants stored for those.
async fn messenger_inbound(
    state: &AppState,
    payload: &Value,
    channel: &str,
    provider: &str,
    attachment_label: &str,
) -> IngestTally {
    let mut tally = IngestTally::default();
    let tenant = match resolve_tenant(state, channel).await {
        Ok(tenant) => tenant,
        Err(error) => {
            tally.errors.push(error);
            return tally;
        }
    };

    for entry in payload.get("entry").and_then(|v| v.as_array()).into_iter().flatten() {
        for event in entry.get("messaging").and_then(|v| v.as_array()).into_iter().flatten() {
            let sender_id = event
                .pointer("/sender/id")
                .map(value_to_string)
                .unwrap_or_default();
            if sender_id.is_empty() {
                continue;
            }

            let Some(message) = event.get("message") else { continue };
            let mid = message.get("mid").map(value_to_string).unwrap_or_default();
            let mut text = message.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let payload_id = message
                .pointer("/quick_reply/payload")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            if text.is_empty() {
                if let Some(id) = &payload_id {
                    text = id.clone();
                } else if message.get("attachments").and_then(|v| v.as_array()).is_some_and(|a| !a.is_empty())
                {
                    text = attachment_label.to_string();
                }
            }
            if text.is_empty() {
                continue;
            }

            if let Err(err) = inbound::get_or_create_contact(
                &state.db,
                channel,
                &sender_id,
                "",
                &tenant,
            )
            .await
            {
                tally.errors.push(format!("{err:?}"));
                continue;
            }

            let inbound_msg = InboundMessage {
                channel,
                from_value: &sender_id,
                body: &text,
                provider,
                provider_message_id: &mid,
                sender_name: "",
            };
            match inbound::process_inbound_message(&state.db, &inbound_msg).await {
                Ok((_, true)) => tally.duplicates += 1,
                Ok((_, false)) => tally.ingested += 1,
                Err(inbound::InboundError::UnknownContact(_)) => tally.unknown_contacts += 1,
                Err(other) => tally.errors.push(format!("{other:?}")),
            }
        }
    }
    tally
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(num) => num.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => String::new(),
    }
}

fn payload_header(payload: &Value, key: &str) -> String {
    payload.get(key).map(value_to_string).unwrap_or_default()
}

#[derive(Default)]
struct IngestTally {
    ingested: usize,
    duplicates: usize,
    unknown_contacts: usize,
    receipts: usize,
    channel_syncs: usize,
    errors: Vec<String>,
}

fn tally_response(tally: IngestTally) -> Response {
    if !tally.errors.is_empty() {
        tracing::error!("Webhook processing errors: {:?}", tally.errors);
    }
    let body = json!({
        "status": if tally.errors.is_empty() { "ok" } else { "incomplete" },
        "ingested": tally.ingested,
        "duplicates": tally.duplicates,
        "unknown_contacts": tally.unknown_contacts,
        "receipts": tally.receipts,
        "channel_syncs": tally.channel_syncs,
        "errors": tally.errors,
    });
    // WHY: same as the WhatsApp handler -- events that could not be written must
    // not be acknowledged as `ok`, or the provider drops them for good.
    if tally.errors.is_empty() {
        Json(body).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

/// Rejects unsigned traffic whenever a secret is on file, and refuses the
/// endpoint outright when none is -- an unauthenticated webhook lets anyone who
/// learns the URL forge inbound messages, fake STOP requests and fake receipts.
async fn messenger_verify(
    state: &AppState,
    params: HubVerifyParams,
    channel: &str,
) -> Response {
    let creds = inbox_credentials(state, channel).await;
    let token = credential(&creds, "webhookVerifyToken");
    let hint = format!(
        "Webhook not configured: save webhookVerifyToken on the {channel} inbox (or set META_WEBHOOK_VERIFY_TOKEN)."
    );
    let expected = if token.is_empty() {
        state.config.meta_webhook_verify_token.clone()
    } else {
        token
    };
    meta_handshake(params, expected, &hint)
}

async fn messenger_post(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
    channel: &str,
    provider: &str,
    allowed_objects: &[&str],
    attachment_label: &str,
    config_secret: &str,
) -> Response {
    let creds = inbox_credentials(state, channel).await;
    let mut secret = credential(&creds, "appSecret");
    if secret.is_empty() {
        secret = config_secret.to_string();
    }
    if secret.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "No app secret on file for the {channel} inbox: inbound cannot be authenticated."
            ),
        );
    }
    if !validate_meta_signature(&secret, body, header(headers, "x-hub-signature-256")) {
        return rejected(
            StatusCode::FORBIDDEN,
            format!("Invalid Meta {channel} webhook signature."),
        );
    }

    let Ok(payload) = serde_json::from_slice::<Value>(body) else {
        return rejected(StatusCode::BAD_REQUEST, "Unreadable webhook payload.");
    };
    let object = payload.get("object").and_then(|v| v.as_str()).unwrap_or("");
    if !allowed_objects.contains(&object) {
        return Json(json!({ "status": "ignored" })).into_response();
    }

    tally_response(messenger_inbound(state, &payload, channel, provider, attachment_label).await)
}

pub async fn meta_facebook_verify(
    State(state): State<AppState>,
    Query(params): Query<HubVerifyParams>,
) -> Response {
    messenger_verify(&state, params, "facebook").await
}

pub async fn meta_facebook_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    messenger_post(
        &state,
        &headers,
        &body,
        "facebook",
        "facebook",
        &["page", "facebook"],
        "[Attachment received]",
        &state.config.meta_app_secret,
    )
    .await
}

pub async fn meta_instagram_verify(
    State(state): State<AppState>,
    Query(params): Query<HubVerifyParams>,
) -> Response {
    messenger_verify(&state, params, "instagram").await
}

pub async fn meta_instagram_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    messenger_post(
        &state,
        &headers,
        &body,
        "instagram",
        "instagram",
        &["instagram", "page"],
        "[Attachment or Story mention received]",
        &state.config.meta_instagram_app_secret,
    )
    .await
}

// ---------------------------------------------------------------------------
// Telegram
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SecretTokenParams {
    secret: Option<String>,
}

pub async fn telegram_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<SecretTokenParams>,
    body: Bytes,
) -> Response {
    let creds = inbox_credentials(&state, "telegram").await;
    let expected = credential(&creds, "secretToken");
    if expected.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Telegram webhook not configured: save secretToken on the Telegram inbox.",
        );
    }
    // Telegram authenticates with the X-Telegram-Bot-Api-Secret-Token header,
    // and also supports a `?secret=` query parameter for setups behind a proxy.
    let provided = header(&headers, "x-telegram-bot-api-secret-token").or(params.secret.as_deref());
    if !secrets_match(&expected, provided) {
        return rejected(StatusCode::FORBIDDEN, "Invalid Telegram secret token.");
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return rejected(StatusCode::BAD_REQUEST, "Unreadable webhook payload.");
    };

    // A plain message and an inline-button callback carry the sender in
    // different places, so normalise both into one shape before ingesting.
    #[derive(Default)]
    struct TelegramEvent {
        chat_id: String,
        sender_name: String,
        text: String,
        provider_message_id: String,
    }

    let mut event = TelegramEvent::default();
    if let Some(msg) = payload.get("message") {
        event.chat_id = msg.pointer("/chat/id").map(value_to_string).unwrap_or_default();
        event.sender_name = first_name_of(msg.get("from"));
        event.text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
        event.provider_message_id = format!(
            "tg_{}",
            msg.get("message_id").map(value_to_string).unwrap_or_default()
        );
    } else if let Some(cb) = payload.get("callback_query") {
        event.chat_id = cb.pointer("/message/chat/id").map(value_to_string).unwrap_or_default();
        event.sender_name = first_name_of(cb.get("from"));
        event.text = cb.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string();
        event.provider_message_id = format!(
            "tg_cb_{}",
            cb.get("id").map(value_to_string).unwrap_or_default()
        );
    }

    if event.chat_id.is_empty() || event.text.is_empty() {
        return Json(json!({ "status": "ignored" })).into_response();
    }

    let tenant = match resolve_tenant(&state, "telegram").await {
        Ok(tenant) => tenant,
        Err(error) => return rejected(StatusCode::SERVICE_UNAVAILABLE, error),
    };
    if let Err(err) = inbound::get_or_create_contact(
        &state.db,
        "telegram",
        &event.chat_id,
        &event.sender_name,
        &tenant,
    )
    .await
    {
        tracing::warn!("Could not provision Telegram contact {}: {err:?}", event.chat_id);
    }

    let inbound_msg = InboundMessage {
        channel: "telegram",
        from_value: &event.chat_id,
        body: &event.text,
        provider: "telegram",
        provider_message_id: &event.provider_message_id,
        sender_name: &event.sender_name,
    };
    let mut tally = IngestTally::default();
    match inbound::process_inbound_message(&state.db, &inbound_msg).await {
        Ok((_, true)) => tally.duplicates += 1,
        Ok((_, false)) => tally.ingested += 1,
        Err(inbound::InboundError::UnknownContact(_)) => tally.unknown_contacts += 1,
        Err(other) => tally.errors.push(format!("{other:?}")),
    }
    tally_response(tally)
}

fn first_name_of(from: Option<&Value>) -> String {
    from.and_then(|v| {
        [v.get("first_name").and_then(|x| x.as_str()), v.get("username").and_then(|x| x.as_str())]
            .into_iter()
            .flatten()
            .find(|candidate| !candidate.is_empty())
            .map(str::to_string)
    })
    .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// LINE
// ---------------------------------------------------------------------------

pub async fn line_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let creds = inbox_credentials(&state, "line").await;
    let secret = credential(&creds, "channelSecret");
    if secret.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "LINE webhook not configured: save channelSecret on the LINE inbox.",
        );
    }
    if !validate_line_signature(&secret, &body, header(&headers, "x-line-signature")) {
        return rejected(StatusCode::FORBIDDEN, "Invalid LINE webhook signature.");
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return rejected(StatusCode::BAD_REQUEST, "Unreadable webhook payload.");
    };

    let tenant = match resolve_tenant(&state, "line").await {
        Ok(tenant) => tenant,
        Err(error) => return rejected(StatusCode::SERVICE_UNAVAILABLE, error),
    };

    let mut tally = IngestTally::default();
    for event in payload.get("events").and_then(|v| v.as_array()).into_iter().flatten() {
        let user_id = event.pointer("/source/userId").map(value_to_string).unwrap_or_default();
        if user_id.is_empty() {
            continue;
        }

        let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let mut text = String::new();
        let mut provider_message_id = String::new();

        if event_type == "message" {
            let msg = event.get("message").cloned().unwrap_or(Value::Null);
            provider_message_id = msg.get("id").map(value_to_string).unwrap_or_default();
            text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if text.is_empty() {
                let kind = msg.get("type").and_then(|v| v.as_str()).unwrap_or("attachment");
                text = format!("{} received", capitalize(kind));
            }
        } else if event_type == "postback" {
            text = event
                .pointer("/postback/data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let stamp = event.get("timestamp").map(value_to_string).unwrap_or_default();
            provider_message_id = format!("line_pb_{stamp}");
        }

        if text.is_empty() {
            continue;
        }

        if provider_message_id.is_empty() {
            let prefix: String = user_id.chars().take(8).collect();
            provider_message_id = format!("line_msg_{prefix}");
        }

        if let Err(err) =
            inbound::get_or_create_contact(&state.db, "line", &user_id, "", &tenant).await
        {
            tracing::warn!("Could not provision LINE contact {user_id}: {err:?}");
        }

        let inbound_msg = InboundMessage {
            channel: "line",
            from_value: &user_id,
            body: &text,
            provider: "line",
            provider_message_id: &provider_message_id,
            sender_name: "",
        };
        match inbound::process_inbound_message(&state.db, &inbound_msg).await {
            Ok((_, true)) => tally.duplicates += 1,
            Ok((_, false)) => tally.ingested += 1,
            Err(inbound::InboundError::UnknownContact(_)) => tally.unknown_contacts += 1,
            Err(other) => tally.errors.push(format!("{other:?}")),
        }
    }
    tally_response(tally)
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Postmark inbound email
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct TokenParams {
    token: Option<String>,
}

pub async fn postmark_inbound(
    State(state): State<AppState>,
    Query(params): Query<TokenParams>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if state.config.postmark_inbound_webhook_secret.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Postmark inbound webhook not configured: set POSTMARK_INBOUND_WEBHOOK_SECRET.",
        );
    }
    // Postmark does not sign requests; the shared secret rides in the callback
    // URL query string. Postmark also sends an optional header we accept.
    let provided = params
        .token
        .as_deref()
        .or(header(&headers, "x-postmark-token"));
    if !secrets_match(&state.config.postmark_inbound_webhook_secret, provided) {
        return rejected(StatusCode::FORBIDDEN, "Invalid or missing webhook token.");
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return rejected(StatusCode::BAD_REQUEST, "Unreadable webhook payload.");
    };

    let from_value = payload
        .pointer("/FromFull/Email")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("From").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
        .to_string();
    let message_id = payload.get("MessageID").map(value_to_string).unwrap_or_default();
    let text = payload.get("TextBody").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let inbound_msg = InboundMessage {
        channel: "email",
        from_value: &from_value,
        body: &text,
        provider: "postmark",
        provider_message_id: &message_id,
        sender_name: "",
    };
    let mut tally = IngestTally::default();
    match inbound::process_inbound_message(&state.db, &inbound_msg).await {
        Ok((_, true)) => tally.duplicates += 1,
        Ok((_, false)) => tally.ingested += 1,
        Err(inbound::InboundError::UnknownContact(_)) => tally.unknown_contacts += 1,
        Err(other) => tally.errors.push(format!("{other:?}")),
    }
    tally_response(tally)
}

// ---------------------------------------------------------------------------
// TikTok Direct Messages and video comments
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct TiktokVerifyParams {
    challenge: Option<String>,
    echo: Option<String>,
    #[serde(rename = "hub.mode")]
    hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    hub_challenge: Option<String>,
}

pub async fn tiktok_webhook_verify(
    State(state): State<AppState>,
    Query(params): Query<TiktokVerifyParams>,
) -> Response {
    let creds = inbox_credentials(&state, "tiktok").await;
    let expected = {
        let from_inbox = credential(&creds, "webhookVerifyToken");
        if from_inbox.is_empty() {
            state.config.tiktok_webhook_verify_token.clone()
        } else {
            from_inbox
        }
    };
    let challenge = params
        .hub_challenge
        .or(params.challenge)
        .or(params.echo)
        .unwrap_or_default();
    let handshake = HubVerifyParams {
        hub_mode: params.hub_mode,
        hub_verify_token: params.hub_verify_token,
        hub_challenge: Some(challenge),
    };
    meta_handshake(
        handshake,
        expected,
        "TikTok webhook not configured: save webhookVerifyToken on the TikTok inbox or set TIKTOK_WEBHOOK_VERIFY_TOKEN.",
    )
}

pub async fn tiktok_webhook_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let creds = inbox_credentials(&state, "tiktok").await;
    let secret = {
        let from_inbox = credential(&creds, "appSecret");
        if from_inbox.is_empty() {
            state.config.tiktok_app_secret.clone()
        } else {
            from_inbox
        }
    };
    if secret.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "TikTok webhook not configured: save appSecret on the TikTok inbox or set TIKTOK_APP_SECRET.",
        );
    }
    if !validate_tiktok_signature(&secret, &body, header(&headers, "x-tiktok-signature")) {
        return rejected(StatusCode::FORBIDDEN, "Invalid TikTok webhook signature.");
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return rejected(StatusCode::BAD_REQUEST, "Unreadable webhook payload.");
    };

    let content = payload
        .get("content")
        .or(payload.get("data"))
        .cloned()
        .unwrap_or(Value::Null);
    let user = payload.get("user").cloned().unwrap_or(Value::Null);

    let sender_open_id = [
        content.get("open_id"),
        content.get("from_user_id"),
        user.get("open_id"),
        payload.get("open_id"),
        payload.get("sender_id"),
    ]
    .into_iter()
    .flatten()
    .map(value_to_string)
    .find(|value| !value.is_empty())
    .unwrap_or_default();

    if sender_open_id.is_empty() {
        return Json(json!({ "status": "ignored", "reason": "No sender open_id provided" }))
            .into_response();
    }

    let sender_name = [
        user.get("nickname"),
        user.get("display_name"),
        content.get("sender_name"),
    ]
    .into_iter()
    .flatten()
    .map(value_to_string)
    .find(|value| !value.is_empty())
    .unwrap_or_default();

    let text = [content.get("text"), content.get("body"), payload.get("text"), payload.get("message")]
        .into_iter()
        .flatten()
        .map(value_to_string)
        .find(|value| !value.is_empty())
        .unwrap_or_default();

    let event_type = [payload.get("event"), payload.get("event_type")]
        .into_iter()
        .flatten()
        .map(value_to_string)
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| "im.direct_message.receive".to_string());

    let message_id = [
        content.get("message_id"),
        content.get("comment_id"),
        payload.get("message_id"),
    ]
    .into_iter()
    .flatten()
    .map(value_to_string)
    .find(|value| !value.is_empty())
    .unwrap_or_default();

    let is_comment = event_type.to_ascii_lowercase().contains("comment");
    let body_text = if is_comment {
        let video_id = [content.get("video_id"), payload.get("video_id")]
            .into_iter()
            .flatten()
            .map(value_to_string)
            .find(|value| !value.is_empty())
            .unwrap_or_default();
        if text.is_empty() {
            format!("[TikTok Video Comment #{video_id}]")
        } else {
            format!("[TikTok Video Comment #{video_id}]: {text}")
        }
    } else if text.is_empty() {
        "[TikTok Direct Message]".to_string()
    } else {
        text.clone()
    };

    let tenant = match resolve_tenant(&state, "tiktok").await {
        Ok(tenant) => tenant,
        Err(error) => return rejected(StatusCode::SERVICE_UNAVAILABLE, error),
    };
    if let Err(err) = inbound::get_or_create_contact(
        &state.db,
        "tiktok",
        &sender_open_id,
        &sender_name,
        &tenant,
    )
    .await
    {
        tracing::warn!("Could not provision TikTok contact {sender_open_id}: {err:?}");
    }

    let mut tally = IngestTally::default();
    // TikTok gives no id for some comment events; without one we cannot
    // deduplicate, so say so instead of inventing a timestamp-based id that
    // would look like a provider id.
    let inbound_msg = InboundMessage {
        channel: "tiktok",
        from_value: &sender_open_id,
        body: &body_text,
        provider: "tiktok",
        provider_message_id: &message_id,
        sender_name: &sender_name,
    };
    match inbound::process_inbound_message(&state.db, &inbound_msg).await {
        Ok((_, true)) => tally.duplicates += 1,
        Ok((_, false)) => tally.ingested += 1,
        Err(inbound::InboundError::UnknownContact(_)) => tally.unknown_contacts += 1,
        Err(other) => tally.errors.push(format!("{other:?}")),
    }

    Json(json!({
        "status": "ok",
        "event": event_type,
        "messageId": message_id,
        "openId": sender_open_id,
        "isComment": is_comment,
        "ingested": tally.ingested,
        "duplicates": tally.duplicates,
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Twilio Voice (inbound IVR, status, recording, transcription)
// ---------------------------------------------------------------------------

/// Every Twilio voice callback is a signed form POST. Accepting JSON instead
/// would skip signature verification, so JSON is refused outright -- Twilio
/// never sends it.
async fn twilio_signed_form(
    state: &AppState,
    headers: &HeaderMap,
    uri: &OriginalUri,
    body: &[u8],
) -> Result<Vec<(String, String)>, Response> {
    if !header(headers, "content-type").is_some_and(|ct| ct.contains("application/x-www-form-urlencoded"))
    {
        return Err(rejected(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Twilio voice callbacks must be form-urlencoded so the signature can be verified.",
        ));
    }
    if state.config.twilio_auth_token.is_empty() {
        return Err(rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Twilio webhook not configured: set TWILIO_AUTH_TOKEN.",
        ));
    }
    let params = parse_form(body);
    if !validate_twilio_signature(
        &state.config.twilio_auth_token,
        &absolute_url(headers, uri),
        &params,
        header(headers, "x-twilio-signature"),
    ) {
        return Err(rejected(StatusCode::FORBIDDEN, "Invalid Twilio signature."));
    }
    Ok(params)
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn twiml(body: String) -> Response {
    (
        StatusCode::OK,
        [("content-type", "application/xml; charset=utf-8")],
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Response>\n{body}</Response>"),
    )
        .into_response()
}

pub async fn voice_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    let params = match twilio_signed_form(&state, &headers, &uri, &body).await {
        Ok(params) => params,
        Err(response) => return response,
    };

    let call_sid = form_value(&params, "CallSid");
    let from_number = form_value(&params, "From");
    let to_number = form_value(&params, "To");
    let digits = form_value(&params, "Digits");
    let call_status = form_value(&params, "CallStatus");

    if call_sid.is_empty() || from_number.is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Twilio voice callback is missing CallSid or From.",
        );
    }

    let creds = inbox_credentials(&state, "voice").await;
    if creds.is_null() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "No voice inbox is configured, so inbound calls cannot be routed.",
        );
    }
    let tenant = match resolve_tenant(&state, "voice").await {
        Ok(tenant) => tenant,
        Err(error) => return rejected(StatusCode::SERVICE_UNAVAILABLE, error),
    };

    let ivr_enabled = creds
        .get("ivrEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let ivr_prompt = credential(&creds, "ivrPrompt");
    let agent_queue = credential(&creds, "agentQueue");
    let sales_queue = credential(&creds, "salesQueue");
    let support_queue = credential(&creds, "supportQueue");
    let voicemail_enabled = creds
        .get("voicemailEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let dial_timeout = creds
        .get("dialTimeout")
        .and_then(|v| v.as_i64())
        .unwrap_or(30);
    let voicemail_max_seconds = creds
        .get("voicemailMaxSeconds")
        .and_then(|v| v.as_i64())
        .unwrap_or(120);

    let status_url = "/api/v1/webhooks/voice/status";
    let contact_id = match inbound::get_or_create_contact(
        &state.db,
        "voice",
        &from_number,
        "",
        &tenant,
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Could not provision voice caller {from_number}: {err:?}");
            return rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to record the caller.",
            );
        }
    };
    let inbox_id = inbox_id_for(&state, "voice").await;
    let conversation_id = match inbound::get_or_create_inbox_conversation(
        &state.db,
        &contact_id,
        "voice",
        inbox_id.as_deref(),
        None,
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Could not open a voice conversation: {err:?}");
            return rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open a conversation for the call.",
            );
        }
    };

    // Deciding what to do with the call happens first, so the stored call
    // record describes the routing that was actually returned to Twilio.
    enum Route {
        Dial(String, String),
        Voicemail(String),
        Reject(String),
    }

    let route = if !ivr_enabled {
        if agent_queue.is_empty() {
            Route::Reject("no agent queue configured for this voice inbox".to_string())
        } else {
            Route::Dial(agent_queue.clone(), "Direct Dial · Connecting to agent inbox".to_string())
        }
    } else if !digits.is_empty() {
        let (queue, label) = match digits.as_str() {
            "1" => (sales_queue.clone(), "Selected option [1: Sales]".to_string()),
            "2" => (support_queue.clone(), "Selected option [2: Support]".to_string()),
            "3" if voicemail_enabled => (String::new(), "Selected option [3: Voicemail]".to_string()),
            "3" => (String::new(), "Voicemail is disabled for this inbox".to_string()),
            other => (
                agent_queue.clone(),
                format!("Unrecognised digit [{other}] · Routing to the agent queue"),
            ),
        };
        if digits == "3" {
            if voicemail_enabled {
                Route::Voicemail(label)
            } else {
                Route::Reject(label)
            }
        } else if queue.is_empty() {
            Route::Reject(format!("{label} · no queue configured"))
        } else {
            Route::Dial(queue, label)
        }
    } else if ivr_prompt.is_empty() {
        Route::Reject("IVR is enabled but no ivrPrompt is configured".to_string())
    } else {
        Route::Dial(
            agent_queue.clone(),
            "Ringing · IVR menu presented".to_string(),
        )
    };

    let twiml_body = match &route {
        Route::Reject(reason) => {
            tracing::warn!("Rejecting inbound call {call_sid}: {reason}");
            format!("  <Say voice=\"client:alice\">We cannot route this call right now.</Say>\n  <Hangup/>\n")
        }
        Route::Voicemail(_) => format!(
            "  <Say voice=\"client:alice\">Please leave your message after the tone. Press pound when finished.</Say>\n  \
             <Record maxLength=\"{voicemail_max_seconds}\" action=\"/api/v1/webhooks/voice/recording\" transcribe=\"true\" \
             transcribeCallback=\"/api/v1/webhooks/voice/transcription\"/>\n"
        ),
        Route::Dial(queue, _) => {
            let gather = if ivr_enabled && digits.is_empty() && !ivr_prompt.is_empty() {
                format!(
                    "  <Gather action=\"/api/v1/webhooks/voice/inbound\" numDigits=\"1\" timeout=\"10\">\n    \
                     <Say voice=\"client:alice\">{}</Say>\n  </Gather>\n",
                    xml_escape(&ivr_prompt)
                )
            } else {
                String::new()
            };
            format!(
                "{gather}  <Dial timeout=\"{dial_timeout}\" record=\"record-from-answer-dual\" action=\"{status_url}\">\n    \
                 <Client>{}</Client>\n  </Dial>\n",
                xml_escape(queue)
            )
        }
    };

    let action_note = match &route {
        Route::Dial(_, label) => label.clone(),
        Route::Voicemail(label) => format!("{label} · Recording voicemail"),
        Route::Reject(reason) => format!("Rejected · {reason}"),
    };

    let metadata = json!({
        "call_sid": call_sid,
        "from": from_number,
        "to": to_number,
        "digits": digits,
        "action_note": action_note,
        "call_status": if call_status.is_empty() { Value::Null } else { json!(call_status) },
        "direction": "inbound",
    });
    let body_text = format!("Inbound Call from {from_number} ({action_note})");
    if let Err(err) = inbound::record_raw_inbound_message(
        &state.db,
        &conversation_id,
        "voice",
        &body_text,
        "twilio_voice",
        &call_sid,
        "",
        Some(metadata),
        None,
    )
    .await
    {
        tracing::error!("Could not store voice call record {call_sid}: {err:?}");
    }

    twiml(twiml_body)
}

async fn inbox_id_for(state: &AppState, channel: &str) -> Option<String> {
    sqlx::query("SELECT id FROM inboxes WHERE channel_type = $1 AND is_active = 1 LIMIT 1")
        .bind(channel)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<String, _>("id").ok())
}

pub async fn voice_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    let params = match twilio_signed_form(&state, &headers, &uri, &body).await {
        Ok(params) => params,
        Err(response) => return response,
    };
    let call_sid = form_value(&params, "CallSid");
    let call_status = form_value(&params, "CallStatus");
    let duration = form_value(&params, "CallDuration").parse::<i64>().unwrap_or(0);
    if call_sid.is_empty() {
        return rejected(StatusCode::BAD_REQUEST, "Missing CallSid.");
    }

    let additions = json!({
        "call_status": call_status,
        "duration": duration,
        "ended_at": chrono::Utc::now().to_rfc3339(),
    });
    let suffix = if call_status == "completed" {
        format!(" · Call completed ({duration}s)")
    } else {
        String::new()
    };
    match inbound::amend_message_metadata(&state.db, &call_sid, additions, &suffix).await {
        Ok(true) => {}
        Ok(false) => tracing::info!("Voice status for unknown call {call_sid} ignored"),
        Err(err) => {
            tracing::error!("Voice status update failed: {err:?}");
            return rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update the call record.",
            );
        }
    }
    Json(json!({ "status": "ok", "callSid": call_sid, "callStatus": call_status, "duration": duration }))
        .into_response()
}

pub async fn voice_recording(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    let params = match twilio_signed_form(&state, &headers, &uri, &body).await {
        Ok(params) => params,
        Err(response) => return response,
    };
    let call_sid = form_value(&params, "CallSid");
    let recording_url = form_value(&params, "RecordingUrl");
    let recording_sid = form_value(&params, "RecordingSid");
    let duration = form_value(&params, "RecordingDuration").parse::<i64>().unwrap_or(0);
    if call_sid.is_empty() {
        return rejected(StatusCode::BAD_REQUEST, "Missing CallSid.");
    }

    let additions = json!({
        "recording_url": recording_url,
        "recording_sid": recording_sid,
        "recording_duration": duration,
    });
    let suffix = format!(" · Recording ({duration}s)");
    if let Err(err) =
        inbound::amend_message_metadata(&state.db, &call_sid, additions, &suffix).await
    {
        tracing::error!("Voice recording update failed: {err:?}");
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to attach the recording.",
        );
    }
    Json(json!({
        "status": "ok",
        "callSid": call_sid,
        "recordingUrl": recording_url,
        "recordingDuration": duration,
    }))
    .into_response()
}

pub async fn voice_transcription(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    let params = match twilio_signed_form(&state, &headers, &uri, &body).await {
        Ok(params) => params,
        Err(response) => return response,
    };
    let call_sid = form_value(&params, "CallSid");
    let transcript = form_value(&params, "TranscriptionText");
    let status_text = form_value(&params, "TranscriptionStatus");
    if call_sid.is_empty() {
        return rejected(StatusCode::BAD_REQUEST, "Missing CallSid.");
    }

    let additions = json!({
        "transcription_text": transcript,
        "transcription_status": status_text,
    });
    let suffix = if transcript.is_empty() {
        String::new()
    } else {
        format!("\nTranscript: \"{transcript}\"")
    };
    if let Err(err) =
        inbound::amend_message_metadata(&state.db, &call_sid, additions, &suffix).await
    {
        tracing::error!("Voice transcription update failed: {err:?}");
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to attach the transcript.",
        );
    }
    Json(json!({ "status": "ok", "callSid": call_sid, "transcriptionStatus": status_text }))
        .into_response()
}

// ---------------------------------------------------------------------------
// Inbound email (SendGrid Inbound Parse / Cloudflare Mail / custom MTA)
// ---------------------------------------------------------------------------

/// Quoted reply text, signature rules and "On ... wrote:" headers are not the
/// customer's new message. These are the delimiter shapes real mail clients
/// emit; anything they match is dropped from the stored body.
fn email_quote_patterns() -> &'static [regex::Regex] {
    static PATTERNS: std::sync::OnceLock<Vec<regex::Regex>> = std::sync::OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            r"(?is)\n\s*On\s+.+?\s+wrote:\s*\n.*",
            r"(?is)\n\s*-{3}\s*Original Message\s*-{3}\s*\n.*",
            r"(?is)\n\s*From:\s+.+?\nSent:\s+.+?\n.*",
            r"(?is)\n\s*_{10,}\s*\n.*",
        ]
        .iter()
        .filter_map(|pattern| regex::Regex::new(pattern).ok())
        .collect()
    })
}

fn clean_email_body(body: &str) -> String {
    let mut cleaned = body.to_string();
    for pattern in email_quote_patterns() {
        cleaned = pattern.replace_all(&cleaned, "").into_owned();
    }
    cleaned.trim().to_string()
}

/// "Alice Example <Alice@Acme.com>" -> ("Alice Example", "alice@acme.com").
fn split_display_name(raw: &str) -> (String, String) {
    let raw = raw.trim();
    if let (Some(open), Some(close)) = (raw.find('<'), raw.find('>')) {
        if open < close {
            let name = raw[..open].trim().trim_matches(|c| c == '"' || c == '\'').trim();
            let address = raw[open + 1..close].trim().to_ascii_lowercase();
            return (name.to_string(), address);
        }
    }
    (String::new(), raw.to_ascii_lowercase())
}

pub async fn email_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<TokenParams>,
    body: Bytes,
) -> Response {
    if state.config.email_inbound_webhook_secret.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Inbound email webhook not configured: set EMAIL_INBOUND_WEBHOOK_SECRET.",
        );
    }
    let provided = header(&headers, "x-webhook-token")
        .or(header(&headers, "authorization"))
        .or(params.token.as_deref());
    if !secrets_match(&state.config.email_inbound_webhook_secret, provided) {
        return rejected(StatusCode::FORBIDDEN, "Invalid or missing webhook token.");
    }

    let is_json = header(&headers, "content-type").is_some_and(|ct| ct.contains("application/json"));
    let payload: Value = if is_json {
        match serde_json::from_slice::<Value>(&body) {
            Ok(value) => value,
            Err(_) => return rejected(StatusCode::BAD_REQUEST, "Unreadable JSON payload."),
        }
    } else {
        let mut map = serde_json::Map::new();
        for (key, value) in parse_form(&body) {
            map.insert(key, json!(value));
        }
        Value::Object(map)
    };

    let first = |keys: &[&str]| -> String {
        keys.iter()
            .filter_map(|key| payload.get(*key))
            .map(value_to_string)
            .find(|value| !value.is_empty())
            .unwrap_or_default()
    };

    let raw_from = first(&["From", "from", "sender"]);
    let mut from_name = first(&["FromName", "from_name"]);
    let (parsed_name, sender_email) = split_display_name(&raw_from);
    if from_name.is_empty() {
        from_name = parsed_name;
    }
    if sender_email.is_empty() || !sender_email.contains('@') {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Inbound email has no parsable sender address.",
        );
    }

    let subject = first(&["Subject", "subject"]);
    let text_body = first(&["TextBody", "text", "body"]);
    let html_body = first(&["HtmlBody", "html"]);
    let message_id = {
        let id = first(&["MessageID", "message_id"]);
        if id.is_empty() {
            payload
                .pointer("/headers/Message-ID")
                .map(value_to_string)
                .unwrap_or_default()
        } else {
            id
        }
    };
    if message_id.is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Inbound email carries no Message-ID, so it cannot be deduplicated.",
        );
    }

    let clean_body = clean_email_body(&text_body);
    let tenant = match resolve_tenant(&state, "email").await {
        Ok(tenant) => tenant,
        Err(error) => return rejected(StatusCode::SERVICE_UNAVAILABLE, error),
    };

    let contact_id = match inbound::get_or_create_contact(
        &state.db,
        "email",
        &sender_email,
        &from_name,
        &tenant,
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Could not provision email contact {sender_email}: {err:?}");
            return rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to record the sender.",
            );
        }
    };

    let email_inbox = sqlx::query("SELECT id, credentials, phone_number FROM inboxes WHERE channel_type = 'email' AND is_active = 1 LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
    let inbox_id: Option<String> = email_inbox
        .as_ref()
        .and_then(|row| row.try_get::<String, _>("id").ok());
    // The address the message was delivered to is either in the payload or the
    // mailbox we provisioned -- never a placeholder domain.
    let delivered_to = {
        let from_payload = first(&["To", "to"]);
        if !from_payload.is_empty() {
            from_payload
        } else {
            email_inbox
                .as_ref()
                .and_then(|row| {
                    row.try_get::<Option<String>, _>("credentials")
                        .ok()
                        .flatten()
                        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
                        .and_then(|creds| {
                            creds
                                .get("inboundAddress")
                                .and_then(|v| v.as_str())
                                .map(str::to_string)
                        })
                })
                .unwrap_or_default()
        }
    };
    let attachments = payload
        .get("Attachments")
        .or_else(|| payload.get("attachments"))
        .and_then(|v| v.as_array())
        .cloned();

    let conversation_id = match inbound::get_or_create_inbox_conversation(
        &state.db,
        &contact_id,
        "email",
        inbox_id.as_deref(),
        Some(&subject),
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Could not open an email conversation: {err:?}");
            return rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open a thread for the email.",
            );
        }
    };

    let stored_body = if subject.is_empty() {
        clean_body.clone()
    } else {
        format!("Subject: {subject}\n\n{clean_body}")
    };
    let metadata = json!({
        "message_id": message_id,
        "subject": subject,
        "from": sender_email,
        "from_name": from_name,
        "to": delivered_to,
        "in_reply_to": first(&["InReplyTo", "in_reply_to"]),
        "references": first(&["References", "references"]),
        "has_html": !html_body.is_empty(),
    });

    let recorded = inbound::record_raw_inbound_message(
        &state.db,
        &conversation_id,
        "email",
        &stored_body,
        "email",
        &message_id,
        &from_name,
        Some(metadata),
        attachments.map(Value::Array),
    )
    .await;

    if let Err(err) = recorded {
        tracing::error!("Could not store inbound email {message_id}: {err:?}");
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to record the message.",
        );
    }

    Json(json!({
        "status": "ok",
        "messageId": message_id,
        "conversationId": conversation_id,
        "from": sender_email,
        "subject": subject,
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Ecosystem domain events (core-newax event bus / erp-1)
// ---------------------------------------------------------------------------

pub async fn ecosystem_event_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if state.config.ecosystem_gateway_key.is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "Ecosystem events not configured: set ECOSYSTEM_GATEWAY_KEY.",
        );
    }
    // The relay either signs each delivery (hex HMAC-SHA256 over the raw body)
    // or presents the shared key directly; both are compared in constant time.
    let provided_signature = header(&headers, "x-gateway-signature");
    if let Some(signature) = provided_signature {
        if !validate_tiktok_signature(&state.config.ecosystem_gateway_key, &body, Some(signature)) {
            return rejected(StatusCode::FORBIDDEN, "Invalid ecosystem event signature.");
        }
    } else {
        let provided = header(&headers, "x-webhook-token");
        if !secrets_match(&state.config.ecosystem_gateway_key, provided) {
            return rejected(StatusCode::FORBIDDEN, "Invalid or missing gateway key.");
        }
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return rejected(StatusCode::BAD_REQUEST, "Malformed JSON payload.");
    };
    let event_name = payload.get("name").map(value_to_string).unwrap_or_default();
    if event_name.is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Missing required 'name' in event payload.",
        );
    }

    let source = [header(&headers, "x-newax-source").map(str::to_string), Some(payload_header(&payload, "source"))]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| "ecosystem".to_string());
    let tenant_id = [header(&headers, "x-tenant-id").map(str::to_string), Some(payload_header(&payload, "tenantId"))]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    let event_data = match payload.get("payload") {
        Some(value) if value.is_object() => value.clone(),
        _ => payload.clone(),
    };

    let outcome = dispatch_ecosystem_event(&state, &event_name, &event_data, &tenant_id).await;

    let now = chrono::Utc::now().to_rfc3339();
    let event_row_id = uuid::Uuid::new_v4().simple().to_string();
    let stored = sqlx::query(
        "INSERT INTO ecosystem_events (id, event_name, source, tenant_id, payload, outcome, received_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&event_row_id)
    .bind(&event_name)
    .bind(&source)
    .bind(&tenant_id)
    .bind(event_data.to_string())
    .bind(&outcome)
    .bind(&now)
    .execute(&state.db)
    .await;

    // WHY: the dispatch above already ran, so a 5xx would make the relay retry and
    // apply the same side effects twice while filing the event twice. The gateway
    // is therefore told the truth instead: the event was handled but is NOT on
    // record, with the row id logged at error level.
    let store_note = match &stored {
        Ok(report) if report.rows_affected() == 1 => None,
        Ok(report) => {
            tracing::error!(
                "Ecosystem event '{event_name}' row {event_row_id} affected {} rows (expected 1) -- \
                 the event was dispatched but is not on record",
                report.rows_affected()
            );
            Some(format!(
                "the event was dispatched but not persisted ({} rows affected)",
                report.rows_affected()
            ))
        }
        Err(err) => {
            tracing::error!(
                "Ecosystem event '{event_name}' could not be stored as row {event_row_id}: {err} -- \
                 the event was dispatched but is not on record"
            );
            Some(format!("the event was dispatched but not persisted: {err}"))
        }
    };

    // WHY: an event filed without a tenant cannot be attributed later, and no
    // tenant-scoped write could be applied to a tenant the relay did not name.
    let tenant_note = if tenant_id.is_empty() {
        tracing::warn!(
            "Ecosystem event '{event_name}' (row {event_row_id}) arrived with no tenant id; it is \
             filed without an owner"
        );
        Some("no tenant id arrived, so the event is filed without an owner".to_string())
    } else {
        None
    };

    Json(json!({
        "success": store_note.is_none(),
        "status": if store_note.is_none() { "processed" } else { "processed_unstored" },
        "stored": store_note.is_none(),
        "event": event_name,
        "source": source,
        "tenantId": tenant_id,
        "eventRowId": event_row_id,
        "action": outcome,
        "error": store_note,
        "warning": tenant_note,
    }))
    .into_response()
}

/// Applies the workflows this backend can genuinely complete today. Anything
/// whose downstream engine has not been ported is recorded as deferred instead
/// of reported as done.
pub(crate) async fn dispatch_ecosystem_event(
    state: &AppState,
    event_name: &str,
    data: &Value,
    tenant_id: &str,
) -> String {
    match event_name {
        "refund.decision" | "return.approved" => {
            let session_id = [payload_header(data, "sessionId"), payload_header(data, "session_id")]
                .into_iter()
                .find(|value| !value.is_empty())
                .unwrap_or_default();
            if session_id.is_empty() {
                return "refund_decision_missing_session_id".to_string();
            }
            let decision = {
                let raw = payload_header(data, "decision");
                if raw.is_empty() {
                    "APPROVED".to_string()
                } else {
                    raw.to_ascii_uppercase()
                }
            };
            let status = match decision.as_str() {
                "APPROVED" => "verified",
                "REJECTED" => "rejected",
                other => return format!("refund_decision_unrecognised_{other}"),
            };
            let now = chrono::Utc::now().to_rfc3339();
            let result = sqlx::query(
                "UPDATE live_proof_sessions SET status = $1, completed_at = $2, updated_at = $3 \
                 WHERE id = $4 AND tenant_id = $5",
            )
            .bind(status)
            .bind(&now)
            .bind(&now)
            .bind(&session_id)
            .bind(tenant_id)
            .execute(&state.db)
            .await;
            match result {
                Ok(rows) if rows.rows_affected() > 0 => {
                    tracing::info!("Live proof session {session_id} set to {status}");
                    format!("live_proof_{status}")
                }
                Ok(_) => "live_proof_session_not_found".to_string(),
                Err(err) => {
                    tracing::error!("Could not update live proof session: {err}");
                    "live_proof_update_failed".to_string()
                }
            }
        }
        "order.placed" | "order.created" | "order.shipped" | "order.fulfilled"
        | "support.ticket.assigned" | "ticket.created" => {
            // The outbound send path these trigger is not on the Rust backend
            // yet; storing the event is real, claiming a notification was sent
            // would not be.
            tracing::warn!("Ecosystem event {event_name} stored but no automation engine is wired for it");
            format!("deferred_no_automation:{event_name}")
        }
        other => format!("unhandled_event:{other}"),
    }
}

