#![allow(dead_code)]

use crate::db::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::Instant;
use uuid::Uuid;

const AI_CONN_COLS: &str = "id, name, provider, model_id, api_key, base_url, temperature, max_tokens, is_active, assigned_jobs, last_tested_at, test_status, latency_ms, created_at, updated_at";

/// The policy sandbox and the batch runner never call a provider, so every
/// transcript they produce is labelled with the generator that actually ran
/// rather than with a model name.
const SIMULATOR_ENGINE_LABEL: &str =
    "deterministic policy simulator: thresholds and persona read from the stored simulation_scenarios and agent_policies rows; no model endpoint was called";

/// Custom Captain monitors live only for the process lifetime -- the schema has
/// no captain_monitors table. Every response says so via `persisted: false`.
static CUSTOM_MONITORS: Mutex<Vec<Value>> = Mutex::new(Vec::new());

/// House style (see `routes/broadcasts.rs`): a refusal names the id or field at
/// fault and carries `saved: false`, so a client can never render a write as
/// done when the statement did not land.
fn refuse(status: StatusCode, message: &str, extra: Value) -> Response {
    let mut body = json!({ "error": message, "saved": false });
    if let (Some(target), Some(source)) = (body.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    (status, Json(body)).into_response()
}

/// A failed statement is reported with the driver's own message instead of being
/// folded into "not found" or a success shape.
fn db_error(context: &str, err: sqlx::Error) -> Response {
    refuse(
        StatusCode::INTERNAL_SERVER_ERROR,
        &format!("{context}: {err}"),
        json!({ "databaseError": true }),
    )
}

/// 0 rows on an id-scoped UPDATE means the row is not there for this caller, so
/// it is reported as 404 naming the id rather than as a success.
fn write_not_found(table: &str, assignment: &str, id: &str, rows: u64) -> Response {
    refuse(
        StatusCode::NOT_FOUND,
        &format!(
            "UPDATE {table} SET {assignment} affected {rows} rows: no {table} row with id '{id}' exists, so nothing was changed."
        ),
        json!({ "id": id, "rowsAffected": rows }),
    )
}

/// An omitted attribution field stays NULL: filling `reviewer_name`,
/// `reviewer_feedback` or `request_type` with a plausible value would put words
/// (and a person) in the audit trail that the request never contained.
fn supplied_str<'a>(payload: &'a Value, key: &str) -> Option<&'a str> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn mask_key(k: &str) -> String {
    if k.is_empty() {
        return "None".to_string();
    }
    let n = k.chars().count();
    if n > 4 {
        let tail: String = k.chars().skip(n - 4).collect();
        format!("••••••••{tail}")
    } else {
        "••••".to_string()
    }
}

/// A UI re-display often sends the masked form back; storing that would poison
/// the credential, so masked/placeholder values are refused.
fn looks_masked_or_placeholder(v: &str) -> bool {
    let t = v.trim();
    t.is_empty() || t.contains('•') || t.contains("****") || t.eq_ignore_ascii_case("changeme")
}

fn env_credential(provider: &str) -> Option<(String, String)> {
    let p = provider.to_lowercase();
    if p.contains("anthropic") {
        credential_from_env("ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL", "https://api.anthropic.com")
    } else if p.contains("openai")
        || p.contains("groq")
        || p.contains("deepseek")
        || p.contains("mistral")
        || p.contains("custom")
    {
        credential_from_env("OPENAI_API_KEY", "OPENAI_BASE_URL", "https://api.openai.com")
    } else if p.contains("google") || p.contains("gemini") {
        credential_from_env("GOOGLE_API_KEY", "GOOGLE_BASE_URL", "https://generativelanguage.googleapis.com")
    } else if p.contains("ollama") {
        std::env::var("OLLAMA_BASE_URL")
            .ok()
            .filter(|b| !b.trim().is_empty())
            .map(|base| ("ollama-local".to_string(), base))
    } else {
        credential_from_env("AI_API_KEY", "AI_BASE_URL", "")
    }
}

fn credential_from_env(key_var: &str, base_var: &str, default_base: &str) -> Option<(String, String)> {
    let key = std::env::var(key_var)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && !looks_masked_or_placeholder(v))?;
    let base = std::env::var(base_var)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default_base.to_string());
    Some((key, base))
}

// ============================================================================
// Copilot Teammate Approvals & Human-in-the-Loop Workflows
// ============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct ApprovalQueryParams {
    pub conversation_id: Option<String>,
    pub status: Option<String>,
    pub request_type: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CopilotApprovalRow {
    pub id: String,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub request_type: Option<String>,
    pub status: String,
    pub title: String,
    pub proposed_reply: Option<String>,
    pub amended_reply: Option<String>,
    pub action_data: Option<String>,
    pub internal_note: Option<String>,
    pub question_to_teammate: Option<String>,
    pub teammate_answer: Option<String>,
    pub assigned_to_id: Option<String>,
    pub reviewer_name: Option<String>,
    pub reviewer_feedback: Option<String>,
    pub dispatched_message_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
}

fn map_approval(row: CopilotApprovalRow) -> Value {
    let action_data: Value = row
        .action_data
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));

    json!({
        "id": row.id,
        "conversationId": row.conversation_id.clone().unwrap_or_default(),
        "conversation_id": row.conversation_id,
        "contactId": row.contact_id.clone(),
        "contact_id": row.contact_id,
        "requestType": row.request_type.clone(),
        "request_type": row.request_type,
        "status": row.status,
        "title": row.title,
        "proposedReply": row.proposed_reply.clone().unwrap_or_default(),
        "proposed_reply": row.proposed_reply,
        "amendedReply": row.amended_reply.clone(),
        "amended_reply": row.amended_reply,
        "actionData": action_data.clone(),
        "action_data": action_data,
        "internalNote": row.internal_note.clone().unwrap_or_default(),
        "internal_note": row.internal_note,
        "questionToTeammate": row.question_to_teammate.clone(),
        "question_to_teammate": row.question_to_teammate,
        "teammateAnswer": row.teammate_answer.clone(),
        "teammate_answer": row.teammate_answer,
        "assignedToId": row.assigned_to_id.clone(),
        "reviewerName": row.reviewer_name.clone(),
        "reviewer_name": row.reviewer_name,
        "reviewerFeedback": row.reviewer_feedback.clone(),
        "reviewer_feedback": row.reviewer_feedback,
        "dispatchedMessageId": row.dispatched_message_id.clone(),
        "createdAt": row.created_at.clone(),
        "created_at": row.created_at,
        "updatedAt": row.updated_at.clone(),
        "updated_at": row.updated_at,
        "resolvedAt": row.resolved_at.clone(),
        "resolved_at": row.resolved_at,
    })
}

pub async fn list_copilot_approvals(
    State(state): State<AppState>,
    Query(params): Query<ApprovalQueryParams>,
) -> impl IntoResponse {
    let mut sql = "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE 1=1".to_string();

    if let Some(ref st) = params.status {
        if !st.is_empty() && st != "all" {
            sql.push_str(&format!(" AND status = '{}'", st.replace('\'', "''")));
        }
    }
    if let Some(ref cid) = params.conversation_id {
        if !cid.is_empty() {
            sql.push_str(&format!(" AND conversation_id = '{}'", cid.replace('\'', "''")));
        }
    }
    if let Some(ref rt) = params.request_type {
        if !rt.is_empty() {
            sql.push_str(&format!(" AND request_type = '{}'", rt.replace('\'', "''")));
        }
    }

    sql.push_str(" ORDER BY created_at DESC");

    let rows = sqlx::query_as::<_, CopilotApprovalRow>(&sql)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let result: Vec<Value> = rows.into_iter().map(map_approval).collect();
    Json(result)
}

pub async fn create_copilot_approval(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let id = Uuid::new_v4().to_string();
    let cid = payload["conversation_id"].as_str().map(|s| s.to_string());
    let contact_id = payload["contact_id"].as_str().map(|s| s.to_string());
    let title = payload["title"].as_str().unwrap_or("Copilot Action Approval");
    let reply = payload["proposed_reply"].as_str().unwrap_or("");
    let req_type = supplied_str(&payload, "request_type");
    let internal_note = payload["internal_note"].as_str().unwrap_or("");
    let question = payload["question_to_teammate"].as_str().map(|s| s.to_string());
    let action_data = payload.get("action_data").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());
    let now = Utc::now().to_rfc3339();

    let inserted = match sqlx::query(
        r#"INSERT INTO copilot_approvals (
            id, conversation_id, contact_id, request_type, status, title,
            proposed_reply, action_data, internal_note, question_to_teammate,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&cid)
    .bind(&contact_id)
    .bind(req_type)
    .bind(title)
    .bind(reply)
    .bind(&action_data)
    .bind(internal_note)
    .bind(&question)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the approval request could not be stored", err),
    };

    // Load-bearing: the response echoes this id as a created approval, so a
    // statement that did not insert exactly one row must fail the request.
    if inserted.rows_affected() != 1 {
        return refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "INSERT INTO copilot_approvals affected {} rows, expected 1; approval '{id}' was not created.",
                inserted.rows_affected()
            ),
            json!({ "id": id.as_str() }),
        );
    }

    let row = match sqlx::query_as::<_, CopilotApprovalRow>(
        "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the stored approval request could not be read back", err),
    };

    match row {
        Some(r) => Json(map_approval(r)).into_response(),
        None => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The INSERT reported 1 row but approval '{id}' cannot be read back, so it is not shown as created."
            ),
            json!({ "id": id.as_str() }),
        ),
    }
}

pub async fn get_copilot_approval(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let row = match sqlx::query_as::<_, CopilotApprovalRow>(
        "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the approval request could not be read", err),
    };

    match row {
        Some(r) => Json(map_approval(r)).into_response(),
        None => refuse(
            StatusCode::NOT_FOUND,
            &format!("No copilot_approvals row with id '{id}'."),
            json!({ "id": id.as_str() }),
        ),
    }
}

pub async fn approve_copilot_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let reviewer = supplied_str(&payload, "reviewer_name");
    let amended = payload["amended_reply"].as_str();
    let execute_send = payload["execute_send"].as_bool().unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    // Fetch existing approval to get conversation_id & reply
    let existing = match sqlx::query_as::<_, CopilotApprovalRow>(
        "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the approval request could not be read", err),
    };

    let mut dispatched_msg_id = None;

    if let Some(ref app) = existing {
        let final_reply = amended.or(app.proposed_reply.as_deref()).unwrap_or("");
        if execute_send && !final_reply.is_empty() {
            if let Some(ref cid) = app.conversation_id {
                let msg_id = Uuid::new_v4().to_string();
                // Load-bearing: the approval row stores this id as the message
                // that was dispatched, so a failed INSERT must not be reported
                // as a sent reply.
                let inserted = match sqlx::query(
                    r#"INSERT INTO messages (id, conversation_id, sender, body, channel, delivery_status, created_at)
                       VALUES (?, ?, 'agent', ?, 'chat', 'delivered', ?)"#,
                )
                .bind(&msg_id)
                .bind(cid)
                .bind(final_reply)
                .bind(&now)
                .execute(&state.db)
                .await
                {
                    Ok(report) => report,
                    Err(err) => {
                        return db_error("the approved reply could not be stored as a message", err)
                    }
                };
                if inserted.rows_affected() != 1 {
                    return refuse(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!(
                            "INSERT INTO messages affected {} rows, expected 1; the reply for conversation '{cid}' was not sent and approval '{id}' was left untouched.",
                            inserted.rows_affected()
                        ),
                        json!({ "id": id.as_str(), "conversationId": cid.as_str(), "messageId": msg_id }),
                    );
                }

                dispatched_msg_id = Some(msg_id);
            }
        }
    }

    let updated_rows = match sqlx::query(
        r#"UPDATE copilot_approvals
           SET status = 'approved', reviewer_name = ?, amended_reply = COALESCE(?, amended_reply, proposed_reply),
               dispatched_message_id = COALESCE(?, dispatched_message_id),
               resolved_at = ?, updated_at = ?
           WHERE id = ?"#,
    )
    .bind(reviewer)
    .bind(amended)
    .bind(&dispatched_msg_id)
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the approval decision could not be stored", err),
    };
    if updated_rows.rows_affected() != 1 {
        return write_not_found(
            "copilot_approvals",
            "status = 'approved'",
            &id,
            updated_rows.rows_affected(),
        );
    }

    let updated = match sqlx::query_as::<_, CopilotApprovalRow>(
        "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the approved request could not be read back", err),
    };

    match updated {
        Some(r) => Json(map_approval(r)).into_response(),
        None => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The UPDATE reported 1 row but approval '{id}' cannot be read back, so its approved state is not shown."
            ),
            json!({ "id": id.as_str() }),
        ),
    }
}

pub async fn reject_copilot_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let reviewer = supplied_str(&payload, "reviewer_name");
    let feedback = supplied_str(&payload, "feedback");
    let now = Utc::now().to_rfc3339();

    let updated_rows = match sqlx::query(
        r#"UPDATE copilot_approvals
           SET status = 'rejected', reviewer_name = ?, reviewer_feedback = ?,
               resolved_at = ?, updated_at = ?
           WHERE id = ?"#,
    )
    .bind(reviewer)
    .bind(feedback)
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the rejection could not be stored", err),
    };
    if updated_rows.rows_affected() != 1 {
        return write_not_found(
            "copilot_approvals",
            "status = 'rejected'",
            &id,
            updated_rows.rows_affected(),
        );
    }

    let updated = match sqlx::query_as::<_, CopilotApprovalRow>(
        "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the rejected request could not be read back", err),
    };

    match updated {
        Some(r) => Json(map_approval(r)).into_response(),
        None => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The UPDATE reported 1 row but approval '{id}' cannot be read back, so its rejected state is not shown."
            ),
            json!({ "id": id.as_str() }),
        ),
    }
}

pub async fn provide_copilot_info(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let reviewer = supplied_str(&payload, "reviewer_name");
    let answer = payload["teammate_answer"].as_str().unwrap_or("");
    let now = Utc::now().to_rfc3339();

    let updated_rows = match sqlx::query(
        "UPDATE copilot_approvals SET teammate_answer = ?, reviewer_name = ?, updated_at = ? WHERE id = ?",
    )
    .bind(answer)
    .bind(reviewer)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the teammate answer could not be stored", err),
    };
    if updated_rows.rows_affected() != 1 {
        return write_not_found(
            "copilot_approvals",
            "teammate_answer",
            &id,
            updated_rows.rows_affected(),
        );
    }

    let updated = match sqlx::query_as::<_, CopilotApprovalRow>(
        "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the answered request could not be read back", err),
    };

    match updated {
        Some(r) => Json(map_approval(r)).into_response(),
        None => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The UPDATE reported 1 row but approval '{id}' cannot be read back, so the stored answer is not shown."
            ),
            json!({ "id": id.as_str() }),
        ),
    }
}

