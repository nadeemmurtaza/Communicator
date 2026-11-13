#![allow(dead_code)]

//! Support layer for the customer-facing live-chat widget runtime.
//!
//! Everything here reads and writes real rows: the widget's boot config comes
//! from the website `inboxes` row and its JSON credentials, pre-chat form
//! structure from `credentials.preChatConfig`, visitor identity from `contacts`
//! (+ `leads` and a `consent_records` row), sessions from `widget_sessions`,
//! agent heartbeats from `agent_presence`, and cursors/read receipts from
//! `messages.created_at`.
//!
//! Nothing in this module invents a reply: `request_auto_reply` hands the
//! thread to the ported AI engine (`services::reply_engine`), and the outbound
//! write it may produce goes through `services::draft_actions` and
//! `services::messaging`, which is the only place in the crate allowed to write
//! `status = 'sent'`.

use crate::db::{AppState, DbPool};
use crate::services::compliance::{self, ConsentStatus};
use crate::services::draft_actions;
use crate::services::inbound::{self, InboundError};
use crate::services::reply_engine;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgRow;
use sqlx::Row;
use std::sync::OnceLock;
use tracing::warn;
use uuid::Uuid;

/// Channel stored on widget-produced `messages` rows.
pub const WIDGET_CHANNEL: &str = "widget";
/// Contact channels a widget visitor may already be filed under; the Python
/// backend used `website`.
pub const CONTACT_CHANNELS: [&str; 4] = ["widget", "website", "live-chat", "live_chat"];
pub const PROVIDER: &str = "web_widget";
pub const SESSION_COOKIE: &str = "communicator_widget_session";
/// Ported from the Python presence manager's TTL.
pub const PRESENCE_TTL_SECONDS: i64 = 30;
pub const MAX_POLL_LIMIT: i64 = 200;
pub const DEFAULT_POLL_LIMIT: i64 = 50;
/// Mirrors the widget composer's limit; longer bodies are refused, not trimmed.
pub const MAX_BODY_CHARS: usize = 8000;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum WidgetError {
    /// No `inboxes` row on a website channel exists at all.
    NoWebsiteInbox,
    /// A website inbox exists but its stored token is not the one presented.
    TokenMismatch,
    /// The inbox has never been issued a widget token, so nothing can match.
    TokenNotConfigured,
    /// The public request carried no `?token=` / `X-Widget-Token` at all.
    WidgetTokenMissing,
    /// The request carried an Origin the inbox's allowed-domain list rejects.
    OriginDenied(String),
    /// Identity validation is switched on but no HMAC secret is on file.
    /// An empty secret never means "accept".
    IdentityValidationUnconfigured,
    IdentityMismatch,
    MissingVisitorIdentity,
    EmptyMessage,
    MessageTooLong(usize),
    SessionTokenMissing,
    SessionUnknown,
    NoSessionForConversation,
    SessionClosed(String),
    /// The presented session belongs to a different conversation, so the caller
    /// has no claim on this thread.
    SessionConversationMismatch,
    ConversationNotFound(String),
    ClosedConversation(String),
    RatingOutOfRange(i64),
    CsatDisabled,
    /// The channel has its pre-chat intake switched off in the database.
    PreChatDisabled,
    Validation(Vec<String>),
    /// A table or column this runtime depends on is not in the schema yet.
    SchemaUnavailable(String),
    Database(sqlx::Error),
    Inbound(InboundError),
}

pub type WidgetResult<T> = Result<T, WidgetError>;

impl WidgetError {
    /// Stable machine-readable code for the widget client.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoWebsiteInbox => "no_website_inbox",
            Self::TokenMismatch => "token_mismatch",
            Self::TokenNotConfigured => "token_not_configured",
            Self::WidgetTokenMissing => "widget_token_missing",
            Self::OriginDenied(_) => "origin_not_allowed",
            Self::IdentityValidationUnconfigured => "identity_validation_unconfigured",
            Self::IdentityMismatch => "identity_mismatch",
            Self::MissingVisitorIdentity => "missing_visitor_identity",
            Self::EmptyMessage => "empty_message",
            Self::MessageTooLong(_) => "message_too_long",
            Self::SessionTokenMissing => "session_token_missing",
            Self::SessionUnknown => "session_unknown",
            Self::NoSessionForConversation => "no_session_for_conversation",
            Self::SessionClosed(_) => "session_closed",
            Self::SessionConversationMismatch => "session_conversation_mismatch",
            Self::ConversationNotFound(_) => "conversation_not_found",
            Self::ClosedConversation(_) => "conversation_closed",
            Self::RatingOutOfRange(_) => "rating_out_of_range",
            Self::CsatDisabled => "csat_disabled",
            Self::PreChatDisabled => "prechat_disabled",
            Self::Validation(_) => "prechat_validation_failed",
            Self::SchemaUnavailable(_) => "schema_unavailable",
            Self::Database(_) => "database_error",
            Self::Inbound(_) => "inbound_pipeline_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::NoWebsiteInbox => {
                "No live chat channel is configured. Connect the Website Live Chat inbox first."
                    .to_string()
            }
            Self::TokenMismatch => {
                "Widget token does not match any live chat channel.".to_string()
            }
            Self::TokenNotConfigured => {
                "The live chat channel has no website token; regenerate one in Live Chat settings."
                    .to_string()
            }
            Self::WidgetTokenMissing => {
                "A widget token is required: pass ?token=<websiteToken> or the X-Widget-Token header."
                    .to_string()
            }
            Self::OriginDenied(origin) => format!(
                "Origin '{origin}' is not in this widget's allowed domains list."
            ),
            Self::IdentityValidationUnconfigured => "This widget requires identity validation but no HMAC secret is configured, so the visitor cannot be verified; refusing the request.".to_string(),
            Self::IdentityMismatch => {
                "Visitor identity hash does not match the supplied email.".to_string()
            }
            Self::MissingVisitorIdentity => {
                "An email address is required to open a chat session.".to_string()
            }
            Self::EmptyMessage => "Message body is empty.".to_string(),
            Self::MessageTooLong(len) => {
                format!("Message body of {len} characters exceeds the widget limit.")
            }
            Self::SessionTokenMissing => "A widget session token is required: open a session with /widget/contact or /widget/prechat/submit and send the token back in the cookie, the X-Widget-Session header or sessionToken.".to_string(),
            Self::SessionUnknown => "Unknown widget session token.".to_string(),
            Self::NoSessionForConversation => {
                "No widget session has been opened for this conversation.".to_string()
            }
            Self::SessionClosed(id) => {
                format!("Widget session {id} is closed; start a new chat to continue.")
            }
            Self::SessionConversationMismatch => {
                "This widget session belongs to a different conversation.".to_string()
            }
            Self::ConversationNotFound(id) => format!("Conversation {id} not found."),
            Self::ClosedConversation(state) => format!(
                "Conversation is {state}, so no further messages can be sent from the widget."
            ),
            Self::RatingOutOfRange(rating) => {
                format!("CSAT rating {rating} is outside the 1..5 range.")
            }
            Self::CsatDisabled => "This inbox has CSAT surveys disabled, so there is nothing to record.".to_string(),
            Self::PreChatDisabled => {
                "Pre-chat intake is switched off for this channel; open a session with /widget/contact."
                    .to_string()
            }
            Self::Validation(errors) => errors.join(" "),
            Self::SchemaUnavailable(object) => format!(
                "Widget runtime storage is not initialised: `{object}` is missing from the database schema."
            ),
            Self::Database(err) => err.to_string(),
            Self::Inbound(err) => match err {
                InboundError::UnknownContact(detail) => detail.clone(),
                InboundError::Db(db) => db.to_string(),
            },
        }
    }
}

impl std::fmt::Display for WidgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl From<sqlx::Error> for WidgetError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

impl From<InboundError> for WidgetError {
    fn from(err: InboundError) -> Self {
        Self::Inbound(err)
    }
}

/// PostgreSQL names missing schema objects in its error text (`relation "x" does not exist`,
/// `column y does not exist`). Surfacing that verbatim is honest: the widget says the
/// storage is not initialised instead of pretending the row was never sent.
pub fn schema_gap(err: &sqlx::Error) -> Option<String> {
    if let sqlx::Error::Database(db) = err {
        let msg = db.message();
        for prefix in ["no such table: ", "no such column: ", "no such: "] {
            if let Some(rest) = msg.strip_prefix(prefix) {
                return Some(rest.trim().to_string());
            }
        }
        if msg.contains("no such table") || msg.contains("no such column") {
            return Some(msg.to_string());
        }
    }
    None
}

fn need<T>(result: Result<T, sqlx::Error>, context: &str) -> WidgetResult<T> {
    result.map_err(|err| match schema_gap(&err) {
        Some(object) => {
            warn!("widget schema gap while {context}: {object}");
            WidgetError::SchemaUnavailable(object)
        }
        None => {
            warn!("widget database error while {context}: {err}");
            WidgetError::Database(err)
        }
    })
}

// ---------------------------------------------------------------------------
// Secrets, hashes and validation primitives
// ---------------------------------------------------------------------------

pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Opaque bearer token for a widget session; only its SHA-256 is stored.
pub fn new_session_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Chatwoot-compatible visitor identity hash: hex HMAC-SHA256 of the identifier
/// keyed with the widget HMAC secret. An empty key is never accepted, so an
/// unconfigured deployment refuses rather than passing anyone through.
pub fn verify_identity_hash(secret: &str, identifier: &str, provided: Option<&str>) -> bool {
    if secret.is_empty() {
        return false;
    }
    let Some(provided) = provided.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(identifier.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    constant_time_eq(expected.as_bytes(), provided.as_bytes())
}

fn email_re() -> &'static regex::Regex {
    static RE: OnceLock<Result<regex::Regex, regex::Error>> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$"))
        .as_ref()
        .expect("ported email pattern compiles")
}

pub fn is_valid_email(value: &str) -> bool {
    email_re().is_match(value.trim())
}

pub fn normalize_email(value: &str) -> String {
    value.trim().to_lowercase()
}

/// The visitor's client address. Nothing in this deployment records connection
/// info, so the proxy header is the only real source; absence is reported as
/// empty rather than filled in with a placeholder.
pub fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .map(|list| list.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.trim().to_string())
        })
        .unwrap_or_default()
}

