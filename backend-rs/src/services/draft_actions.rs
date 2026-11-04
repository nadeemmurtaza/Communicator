#![allow(dead_code)]

//! Port of `services/draft_actions.py`.
//!
//! Python's `approve_and_send` marked a draft SENT whatever the provider said,
//! and the Rust route it replaced never contacted a provider at all -- it wrote
//! a message row with `status = 'delivered'` and called that a send. Both are
//! fixed here: the send goes through `services::messaging` (consent gate,
//! business-hours gate, provider call, receipt) and the draft only becomes `sent`
//! when the provider accepted it.

use crate::db::AppState;
use crate::services::messaging::{self, OutboundRequest};
use serde_json::{json, Value};
use sqlx::Row;

/// Draft states that must not be re-sent: the reply already left, or a human
/// killed it.
const NON_RESENDABLE: &[&str] = &["sent", "rejected"];

#[derive(Debug, Clone)]
pub enum DraftActionError {
    NotFound(String),
    Blocked { draft_id: String, status: String },
    ConversationMissing(String),
    ContactMissing(String),
    DbFailed { context: String, reason: String },
}

impl DraftActionError {
    pub fn status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            DraftActionError::NotFound(_) | DraftActionError::ConversationMissing(_) => {
                StatusCode::NOT_FOUND
            }
            DraftActionError::Blocked { .. } => StatusCode::CONFLICT,
            DraftActionError::ContactMissing(_) => StatusCode::BAD_REQUEST,
            DraftActionError::DbFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn json(&self) -> Value {
        let mut body = json!({ "saved": false });
        match self {
            DraftActionError::NotFound(id) => {
                body["error"] = json!("no draft with that id exists");
                body["draftId"] = json!(id);
            }
            DraftActionError::Blocked { draft_id, status } => {
                body["error"] = json!(format!(
                    "a draft in status '{status}' cannot be approved again: nothing was re-sent"
                ));
                body["draftId"] = json!(draft_id);
                body["draftStatus"] = json!(status);
            }
            DraftActionError::ConversationMissing(id) => {
                body["error"] = json!(format!(
                    "the conversation this draft belongs to ({id}) is not in the database, so the \
                     reply has nowhere to go"
                ));
            }
            DraftActionError::ContactMissing(id) => {
                body["error"] = json!(format!(
                    "the conversation behind this draft ({id}) has no contact row, so there is no \
                     address to send to"
                ));
            }
            DraftActionError::DbFailed { context, reason } => {
                body["error"] = json!(format!("could not {context}: {reason}"));
                body["databaseError"] = json!(true);
            }
        }
        body
    }
}

/// Writes one `audit_logs` row for a draft action. Returns `None` on success and
/// the warning text otherwise, so a missing audit trail is never invisible.
pub async fn record_draft_audit(
    state: &AppState,
    entity_id: &str,
    action: &str,
    actor: &str,
    description: &str,
    details: Value,
) -> Option<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let details_text = details.to_string();
    let result = sqlx::query(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, severity, \
         compliance_category, description, details, created_at) \
         VALUES (?, 'draft', ?, ?, ?, 'info', 'audit_trail', ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_id)
    .bind(action)
    .bind(actor)
    .bind(description)
    .bind(&details_text)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 1 => None,
        Ok(report) => Some(format!(
            "the '{action}' audit row was NOT written: the INSERT affected {} rows",
            report.rows_affected()
        )),
        Err(err) => Some(format!("the '{action}' audit row could not be written: {err}")),
    }
}

#[derive(Debug, Clone)]
pub struct ApprovalReport {
    pub draft_id: String,
    pub draft_status: String,
    pub sent: bool,
    pub message_id: Option<String>,
    pub send: Value,
    pub audit_note: Option<String>,
    pub warnings: Vec<String>,
}

