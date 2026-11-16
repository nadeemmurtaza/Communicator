//! Suggestions API — Phase 9: AI-Powered Suggestions.
//!
//! Thin gRPC client forwarding to svc-suggestions; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::db::AppState;

use newax_proto::suggestions::v1::{
    suggestions_service_client::SuggestionsServiceClient,
    Empty, IdRequest, JsonResponse, ListSuggestionsRequest,
    ApproveSuggestionRequest, BatchAnalyzeRequest,
};

async fn client() -> Result<SuggestionsServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("SUGGESTIONS_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50083".to_string());
    SuggestionsServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("suggestions service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

fn bad_request_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.message().to_string())
}

fn parse_json(s: &str) -> Result<serde_json::Value, (StatusCode, String)> {
    serde_json::from_str(s).map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid json: {e}")))
}

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

pub async fn list_suggestions(
    State(_state): State<AppState>,
    Query(params): Query<SuggestionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50);
    let mut c = client().await?;
    let resp = c
        .list_suggestions(ListSuggestionsRequest {
            status: params.status,
            suggestion_type: params.suggestion_type,
            limit,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let suggestions_list = parse_json(&resp.json)?;
    Ok(Json(json!({
        "suggestions": suggestions_list,
        "count": suggestions_list.as_array().map(|a| a.len()).unwrap_or(0),
    })))
}

pub async fn get_suggestion_summary(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    let summary = parse_json(&resp.json)?;
    Ok(Json(json!({
        "totalSuggestions": summary.get("total_suggestions"),
        "openSuggestions": summary.get("open_suggestions"),
        "approvedSuggestions": summary.get("approved_suggestions"),
        "dismissedSuggestions": summary.get("dismissed_suggestions"),
        "byType": summary.get("by_type"),
        "byCategory": summary.get("by_category"),
        "recentSuggestions": summary.get("recent_suggestions"),
    })))
}

pub async fn approve_suggestion(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ApprovePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let approved_by = payload.approved_by.unwrap_or_else(|| "system".to_string());
    let mut c = client().await?;
    let resp = c
        .approve_suggestion(ApproveSuggestionRequest { id, approved_by })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    let suggestion = parse_json(&resp.json)?;
    Ok(Json(json!({
        "suggestion": suggestion,
        "status": "approved",
    })))
}

pub async fn dismiss_suggestion(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.dismiss_suggestion(IdRequest { id }).await.map_err(grpc_err)?.into_inner();
    let suggestion = parse_json(&resp.json)?;
    Ok(Json(json!({
        "suggestion": suggestion,
        "status": "dismissed",
    })))
}

pub async fn trigger_batch_analysis(
    State(_state): State<AppState>,
    Query(params): Query<BatchAnalyzeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let hours_back = params.hours_back.unwrap_or(24);
    let mut c = client().await?;
    let resp = c
        .batch_analyze(BatchAnalyzeRequest { hours_back })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let result = parse_json(&resp.json)?;
    Ok(Json(json!({
        "status": "completed",
        "feedbackAnalyzed": result.get("feedback_analyzed"),
        "suggestionsCreated": result.get("suggestions_created"),
        "errors": result.get("errors"),
        "errorDetails": result.get("error_details"),
        "hoursBack": hours_back,
    })))
}

pub async fn analyze_feedback(
    State(_state): State<AppState>,
    Path(feedback_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .analyze_feedback(IdRequest { id: feedback_id.clone() })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let suggestions_list = parse_json(&resp.json)?;
    Ok(Json(json!({
        "status": "completed",
        "feedbackId": feedback_id,
        "suggestionsCreated": suggestions_list.as_array().map(|a| a.len()).unwrap_or(0),
        "suggestions": suggestions_list,
    })))
}