pub fn origin_header(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

/// Session token from the `X-Widget-Session` header, the session cookie, or a
/// body field -- in that order.
pub fn session_token_from(
    headers: &axum::http::HeaderMap,
    body_field: Option<&str>,
) -> Option<String> {
    if let Some(token) = headers
        .get("x-widget-session")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(token.to_string());
    }
    if let Some(token) = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie| {
            cookie.split(';').find_map(|pair| {
                let (name, value) = pair.trim().split_once('=')?;
                (name.trim() == SESSION_COOKIE && !value.trim().is_empty())
                    .then(|| value.trim().to_string())
            })
        })
    {
        return Some(token);
    }
    body_field
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

// ---------------------------------------------------------------------------
// Website inbox + credentials
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WebsiteInbox {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub is_active: bool,
    pub avatar_url: Option<String>,
    pub greeting_enabled: bool,
    pub greeting_message: Option<String>,
    pub out_of_office_enabled: bool,
    pub out_of_office_message: Option<String>,
    pub csat_enabled: bool,
    pub working_hours_enabled: bool,
    pub working_hours_start: Option<i64>,
    pub working_hours_end: Option<i64>,
    pub timezone: Option<String>,
    pub tenant_key: Option<String>,
    pub credentials: Value,
}

const INBOX_SELECT: &str = "SELECT id, name, channel_type, is_active, avatar_url, greeting_enabled, \
     greeting_message, out_of_office_enabled, out_of_office_message, csat_enabled, \
     working_hours_enabled, working_hours_start, working_hours_end, timezone, tenant_key, credentials \
     FROM inboxes WHERE channel_type IN ('website', 'live-chat', 'live_chat') \
     ORDER BY is_active DESC, updated_at DESC";

fn text(row: &PgRow, column: &str) -> String {
    row.try_get::<String, _>(column).unwrap_or_default()
}

fn opt_text(row: &PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
}

fn flag(row: &PgRow, column: &str) -> bool {
    row.try_get::<bool, _>(column).unwrap_or(false)
}

fn row_to_inbox(row: &PgRow) -> WebsiteInbox {
    let credentials = opt_text(row, "credentials")
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(|value| value.is_object())
        .unwrap_or_else(|| json!({}));

    WebsiteInbox {
        id: text(row, "id"),
        name: text(row, "name"),
        channel_type: text(row, "channel_type"),
        is_active: flag(row, "is_active"),
        avatar_url: opt_text(row, "avatar_url"),
        greeting_enabled: flag(row, "greeting_enabled"),
        greeting_message: opt_text(row, "greeting_message"),
        out_of_office_enabled: flag(row, "out_of_office_enabled"),
        out_of_office_message: opt_text(row, "out_of_office_message"),
        csat_enabled: flag(row, "csat_enabled"),
        working_hours_enabled: flag(row, "working_hours_enabled"),
        working_hours_start: row
            .try_get::<Option<i64>, _>("working_hours_start")
            .ok()
            .flatten(),
        working_hours_end: row.try_get::<Option<i64>, _>("working_hours_end").ok().flatten(),
        timezone: opt_text(row, "timezone"),
        tenant_key: opt_text(row, "tenant_key"),
        credentials,
    }
}

pub async fn load_website_inboxes(pool: &DbPool) -> WidgetResult<Vec<WebsiteInbox>> {
    let rows = need(
        sqlx::query(INBOX_SELECT).fetch_all(pool).await,
        "loading website inboxes",
    )?;
    Ok(rows.iter().map(row_to_inbox).collect())
}

/// Resolves the inbox a widget token belongs to. The Python router fell back to
/// "the first website inbox" whenever the token matched nothing, which let any
/// caller boot any widget; here the stored token has to match.
pub async fn inbox_for_token(pool: &DbPool, token: &str) -> WidgetResult<WebsiteInbox> {
    let inboxes = load_website_inboxes(pool).await?;
    if inboxes.is_empty() {
        return Err(WidgetError::NoWebsiteInbox);
    }
    let presented = token.trim();
    if presented.is_empty() {
        return Err(WidgetError::TokenMismatch);
    }
    for inbox in &inboxes {
        if cred_str(&inbox.credentials, &["websiteToken", "website_token"]) == Some(presented) {
            return Ok(inbox.clone());
        }
    }
    if inboxes
        .iter()
        .any(|i| cred_str(&i.credentials, &["websiteToken", "website_token"]).is_some())
    {
        Err(WidgetError::TokenMismatch)
    } else {
        Err(WidgetError::TokenNotConfigured)
    }
}

/// First website inbox, for flows that authenticate with a session token
/// instead of the public widget token.
pub async fn any_website_inbox(pool: &DbPool) -> WidgetResult<WebsiteInbox> {
    load_website_inboxes(pool)
        .await
        .and_then(|mut inboxes| inboxes.drain(..).next().ok_or(WidgetError::NoWebsiteInbox))
}

/// Reads a credential string, discarding values the UI masks back for display
/// (`••••1234`) -- those are not secrets and must not authenticate anything.
pub fn cred_str<'a>(creds: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .filter_map(|key| creds.get(*key).and_then(|v| v.as_str()))
        .map(str::trim)
        .find(|value| !value.is_empty() && !value.contains('\u{2022}'))
}

pub fn cred_bool(creds: &Value, keys: &[&str], fallback: bool) -> bool {
    keys.iter()
        .find_map(|key| creds.get(*key).and_then(|v| v.as_bool()))
        .unwrap_or(fallback)
}

/// Per-inbox `hmacSecret` wins, else the process-level configuration. `None`
/// when nothing real is configured; callers refuse instead of accepting.
pub fn widget_hmac_secret(inbox: &WebsiteInbox, config_secret: &str) -> Option<String> {
    cred_str(&inbox.credentials, &["hmacSecret", "hmac_secret"])
        .map(str::to_string)
        .or_else(|| {
            let trimmed = config_secret.trim();
            (!trimmed.is_empty() && !trimmed.contains('\u{2022}')).then(|| trimmed.to_string())
        })
}

pub fn identity_validation_enabled(inbox: &WebsiteInbox) -> bool {
    cred_bool(
        &inbox.credentials,
        &["identityValidation", "identity_validation"],
        false,
    )
}

/// Process-level widget identity secret.
///
/// WIRING NEEDED: `Config` should grow `widget_identity_hmac_secret: String`
/// (env `WIDGET_IDENTITY_HMAC_SECRET`, `unwrap_or_default()`, plus a
/// `apply_vault_secret` mapping) and callers should then pass
/// `state.config.widget_identity_hmac_secret` to `widget_hmac_secret`. Until
/// that field exists the value is read straight from the environment here; an
/// empty result always means "cannot verify", never "accept".
/// Convenience for handlers: per-inbox secret, else the deployment secret
/// from configuration. An empty configured secret yields `None`, which the
/// caller turns into a refusal rather than a guess.
pub fn resolve_hmac_secret(inbox: &WebsiteInbox, configured: &str) -> Option<String> {
    widget_hmac_secret(inbox, configured)
}

pub fn allowed_domains_of(inbox: &WebsiteInbox) -> Option<String> {
    cred_str(&inbox.credentials, &["allowedDomains", "allowed_domains"]).map(str::to_string)
}

// ---------------------------------------------------------------------------
// Origin / allowed-domain guard
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginDecision {
    /// Allowed by an explicit entry in the allowlist.
    Allowed,
    /// Rejected: the allowlist is configured and the origin is not on it.
    Denied(String),
    /// The request carried no Origin header (curl, native clients).
    NoOrigin,
    /// The inbox lists no domains, so nothing can be enforced.
    NotConfigured,
}

fn host_of(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let without_scheme = value.split("://").last().unwrap_or(value);
    let host = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .split(':')
        .next()
        .unwrap_or(without_scheme);
    (!host.is_empty()).then(|| host.to_lowercase())
}

/// `allowedDomains` is a comma/space separated host list, optionally wildcarded
/// (`*.example.com`) or `*`. Same shape the customizer stores, enforced for real
/// instead of only echoed back to the client.
pub fn check_origin(origin: Option<&str>, allowed_domains: Option<&str>) -> OriginDecision {
    let Some(origin) = host_of(origin.unwrap_or("")) else {
        return OriginDecision::NoOrigin;
    };
    let entries: Vec<String> = allowed_domains
        .unwrap_or("")
        .split([',', ' ', '\n', '\t', ';'])
        .filter_map(host_of)
        .collect();
    if entries.is_empty() {
        return OriginDecision::NotConfigured;
    }
    if entries.iter().any(|entry| entry == "*") {
        return OriginDecision::Allowed;
    }
    for entry in &entries {
        if entry == &origin {
            return OriginDecision::Allowed;
        }
        if let Some(suffix) = entry.strip_prefix("*.") {
            if origin.ends_with(&format!(".{suffix}")) || origin == *suffix {
                return OriginDecision::Allowed;
            }
        }
    }
    OriginDecision::Denied(origin)
}

// ---------------------------------------------------------------------------
// Widget styling, pre-chat config and routing rules
// ---------------------------------------------------------------------------

/// Factory styling, ported from the Python widget customizer defaults. Only
/// used for presentation fields the operator never set; every stored value in
/// `inboxes.credentials` wins. `avatarUrl`, `allowedDomains` and
/// `websiteDomain` deliberately have no invented default.
pub fn factory_styling() -> Value {
    json!({
        "widgetColor": "#1570EF",
        "themeMode": "light",
        "launcherStyle": "round_bubble",
        "launcherIcon": "chat_bubble",
        "launcherText": "Chat with us",
        "widgetPosition": "right",
        "greetingTitle": "Hi there!",
        "greetingTagline": "We typically reply in under 2 minutes.",
        "borderRadius": "rounded_2xl",
        "playSound": true,
        "showBranding": true,
        "preChatFormEnabled": true,
        "identityValidation": false
    })
}