impl ApprovalReport {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.draft_id,
            "draftId": self.draft_id,
            "status": self.draft_status,
            "draftStatus": self.draft_status,
            "sent": self.sent,
            "messageId": self.message_id,
            "send": self.send,
            "auditWritten": self.audit_note.is_none(),
            "auditWarning": self.audit_note,
            "warnings": self.warnings,
        })
    }
}

async fn load_draft(state: &AppState, draft_id: &str) -> Result<Option<sqlx::sqlite::SqliteRow>, DraftActionError> {
    sqlx::query(
        "SELECT id, conversation_id, body, confidence, citations, status, created_at \
         FROM drafts WHERE id = ? LIMIT 1",
    )
    .bind(draft_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| DraftActionError::DbFailed {
        context: format!("read draft {draft_id}"),
        reason: err.to_string(),
    })
}

/// Port of `approve_and_send`, with the send delegated to the real outbound
/// pipeline. `actor` is `agent` for a human click and `ai` for the auto-dispatch.
pub async fn approve_and_send(
    state: &AppState,
    draft_id: &str,
    actor: &str,
) -> Result<ApprovalReport, DraftActionError> {
    let Some(draft) = load_draft(state, draft_id).await? else {
        return Err(DraftActionError::NotFound(draft_id.to_string()));
    };

    let current_status =
        crate::db::required_text(&draft, "status").map_err(|column| DraftActionError::DbFailed {
            context: format!("read the {column} of draft {draft_id}"),
            reason: "the column is absent or not text, so it is unknown whether this draft was \
                     already sent"
                .to_string(),
        })?;
    if NON_RESENDABLE
        .iter()
        .any(|s| current_status.eq_ignore_ascii_case(s))
    {
        return Err(DraftActionError::Blocked {
            draft_id: draft_id.to_string(),
            status: current_status,
        });
    }

    let body = crate::db::required_text(&draft, "body").map_err(|column| {
        DraftActionError::DbFailed {
            context: format!("read the {column} of draft {draft_id}"),
            reason: "there is no draft text to send".to_string(),
        }
    })?;
    let conversation_id =
        crate::db::required_text(&draft, "conversation_id").map_err(|column| {
            DraftActionError::DbFailed {
                context: format!("read the {column} of draft {draft_id}"),
                reason: "the draft is not linked to a conversation, so there is nothing to send"
                    .to_string(),
            }
        })?;

    let conversation = sqlx::query(
        "SELECT id, COALESCE(contact_id, '') AS contact_id, COALESCE(inbox_id, '') AS inbox_id, \
                COALESCE(channel, '') AS channel FROM conversations WHERE id = ? LIMIT 1",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| DraftActionError::DbFailed {
        context: format!("read conversation {conversation_id}"),
        reason: err.to_string(),
    })?;
    let Some(conversation) = conversation else {
        return Err(DraftActionError::ConversationMissing(conversation_id));
    };

    let contact_id: String = conversation.try_get("contact_id").unwrap_or_default();
    if contact_id.trim().is_empty() {
        return Err(DraftActionError::ContactMissing(conversation_id.clone()));
    }

    let (projection, _missing) = crate::routes::contacts::contact_projection(&state.db).await;
    let select_sql = format!(
        "SELECT {} FROM contacts WHERE id = ? LIMIT 1",
        projection.join(", ")
    );
    let contact = sqlx::query(&select_sql)
        .bind(&contact_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| DraftActionError::DbFailed {
            context: format!("read contact {contact_id}"),
            reason: err.to_string(),
        })?;
    let Some(contact) = contact else {
        return Err(DraftActionError::ContactMissing(conversation_id.clone()));
    };

    let contact_value =
        crate::db::required_text(&contact, "value").map_err(|_| {
            DraftActionError::ContactMissing(conversation_id.clone())
        })?;
    let contact_channel = contact
        .try_get::<Option<String>, _>("channel")
        .ok()
        .flatten()
        .unwrap_or_default();
    let conversation_channel = conversation
        .try_get::<Option<String>, _>("channel")
        .ok()
        .flatten()
        .unwrap_or_default();
    let channel = if conversation_channel.trim().is_empty() {
        contact_channel.as_str()
    } else {
        conversation_channel.as_str()
    };
    if channel.trim().is_empty() {
        return Err(DraftActionError::DbFailed {
            context: format!("send draft {draft_id}"),
            reason: "neither the conversation nor the contact records a channel, so there is no \
                     platform to send through"
                .to_string(),
        });
    }
    let tenant_key: Option<String> = contact
        .try_get::<Option<String>, _>("tenant_key")
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty());
    let inbox_id: Option<String> = conversation
        .try_get::<String, _>("inbox_id")
        .ok()
        .filter(|v| !v.trim().is_empty());

    let (sender_name, sender_type) = if actor.eq_ignore_ascii_case("ai") {
        ("AI Copilot", "ai")
    } else {
        ("agent", "agent")
    };

    let request = OutboundRequest {
        conversation_id: &conversation_id,
        contact_id: &contact_id,
        contact_value: &contact_value,
        channel,
        body: &body,
        subject: None,
        inbox_id: inbox_id.as_deref(),
        tenant_key: tenant_key.as_deref(),
        sender_name,
        sender_type,
        is_private: false,
        respect_business_hours: false,
        template: None,
    };

    let mut warnings: Vec<String> = Vec::new();
    warnings.push(
        "the harvester outcome pushback (Python draft_actions.py:39-44) has no Rust client port, \
         so the outreach outcome was not reported back to Harvester"
            .to_string(),
    );

    let outcome = match messaging::send_outbound(state, &request).await {
        Ok(outcome) => outcome,
        Err(err) => {
            // Nothing left the process, so the draft must not move to `sent`.
            return Ok(ApprovalReport {
                draft_id: draft_id.to_string(),
                draft_status: current_status.clone(),
                sent: false,
                message_id: None,
                send: json!({ "error": err.to_string(), "status": "not_sent" }),
                audit_note: record_draft_audit(
                    state,
                    draft_id,
                    "approval_failed",
                    actor,
                    &format!("approving draft {draft_id} could not reach the outbound pipeline"),
                    json!({ "reason": err.to_string(), "draftStatus": current_status }),
                )
                .await,
                warnings,
            });
        }
    };

    let accepted = outcome.was_sent();
    let send_json = outcome.result.to_json();
    let new_status = if accepted { "sent" } else { "send_failed" };

    let now = chrono::Utc::now().to_rfc3339();
    let updated = sqlx::query("UPDATE drafts SET status = ?, updated_at = ? WHERE id = ?")
        .bind(new_status)
        .bind(&now)
        .bind(draft_id)
        .execute(&state.db)
        .await;
    let mut draft_status = new_status.to_string();
    match updated {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => {
            warnings.push(format!(
                "the draft status could not be moved to '{new_status}': the UPDATE affected {} rows",
                report.rows_affected()
            ));
            draft_status = current_status.clone();
        }
        Err(err) => {
            warnings.push(format!(
                "the draft status could not be moved to '{new_status}': {err}"
            ));
            draft_status = current_status.clone();
        }
    }

    let action = if actor.eq_ignore_ascii_case("ai") {
        "auto_sent"
    } else {
        "approved_and_sent"
    };
    let audit_note = record_draft_audit(
        state,
        draft_id,
        action,
        actor,
        &format!("draft {draft_id} sent on channel {channel}"),
        json!({
            "provider": outcome.result.provider,
            "providerMessageId": outcome.result.provider_message_id,
            "sendStatus": outcome.result.status.as_str(),
            "httpStatus": outcome.result.http_status,
            "error": outcome.result.error,
            "simulated": outcome.result.simulated,
            "messageId": outcome.message_id,
            "draftStatusAfter": draft_status,
        }),
    )
    .await;

    Ok(ApprovalReport {
        draft_id: draft_id.to_string(),
        draft_status,
        sent: accepted,
        message_id: Some(outcome.message_id.clone()),
        send: send_json,
        audit_note,
        warnings,
    })
}