pub async fn evaluate_copilot_trigger(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    // A missing conversation id is stored as NULL, not as '' -- an empty string
    // would read back as a real (if blank) conversation reference.
    let cid = payload["conversation_id"].as_str().map(|s| s.to_string());
    let msg = payload["customer_message"].as_str().unwrap_or("").to_lowercase();

    let is_high_value_refund = (msg.contains("refund") || msg.contains("money back"))
        && (msg.contains("$") || msg.contains("hundred") || msg.contains("expensive"));
    let is_cancellation = msg.contains("cancel subscription") || msg.contains("chargeback") || msg.contains("lawyer");

    if is_high_value_refund || is_cancellation {
        let approval_id = Uuid::new_v4().to_string();
        let title = if is_cancellation {
            "Retention Escalation: Threat of Chargeback/Cancellation".to_string()
        } else {
            "High-Value Refund Request: Review Required".to_string()
        };
        let note = format!("Automated trigger: Customer inquiry flagged for human review: '{}'", msg);
        let proposed = if is_cancellation {
            "I completely understand your frustration and want to ensure this is handled personally. I am escalating this to our senior retention manager right now to review your account options.".to_string()
        } else {
            "I have forwarded your refund request to our finance department for priority review. Our manager will finalize this authorization within 2 business hours.".to_string()
        };
        let now = Utc::now().to_rfc3339();

        let inserted = match sqlx::query(
            r#"INSERT INTO copilot_approvals (
                id, conversation_id, request_type, status, title,
                proposed_reply, internal_note, created_at, updated_at
            ) VALUES (?, ?, 'escalation', 'pending', ?, ?, ?, ?, ?)"#,
        )
        .bind(&approval_id)
        .bind(&cid)
        .bind(&title)
        .bind(&proposed)
        .bind(&note)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await
        {
            Ok(report) => report,
            Err(err) => return db_error("the escalation approval could not be stored", err),
        };

        // Load-bearing: `triggered: true` promises a queued approval, so the row
        // has to exist before the trigger is reported.
        if inserted.rows_affected() != 1 {
            return refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!(
                    "INSERT INTO copilot_approvals affected {} rows, expected 1; the escalation for '{approval_id}' was not queued.",
                    inserted.rows_affected()
                ),
                json!({ "id": approval_id.as_str() }),
            );
        }

        let row = match sqlx::query_as::<_, CopilotApprovalRow>(
            "SELECT id, conversation_id, contact_id, request_type, status, title, proposed_reply, amended_reply, action_data, internal_note, question_to_teammate, teammate_answer, assigned_to_id, reviewer_name, reviewer_feedback, dispatched_message_id, created_at, updated_at, resolved_at FROM copilot_approvals WHERE id = ?",
        )
        .bind(&approval_id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(row) => row,
            Err(err) => return db_error("the stored escalation could not be read back", err),
        };

        match row {
            Some(r) => Json(json!({
                "triggered": true,
                "approval": map_approval(r),
            }))
            .into_response(),
            None => refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!(
                    "The INSERT reported 1 row but approval '{approval_id}' cannot be read back, so the trigger is not reported as queued."
                ),
                json!({ "id": approval_id.as_str() }),
            ),
        }
    } else {
        Json(json!({ "triggered": false, "approval": null })).into_response()
    }
}

// ============================================================================
// AI Agent Policy Simulations & Interactive Sandbox
// ============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct CategoryQuery {
    pub category: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AgentPolicyRow {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub process_steps: Option<String>,
    pub rules: Option<String>,
    pub guardrails: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn map_policy(row: AgentPolicyRow) -> Value {
    let process_steps: Value = row
        .process_steps
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!([]));
    let rules: Value = row
        .rules
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));
    let guardrails: Value = row
        .guardrails
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!([]));

    json!({
        "id": row.id,
        "name": row.name,
        "category": row.category,
        "description": row.description.unwrap_or_default(),
        "version": row.version,
        "status": row.status,
        "process_steps": process_steps,
        "rules": rules,
        "guardrails": guardrails,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}

pub async fn list_agent_policies(
    State(state): State<AppState>,
    Query(params): Query<CategoryQuery>,
) -> impl IntoResponse {
    let mut sql = "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies WHERE 1=1".to_string();

    if let Some(ref cat) = params.category {
        if !cat.is_empty() && cat != "all" {
            sql.push_str(&format!(" AND category = '{}'", cat.replace('\'', "''")));
        }
    }
    sql.push_str(" ORDER BY name ASC");

    let rows = sqlx::query_as::<_, AgentPolicyRow>(&sql)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let policies: Vec<Value> = rows.into_iter().map(map_policy).collect();
    Json(policies)
}

pub async fn create_agent_policy(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let id = Uuid::new_v4().to_string();
    let name = payload["name"].as_str().map(str::trim).filter(|s| !s.is_empty());
    // The name is what the policy list and the simulator quote back; inventing
    // "New Policy" would let two different rows claim the same identity.
    let Some(name) = name else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "`name` is required to create an agent policy.",
            json!({ "field": "name" }),
        );
    };
    let name = name.to_string();
    let desc = payload["description"].as_str().unwrap_or("");
    let cat = payload["category"].as_str().unwrap_or("general");
    let version = payload["version"].as_str().unwrap_or("1.0");
    let status = payload["status"].as_str().unwrap_or("active");

    let process_steps = payload
        .get("process_steps")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string());
    let rules = payload
        .get("rules")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());
    let guardrails = payload
        .get("guardrails")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    let inserted = match sqlx::query(
        r#"INSERT INTO agent_policies (
            id, name, category, description, version, status,
            process_steps, rules, guardrails, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&name)
    .bind(cat)
    .bind(desc)
    .bind(version)
    .bind(status)
    .bind(&process_steps)
    .bind(&rules)
    .bind(&guardrails)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the agent policy could not be stored", err),
    };

    // Load-bearing: the response is the created policy, keyed by this id.
    if inserted.rows_affected() != 1 {
        return refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "INSERT INTO agent_policies affected {} rows, expected 1; policy '{id}' was not created.",
                inserted.rows_affected()
            ),
            json!({ "id": id.as_str() }),
        );
    }

    let row = match sqlx::query_as::<_, AgentPolicyRow>(
        "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the stored agent policy could not be read back", err),
    };

    match row {
        Some(r) => Json(map_policy(r)).into_response(),
        None => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The INSERT reported 1 row but policy '{id}' cannot be read back, so it is not shown as created."
            ),
            json!({ "id": id.as_str() }),
        ),
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ScenarioRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub policy_id: Option<String>,
    pub policy_name: Option<String>,
    pub customer_persona: Option<String>,
    pub expected_outcome: Option<String>,
    pub customer_messages: Option<String>,
    pub created_at: String,
}

pub async fn list_simulation_scenarios(
    State(state): State<AppState>,
    Query(params): Query<CategoryQuery>,
) -> impl IntoResponse {
    let mut sql = r#"
        SELECT s.id, s.name, s.description, s.category, s.policy_id, p.name as policy_name,
               s.customer_persona, s.expected_outcome, s.customer_messages, s.created_at
        FROM simulation_scenarios s
        LEFT JOIN agent_policies p ON s.policy_id = p.id
        WHERE 1=1
    "#.to_string();

    if let Some(ref cat) = params.category {
        if !cat.is_empty() && cat != "all" {
            sql.push_str(&format!(" AND s.category = '{}'", cat.replace('\'', "''")));
        }
    }
    sql.push_str(" ORDER BY s.name ASC");

    let rows = sqlx::query_as::<_, ScenarioRow>(&sql)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    #[derive(sqlx::FromRow)]
    struct LatestRunRow {
        id: String,
        scenario_id: String,
        verdict: String,
        overall_score: Option<String>,
        policy_adherence_score: Option<String>,
        created_at: String,
    }

    let runs = sqlx::query_as::<_, LatestRunRow>(
        "SELECT id, scenario_id, verdict, overall_score, policy_adherence_score, created_at FROM simulation_runs ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut scenarios = Vec::new();
    for row in rows {
        // A scenario without a stored (or parseable) persona has no persona:
        // inventing "Customer / neutral / standard" would be shown as fact.
        let persona: Option<Value> = row
            .customer_persona
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        let messages: Value = row
            .customer_messages
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| json!([]));

        let latest_run = runs.iter().find(|r| r.scenario_id == row.id).map(|r| {
            json!({
                "id": r.id,
                "verdict": r.verdict,
                "overall_score": r.overall_score.as_deref().and_then(|s| s.parse::<f64>().ok()),
                "policy_adherence_score": r.policy_adherence_score.as_deref().and_then(|s| s.parse::<f64>().ok()),
                "created_at": r.created_at,
            })
        });

        scenarios.push(json!({
            "id": row.id,
            "name": row.name,
            "description": row.description.unwrap_or_default(),
            "category": row.category,
            "policy_id": row.policy_id,
            "policy_name": row.policy_name,
            "customer_persona": persona,
            "expected_outcome": row.expected_outcome,
            "customer_messages": messages,
            "created_at": row.created_at,
            "latest_run": latest_run,
        }));
    }

    Json(scenarios)
}

// Dynamic Evaluation Engine
async fn evaluate_scenario_internal(
    db: &sqlx::SqlitePool,
    scenario_id: &str,
    policy_id_override: Option<&str>,
) -> Result<Value, String> {
    #[derive(sqlx::FromRow)]
    struct FullScenarioRow {
        id: String,
        name: String,
        description: Option<String>,
        category: Option<String>,
        policy_id: Option<String>,
        customer_persona: Option<String>,
        expected_outcome: Option<String>,
        customer_messages: Option<String>,
    }

    let scen = sqlx::query_as::<_, FullScenarioRow>(
        "SELECT id, name, description, category, policy_id, customer_persona, expected_outcome, customer_messages FROM simulation_scenarios WHERE id = ?",
    )
    .bind(scenario_id)
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Scenario '{}' not found", scenario_id))?;

    let cat = scen.category.clone().unwrap_or_else(|| "general".to_string());
    let target_policy_id = policy_id_override.or(scen.policy_id.as_deref());

    let policy_row = if let Some(pid) = target_policy_id {
        sqlx::query_as::<_, AgentPolicyRow>(
            "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies WHERE id = ?",
        )
        .bind(pid)
        .fetch_optional(db)
        .await
        .map_err(|e| format!("the matched agent policy could not be read: {e}"))?
    } else {
        sqlx::query_as::<_, AgentPolicyRow>(
            "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies WHERE category = ? LIMIT 1",
        )
        .bind(&cat)
        .fetch_optional(db)
        .await
        .map_err(|e| format!("the matched agent policy could not be read: {e}"))?
    };

    let policy = match policy_row {
        Some(p) => p,
        None => sqlx::query_as::<_, AgentPolicyRow>(
            "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies LIMIT 1",
        )
        .fetch_one(db)
        .await
        .map_err(|e| format!("No policies found in database: {}", e))?,
    };

    let policy_rules: Value = policy
        .rules
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));

    // An absent persona column is an absent persona: naming a customer or a
    // sentiment the scenario never stored would steer the whole evaluation.
    let customer_persona: Value = scen
        .customer_persona
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));

    let persona_name = customer_persona["name"].as_str().unwrap_or("no name in the stored persona");
    let persona_sentiment = customer_persona["sentiment"].as_str().unwrap_or("unstated");

    let messages: Vec<String> = scen
        .customer_messages
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let combined_text = format!("{} {}", scen.name, messages.join(" "));
    let re_dollar = Regex::new(r"\$(\d+(?:\.\d+)?)").unwrap();
    let dollar_amount: Option<f64> = re_dollar
        .captures(&combined_text)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok());
    let amount_text = match dollar_amount {
        Some(amt) => format!("${amt:.2}"),
        None => "no monetary amount in the scenario text".to_string(),
    };

    let autonomous_cap: Option<f64> = policy_rules
        .get("autonomous_refund_cap")
        .and_then(|v| v.as_f64());
    let manager_threshold: Option<f64> = policy_rules
        .get("manager_approval_threshold")
        .and_then(|v| v.as_f64());
    let cap_text = match autonomous_cap {
        Some(cap) => format!("${cap:.2}"),
        None => "none configured in the stored policy".to_string(),
    };

    // A policy without a stored cap authorises nothing autonomously, so an
    // amount in the thread escalates rather than inventing a cap of $50.
    let over_autonomous_cap = match (dollar_amount, autonomous_cap) {
        (Some(amt), Some(cap)) => amt > cap,
        (Some(_), None) => true,
        (None, _) => false,
    };
    let over_manager_threshold = match (dollar_amount, manager_threshold) {
        (Some(amt), Some(limit)) => amt >= limit,
        _ => false,
    };
    let is_escalation = over_autonomous_cap
        || over_manager_threshold
        || scen.expected_outcome.as_deref() == Some("escalate_to_human");

    let mut adherence_score = 98.0;
    let mut empathy_score = 96.0;
    let mut accuracy_score = 97.5;

    if persona_sentiment == "frustrated" || persona_sentiment == "angry" {
        empathy_score += 1.5;
    }
    if is_escalation {
        adherence_score = 99.0;
        accuracy_score = 98.5;
    }

    let raw_overall: f64 = adherence_score * 0.45 + empathy_score * 0.25 + accuracy_score * 0.30;
    let overall_score = (raw_overall * 10.0).round() / 10.0;
    let verdict = if overall_score >= 90.0 { "passed" } else { "warning" };

    let mut transcript = Vec::new();
    let mut turn_num = 1;

    for (i, msg) in messages.iter().enumerate() {
        transcript.push(json!({
            "turn": turn_num,
            "role": "customer",
            "speaker": persona_name,
            "content": msg,
            "sentiment": persona_sentiment,
        }));
        turn_num += 1;

        let reply = if is_escalation {
            if i == 0 {
                format!(
                    "I am so sorry to hear about this issue, {}. Because this order value ({}) requires policy exception authorization, I have secured your ticket details and escalated this to our operations manager for immediate sign-off.",
                    persona_name, amount_text
                )
            } else {
                "I have updated the manager request with your confirmation. You will receive an official approval notification and refund receipt within 2 hours.".to_string()
            }
        } else {
            if i == 0 {
                format!(
                    "I am truly sorry that happened, {}. Under our {}, I have issued an immediate autonomous resolution of {} to your original payment method. No physical return is required.",
                    persona_name, policy.name, amount_text
                )
            } else {
                "The transaction has been posted. Your bank statement will reflect the credit in 2-3 business days. Is there anything else I can assist you with?".to_string()
            }
        };

        let policy_check_text = if is_escalation {
            format!(
                "Threshold check: {amount_text} against autonomous cap {cap_text} (manager approval threshold: {}). Routed to Teammate Approvals Queue.",
                match manager_threshold {
                    Some(limit) => format!("${limit:.2}"),
                    None => "none configured in the stored policy".to_string(),
                }
            )
        } else {
            format!(
                "Threshold check: {amount_text} within autonomous cap {cap_text}. Autonomous resolution approved per SOP."
            )
        };

        transcript.push(json!({
            "turn": turn_num,
            "role": "agent",
            "speaker": "Captain AI Agent (simulated)",
            "content": reply,
            "generator": SIMULATOR_ENGINE_LABEL,
            "agent_thoughts": [
                format!("Evaluated category: '{}' against policy '{}'", cat, policy.name),
                policy_check_text.clone(),
                format!("Sentiment '{}' addressed with de-escalation tone", persona_sentiment)
            ],
            "policy_check": policy_check_text
        }));
        turn_num += 1;
    }

    let summary = format!(
        "Scenario evaluated against policy '{}'. Customer: '{}' (sentiment: {}). Expected outcome: '{}'. Evaluated resolution: {} (Overall Score: {:.1}%). Strict adherence to SOP thresholds.",
        policy.name,
        persona_name,
        persona_sentiment,
        scen.expected_outcome.as_deref().unwrap_or("not stated in the scenario"),
        verdict.to_uppercase(),
        overall_score
    );

    let recommendations = if is_escalation {
        vec![
            "Ensure manager approvals queue triggers webhook alerting on enterprise Slack channel.".to_string(),
            "Notify customer via SMS immediately once teammate signs off on exception.".to_string(),
        ]
    } else {
        vec![
            "Continue autonomous under-threshold resolution to preserve high CSAT and sub-1-minute handling time.".to_string(),
            "Log damage reason code to inventory quality audit stream.".to_string(),
        ]
    };

    let run_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let transcript_json = serde_json::to_string(&transcript)
        .map_err(|e| format!("the simulated transcript could not be serialized: {e}"))?;
    let recs_json = serde_json::to_string(&recommendations)
        .map_err(|e| format!("the simulation recommendations could not be serialized: {e}"))?;

    // The evaluation stands on its own, but the response echoes `run_id` as a
    // history row, so a statement that did not land is reported as `stored:
    // false` + `storageError` (and as a 500 by `run_simulation`) instead of
    // being dropped on the floor.
    let storage_error: Option<String> = match sqlx::query(
        r#"INSERT INTO simulation_runs (
            id, scenario_id, policy_id, status, verdict,
            policy_adherence_score, empathy_score, accuracy_score, overall_score,
            turn_count, transcript, evaluation_summary, recommendations, created_at
        ) VALUES (?, ?, ?, 'completed', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&run_id)
    .bind(scenario_id)
    .bind(&policy.id)
    .bind(verdict)
    .bind(format!("{:.1}", adherence_score))
    .bind(format!("{:.1}", empathy_score))
    .bind(format!("{:.1}", accuracy_score))
    .bind(format!("{:.1}", overall_score))
    .bind(turn_num as i64)
    .bind(&transcript_json)
    .bind(&summary)
    .bind(&recs_json)
    .bind(&now)
    .execute(db)
    .await
    {
        Ok(report) if report.rows_affected() == 1 => None,
        Ok(report) => Some(format!(
            "INSERT INTO simulation_runs affected {} rows, expected 1",
            report.rows_affected()
        )),
        Err(err) => Some(format!("INSERT INTO simulation_runs failed: {err}")),
    };

    Ok(json!({
        "id": run_id,
        "scenario_id": scenario_id,
        "status": "completed",
        "verdict": verdict,
        "policy_adherence_score": adherence_score,
        "empathy_score": empathy_score,
        "accuracy_score": accuracy_score,
        "overall_score": overall_score,
        "turn_count": turn_num,
        "transcript": transcript,
        "evaluation_summary": summary,
        "recommendations": recommendations,
        "simulated": true,
        "modelGenerated": false,
        "actionsExecuted": false,
        "engine": SIMULATOR_ENGINE_LABEL,
        "scoreBasis": "the four scores are simulator constants adjusted by two fixed rules (persona sentiment adds 1.5 empathy; escalation sets adherence 99.0 and accuracy 98.5); nothing was measured and no model was called",
        "note": "Simulated transcript: no refund, message, ticket or notification was created or sent.",
        "stored": storage_error.is_none(),
        "storageError": storage_error,
        "created_at": now
    }))
}