pub fn styling(inbox: &WebsiteInbox) -> Value {
    let creds = &inbox.credentials;
    let mut out = serde_json::Map::new();
    if let Some(factory) = factory_styling().as_object().cloned() {
        for (key, default) in factory {
            out.insert(key.clone(), creds.get(&key).cloned().unwrap_or(default));
        }
    }
    out.insert(
        "avatarUrl".to_string(),
        cred_str(creds, &["avatarUrl", "avatar_url"])
            .map(Value::from)
            .or_else(|| inbox.avatar_url.clone().map(Value::from))
            .unwrap_or(Value::Null),
    );
    out.insert(
        "allowedDomains".to_string(),
        allowed_domains_of(inbox).map(Value::from).unwrap_or(Value::Null),
    );
    out.insert(
        "websiteDomain".to_string(),
        cred_str(creds, &["websiteDomain", "website_domain"])
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    out.insert(
        "preChatFields".to_string(),
        prechat_config(inbox)
            .get("fields")
            .cloned()
            .unwrap_or_else(|| json!([])),
    );
    out.insert("csatEnabled".to_string(), json!(inbox.csat_enabled));
    out.insert("greetingEnabled".to_string(), json!(inbox.greeting_enabled));
    out.insert(
        "greetingMessage".to_string(),
        inbox
            .greeting_message
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    Value::Object(out)
}

/// Factory pre-chat intake form and routing rules, ported from
/// DEFAULT_PRECHAT_FIELDS / DEFAULT_ROUTING_RULES.
pub fn factory_prechat_config() -> Value {
    json!({
        "enabled": true,
        "title": "Welcome to Live Chat",
        "description": "Please share a few details so our team can assist you right away.",
        "submit_button_text": "Start Conversation",
        "require_email": true,
        "fields": [
            { "id": "f_name", "name": "fullName", "label": "Full Name", "type": "text",
              "placeholder": "e.g. Jane Doe", "required": true, "options": [], "order": 1 },
            { "id": "f_email", "name": "email", "label": "Email Address", "type": "email",
              "placeholder": "e.g. jane@company.com", "required": true, "options": [], "order": 2 },
            { "id": "f_inquiry", "name": "inquiryType", "label": "What can we help you with?",
              "type": "select", "placeholder": "Select inquiry reason", "required": true,
              "options": ["Sales & Pricing", "Technical Support", "Billing & Invoices", "Partnerships & Other"],
              "order": 3 },
            { "id": "f_company", "name": "company", "label": "Company / Organization",
              "type": "text", "placeholder": "e.g. Acme Innovations", "required": false,
              "options": [], "order": 4 },
            { "id": "f_message", "name": "message", "label": "Initial Question or Requirement",
              "type": "textarea", "placeholder": "Briefly describe your request...",
              "required": false, "options": [], "order": 5 }
        ],
        "routing_rules": [
            { "id": "rule_billing", "field": "inquiryType", "operator": "equals",
              "value": "Billing & Invoices", "assign_priority": "high", "assign_tag": "billing",
              "team_target": "Finance & Billing" },
            { "id": "rule_sales", "field": "inquiryType", "operator": "equals",
              "value": "Sales & Pricing", "assign_priority": "high", "assign_tag": "sales_lead",
              "team_target": "Sales Operations" },
            { "id": "rule_tech", "field": "inquiryType", "operator": "equals",
              "value": "Technical Support", "assign_priority": "medium", "assign_tag": "tier1_support",
              "team_target": "Engineering Support" }
        ]
    })
}

/// Reads the operator's stored form definition, falling back to the factory
/// shape only when nothing has been saved for this inbox.
pub fn prechat_config(inbox: &WebsiteInbox) -> Value {
    let creds = &inbox.credentials;
    let factory = factory_prechat_config();
    let mut cfg = match creds.get("preChatConfig").filter(|v| v.is_object()) {
        Some(stored) => stored.clone(),
        None => factory.clone(),
    };
    let Some(object) = cfg.as_object_mut() else {
        return factory;
    };

    if !object.contains_key("fields") {
        // Legacy flat form written by older passes as credentials.preChatFields.
        match creds.get("preChatFields").and_then(|v| v.as_array()) {
            Some(fields) => {
                let migrated: Vec<Value> = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let name = field
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("field")
                            .to_string();
                        json!({
                            "id": format!("field_{index}"),
                            "name": name,
                            "label": field.get("label").and_then(|v| v.as_str()).unwrap_or(&name),
                            "type": field.get("type").and_then(|v| v.as_str()).unwrap_or("text"),
                            "placeholder": field.get("placeholder").and_then(|v| v.as_str()).unwrap_or(""),
                            "required": field.get("required").and_then(|v| v.as_bool()).unwrap_or(false),
                            "options": field.get("options").cloned().unwrap_or_else(|| json!([])),
                            "order": index as i64 + 1
                        })
                    })
                    .collect();
                object.insert("fields".to_string(), Value::Array(migrated));
            }
            None => {
                object.insert("fields".to_string(), factory["fields"].clone());
            }
        }
    }
    object
        .entry("routing_rules".to_string())
        .or_insert_with(|| factory["routing_rules"].clone());
    for key in ["title", "description", "submit_button_text", "require_email"] {
        if !object.contains_key(key) {
            if let Some(value) = factory.get(key) {
                object.insert(key.to_string(), value.clone());
            }
        }
    }
    // The channel-level switch the customizer writes beats the form's own flag.
    let enabled = cred_bool(&inbox.credentials, &["preChatFormEnabled"], {
        object
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    });
    object.insert("enabled".to_string(), json!(enabled));
    Value::Object(serde_json::Map::from_iter(object.clone()))
}

fn stringify(value: &Value) -> String {
    match value {
        Value::String(text) => text.trim().to_string(),
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(num) => num.to_string(),
        other => other.to_string(),
    }
}

pub fn answer_str(answers: &Value, keys: &[&str]) -> String {
    keys.iter()
        .filter_map(|key| answers.get(*key))
        .map(stringify)
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

/// Ported validation: required fields, email format, allowed select choices,
/// numeric fields, and the channel-wide "email required" rule.
pub fn validate_prechat_answers(cfg: &Value, answers: &Value) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let fields: Vec<Value> = cfg
        .get("fields")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let require_email = cfg
        .get("require_email")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    for field in &fields {
        let Some(name) = field.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let label = field.get("label").and_then(|v| v.as_str()).unwrap_or(name);
        let value = answers.get(name).map(stringify).unwrap_or_default();
        let required = field
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if required && value.is_empty() {
            errors.push(format!("Field '{label}' is required."));
            continue;
        }
        if value.is_empty() {
            continue;
        }

        match field.get("type").and_then(|v| v.as_str()).unwrap_or("text") {
            "email" => {
                if !is_valid_email(&value) {
                    errors.push(format!("Field '{label}' must be a valid email address."));
                }
            }
            "select" => {
                let options: Vec<String> = field
                    .get("options")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(stringify).collect())
                    .unwrap_or_default();
                if !options.is_empty() && !options.iter().any(|option| option == &value) {
                    errors.push(format!(
                        "Selected value '{value}' for '{label}' is not among allowed choices: {}.",
                        options.join(", ")
                    ));
                }
            }
            "number" => {
                if value.parse::<f64>().is_err() {
                    errors.push(format!("Field '{label}' must be a number."));
                }
            }
            _ => {}
        }
    }

    if require_email {
        let email = answer_str(answers, &["email", "emailAddress"]);
        let already_reported = errors
            .iter()
            .any(|error| error.to_lowercase().contains("email"));
        if !already_reported && (!email.is_empty() && !is_valid_email(&email) || email.is_empty()) {
            errors.push("A valid email address is required to initiate live chat.".to_string());
        }
    }

    errors
}

fn priority_weight(priority: &str) -> i32 {
    match priority {
        "low" => 1,
        "medium" => 2,
        "high" => 3,
        "urgent" => 4,
        _ => 0,
    }
}

/// Ported conditional routing: the highest matching priority wins, tags
/// accumulate, the first non-empty team target wins.
pub fn evaluate_routing_rules(cfg: &Value, answers: &Value) -> Value {
    let rules: Vec<Value> = cfg
        .get("routing_rules")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut priority = "medium".to_string();
    let mut tags: Vec<String> = Vec::new();
    let mut team_target: Option<String> = None;
    let mut applied: Vec<String> = Vec::new();

    for rule in &rules {
        let field = rule.get("field").and_then(|v| v.as_str()).unwrap_or("");
        let operator = rule
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("equals")
            .to_lowercase();
        let target = answer_str(rule, &["value"]).to_lowercase();
        let mut submitted = answer_str(answers, &[field]).to_lowercase();
        if submitted.is_empty() && field == "inquiryType" {
            submitted = answer_str(answers, &["inquiry_type", "inquiry", "topic"]).to_lowercase();
        }

        let matched = match operator.as_str() {
            "equals" => submitted == target,
            "contains" => !target.is_empty() && submitted.contains(&target),
            "not_equals" => !submitted.is_empty() && submitted != target,
            _ => false,
        };
        if !matched {
            continue;
        }

        applied.push(answer_str(rule, &["id"]));
        let rule_priority = answer_str(rule, &["assign_priority"]).to_lowercase();
        if priority_weight(&rule_priority) > priority_weight(&priority) {
            priority = rule_priority;
        }
        let tag = answer_str(rule, &["assign_tag"]).to_lowercase();
        if !tag.is_empty() && !tags.contains(&tag) {
            tags.push(tag);
        }
        let team = answer_str(rule, &["team_target"]);
        if !team.is_empty() && team_target.is_none() {
            team_target = Some(team);
        }
    }

    json!({
        "priority": priority,
        "tags": tags,
        "team_target": team_target,
        "applied_rules": applied
    })
}

// ---------------------------------------------------------------------------
// Contact, conversation and message persistence
// ---------------------------------------------------------------------------

