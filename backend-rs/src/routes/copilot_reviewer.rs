//! Thin gRPC client forwarding copilot reviewer requests to `svc-copilot-reviewer` (port 50094).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::copilot_reviewer::v1::{
    copilot_reviewer_service_client::CopilotReviewerServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("COPILOT_REVIEWER_GRPC").unwrap_or("http://127.0.0.1:50094")
}

async fn client() -> CopilotReviewerServiceClient<tonic::transport::Channel> {
    CopilotReviewerServiceClient::connect(svc_addr().to_string())
        .await
        .expect("copilot-reviewer gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn err_response(message: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": message }))).into_response()
}

pub async fn get_draft_context(
    State(_state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Response {
    let mut req = newax_proto::copilot_reviewer::v1::GetDraftContextRequest::default();
    req.draft_id = draft_id;
    match client().await.get_draft_context(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_pending_contexts(
    State(_state): State<AppState>,
) -> Response {
    match client().await.get_pending_contexts(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn refresh_draft_context(
    State(_state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Response {
    let mut req = newax_proto::copilot_reviewer::v1::RefreshDraftContextRequest::default();
    req.draft_id = draft_id;
    match client().await.refresh_draft_context(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}
