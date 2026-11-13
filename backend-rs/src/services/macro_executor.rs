#![allow(dead_code)]

//! Macro execution: run a saved multi-action macro against a conversation.
//!
//! Port of `services/macro_executor.py`. The Python executor handled
//! `send_message` by inserting a `Message` row with
//! `status = MessageStatus.SENT` and a fabricated
//! `provider_message_id = f"macro_msg_{uuid4().hex[:8]}"` -- the message had
//! never touched a provider, yet the thread showed it as delivered and
//! `last_outbound_at` moved on. Here `send_message` goes through
//! [`crate::services::messaging`], which is the only code that may write
//! `status = 'sent'`, and the action report carries the provider's real answer
//! (including the refusal text when nothing was sent).
//!
//! Two Python behaviours are deliberately *not* copied:
//!
//! * the default `agent_name = "Riley Brooks"` -- a real person's name baked
//!   into the service. The actor is whatever the caller passes; when nothing is
//!   passed the action is attributed to `"macro_executor"` and the report says
//!   no actor was supplied.
//! * `except ValueError: pass` on an invalid `change_status` -- a typo'd state
//!   was silently ignored. It is now reported as a failed action with the list
//!   of states the column actually holds.

use crate::db::AppState;
use crate::services::messaging::{self, OutboundRequest};
use serde_json::{json, Map, Value};
use sqlx::Row;
use std::env;

/// States a macro may write. Derived from `ConversationState` in
/// `models/conversation.py` (the schema both backends share) plus `resolved` /
/// `closed`, which the routing queries in `routes/team_routing.rs` already treat
/// as terminal states. Anything else is rejected with this list in the report.
const CONVERSATION_STATES: &[&str] = &[
    "new",
    "awaiting_reply",
    "engaged",
    "hot_lead",
    "handed_off",
    "opted_out",
    "closed_won",
    "closed_lost",
    "resolved",
    "closed",
];

#[derive(Debug)]
pub enum MacroError {
    NotFound(String),
    InvalidActions(String),
    Db(sqlx::Error),
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(what) => write!(f, "{what}"),
            Self::InvalidActions(why) => write!(f, "macro actions could not be read: {why}"),
            Self::Db(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for MacroError {}

impl From<sqlx::Error> for MacroError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err)
    }
}

#[derive(Clone, Debug)]
pub struct MacroExecution {
    pub macro_id: String,
    pub macro_name: String,
    pub conversation_id: String,
    pub executed_at: String,
    /// One entry per action, in order: what it did, or why it did not.
    pub actions: Vec<Value>,
    /// Actions that could not be completed by this backend (missing table,
    /// missing subsystem). Never counted as success.
    pub deferred: Vec<Value>,
    /// Actions that touched the wire: the real `SendResult` JSON per send.
    pub sends: Vec<Value>,
    pub conversation_state: Option<String>,
    pub conversation_priority: Option<String>,
    pub tags: Vec<String>,
}

impl MacroExecution {
    pub fn to_json(&self) -> Value {
        json!({
            "status": if self.sends.iter().any(|s| s["delivered"].as_bool() == Some(false)) {
                "completed_with_undelivered"
            } else if self.deferred.is_empty() { "ok" } else { "ok_with_deferred" },
            "macroId": self.macro_id,
            "macroName": self.macro_name,
            "conversationId": self.conversation_id,
            "executedActionsCount": self.actions.len(),
            "executedActions": self.actions,
            "deferred": self.deferred,
            "sends": self.sends,
            "conversationState": self.conversation_state,
            "conversationPriority": self.conversation_priority,
            "conversationTags": self.tags,
            "executedAt": self.executed_at,
        })
    }
}

/// Every macro row, oldest first (the picker list).
pub async fn list_macros(state: &AppState) -> Result<Vec<Value>, MacroError> {
    let rows = sqlx::query("SELECT id, name, description, actions, created_at, updated_at FROM macros ORDER BY created_at ASC")
        .fetch_all(&state.db)
        .await?;
    Ok(rows
        .iter()
        .map(|row| {
            let actions_raw: Option<String> = row.try_get("actions").ok().flatten();
            json!({
                "id": text(row, "id"),
                "name": text(row, "name"),
                "description": text(row, "description"),
                "actions": actions_raw
                    .as_deref()
                    .and_then(|raw| serde_json::from_str::<Value>(raw).ok()),
                "actionsParseError": actions_raw
                    .filter(|raw| !raw.is_empty())
                    .map(|raw| serde_json::from_str::<Value>(&raw).err().map(|e| e.to_string()))
                    .flatten(),
                "createdAt": text(row, "created_at"),
                "updatedAt": text(row, "updated_at"),
            })
        })
        .collect())
}

