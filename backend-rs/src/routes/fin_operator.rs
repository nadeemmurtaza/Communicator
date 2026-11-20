//! Fin Operator — thin gRPC client forwarding to svc-fin-operator

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;

use newax_proto::fin_operator::v1::fin_operator_service_client::FinOperatorServiceClient;
use newax_proto::fin_operator::v1::{
    Empty, UpdateConfigRequest, ListProposalsRequest, IdRequest,
    ApproveProposalRequest, DismissProposalRequest, ListLogsRequest, ListSnapshotsRequest, JsonBody,
};

#[derive(Deserialize, Default)]
pub struct ProposalQueryParams {
    pub status: Option<String>,
    pub proposal_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize, Default)]
pub struct LogQueryParams {
    pub action_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize, Default)]
pub struct SnapshotQueryParams {
    pub limit: Option<i32>,
}

async fn client(state: &AppState) -> Result<FinOperatorServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = if state.config.fin_operator_grpc_url.is_empty() {
        "http://127.0.0.1:50104".to_string()
    } else {
        state.config.fin_operator_grpc_url.clone()
    };
    FinOperatorServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("gRPC connection failed: {}", e)))
}

fn parse_json_resp(resp: tonic::Response<newax_proto::fin_operator::v1::JsonResponse>) -> serde_json::Value {
    let json_str = resp.into_inner().json;
    serde_json::from_str(&json_str).unwrap_or_else(|_| serde_json::json!({"raw": json_str}))
}

/// GET /operator/summary
pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let resp = c.get_summary(tonic::Request::new(Empty {})).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /operator/config
pub async fn get_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let resp = c.get_config(tonic::Request::new(Empty {})).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// PUT /operator/config
pub async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = UpdateConfigRequest { body: Some(JsonBody { json: json_str }) };
    let resp = c.update_config(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// POST /operator/scan
pub async fn run_scan(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let resp = c.run_scan(tonic::Request::new(Empty {})).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /operator/proposals
pub async fn list_proposals(
    State(state): State<AppState>,
    Query(params): Query<ProposalQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let status = params.status.unwrap_or_default();
    let proposal_type = params.proposal_type.unwrap_or_default();
    let limit = params.limit.unwrap_or(50);
    let req = ListProposalsRequest { status, proposal_type, limit };
    let resp = c.list_proposals(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /operator/proposals/{id}
pub async fn get_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id };
    let resp = c.get_proposal(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// POST /operator/proposals/{id}/approve
pub async fn approve_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let reviewer = body.get("reviewer").and_then(|v| v.as_str()).unwrap_or("admin").to_string();
    let req = ApproveProposalRequest { id, reviewer };
    let resp = c.approve_proposal(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// POST /operator/proposals/{id}/dismiss
pub async fn dismiss_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let reviewer = body.get("reviewer").and_then(|v| v.as_str()).unwrap_or("admin").to_string();
    let req = DismissProposalRequest { id, reviewer };
    let resp = c.dismiss_proposal(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /operator/logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let action_type = params.action_type.unwrap_or_default();
    let limit = params.limit.unwrap_or(50);
    let req = ListLogsRequest { action_type, limit };
    let resp = c.list_logs(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /operator/snapshots
pub async fn list_snapshots(
    State(state): State<AppState>,
    Query(params): Query<SnapshotQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let limit = params.limit.unwrap_or(20);
    let req = ListSnapshotsRequest { limit };
    let resp = c.list_snapshots(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}
