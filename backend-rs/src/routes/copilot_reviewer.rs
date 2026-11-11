#![allow(dead_code)]

//! Copilot for Human Reviewers — API endpoints (Phase 10).
//!
//! Endpoints:
//! - GET  /copilot-reviewer/drafts/{draft_id}/context  — Get copilot context for a draft
//! - GET  /copilot-reviewer/pending-contexts           — Get context for all pending drafts
//! - POST /copilot-reviewer/drafts/{draft_id}/refresh  — Force-refresh copilot context

use crate::db::AppState;
use crate::services::copilot_reviewer;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

/// Get copilot context for a specific draft.
/// Returns conversation summary, customer profile, and suggested edits.
pub async fn get_draft_context(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match copilot_reviewer::get_copilot_context_for_draft(&state, &draft_id).await {
        Ok(ctx) => {
            let serialized = serde_json::to_value(&ctx).unwrap_or_else(|_| json!({}));
            (StatusCode::OK, Json(serialized))
        }
        Err(e) => {
            tracing::error!("Failed to get copilot context for draft {}: {}", draft_id, e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Draft not found: {}", e) })),
            )
        }
    }
}

/// Get copilot context for all pending review drafts.
/// Returns an array of context objects for each pending draft.
pub async fn get_pending_contexts(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    match copilot_reviewer::get_pending_drafts_context(&state).await {
        Ok(contexts) => {
            let serialized = serde_json::to_value(&contexts).unwrap_or_else(|_| json!([]));
            (StatusCode::OK, Json(serialized))
        }
        Err(e) => {
            tracing::error!("Failed to get pending draft contexts: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to fetch contexts: {}", e) })),
            )
        }
    }
}

/// Force-refresh copilot context for a draft by deleting cached data and regenerating.
pub async fn refresh_draft_context(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Delete cached context
    let result = sqlx::query("DELETE FROM copilot_context WHERE draft_id = ?")
        .bind(&draft_id)
        .execute(&state.db)
        .await;

    if let Err(e) = result {
        tracing::error!("Failed to delete cached context: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to clear cache: {}", e) })),
        );
    }

    // Regenerate context
    match copilot_reviewer::get_copilot_context_for_draft(&state, &draft_id).await {
        Ok(ctx) => {
            let serialized = serde_json::to_value(&ctx).unwrap_or_else(|_| json!({}));
            (StatusCode::OK, Json(serialized))
        }
        Err(e) => {
            tracing::error!("Failed to refresh copilot context for draft {}: {}", draft_id, e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Draft not found: {}", e) })),
            )
        }
    }
}
