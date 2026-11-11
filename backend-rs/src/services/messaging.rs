#![allow(dead_code)]

//! The only path allowed to put an outbound message on the wire.
//!
//! Port of `services/messaging.py`. The Python original wrote
//! `status = SENT` as soon as `provider.send()` returned without raising,
//! which -- because several Python adapters answered a failed or unconfigured
//! send with a fabricated `simulated: true` result -- meant rows claimed to be
//! SENT for messages that never left the process. That is impossible here:
//!
//! * a send is inserted as `queued` *before* the provider is contacted, so a
//!   crash mid-request leaves `queued`, never `sent`;
//! * the only write of `status = 'sent'` in this file is inside
//!   [`apply_send_result`], behind [`SendResult::is_accepted`], which requires
//!   a real 2xx from the provider AND `simulated == false`;
//! * every adapter returns [`SendResult`] rather than `Result<_, Error>`, so a
//!   caller cannot "forget" to handle the failure case.

use crate::db::{AppState, DbPool};
use crate::services::{business_hours, channels, compliance};
use serde_json::Value;
use sqlx::Row;

/// Did the provider actually take the message?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendStatus {
    /// The provider answered 2xx. Safe to mark SENT.
    Accepted,
    /// The provider answered and refused (bad token, 24h window closed,
    /// unknown recipient, opt-out honoured). Recorded FAILED with its reason.
    Rejected,
    /// Nothing left this process: credentials missing, channel not wired up,
    /// consent gate closed, business-hours gate closed. Recorded NOT_SENT.
    NotSent,
}

impl SendStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::NotSent => "not_sent",
        }
    }
}

/// What one channel send produced. Every adapter returns this, never a panic
/// and never a synthetic success.
#[derive(Clone, Debug)]
pub struct SendResult {
    pub provider: String,
    /// The id the provider handed back, or empty when that provider returns
    /// none (LINE's push API). Never a locally generated stand-in.
    pub provider_message_id: String,
    pub status: SendStatus,
    /// HTTP status the provider actually replied with, if we got a response.
    pub http_status: Option<u16>,
    /// The provider's own error text, or the field that is missing.
    pub error: Option<String>,
    /// True for anything that did not go out over the network.
    pub simulated: bool,
    /// Response body (or the refusal detail) kept for the audit trail.
    pub raw: Value,
}

impl SendResult {
    /// Only adapters that received a 2xx from the real provider may call this.
    pub fn accepted(
        provider: &str,
        provider_message_id: String,
        http_status: u16,
        raw: Value,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            provider_message_id,
            status: SendStatus::Accepted,
            http_status: Some(http_status),
            error: None,
            simulated: false,
            raw,
        }
    }

    /// The provider answered and said no.
    pub fn rejected(
        provider: &str,
        http_status: Option<u16>,
        error: String,
        raw: Value,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            provider_message_id: String::new(),
            status: SendStatus::Rejected,
            http_status,
            error: Some(error),
            // A refusal that reached the provider is not a simulation, but the
            // flag is what the callers check, so keep the two orthogonal and
            // let `is_accepted` fail on the status.
            simulated: false,
            raw,
        }
    }

    /// Nothing left the process. `reason` must name the cause (and the missing
    /// field) so the operator can fix it instead of guessing.
    pub fn not_sent(provider: &str, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            provider: provider.to_string(),
            provider_message_id: String::new(),
            status: SendStatus::NotSent,
            http_status: None,
            error: Some(reason.clone()),
            simulated: true,
            raw: Value::Object(
                serde_json::Map::from_iter([("not_sent_reason".to_string(), Value::String(reason))]),
            ),
        }
    }

    pub fn with_raw(mut self, raw: Value) -> Self {
        self.raw = raw;
        self
    }

    /// The single definition of "this send happened". `simulated` is checked
    /// even though no constructor sets it on an Accepted result: an adapter
    /// added later cannot reach SENT by lying about `status` alone.
    pub fn is_accepted(&self) -> bool {
        self.status == SendStatus::Accepted
            && !self.simulated
            && self.http_status.is_some_and(|code| (200..300).contains(&code))
    }

    /// Value written to `messages.status`.
    pub fn message_status(&self) -> &'static str {
        match self {
            s if s.is_accepted() => "sent",
            s if s.status == SendStatus::Rejected => "failed",
            _ => "not_sent",
        }
    }

    pub fn reason(&self) -> String {
        self.error
            .clone()
            .unwrap_or_else(|| "provider returned no error detail".to_string())
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "provider": self.provider,
            "providerMessageId": self.provider_message_id,
            "status": self.status.as_str(),
            "httpStatus": self.http_status,
            "error": self.error,
            "simulated": self.simulated,
            "delivered": self.is_accepted(),
        })
    }
}