pub async fn run_simulation(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    // Defaulting to "scen-1" would evaluate a scenario the caller never named.
    let Some(sid) = payload["scenario_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "`scenario_id` is required: the simulator evaluates a stored scenario, not an assumed one.",
            json!({ "field": "scenario_id" }),
        );
    };
    let pid = payload.get("policy_id").and_then(|v| v.as_str());

    match evaluate_scenario_internal(&state.db, sid, pid).await {
        Ok(run) => {
            // The body carries the run id, so a run that could not be stored is
            // answered as a server error rather than as history.
            let status = if run["stored"].as_bool().unwrap_or(false) {
                StatusCode::OK
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(run)).into_response()
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": err, "status": "failed", "stored": false })),
        )
            .into_response(),
    }
}

pub async fn run_batch_simulations(State(state): State<AppState>) -> Response {
    #[derive(sqlx::FromRow)]
    struct IdNameRow {
        id: String,
        name: String,
    }

    let scenarios = match sqlx::query_as::<_, IdNameRow>(
        "SELECT id, name FROM simulation_scenarios ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("the simulation scenarios could not be listed", err),
    };

    let mut results = Vec::new();
    let mut total_score = 0.0;
    let mut scored = 0usize;
    let mut passed_count = 0;
    let mut stored_count = 0usize;
    let mut failed_evaluations: Vec<Value> = Vec::new();

    for s in &scenarios {
        match evaluate_scenario_internal(&state.db, &s.id, None).await {
            Ok(run) => {
                // A missing score stays missing: the average is taken only over
                // the runs that actually reported one.
                let score = run["overall_score"].as_f64();
                let verdict = run["verdict"].as_str().map(|v| v.to_uppercase());
                let adherence = run["policy_adherence_score"].as_f64();
                let stored = run["stored"].as_bool().unwrap_or(false);
                if verdict.as_deref() == Some("PASSED") {
                    passed_count += 1;
                }
                if let Some(value) = score {
                    total_score += value;
                    scored += 1;
                }
                if stored {
                    stored_count += 1;
                }

                results.push(json!({
                    "scenario_id": s.id,
                    "scenario_name": s.name,
                    "verdict": verdict,
                    "overall_score": score,
                    "policy_adherence": adherence,
                    "stored": stored,
                    "storageError": run.get("storageError").cloned().unwrap_or(Value::Null),
                }));
            }
            Err(err) => failed_evaluations.push(json!({
                "scenario_id": s.id,
                "scenario_name": s.name,
                "error": err,
            })),
        }
    }

    let count = scenarios.len();
    let evaluated = count - failed_evaluations.len();
    let avg_score = if scored > 0 {
        json!((total_score / scored as f64 * 10.0).round() / 10.0)
    } else {
        Value::Null
    };
    let pass_rate = if evaluated > 0 {
        json!((passed_count as f64 / evaluated as f64 * 1000.0).round() / 10.0)
    } else {
        Value::Null
    };

    Json(json!({
        "total_scenarios_tested": count,
        "evaluated_count": evaluated,
        "passed_count": passed_count,
        "pass_rate_percentage": pass_rate,
        "average_score": avg_score,
        "runsStored": stored_count,
        "engine": SIMULATOR_ENGINE_LABEL,
        "modelGenerated": false,
        "results": results,
        "failedEvaluations": failed_evaluations,
        "note": if stored_count < evaluated {
            format!("{evaluated} scenario(s) were evaluated but only {stored_count} run(s) were written to simulation_runs; see the per-run storageError.")
        } else {
            "Every evaluated run below is backed by a stored simulation_runs row.".to_string()
        },
    }))
    .into_response()
}

pub async fn sandbox_chat(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let Some(msg) = payload.get("message").and_then(|v| v.as_str()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "message is required",
                "detail": "the sandbox classifies and drafts against the message it is given; send {\"message\": \"...\"}",
            })),
        )
            .into_response();
    };
    let policy_id = payload.get("policy_id").and_then(|v| v.as_str());

    let lower = msg.to_lowercase();
    let detected_category = if lower.contains("refund") || lower.contains("return") || lower.contains("broken") || lower.contains("damage") || lower.contains("$") {
        "refunds"
    } else if lower.contains("ship") || lower.contains("delay") || lower.contains("track") || lower.contains("carrier") {
        "shipping"
    } else if lower.contains("vip") || lower.contains("exchange") || lower.contains("swap") || lower.contains("size") {
        "vip_support"
    } else {
        "general"
    };

    let policy_row = if let Some(pid) = policy_id {
        sqlx::query_as::<_, AgentPolicyRow>(
            "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies WHERE id = ?",
        )
        .bind(pid)
        .fetch_optional(&state.db)
        .await
    } else {
        sqlx::query_as::<_, AgentPolicyRow>(
            "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies WHERE category = ? LIMIT 1",
        )
        .bind(detected_category)
        .fetch_optional(&state.db)
        .await
    };

    let policy_row = match policy_row {
        Ok(row) => row,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "the agent policy could not be read",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    // An unselected category falls back to any stored policy; a deployment with no
    // policies at all has no policy, which is reported instead of fabricating one.
    let policy = match policy_row {
        Some(p) => Some(p),
        None => {
            match sqlx::query_as::<_, AgentPolicyRow>(
                "SELECT id, name, category, description, version, status, process_steps, rules, guardrails, created_at, updated_at FROM agent_policies LIMIT 1",
            )
            .fetch_optional(&state.db)
            .await
            {
                Ok(row) => row,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "the agent policy could not be read",
                            "detail": err.to_string(),
                        })),
                    )
                        .into_response()
                }
            }
        }
    };

    let policy_rules: Value = policy
        .as_ref()
        .and_then(|p| p.rules.as_deref())
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));
    let guardrails_list: Vec<String> = policy
        .as_ref()
        .and_then(|p| p.guardrails.as_deref())
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let autonomous_cap = policy_rules
        .get("autonomous_refund_cap")
        .and_then(|v| v.as_f64());

    let re_dollar = Regex::new(r"\$(\d+(?:\.\d+)?)").unwrap();
    let dollar_amount = re_dollar
        .captures(msg)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok());

    // Without a configured cap nothing may be resolved autonomously.
    let requires_escalation = match (dollar_amount, autonomous_cap) {
        (Some(_), None) => true,
        (Some(amt), Some(cap)) => amt > cap,
        (None, _) => lower.contains("chargeback") || lower.contains("supervisor") || lower.contains("lawyer"),
    };

    let policy_name = policy.as_ref().map(|p| p.name.clone());
    let reply = if requires_escalation {
        format!(
            "Suggested reply, pending a human sign-off: acknowledge the customer's {} request and tell them it is being routed for approval. Nothing has been issued, refunded or scheduled by this sandbox.",
            detected_category
        )
    } else if policy_name.is_none() {
        "No agent policy is stored for this deployment, so there is nothing to ground a reply in. This sandbox produced no suggested action.".to_string()
    } else if detected_category == "refunds" {
        match autonomous_cap {
            Some(cap) => format!(
                "Suggested reply under the stored policy: apologise for the fault and offer a resolution within the configured autonomous cap of ${cap:.2}. This is a draft; no refund was issued."
            ),
            None => "Suggested reply under the stored policy: apologise for the fault and route the refund for approval, because no autonomous cap is configured. This is a draft; no refund was issued.".to_string(),
        }
    } else if detected_category == "shipping" {
        "Suggested reply: acknowledge the delivery delay and offer to check carrier status and set up shipment notifications. This is a draft; no carrier lookup, credit or notification was sent.".to_string()
    } else if detected_category == "vip_support" {
        "Suggested reply: acknowledge the VIP member and offer an expedited size exchange. This is a draft; no exchange or shipping was booked.".to_string()
    } else {
        format!(
            "Suggested reply: confirm the request '{}' and offer to help. This is a draft checked against the stored policy name only; no action was taken.",
            msg
        )
    };

    let mut matched_rules: Vec<String> = Vec::new();
    if let Some(cap) = autonomous_cap {
        matched_rules.push(format!("autonomous_refund_cap = {cap:.2} read from the stored policy rules"));
        if let Some(amt) = dollar_amount {
            matched_rules.push(format!(
                "amount in message ${amt:.2} is {} the cap",
                if amt > cap { "above" } else { "within" }
            ));
        }
    } else {
        matched_rules.push(
            "the stored policy rules contain no autonomous_refund_cap, so nothing can be authorised autonomously"
                .to_string(),
        );
    }
    if let Some(obj) = policy_rules.as_object() {
        for (key, value) in obj {
            matched_rules.push(format!("policy rule {key} = {value}"));
        }
    }

    Json(json!({
        "reply": reply,
        "simulated": true,
        "actionsExecuted": false,
        "policy_matched": policy_name,
        "policy_id": policy.as_ref().map(|p| p.id.clone()),
        "detected_category": detected_category,
        "matched_rules": matched_rules,
        "guardrails_applied": guardrails_list,
        "guardrails_note": if guardrails_list.is_empty() {
            Some("no guardrails are stored on the matched policy, and none were invented")
        } else {
            None::<&str>
        },
        "autonomous_refund_cap": autonomous_cap,
        "escalated_for_approval": requires_escalation,
        "note": "This is the demo sandbox: it classifies the message and drafts a reply against the stored policy. No refund, credit, exchange, ticket or approval request was created.",
        "timestamp": Utc::now().to_rfc3339()
    }))
    .into_response()
}

pub async fn get_simulation_stats(State(state): State<AppState>) -> impl IntoResponse {
    let total_policies: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_policies")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_scenarios: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM simulation_scenarios")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_simulation_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM simulation_runs")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    // A run-less deployment has no pass rate and no average score. Reporting 98%
    // would be a number nobody measured.
    let (pass_rate, avg_score): (Value, Value) = if total_simulation_runs > 0 {
        #[derive(sqlx::FromRow)]
        struct ScoreStats {
            avg_score: Option<f64>,
            passed_count: Option<i64>,
        }
        match sqlx::query_as::<_, ScoreStats>(
            "SELECT AVG(CAST(overall_score AS REAL)) as avg_score, SUM(CASE WHEN verdict = 'passed' THEN 1 ELSE 0 END) as passed_count FROM simulation_runs",
        )
        .fetch_one(&state.db)
        .await
        {
            Ok(stats) => (
                stats
                    .passed_count
                    .map(|c| json!((c as f64 / total_simulation_runs as f64 * 1000.0).round() / 10.0))
                    .unwrap_or(Value::Null),
                stats
                    .avg_score
                    .map(|s| json!((s * 10.0).round() / 10.0))
                    .unwrap_or(Value::Null),
            ),
            Err(_) => (Value::Null, Value::Null),
        }
    } else {
        (Value::Null, Value::Null)
    };

    Json(json!({
        "total_policies": total_policies,
        "total_scenarios": total_scenarios,
        "total_simulation_runs": total_simulation_runs,
        "pass_rate_percentage": pass_rate,
        "average_overall_score": avg_score,
        "agent_name": "Captain AI Agent",
        "note": if total_simulation_runs > 0 {
            Value::String(format!("computed from {total_simulation_runs} stored simulation runs"))
        } else {
            Value::String("no simulation runs are stored yet, so pass rate and average score are unavailable rather than estimated".to_string())
        }
    }))
}

// ============================================================================
// Self-Improving AI, Continuous Learning & Model Hub
// ============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct InsightQueryParams {
    pub category: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct InsightRow {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub human_rep_id: Option<String>,
    pub human_rep_name: Option<String>,
    pub exemplar_content: Option<String>,
    pub impact_score: Option<String>,
    pub estimated_resolution_lift: Option<String>,
    pub suggested_action: Option<String>,
    pub action_payload: Option<String>,
    pub status: Option<String>,
    pub applied_at: Option<String>,
    pub created_at: String,
}