async fn load_macro(state: &AppState, macro_id: &str) -> Result<(String, Vec<Value>), MacroError> {
    let row = sqlx::query("SELECT id, name, actions FROM macros WHERE id = $1 LIMIT 1")
        .bind(macro_id)
        .fetch_optional(&state.db)
        .await?;
    let Some(row) = row else {
        return Err(MacroError::NotFound(format!("Macro {macro_id} not found.")));
    };
    let name = text(&row, "name");
    let raw: Option<String> = row.try_get("actions").ok().flatten();
    match raw.as_deref() {
        None | Some("") => Err(MacroError::InvalidActions(format!(
            "Macro '{name}' ({macro_id}) has no actions JSON stored."
        ))),
        Some(raw) => match serde_json::from_str::<Value>(raw) {
            Ok(Value::Array(actions)) => Ok((name, actions)),
            Ok(Value::Object(map)) => Ok((name, map.get("actions").and_then(Value::as_array).cloned().unwrap_or_default())),
            Ok(other) => Err(MacroError::InvalidActions(format!(
                "Macro '{name}' actions are {other}, not a list."
            ))),
            Err(err) => Err(MacroError::InvalidActions(format!(
                "Macro '{name}' actions JSON is invalid: {err}"
            ))),
        },
    }
}

fn text(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get::<String, _>(column)
        .or_else(|_| row.try_get::<Option<String>, _>(column).map(|v| v.unwrap_or_default()))
        .unwrap_or_default()
}