#[derive(Debug)]
pub enum MessagingError {
    Db(sqlx::Error),
    NotFound(String),
}

impl From<sqlx::Error> for MessagingError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err)
    }
}

impl std::fmt::Display for MessagingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(err) => write!(f, "database error: {err}"),
            Self::NotFound(what) => write!(f, "{what}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SendOutcome {
    pub message_id: String,
    pub conversation_id: String,
    pub result: SendResult,
}

impl SendOutcome {
    pub fn was_sent(&self) -> bool {
        self.result.is_accepted()
    }
}

pub struct OutboundRequest<'a> {
    pub conversation_id: &'a str,
    pub contact_id: &'a str,
    pub contact_value: &'a str,
    pub channel: &'a str,
    pub body: &'a str,
    pub subject: Option<&'a str>,
    pub inbox_id: Option<&'a str>,
    pub tenant_key: Option<&'a str>,
    pub sender_name: &'a str,
    pub sender_type: &'a str,
    /// Internal note: recorded for the team, never handed to a provider.
    pub is_private: bool,
    /// Honour the tenant's business-hours gate before dialling the provider.
    pub respect_business_hours: bool,
    /// A provider template instead of free text -- broadcasts only. The wording
    /// comes from what the operator configured; nothing here composes it.
    pub template: Option<&'a channels::TemplateSpec>,
}

impl<'a> OutboundRequest<'a> {
    pub fn new(
        conversation_id: &'a str,
        contact_id: &'a str,
        contact_value: &'a str,
        channel: &'a str,
        body: &'a str,
    ) -> Self {
        Self {
            conversation_id,
            contact_id,
            contact_value,
            channel,
            body,
            subject: None,
            inbox_id: None,
            tenant_key: None,
            sender_name: "agent",
            sender_type: "agent",
            is_private: false,
            respect_business_hours: false,
            template: None,
        }
    }

    pub fn subject(mut self, value: Option<&'a str>) -> Self {
        self.subject = value;
        self
    }
    pub fn inbox_id(mut self, value: Option<&'a str>) -> Self {
        self.inbox_id = value;
        self
    }
    pub fn tenant_key(mut self, value: Option<&'a str>) -> Self {
        self.tenant_key = value;
        self
    }
    pub fn sender(mut self, name: &'a str, sender_type: &'a str) -> Self {
        self.sender_name = name;
        self.sender_type = sender_type;
        self
    }
    pub fn private_note(mut self, value: bool) -> Self {
        self.is_private = value;
        self
    }
    pub fn business_hours_gate(mut self, value: bool) -> Self {
        self.respect_business_hours = value;
        self
    }
    /// Send a configured provider template rather than free text.
    pub fn template(mut self, value: Option<&'a channels::TemplateSpec>) -> Self {
        self.template = value;
        self
    }
}

