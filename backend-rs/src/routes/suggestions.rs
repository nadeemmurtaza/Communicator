#![allow(dead_code)]

//! Suggestions API — Phase 9: AI-Powered Suggestions.
//!
//! Endpoints for managing AI-generated suggestions that come from negative
//! feedback analysis.  Suggestions help the team improve the knowledge base,
//! add FAQ entries, and fix agent behavior — closing the self-improvement
//! flywheel.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::db::AppState;
use crate::services::suggestions;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SuggestionsQuery {
    pub status: Option<String>,
    pub suggestion_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct BatchAnalyzeQuery {
    pub hours_back: Option<i64>,
}

#[derive(Deserialize)]
pub struct ApprovePayload {
    pub approved_by: Option<String>,
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

/// List all suggestions with optional filtering.
pub async fn list_suggestions(
    State(state): State<AppState>,
    Query(params): Query<SuggestionsQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = params.limit.unwrap_or(50);

    match suggestions::list_suggestions(
        &state,
        params.status.as_deref(),
        params.suggestion_type.as_deref(),
        limit,
    )
    .await
    {
        Ok(suggestions_list) => (
            StatusCode::OK,
            Json(json!({
                "suggestions": suggestions_list,
                "count": suggestions_list.len(),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to list suggestions: {}", e),
            })),
        ),
    }
}

/// Get suggestion summary analytics.
pub async fn get_suggestion_summary(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    match suggestions::get_suggestion_summary(&state).await {
        Ok(summary) => (
            StatusCode::OK,
            Json(json!({
                "totalSuggestions": summary.total_suggestions,
                "openSuggestions": summary.open_suggestions,
                "approvedSuggestions": summary.approved_suggestions,
                "dismissedSuggestions": summary.dismissed_suggestions,
                "byType": summary.by_type,
                "byCategory": summary.by_category,
                "recentSuggestions": summary.recent_suggestions,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to get suggestion summary: {}", e),
            })),
        ),
    }
}

/// Approve a suggestion.
pub async fn approve_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ApprovePayload>,
) -> (StatusCode, Json<serde_json::Value>) {
    let approved_by = payload.approved_by.unwrap_or_else(|| "system".to_string());

    match suggestions::approve_suggestion(&state, &id, &approved_by).await {
        Ok(suggestion) => (
            StatusCode::OK,
            Json(json!({
                "suggestion": suggestion,
                "status": "approved",
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to approve suggestion: {}", e),
            })),
        ),
    }
}

/// Dismiss a suggestion.
pub async fn dismiss_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match suggestions::dismiss_suggestion(&state, &id).await {
        Ok(suggestion) => (
            StatusCode::OK,
            Json(json!({
                "suggestion": suggestion,
                "status": "dismissed",
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to dismiss suggestion: {}", e),
            })),
        ),
    }
}

/// Trigger batch analysis of negative feedback.
/// This analyzes all unprocessed negative feedback from the last N hours
/// and generates AI-powered suggestions.
pub async fn trigger_batch_analysis(
    State(state): State<AppState>,
    Query(params): Query<BatchAnalyzeQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let hours_back = params.hours_back.unwrap_or(24);

    match suggestions::batch_analyze_negative_feedback(&state, hours_back).await {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "status": "completed",
                "feedbackAnalyzed": result.feedback_analyzed,
                "suggestionsCreated": result.suggestions_created,
                "errors": result.errors,
                "errorDetails": result.error_details,
                "hoursBack": hours_back,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Batch analysis failed: {}", e),
            })),
        ),
    }
}

/// Analyze a single feedback entry and generate suggestions.
/// Used when negative feedback is submitted to trigger immediate analysis.
pub async fn analyze_feedback(
    State(state): State<AppState>,
    Path(feedback_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match suggestions::analyze_negative_feedback(&state, &feedback_id).await {
        Ok(suggestions_list) => (
            StatusCode::OK,
            Json(json!({
                "status": "completed",
                "feedbackId": feedback_id,
                "suggestionsCreated": suggestions_list.len(),
                "suggestions": suggestions_list,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Feedback analysis failed: {}", e),
            })),
        ),
    }
}