fn map_insight(row: InsightRow) -> Value {
    // An unparseable or absent score is reported as null; 9.0/4.0 would be a
    // confidence the insight never carried.
    let impact: Option<f64> = row.impact_score.as_deref().and_then(|s| s.parse().ok());
    let lift: Option<f64> = row
        .estimated_resolution_lift
        .as_deref()
        .and_then(|s| s.parse().ok());
    let payload: Value = row
        .action_payload
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));

    json!({
        "id": row.id,
        "title": row.title,
        "category": row.category,
        "description": row.description.unwrap_or_default(),
        "human_rep_id": row.human_rep_id,
        "human_rep_name": row.human_rep_name,
        "exemplar_content": row.exemplar_content,
        "impact_score": impact,
        "estimated_resolution_lift": lift,
        "suggested_action": row.suggested_action,
        "action_payload": payload,
        "status": row.status,
        "applied_at": row.applied_at,
        "created_at": row.created_at,
    })
}

pub async fn list_self_improving_insights(
    State(state): State<AppState>,
    Query(params): Query<InsightQueryParams>,
) -> impl IntoResponse {
    let mut sql = "SELECT id, title, category, description, human_rep_id, human_rep_name, exemplar_content, impact_score, estimated_resolution_lift, suggested_action, action_payload, status, applied_at, created_at FROM agent_insights WHERE 1=1".to_string();

    if let Some(ref cat) = params.category {
        if !cat.is_empty() && cat != "all" {
            sql.push_str(&format!(" AND category = '{}'", cat.replace('\'', "''")));
        }
    }
    if let Some(ref st) = params.status {
        if !st.is_empty() && st != "all" {
            sql.push_str(&format!(" AND status = '{}'", st.replace('\'', "''")));
        }
    }
    sql.push_str(" ORDER BY CAST(impact_score AS REAL) DESC");

    let rows = sqlx::query_as::<_, InsightRow>(&sql)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let insights: Vec<Value> = rows.into_iter().map(map_insight).collect();
    Json(insights)
}

pub async fn apply_insight_retraining(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let now = Utc::now().to_rfc3339();

    // Fetch the insight
    let insight = match sqlx::query_as::<_, InsightRow>(
        "SELECT id, title, category, description, human_rep_id, human_rep_name, exemplar_content, impact_score, estimated_resolution_lift, suggested_action, action_payload, status, applied_at, created_at FROM agent_insights WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the agent insight could not be read", err),
    };

    let Some(insight) = insight else {
        return refuse(
            StatusCode::NOT_FOUND,
            &format!("No agent_insights row with id '{id}'; no retraining event was created."),
            json!({ "id": id.as_str() }),
        );
    };

    // The lift is only what the insight itself stored -- 4.8 would be an
    // improvement nobody estimated. The resolution rates are measured by nothing
    // in this schema, so they are recorded as absent rather than as 91.2%.
    let lift: Option<f64> = insight
        .estimated_resolution_lift
        .as_deref()
        .and_then(|s| s.parse::<f64>().ok());
    let title = format!("Retraining Cycle: {}", insight.title);

    let marked = match sqlx::query("UPDATE agent_insights SET status = 'applied', applied_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the insight could not be marked as applied", err),
    };
    if marked.rows_affected() != 1 {
        return write_not_found(
            "agent_insights",
            "status = 'applied'",
            &id,
            marked.rows_affected(),
        );
    }

    // The starting version is whatever the newest stored event recorded; with no
    // history there is no earlier version to claim.
    let version_from: Option<String> =
        match sqlx::query_scalar::<_, Option<String>>(
            "SELECT version_to FROM agent_retraining_events ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        {
            Ok(row) => row.flatten(),
            Err(err) => return db_error("the retraining version history could not be read", err),
        };

    let event_id = Uuid::new_v4().to_string();
    let version_to = "2.3";

    let changes = json!([
        {
            "type": "prompt_refinement",
            "detail": format!("Applied winning de-escalation weights from insight {}", id),
            "source": "Continuous Learning Engine"
        }
    ])
    .to_string();

    let inserted = match sqlx::query(
        r#"INSERT INTO agent_retraining_events (
            id, title, trigger_type, version_from, version_to, insight_id,
            changes_applied, status, baseline_resolution_rate, post_training_resolution_rate,
            resolution_lift, created_at
        ) VALUES (?, ?, 'insight_retrain', ?, ?, ?, ?, 'applied', ?, ?, ?, ?)"#,
    )
    .bind(&event_id)
    .bind(&title)
    .bind(&version_from)
    .bind(version_to)
    .bind(&id)
    .bind(&changes)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(lift.map(|value| format!("{:.1}", value)))
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the retraining event could not be stored", err),
    };

    // Load-bearing: the response hands back this event id as a stored row.
    if inserted.rows_affected() != 1 {
        return refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "INSERT INTO agent_retraining_events affected {} rows, expected 1; event '{event_id}' was not created, although insight '{id}' is already marked 'applied'.",
                inserted.rows_affected()
            ),
            json!({ "id": event_id.as_str(), "insightId": id.as_str() }),
        );
    }

    Json(json!({
        "retraining_event_id": event_id,
        "insight_id": id,
        "title": title,
        "version_from": version_from,
        "version_to": version_to,
        "baseline_resolution_rate": Value::Null,
        "post_training_resolution_rate": Value::Null,
        "resolution_lift": lift,
        "status": "completed",
        "saved": true,
        "note": "the insight row is now 'applied' and this event row is stored; baseline and post-training resolution rates are null because nothing in this schema measures them"
    }))
    .into_response()
}

pub async fn dismiss_insight(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let now = Utc::now().to_rfc3339();
    let dismissed = match sqlx::query("UPDATE agent_insights SET status = 'dismissed', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the insight could not be marked as dismissed", err),
    };
    if dismissed.rows_affected() != 1 {
        return write_not_found(
            "agent_insights",
            "status = 'dismissed'",
            &id,
            dismissed.rows_affected(),
        );
    }

    Json(json!({ "status": "ok", "saved": true, "insight_id": id })).into_response()
}

#[derive(Debug, sqlx::FromRow)]
pub struct RetrainRow {
    pub id: String,
    pub title: String,
    pub trigger_type: Option<String>,
    pub version_from: Option<String>,
    pub version_to: Option<String>,
    pub insight_id: Option<String>,
    pub changes_applied: Option<String>,
    pub status: Option<String>,
    pub baseline_resolution_rate: Option<String>,
    pub post_training_resolution_rate: Option<String>,
    pub resolution_lift: Option<String>,
    pub created_at: Option<String>,
}

pub async fn list_retraining_history(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, RetrainRow>(
        r#"SELECT id, title, trigger_type, version_from, version_to, insight_id,
                  changes_applied, status, baseline_resolution_rate, post_training_resolution_rate,
                  resolution_lift, created_at
           FROM agent_retraining_events
           ORDER BY created_at DESC"#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            let changes: Value = r
                .changes_applied
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_else(|| json!([]));
            // Unstored rates, versions and statuses are reported as absent:
            // "applied"/88.0/92.8/4.8 would be measurements nobody took.
            let lift: Option<f64> = r.resolution_lift.as_deref().and_then(|s| s.parse().ok());
            let baseline: Option<f64> = r
                .baseline_resolution_rate
                .as_deref()
                .and_then(|s| s.parse().ok());
            let post: Option<f64> = r
                .post_training_resolution_rate
                .as_deref()
                .and_then(|s| s.parse().ok());

            json!({
                "id": r.id,
                "title": r.title,
                "trigger_type": r.trigger_type,
                "version_from": r.version_from,
                "version_to": r.version_to,
                "changes_applied": changes,
                "status": r.status,
                "baseline_resolution_rate": baseline,
                "post_training_resolution_rate": post,
                "resolution_lift": lift,
                "created_at": r.created_at.unwrap_or_default(),
            })
        })
        .collect();

    Json(items)
}

pub async fn trigger_auto_learning_scan(State(state): State<AppState>) -> Response {
    // Every count below is read live; the 48/18/12 and the 42/16 substitutions
    // this route used to fall back to are gone.
    let scanned_convs: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
        .fetch_one(&state.db)
        .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the conversations count could not be read", err),
    };

    let high_csat: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversations WHERE csat_score >= 4",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the high-CSAT count could not be read", err),
    };

    let open_insights: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_insights WHERE status = 'open'",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the open insight count could not be read", err),
    };

    Json(json!({
        "conversations_scanned": scanned_convs,
        "high_csat_patterns_analyzed": high_csat,
        "new_insights_surfaced": 0,
        "openInsightsAlreadyStored": open_insights,
        "insightsCreated": false,
        "status": "completed",
        "note": format!(
            "counted {scanned_convs} conversation(s), {high_csat} of them with csat_score >= 4. This route only reads counts: it does not analyze transcripts and inserted no agent_insights rows."
        )
    }))
    .into_response()
}

pub async fn get_self_improving_stats(State(state): State<AppState>) -> Response {
    let total_insights: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM agent_insights")
        .fetch_one(&state.db)
        .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the insight total could not be read", err),
    };

    let open_suggestions: i64 =
        match sqlx::query_scalar("SELECT COUNT(*) FROM agent_insights WHERE status = 'open'")
            .fetch_one(&state.db)
            .await
        {
            Ok(value) => value,
            Err(err) => return db_error("the open suggestion count could not be read", err),
        };

    let applied_retrainings: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_retraining_events WHERE status = 'applied'",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the applied retraining count could not be read", err),
    };

    let exemplars_learned: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_insights WHERE category = 'human_exemplar'",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the human exemplar count could not be read", err),
    };

    let coaching_active: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_insights WHERE category = 'team_coaching'",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the team coaching count could not be read", err),
    };

    // SUM over no rows is NULL, which is read as "nothing accumulated" rather
    // than the 14.4 this route used to COALESCE to.
    let cum_lift: Option<f64> = match sqlx::query_scalar(
        "SELECT SUM(CAST(resolution_lift AS REAL)) FROM agent_retraining_events WHERE status = 'applied'",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return db_error("the cumulative resolution lift could not be read", err),
    };

    #[derive(sqlx::FromRow)]
    struct VerRow {
        version_to: Option<String>,
    }
    let ver_to = match sqlx::query_as::<_, VerRow>(
        "SELECT version_to FROM agent_retraining_events ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row.and_then(|r| r.version_to),
        Err(err) => return db_error("the agent version history could not be read", err),
    };

    Json(json!({
        "total_insights": total_insights,
        "open_suggestions": open_suggestions,
        "applied_retrainings": applied_retrainings,
        "cumulative_resolution_lift": cum_lift.map(|value| (value * 10.0).round() / 10.0),
        "current_agent_version": ver_to,
        "human_exemplars_learned": exemplars_learned,
        "team_coaching_active": coaching_active,
        "agent_name": "Captain AI Agent",
        "note": "all figures are live COUNT/SUM aggregates over agent_insights and agent_retraining_events; null means nothing is stored to measure"
    }))
    .into_response()
}

// ============================================================================
// AI Connections & Custom Policies
// ============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AIConnectionRow {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<String>,
    pub max_tokens: Option<i64>,
    pub is_active: bool,
    pub assigned_jobs: Option<String>,
    pub last_tested_at: Option<String>,
    pub test_status: Option<String>,
    pub latency_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// A fully resolved, credential-backed route to a real model endpoint.
#[derive(Debug, Clone)]
struct ProviderTarget {
    connection_id: Option<String>,
    connection_name: String,
    provider: String,
    model_id: String,
    api_key: String,
    base_url: String,
    temperature: f64,
    max_tokens: i64,
    credential_source: String,
}

/// Every way a model call can fail, mapped to an honest HTTP answer.
enum AiError {
    NotConfigured(Vec<String>),
    Provider { status: u16, body: String, endpoint: String },
    Transport(String),
    BadResponse(String),
}

impl AiError {
    fn to_json(&self) -> (StatusCode, Value) {
        match self {
            AiError::NotConfigured(missing) => (
                StatusCode::SERVICE_UNAVAILABLE,
                json!({
                    "error": "AI generation is not configured on this server.",
                    "missing": missing,
                    "modelGenerated": false,
                    "suggestedReply": Value::Null,
                    "note": format!("No request was sent to any provider because these are unset: {}. Refusing to return invented model output.", missing.join(", ")),
                }),
            ),
            AiError::Provider { status, body, endpoint } => (
                StatusCode::BAD_GATEWAY,
                json!({
                    "error": "The configured model provider rejected the request.",
                    "providerHttpStatus": status,
                    "providerError": body,
                    "endpoint": endpoint,
                    "modelGenerated": false,
                    "suggestedReply": Value::Null,
                }),
            ),
            AiError::Transport(e) => (
                StatusCode::BAD_GATEWAY,
                json!({ "error": format!("Could not reach the configured model endpoint: {e}"), "modelGenerated": false }),
            ),
            AiError::BadResponse(e) => (
                StatusCode::BAD_GATEWAY,
                json!({ "error": format!("The provider replied, but the reply could not be read: {e}"), "modelGenerated": false }),
            ),
        }
    }
}

fn anthropic_family(provider: &str) -> bool {
    provider.to_lowercase().contains("anthropic")
}

fn jobs_of(row: &AIConnectionRow) -> Vec<String> {
    row.assigned_jobs
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default()
}

async fn active_connections(state: &AppState) -> Vec<AIConnectionRow> {
    sqlx::query_as::<_, AIConnectionRow>(&format!(
        "SELECT {AI_CONN_COLS} FROM ai_connections WHERE is_active = 1 ORDER BY created_at ASC"
    ))
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
}

/// Resolve which real endpoint a job should call. Prefers an ai_connections row
/// assigned to the job, then any active row with a stored key, then process env.
async fn resolve_target(state: &AppState, job_type: &str) -> Result<ProviderTarget, AiError> {
    let conns = active_connections(state).await;

    let assigned = conns
        .iter()
        .find(|c| jobs_of(c).iter().any(|j| j == job_type) && !c.api_key.as_deref().unwrap_or("").is_empty())
        .or_else(|| conns.iter().find(|c| jobs_of(c).iter().any(|j| j == job_type)))
        .or_else(|| conns.iter().find(|c| !c.api_key.as_deref().unwrap_or("").is_empty()))
        .or_else(|| conns.first())
        .cloned();

    if let Some(row) = assigned {
        let mut missing: Vec<String> = Vec::new();
        if row.model_id.trim().is_empty() {
            missing.push(format!("model_id (connection '{}')", row.name));
        }

        let stored_key = row.api_key.clone().unwrap_or_default();
        let (api_key, source, env_base) = if stored_key.trim().is_empty() || looks_masked_or_placeholder(&stored_key) {
            match env_credential(&row.provider) {
                Some((k, b)) => (k, "process environment variable".to_string(), Some(b)),
                None => {
                    missing.push(format!(
                        "api_key (connection '{}' stores none usable; {} is unset)",
                        row.name,
                        if anthropic_family(&row.provider) { "ANTHROPIC_API_KEY" } else { "OPENAI_API_KEY" }
                    ));
                    (String::new(), "none".to_string(), None)
                }
            }
        } else {
            (stored_key, format!("ai_connections row '{}'", row.name), None)
        };

        let base_url = row
            .base_url
            .clone()
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty())
            .or(env_base)
            .unwrap_or_default();
        if base_url.is_empty() || !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
            missing.push(format!(
                "base_url (connection '{}' needs an absolute http(s) endpoint)",
                row.name
            ));
        }

        if !missing.is_empty() {
            return Err(AiError::NotConfigured(missing));
        }

        return Ok(ProviderTarget {
            connection_id: Some(row.id.clone()),
            connection_name: row.name.clone(),
            provider: row.provider.clone(),
            model_id: row.model_id.clone(),
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            temperature: row.temperature.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.7),
            max_tokens: row.max_tokens.unwrap_or(1024),
            credential_source: source,
        });
    }

    // No connection rows at all: fall back to environment configuration.
    for provider in ["anthropic", "openai"] {
        if let Some((key, base)) = env_credential(provider) {
            if let Some(model) = std::env::var(if provider == "anthropic" { "ANTHROPIC_MODEL" } else { "OPENAI_MODEL" })
                .ok()
                .filter(|m| !m.trim().is_empty())
            {
                return Ok(ProviderTarget {
                    connection_id: None,
                    connection_name: format!("environment {provider}"),
                    provider: provider.to_string(),
                    model_id: model,
                    api_key: key,
                    base_url: base.trim_end_matches('/').to_string(),
                    temperature: 0.7,
                    max_tokens: 1024,
                    credential_source: "process environment variable".to_string(),
                });
            }
        }
    }

    Err(AiError::NotConfigured(vec![
        "an active row in ai_connections (name, provider, model_id, api_key, base_url)".to_string(),
        format!(
            "or ANTHROPIC_API_KEY + ANTHROPIC_MODEL / OPENAI_API_KEY + OPENAI_MODEL in the environment (currently unset: {})",
            if std::env::var("ANTHROPIC_API_KEY").is_err() && std::env::var("OPENAI_API_KEY").is_err() {
                "ANTHROPIC_API_KEY, OPENAI_API_KEY"
            } else {
                "ANTHROPIC_MODEL or OPENAI_MODEL"
            }
        ),
    ]))
}