/// Runs every action of `macro_id` against `conversation_id`.
///
/// `executed_by` is the human (or system) that pressed the button; it is stored
/// on the conversation's `custom_attributes.last_macro_executed` exactly as
/// given -- and if it is empty nothing is invented.
pub async fn execute_macro(
    state: &AppState,
    conversation_id: &str,
    macro_id: &str,
    executed_by: Option<&str>,
) -> Result<MacroExecution, MacroError> {
    let (macro_name, actions) = load_macro(state, macro_id).await?;

    let row = sqlx::query(
        "SELECT id, state, priority, team_id, assigned_to_id, contact_id, inbox_id, tenant_key, \
         channel, resolution_type, snoozed_until, custom_attributes, last_outbound_at \
         FROM conversations WHERE id = $1 LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await?;
    let Some(conversation) = row else {
        return Err(MacroError::NotFound(format!(
            "Conversation {conversation_id} not found; no macro action was applied."
        )));
    };

    let actor = executed_by.map(str::trim).filter(|value| !value.is_empty());
    let attribution = actor.unwrap_or("macro_executor");
    if actor.is_none() {
        tracing::warn!("macro {macro_id} executed with no actor supplied; recorded as 'macro_executor'");
    }

    let mut attrs: Map<String, Value> = json_object_from_str(&text(&conversation, "custom_attributes"));
    let mut tags: Vec<String> = attrs
        .get("tags")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();

    // Every column is kept as an `Option` so an unchanged NULL is written back
    // as NULL instead of as an empty string.
    let mut state_value = opt_text(&conversation, "state");
    let mut priority = opt_text(&conversation, "priority");
    let mut resolution_type = opt_text(&conversation, "resolution_type");
    let mut team_id: Option<String> = opt_text(&conversation, "team_id");
    let mut assigned_to_id: Option<String> = opt_text(&conversation, "assigned_to_id");
    let mut snoozed_until: Option<String> = opt_text(&conversation, "snoozed_until");
    let mut last_outbound_at: Option<String> = opt_text(&conversation, "last_outbound_at");

    let now = chrono::Utc::now().to_rfc3339();
    let mut reports: Vec<Value> = Vec::new();
    let mut deferred: Vec<Value> = Vec::new();
    let mut sends: Vec<Value> = Vec::new();

    for action in &actions {
        let a_type = action.get("type").and_then(Value::as_str).unwrap_or("").to_string();
        if a_type.is_empty() {
            reports.push(json!({"action": "", "status": "failed", "error": "action has no 'type'"}));
            continue;
        }

        match a_type.as_str() {
            "resolve_conversation" => {
                state_value = Some("closed_won".to_string());
                resolution_type = Some(
                    action
                        .get("resolution_type")
                        .and_then(Value::as_str)
                        .filter(|v| !v.trim().is_empty())
                        .unwrap_or("human_assisted")
                        .to_string(),
                );
                reports.push(json!({"action": a_type, "status": "applied", "state": state_value, "resolutionType": resolution_type}));
            }
            "change_priority" => {
                let requested = action.get("priority").and_then(Value::as_str).unwrap_or("");
                let lowered = requested.trim().to_ascii_lowercase();
                if lowered.is_empty() {
                    reports.push(json!({"action": a_type, "status": "failed", "error": "no 'priority' value on the action"}));
                    continue;
                }
                priority = Some(lowered);
                reports.push(json!({"action": a_type, "status": "applied", "priority": priority}));
            }
            "change_status" => {
                let requested = action.get("status").and_then(Value::as_str).unwrap_or("").trim().to_ascii_lowercase();
                if CONVERSATION_STATES.iter().any(|known| *known == requested) {
                    state_value = Some(requested.clone());
                    reports.push(json!({"action": a_type, "status": "applied", "state": requested}));
                } else {
                    // Python swallowed this; here it is a visible failure.
                    reports.push(json!({
                        "action": a_type, "status": "failed",
                        "error": format!("'{requested}' is not a conversation state this backend writes."),
                        "allowed": CONVERSATION_STATES,
                    }));
                }
            }
            "assign_team" => {
                if let Some(id) = action.get("team_id").and_then(Value::as_str).filter(|v| !v.is_empty()) {
                    team_id = Some(id.to_string());
                    reports.push(json!({"action": a_type, "status": "applied", "teamId": id}));
                } else {
                    let name = action.get("team_name").and_then(Value::as_str).unwrap_or("").trim().to_string();
                    match resolve_team_id(state, &name).await {
                        Ok(Some(found)) => {
                            team_id = Some(found.0.clone());
                            reports.push(json!({"action": a_type, "status": "applied", "team": found.1, "teamId": found.0}));
                        }
                        Ok(None) => {
                            let note = json!({"action": a_type, "status": "noted", "team": name, "note": "no matching row in teams; conversation left unassigned"});
                            reports.push(note.clone());
                            deferred.push(note);
                        }
                        Err(err) => {
                            let note = json!({"action": a_type, "status": "deferred", "team": name, "reason": err});
                            reports.push(note.clone());
                            deferred.push(note);
                        }
                    }
                }
            }
            "assign_agent" => {
                if let Some(id) = action.get("agent_id").and_then(Value::as_str).filter(|v| !v.is_empty()) {
                    assigned_to_id = Some(id.to_string());
                    reports.push(json!({"action": a_type, "status": "applied", "agentId": id}));
                } else {
                    let name = action.get("agent_name").and_then(Value::as_str).unwrap_or("").trim().to_string();
                    match resolve_agent_id(state, &name).await {
                        Ok(Some(found)) => {
                            assigned_to_id = Some(found.0.clone());
                            reports.push(json!({"action": a_type, "status": "applied", "agent": found.1, "agentId": found.0}));
                        }
                        Ok(None) => {
                            let note = json!({"action": a_type, "status": "noted", "agent": name, "note": "no matching row in team_members; conversation left unassigned"});
                            reports.push(note.clone());
                            deferred.push(note);
                        }
                        Err(err) => {
                            let note = json!({"action": a_type, "status": "deferred", "agent": name, "reason": err});
                            reports.push(note.clone());
                            deferred.push(note);
                        }
                    }
                }
            }
            "add_tags" => {
                let added = string_list(action, "tags");
                for tag in &added {
                    if !tags.iter().any(|existing| existing == tag) {
                        tags.push(tag.clone());
                    }
                }
                reports.push(json!({"action": a_type, "status": "applied", "added": added, "tags": tags}));
            }
            "remove_tags" => {
                let removed = string_list(action, "tags");
                tags.retain(|tag| !removed.contains(tag));
                reports.push(json!({"action": a_type, "status": "applied", "removed": removed, "tags": tags}));
            }
            "send_message" => {
                let body = action.get("message").and_then(Value::as_str).unwrap_or("").trim().to_string();
                if body.is_empty() {
                    reports.push(json!({"action": a_type, "status": "failed", "error": "action has no 'message' text"}));
                    continue;
                }
                match send_thread_message(state, &conversation, conversation_id, &body, attribution, false).await {
                    Ok(outcome) => {
                        let report = outcome.result.to_json();
                        let status = outcome.result.message_status();
                        if status != "sent" {
                            tracing::warn!(
                                "macro {macro_id} send_message was NOT delivered on conversation {conversation_id}: {}",
                                outcome.result.reason()
                            );
                        } else {
                            last_outbound_at = Some(now.clone());
                        }
                        sends.push(report.clone());
                        reports.push(json!({
                            "action": a_type,
                            // Straight from the provider's answer: sent | failed | not_sent.
                            "status": status,
                            "delivered": outcome.was_sent(),
                            "messageId": outcome.message_id,
                            "delivery": report,
                        }));
                    }
                    Err(err) => {
                        // The row exists as `queued`/`not_sent`; it is never SENT.
                        reports.push(json!({
                            "action": a_type, "status": "failed",
                            "error": err.to_string(),
                            "note": "no message was marked sent"
                        }));
                    }
                }
            }
            "add_private_note" => {
                let note = action.get("note").and_then(Value::as_str).unwrap_or("").trim().to_string();
                if note.is_empty() {
                    reports.push(json!({"action": a_type, "status": "failed", "error": "action has no 'note' text"}));
                    continue;
                }
                let contact_id = opt_text(&conversation, "contact_id");
                match contact_id {
                    Some(contact_id) => match record_contact_note(state, &contact_id, attribution, &note).await {
                        Ok(()) => reports.push(json!({"action": a_type, "noteSaved": true})),
                        Err(reason) => {
                            let entry = json!({"action": a_type, "status": "deferred", "reason": reason});
                            deferred.push(entry.clone());
                            reports.push(entry);
                        }
                    },
                    None => {
                        reports.push(json!({
                            "action": a_type, "status": "failed",
                            "error": "conversation has no contact, so there is nothing to attach a note to"
                        }));
                        continue;
                    }
                }
                // The thread copy is private: `send_outbound` with is_private
                // records it as an internal note and never contacts a provider.
                match send_thread_message(state, &conversation, conversation_id, &note, attribution, true).await {
                    Ok(outcome) => {
                        let report = outcome.result.to_json();
                        sends.push(report.clone());
                        reports.push(json!({
                            "action": a_type, "status": "recorded", "messageId": outcome.message_id,
                            "deliveredToContact": outcome.was_sent(), "delivery": report,
                        }));
                    }
                    Err(err) => reports.push(json!({"action": a_type, "status": "failed", "error": err.to_string()})),
                }
            }
            "send_csat_survey" => {
                attrs.insert("csat_dispatched_at".to_string(), Value::String(now.clone()));
                let note = json!({
                    "action": a_type,
                    "status": "recorded_only",
                    "csatDispatchedAt": now.clone(),
                    "note": "This backend stamps the dispatch marker only; no survey was delivered \
                             to the customer because there is no CSAT message template wired to a \
                             channel adapter yet."
                });
                reports.push(note.clone());
                deferred.push(note);
            }
            "snooze_conversation" => {
                let hours = action
                    .get("duration_hours")
                    .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                    .unwrap_or(0.0);
                if hours <= 0.0 || hours > 24.0 * 365.0 {
                    reports.push(json!({
                        "action": a_type, "status": "failed",
                        "error": format!("duration_hours {hours} is outside 0..8760; nothing was snoozed")
                    }));
                    continue;
                }
                let until = chrono::Utc::now() + chrono::Duration::seconds((hours * 3600.0) as i64);
                snoozed_until = Some(until.to_rfc3339());
                // The Python executor only set `snoozed_until`; it did not move
                // the conversation state, so neither does this one -- inventing a
                // 'snoozed' state would be a state the rest of the product does
                // not produce.
                reports.push(json!({
                    "action": a_type, "status": "applied", "snoozedUntil": snoozed_until,
                    "note": "conversation state left as-is, matching the executor this replaces"
                }));
            }
            "mutate_custom_attributes" => {
                let Some(extra) = action.get("attributes").and_then(Value::as_object) else {
                    reports.push(json!({"action": a_type, "status": "failed", "error": "action has no 'attributes' object"}));
                    continue;
                };
                let keys: Vec<String> = extra.keys().cloned().collect();
                for (key, value) in extra.iter() {
                    attrs.insert(key.clone(), value.clone());
                }
                reports.push(json!({"action": a_type, "status": "applied", "mutated": keys}));
            }
            other => {
                reports.push(json!({
                    "action": other, "status": "unsupported",
                    "error": format!("No executor for macro action '{other}' in this build; it was skipped, not simulated."),
                    "supported": [
                        "resolve_conversation", "change_priority", "change_status", "assign_team",
                        "assign_agent", "add_tags", "remove_tags", "send_message",
                        "add_private_note", "send_csat_survey", "snooze_conversation",
                        "mutate_custom_attributes"
                    ]
                }));
            }
        }
    }

    tags.sort();
    tags.dedup();
    attrs.insert("tags".to_string(), Value::Array(tags.iter().map(|tag| Value::String(tag.clone())).collect()));
    attrs.insert(
        "last_macro_executed".to_string(),
        json!({
            "macro_id": macro_id,
            "macro_name": macro_name.clone(),
            "executed_at": now.clone(),
            "executed_by": attribution,
            "actor_supplied": actor.is_some(),
        }),
    );

    // One write for the whole execution: the columns are the ones this executor
    // owns, and the values are either what the row already held or what an
    // action changed -- NULLs stay NULL.
    sqlx::query(
        "UPDATE conversations SET state = $1, priority = $2, resolution_type = $3, team_id = $4, \
         assigned_to_id = $5, snoozed_until = $6, custom_attributes = $7, last_outbound_at = $8, \
         updated_at = $9 WHERE id = $10",
    )
    .bind(&state_value)
    .bind(&priority)
    .bind(&resolution_type)
    .bind(&team_id)
    .bind(&assigned_to_id)
    .bind(&snoozed_until)
    .bind(Value::Object(attrs.clone()).to_string())
    .bind(&last_outbound_at)
    .bind(&now)
    .bind(conversation_id)
    .execute(&state.db)
    .await?;

    Ok(MacroExecution {
        macro_id: macro_id.to_string(),
        macro_name,
        conversation_id: conversation_id.to_string(),
        executed_at: now,
        actions: reports,
        deferred,
        sends,
        conversation_state: state_value,
        conversation_priority: priority,
        tags,
    })
}

/// `custom_attributes` is a JSON blob column. A row that is not a JSON object
/// is logged and started fresh rather than silently merged with a guess.
fn json_object_from_str(raw: &str) -> Map<String, Value> {
    if raw.trim().is_empty() {
        return Map::new();
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(map)) => map,
        Ok(other) => {
            tracing::warn!("conversation custom_attributes is {other}, not an object; starting fresh");
            Map::new()
        }
        Err(err) => {
            tracing::warn!("conversation custom_attributes is not JSON ({err}); starting fresh");
            Map::new()
        }
    }
}

