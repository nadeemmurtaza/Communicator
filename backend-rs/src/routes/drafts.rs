//! AI reply drafts. The generation and the send are delegated to
//! `services::reply_engine` and `services::draft_actions`; this file only turns
//! those results into HTTP responses. Nothing here composes reply text, a
//! confidence number, or a delivery status.

use crate::db::AppState;
use crate::routes::auth::Tenant;
use crate::services::{draft_actions, reply_engine};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

const DRAFT_COLUMNS: &str =
    "id, conversation_id, body, confidence, citations, status, created_at";

#[derive(Deserialize)]
pub struct DraftQuery {
    pub status: Option<String>,
}

/// `citations` is a JSON array stored as text. A value that does not parse is
/// reported as unparseable instead of being returned as a string the UI cannot
/// iterate, and never replaced with an empty list pretending to be accurate.
fn citations_value(raw: Option<String>) -> (Value, Option<String>) {
    match raw {
        None => (
            Value::Null,
            Some("no citations column value is stored on this draft".to_string()),
        ),
        Some(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value @ Value::Array(_)) => (value, None),
            Ok(other) => (
                Value::Null,
                Some(format!("drafts.citations holds {other}, which is not an array")),
            ),
            Err(err) => (
                Value::Null,
                Some(format!("drafts.citations could not be parsed as JSON: {err}")),
            ),
        },
    }
}

fn read_text(
    row: &sqlx::postgres::PgRow,
    column: &'static str,
    unreadable: &mut Vec<&'static str>,
) -> Value {
    match row.try_get::<String, _>(column) {
        Ok(value) => json!(value),
        Err(_) => {
            unreadable.push(column);
            Value::Null
        }
    }
}

/// A column that cannot be read is emitted as `null` with its name listed in
/// `unreadableColumns`, not as an empty string or a zero that looks like data.
fn draft_json(row: &sqlx::postgres::PgRow) -> Value {
    let mut unreadable: Vec<&'static str> = Vec::new();
    let id = read_text(row, "id", &mut unreadable);
    let conversation_id = read_text(row, "conversation_id", &mut unreadable);
    let body = read_text(row, "body", &mut unreadable);
    let status = read_text(row, "status", &mut unreadable);
    let created_at = read_text(row, "created_at", &mut unreadable);
    let confidence = match row.try_get::<Option<f64>, _>("confidence") {
        Ok(value) => json!(value),
        Err(_) => {
            unreadable.push("confidence");
            Value::Null
        }
    };
    let citations_raw: Option<String> = row
        .try_get::<Option<String>, _>("citations")
        .ok()
        .flatten();
    let (citations, citation_warning) = citations_value(citations_raw);

    json!({
        "id": id,
        "draftId": id,
        "conversationId": conversation_id,
        "conversation_id": conversation_id,
        "body": body,
        "confidence": confidence,
        "citations": citations,
        "status": status,
        "draftStatus": status,
        "createdAt": created_at,
        "created_at": created_at,
        "citationWarning": citation_warning,
        "unreadableColumns": unreadable,
    })
}

pub async fn list_drafts(
    State(state): State<AppState>,
    Query(query): Query<DraftQuery>,
) -> Response {
    let status_filter = query.status.unwrap_or_else(|| "pending_review".to_string());

    let rows = match sqlx::query(&format!(
        "SELECT {DRAFT_COLUMNS} FROM drafts WHERE status = $1 ORDER BY created_at DESC"
    ))
    .bind(&status_filter)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "drafts": [],
                    "error": format!("the drafts list could not be read: {err}"),
                    "databaseError": true,
                })),
            )
                .into_response();
        }
    };

    let drafts: Vec<Value> = rows.iter().map(draft_json).collect();
    (
        StatusCode::OK,
        Json(json!({
            "drafts": drafts,
            "count": drafts.len(),
            "statusFilter": status_filter,
        })),
    )
        .into_response()
}

pub async fn approve_draft(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    match draft_actions::approve_and_send(&state, &id, "agent").await {
        Ok(report) => {
            let status = if report.sent {
                StatusCode::OK
            } else {
                StatusCode::BAD_GATEWAY
            };
            let mut body = report.to_json();
            body["tenantKey"] = json!(tenant_key);
            (status, Json(body)).into_response()
        }
        Err(err) => (err.status(), Json(err.json())).into_response(),
    }
}

#[derive(Deserialize)]
pub struct RejectDraftPayload {
    pub reason: Option<String>,
}