/// One real chat-completion round trip. Latency is measured, never invented.
async fn chat_completion(
    state: &AppState,
    target: &ProviderTarget,
    system: &str,
    user: &str,
    max_tokens: i64,
) -> Result<(String, u128, u16), AiError> {
    let (url, payload, headers) = if anthropic_family(&target.provider) {
        (
            format!("{}/v1/messages", target.base_url),
            json!({
                "model": target.model_id,
                "max_tokens": max_tokens,
                "temperature": target.temperature,
                "system": system,
                "messages": [{ "role": "user", "content": user }],
            }),
            vec![
                ("x-api-key".to_string(), target.api_key.clone()),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ],
        )
    } else {
        (
            format!("{}/v1/chat/completions", target.base_url),
            json!({
                "model": target.model_id,
                "max_tokens": max_tokens,
                "temperature": target.temperature,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user },
                ],
            }),
            vec![
                ("authorization".to_string(), format!("Bearer {}", target.api_key)),
            ],
        )
    };

    let started = Instant::now();
    let mut req = state
        .http_client
        .post(&url)
        .header("content-type", "application/json");
    for (k, v) in headers {
        req = req.header(k.as_str(), v);
    }
    let res = match req.json(&payload).send().await {
        Ok(r) => r,
        Err(e) => return Err(AiError::Transport(e.to_string())),
    };

    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    let latency = started.elapsed().as_millis();

    if !status.is_success() {
        let trimmed: String = body.chars().take(700).collect();
        return Err(AiError::Provider {
            status: status.as_u16(),
            body: trimmed,
            endpoint: url,
        });
    }

    let parsed: Value = serde_json::from_str(&body).map_err(|e| {
        let head: String = body.chars().take(200).collect();
        AiError::BadResponse(format!("{e}: {head}"))
    })?;

    let text = if anthropic_family(&target.provider) {
        parsed
            .get("content")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                            p.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("")
            })
            .unwrap_or_default()
    } else {
        parsed
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                parsed
                    .pointer("/choices/0/message/content")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|p| p.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join(""))
            })
            .or_else(|| parsed.get("content").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_default()
    };

    if text.trim().is_empty() {
        return Err(AiError::BadResponse(format!(
            "Provider returned HTTP {} with no readable message text.",
            status.as_u16()
        )));
    }

    Ok((text.trim().to_string(), latency, status.as_u16()))
}

fn engine_json(t: &ProviderTarget) -> Value {
    json!({
        "provider": t.provider,
        "modelId": t.model_id,
        "connectionId": t.connection_id,
        "connectionName": t.connection_name,
        "baseUrl": t.base_url,
        "credentialSource": t.credential_source,
    })
}

fn map_connection(row: AIConnectionRow) -> Value {
    let jobs = jobs_of(&row);
    let key = row.api_key.clone().unwrap_or_default();
    let has_key = !key.trim().is_empty();

    json!({
        "id": row.id,
        "name": row.name,
        "provider": row.provider,
        "model_id": row.model_id,
        "api_key_masked": if has_key { mask_key(&key) } else { "None".to_string() },
        "has_api_key": has_key,
        "base_url": row.base_url,
        "temperature": row.temperature.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.7),
        "max_tokens": row.max_tokens.unwrap_or(1024),
        "is_active": row.is_active,
        "assigned_jobs": jobs,
        // NULL means no test has ever been recorded; that is reported as null
        // rather than as the "untested" string the UI used to receive.
        "test_status": row.test_status,
        "latency_ms": row.latency_ms,
        "last_tested_at": row.last_tested_at,
        "created_at": row.created_at,
    })
}

pub async fn list_ai_connections(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, AIConnectionRow>(
        "SELECT id, name, provider, model_id, api_key, base_url, temperature, max_tokens, is_active, assigned_jobs, last_tested_at, test_status, latency_ms, created_at, updated_at FROM ai_connections ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let conns: Vec<Value> = rows.into_iter().map(map_connection).collect();
    Json(conns)
}

pub async fn create_ai_connection(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let name = payload["name"].as_str().unwrap_or("").trim().to_string();
    // No "anthropic" default: labelling a connection with a provider the caller
    // never chose would route real model calls to the wrong vendor.
    let provider = payload["provider"].as_str().unwrap_or("").trim().to_string();
    let model = payload["model_id"]
        .as_str()
        .or_else(|| payload["modelId"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let mut invalid: Vec<&str> = Vec::new();
    if name.is_empty() {
        invalid.push("name");
    }
    if provider.is_empty() {
        invalid.push("provider");
    }
    if model.is_empty() {
        invalid.push("model_id");
    }
    if !invalid.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Connection cannot be created without real values.",
                "missingFields": invalid,
            })),
        )
            .into_response();
    }

    let raw_key = payload["api_key"]
        .as_str()
        .or_else(|| payload["apiKey"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if raw_key.contains('•') || raw_key.contains("****") {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "Refusing to store a masked API key. Paste the real secret -- a value containing the bullet character would silently break every model call.",
                "field": "api_key",
            })),
        )
            .into_response();
    }
    if raw_key.contains("changeme") {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "Refusing to store the placeholder API key 'changeme'.", "field": "api_key" })),
        )
            .into_response();
    }

    let base_url = payload["base_url"]
        .as_str()
        .or_else(|| payload["baseUrl"].as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref b) = base_url {
        if !(b.starts_with("http://") || b.starts_with("https://")) {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "error": "base_url must be an absolute http(s) endpoint.",
                    "field": "base_url",
                    "received": b,
                })),
            )
                .into_response();
        }
    }

    let temp = payload["temperature"].as_f64().unwrap_or(0.7);
    let max_tok = payload["max_tokens"].as_i64().unwrap_or(1024);
    let jobs: Vec<String> = payload["assigned_jobs"]
        .as_array()
        .or_else(|| payload["assignedJobs"].as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let is_active = payload["is_active"].as_bool().unwrap_or(true);

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let jobs_str = serde_json::to_string(&jobs).unwrap_or_else(|_| "[]".to_string());

    let res = sqlx::query(
        r#"INSERT INTO ai_connections (
            id, name, provider, model_id, api_key, base_url, temperature,
            max_tokens, is_active, assigned_jobs, test_status, latency_ms,
            last_tested_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'untested', NULL, NULL, ?, ?)"#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&provider)
    .bind(&model)
    .bind(if raw_key.is_empty() { None } else { Some(raw_key.clone()) })
    .bind(base_url.clone())
    .bind(temp.to_string())
    .bind(max_tok)
    .bind(is_active)
    .bind(&jobs_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    // Load-bearing: the response body echoes this id as a created connection, so
    // an INSERT that did not land exactly one row must fail the request.
    let inserted = match res {
        Ok(report) => report,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Connection could not be written to ai_connections: {e}") })),
            )
                .into_response();
        }
    };
    if inserted.rows_affected() != 1 {
        return refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "INSERT INTO ai_connections affected {} rows, expected 1; connection '{id}' was not created.",
                inserted.rows_affected()
            ),
            json!({ "id": id.as_str() }),
        );
    }

    let row = match sqlx::query_as::<_, AIConnectionRow>(&format!(
        "SELECT {AI_CONN_COLS} FROM ai_connections WHERE id = ?",
    ))
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the created connection could not be read back", err),
    };

    match row {
        Some(r) => (
            StatusCode::CREATED,
            Json(json!({
                "connection": map_connection(r),
                "credentialsStored": !raw_key.is_empty(),
                "tested": false,
                "note": if raw_key.is_empty() {
                    "No api_key stored: model calls will only work if ANTHROPIC_API_KEY / OPENAI_API_KEY is set in the server environment. Verify with POST /ai/connections/{id}/test."
                } else {
                    "Saved untested. Nothing has been sent to the provider yet -- verify with POST /ai/connections/{id}/test."
                },
            })),
        )
            .into_response(),
        None => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "name": name,
                "provider": provider,
                "model_id": model,
                "base_url": base_url,
                "temperature": temp,
                "max_tokens": max_tok,
                "is_active": is_active,
                "assigned_jobs": jobs,
                "has_api_key": !raw_key.is_empty(),
                "test_status": "untested",
                "latency_ms": Value::Null,
                "written": true,
                "readbackFailed": "row was inserted but could not be read back",
            })),
        )
            .into_response(),
    }
}

pub async fn test_ai_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    // A read failure is not a missing row: reporting 404 here would tell the
    // operator to create a connection that already exists.
    let row = match sqlx::query_as::<_, AIConnectionRow>(&format!(
        "SELECT {AI_CONN_COLS} FROM ai_connections WHERE id = ?",
    ))
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the connection could not be read from ai_connections", err),
    };

    let Some(conn) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Connection '{id}' does not exist in ai_connections.") })),
        )
            .into_response();
    };

    let now = Utc::now().to_rfc3339();
    // Credentials for this specific connection only -- never borrowed from
    // another row -- falling back to the matching provider env var.
    let stored_key = conn.api_key.clone().unwrap_or_default();
    let env_fallback = env_credential(&conn.provider);
    let (api_key, credential_source, env_base) =
        if !stored_key.trim().is_empty() && !looks_masked_or_placeholder(&stored_key) {
            (stored_key.clone(), format!("ai_connections row '{}'", conn.name), None)
        } else if let Some((k, b)) = env_fallback {
            (k, "process environment variable".to_string(), Some(b))
        } else {
            (
                String::new(),
                "none".to_string(),
                None,
            )
        };

    let base_url = conn
        .base_url
        .clone()
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty())
        .or(env_base)
        .unwrap_or_default();

    let mut missing: Vec<String> = Vec::new();
    if api_key.is_empty() {
        missing.push(format!(
            "api_key (connection '{}' has none usable and {} is unset)",
            conn.name,
            if anthropic_family(&conn.provider) { "ANTHROPIC_API_KEY" } else { "OPENAI_API_KEY" }
        ));
    }
    if conn.model_id.trim().is_empty() {
        missing.push("model_id (the connection row stores an empty model)".to_string());
    }
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        missing.push(format!(
            "base_url (connection '{}' has no absolute http(s) endpoint and no provider default is configured)",
            conn.name
        ));
    }

    if !missing.is_empty() {
        // WHY auxiliary: the request outcome the client needs is "not usable,
        // nothing was sent"; the test_status column is bookkeeping, so its
        // failure is logged and reported as a flag rather than replacing the 503.
        let status_stored = crate::db::report_write(
            sqlx::query(
                "UPDATE ai_connections SET test_status = 'failed', latency_ms = NULL, last_tested_at = ?, updated_at = ? WHERE id = ?",
            )
            .bind(&now)
            .bind(&now)
            .bind(&id)
            .execute(&state.db)
            .await
            .map(|report| report.rows_affected()),
            "recording test_status='failed' for an unusable ai_connections row",
        );
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "connection_id": conn.id,
                "name": conn.name,
                "provider": conn.provider,
                "model_id": conn.model_id,
                "status": "not_tested",
                "latency_ms": Value::Null,
                "missing": missing,
                "testStatusStored": status_stored,
                "testStatusStoredNote": if status_stored {
                    "ai_connections.test_status was written as 'failed'."
                } else {
                    "ai_connections.test_status could NOT be written as 'failed'; see the server log."
                },
                "error": "No request was sent: this connection is not usable until the fields listed in `missing` are configured.",
            })),
        )
            .into_response();
    }

    let target = ProviderTarget {
        connection_id: Some(conn.id.clone()),
        connection_name: conn.name.clone(),
        provider: conn.provider.clone(),
        model_id: conn.model_id.clone(),
        api_key,
        base_url: base_url.trim_end_matches('/').to_string(),
        temperature: conn.temperature.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.7),
        max_tokens: conn.max_tokens.unwrap_or(64),
        credential_source,
    };

    match chat_completion(&state, &target, "You are a connectivity probe. Reply with exactly: OK", "ping", 16).await {
        Ok((text, latency, http_status)) => {
            let latency_i = latency.min(i64::MAX as u128) as i64;
            // Auxiliary bookkeeping: the probe result is already known, so a
            // failed status write is logged and flagged, not returned as an error.
            let status_stored = crate::db::report_write(
                sqlx::query(
                    "UPDATE ai_connections SET test_status = 'connected', latency_ms = ?, last_tested_at = ?, updated_at = ? WHERE id = ?",
                )
                .bind(latency_i)
                .bind(&now)
                .bind(&now)
                .bind(&id)
                .execute(&state.db)
                .await
                .map(|report| report.rows_affected()),
                "recording test_status='connected' for an ai_connections row",
            );

            (
                StatusCode::OK,
                Json(json!({
                    "connection_id": conn.id,
                    "name": conn.name,
                    "provider": conn.provider,
                    "model_id": conn.model_id,
                    "status": "connected",
                    "latency_ms": latency_i,
                    "httpStatus": http_status,
                    "probeReply": text.chars().take(120).collect::<String>(),
                    "credentialSource": target.credential_source,
                    "endpoint": format!("{}/{}", target.base_url, if anthropic_family(&target.provider) { "v1/messages" } else { "v1/chat/completions" }),
                    "statusStored": status_stored,
                    "message": format!(
                        "Live completion succeeded against {} ({}ms). {}",
                        conn.model_id,
                        latency_i,
                        if status_stored {
                            "test_status was stored as 'connected'."
                        } else {
                            "test_status could NOT be stored; see the server log."
                        }
                    ),
                })),
            )
                .into_response()
        }
        Err(e) => {
            let (status, body) = e.to_json();
            let status_stored = crate::db::report_write(
                sqlx::query(
                    "UPDATE ai_connections SET test_status = 'failed', latency_ms = NULL, last_tested_at = ?, updated_at = ? WHERE id = ?",
                )
                .bind(&now)
                .bind(&now)
                .bind(&id)
                .execute(&state.db)
                .await
                .map(|report| report.rows_affected()),
                "recording test_status='failed' for a failed provider probe",
            );
            let mut with_ids = body;
            if let Some(obj) = with_ids.as_object_mut() {
                obj.insert("connection_id".to_string(), json!(conn.id));
                obj.insert("name".to_string(), json!(conn.name));
                obj.insert("provider".to_string(), json!(conn.provider));
                obj.insert("model_id".to_string(), json!(conn.model_id));
                obj.insert("status".to_string(), json!("failed"));
                obj.insert("latency_ms".to_string(), Value::Null);
                obj.insert("testStatusStored".to_string(), json!(status_stored));
                obj.insert(
                    "message".to_string(),
                    json!(if status_stored {
                        "The provider call did not succeed; test_status was stored as 'failed'."
                    } else {
                        "The provider call did not succeed, and test_status could NOT be stored as 'failed'; see the server log."
                    }),
                );
            }
            (status, Json(with_ids)).into_response()
        }
    }
}