/// Consent gate, business-hours gate, queue row, provider call, receipt write.
///
/// Refusals are returned inside `SendOutcome.result` (status `NotSent`) rather
/// than as `Err`, because the caller still has to record the message row and
/// tell the operator which field is missing.
pub async fn send_outbound(
    state: &AppState,
    request: &OutboundRequest<'_>,
) -> Result<SendOutcome, MessagingError> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();
    let message_id = uuid::Uuid::new_v4().simple().to_string();

    // 1. Persist the intent first. A process that dies here leaves `queued`,
    //    which is the truth; nothing in this file can produce `sent` without a
    //    provider response.
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, direction, channel, body, status, \
         delivery_status, sender_type, sender_name, sender, is_private, created_at, sent_at) \
         VALUES (?, ?, 'outbound', ?, ?, 'queued', 'queued', ?, ?, ?, ?, ?, NULL)",
    )
    .bind(&message_id)
    .bind(request.conversation_id)
    .bind(request.channel)
    .bind(request.body)
    .bind(request.sender_type)
    .bind(request.sender_name)
    .bind(request.sender_type)
    .bind(request.is_private)
    .bind(&now)
    .execute(pool)
    .await?;

    if request.is_private {
        // An internal note is not a send: mark it as recorded, never SENT, and
        // do not touch a provider.
        let result = SendResult {
            provider: "internal".to_string(),
            provider_message_id: String::new(),
            status: SendStatus::NotSent,
            http_status: None,
            error: Some("private note: not dispatched to any provider".to_string()),
            simulated: false,
            raw: serde_json::json!({ "privateNote": true }),
        };
        sqlx::query("UPDATE messages SET status = 'internal_note', delivery_status = 'internal' WHERE id = ?")
            .bind(&message_id)
            .execute(pool)
            .await?;
        return Ok(SendOutcome {
            message_id,
            conversation_id: request.conversation_id.to_string(),
            result,
        });
    }

    // 2. Compliance gate -- a Contact without an explicit OPTED_IN record never
    //    gets their number dialled.
    if !compliance::is_sendable(pool, request.contact_id).await? {
        let status = compliance::get_consent_status(pool, request.contact_id).await?;
        let reason = format!(
            "Refusing to send: contact {} has no OPTED_IN consent record (current status {}).",
            request.contact_id,
            status.as_str()
        );
        let result = SendResult::not_sent(request.channel, reason);
        apply_send_result(pool, &message_id, &result).await?;
        return Ok(SendOutcome {
            message_id,
            conversation_id: request.conversation_id.to_string(),
            result,
        });
    }

    // 3. Optional business-hours gate; the away message is what the operator
    //    configured, so we report the deferral instead of silently sending.
    if request.respect_business_hours {
        // Tenant comes from the conversation's own row, then the configured
        // fallback tenant; nothing is assumed. An empty tenant means "single
        // deployment", which `check_tenant_hours` resolves from the one row.
        let tenant = request
            .tenant_key
            .filter(|value| !value.is_empty())
            .unwrap_or(state.config.default_tenant_key.as_str());
        let hours = business_hours::check_tenant_hours(state, tenant).await;
        if let business_hours::HoursVerdict::Closed { window, .. } = hours {
            let reason = format!(
                "Deferred: outside business hours ({window}). Set the away message or turn the \
                 business-hours gate off to send now."
            );
            let result = SendResult::not_sent(request.channel, reason);
            apply_send_result(pool, &message_id, &result).await?;
            return Ok(SendOutcome {
                message_id,
                conversation_id: request.conversation_id.to_string(),
                result,
            });
        }
    }

    // 4. The provider call itself.
    let ctx = channels::SendContext {
        destination: request.contact_value.to_string(),
        body: request.body.to_string(),
        subject: request.subject.map(str::to_string),
        inbox_id: request.inbox_id.map(str::to_string),
        template: request.template.cloned(),
    };
    let result = channels::send_with(state, request.channel, &ctx).await;

    // 5. Only a real acceptance may write `sent` / advance the conversation.
    let accepted = result.is_accepted();
    apply_send_result(pool, &message_id, &result).await?;

    if accepted {
        sqlx::query(
            "UPDATE conversations SET state = CASE WHEN state IN ('opted_out','closed_won','closed_lost') \
             THEN state ELSE 'awaiting_reply' END, last_outbound_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&now)
        .bind(request.conversation_id)
        .execute(pool)
        .await?;
    } else {
        tracing::warn!(
            channel = request.channel,
            conversation_id = request.conversation_id,
            message_id = %message_id,
            provider = %result.provider,
            status = result.status.as_str(),
            http = ?result.http_status,
            "[OUTBOUND NOT SENT] {}",
            result.reason()
        );
    }

    Ok(SendOutcome {
        message_id,
        conversation_id: request.conversation_id.to_string(),
        result,
    })
}

