#![allow(dead_code)]

//! CX QA Scoring API routes — Phase 16.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::db::AppState;
use crate::services::cx_qa_scoring;

/// GET /cx-qa/summary — Get CX score summary with trend analysis.
pub async fn get_cx_summary(
    State(state): State<AppState>,
) -> Result<Json<cx_qa_scoring::CXSummary>, (StatusCode, String)> {
    match cx_qa_scoring::get_cx_summary(&state).await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /cx-qa/{conversation_id} — Get QA score for a conversation.
pub async fn get_qa_score(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<cx_qa_scoring::QAScore>, (StatusCode, String)> {
    match cx_qa_scoring::score_conversation(&state, &conversation_id).await {
        Ok(score) => Ok(Json(score)),
        Err(e) => {
            if e.contains("No messages") {
                Err((StatusCode::NOT_FOUND, e))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e))
            }
        }
    }
}

/// POST /cx-qa/batch-score — Score all unresolved conversations.
pub async fn batch_score(
    State(state): State<AppState>,
) -> Result<Json<cx_qa_scoring::BatchScoringResult>, (StatusCode, String)> {
    match cx_qa_scoring::batch_score_conversations(&state).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /cx-qa/rules — List all QA scoring rules.
pub async fn list_qa_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<cx_qa_scoring::QAScoringRule>>, (StatusCode, String)> {
    match cx_qa_scoring::list_qa_rules(&state).await {
        Ok(rules) => Ok(Json(rules)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// PUT /cx-qa/rules/{id} — Update a QA scoring rule.
pub async fn update_qa_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<cx_qa_scoring::QAScoringRule>, (StatusCode, String)> {
    let is_enabled = payload.get("is_enabled").and_then(|v| v.as_bool());
    let score_impact = payload.get("score_impact").and_then(|v| v.as_f64());

    match cx_qa_scoring::update_qa_rule(&state, &rule_id, is_enabled, score_impact).await {
        Ok(rule) => Ok(Json(rule)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