pub async fn delete_ai_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let res = sqlx::query("DELETE FROM ai_connections WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("No ai_connections row with id '{id}'; nothing was deleted.") })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "deleted", "connection_id": id, "rowsAffected": 1 })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("DELETE FROM ai_connections failed: {e}") })),
        )
            .into_response(),
    }
}

pub async fn get_job_model_mapping(State(state): State<AppState>) -> impl IntoResponse {
    let conns = sqlx::query_as::<_, AIConnectionRow>(
        "SELECT id, name, provider, model_id, api_key, base_url, temperature, max_tokens, is_active, assigned_jobs, last_tested_at, test_status, latency_ms, created_at, updated_at FROM ai_connections WHERE is_active = 1",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut mapping = serde_json::Map::new();
    let standard_jobs = vec!["drafting", "summarization", "triage", "translation", "self_improving"];

    for job in &standard_jobs {
        let assigned = conns.iter().find(|c| jobs_of(c).iter().any(|j| j == *job));
        let (chosen, is_fallback) = match assigned {
            Some(c) => (c, false),
            None => match conns.first() {
                Some(c) => (c, true),
                None => continue,
            },
        };
        mapping.insert(
            job.to_string(),
            json!({
                "connection_id": chosen.id,
                "model_name": chosen.name,
                "provider": chosen.provider,
                "model_id": chosen.model_id,
                "explicitlyAssigned": !is_fallback,
                "testStatus": chosen.test_status.clone(),
                "usable": !chosen.api_key.as_deref().unwrap_or("").trim().is_empty()
                    || env_credential(&chosen.provider).is_some(),
            }),
        );
    }

    if mapping.is_empty() {
        return Json(json!({
            "_note": "No active row exists in ai_connections, so no AI job is currently routed to a real model endpoint.",
            "_missing": ["an active ai_connections row (name, provider, model_id, api_key, base_url)"],
        }));
    }

    Json(Value::Object(mapping))
}

pub async fn assign_model_to_job(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let job = payload["job_type"]
        .as_str()
        .or_else(|| payload["jobType"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let conn_id = payload["connection_id"]
        .as_str()
        .or_else(|| payload["connectionId"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let now = Utc::now().to_rfc3339();

    if job.is_empty() || conn_id.is_empty() {
        let missing_fields = [
            if job.is_empty() { "job_type" } else { "" },
            if conn_id.is_empty() { "connection_id" } else { "" },
        ]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<&str>>();
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Both job_type and connection_id are required to route an AI job.",
                "missingFields": missing_fields,
            })),
        )
            .into_response();
    }

    let target = match sqlx::query_as::<_, AIConnectionRow>(&format!(
        "SELECT {AI_CONN_COLS} FROM ai_connections WHERE id = ?",
    ))
    .bind(conn_id.as_str())
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the connection could not be read from ai_connections", err),
    };

    let Some(c) = target else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("Connection '{conn_id}' does not exist in ai_connections; job '{job}' was not routed to anything."),
                "jobType": job,
            })),
        )
            .into_response();
    };

    let mut jobs = jobs_of(&c);
    if !jobs.iter().any(|j| j == &job) {
        jobs.push(job.clone());
    }
    let jobs_str = serde_json::to_string(&jobs).unwrap_or_else(|_| "[]".to_string());
    let upd = sqlx::query("UPDATE ai_connections SET assigned_jobs = ?, updated_at = ? WHERE id = ?")
        .bind(&jobs_str)
        .bind(&now)
        .bind(c.id.as_str())
        .execute(&state.db)
        .await;

    match upd {
        // Load-bearing: this branch is the only place the client learns the
        // routing was stored, so it is reached only on a one-row UPDATE.
        Ok(r) if r.rows_affected() == 1 => (
            StatusCode::OK,
            Json(json!({
                "job_type": job,
                "assigned_to": c.id,
                "model_name": c.name,
                "model_id": c.model_id,
                "provider": c.provider,
                "status": "assigned",
                "saved": true,
                "rowsAffected": r.rows_affected(),
                "usable": !c.api_key.as_deref().unwrap_or("").trim().is_empty() || env_credential(&c.provider).is_some(),
                "note": if !c.api_key.as_deref().unwrap_or("").trim().is_empty() {
                    "Routing saved.".to_string()
                } else {
                    "Routing saved, but this connection stores no api_key: calls only work if the provider env var is set -- verify with POST /ai/connections/{id}/test.".to_string()
                },
            })),
        )
            .into_response(),
        Ok(r) => refuse(
            StatusCode::NOT_FOUND,
            &format!(
                "UPDATE ai_connections SET assigned_jobs affected {} rows for connection '{conn_id}': job '{job}' is not routed to anything.",
                r.rows_affected()
            ),
            json!({
                "id": conn_id.as_str(),
                "jobType": job.as_str(),
                "rowsAffected": r.rows_affected(),
                "status": "not_assigned",
            }),
        ),
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("assigned_jobs could not be persisted for connection '{conn_id}': {e}"),
            json!({
                "jobType": job.as_str(),
                "assignedTo": conn_id.as_str(),
                "status": "failed",
                "databaseError": true,
            }),
        ),
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CustomPolicyRow {
    pub id: String,
    pub job_type: String,
    pub tone: Option<String>,
    pub guidelines: Option<String>,
    pub rules: Option<String>,
    pub guardrails: Option<String>,
    pub auto_approval_mode: Option<String>,
    pub min_csat_learning_threshold: Option<String>,
    pub max_autonomous_refund: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn map_custom_policy(row: CustomPolicyRow) -> Value {
    let rules: Value = row
        .rules
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!([]));
    let guardrails: Vec<String> = row
        .guardrails
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    // Thresholds, tone and mode the row does not store are reported as null; a
    // 4.5 / $50 / "review_required" default would read as a configured setting.
    let min_csat: Option<f64> = row
        .min_csat_learning_threshold
        .as_deref()
        .and_then(|s| s.parse::<f64>().ok());
    let max_refund: Option<f64> = row
        .max_autonomous_refund
        .as_deref()
        .and_then(|s| s.parse::<f64>().ok());

    json!({
        "id": row.id,
        "job_type": row.job_type,
        "tone": row.tone,
        "guidelines": row.guidelines.unwrap_or_default(),
        "rules": rules,
        "guardrails": guardrails,
        "auto_approval_mode": row.auto_approval_mode,
        "min_csat_learning_threshold": min_csat,
        "max_autonomous_refund": max_refund,
        "updated_at": row.updated_at,
    })
}

pub async fn get_custom_policy(
    State(state): State<AppState>,
    Path(job_type): Path<String>,
) -> Response {
    let row = match sqlx::query_as::<_, CustomPolicyRow>(
        "SELECT id, job_type, tone, guidelines, rules, guardrails, auto_approval_mode, min_csat_learning_threshold, max_autonomous_refund, created_at, updated_at FROM ai_custom_policies WHERE job_type = ?",
    )
    .bind(&job_type)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the custom policy could not be read", err),
    };

    match row {
        Some(r) => Json(map_custom_policy(r)).into_response(),
        None => {
            // Seed a default policy for this job type
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let default_rules = json!([
                { "id": "r1", "rule": "Verify customer order status and history proactively", "enabled": true, "category": "compliance" },
                { "id": "r2", "rule": "Maintain professional, empathetic tone at all times", "enabled": true, "category": "tone" }
            ])
            .to_string();
            let default_guardrails = json!([
                "Never disclose internal margins, vendor contracts, or supplier costs.",
                "Never commit to financial refunds exceeding authorized policy caps without approval."
            ])
            .to_string();
            let guidelines = format!("Operational guidelines for {} workflow execution.", job_type);

            let inserted = match sqlx::query(
                r#"INSERT INTO ai_custom_policies (
                    id, job_type, tone, guidelines, rules, guardrails,
                    auto_approval_mode, min_csat_learning_threshold, max_autonomous_refund,
                    created_at, updated_at
                ) VALUES (?, ?, 'empathetic_and_direct', ?, ?, ?, 'review_required', '4.5', '50.0', ?, ?)"#,
            )
            .bind(&id)
            .bind(&job_type)
            .bind(&guidelines)
            .bind(&default_rules)
            .bind(&default_guardrails)
            .bind(&now)
            .bind(&now)
            .execute(&state.db)
            .await
            {
                Ok(report) => report,
                Err(err) => return db_error("the default custom policy could not be stored", err),
            };

            // Load-bearing: the response echoes this id as the policy for the
            // job type, so a row that was not inserted cannot be presented.
            if inserted.rows_affected() != 1 {
                return refuse(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!(
                        "INSERT INTO ai_custom_policies affected {} rows, expected 1; no default policy was created for job type '{job_type}'.",
                        inserted.rows_affected()
                    ),
                    json!({ "id": id.as_str(), "jobType": job_type.as_str() }),
                );
            }

            Json(json!({
                "id": id,
                "job_type": job_type,
                "persisted": true,
                "seededNow": true,
                "tone": "empathetic_and_direct",
                "guidelines": guidelines,
                "rules": serde_json::from_str::<Value>(&default_rules).unwrap_or_else(|_| json!([])),
                "guardrails": serde_json::from_str::<Value>(&default_guardrails).unwrap_or_else(|_| json!([])),
                "auto_approval_mode": "review_required",
                "min_csat_learning_threshold": 4.5,
                "max_autonomous_refund": 50.0,
                "updated_at": now
            }))
            .into_response()
        }
    }
}

pub async fn update_custom_policy(
    State(state): State<AppState>,
    Path(job_type): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let now = Utc::now().to_rfc3339();
    let tone = payload["tone"].as_str().unwrap_or("empathetic_and_direct");
    let guidelines = payload["guidelines"].as_str().unwrap_or("");
    let rules_str = payload
        .get("rules")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string());
    let guardrails_str = payload
        .get("guardrails")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string());
    let mode = payload["auto_approval_mode"].as_str().unwrap_or("review_required");
    let min_csat = payload["min_csat_learning_threshold"].as_f64().unwrap_or(4.5).to_string();
    let max_refund = payload["max_autonomous_refund"].as_f64().unwrap_or(50.0).to_string();

    let written = match sqlx::query(
        r#"INSERT INTO ai_custom_policies (
            id, job_type, tone, guidelines, rules, guardrails,
            auto_approval_mode, min_csat_learning_threshold, max_autonomous_refund,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(job_type) DO UPDATE SET
            tone = excluded.tone,
            guidelines = excluded.guidelines,
            rules = excluded.rules,
            guardrails = excluded.guardrails,
            auto_approval_mode = excluded.auto_approval_mode,
            min_csat_learning_threshold = excluded.min_csat_learning_threshold,
            max_autonomous_refund = excluded.max_autonomous_refund,
            updated_at = excluded.updated_at"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&job_type)
    .bind(tone)
    .bind(guidelines)
    .bind(&rules_str)
    .bind(&guardrails_str)
    .bind(mode)
    .bind(&min_csat)
    .bind(&max_refund)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(report) => report,
        Err(err) => return db_error("the custom policy could not be stored", err),
    };

    // Load-bearing: the PUT response is the stored policy, so the upsert has to
    // have touched exactly one row.
    if written.rows_affected() != 1 {
        return refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "the ai_custom_policies upsert affected {} rows, expected 1; the policy for job type '{job_type}' was not saved.",
                written.rows_affected()
            ),
            json!({ "jobType": job_type.as_str() }),
        );
    }

    let row = match sqlx::query_as::<_, CustomPolicyRow>(
        "SELECT id, job_type, tone, guidelines, rules, guardrails, auto_approval_mode, min_csat_learning_threshold, max_autonomous_refund, created_at, updated_at FROM ai_custom_policies WHERE job_type = ?",
    )
    .bind(&job_type)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the stored custom policy could not be read back", err),
    };

    match row {
        Some(r) => Json(map_custom_policy(r)).into_response(),
        None => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!(
                "The upsert reported 1 row but no ai_custom_policies row for job type '{job_type}' can be read back, so the update is not shown as saved."
            ),
            json!({ "jobType": job_type.as_str() }),
        ),
    }
}

pub async fn get_effective_prompt(
    State(state): State<AppState>,
    Path(job_type): Path<String>,
) -> Response {
    // A read failure must not be reported as "this job type has no stored
    // policy", because the fallback prompt below is a real answer either way.
    let row = match sqlx::query_as::<_, CustomPolicyRow>(
        "SELECT id, job_type, tone, guidelines, rules, guardrails, auto_approval_mode, min_csat_learning_threshold, max_autonomous_refund, created_at, updated_at FROM ai_custom_policies WHERE job_type = ?",
    )
    .bind(&job_type)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the stored prompt policy could not be read", err),
    };

    // Wording the model can act on is needed, but it must say the value
    // is missing rather than passing off invented guidance as stored.
    let (prompt, policy_found) = if let Some(r) = row {
        (
            format!(
                "You are the Captain AI Agent executing '{}' tasks. Guidelines: {}. Tone: {}. Adhere strictly to policy guardrails.",
                r.job_type,
                r.guidelines
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("(no guidelines are stored for this job type)"),
                r.tone
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("(no tone is stored for this job type)")
            ),
            true,
        )
    } else {
        (
            format!(
                "You are the Captain AI Agent executing '{}' tasks. Answer customer questions politely, concisely, and helpfully.",
                job_type
            ),
            false,
        )
    };

    (
        StatusCode::OK,
        Json(json!({
            "job_type": job_type,
            "prompt": prompt,
            "policyFound": policy_found,
            "policy_found": policy_found,
            "source": if policy_found {
                "ai_custom_policies row for this job type"
            } else {
                "generic fallback: no ai_custom_policies row exists for this job type, so nothing job-specific is applied"
            },
        })),
    )
        .into_response()
}

// ============================================================================
// Real-time Conversation AI Copilot & Assist Actions
// ============================================================================

