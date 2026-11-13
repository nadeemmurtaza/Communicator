//! Customer-facing live-chat widget runtime.
//!
//! Ports `src/communicator/api/routes/widget.py` (boot config, contact/session
//! boot, messaging, history, presence, pre-chat form and intake) and adds the
//! session lifecycle the widget needs: real sessions, real cursors, real read
//! receipts, real presence.
//!
//! Deliberate divergences from Python, all honesty fixes:
//!   * no token fallback to `inboxes[0]` -- a wrong token is a 404, not whatever
//!     channel happens to be first in the table;
//!   * no hardcoded `isOnline: true` / `agentOnline: true` -- presence comes from
//!     `inboxes`, `agent_presence` and `team_members`, and is reported as
//!     `unknown` (JSON `null`) when those rows do not exist;
//!   * no fabricated `guest_<hex>@livechat.local` emails and no invented
//!     `bayworks.com` domain;
//!   * no invented assistant text -- `send_message` runs the ported reply
//!     engine and reports what it really did (`autoReplyState`: generated /
//!     stored / pushed, with the service's own refusal text);

use crate::db::AppState;
use crate::services::widget_support::{self as widget, WidgetSession};
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::warn;

pub type WidgetResponse = Result<Response, widget::WidgetError>;

// ---------------------------------------------------------------------------
// Plumbing
// ---------------------------------------------------------------------------