/// Writes the provider's answer onto the queued row. This is the only place
/// `status = 'sent'` is produced in the outbound path, and it is unreachable
/// unless `is_accepted()` (real 2xx, not simulated).
pub async fn apply_send_result(
    pool: &DbPool,
    message_id: &str,
    result: &SendResult,
) -> Result<(), sqlx::Error> {
    let status = result.message_status();
    let sent_at = if result.is_accepted() {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        None
    };
    let provider_message_id = if result.provider_message_id.is_empty() {
        None
    } else {
        Some(result.provider_message_id.clone())
    };

    let mut metadata = serde_json::Map::new();
    metadata.insert("sendStatus".to_string(), Value::String(result.status.as_str().to_string()));
    metadata.insert("simulated".to_string(), Value::Bool(result.simulated));
    if let Some(http) = result.http_status {
        metadata.insert("httpStatus".to_string(), Value::Number(http.into()));
    }
    if let Some(error) = &result.error {
        metadata.insert("error".to_string(), Value::String(error.clone()));
    }
    match &result.raw {
        Value::Object(body) => {
            for (key, value) in body {
                metadata.insert(key.clone(), value.clone());
            }
        }
        // The `Option<Value>` from `Map::insert` is the value this key held
        // before; `metadata` is built from scratch in this function, so the
        // key cannot have been set and the returned `None` carries no
        // information -- the arm is a statement so the types line up.
        other => {
            metadata.insert("response".to_string(), other.clone());
        }
    }

    sqlx::query(
        "UPDATE messages SET status = ?, delivery_status = ?, provider = ?, provider_message_id = ?, \
         provider_metadata = ?, sent_at = COALESCE(sent_at, ?) WHERE id = ?",
    )
    .bind(status)
    .bind(status)
    .bind(&result.provider)
    .bind(provider_message_id)
    .bind(Value::Object(metadata).to_string())
    .bind(sent_at)
    .bind(message_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Loads the Contact and its open Conversation, then sends. Used by the paths
/// that only know a contact id (macros, automations, follow-ups).
pub async fn send_to_contact(
    state: &AppState,
    contact_id: &str,
    body: &str,
    subject: Option<&str>,
    sender_name: &str,
    sender_type: &str,
) -> Result<SendOutcome, MessagingError> {
    let row = sqlx::query("SELECT id, value, channel, tenant_key FROM contacts WHERE id = ? LIMIT 1")
        .bind(contact_id)
        .fetch_optional(&state.db)
        .await?;
    let Some(row) = row else {
        return Err(MessagingError::NotFound(format!(
            "Contact {contact_id} does not exist; nothing was sent."
        )));
    };
    let value: String = row.try_get("value").unwrap_or_default();
    let channel: String = row.try_get("channel").unwrap_or_else(|_| "whatsapp".to_string());
    let tenant_key: Option<String> = row.try_get("tenant_key").ok().flatten();

    let conversation_id = open_conversation_for(&state.db, contact_id, &channel).await?;

    let request = OutboundRequest::new(&conversation_id, contact_id, &value, &channel, body)
        .subject(subject)
        .tenant_key(tenant_key.as_deref())
        .sender(sender_name, sender_type);
    send_outbound(state, &request).await
}

/// The newest non-terminal conversation, or a new one. Terminal threads stay
/// terminal -- an opted-out contact is not reopened by an automated send.
pub async fn open_conversation_for(
    pool: &DbPool,
    contact_id: &str,
    channel: &str,
) -> Result<String, MessagingError> {
    let existing = sqlx::query(
        "SELECT id FROM conversations WHERE contact_id = ? \
         AND state NOT IN ('opted_out','closed_won','closed_lost') \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(contact_id)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = existing {
        return Ok(row.try_get("id")?);
    }

    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO conversations (id, contact_id, channel, state, priority, human_takeover, \
         created_at, updated_at) VALUES (?, ?, ?, 'engaged', 'medium', 0, ?, ?)",
    )
    .bind(&id)
    .bind(contact_id)
    .bind(channel)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(id)
}

/// Reads a conversation plus its contact for the send path, so callers do not
/// each re-derive the join.
pub async fn conversation_send_context(
    pool: &DbPool,
    conversation_id: &str,
) -> Result<Option<(String, String, String, Option<String>, Option<String>)>, MessagingError> {
    let row = sqlx::query(
        "SELECT c.contact_id, c.inbox_id, c.tenant_key, ct.value, ct.channel \
         FROM conversations c LEFT JOIN contacts ct ON ct.id = c.contact_id WHERE c.id = ? LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else { return Ok(None) };
    let contact_id: Option<String> = row.try_get("contact_id").ok().flatten();
    let inbox_id: Option<String> = row.try_get("inbox_id").ok().flatten();
    let tenant_key: Option<String> = row.try_get("tenant_key").ok().flatten();
    let value: Option<String> = row.try_get("value").ok().flatten();
    let channel: Option<String> = row.try_get("channel").ok().flatten();

    let contact_id = contact_id.filter(|id| !id.is_empty());
    match (contact_id, value) {
        (Some(contact_id), Some(value)) => Ok(Some((
            contact_id,
            value,
            channel.unwrap_or_else(|| "whatsapp".to_string()),
            inbox_id,
            tenant_key,
        ))),
        _ => Err(MessagingError::NotFound(format!(
            "Conversation {conversation_id} has no contact to send to."
        ))),
    }
}

/// Send a body that is already known to belong to a conversation.
pub async fn send_in_conversation(
    state: &AppState,
    conversation_id: &str,
    channel: &str,
    body: &str,
    subject: Option<&str>,
    sender_name: &str,
    sender_type: &str,
    is_private: bool,
) -> Result<SendOutcome, MessagingError> {
    let (contact_id, contact_value, resolved_channel, inbox_id, tenant_key) =
        match conversation_send_context(&state.db, conversation_id).await? {
            Some(context) => context,
            None => {
                return Err(MessagingError::NotFound(format!(
                    "Conversation {conversation_id} not found."
                )))
            }
        };

    let request = OutboundRequest {
        conversation_id,
        contact_id: &contact_id,
        contact_value: &contact_value,
        channel: if channel.is_empty() { resolved_channel.as_str() } else { channel },
        body,
        subject,
        inbox_id: inbox_id.as_deref(),
        tenant_key: tenant_key.as_deref(),
        sender_name,
        sender_type,
        is_private,
        respect_business_hours: false,
        template: None,
    };
    send_outbound(state, &request).await
}

/// Convenience for the queued-then-sent state machine used by retries: returns
/// the rows that claim to be outbound but never got a provider acceptance.
pub async fn list_unaccepted_outbound(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<(String, String, String, String)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, conversation_id, channel, COALESCE(body, '') FROM messages \
         WHERE direction = 'outbound' AND is_private = 0 AND status IN ('queued','not_sent','failed') \
         ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|row| {
            (
                row.try_get("id").unwrap_or_default(),
                row.try_get("conversation_id").unwrap_or_default(),
                row.try_get::<String, _>("channel").unwrap_or_default(),
                row.try_get::<String, _>("body").unwrap_or_default(),
            )
        })
        .collect())
}