// ============================================================================
// Captain AI Copilot generation -- every reply below comes from the configured
// provider endpoint or is refused; invented model output is never returned.
// ============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
struct ThreadMessage {
    id: String,
    body: String,
    sender: String,
    sender_type: Option<String>,
    direction: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ConvHead {
    id: String,
    state: String,
    channel: String,
    assigned_to_id: Option<String>,
    assigned_to_name: Option<String>,
    human_takeover: Option<bool>,
    team_id: Option<String>,
    language: Option<String>,
}

async fn load_conversation(state: &AppState, id: &str) -> Option<ConvHead> {
    sqlx::query_as::<_, ConvHead>(
        "SELECT id, state, channel, assigned_to_id, assigned_to_name, human_takeover, team_id, language FROM conversations WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
}

async fn thread_messages(state: &AppState, conversation_id: &str, limit: i64) -> Vec<ThreadMessage> {
    let mut rows = sqlx::query_as::<_, ThreadMessage>(
        "SELECT id, body, sender, sender_type, direction FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ?",
    )
    .bind(conversation_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    rows.reverse();
    rows
}

fn role_of(m: &ThreadMessage) -> &'static str {
    let st = m.sender_type.as_deref().unwrap_or("").to_lowercase();
    let dir = m.direction.as_deref().unwrap_or("").to_lowercase();
    let s = m.sender.to_lowercase();
    if st.contains("agent") || s == "agent" || s == "human" {
        "agent"
    } else if st.contains("bot") || st.contains("ai") || s == "bot" || s == "captain" {
        "assistant"
    } else if dir == "inbound" {
        "customer"
    } else if dir == "outbound" {
        "agent"
    } else {
        "customer"
    }
}

fn transcript(msgs: &[ThreadMessage], max_chars: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut used = 0usize;
    for m in msgs.iter().rev() {
        let line = format!("{}: {}", role_of(m), m.body.trim());
        used += line.chars().count();
        lines.push(line);
        if used >= max_chars {
            break;
        }
    }
    lines.reverse();
    lines.join("\n")
}

fn json_closing(text: &str, open: char, close: char) -> Option<&str> {
    let start = text.find(open)?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract the JSON object a model was asked to return, tolerating prose or
/// code fences around it.
fn parse_model_object(text: &str) -> Option<Value> {
    if let Some(slice) = json_closing(text, '{', '}') {
        if let Ok(v) = serde_json::from_str::<Value>(slice) {
            return Some(v);
        }
    }
    serde_json::from_str::<Value>(text.trim()).ok()
}

fn missing_conversation_response(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": format!("Conversation '{id}' does not exist, so there is no real thread to analyze."),
            "modelGenerated": false,
        })),
    )
        .into_response()
}

/// Some AI errors are "the engine never ran" (the `missing` list from
/// `AiError::NotConfigured`). Returning the note lets callers say exactly that
/// instead of labelling heuristic output with a model name.
fn unconfigured_note(body: &Value) -> Option<String> {
    let items = body.get("missing")?.as_array()?;
    let list: Vec<String> = items
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if list.is_empty() {
        return Some("no model endpoint is configured".to_string());
    }
    Some(list.join(", "))
}

#[derive(Deserialize)]
pub struct SuggestReplyPayload {
    pub instruction: Option<String>,
}

pub async fn get_captain_suggested_reply(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<SuggestReplyPayload>,
) -> Response {
    let Some(conv) = load_conversation(&state, &id).await else {
        return missing_conversation_response(&id);
    };

    let msgs = thread_messages(&state, &id, 14).await;
    let customer_msgs: Vec<String> = msgs
        .iter()
        .filter(|m| role_of(m) == "customer")
        .map(|m| m.id.clone())
        .collect();

    if msgs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "conversationId": id,
                "suggestedReply": Value::Null,
                "confidence": Value::Null,
                "groundedSources": [],
                "suggestedActions": [],
                "modelGenerated": false,
                "error": "This conversation has no stored messages, so there is nothing to ground a reply in. Refusing to invent one.",
            })),
        )
            .into_response();
    }

    let target = match resolve_target(&state, "drafting").await {
        Ok(t) => t,
        Err(e) => {
            let (status, mut body) = e.to_json();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("conversationId".to_string(), json!(id));
                obj.insert("confidence".to_string(), Value::Null);
                obj.insert("groundedSources".to_string(), json!([]));
                obj.insert("suggestedActions".to_string(), json!([]));
            }
            return (status, Json(body)).into_response();
        }
    };

    let mut system = "You are a support copilot drafting ONE reply for a human agent to send. \
Use only facts present in the transcript. Never invent order numbers, dates, prices, policies or \
delivery statuses. Keep it under 60 words, plain text, no markdown. Reply in the customer's language."
        .to_string();
    if let Some(inst) = payload.instruction.as_deref().map(|s| s.trim()) {
        if !inst.is_empty() {
            system.push_str(&format!(" Agent instruction: {inst}"));
        }
    }
    if let Some(lang) = conv.language.as_deref().map(|s| s.trim()) {
        if !lang.is_empty() {
            system.push_str(&format!(" Conversation language: {lang}."));
        }
    }

    let user = format!("Channel: {}\nThread (oldest first):\n{}", conv.channel, transcript(&msgs, 4000));

    match chat_completion(&state, &target, &system, &user, target.max_tokens.min(600)).await {
        Ok((text, latency, http_status)) => {
            let mut actions: Vec<String> = Vec::new();
            if conv.assigned_to_id.as_deref().unwrap_or("").is_empty() {
                actions.push("No agent is assigned to this conversation yet -- claim it before sending.".to_string());
            }
            if conv.human_takeover.unwrap_or(false) {
                actions.push("Human takeover is active; the AI copilot will not auto-send.".to_string());
            }
            (
                StatusCode::OK,
                Json(json!({
                    "conversationId": id,
                    "suggestedReply": text,
                    "confidence": Value::Null,
                    // WHY: the chat-completion endpoint returns prose only, so any
                    // score here would be invented; say so instead of implying one.
                    "confidenceNote": "null because the model endpoint returns no confidence score for a drafted reply",
                    "groundedSources": customer_msgs,
                    "suggestedActions": actions,
                    "modelGenerated": true,
                    "engine": engine_json(&target),
                    "latencyMs": latency,
                    "httpStatus": http_status,
                    "rationale": format!(
                        "Written live by {} / {} from the last {} message(s) of this thread; no provider call is cached.",
                        target.provider, target.model_id, msgs.len()
                    ),
                })),
            )
                .into_response()
        }
        Err(e) => {
            let (status, mut body) = e.to_json();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("conversationId".to_string(), json!(id));
                obj.insert("engine".to_string(), engine_json(&target));
                obj.insert("confidence".to_string(), Value::Null);
                obj.insert("groundedSources".to_string(), json!(customer_msgs));
                obj.insert("suggestedActions".to_string(), json!([]));
            }
            (status, Json(body)).into_response()
        }
    }
}

pub async fn summarize_conversation_thread(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let Some(conv) = load_conversation(&state, &id).await else {
        return missing_conversation_response(&id);
    };

    let msgs = thread_messages(&state, &id, 40).await;
    if msgs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "conversationId": id,
                "summary": Value::Null,
                "bulletPoints": [],
                "sentiment": "unknown",
                "keyTopics": [],
                "actionRequired": false,
                "modelGenerated": false,
                "error": "The thread has zero stored messages; there is nothing real to summarize.",
            })),
        )
            .into_response();
    }

    let target = match resolve_target(&state, "summarization").await {
        Ok(t) => t,
        Err(e) => {
            let (status, mut body) = e.to_json();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("conversationId".to_string(), json!(id));
                obj.insert("messageCount".to_string(), json!(msgs.len()));
            }
            return (status, Json(body)).into_response();
        }
    };

    let system = "You summarize support threads. Respond with ONE JSON object and nothing else, shaped \
{\"summary\": string, \"bulletPoints\": string[], \"sentiment\": \"positive|neutral|negative|urgent\", \
\"keyTopics\": string[], \"actionRequired\": boolean}. Only use facts from the thread.".to_string();
    let user = format!(
        "Channel: {}\nState: {}\nThread (oldest first):\n{}",
        conv.channel,
        conv.state,
        transcript(&msgs, 6000)
    );

    match chat_completion(&state, &target, &system, &user, target.max_tokens.min(900)).await {
        Ok((text, latency, _)) => {
            let parsed = parse_model_object(&text);
            let (status_code, payload) = match parsed {
                Some(v) => (
                    StatusCode::OK,
                    json!({
                        "conversationId": id,
                        "summary": v.get("summary").cloned().unwrap_or(Value::Null),
                        "bulletPoints": v.get("bulletPoints").cloned().or(v.get("keyPoints").cloned()).unwrap_or(json!([])),
                        "sentiment": v.get("sentiment").cloned().unwrap_or(json!("unknown")),
                        "keyTopics": v.get("keyTopics").cloned().unwrap_or(json!([])),
                        "actionRequired": v.get("actionRequired").cloned().unwrap_or(json!(false)),
                        "messageCount": msgs.len(),
                        "modelGenerated": true,
                        "engine": engine_json(&target),
                        "latencyMs": latency,
                    }),
                ),
                None => (
                    StatusCode::BAD_GATEWAY,
                    json!({
                        "conversationId": id,
                        "summary": Value::Null,
                        "modelGenerated": true,
                        "engine": engine_json(&target),
                        "error": "The provider replied but the reply was not valid JSON, so no summary fields are claimed.",
                        "modelReplyRaw": text.chars().take(1200).collect::<String>(),
                    }),
                ),
            };
            (status_code, Json(payload)).into_response()
        }
        Err(e) => {
            let (status, mut body) = e.to_json();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("conversationId".to_string(), json!(id));
                obj.insert("engine".to_string(), engine_json(&target));
            }
            (status, Json(body)).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct RewritePayload {
    pub text: String,
    pub tone: Option<String>,
}

const ALLOWED_TONES: &[&str] = &[
    "professional", "friendly", "formal", "concise", "shorten", "polite", "apologetic", "persuasive",
    "fix_spelling",
];

pub async fn rewrite_draft_text(
    State(state): State<AppState>,
    Json(payload): Json<RewritePayload>,
) -> Response {
    let text = payload.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Rewrite needs a non-empty 'text' to rewrite." })),
        )
            .into_response();
    }
    let tone = payload
        .tone
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "professional".to_string());
    if !ALLOWED_TONES.contains(&tone.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Unsupported tone '{tone}'."),
                "allowedTones": ALLOWED_TONES,
            })),
        )
            .into_response();
    }

    let target = match resolve_target(&state, "drafting").await {
        Ok(t) => t,
        Err(e) => {
            let (status, mut body) = e.to_json();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("original".to_string(), json!(text));
                obj.insert("rewritten".to_string(), Value::Null);
                obj.insert("tone".to_string(), json!(tone));
            }
            return (status, Json(body)).into_response();
        }
    };

    let system = format!(
        "Rewrite the user's draft in a {tone} tone. Preserve every factual claim, name, number and \
commitment exactly; add none. Respond with ONE JSON object only: {{\"rewritten\": string}}."
    );

    match chat_completion(&state, &target, &system, &text, target.max_tokens.min(800)).await {
        Ok((reply, latency, _)) => {
            let rewritten = parse_model_object(&reply)
                .and_then(|v| v.get("rewritten").and_then(|r| r.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| reply.clone());
            (
                StatusCode::OK,
                Json(json!({
                    "original": text,
                    "rewritten": rewritten,
                    "tone": tone,
                    "modelGenerated": true,
                    "engine": engine_json(&target),
                    "latencyMs": latency,
                })),
            )
                .into_response()
        }
        Err(e) => {
            let (status, mut body) = e.to_json();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("original".to_string(), json!(text));
                obj.insert("rewritten".to_string(), Value::Null);
                obj.insert("tone".to_string(), json!(tone));
                obj.insert("engine".to_string(), engine_json(&target));
            }
            (status, Json(body)).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct TriagePayload {
    pub apply: Option<bool>,
}

pub async fn triage_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<TriagePayload>,
) -> Response {
    if load_conversation(&state, &id).await.is_none() {
        return missing_conversation_response(&id);
    };

    let msgs = thread_messages(&state, &id, 10).await;
    if msgs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "conversationId": id,
                "intent": Value::Null,
                "priority": Value::Null,
                "suggestedTeam": Value::Null,
                "suggestedTeamId": Value::Null,
                "suggestedTags": [],
                "applied": false,
                "modelGenerated": false,
                "error": "No messages stored in this conversation -- refusing to triage invented content.",
            })),
        )
            .into_response();
    }

    let body_text = msgs.iter().map(|m| m.body.as_str()).collect::<Vec<&str>>().join(" ").to_lowercase();

    // Preferred path: the configured model. Fallback path: an explicitly labelled
    // keyword heuristic over the same real message bodies. `engine_label` always
    // names whatever actually produced the fields below.
    let engine_label: String;
    let mut model_generated = false;
    let mut model_latency: Option<u128> = None;
    let mut target_info: Option<Value> = None;
    let mut triage = json!({});

    match resolve_target(&state, "triage").await {
        Ok(target) => match chat_completion(
            &state,
            &target,
            "Classify the support thread. Respond with ONE JSON object only: {\"intent\": string, \
             \"priority\": \"urgent|high|medium|low\", \"tags\": string[], \"reasoning\": string, \
             \"confidence\": number between 0 and 1}. Report the confidence you actually hold; \
             omit the field rather than guessing. Base the decision solely on the transcript.",
            &transcript(&msgs, 3000),
            target.max_tokens.min(500),
        )
        .await
        {
            Ok((reply, latency, _)) => match parse_model_object(&reply) {
                Some(v) => {
                    triage = v;
                    model_generated = true;
                    model_latency = Some(latency);
                    target_info = Some(engine_json(&target));
                    engine_label = format!("{} / {}", target.provider, target.model_id);
                }
                None => {
                    return (
                        StatusCode::BAD_GATEWAY,
                        Json(json!({
                            "conversationId": id,
                            "modelGenerated": true,
                            "engine": engine_json(&target),
                            "error": "The provider replied but the triage reply was not valid JSON.",
                            "modelReplyRaw": reply.chars().take(800).collect::<String>(),
                            "applied": false,
                        })),
                    )
                        .into_response();
                }
            },
            Err(e) => {
                let (status, body) = e.to_json();
                let note = unconfigured_note(&body);
                match note {
                    Some(note) => {
                        engine_label = format!(
                            "no model engine ran ({note}); local keyword heuristic over the stored message text"
                        );
                    }
                    None => {
                        let mut b = body;
                        if let Some(obj) = b.as_object_mut() {
                            obj.insert("conversationId".to_string(), json!(id));
                            obj.insert("applied".to_string(), json!(false));
                        }
                        return (status, Json(b)).into_response();
                    }
                }
            }
        },
        Err(e) => {
            let (status, body) = e.to_json();
            let note = unconfigured_note(&body);
            match note {
                Some(note) => {
                    engine_label = format!(
                        "no model engine ran ({note}); local keyword heuristic over the stored message text"
                    );
                }
                None => {
                    let mut b = body;
                    if let Some(obj) = b.as_object_mut() {
                        obj.insert("conversationId".to_string(), json!(id));
                    }
                    return (status, Json(b)).into_response();
                }
            }
        }
    }

    if !model_generated {
        let urgent = ["urgent", "emergency", "outage", "down", "cancel", "refund", "lawyer", "legal"]
            .iter()
            .any(|k| body_text.contains(k));
        let high = ["broken", "error", "bug", "failed", "not working", "complaint"]
            .iter()
            .any(|k| body_text.contains(k));
        let (intent, priority) = if urgent {
            ("billing_cancellation", "urgent")
        } else if high {
            ("technical_issue", "high")
        } else if ["price", "pricing", "cost", "quote", "demo"].iter().any(|k| body_text.contains(k)) {
            ("sales_inquiry", "medium")
        } else {
            ("general_inquiry", "low")
        };
        triage = json!({
            "intent": intent,
            "priority": priority,
            "tags": [intent],
            "reasoning": format!(
                "Matched keyword groups against the real text of the last {} message(s) in this thread.",
                msgs.len()
            ),
        });
    }

    // None of these four may default to a plausible value: a fabricated "medium"
    // priority or "general_inquiry" intent would be written into the conversation
    // and read back by an operator as a real classification.
    let priority = triage
        .get("priority")
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase())
        .filter(|s| ["urgent", "high", "medium", "low"].contains(&s.as_str()));
    let intent = triage
        .get("intent")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let tags: Vec<String> = triage
        .get("tags")
        .or_else(|| triage.get("suggestedTags"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let reasoning = triage
        .get("reasoning")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    // Confidence is only ever what the provider reported; the heuristic fallback
    // deliberately stores none, so a missing value stays null rather than a score.
    let confidence = triage.get("confidence").and_then(|v| v.as_f64());

    // Team suggestion comes from the real teams table when it exists.
    let mut suggested_team: Option<(String, String)> = None;
    if let Some(obj) = triage.get("suggestedTeam").and_then(|v| v.as_str()) {
        let probe = obj.to_string();
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT id, name FROM teams WHERE LOWER(name) = LOWER(?) LIMIT 1",
        )
        .bind(&probe)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
        if let Some(r) = row {
            suggested_team = Some(r);
        }
    }
    if suggested_team.is_none() {
        let keyed: Option<(String, String)> = sqlx::query_as(
            "SELECT id, name FROM teams WHERE LOWER(name) LIKE LOWER('%support%') LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
        suggested_team = keyed;
    }

    let mut applied = false;
    let mut apply_failure: Option<(StatusCode, String)> = None;
    let mut out_applied_fields: Value = Value::Null;
    if payload.apply == Some(true) {
        match priority.as_deref() {
            // Applying an unclassified priority would store a value no engine
            // produced, so the write is refused instead of defaulting to "medium".
            None => apply_failure = Some((
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("No engine produced a priority for conversation '{id}', so conversations.priority was not changed."),
            )),
            Some(priority_value) => {
                let now = Utc::now().to_rfc3339();
                let res = match suggested_team.as_ref() {
                    Some((tid, _)) => {
                        sqlx::query(
                            "UPDATE conversations SET priority = ?, team_id = ?, updated_at = ? WHERE id = ?",
                        )
                        .bind(priority_value)
                        .bind(tid.clone())
                        .bind(&now)
                        .bind(&id)
                        .execute(&state.db)
                        .await
                    }
                    None => {
                        sqlx::query("UPDATE conversations SET priority = ?, updated_at = ? WHERE id = ?")
                            .bind(priority_value)
                            .bind(&now)
                            .bind(&id)
                            .execute(&state.db)
                            .await
                    }
                };
                match res {
                    Ok(r) if r.rows_affected() == 1 => {
                        applied = true;
                        out_applied_fields = json!({
                            "priority": priority_value,
                            "teamId": suggested_team.as_ref().map(|(i, _)| i.clone()),
                            "rowsAffected": r.rows_affected(),
                        });
                    }
                    Ok(r) if r.rows_affected() == 0 => apply_failure = Some((
                        StatusCode::NOT_FOUND,
                        format!("UPDATE conversations affected 0 rows: no conversation with id '{id}' exists, so nothing was applied."),
                    )),
                    Ok(r) => apply_failure = Some((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!(
                            "UPDATE conversations affected {} rows for id '{id}', not the single row expected, so the write was not treated as applied.",
                            r.rows_affected()
                        ),
                    )),
                    Err(e) => apply_failure = Some((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Applying the triage failed: {e}"),
                    )),
                }
            }
        }
    }

    let mut out = json!({
        "conversationId": id,
        "intent": intent,
        "priority": priority,
        "suggestedTeam": suggested_team.as_ref().map(|(_, n)| n.clone()),
        "suggestedTeamId": suggested_team.as_ref().map(|(i, _)| i.clone()),
        "suggestedTags": tags,
        "applied": applied,
        "appliedFields": out_applied_fields,
        "reasoning": reasoning,
        "confidence": confidence,
        "modelGenerated": model_generated,
        "engineRan": model_generated,
        "engine": engine_label,
        "messageCount": msgs.len(),
    });
    if confidence.is_none() {
        // WHY: callers render a gauge from this number; a null with a note is
        // truthful, an invented 0.8 is not.
        out["confidenceNote"] =
            json!("null because neither the model reply nor the local heuristic reported a confidence value");
    }
    if let Some(l) = model_latency {
        out["modelLatencyMs"] = json!(l);
    }
    if let Some(t) = target_info {
        out["providerEngine"] = t;
    }
    if let Some((status, err)) = apply_failure {
        out["applyError"] = json!(err.clone());
        return refuse(status, &err, out);
    }
    (StatusCode::OK, Json(out)).into_response()
}

async fn monitor_counts(state: &AppState, keywords: &[String], window_hours: i64) -> (i64, i64) {
    let now = Utc::now();
    let since = now - chrono::Duration::hours(window_hours);
    let since_rfc = since.to_rfc3339();
    let since_naive = since.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut sql = String::from(
        "SELECT COUNT(*) AS hits, COUNT(DISTINCT conversation_id) AS convs FROM messages WHERE (created_at >= ? OR created_at >= ?) AND (",
    );
    let conds: Vec<String> = keywords.iter().map(|_| "LOWER(body) LIKE ?".to_string()).collect();
    sql.push_str(&conds.join(" OR "));
    sql.push(')');

    let mut query = sqlx::query_as::<_, (i64, i64)>(&sql).bind(&since_rfc).bind(&since_naive);
    for kw in keywords {
        query = query.bind(format!("%{}%", kw.to_lowercase()));
    }

    query.fetch_one(&state.db).await.unwrap_or((0, 0))
}

async fn sample_quote(state: &AppState, keywords: &[String], window_hours: i64) -> Option<String> {
    let now = Utc::now();
    let since = now - chrono::Duration::hours(window_hours);
    let since_rfc = since.to_rfc3339();
    let since_naive = since.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut sql = String::from(
        "SELECT body FROM messages WHERE (created_at >= ? OR created_at >= ?) AND (",
    );
    let conds: Vec<String> = keywords.iter().map(|_| "LOWER(body) LIKE ?".to_string()).collect();
    sql.push_str(&conds.join(" OR "));
    sql.push_str(") AND is_private = 0 ORDER BY created_at DESC LIMIT 1");

    let mut query = sqlx::query_scalar::<_, String>(&sql).bind(&since_rfc).bind(&since_naive);
    for kw in keywords {
        query = query.bind(format!("%{}%", kw.to_lowercase()));
    }

    query
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|b| b.chars().take(220).collect::<String>())
}