impl IntoResponse for widget::WidgetError {
    fn into_response(self) -> Response {
        use widget::WidgetError as E;
        let status = match &self {
            E::NoWebsiteInbox
            | E::TokenNotConfigured
            | E::IdentityValidationUnconfigured
            | E::SchemaUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            E::TokenMismatch | E::NoSessionForConversation | E::ConversationNotFound(_) => {
                StatusCode::NOT_FOUND
            }
            E::WidgetTokenMissing | E::EmptyMessage => StatusCode::BAD_REQUEST,
            E::OriginDenied(_) | E::IdentityMismatch | E::SessionConversationMismatch => {
                StatusCode::FORBIDDEN
            }
            E::MissingVisitorIdentity | E::RatingOutOfRange(_) | E::Validation(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            E::MessageTooLong(_) => StatusCode::PAYLOAD_TOO_LARGE,
            E::SessionTokenMissing | E::SessionUnknown => StatusCode::UNAUTHORIZED,
            E::SessionClosed(_)
            | E::ClosedConversation(_)
            | E::CsatDisabled
            | E::PreChatDisabled => StatusCode::CONFLICT,
            E::Database(_) | E::Inbound(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = match &self {
            E::Validation(errors) => json!({
                "detail": {
                    "message": "Pre-chat form validation failed",
                    "errors": errors,
                },
                "code": self.code(),
            }),
            E::SchemaUnavailable(object) => json!({
                "detail": self.message(),
                "code": self.code(),
                "missing": object,
            }),
            other => json!({ "detail": other.message(), "code": other.code() }),
        };

        (status, Json(body)).into_response()
    }
}

fn insert_header(response: &mut Response, name: &str, value: String) {
    let name = match axum::http::HeaderName::try_from(name) {
        Ok(name) => name,
        Err(_) => return,
    };
    if let Ok(value) = value.parse::<HeaderValue>() {
        response.headers_mut().insert(name, value);
    }
}

fn build(body: Value, set_cookie: Option<String>, cursor: Option<String>) -> Response {
    let mut response = (StatusCode::OK, Json(body)).into_response();
    if let Some(cookie) = set_cookie {
        if let Ok(value) = cookie.parse::<HeaderValue>() {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    if let Some(ref cursor) = cursor {
        insert_header(&mut response, "x-widget-cursor", cursor.clone());
    }
    response
}

/// Session cookie. A widget in a cross-site iframe only gets its cookies back
/// when `SameSite=None` is paired with `Secure`, so the flag follows the request
/// scheme instead of being hardcoded; the token is also returned in the body for
/// clients that cannot store cookies.
fn session_cookie(token: &str, headers: &HeaderMap) -> String {
    let over_https = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("https"))
        .unwrap_or_else(|| {
            widget::origin_header(headers)
                .map(|origin| origin.starts_with("https://"))
                .unwrap_or(false)
        });
    format!(
        "{cookie}={token}; Path=/; HttpOnly; SameSite={same_site}{secure}",
        cookie = widget::SESSION_COOKIE,
        same_site = if over_https { "None" } else { "Lax" },
        secure = if over_https { "; Secure" } else { "" },
    )
}

fn resolve_token(headers: &HeaderMap, query_or_body: Option<&str>) -> Result<String, widget::WidgetError> {
    let header_token = headers
        .get("x-widget-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    header_token.or_else(|| {
        query_or_body
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
    .ok_or(widget::WidgetError::WidgetTokenMissing)
}

/// The Python router only echoed `allowedDomains` back to the client; here it is
/// enforced. A request with no Origin header (curl, native shells, server-to-
/// server) is not punished for a browser rule, and an inbox with no list at all
/// has nothing to enforce -- both are logged rather than passed silently.
fn guard_origin(headers: &HeaderMap, inbox: &widget::WebsiteInbox) -> Result<(), widget::WidgetError> {
    match widget::check_origin(
        widget::origin_header(headers).as_deref(),
        widget::allowed_domains_of(inbox).as_deref(),
    ) {
        widget::OriginDecision::Allowed | widget::OriginDecision::NoOrigin => Ok(()),
        widget::OriginDecision::NotConfigured => {
            warn!(
                "widget request for inbox `{}` has no allowedDomains rule to enforce",
                inbox.id
            );
            Ok(())
        }
        widget::OriginDecision::Denied(origin) => {
            warn!("widget request from origin `{origin}` rejected for inbox `{}`", inbox.id);
            Err(widget::WidgetError::OriginDenied(origin))
        }
    }
}

/// Chatwoot-style `identityValidation`. When the channel switches it on but no
/// secret is stored, the request is refused: an empty secret never means accept.
fn guard_identity(
    inbox: &widget::WebsiteInbox,
    configured_secret: &str,
    email: &str,
    provided_hash: Option<&str>,
) -> Result<(), widget::WidgetError> {
    if !widget::identity_validation_enabled(inbox) {
        return Ok(());
    }
    let Some(secret) = widget::resolve_hmac_secret(inbox, configured_secret) else {
        warn!(
            "inbox `{}` enables widget identity validation but has no hmacSecret configured",
            inbox.id
        );
        return Err(widget::WidgetError::IdentityValidationUnconfigured);
    };
    if widget::verify_identity_hash(&secret, email, provided_hash) {
        Ok(())
    } else {
        Err(widget::WidgetError::IdentityMismatch)
    }
}

async fn require_session(
    state: &AppState,
    headers: &HeaderMap,
    body_token: Option<&str>,
) -> Result<WidgetSession, widget::WidgetError> {
    let token = widget::session_token_from(headers, body_token)
        .ok_or(widget::WidgetError::SessionTokenMissing)?;
    widget::load_session_by_token(&state.db, &token)
        .await?
        .ok_or(widget::WidgetError::SessionUnknown)
}

/// Binds a session token to the conversation it opened, so a valid token cannot
/// read or write a neighbour's thread. Returns the stored conversation state.
async fn require_thread(
    state: &AppState,
    session: &WidgetSession,
    conversation_id: &str,
) -> Result<String, widget::WidgetError> {
    if session.conversation_id != conversation_id {
        return Err(widget::WidgetError::SessionConversationMismatch);
    }
    widget::conversation_state(&state.db, conversation_id)
        .await?
        .ok_or_else(|| widget::WidgetError::ConversationNotFound(conversation_id.to_string()))
}

fn thread_is_terminal(state: &str) -> bool {
    matches!(state, "closed_won" | "closed_lost" | "opted_out")
}

/// Inbox for a public (token-authenticated) call, falling back to the inbox the
/// visitor's own session was opened against.
async fn inbox_for_call(
    state: &AppState,
    headers: &HeaderMap,
    token: Option<&str>,
    session: Option<&WidgetSession>,
) -> Result<widget::WebsiteInbox, widget::WidgetError> {
    let effective = match resolve_token(headers, token) {
        Ok(explicit) => explicit,
        Err(_) => session
            .map(|session| session.website_token.clone())
            .filter(|value| !value.is_empty())
            .ok_or(widget::WidgetError::WidgetTokenMissing)?,
    };
    widget::inbox_for_token(&state.db, &effective).await
}

/// The Python pre-chat GET contract, built from the stored form definition.
fn prechat_shape(cfg: &Value) -> Value {
    json!({
        "enabled": cfg.get("enabled").and_then(|value| value.as_bool()).unwrap_or(false),
        "title": cfg.get("title"),
        "description": cfg.get("description"),
        "submit_button_text": cfg.get("submit_button_text"),
        "require_email": cfg
            .get("require_email")
            .and_then(|value| value.as_bool())
            .unwrap_or(true),
        "fields": cfg.get("fields").cloned().unwrap_or_else(|| json!([])),
    })
}

fn served_cursor(rows: &[Value], fallback: &str) -> String {
    rows.iter()
        .filter_map(|row| row.get("cursor").and_then(|value| value.as_str()))
        .max()
        .map(|s| s.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn last_cursor_token(rows: &[Value], served: &str) -> String {
    rows.iter()
        .rev()
        .find_map(|row| row.get("cursorToken").and_then(|value| value.as_str()))
        .map(str::to_string)
        .unwrap_or_else(|| served.to_string())
}

// ---------------------------------------------------------------------------
// Request shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    #[serde(alias = "website_token")]
    pub token: Option<String>,
    #[serde(alias = "conversation_id")]
    pub conversation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptQuery {
    pub format: Option<String>,
    #[serde(alias = "session_token", alias = "sessionToken")]
    pub session: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
    #[serde(alias = "session_token", alias = "sessionToken")]
    pub session: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionQuery {
    #[serde(alias = "session_token", alias = "sessionToken")]
    pub session: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContactIn {
    #[serde(alias = "websiteToken")]
    pub website_token: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    #[serde(alias = "identifierHash", alias = "user_hash", alias = "userHash")]
    pub identifier_hash: Option<String>,
    #[serde(alias = "customAttributes")]
    pub custom_attributes: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct PreChatSubmitIn {
    #[serde(alias = "websiteToken")]
    pub website_token: Option<String>,
    #[serde(default)]
    pub answers: Value,
    #[serde(alias = "identifierHash", alias = "user_hash", alias = "userHash")]
    pub identifier_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessageIn {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    #[serde(default)]
    pub body: String,
    #[serde(alias = "sessionToken", alias = "session_token")]
    pub session: Option<String>,
    #[serde(alias = "pageUrl")]
    pub page_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PresenceIn {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(alias = "pageUrl")]
    pub page_url: Option<String>,
    #[serde(alias = "sessionToken", alias = "session_token")]
    pub session: Option<String>,
}

fn default_status() -> String {
    "online".to_string()
}

#[derive(Debug, Deserialize)]
pub struct TypingIn {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    #[serde(
        alias = "is_typing",
        alias = "typing",
        default = "default_true",
        deserialize_with = "coerce_bool"
    )]
    pub is_typing: bool,
    #[serde(alias = "sessionToken", alias = "session_token")]
    pub session: Option<String>,
}

fn default_true() -> bool {
    true
}

/// The widget builds send this as `true`, `"true"` and `1`; a dropped typing
/// flag is a visible regression, so all three are accepted.
fn coerce_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Flag {
        Bool(bool),
        Number(i64),
        Text(String),
    }
    Ok(match Flag::deserialize(deserializer)? {
        Flag::Bool(value) => value,
        Flag::Number(value) => value != 0,
        Flag::Text(value) => {
            let value = value.trim().to_lowercase();
            value == "true" || value == "1" || value == "yes"
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct ReadIn {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    #[serde(alias = "upToCursor", alias = "up_to_cursor", alias = "cursor")]
    pub up_to_cursor: Option<String>,
    #[serde(alias = "sessionToken", alias = "session_token")]
    pub session: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RateIn {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    pub rating: i64,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(alias = "sessionToken", alias = "session_token")]
    pub session: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CloseIn {
    #[serde(alias = "conversationId")]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(alias = "sessionToken", alias = "session_token")]
    pub session: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /widget/config -- widget boot
// ---------------------------------------------------------------------------

pub async fn get_widget_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<TokenQuery>,
) -> WidgetResponse {
    let token = resolve_token(&headers, params.token.as_deref())?;
    let inbox = widget::inbox_for_token(&state.db, &token).await?;
    guard_origin(&headers, &inbox)?;

    let presence = widget::compute_human_presence(&state.db, &inbox, None).await;
    let styling = widget::styling(&inbox);

    let mut body = serde_json::Map::new();
    body.insert("inboxId".to_string(), json!(inbox.id));
    body.insert("name".to_string(), json!(inbox.name));
    body.insert(
        "websiteToken".to_string(),
        widget::cred_str(&inbox.credentials, &["websiteToken", "website_token"])
            .map(Value::from)
            .unwrap_or(json!(token)),
    );
    body.insert("channelType".to_string(), json!(inbox.channel_type));
    body.insert("isActive".to_string(), json!(inbox.is_active));
    if let Some(map) = styling.as_object() {
        for (key, value) in map {
            body.insert(key.clone(), value.clone());
        }
    }
    // Python returned a literal `true` here.
    body.insert(
        "isOnline".to_string(),
        match presence.is_online() {
            Some(online) => json!(online),
            None => Value::Null,
        },
    );
    body.insert("presence".to_string(), presence.json());
    body.insert("preChat".to_string(), prechat_shape(&widget::prechat_config(&inbox)));
    body.insert(
        "identityValidation".to_string(),
        json!(widget::identity_validation_enabled(&inbox)),
    );
    Ok(build(Value::Object(body), None, None))
}

// ---------------------------------------------------------------------------
// GET /widget/prechat + POST /widget/prechat/submit
// ---------------------------------------------------------------------------

pub async fn get_prechat_form(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<TokenQuery>,
) -> WidgetResponse {
    let token = resolve_token(&headers, params.token.as_deref())?;
    let inbox = widget::inbox_for_token(&state.db, &token).await?;
    guard_origin(&headers, &inbox)?;
    Ok(build(prechat_shape(&widget::prechat_config(&inbox)), None, None))
}

pub async fn submit_prechat_form(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PreChatSubmitIn>,
) -> WidgetResponse {
    let website_token = resolve_token(&headers, payload.website_token.as_deref())?;
    let inbox = widget::inbox_for_token(&state.db, &website_token).await?;
    guard_origin(&headers, &inbox)?;

    let answers: Value = match payload.answers {
        Value::Object(map) => Value::Object(map),
        _ => {
            warn!("pre-chat submission arrived without an answers object; validating an empty form");
            json!({})
        }
    };
    let cfg = widget::prechat_config(&inbox);
    if !cfg.get("enabled").and_then(|value| value.as_bool()).unwrap_or(false) {
        return Err(widget::WidgetError::PreChatDisabled);
    }

    // Ported required-field / email / select / number validation: 422 with the
    // per-field error list, same shape as the Python endpoint.
    let errors = widget::validate_prechat_answers(&cfg, &answers);
    if !errors.is_empty() {
        return Err(widget::WidgetError::Validation(errors));
    }

    // Python synthesised `guest_<hex>@livechat.local` when no email was given.
    // There is no such thing as an anonymous visitor row here: refuse instead.
    let email = widget::answer_str(&answers, &["email", "emailAddress", "visitor_email"]);
    if email.trim().is_empty() {
        return Err(widget::WidgetError::MissingVisitorIdentity);
    }
    let email = widget::normalize_email(&email);
    guard_identity(&inbox, &state.config.widget_identity_hmac_secret, &email, payload.identifier_hash.as_deref())?;

    let full_name = widget::answer_str(&answers, &["fullName", "name", "full_name"]);
    let phone = widget::answer_str(&answers, &["phone", "phoneNumber"]);
    let company = widget::answer_str(&answers, &["company", "companyName"]);
    let initial_message = widget::answer_str(&answers, &["message", "initialMessage", "question"]);
    let client_ip = widget::client_ip(&headers);
    let origin = widget::origin_header(&headers).unwrap_or_default();

    let tenant = widget::resolve_tenant(&state.db, &inbox, &state.config.default_tenant_key).await;
    let (contact_id, _contact_created) =
        widget::resolve_widget_contact(&state.db, &email, &full_name, &tenant, "prechat_intake_form")
            .await?;

    let routing = widget::evaluate_routing_rules(&cfg, &answers);
    let attributes = json!({
        "source": "website_live_chat",
        "visitor_email": email,
        "intake_submission": answers,
        "tags": routing.get("tags").cloned().unwrap_or_else(|| json!([])),
        "assigned_team": routing.get("team_target").cloned().unwrap_or(Value::Null),
        "client_ip": client_ip,
        "company": company,
        "phone": phone,
    });
    let priority = routing
        .get("priority")
        .and_then(|value| value.as_str())
        .unwrap_or("medium")
        .to_string();

    let (conversation_id, is_new) =
        widget::open_widget_conversation(&state.db, &contact_id, &inbox.id, &priority, attributes)
            .await?;
    let greeting_id = if is_new {
        widget::inject_greeting(&state.db, &inbox, &conversation_id).await?
    } else {
        None
    };

    // The visitor's initial question from the form is a genuine inbound row, not
    // a note on the conversation.
    let mut initial_message_id: Option<String> = None;
    let mut cursor = widget::max_message_cursor(&state.db, &conversation_id).await?;
    if !initial_message.is_empty() {
        let (id, rowid, _sent_at) = widget::record_visitor_message(
            &state.db,
            &conversation_id,
            &initial_message,
            if full_name.is_empty() { "Visitor" } else { &full_name },
            json!({
                "source": "prechat_intake",
                "websiteInboxId": inbox.id,
                "origin": origin,
                "clientIp": client_ip,
            }),
        )
        .await?;
        initial_message_id = Some(id);
        cursor = rowid.to_string();
    }

    let (session, session_token) = widget::create_session(
        &state.db,
        &conversation_id,
        &contact_id,
        &inbox.id,
        &website_token,
        &origin,
        &client_ip,
    )
    .await?;
    let (stored_name, stored_email) = widget::contact_profile(&state.db, &contact_id)
        .await?
        .unwrap_or_else(|| (full_name.clone(), email.clone()));

    let body = json!({
        "success": true,
        "contact_id": contact_id,
        "conversation_id": conversation_id,
        "visitor_name": stored_name,
        "visitor_email": stored_email,
        "routing_applied": routing,
        "initial_message_id": initial_message_id,
        "is_new_conversation": is_new,
        "greeting_message_id": greeting_id,
        "sessionToken": session_token,
        "session": widget::session_json(&session),
        "cursor": cursor,
    });
    Ok(build(body, Some(session_cookie(&session_token, &headers)), Some(cursor)))
}

// ---------------------------------------------------------------------------
// POST /widget/contact -- session boot
// ---------------------------------------------------------------------------

pub async fn register_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ContactIn>,
) -> WidgetResponse {
    let website_token = resolve_token(&headers, payload.website_token.as_deref())?;
    let inbox = widget::inbox_for_token(&state.db, &website_token).await?;
    guard_origin(&headers, &inbox)?;

    let email = widget::normalize_email(payload.email.as_deref().unwrap_or(""));
    if email.is_empty() {
        // Python filed a contact under whatever string arrived, including "";
        // an unidentifiable visitor cannot own a conversation row, so refuse.
        return Err(widget::WidgetError::MissingVisitorIdentity);
    }
    guard_identity(&inbox, &state.config.widget_identity_hmac_secret, &email, payload.identifier_hash.as_deref())?;

    let client_ip = widget::client_ip(&headers);
    let origin = widget::origin_header(&headers).unwrap_or_default();
    let display_name = payload.name.clone().unwrap_or_default();
    let tenant = widget::resolve_tenant(&state.db, &inbox, &state.config.default_tenant_key).await;
    let (contact_id, contact_created) =
        widget::resolve_widget_contact(&state.db, &email, &display_name, &tenant, "website_live_chat")
            .await?;

    let mut attributes = serde_json::Map::new();
    attributes.insert("source".to_string(), json!("website_live_chat"));
    attributes.insert("visitor_email".to_string(), json!(email));
    attributes.insert("client_ip".to_string(), json!(client_ip));
    if let Some(phone) = payload.phone.filter(|value| !value.trim().is_empty()) {
        attributes.insert("phone".to_string(), json!(phone));
    }
    if let Some(Value::Object(extra)) = payload.custom_attributes.clone() {
        for (key, value) in extra {
            attributes.insert(key, value);
        }
    }

    let (conversation_id, is_new) = widget::open_widget_conversation(
        &state.db,
        &contact_id,
        &inbox.id,
        // Python's default for a hand-opened widget thread; pre-chat routing
        // rules override this when they match.
        "medium",
        Value::Object(attributes),
    )
    .await?;
    let greeting_id = if is_new {
        widget::inject_greeting(&state.db, &inbox, &conversation_id).await?
    } else {
        None
    };

    let (session, session_token) = widget::create_session(
        &state.db,
        &conversation_id,
        &contact_id,
        &inbox.id,
        &website_token,
        &origin,
        &client_ip,
    )
    .await?;
    let (stored_name, stored_email) = widget::contact_profile(&state.db, &contact_id)
        .await?
        .unwrap_or_else(|| (display_name, email.clone()));
    let presence = widget::compute_human_presence(&state.db, &inbox, Some(&conversation_id)).await;

    let body = json!({
        "contactId": contact_id,
        "conversationId": conversation_id,
        "name": stored_name,
        "email": stored_email,
        "isNewConversation": is_new,
        "isNewContact": contact_created,
        "greetingMessageId": greeting_id,
        "sessionToken": session_token,
        "session": widget::session_json(&session),
        "cursor": session.last_cursor,
        "agentOnline": presence.is_online(),
        "presence": presence.json(),
    });
    Ok(build(body, Some(session_cookie(&session_token, &headers)), Some(session.last_cursor)))
}

// ---------------------------------------------------------------------------
// POST /widget/messages
// ---------------------------------------------------------------------------

pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MessageIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    widget::ensure_open(&session)?;
    let thread_state = require_thread(&state, &session, &payload.conversation_id).await?;
    if thread_is_terminal(&thread_state) {
        return Err(widget::WidgetError::ClosedConversation(thread_state));
    }

    let inbox = match widget::inbox_for_token(&state.db, &session.website_token).await {
        Ok(inbox) => {
            guard_origin(&headers, &inbox)?;
            Some(inbox)
        }
        // The session was opened against a channel that has since been removed
        // or retokened: the message still belongs to the thread, so store it and
        // warn instead of dropping the visitor's text.
        Err(widget::WidgetError::NoWebsiteInbox) | Err(widget::WidgetError::TokenMismatch) => {
            warn!(
                "widget message for session {} cannot be re-matched to an inbox; origin guard skipped",
                session.id
            );
            None
        }
        Err(err) => return Err(err),
    };

    let body = payload.body.trim().to_string();
    if body.is_empty() {
        return Err(widget::WidgetError::EmptyMessage);
    }
    if body.chars().count() > widget::MAX_BODY_CHARS {
        return Err(widget::WidgetError::MessageTooLong(body.chars().count()));
    }

    let (stored_name, _email) = widget::contact_profile(&state.db, &session.contact_id)
        .await?
        .unwrap_or_default();
    let sender_name = if stored_name.is_empty() { "Visitor" } else { stored_name.as_str() };

    let (message_id, cursor, sent_at) = widget::record_visitor_message(
        &state.db,
        &session.conversation_id,
        &body,
        sender_name,
        json!({
            "source": "website_live_chat",
            "sessionId": session.id,
            "websiteInboxId": session.inbox_id,
            "origin": widget::origin_header(&headers).unwrap_or_default(),
            "clientIp": widget::client_ip(&headers),
            "pageUrl": payload.page_url.unwrap_or_default(),
        }),
    )
    .await?;
    let cursor = cursor.to_string();

    // A visitor writing back into a resolved thread reopens it, matching the
    // shared inbound pipeline's idea of a non-terminal conversation.
    if thread_state == "resolved" {
        // WHY: the stored message and the reply claim below stand whether or not
        // this bookkeeping write lands, but the log must not say the thread was
        // reopened when it was not.
        let reopened = crate::db::report_write(
            sqlx::query(
                "UPDATE conversations SET state = 'engaged', resolution_type = NULL, updated_at = ? \
                 WHERE id = ?",
            )
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(&session.conversation_id)
            .execute(&state.db)
            .await
            .map(|report| report.rows_affected()),
            "widget reopening a resolved conversation on a visitor reply",
        );
        if reopened {
            warn!(
                "widget visitor replied on resolved conversation {}; state reopened",
                session.conversation_id
            );
        } else {
            warn!(
                "widget visitor replied on resolved conversation {}; the message is stored, but the \
                 conversation state was NOT reopened and is still 'resolved'",
                session.conversation_id
            );
        }
    }

    widget::advance_session_cursor(&state.db, &session.id, &cursor).await?;

    // The auto-reply is the ported AI engine's own output, and its three
    // outcomes are reported separately because they fail separately: the model
    // can answer while the draft gate leaves it for a human, and a reply can be
    // stored for polling without anything reaching a provider.
    let auto_reply =
        widget::request_auto_reply(&state, &session.conversation_id, &message_id).await;
    for warning in &auto_reply.warnings {
        warn!(
            "widget session {} / conversation {}: auto-reply warning: {warning}",
            session.id, session.conversation_id
        );
    }
    let status = auto_reply.thread_status();
    let auto_reply_flag = auto_reply.stored;
    let reply = auto_reply.message.clone().unwrap_or(Value::Null);

    let presence = match &inbox {
        Some(inbox) => Some(
            widget::compute_human_presence(&state.db, inbox, Some(&session.conversation_id)).await,
        ),
        None => None,
    };

    let body_json = json!({
        "id": message_id,
        "conversationId": session.conversation_id,
        "body": body,
        "sentAt": sent_at,
        "senderType": "customer",
        "senderName": sender_name,
        "cursor": cursor,
        "cursorToken": format!("{cursor}:{sent_at}"),
        "status": status,
        "autoReply": auto_reply_flag,
        "auto_reply": auto_reply_flag,
        "reply": reply,
        "autoReplyState": auto_reply.to_json(),
        "agentTyping": presence.as_ref().map(|p| json!(p.agent_typing)).unwrap_or(Value::Null),
        "agentOnline": presence.as_ref().and_then(|p| p.is_online()),
    });
    Ok(build(body_json, None, Some(cursor)))
}

// ---------------------------------------------------------------------------
// History and polling
// ---------------------------------------------------------------------------

/// Full visitor-visible history (private internal notes are excluded by the
/// query). The response stays the Python array; the cursor travels in the
/// `X-Widget-Cursor` header so old clients are unaffected.
pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, params.session.as_deref()).await?;
    require_thread(&state, &session, &conversation_id).await?;

    let after = widget::cursor_rowid(params.cursor.as_deref());
    let limit = params.limit.unwrap_or(widget::MAX_POLL_LIMIT);
    let rows = widget::visible_messages(&state.db, &conversation_id, after.clone(), limit).await?;
    let served = served_cursor(&rows, &after);
    if served > session.last_cursor {
        widget::advance_session_cursor(&state.db, &session.id, &served).await?;
    }

    let mut response = (StatusCode::OK, Json(Value::Array(rows))).into_response();
    insert_header(&mut response, "x-widget-cursor", served.to_string());
    Ok(response)
}

/// Incremental poll: only rows past the supplied cursor. Nothing is marked read
/// here -- being served is not a read receipt.
pub async fn poll_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, params.session.as_deref()).await?;
    let thread_state = require_thread(&state, &session, &conversation_id).await?;

    let after = match params.cursor.as_deref() {
        Some(raw) => widget::cursor_rowid(Some(raw)),
        None => session.last_cursor.clone(),
    };
    let limit = params
        .limit
        .unwrap_or(widget::DEFAULT_POLL_LIMIT)
        .clamp(1, widget::MAX_POLL_LIMIT);

    // One extra row reveals whether more is pending, without a second query.
    let rows = widget::visible_messages(&state.db, &conversation_id, after.clone(), limit + 1).await?;
    let has_more = rows.len() as i64 > limit;
    let page: Vec<Value> = rows.into_iter().take(limit as usize).collect();
    let served = served_cursor(&page, &after);
    if served > session.last_cursor {
        widget::advance_session_cursor(&state.db, &session.id, &served).await?;
    }

    let inbox = widget::inbox_for_token(&state.db, &session.website_token).await.ok();
    let presence = match &inbox {
        Some(inbox) => Some(
            widget::compute_human_presence(&state.db, inbox, Some(&conversation_id))
                .await
                .json(),
        ),
        None => None,
    };
    let unread =
        widget::unread_outbound_count(&state.db, &conversation_id, session.last_cursor).await?;
    let thread_label: String = if thread_is_terminal(&thread_state) {
        "closed".to_string()
    } else {
        thread_state
    };

    Ok(build(
        json!({
            "messages": page,
            "cursor": served.clone(),
            "cursorToken": last_cursor_token(&page, &served),
            "hasMore": has_more,
            "conversationState": thread_label,
            "sessionState": session.state,
            "unreadAgentMessages": unread,
            "presence": presence,
        }),
        None,
        Some(served),
    ))
}

/// Applies a read receipt only to rows this session was actually served: the
/// claimed cursor is clamped to `widget_sessions.last_cursor`.
pub async fn mark_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReadIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    require_thread(&state, &session, &payload.conversation_id).await?;

    let claimed = payload
        .up_to_cursor
        .as_deref()
        .map(|raw| widget::cursor_rowid(Some(raw)))
        .unwrap_or(session.last_cursor.clone());
    let up_to = if claimed < session.last_cursor { claimed.clone() } else { session.last_cursor.clone() };
    let marked = widget::mark_messages_read(&state.db, &session.conversation_id, up_to.clone()).await?;

    Ok(build(
        json!({
            "status": "ok",
            "conversationId": session.conversation_id,
            "markedRead": marked,
            "cursor": up_to.clone(),
            "requested": claimed.clone(),
            "clamped": claimed > up_to,
        }),
        None,
        Some(up_to),
    ))
}

// ---------------------------------------------------------------------------
// Presence and typing
// ---------------------------------------------------------------------------

/// "Are humans online?" -- computed, never hardcoded.
pub async fn get_presence(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<TokenQuery>,
) -> WidgetResponse {
    let session = match widget::session_token_from(&headers, None) {
        Some(token) => widget::load_session_by_token(&state.db, &token).await?,
        None => None,
    };
    let inbox = inbox_for_call(&state, &headers, params.token.as_deref(), session.as_ref()).await?;
    guard_origin(&headers, &inbox)?;

    // Agent names are only ever revealed for the caller's own thread: an
    // anonymous request gets channel-level presence, and a `?conversationId=`
    // that is not the caller's session is ignored rather than honored.
    let conversation_id = session
        .as_ref()
        .map(|session| session.conversation_id.as_str())
        .filter(|owned| {
            params.conversation_id.is_none()
                || params.conversation_id.as_deref() == Some(*owned)
        });
    let presence = widget::compute_human_presence(&state.db, &inbox, conversation_id).await;
    let last_seen = match conversation_id {
        Some(id) => widget::last_seen_in_thread(&state.db, id).await?,
        None => None,
    };

    Ok(build(
        json!({
            "status": "ok",
            "agentOnline": presence.is_online(),
            "presence": presence.json(),
            "lastActivityAt": last_seen,
            "conversationId": conversation_id,
        }),
        None,
        None,
    ))
}

/// Visitor heartbeat. Python answered `{"agentOnline": true}` out of thin air;
/// this value comes from the same computation as `get_presence`.
pub async fn update_presence(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PresenceIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    widget::ensure_open(&session)?;
    require_thread(&state, &session, &payload.conversation_id).await?;

    let stored_at = widget::record_visitor_presence(
        &state.db,
        &session.id,
        &session.conversation_id,
        &payload.status,
        payload.page_url.as_deref().unwrap_or(""),
    )
    .await?;

    let presence = match widget::inbox_for_token(&state.db, &session.website_token).await {
        Ok(inbox) => Some(
            widget::compute_human_presence(&state.db, &inbox, Some(&session.conversation_id)).await,
        ),
        Err(err) => {
            warn!("visitor heartbeat has no inbox to compute agent presence against: {err}");
            None
        }
    };

    Ok(build(
        json!({
            "status": "ok",
            "agentOnline": presence.as_ref().and_then(|p| p.is_online()),
            "visitorPresence": payload.status,
            "lastPresenceAt": stored_at,
            "sessionId": session.id,
            "presence": presence.map(|p| p.json()),
        }),
        None,
        None,
    ))
}

pub async fn set_typing(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TypingIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    widget::ensure_open(&session)?;
    require_thread(&state, &session, &payload.conversation_id).await?;

    let last_typing_at = widget::record_typing(&state.db, &session.id, payload.is_typing).await?;
    Ok(build(
        json!({
            "status": "ok",
            "isTyping": payload.is_typing,
            "lastTypingAt": last_typing_at,
            "sessionId": session.id,
        }),
        None,
        None,
    ))
}

// ---------------------------------------------------------------------------
// Session lifecycle
// ---------------------------------------------------------------------------

pub async fn session_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<SessionQuery>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, params.session.as_deref()).await?;
    // WHY: a missing conversations row is not an empty thread state; defaulting
    // it to "" made `acceptsMessages` claim a thread the database does not have.
    let thread_state = require_thread(&state, &session, &session.conversation_id).await?;
    let conversation =
        widget::conversation_summary(&state.db, &session.conversation_id).await?;
    let unread =
        widget::unread_outbound_count(&state.db, &session.conversation_id, session.last_cursor.clone())
            .await?;
    let last_seen = widget::last_seen_in_thread(&state.db, &session.conversation_id).await?;

    Ok(build(
        json!({
            "session": widget::session_json(&session),
            "conversation": conversation,
            "conversationState": thread_state,
            "cursor": session.last_cursor,
            "unreadAgentMessages": unread,
            "lastActivityAt": last_seen,
            "acceptsMessages": session.state == "open" && !thread_is_terminal(&thread_state),
        }),
        None,
        Some(session.last_cursor),
    ))
}

async fn close_session_inner(
    state: &AppState,
    session: &WidgetSession,
    requested_reason: Option<&str>,
    default_reason: &str,
) -> Result<Value, widget::WidgetError> {
    let reason = requested_reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| default_reason.to_string());

    let already_closed = session.state != "open";
    let mut closed_at = session.closed_at.clone();
    if !already_closed {
        widget::close_session(&state.db, &session.id, &reason).await?;
        closed_at = Some(chrono::Utc::now().to_rfc3339());
    }
    let transcript = widget::build_transcript(&state.db, &session.conversation_id).await?;

    Ok(json!({
        "status": if already_closed { "already_closed" } else { "closed" },
        "sessionId": session.id,
        "conversationId": session.conversation_id,
        "closeReason": reason,
        "closedAt": closed_at,
        "messages": transcript.len(),
        "cursor": session.last_cursor,
        "transcriptUrl": "/api/v1/widget/transcript",
        "transcriptRequires": "the session token, in the X-Widget-Session header, the widget session cookie or ?session=",
    }))
}

pub async fn close_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CloseIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    if let Some(conversation_id) = payload.conversation_id.as_deref() {
        require_thread(&state, &session, conversation_id).await?;
    }
    let body = close_session_inner(&state, &session, payload.reason.as_deref(), "closed_by_visitor").await?;
    Ok(build(body, None, Some(session.last_cursor)))
}

pub async fn session_leave(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CloseIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    if let Some(conversation_id) = payload.conversation_id.as_deref() {
        require_thread(&state, &session, conversation_id).await?;
    }
    let body = close_session_inner(&state, &session, payload.reason.as_deref(), "visitor_left").await?;
    Ok(build(body, None, Some(session.last_cursor)))
}

/// The thread as the visitor saw it. `?format=text` renders the transcript the
/// "email me this chat" flow needs; the default JSON keeps per-row cursors.
pub async fn get_transcript(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<TranscriptQuery>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, params.session.as_deref()).await?;
    let rows = widget::build_transcript(&state.db, &session.conversation_id).await?;

    if params
        .format
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("text"))
        .unwrap_or(false)
    {
        return Ok((
            StatusCode::OK,
            [("content-type", "text/plain; charset=utf-8")],
            widget::render_transcript(&rows),
        )
            .into_response());
    }