/// Port of `reject_draft`: the reason is stored, not dropped.
pub async fn reject_draft(
    state: &AppState,
    draft_id: &str,
    reason: &str,
    actor: &str,
) -> Result<ApprovalReport, DraftActionError> {
    let Some(draft) = load_draft(state, draft_id).await? else {
        return Err(DraftActionError::NotFound(draft_id.to_string()));
    };
    let current_status = draft
        .try_get::<Option<String>, _>("status")
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty());

    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE drafts SET status = 'rejected', reviewer = ?, reviewer_feedback = ?, updated_at = ? \
         WHERE id = ?",
    )
    .bind(actor)
    .bind(reason)
    .bind(&now)
    .bind(draft_id)
    .execute(&state.db)
    .await
    .map_err(|err| DraftActionError::DbFailed {
        context: format!("reject draft {draft_id}"),
        reason: err.to_string(),
    })?;

    let mut warnings = Vec::new();
    if result.rows_affected() != 1 {
        warnings.push(format!(
            "the draft was NOT marked rejected: the UPDATE affected {} rows",
            result.rows_affected()
        ));
    }

    let audit_note = record_draft_audit(
        state,
        draft_id,
        "rejected",
        actor,
        &format!("draft {draft_id} rejected"),
        json!({ "feedback": reason, "previousStatus": current_status }),
    )
    .await;

    Ok(ApprovalReport {
        draft_id: draft_id.to_string(),
        draft_status: "rejected".to_string(),
        sent: false,
        message_id: None,
        send: json!({ "status": "not_sent", "reason": "a human rejected the draft" }),
        audit_note,
        warnings,
    })
}

