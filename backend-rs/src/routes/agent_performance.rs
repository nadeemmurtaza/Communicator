//! Thin gRPC client forwarding agent performance requests to `svc-agent-performance` (port 50087).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::agent_performance::v1::{
    agent_performance_service_client::AgentPerformanceServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("AGENT_PERFORMANCE_GRPC").unwrap_or("http://127.0.0.1:50087")
}

async fn client() -> AgentPerformanceServiceClient<tonic::transport::Channel> {
    AgentPerformanceServiceClient::connect(svc_addr().to_string())
        .await
        .expect("agent-performance gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn created_response(v: Value) -> Response {
    (StatusCode::CREATED, Json(v)).into_response()
}

fn err_response(message: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": message }))).into_response()
}

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

pub async fn get_summary(State(_state): State<AppState>) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn list_snapshots(
    State(_state): State<AppState>,
    Query(query): Query<ListSnapshotsQuery>,
) -> Response {
    let mut req = newax_proto::agent_performance::v1::ListSnapshotsRequest::default();
    req.agent_id = query.agent_id.clone().unwrap_or_default();
    req.days = query.days.unwrap_or(7);
    match client().await.list_snapshots(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_snapshot(
    State(_state): State<AppState>,
    Json(data): Json<Value>,
) -> Response {
    let agent_id = data.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
    let agent_name = data.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    let mut req = newax_proto::agent_performance::v1::CreateSnapshotRequest::default();
    req.agent_id = agent_id.to_string();
    req.agent_name = agent_name.to_string();
    req.data = serde_json::to_string(&data).unwrap_or_default();
    match client().await.create_snapshot(req).await {
        Ok(resp) => created_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn list_insights(
    State(_state): State<AppState>,
    Query(query): Query<ListInsightsQuery>,
) -> Response {
    let mut req = newax_proto::agent_performance::v1::ListInsightsRequest::default();
    req.agent_id = query.agent_id.clone().unwrap_or_default();
    req.addressed = query.addressed.unwrap_or(false);
    req.has_addressed = query.addressed.is_some();
    match client().await.list_insights(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_insight(
    State(_state): State<AppState>,
    Json(data): Json<Value>,
) -> Response {
    let agent_id = data.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
    let agent_name = data.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    let mut req = newax_proto::agent_performance::v1::CreateInsightRequest::default();
    req.agent_id = agent_id.to_string();
    req.agent_name = agent_name.to_string();
    req.data = serde_json::to_string(&data).unwrap_or_default();
    match client().await.create_insight(req).await {
        Ok(resp) => created_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn address_insight(
    State(_state): State<AppState>,
    Path(insight_id): Path<String>,
    Json(data): Json<Value>,
) -> Response {
    let addressed_by = data.get("addressed_by").and_then(|v| v.as_str()).unwrap_or("admin");
    let mut req = newax_proto::agent_performance::v1::AddressInsightRequest::default();
    req.insight_id = insight_id;
    req.addressed_by = addressed_by.to_string();
    match client().await.address_insight(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_comparison(
    State(_state): State<AppState>,
    Query(query): Query<ComparisonQuery>,
) -> Response {
    let mut req = newax_proto::agent_performance::v1::GetComparisonRequest::default();
    req.days = query.days.unwrap_or(7);
    match client().await.get_comparison(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn generate_comparison(State(_state): State<AppState>) -> Response {
    match client().await.generate_comparison(Empty {}).await {
        Ok(resp) => created_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_settings(State(_state): State<AppState>) -> Response {
    match client().await.get_settings(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Json(data): Json<Value>,
) -> Response {
    let mut req = newax_proto::agent_performance::v1::UpdateSettingsRequest::default();
    req.data = serde_json::to_string(&data).unwrap_or_default();
    match client().await.update_settings(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}