/// Which tenant owns a widget visitor: the website inbox's own key, else the
/// configured default, else no key at all -- recorded as empty and warned about,
/// never invented.
pub async fn resolve_tenant(pool: &DbPool, inbox: &WebsiteInbox, default: &str) -> String {
    if let Some(key) = inbox.tenant_key.as_deref() {
        return key.to_string();
    }
    let from_any = sqlx::query(
        "SELECT tenant_key FROM inboxes WHERE channel_type IN ('website','live-chat','live_chat') \
         AND tenant_key IS NOT NULL AND tenant_key != '' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .and_then(|row| opt_text(&row, "tenant_key"));
    from_any
        .or_else(|| {
            let trimmed = default.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .unwrap_or_else(|| {
            warn!(
                "widget visitor provisioned without a tenant key: \
                 set inboxes.tenant_key or DEFAULT_TENANT_KEY"
            );
            String::new()
        })
}

/// Finds the visitor across every website channel alias they may be filed
/// under, otherwise provisions a real Lead + Contact through the shared inbound
/// pipeline plus an explicit OPTED_IN consent row.
pub async fn resolve_widget_contact(
    pool: &DbPool,
    email: &str,
    display_name: &str,
    tenant_key: &str,
    consent_source: &str,
) -> WidgetResult<(String, bool)> {
    let email = normalize_email(email);
    if email.is_empty() {
        return Err(WidgetError::MissingVisitorIdentity);
    }

    let placeholders = CONTACT_CHANNELS
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 2))
        .collect::<Vec<_>>()
        .join(",");
    let lookup_sql = format!(
        "SELECT id, person_name FROM contacts WHERE value = $1 AND channel IN ({placeholders}) \
         ORDER BY updated_at DESC LIMIT 1"
    );
    let mut query = sqlx::query(&lookup_sql);
    query = query.bind(&email);
    for channel in CONTACT_CHANNELS {
        query = query.bind(channel);
    }
    let existing = need(
        query.fetch_optional(pool).await,
        "looking up the widget contact",
    )?;

    if let Some(row) = existing {
        let id: String = row.try_get("id").unwrap_or_default();
        let current: Option<String> = row.try_get("person_name").ok().flatten();
        let provided = display_name.trim();
        let replaceable = current.as_deref().map(orphan_name).unwrap_or(true);
        if !provided.is_empty() && replaceable {
            // Load-bearing: the widget response's `visitor_name` and the agent
            // inbox both come from `person_name`, so a rename that did not land
            // has to be reported -- otherwise the visitor believes they gave a
            // name and the CRM keeps showing the import placeholder. The name
            // itself is not logged; the contact id identifies the row.
            match sqlx::query("UPDATE contacts SET person_name = $1, updated_at = $2 WHERE id = $3")
                .bind(provided)
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(&id)
                .execute(pool)
                .await
            {
                Ok(report) if report.rows_affected() == 1 => {
                    // The stored name is the visitor's own now; that is the only
                    // outcome worth silence.
                }
                Ok(report) => tracing::error!(
                    "[DB] widget contact {id}: UPDATE contacts.person_name with the name the \
                     visitor typed affected {} rows (expected 1) -- the stored name was not \
                     changed, so the agent inbox still shows the placeholder",
                    report.rows_affected()
                ),
                Err(err) => tracing::error!(
                    "[DB] widget contact {id}: UPDATE contacts.person_name with the name the \
                     visitor typed failed ({}): {err}",
                    schema_gap(&err).unwrap_or_else(|| "query error".to_string())
                ),
            }
        }
        return Ok((id, false));
    }

    let contact_id =
        inbound::get_or_create_contact(pool, WIDGET_CHANNEL, &email, display_name.trim(), tenant_key)
            .await?;
    // Load-bearing: `get_or_create_contact` files the identity in `contacts.value`
    // only, so this statement is the single writer of `contacts.email`. A silent
    // failure would leave the CRM listing the visitor with no email at all.
    match sqlx::query("UPDATE contacts SET email = $1 WHERE id = $2")
        .bind(&email)
        .bind(&contact_id)
        .execute(pool)
        .await
    {
        Ok(report) if report.rows_affected() == 1 => {
            // Email stored; nothing to report.
        }
        Ok(report) => tracing::error!(
            "[DB] widget contact {contact_id}: UPDATE contacts.email affected {} rows (expected 1) \
             -- the row exists without its email column set",
            report.rows_affected()
        ),
        Err(err) => tracing::error!(
            "[DB] widget contact {contact_id}: UPDATE contacts.email failed ({}): {err} -- the \
             visitor is stored under `value` only",
            schema_gap(&err).unwrap_or_else(|| "query error".to_string())
        ),
    }
    compliance::record_consent(
        pool,
        &contact_id,
        ConsentStatus::OptedIn,
        consent_source,
        "Visitor initiated live chat session",
    )
    .await?;
    Ok((contact_id, true))
}

/// Names the earlier import passes wrote; a name the visitor actually gives
/// replaces them, a real human name is never overwritten.
fn orphan_name(name: &str) -> bool {
    name.is_empty()
        || name.starts_with("Contact ")
        || name.ends_with(" User")
        || name.ends_with(" Lead")
        || name == "Website Visitor"
}

/// The visitor's stored profile, read back from `contacts` so responses report
/// the persisted name/value instead of echoing the request body.
pub async fn contact_profile(
    pool: &DbPool,
    contact_id: &str,
) -> WidgetResult<Option<(String, String)>> {
    let row = need(
        sqlx::query("SELECT person_name, value FROM contacts WHERE id = $1")
            .bind(contact_id)
            .fetch_optional(pool)
            .await,
        "reading the visitor profile",
    )?;
    Ok(row.map(|row| {
        (
            opt_text(&row, "person_name").unwrap_or_default(),
            opt_text(&row, "value").unwrap_or_default(),
        )
    }))
}

/// Reads a conversation's stored `custom_attributes` as JSON.
pub async fn conversation_attributes(pool: &DbPool, conversation_id: &str) -> Value {
    sqlx::query("SELECT custom_attributes FROM conversations WHERE id = $1")
        .bind(conversation_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|row| opt_text(&row, "custom_attributes"))
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| json!({}))
}

pub async fn merge_conversation_attributes(
    pool: &DbPool,
    conversation_id: &str,
    additions: Value,
) -> WidgetResult<()> {
    let mut merged = match conversation_attributes(pool, conversation_id).await {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    if let Value::Object(extra) = additions {
        for (key, value) in extra {
            merged.insert(key, value);
        }
    }
    let raw = serde_json::to_string(&Value::Object(merged)).unwrap_or_else(|_| "{}".to_string());
    need(
        sqlx::query("UPDATE conversations SET custom_attributes = $1, updated_at = $2 WHERE id = $3")
            .bind(&raw)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(conversation_id)
            .execute(pool)
            .await,
        "merging conversation attributes",
    )?;
    Ok(())
}

/// Finds the live widget conversation for a contact, or opens one scoped to the
/// website inbox. Won/lost threads reopen; the insert itself is the shared
/// inbound helper so the column set stays in one place.
pub async fn open_widget_conversation(
    pool: &DbPool,
    contact_id: &str,
    inbox_id: &str,
    priority: &str,
    attributes: Value,
) -> WidgetResult<(String, bool)> {
    let existing = need(
        sqlx::query(
            "SELECT id, state FROM conversations WHERE contact_id = $1 AND \
             (inbox_id = $2 OR inbox_id IS NULL) ORDER BY created_at DESC LIMIT 1",
        )
        .bind(contact_id)
        .bind(inbox_id)
        .fetch_optional(pool)
        .await,
        "looking up the widget conversation",
    )?;

    let is_new = match &existing {
        Some(row) => {
            let state = opt_text(row, "state").unwrap_or_default().to_lowercase();
            matches!(state.as_str(), "closed_won" | "closed_lost")
        }
        None => true,
    };

    let conversation_id = inbound::get_or_create_inbox_conversation(
        pool,
        contact_id,
        WIDGET_CHANNEL,
        Some(inbox_id),
        None,
    )
    .await?;

    if is_new {
        need(
            sqlx::query(
                "UPDATE conversations SET priority = $1, custom_attributes = $2, updated_at = $3 WHERE id = $4",
            )
            .bind(priority)
            .bind(attributes.to_string())
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(&conversation_id)
            .execute(pool)
            .await,
            "storing widget conversation attributes",
        )?;
    }

    Ok((conversation_id, is_new))
}

/// Writes the inbox greeting as a genuine outbound row. The text is the stored
/// `inboxes.greeting_message`; when that column is empty nothing is written and
/// the caller is told the greeting was skipped rather than shown an invented one.
pub async fn inject_greeting(
    pool: &DbPool,
    inbox: &WebsiteInbox,
    conversation_id: &str,
) -> WidgetResult<Option<String>> {
    if !inbox.greeting_enabled {
        return Ok(None);
    }
    let Some(body) = inbox.greeting_message.as_deref() else {
        warn!(
            "inbox {} has greeting_enabled but no greeting_message; no greeting written",
            inbox.id
        );
        return Ok(None);
    };

    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().simple().to_string();
    let provider_message_id = format!("greet_{}", Uuid::new_v4().simple());
    need(
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, direction, channel, body, status, provider, \
             provider_message_id, sender_type, sender_name, sender, is_private, created_at, sent_at) \
             VALUES ($1, $2, 'outbound', $3, $4, 'delivered', $5, $6, 'agent', $7, 'agent', 0, $8, $9)",
        )
        .bind(&id)
        .bind(conversation_id)
        .bind(WIDGET_CHANNEL)
        .bind(body)
        .bind(PROVIDER)
        .bind(&provider_message_id)
        .bind(&inbox.name)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await,
        "writing the greeting message",
    )?;
    // Auxiliary: the greeting the visitor sees is the `messages` row above, and
    // the route already returned its id. This bump only orders the agent inbox,
    // so a miss is logged through the shared sink instead of failing the boot.
    crate::db::report_write(
        sqlx::query("UPDATE conversations SET last_outbound_at = $1, updated_at = $2 WHERE id = $3")
            .bind(&now)
            .bind(&now)
            .bind(conversation_id)
            .execute(pool)
            .await
            .map(|report| report.rows_affected()),
        &format!(
            "UPDATE conversations SET last_outbound_at for greeting {id} in conversation \
             {conversation_id}"
        ),
    );
    Ok(Some(id))
}