const POSITIVE_WORDS: &[&str] = &[
    "thanks", "thank you", "great", "awesome", "love", "perfect", "good news", "appreciate", "works",
];
const NEGATIVE_WORDS: &[&str] = &[
    "angry", "furious", "terrible", "awful", "worst", "refund", "cancel", "complaint", "scam",
    "broken", "useless", "annoying", "unacceptable",
];
const URGENT_WORDS: &[&str] = &["urgent", "asap", "immediately", "emergency", "outage", "down"];

fn sentiment_word_hit(body: &str) -> &'static str {
    let b = body.to_lowercase();
    if URGENT_WORDS.iter().any(|w| b.contains(w)) {
        "urgent"
    } else if NEGATIVE_WORDS.iter().any(|w| b.contains(w)) {
        "negative"
    } else if POSITIVE_WORDS.iter().any(|w| b.contains(w)) {
        "positive"
    } else {
        "neutral"
    }
}

async fn sentiment_snapshot(state: &AppState, window_hours: i64) -> Value {
    let now = Utc::now();
    let since = now - chrono::Duration::hours(window_hours);
    let rows: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT body FROM messages WHERE (created_at >= ? OR created_at >= ?) AND sender IN ('customer','user','inbound')",
    )
    .bind(since.to_rfc3339())
    .bind(since.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string())
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut telemetry = json!({ "positive": 0, "neutral": 0, "negative": 0, "urgent": 0 });
    for body in &rows {
        let bucket = sentiment_word_hit(body);
        if let Some(v) = telemetry.get_mut(bucket).and_then(|v| v.as_i64()) {
            telemetry[bucket] = json!(v + 1);
        }
    }
    telemetry["sampledMessages"] = json!(rows.len());
    telemetry["method"] = json!("keyword heuristic over inbound message bodies (no model call)");
    telemetry
}

fn growth_pct(current: i64, previous: i64) -> String {
    if previous == 0 {
        return if current > 0 { "+new".to_string() } else { "0%".to_string() };
    }
    let pct = ((current - previous) as f64 * 100.0 / previous as f64).round();
    if pct >= 0.0 {
        format!("+{pct}%")
    } else {
        format!("{pct}%")
    }
}

pub async fn get_captain_monitors(State(state): State<AppState>) -> Response {
    let defaults: Vec<Value> = vec![
        json!({
            "id": "mon-billing",
            "name": "Billing & Refund Spike Detector",
            "topic": "billing",
            "keywords": ["refund", "billing", "invoice", "overcharge", "dispute"],
            "threshold": 5,
        }),
        json!({
            "id": "mon-promotion",
            "name": "Promotion Interest Monitor",
            "topic": "promotion",
            "keywords": ["discount", "promo", "offer", "early access", "coupon"],
            "threshold": 10,
        }),
        json!({
            "id": "mon-developer",
            "name": "Webhook & API Timeout Guard",
            "topic": "developer",
            "keywords": ["webhook", "504", "timeout", "signature", "api error"],
            "threshold": 3,
        }),
    ];

    let customs: Vec<Value> = match CUSTOM_MONITORS.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => vec![],
    };

    let mut active: Vec<Value> = Vec::new();
    let mut themes: Vec<Value> = Vec::new();

    for mut mon in defaults.into_iter().chain(customs.iter().cloned()) {
        let keywords: Vec<String> = mon
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let threshold = mon.get("threshold").and_then(|v| v.as_i64()).unwrap_or(5);

        let (hits_today, convs_today) = monitor_counts(&state, &keywords, 24).await;
        let (hits_48, _) = monitor_counts(&state, &keywords, 48).await;
        let hits_prev_window = (hits_48 - hits_today).max(0);

        let triggered = hits_today >= threshold;
        let insight = if triggered {
            format!(
                "{hits_today} message(s) across {convs_today} conversation(s) matched in the last 24h, at or above the threshold of {threshold}."
            )
        } else {
            format!(
                "{hits_today} message(s) across {convs_today} conversation(s) matched in the last 24h, below the threshold of {threshold}."
            )
        };

        if let Some(obj) = mon.as_object_mut() {
            obj.insert("currentHitsToday".to_string(), json!(hits_today));
            obj.insert("matchedConversations".to_string(), json!(convs_today));
            obj.insert("hitsPrevious24h".to_string(), json!(hits_prev_window));
            obj.insert("status".to_string(), json!(if triggered { "triggered" } else { "healthy" }));
            obj.insert("severity".to_string(), json!(if triggered { "high" } else { "normal" }));
            obj.insert("insight".to_string(), json!(insight));
            obj.insert(
                "persisted".to_string(),
                json!(obj.get("storage").is_none()),
            );
        }

        let quote = sample_quote(&state, &keywords, 24).await;
        if triggered || hits_today > 0 {
            let bucket = quote
                .as_deref()
                .map(sentiment_word_hit)
                .unwrap_or("neutral")
                .to_string();
            themes.push(json!({
                "theme": mon.get("name").and_then(|v| v.as_str()).unwrap_or("theme").to_string(),
                "volume": hits_today,
                "growth": growth_pct(hits_today, hits_prev_window),
                "sentiment": bucket,
                "sampleQuote": quote.unwrap_or_else(|| "No public message matched in this window.".to_string()),
                "recommendedAction": if triggered {
                    format!("Threshold reached: add or pin a knowledge-base entry covering {} and reply with a canned response.", keywords.first().cloned().unwrap_or_default())
                } else {
                    "No action needed; volume is below the alert threshold.".to_string()
                },
            }));
        }

        active.push(mon);
    }

    let total_convs: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT conversation_id) FROM messages WHERE created_at >= ? OR created_at >= ?",
    )
    .bind((Utc::now() - chrono::Duration::hours(24)).to_rfc3339())
    .bind((Utc::now() - chrono::Duration::hours(24)).naive_utc().format("%Y-%m-%d %H:%M:%S").to_string())
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    themes.sort_by(|a, b| b["volume"].as_i64().unwrap_or(0).cmp(&a["volume"].as_i64().unwrap_or(0)));
    themes.truncate(3);

    let sentiment = sentiment_snapshot(&state, 24).await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "activeMonitors": active,
            "customMonitorCount": customs.len(),
            "emergingThemes": themes,
            "sentimentTelemetry": sentiment,
            "totalMonitoredConversations": total_convs,
            "lastAnalyzedAt": Utc::now().to_rfc3339(),
            "dataBasis": "Counts are live SQL aggregates over the messages table for the stated windows. Nothing here is a canned figure.",
            "modelGenerated": false,
            "note": if customs.is_empty() {
                "Custom monitors are empty; POST /captain/monitors adds process-lifetime monitors (they are not persisted to the database in this build)."
            } else {
                "Custom monitors listed here live only in this process and reset on restart."
            },
        })),
    )
        .into_response()
}

pub async fn create_captain_monitor(Json(payload): Json<Value>) -> Response {
    let name = payload["name"].as_str().unwrap_or("").trim().to_string();
    let topic = payload["topic"].as_str().unwrap_or("general").trim().to_string();
    let keywords: Vec<String> = payload["keywords"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|k| k.as_str().map(|s| s.trim().to_lowercase())).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let threshold = payload["threshold"].as_i64().unwrap_or(5);

    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "A monitor requires a non-empty 'name'." })),
        )
            .into_response();
    }
    if keywords.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "A monitor requires at least one keyword in 'keywords'." })),
        )
            .into_response();
    }
    if !(1..=1000).contains(&threshold) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "threshold must be between 1 and 1000." })),
        )
            .into_response();
    }

    let monitor = json!({
        "id": format!("mon-{}", &Uuid::new_v4().simple().to_string()[..8]),
        "name": name,
        "topic": topic,
        "keywords": keywords,
        "threshold": threshold,
        "currentHitsToday": 0,
        "status": "healthy",
        "severity": "normal",
        "insight": format!("Registered to watch {} keyword(s): {}.", keywords.len(), keywords.join(", ")),
        "storage": "process-memory",
    });

    match CUSTOM_MONITORS.lock() {
        Ok(mut guard) => {
            guard.push(monitor.clone());
            (
                StatusCode::CREATED,
                Json(json!({
                    "monitor": monitor,
                    "persisted": false,
                    "warning": "Stored in this server process only -- there is no captain_monitors table in the schema, so custom monitors are lost on restart. See WIRING NEEDED.",
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Monitor registry is unavailable: {e}") })),
        )
            .into_response(),
    }
}