pub async fn reject_draft(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
    payload: Option<Json<RejectDraftPayload>>,
) -> Response {
    // Python also accepted `?feedback=`; the body reason wins when both exist.
    let reason = payload
        .and_then(|Json(body)| body.reason)
        .unwrap_or_default()
        .trim()
        .to_string();
    if reason.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "a rejection needs a reason: drafts.reviewer_feedback would otherwise \
                         record nothing about why the draft was refused",
            })),
        )
            .into_response();
    }

    match draft_actions::reject_draft(&state, &id, &reason, "agent").await {
        Ok(report) => {
            let mut body = report.to_json();
            body["reason"] = json!(reason);
            body["tenantKey"] = json!(tenant_key);
            body["success"] = json!(!report
                .warnings
                .iter()
                .any(|w| w.starts_with("the draft was NOT marked rejected")));
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(err) => (err.status(), Json(err.json())).into_response(),
    }
}

#[derive(Deserialize)]
pub struct EditDraftPayload {
    pub body: String,
}

pub async fn edit_draft(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<EditDraftPayload>,
) -> Response {
    if payload.body.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "the edited draft body is empty; nothing was written",
            })),
        )
            .into_response();
    }

    let current = match sqlx::query(&format!("SELECT {DRAFT_COLUMNS}, reviewer_feedback FROM drafts WHERE id = $1 LIMIT 1"))
        .bind(&id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Draft not found", "draftId": id })),
            )
                .into_response()
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("the draft could not be read: {err}"),
                    "databaseError": true,
                })),
            )
                .into_response()
        }
    };

    let status = match crate::db::required_text(&current, "status") {
        Ok(status) => status,
        Err(_) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "success": false,
                    "error": "the draft's current status could not be read, so it cannot be \
                             checked for whether editing is allowed",
                    "draftId": id,
                })),
            )
                .into_response();
        }
    };
    // A human rewrite demotes an auto-approved draft back to review: the
    // confidence that earned auto-send applied to the model's wording, not this.
    if !(status.eq_ignore_ascii_case("pending_review") || status.eq_ignore_ascii_case("auto_approved"))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("a draft in status '{status}' cannot be edited"),
                "draftStatus": status,
            })),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    let updated = sqlx::query(
        "UPDATE drafts SET body = $1, status = 'pending_review', updated_at = $2 WHERE id = $3",
    )
    .bind(&payload.body)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    let (written, detail) = match updated {
        Ok(report) if report.rows_affected() == 1 => (true, None),
        Ok(report) => (
            false,
            Some(format!(
                "the UPDATE affected {} rows, so the edited body was not stored",
                report.rows_affected()
            )),
        ),
        Err(err) => (false, Some(format!("the database refused the UPDATE: {err}"))),
    };

    let mut body = json!({
        "id": id,
        "draftId": id,
        "conversationId": current
            .try_get::<Option<String>, _>("conversation_id")
            .ok()
            .flatten(),
        "body": payload.body,
        "confidence": current
            .try_get::<Option<f64>, _>("confidence")
            .ok()
            .flatten(),
        "citations": citations_value(
            current
                .try_get::<Option<String>, _>("citations")
                .ok()
                .flatten(),
        )
        .0,
        "status": "pending_review",
        "previousStatus": status,
        "saved": written,
        "updatedAt": now,
    });
    if let Some(reason) = detail {
        body["error"] = json!(reason);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response();
    }
    (StatusCode::OK, Json(body)).into_response()
}

pub async fn generate_draft(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(conversation_id): Path<String>,
) -> Response {
    match reply_engine::generate_reply(&state, &conversation_id, Some(&tenant_key)).await {
        Ok(draft) => {
            let status = if draft.draft_written {
                StatusCode::CREATED
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            let mut body = draft.to_json();
            body["tenantKey"] = json!(tenant_key);
            (status, Json(body)).into_response()
        }
        Err(err) => (err.status(), Json(err.json())).into_response(),
    }
}

/// Used by the scheduler: send everything the reply engine left auto-approved.
pub async fn dispatch_auto_approved_drafts(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
) -> Response {
    let reports = match draft_actions::dispatch_auto_approved(&state, &tenant_key).await {
        Ok(reports) => reports,
        Err(err) => return (err.status(), Json(err.json())).into_response(),
    };

    let results: Vec<Value> = reports
        .into_iter()
        .map(|report| match report {
            Ok(ok) => ok.to_json(),
            Err(err) => err.json(),
        })
        .collect();
    let sent = results
        .iter()
        .filter(|item| item.get("sent").and_then(Value::as_bool) == Some(true))
        .count();

    (
        StatusCode::OK,
        Json(json!({
            "attempted": results.len(),
            "sent": sent,
            "results": results,
        })),
    )
        .into_response()
}