/// The one path a widget message takes: a real `messages` row, direction
/// inbound, channel `widget`, through the shared inbound writer (which dedupes
/// on the provider id and bumps `last_inbound_at`). Returns the row id and its
/// DB-derived cursor.
pub async fn record_visitor_message(
    pool: &DbPool,
    conversation_id: &str,
    body: &str,
    sender_name: &str,
    metadata: Value,
) -> WidgetResult<(String, String, String)> {
    let provider_message_id = format!("in_web_{}", Uuid::new_v4().simple());
    let message_id = inbound::record_raw_inbound_message(
        pool,
        conversation_id,
        WIDGET_CHANNEL,
        body,
        PROVIDER,
        &provider_message_id,
        sender_name,
        Some(metadata),
        None,
    )
    .await?;

    let row = need(
        sqlx::query("SELECT id, sent_at, created_at FROM messages WHERE id = $1")
            .bind(&message_id)
            .fetch_optional(pool)
            .await,
        "reading the stored message cursor",
    )?;
    match row {
        Some(row) => {
            // Use created_at as cursor for PostgreSQL-compatible pagination
            let cursor: String = row.try_get("created_at").unwrap_or_default();
            let sent_at = opt_text(&row, "sent_at").unwrap_or_default();
            Ok((message_id, cursor, sent_at))
        }
        None => {
            warn!("widget message {message_id} was written but cannot be read back");
            Ok((message_id, String::new(), String::new()))
        }
    }
}

/// What the auto-reply path actually did. The three facts are separate because
/// they fail separately: the model can answer while the draft gate refuses, and
/// a reply can be stored in the thread while nothing is pushed to the visitor.
#[derive(Debug, Clone)]
pub struct AutoReplyOutcome {
    /// The inbound message this reply answers.
    pub in_reply_to: String,
    /// The AI engine produced text and the engine's own write was attempted.
    pub generated: bool,
    /// A `messages` row in this conversation exists and was read back.
    pub stored: bool,
    /// A provider accepted the send. Only true when `messaging::apply_send_result`
    /// wrote `sent` behind `SendResult::is_accepted()`.
    pub pushed: bool,
    /// Exact reason for whatever did not happen, quoted from the service that
    /// refused it instead of reworded here.
    pub reason: String,
    pub draft_id: Option<String>,
    pub draft_status: Option<String>,
    pub confidence: Option<f64>,
    pub message_id: Option<String>,
    /// Status the read-back `messages` row actually carries.
    pub message_status: Option<String>,
    /// The stored row in the exact shape the poll endpoint serves it.
    pub message: Option<Value>,
    /// The outbound pipeline's own report, verbatim.
    pub send: Value,
    pub warnings: Vec<String>,
}

impl AutoReplyOutcome {
    fn unanswered(in_reply_to: &str, reason: String) -> Self {
        Self {
            in_reply_to: in_reply_to.to_string(),
            generated: false,
            stored: false,
            pushed: false,
            reason,
            draft_id: None,
            draft_status: None,
            confidence: None,
            message_id: None,
            message_status: None,
            message: None,
            send: Value::Null,
            warnings: Vec::new(),
        }
    }

    /// The thread status the widget client is told. `answered` only when there is
    /// a stored row the visitor can read; otherwise the thread is still queued
    /// for a human, which is the truth.
    pub fn thread_status(&self) -> &'static str {
        if self.stored {
            "answered"
        } else {
            "queued"
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "generated": self.generated,
            "stored": self.stored,
            "pushed": self.pushed,
            "reason": self.reason,
            "inReplyTo": self.in_reply_to,
            "in_reply_to": self.in_reply_to,
            "draftId": self.draft_id,
            "draft_id": self.draft_id,
            "draftStatus": self.draft_status,
            "draft_status": self.draft_status,
            "confidence": self.confidence,
            "messageId": self.message_id,
            "message_id": self.message_id,
            "messageStatus": self.message_status,
            "send": self.send,
            "warnings": self.warnings,
        })
    }
}

/// The refusal a service module already composed. `ReplyError::json()` and
/// `DraftActionError::json()` both carry the operator-facing text (including the
/// missing environment variable name and the provider's own error), so it is
/// quoted rather than paraphrased -- and a module that reported no text is
/// reported as having reported no text.
fn service_refusal(payload: Value) -> String {
    payload
        .get("error")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "the reply service reported a failure without any error text; its payload was: \
                 {payload}"
            )
        })
}

/// The widget's auto-reply path.
///
/// Three separate steps, each reported as itself:
///   1. `services::reply_engine::generate_reply` -- the ported AI engine. It
///      builds the prompt from real rows, calls the configured provider through
///      `services::claude_client` (credential read from `state.config` / the
///      environment there), and stores what the model actually said as a `drafts`
///      row. Unconfigured or failing provider => no text, with its exact reason.
///   2. The engine's own auto-send gate (`REPLY_AUTO_SEND_CONFIDENCE`). A draft
///      it left for a human is not promoted here, so no reply is written.
///   3. `services::draft_actions::approve_and_send` -> `services::messaging::send_outbound`,
///      the crate's only outbound path: consent gate, business-hours gate, channel
///      adapter, and `status = 'sent'` only from `apply_send_result` behind
///      `is_accepted()`. Live web chat has no push transport
///      (`channels::WebChatAdapter` answers `not_sent`), so the honest outcome is
///      a stored row the visitor picks up by polling
///      `GET /widget/conversations/<id>/messages/since` and `pushed: false`.
pub async fn request_auto_reply(
    state: &AppState,
    conversation_id: &str,
    message_id: &str,
) -> AutoReplyOutcome {
    // 1. The thread's real state first. `human_takeover` is the operator's own
    //    flag and routes/automation.rs documents that the reply engine reads it;
    //    if it cannot be read the reply is withheld rather than risk talking
    //    over a human who is already in the conversation.
    let thread = sqlx::query(
        "SELECT COALESCE(state, '') AS state, human_takeover FROM conversations WHERE id = $1 \
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await;
    let thread = match thread {
        Ok(Some(row)) => row,
        Ok(None) => {
            let reason = format!(
                "there is no conversations row {conversation_id}, so the widget inbound \
                 {message_id} has no thread to reply in and no auto-reply was generated"
            );
            warn!("widget auto-reply skipped: {reason}");
            return AutoReplyOutcome::unanswered(message_id, reason);
        }
        Err(err) => {
            let reason = format!(
                "conversations row {conversation_id} could not be read ({err}), so the widget \
                 inbound {message_id} was left for a human instead of auto-replying over a thread \
                 whose owner is unknown"
            );
            warn!("widget auto-reply skipped: {reason}");
            return AutoReplyOutcome::unanswered(message_id, reason);
        }
    };

    let taken_over = match thread.try_get::<i64, _>("human_takeover") {
        Ok(value) => value != 0,
        Err(int_err) => match thread.try_get::<bool, _>("human_takeover") {
            Ok(value) => value,
            Err(bool_err) => {
                let reason = format!(
                    "conversations.human_takeover for {conversation_id} could not be read \
                     ({int_err}; {bool_err}), so it is unknown whether a human owns this thread \
                     and no auto-reply was generated"
                );
                warn!("widget auto-reply skipped: {reason}");
                return AutoReplyOutcome::unanswered(message_id, reason);
            }
        },
    };
    if taken_over {
        let stored_state = opt_text(&thread, "state").unwrap_or_else(|| "(no state)".to_string());
        let reason = format!(
            "a human agent holds conversation {conversation_id} (conversations.human_takeover is \
             set, state '{stored_state}'), so no AI reply was generated for inbound {message_id}"
        );
        warn!("widget auto-reply skipped: {reason}");
        return AutoReplyOutcome::unanswered(message_id, reason);
    }

    // 2. The real AI machinery. Tenant is not guessed: the engine resolves it
    //    from the contact/conversation rows and then the configured default, and
    //    errors out loudly when nothing scopes it.
    let draft = match reply_engine::generate_reply(state, conversation_id, None).await {
        Ok(draft) => draft,
        Err(err) => {
            let status = err.status();
            let reason = service_refusal(err.json());
            warn!(
                "widget inbound {message_id}: no auto-reply for conversation {conversation_id} \
                 (HTTP {status}): {reason}"
            );
            return AutoReplyOutcome::unanswered(message_id, reason);
        }
    };

    let mut outcome = AutoReplyOutcome::unanswered(message_id, String::new());
    outcome.generated = true;
    outcome.draft_id = Some(draft.id.clone());
    outcome.draft_status = Some(draft.status.clone());
    outcome.confidence = draft.model.confidence;
    outcome.warnings = draft.warnings.clone();
    if !draft.draft_written {
        outcome.warnings.push(format!(
            "the reply engine's drafts row for {} was not written, so the drafted text exists only \
             in this response",
            draft.id
        ));
    }

    // 3. The engine's own gate. It is the only thing allowed to promote a reply,
    //    and it does so from the model's real confidence against the operator's
    //    configured threshold -- never from a number chosen here.
    if draft.status != "auto_approved" {
        outcome.reason = match (draft.auto_send_threshold, draft.model.confidence) {
            (None, _) => format!(
                "the model drafted a reply (draft {}, status '{}'), but REPLY_AUTO_SEND_CONFIDENCE \
                 is not set so nothing is auto-approved: the reply waits for a human in Drafts and \
                 nothing was stored in the conversation",
                draft.id, draft.status
            ),
            (Some(limit), Some(confidence)) => format!(
                "the model drafted a reply (draft {}, status '{}') at confidence {confidence}, \
                 below the configured REPLY_AUTO_SEND_CONFIDENCE {limit}: it waits for a human in \
                 Drafts and nothing was stored in the conversation",
                draft.id, draft.status
            ),
            (Some(limit), None) => format!(
                "the model drafted a reply (draft {}, status '{}') and reported no confidence, so \
                 it cannot clear REPLY_AUTO_SEND_CONFIDENCE {limit}: it waits for a human in Drafts \
                 and nothing was stored in the conversation",
                draft.id, draft.status
            ),
        };
        if draft.escalation {
            outcome.reason.push_str(
                " The model flagged this reply as an escalation, so a human decides it regardless \
                 of confidence.",
            );
        }
        warn!("widget inbound {message_id}: {}", outcome.reason);
        return outcome;
    }

    // 4. The only outbound path in the crate. `actor = "ai"` keeps the sender
    //    identity the draft pipeline already uses and moves the draft off
    //    `auto_approved`, so the scheduled dispatcher cannot send it a second time.
    let report = match draft_actions::approve_and_send(state, &draft.id, "ai").await {
        Ok(report) => report,
        Err(err) => {
            let reason = service_refusal(err.json());
            warn!(
                "widget inbound {message_id}: the auto-approved reply for conversation \
                 {conversation_id} could not reach the outbound pipeline: {reason}"
            );
            outcome.reason = reason;
            return outcome;
        }
    };
    outcome.pushed = report.sent;
    outcome.message_id = report.message_id.clone();
    outcome.send = report.send.clone();
    outcome.warnings.extend(report.warnings.iter().cloned());
    if let Some(note) = &report.audit_note {
        outcome.warnings.push(note.clone());
    }

    let Some(stored_id) = report.message_id.clone() else {
        outcome.reason = format!(
            "the outbound pipeline returned no message id (its report was: {}), so there is no \
             row to verify and the visitor has nothing to poll",
            report.send
        );
        warn!("widget inbound {message_id}: {}", outcome.reason);
        return outcome;
    };

    // The read-back is the proof of `stored`: what the database holds, not what
    // the send call claimed.
    match sqlx::query(&format!(
        "SELECT conversation_id, {MESSAGE_COLUMNS} FROM messages WHERE id = $1 LIMIT 1"
    ))
    .bind(&stored_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            let owner = row
                .try_get::<Option<String>, _>("conversation_id")
                .ok()
                .flatten();
            match owner {
                Some(owner) if owner == conversation_id => {
                    outcome.stored = true;
                    outcome.message_status = opt_text(&row, "status");
                    outcome.message = Some(message_json(&row));
                }
                Some(other) => {
                    outcome.reason = format!(
                        "outbound row {stored_id} is filed under conversation {other}, not \
                         {conversation_id}, so this thread has no reply the visitor can poll"
                    );
                    warn!("widget inbound {message_id}: {}", outcome.reason);
                }
                None => {
                    outcome.reason = format!(
                        "outbound row {stored_id} exists but its conversation_id could not be \
                         read, so it cannot be confirmed as a reply in {conversation_id}"
                    );
                    warn!("widget inbound {message_id}: {}", outcome.reason);
                }
            }
        }
        Ok(None) => {
            outcome.reason = format!(
                "the outbound pipeline reported message {stored_id} but no messages row with that \
                 id is in the database, so nothing was stored for conversation {conversation_id}"
            );
            warn!("widget inbound {message_id}: {}", outcome.reason);
        }
        Err(err) => {
            outcome.reason = format!(
                "the stored state of outbound {stored_id} could not be verified ({err}); the \
                 conversation may or may not have a reply to poll"
            );
            warn!("widget inbound {message_id}: {}", outcome.reason);
        }
    }

    outcome.reason = if outcome.pushed {
        format!(
            "the provider accepted the reply; messages row {stored_id} carries status '{}'",
            outcome.message_status.as_deref().unwrap_or("(unreadable)")
        )
    } else {
        let refusal = report
            .send
            .get("error")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "the outbound pipeline reported status {} and gave no error detail, so nothing \
                     was confirmed delivered",
                    report
                        .send
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("(absent)")
                )
            });
        if outcome.stored {
            outcome.warnings.push(format!(
                "the reply is stored as messages row {stored_id} (status '{}') and reaches the \
                 visitor only through GET /widget/conversations/{conversation_id}/messages/since; \
                 nothing was pushed to them.",
                outcome.message_status.as_deref().unwrap_or("(unreadable)")
            ));
        }
        refusal
    };
    warn!(
        "widget inbound {message_id} in conversation {conversation_id}: auto-reply generated=true, \
         stored={}, pushed={}, status='{}' -- {}",
        outcome.stored,
        outcome.pushed,
        outcome.draft_status.as_deref().unwrap_or("(absent)"),
        outcome.reason
    );
    outcome
}