    Ok(build(
        json!({
            "conversationId": session.conversation_id,
            "sessionId": session.id,
            "cursor": session.last_cursor,
            "messages": rows,
        }),
        None,
        Some(session.last_cursor),
    ))
}

// ---------------------------------------------------------------------------
// CSAT
// ---------------------------------------------------------------------------

pub async fn rate_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RateIn>,
) -> WidgetResponse {
    let session = require_session(&state, &headers, payload.session.as_deref()).await?;
    // Surveys are answered after the chat ends, so a closed session is fine; a
    // token for a different conversation is not.
    require_thread(&state, &session, &payload.conversation_id).await?;

    let inbox = widget::inbox_for_token(&state.db, &session.website_token)
        .await
        .ok();
    if let Some(inbox) = &inbox {
        guard_origin(&headers, inbox)?;
        if !inbox.csat_enabled {
            return Err(widget::WidgetError::CsatDisabled);
        }
    } else {
        warn!(
            "CSAT submitted for session {} but its inbox no longer resolves; rating will still be stored on the conversation",
            session.id
        );
    }

    let comment = payload.comment.clone().unwrap_or_default();
    let updated =
        widget::record_csat(&state.db, &session.conversation_id, payload.rating, comment.trim())
            .await?;
    if updated == 0 {
        return Err(widget::WidgetError::ConversationNotFound(
            session.conversation_id.clone(),
        ));
    }

    Ok(build(
        json!({
            "status": "recorded",
            "conversationId": session.conversation_id,
            "rating": payload.rating,
            "comment": comment.trim(),
            "inboxId": inbox.map(|inbox| inbox.id),
        }),
        None,
        None,
    ))
}
