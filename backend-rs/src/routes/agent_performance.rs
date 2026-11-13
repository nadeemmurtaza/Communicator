//! Agent Performance Analytics Routes — Phase 30

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;
use crate::services::agent_performance;

#[derive(Deserialize)]
pub struct ListSnapshotsQuery {
    pub agent_id: Option<String>,
    pub days: Option<i32>,
}

#[derive(Deserialize)]
pub struct ListInsightsQuery {
    pub agent_id: Option<String>,
    pub addressed: Option<bool>,
}

#[derive(Deserialize)]
pub struct ComparisonQuery {
    pub days: Option<i32>,
}

/// GET /agent-performance/summary
pub async fn get_summary(state: State<AppState>) -> (StatusCode, Json<Value>) {
    match agent_performance::get_performance_summary(&state).await {
        Ok(summary) => (StatusCode::OK, Json(serde_json::to_value(summary).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// GET /agent-performance/snapshots
pub async fn list_snapshots(
    State(state): State<AppState>,
    Query(query): Query<ListSnapshotsQuery>,
) -> (StatusCode, Json<Value>) {
    let days = query.days.unwrap_or(7);
    match agent_performance::list_snapshots(&state, query.agent_id.as_deref(), days).await {
        Ok(snapshots) => (StatusCode::OK, Json(serde_json::to_value(snapshots).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// POST /agent-performance/snapshots
pub async fn create_snapshot(
    State(state): State<AppState>,
    Json(data): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = data.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
    let agent_name = data.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    match agent_performance::create_snapshot(&state, agent_id, agent_name, &data).await {
        Ok(result) => (StatusCode::CREATED, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// GET /agent-performance/insights
pub async fn list_insights(
    State(state): State<AppState>,
    Query(query): Query<ListInsightsQuery>,
) -> (StatusCode, Json<Value>) {
    match agent_performance::list_insights(&state, query.agent_id.as_deref(), query.addressed).await {
        Ok(insights) => (StatusCode::OK, Json(serde_json::to_value(insights).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// POST /agent-performance/insights
pub async fn create_insight(
    State(state): State<AppState>,
    Json(data): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = data.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
    let agent_name = data.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    match agent_performance::create_insight(&state, agent_id, agent_name, &data).await {
        Ok(result) => (StatusCode::CREATED, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// PUT /agent-performance/insights/{id}/address
pub async fn address_insight(
    State(state): State<AppState>,
    Path(insight_id): Path<String>,
    Json(data): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let addressed_by = data.get("addressed_by").and_then(|v| v.as_str()).unwrap_or("admin");
    match agent_performance::address_insight(&state, &insight_id, addressed_by).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// GET /agent-performance/comparison
pub async fn get_comparison(
    State(state): State<AppState>,
    Query(query): Query<ComparisonQuery>,
) -> (StatusCode, Json<Value>) {
    let days = query.days.unwrap_or(7);
    match agent_performance::get_comparison_metrics(&state, days).await {
        Ok(metrics) => (StatusCode::OK, Json(serde_json::to_value(metrics).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// POST /agent-performance/comparison/generate
pub async fn generate_comparison(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match agent_performance::generate_comparison_snapshot(&state).await {
        Ok(metrics) => (StatusCode::CREATED, Json(serde_json::to_value(metrics).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// GET /agent-performance/settings
pub async fn get_settings(state: State<AppState>) -> (StatusCode, Json<Value>) {
    match agent_performance::get_settings(&state).await {
        Ok(settings) => (StatusCode::OK, Json(serde_json::to_value(settings).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// PUT /agent-performance/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(data): Json<Value>,
) -> (StatusCode, Json<Value>) {
    match agent_performance::update_settings(&state, &data).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}