// ---------------------------------------------------------------------------
// Cursors, history, transcripts and read receipts
// ---------------------------------------------------------------------------

pub async fn max_message_cursor(pool: &DbPool, conversation_id: &str) -> WidgetResult<String> {
    let row = need(
        sqlx::query(
            "SELECT COALESCE(MAX(created_at), '') AS cursor FROM messages WHERE conversation_id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(pool)
        .await,
        "reading the conversation cursor",
    )?;
    Ok(row
        .and_then(|row| row.try_get::<String, _>("cursor").ok())
        .unwrap_or_default())
}

/// Cursors are `"<messages.created_at>:<sent_at>"`; a bare timestamp is accepted too so a
/// client can replay a cursor it stored earlier.
pub fn cursor_rowid(cursor: Option<&str>) -> String {
    cursor
        .unwrap_or("")
        .split(':')
        .next()
        .map(|head| head.trim().to_string())
        .unwrap_or_default()
}

fn message_json(row: &PgRow) -> Value {
    let direction = opt_text(row, "direction").unwrap_or_default();
    let created_at = opt_text(row, "created_at").unwrap_or_default();
    let sent_at = opt_text(row, "sent_at")
        .or_else(|| opt_text(row, "created_at"))
        .unwrap_or_default();
    json!({
        "id": text(row, "id"),
        "cursor": created_at,
        "cursorToken": format!("{created_at}:{sent_at}"),
        "body": text(row, "body"),
        "direction": direction.to_lowercase(),
        "senderType": opt_text(row, "sender_type").unwrap_or_else(|| "contact".to_string()),
        "senderName": opt_text(row, "sender_name").unwrap_or_else(|| {
            if direction.eq_ignore_ascii_case("outbound") { "Agent" } else { "You" }.to_string()
        }),
        "sentAt": sent_at,
        "status": opt_text(row, "status"),
    })
}

const MESSAGE_COLUMNS: &str =
    "id, direction, body, sender_type, sender_name, status, sent_at, created_at";

/// Visitor-visible history. Internal private notes are excluded by the query
/// itself, never filtered after the fact.
pub async fn visible_messages(
    pool: &DbPool,
    conversation_id: &str,
    after_cursor: String,
    limit: i64,
) -> WidgetResult<Vec<Value>> {
    let rows = need(
        sqlx::query(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages \
             WHERE conversation_id = $1 AND created_at > $2 AND COALESCE(is_private, 0) = 0 \
             ORDER BY created_at ASC LIMIT $3"
        ))
        .bind(conversation_id)
        .bind(&after_cursor)
        .bind(limit.clamp(1, MAX_POLL_LIMIT))
        .fetch_all(pool)
        .await,
        "reading visitor messages",
    )?;
    Ok(rows.iter().map(message_json).collect())
}

/// Every visible row, for a full transcript download at session close.
pub async fn build_transcript(pool: &DbPool, conversation_id: &str) -> WidgetResult<Vec<Value>> {
    let rows = need(
        sqlx::query(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages \
             WHERE conversation_id = $1 AND COALESCE(is_private, 0) = 0 ORDER BY created_at ASC"
        ))
        .bind(conversation_id)
        .fetch_all(pool)
        .await,
        "assembling the visitor transcript",
    )?;
    Ok(rows.iter().map(message_json).collect())
}

pub fn render_transcript(rows: &[Value]) -> String {
    let mut out = String::new();
    for row in rows {
        let direction = row.get("direction").and_then(|v| v.as_str()).unwrap_or("");
        let who = row
            .get("senderName")
            .and_then(|v| v.as_str())
            .unwrap_or(if direction == "outbound" { "Agent" } else { "You" });
        let when = row.get("sentAt").and_then(|v| v.as_str()).unwrap_or("");
        let body = row.get("body").and_then(|v| v.as_str()).unwrap_or("");
        out.push_str(&format!("[{when}] {who}: {body}\n"));
    }
    out
}

/// Applies a read receipt only to rows the session was actually served: the
/// client states the cursor it consumed, anything ahead of it stays unread, and
/// inbound rows (the visitor's own) are never marked.
pub async fn mark_messages_read(
    pool: &DbPool,
    conversation_id: &str,
    up_to_cursor: String,
) -> WidgetResult<u64> {
    let result = need(
        sqlx::query(
            "UPDATE messages SET status = 'read', delivery_status = 'read' \
             WHERE conversation_id = $1 AND created_at <= $2 AND direction = 'outbound' \
               AND COALESCE(is_private, 0) = 0 \
               AND (status IS NULL OR status != 'read')",
        )
        .bind(conversation_id)
        .bind(&up_to_cursor)
        .execute(pool)
        .await,
        "applying read receipts",
    )?;
    Ok(result.rows_affected())
}

pub async fn unread_outbound_count(
    pool: &DbPool,
    conversation_id: &str,
    after_cursor: String,
) -> WidgetResult<i64> {
    let row = need(
        sqlx::query(
            "SELECT COUNT(*) AS total FROM messages WHERE conversation_id = $1 AND created_at > $2 \
             AND direction = 'outbound' AND COALESCE(is_private, 0) = 0",
        )
        .bind(conversation_id)
        .bind(&after_cursor)
        .fetch_optional(pool)
        .await,
        "counting unread messages",
    )?;
    Ok(row
        .and_then(|row| row.try_get::<i64, _>("total").ok())
        .unwrap_or(0))
}

pub async fn conversation_state(pool: &DbPool, conversation_id: &str) -> WidgetResult<Option<String>> {
    let row = need(
        sqlx::query("SELECT state FROM conversations WHERE id = $1")
            .bind(conversation_id)
            .fetch_optional(pool)
            .await,
        "reading conversation state",
    )?;
    Ok(row.and_then(|row| opt_text(&row, "state")).map(|s| s.to_lowercase()))
}

pub async fn conversation_contact(
    pool: &DbPool,
    conversation_id: &str,
) -> WidgetResult<Option<String>> {
    let row = need(
        sqlx::query("SELECT contact_id FROM conversations WHERE id = $1")
            .bind(conversation_id)
            .fetch_optional(pool)
            .await,
        "reading the conversation owner",
    )?;
    Ok(row.and_then(|row| opt_text(&row, "contact_id")))
}