/// Port of `dispatch_auto_approved_drafts`: sends everything the reply engine
/// left in `auto_approved` for one tenant. Drafts have no tenant column of their
/// own, so the scope is the conversation's contact -- and if that column is not
/// in this schema, dispatching would send another account's replies, which is a
/// refusal, not a fallback.
pub async fn dispatch_auto_approved(
    state: &AppState,
    tenant_key: &str,
) -> Result<Vec<Result<ApprovalReport, DraftActionError>>, DraftActionError> {
    if tenant_key.trim().is_empty() {
        return Err(DraftActionError::DbFailed {
            context: "list auto-approved drafts".to_string(),
            reason: "no tenant scope on the request, so it is unknown whose drafts to send"
                .to_string(),
        });
    }

    if !crate::routes::contacts::column_exists(&state.db, "contacts", "tenant_key").await {
        return Err(DraftActionError::DbFailed {
            context: "list auto-approved drafts".to_string(),
            reason: "this database has no contacts.tenant_key, so auto-approved drafts cannot be \
                     limited to one tenant without risking another tenant's replies"
                .to_string(),
        });
    }

    let rows = sqlx::query(
        "SELECT d.id AS id FROM drafts d \
         JOIN conversations c ON c.id = d.conversation_id \
         JOIN contacts ct ON ct.id = c.contact_id \
         WHERE d.status = 'auto_approved' AND COALESCE(ct.tenant_key, '') = ? \
         ORDER BY d.created_at",
    )
    .bind(tenant_key)
    .fetch_all(&state.db)
    .await
    .map_err(|err| DraftActionError::DbFailed {
        context: "list auto-approved drafts".to_string(),
        reason: err.to_string(),
    })?;

    let mut reports = Vec::new();
    for row in rows {
        match row.try_get::<String, _>("id") {
            Ok(id) => reports.push(approve_and_send(state, &id, "ai").await),
            Err(err) => reports.push(Err(DraftActionError::DbFailed {
                context: "list auto-approved drafts".to_string(),
                reason: format!("a draft id could not be read from its row: {err}"),
            })),
        }
    }
    Ok(reports)
}