fn string_list(action: &Value, key: &str) -> Vec<String> {
    action
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| match item {
                    Value::String(text) => Some(text.trim().to_string()).filter(|v| !v.is_empty()),
                    Value::Number(num) => Some(num.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn opt_text(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
}

/// The one way a macro action puts text on a thread.
///
/// `send_thread_message` is the name the ported executor called; the real work
/// is `crate::services::messaging`, the only module allowed to mark a message
/// `sent`. The contact, its address, the inbox and the tenant are read from the
/// database (`conversation_send_context`) rather than guessed, so an
/// incomplete conversation is refused with a message naming what is missing
/// instead of being sent to an invented recipient.
///
/// * `channel`: the conversation row's own channel wins, the contact's channel
///   is the fallback -- the same precedence `messaging::send_in_conversation`
///   uses.
/// * `sender_name`: the attribution the caller pressed the button, passed
///   through verbatim; `sender_type` is `agent`, matching the private-note and
///   outbound paths in the Python routes.
/// * `respect_business_hours`: the operator-configurable gate, so a macro run
///   outside the tenant's window is reported as deferred rather than dialled.
async fn send_thread_message(
    state: &AppState,
    conversation: &sqlx::postgres::PgRow,
    conversation_id: &str,
    body: &str,
    attribution: &str,
    is_private: bool,
) -> Result<messaging::SendOutcome, messaging::MessagingError> {
    let (contact_id, contact_value, contact_channel, inbox_id, tenant_key) =
        messaging::conversation_send_context(&state.db, conversation_id)
            .await?
            .ok_or_else(|| {
                messaging::MessagingError::NotFound(format!(
                    "Conversation {conversation_id} has no contact to send to; nothing left this \
                     process."
                ))
            })?;

    let row_channel = opt_text(conversation, "channel");
    let channel = match row_channel.as_deref() {
        Some(value) => value,
        None => contact_channel.as_str(),
    };

    let request = OutboundRequest {
        conversation_id,
        contact_id: &contact_id,
        contact_value: &contact_value,
        channel,
        body,
        subject: None,
        inbox_id: inbox_id.as_deref(),
        tenant_key: tenant_key.as_deref(),
        sender_name: attribution,
        sender_type: "agent",
        is_private,
        respect_business_hours: respect_business_hours(),
        template: None,
    };
    messaging::send_outbound(state, &request).await
}

/// Team assignment: `teams` is only queried when the operator named a team.
/// A missing table is a real, reported condition -- not an invented id.
async fn resolve_team_id(state: &AppState, name: &str) -> Result<Option<(String, String)>, String> {
    if name.is_empty() {
        return Ok(None);
    }
    let row = sqlx::query("SELECT id, name FROM teams WHERE name LIKE $1 LIMIT 1")
        .bind(format!("%{name}%"))
        .fetch_optional(&state.db)
        .await
        .map_err(|err| describe_missing(&err, "teams"))?;
    let Some(row) = row else { return Ok(None) };
    Ok(Some((text(&row, "id"), text(&row, "name"))))
}

async fn resolve_agent_id(state: &AppState, name: &str) -> Result<Option<(String, String)>, String> {
    if name.is_empty() {
        return Ok(None);
    }
    let row = sqlx::query("SELECT id, name FROM team_members WHERE name LIKE $1 LIMIT 1")
        .bind(format!("%{name}%"))
        .fetch_optional(&state.db)
        .await
        .map_err(|err| describe_missing(&err, "team_members"))?;
    let Some(row) = row else { return Ok(None) };
    Ok(Some((text(&row, "id"), text(&row, "name"))))
}

async fn record_contact_note(
    state: &AppState,
    contact_id: &str,
    author: &str,
    body: &str,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO contact_notes (id, contact_id, author_name, body, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(contact_id)
    .bind(author)
    .bind(body)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| describe_missing(&err, "contact_notes"))?;
    Ok(())
}

fn describe_missing(err: &sqlx::Error, table: &str) -> String {
    let text = err.to_string();
    if text.contains("no such table") {
        format!(
            "The {table} table does not exist in this database, so the action could not be \
             completed and was not reported as done. {text}"
        )
    } else if text.contains("no such column") {
        format!("The {table} schema is missing a column this action needs. {text}")
    } else {
        format!("Could not use the {table} table: {text}")
    }
}

/// Which macro actions this build can actually perform -- for the UI, so an
/// action is not offered if pressing it would only write a report.
pub fn supported_actions() -> &'static [&'static str] {
    &[
        "resolve_conversation",
        "change_priority",
        "change_status",
        "assign_team",
        "assign_agent",
        "add_tags",
        "remove_tags",
        "send_message",
        "add_private_note",
        "send_csat_survey",
        "snooze_conversation",
        "mutate_custom_attributes",
    ]
}

/// Whether sends from macros honour the tenant's business-hours gate. Macro
/// authors press a button during office hours, so the default is off; the env
/// var lets an operator make automation polite.
pub fn respect_business_hours() -> bool {
    env::var("MACRO_BUSINESS_HOURS_GATE")
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}