pub async fn conversation_summary(pool: &DbPool, conversation_id: &str) -> WidgetResult<Option<Value>> {
    let row = need(
        sqlx::query(
            "SELECT id, contact_id, state, inbox_id, priority, csat_score, last_inbound_at, \
             updated_at FROM conversations WHERE id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(pool)
        .await,
        "reading the conversation",
    )?;
    Ok(row.map(|row| {
        json!({
            "id": text(&row, "id"),
            "contactId": opt_text(&row, "contact_id"),
            "state": opt_text(&row, "state"),
            "inboxId": opt_text(&row, "inbox_id"),
            "priority": opt_text(&row, "priority"),
            "csatScore": row.try_get::<Option<i64>, _>("csat_score").ok().flatten(),
            "lastInboundAt": opt_text(&row, "last_inbound_at"),
            "updatedAt": opt_text(&row, "updated_at"),
        })
    }))
}

// ---------------------------------------------------------------------------
// Widget sessions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WidgetSession {
    pub id: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub inbox_id: String,
    pub website_token: String,
    pub origin: String,
    pub client_ip: String,
    pub state: String,
    pub last_cursor: String,
    pub visitor_presence: String,
    pub last_presence_at: Option<String>,
    pub last_typing_at: Option<String>,
    pub closed_at: Option<String>,
    pub close_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

const SESSION_COLUMNS: &str = "id, conversation_id, contact_id, inbox_id, website_token, origin, \
     client_ip, state, last_cursor, visitor_presence, last_presence_at, last_typing_at, closed_at, \
     close_reason, created_at, updated_at";

fn row_to_session(row: &PgRow) -> WidgetSession {
    WidgetSession {
        id: text(row, "id"),
        conversation_id: text(row, "conversation_id"),
        contact_id: text(row, "contact_id"),
        inbox_id: opt_text(row, "inbox_id").unwrap_or_default(),
        website_token: opt_text(row, "website_token").unwrap_or_default(),
        origin: opt_text(row, "origin").unwrap_or_default(),
        client_ip: opt_text(row, "client_ip").unwrap_or_default(),
        state: opt_text(row, "state").unwrap_or_else(|| "open".to_string()),
        last_cursor: row
            .try_get::<Option<String>, _>("last_cursor")
            .ok()
            .flatten()
            .unwrap_or_default(),
        visitor_presence: opt_text(row, "visitor_presence")
            .unwrap_or_else(|| "unknown".to_string()),
        last_presence_at: opt_text(row, "last_presence_at"),
        last_typing_at: opt_text(row, "last_typing_at"),
        closed_at: opt_text(row, "closed_at"),
        close_reason: opt_text(row, "close_reason"),
        created_at: opt_text(row, "created_at").unwrap_or_default(),
        updated_at: opt_text(row, "updated_at").unwrap_or_default(),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_session(
    pool: &DbPool,
    conversation_id: &str,
    contact_id: &str,
    inbox_id: &str,
    website_token: &str,
    origin: &str,
    client_ip: &str,
) -> WidgetResult<(WidgetSession, String)> {
    let token = new_session_token();
    let token_hash = sha256_hex(&token);
    let id = Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let cursor = max_message_cursor(pool, conversation_id)
        .await
        .unwrap_or_default();

    need(
        sqlx::query(
            "INSERT INTO widget_sessions (id, token_hash, conversation_id, contact_id, inbox_id, \
             website_token, origin, client_ip, state, last_cursor, visitor_presence, \
             last_presence_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'open', $9, 'online', $10, $11, $12)",
        )
        .bind(&id)
        .bind(&token_hash)
        .bind(conversation_id)
        .bind(contact_id)
        .bind(inbox_id)
        .bind(website_token)
        .bind(origin)
        .bind(client_ip)
        .bind(&cursor)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await,
        "creating the widget session",
    )?;

    let session = WidgetSession {
        id,
        conversation_id: conversation_id.to_string(),
        contact_id: contact_id.to_string(),
        inbox_id: inbox_id.to_string(),
        website_token: website_token.to_string(),
        origin: origin.to_string(),
        client_ip: client_ip.to_string(),
        state: "open".to_string(),
        last_cursor: cursor,
        visitor_presence: "online".to_string(),
        last_presence_at: Some(now.clone()),
        last_typing_at: None,
        closed_at: None,
        close_reason: None,
        created_at: now.clone(),
        updated_at: now,
    };
    Ok((session, token))
}

pub async fn load_session_by_token(
    pool: &DbPool,
    token: &str,
) -> WidgetResult<Option<WidgetSession>> {
    if token.trim().is_empty() {
        return Err(WidgetError::SessionTokenMissing);
    }
    let token_hash = sha256_hex(token.trim());
    let row = need(
        sqlx::query(&format!(
            "SELECT {SESSION_COLUMNS} FROM widget_sessions WHERE token_hash = $1 LIMIT 1"
        ))
        .bind(&token_hash)
        .fetch_optional(pool)
        .await,
        "loading the widget session",
    )?;
    Ok(row.map(|row| row_to_session(&row)))
}

pub async fn load_open_session_for_conversation(
    pool: &DbPool,
    conversation_id: &str,
) -> WidgetResult<Option<WidgetSession>> {
    let row = need(
        sqlx::query(&format!(
            "SELECT {SESSION_COLUMNS} FROM widget_sessions WHERE conversation_id = $1 \
             AND state = 'open' ORDER BY updated_at DESC LIMIT 1"
        ))
        .bind(conversation_id)
        .fetch_optional(pool)
        .await,
        "loading the widget session by conversation",
    )?;
    Ok(row.map(|row| row_to_session(&row)))
}

/// A session has to be open before it may write into the thread.
pub fn ensure_open(session: &WidgetSession) -> WidgetResult<()> {
    if session.state == "open" {
        return Ok(());
    }
    Err(WidgetError::SessionClosed(session.id.clone()))
}

/// Records how far the visitor has been served. `MAX` keeps it monotonic, so a
/// replayed older cursor cannot rewind the session.
pub async fn advance_session_cursor(
    pool: &DbPool,
    session_id: &str,
    cursor: &str,
) -> WidgetResult<()> {
    need(
        sqlx::query(
            "UPDATE widget_sessions SET last_cursor = GREATEST(last_cursor, $1), updated_at = $2 WHERE id = $3",
        )
        .bind(cursor)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(session_id)
        .execute(pool)
        .await,
        "advancing the widget session cursor",
    )?;
    Ok(())
}

pub async fn record_visitor_presence(
    pool: &DbPool,
    session_id: &str,
    conversation_id: &str,
    status: &str,
    page_url: &str,
) -> WidgetResult<String> {
    let now = chrono::Utc::now().to_rfc3339();
    let (presence, column) = match status {
        "typing" => ("typing", "last_typing_at"),
        "left" | "offline" => ("left", "last_presence_at"),
        "online" | "viewing" => ("online", "last_presence_at"),
        // Anything else is stored as reported instead of coerced into a state
        // nobody configured.
        other => (other, "last_presence_at"),
    };

    need(
        sqlx::query(&format!(
            "UPDATE widget_sessions SET visitor_presence = $1, {column} = $2, updated_at = $3 WHERE id = $4"
        ))
        .bind(presence)
        .bind(&now)
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await,
        "storing visitor presence",
    )?;

    if !page_url.is_empty() {
        merge_conversation_attributes(
            pool,
            conversation_id,
            json!({
                "last_page_url": page_url,
                "visitor_presence": presence,
                "last_presence_at": now,
            }),
        )
        .await?;
    }
    Ok(now)
}

/// Typing indicator. `None` means no session row matched.
pub async fn record_typing(
    pool: &DbPool,
    session_id: &str,
    is_typing: bool,
) -> WidgetResult<Option<String>> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = need(
        sqlx::query(
            "UPDATE widget_sessions SET last_typing_at = $1, visitor_presence = $2, updated_at = $3 \
             WHERE id = $4",
        )
        .bind(is_typing.then_some(now.clone()))
        .bind(if is_typing { "typing" } else { "online" })
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await,
        "storing the typing indicator",
    )?;
    if result.rows_affected() == 0 {
        return Ok(None);
    }
    Ok(is_typing.then_some(now))
}

pub async fn close_session(pool: &DbPool, session_id: &str, reason: &str) -> WidgetResult<bool> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = need(
        sqlx::query(
            "UPDATE widget_sessions SET state = 'closed', closed_at = $1, close_reason = $2, \
             visitor_presence = 'left', updated_at = $3 WHERE id = $4 AND state != 'closed'",
        )
        .bind(&now)
        .bind(reason)
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await,
        "closing the widget session",
    )?;
    Ok(result.rows_affected() > 0)
}

pub fn session_json(session: &WidgetSession) -> Value {
    json!({
        "sessionId": session.id,
        "conversationId": session.conversation_id,
        "contactId": session.contact_id,
        "inboxId": session.inbox_id,
        "state": session.state,
        "cursor": session.last_cursor.as_str(),
        "visitorPresence": session.visitor_presence,
        "lastPresenceAt": session.last_presence_at,
        "lastTypingAt": session.last_typing_at,
        "closedAt": session.closed_at,
        "closeReason": session.close_reason,
        "createdAt": session.created_at,
        "updatedAt": session.updated_at,
    })
}

// ---------------------------------------------------------------------------
// Presence: agents, last-seen, "are humans online"
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HumanPresence {
    /// "online" | "offline" | "unknown" -- unknown means the data does not
    /// exist, never a guessed positive.
    pub status: &'static str,
    pub reason: String,
    /// `None` when no roster table exists at all, distinct from `Some(0)`.
    pub agents_online: Option<i64>,
    pub agents_total: Option<i64>,
    pub typing_agents: Vec<String>,
    pub source: String,
    pub agent_typing: bool,
    pub last_agent_activity_at: Option<String>,
}

impl HumanPresence {
    /// Three-valued on purpose: `None` is "the database cannot tell us", which
    /// the widget renders as an unknown badge instead of a green dot.
    pub fn is_online(&self) -> Option<bool> {
        match self.status {
            "online" => Some(true),
            "offline" => Some(false),
            _ => None,
        }
    }

    pub fn json(&self) -> Value {
        json!({
            "status": self.status,
            "reason": self.reason,
            "agentsOnline": self.agents_online,
            "agentsTotal": self.agents_total,
            "typingAgents": self.typing_agents,
            "agentTyping": self.agent_typing,
            "lastAgentActivityAt": self.last_agent_activity_at,
            "source": self.source,
            "ttlSeconds": PRESENCE_TTL_SECONDS,
            "checkedAt": chrono::Utc::now().to_rfc3339(),
        })
    }
}

struct Heartbeat {
    agent_name: String,
    action: String,
    last_seen_at: String,
}

/// Fresh agent heartbeats from `agent_presence`. A missing table means no data,
/// not "nobody is online".
async fn active_heartbeats(pool: &DbPool, conversation_id: Option<&str>) -> WidgetResult<Vec<Heartbeat>> {
    let cutoff =
        (chrono::Utc::now() - chrono::Duration::seconds(PRESENCE_TTL_SECONDS)).to_rfc3339();
    let result = match conversation_id {
        Some(id) => sqlx::query(
            "SELECT agent_name, action, last_seen_at FROM agent_presence \
             WHERE conversation_id = $1 AND last_seen_at > $2 ORDER BY last_seen_at DESC",
        )
        .bind(id)
        .bind(&cutoff)
        .fetch_all(pool)
        .await,
        None => sqlx::query(
            "SELECT agent_name, action, last_seen_at FROM agent_presence WHERE last_seen_at > $1 \
             ORDER BY last_seen_at DESC",
        )
        .bind(&cutoff)
        .fetch_all(pool)
        .await,
    };
    let rows = match result {
        Ok(rows) => rows,
        // A missing heartbeat table means nobody has ever been seen; that is "no
        // data", which the caller reports as unknown, not "agents offline".
        Err(err) if schema_gap(&err).is_some() => return Ok(Vec::new()),
        Err(err) => {
            warn!("widget database error while reading agent presence: {err}");
            return Err(WidgetError::Database(err));
        }
    };
    Ok(rows
        .iter()
        .map(|row| Heartbeat {
            agent_name: text(row, "agent_name"),
            action: text(row, "action"),
            last_seen_at: text(row, "last_seen_at"),
        })
        .collect())
}

pub async fn record_agent_presence(
    pool: &DbPool,
    conversation_id: &str,
    agent_name: &str,
    agent_id: Option<&str>,
    agent_avatar: Option<&str>,
    action: &str,
) -> WidgetResult<Value> {
    let now = chrono::Utc::now().to_rfc3339();
    if action == "left" {
        need(
            sqlx::query("DELETE FROM agent_presence WHERE conversation_id = $1 AND agent_name = $2")
                .bind(conversation_id)
                .bind(agent_name)
                .execute(pool)
                .await,
            "clearing agent presence",
        )?;
        return Ok(json!({ "action": "left", "lastSeenAt": now }));
    }

    need(
        sqlx::query(
            "INSERT INTO agent_presence (id, conversation_id, agent_id, agent_name, agent_avatar, \
             action, last_seen_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT(conversation_id, agent_name) DO UPDATE SET \
             action = excluded.action, last_seen_at = excluded.last_seen_at, updated_at = excluded.updated_at",
        )
        .bind(Uuid::new_v4().simple().to_string())
        .bind(conversation_id)
        .bind(agent_id)
        .bind(agent_name)
        .bind(agent_avatar)
        .bind(action)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await,
        "storing agent presence",
    )?;
    Ok(json!({ "action": action, "agentName": agent_name, "lastSeenAt": now }))
}

enum WorkingHours {
    Open,
    Closed(String),
    /// A timezone we cannot resolve is not evidence that anyone is available.
    Unknown(String),
}

fn working_hours_state(inbox: &WebsiteInbox) -> WorkingHours {
    let timezone = inbox.timezone.clone().unwrap_or_else(|| "UTC".to_string());
    if !matches!(timezone.to_uppercase().as_str(), "UTC" | "GMT") {
        return WorkingHours::Unknown(format!(
            "working hours are configured for timezone '{timezone}', which this backend cannot \
             resolve (no timezone database dependency); availability is unknown"
        ));
    }
    let (Some(start), Some(end)) = (inbox.working_hours_start, inbox.working_hours_end) else {
        return WorkingHours::Unknown(
            "working hours are enabled but neither a start nor an end hour was ever saved, so \
             availability is unknown"
                .to_string(),
        );
    };
    let now: i64 = match chrono::Utc::now().format("%H").to_string().parse() {
        Ok(hour) => hour,
        Err(_) => {
            return WorkingHours::Unknown("the current UTC hour could not be read".to_string())
        }
    };
    if now >= start && now < end {
        WorkingHours::Open
    } else {
        let tz_label = inbox.timezone.clone().unwrap_or_else(|| "UTC (not saved)".to_string());
        WorkingHours::Closed(format!(
            "outside the saved working hours {start}:00-{end}:00, compared against {tz_label}"
        ))
    }
}

/// Real "are humans online": inbox state first (a disabled channel or an
/// out-of-office window is decisive), then live heartbeats, then the stored
/// agent roster. `unknown` whenever none of those exist in the database.
pub async fn compute_human_presence(
    pool: &DbPool,
    inbox: &WebsiteInbox,
    conversation_id: Option<&str>,
) -> HumanPresence {
    let heartbeats = active_heartbeats(pool, conversation_id)
        .await
        .unwrap_or_default();
    let typing_agents: Vec<String> = heartbeats
        .iter()
        .filter(|beat| beat.action == "typing")
        .map(|beat| beat.agent_name.clone())
        .collect();
    let last_activity_at = heartbeats
        .iter()
        .map(|beat| beat.last_seen_at.clone())
        .max();
    let agents_are_typing = !typing_agents.is_empty();

    if !inbox.is_active {
        return HumanPresence {
            status: "offline",
            reason: format!("live chat inbox '{}' is switched off in the database", inbox.name),
            agents_online: None,
            agents_total: None,
            typing_agents: Vec::new(),
            source: "inboxes.is_active".to_string(),
            agent_typing: false,
            last_agent_activity_at: last_activity_at,
        };
    }

    if inbox.out_of_office_enabled {
        return HumanPresence {
            status: "offline",
            reason: match inbox.out_of_office_message.as_deref() {
                Some(message) => message.to_string(),
                None => "out of office is enabled on this channel".to_string(),
            },
            agents_online: None,
            agents_total: None,
            typing_agents: Vec::new(),
            source: "inboxes.out_of_office_enabled".to_string(),
            agent_typing: false,
            last_agent_activity_at: last_activity_at,
        };
    }

    // Working hours are only authoritative when the channel switched them on.
    // A timezone we cannot resolve is reported as unknown, never as "closed".
    if inbox.working_hours_enabled {
        let closed_state: Option<(&'static str, String)> = match working_hours_state(inbox) {
            WorkingHours::Open => None,
            WorkingHours::Closed(reason) => Some(("offline", reason)),
            WorkingHours::Unknown(reason) => Some(("unknown", reason)),
        };
        if let Some((status, reason)) = closed_state {
            return HumanPresence {
                status,
                reason,
                agents_online: None,
                agents_total: None,
                typing_agents,
                source: "inboxes.working_hours".to_string(),
                agent_typing: agents_are_typing,
                last_agent_activity_at: last_activity_at,
            };
        }
    }

    // Heartbeats outrank the static roster: an agent seen in the last 30s is
    // online whatever their saved availability says.
    if !heartbeats.is_empty() {
        return HumanPresence {
            status: "online",
            reason: format!(
                "{} agent heartbeat(s) within {}s",
                heartbeats.len(),
                PRESENCE_TTL_SECONDS
            ),
            agents_online: Some(heartbeats.len() as i64),
            agents_total: None,
            typing_agents,
            source: "agent_presence".to_string(),
            agent_typing: agents_are_typing,
            last_agent_activity_at: last_activity_at,
        };
    }

    let roster = sqlx::query(
        "SELECT availability_status, COUNT(*) AS total FROM team_members GROUP BY availability_status",
    )
    .fetch_all(pool)
    .await;
    let rows = match roster {
        Ok(rows) => rows,
        Err(err) => {
            let reason = match schema_gap(&err) {
                Some(object) => {
                    format!("no agent roster in the database (`{object}` is missing)")
                }
                None => format!("the agent roster could not be read: {err}"),
            };
            warn!("presence falling back to unknown: {reason}");
            return HumanPresence {
                status: "unknown",
                reason,
                agents_online: None,
                agents_total: None,
                typing_agents,
                source: "none".to_string(),
                agent_typing: false,
                last_agent_activity_at: last_activity_at,
            };
        }
    };

    let mut online = 0i64;
    let mut total = 0i64;
    for row in &rows {
        let status = text(row, "availability_status").to_lowercase();
        let count: i64 = row.try_get("total").unwrap_or(0);
        total += count;
        if matches!(status.as_str(), "online" | "busy" | "available") {
            online += count;
        }
    }

    if total == 0 {
        return HumanPresence {
            status: "unknown",
            reason: "no team members are stored, so availability cannot be determined".to_string(),
            agents_online: Some(0),
            agents_total: Some(0),
            typing_agents,
            source: "team_members".to_string(),
            agent_typing: false,
            last_agent_activity_at: last_activity_at,
        };
    }

    HumanPresence {
        status: if online > 0 { "online" } else { "offline" },
        reason: format!(
            "{online} of {total} agent(s) available per team_members.availability_status"
        ),
        agents_online: Some(online),
        agents_total: Some(total),
        typing_agents,
        source: "team_members".to_string(),
        agent_typing: agents_are_typing,
        last_agent_activity_at: last_activity_at,
    }
}

/// Last moment the visitor's own thread saw inbound or outbound traffic, read
/// from the conversation row rather than from a handler clock.
pub async fn last_seen_in_thread(pool: &DbPool, conversation_id: &str) -> WidgetResult<Option<String>> {
    let row = need(
        sqlx::query(
            "SELECT MAX(COALESCE(last_inbound_at, updated_at)) AS last_at FROM conversations WHERE id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(pool)
        .await,
        "reading the last activity time",
    )?;
    Ok(row.and_then(|row| opt_text(&row, "last_at")))
}

// ---------------------------------------------------------------------------
// CSAT
// ---------------------------------------------------------------------------

/// Records the rating on the conversation itself (`conversations.csat_score`
/// and `csat_comment`), the columns the reporting routes already aggregate.
pub async fn record_csat(
    pool: &DbPool,
    conversation_id: &str,
    rating: i64,
    comment: &str,
) -> WidgetResult<i64> {
    if !(1..=5).contains(&rating) {
        return Err(WidgetError::RatingOutOfRange(rating));
    }
    let now = chrono::Utc::now().to_rfc3339();
    need(
        sqlx::query(
            "UPDATE conversations SET csat_score = $1, csat_comment = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(rating)
        .bind((!comment.is_empty()).then_some(comment))
        .bind(&now)
        .bind(conversation_id)
        .execute(pool)
        .await,
        "recording the csat rating",
    )
    .map(|result| result.rows_affected() as i64)
}
