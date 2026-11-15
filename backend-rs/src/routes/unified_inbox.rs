//! Unified Inbox API routes (Phase 19).
//!
//! Thin gRPC client forwarding to svc-unified-inbox; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tonic::transport::Channel;

use crate::db::AppState;
use newax_proto::unified_inbox::v1::{
    unified_inbox_service_client::UnifiedInboxServiceClient,
    IdRequest, JsonResponse, ListInboxRequest, ResolveCustomerRequest,
    CreateMappingRequest, GetSettingsRequest, UpdateSettingsRequest,
};

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn grpc_err(status: tonic::Status) -> Response {
    let code = match status.code() {
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    err(code, status.message().to_string())
}

async fn client() -> Result<UnifiedInboxServiceClient<Channel>, Response> {
    let url = std::env::var("UNIFIED_INBOX_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50081".to_string());
    UnifiedInboxServiceClient::connect(url)
        .await
        .map_err(|e| err(StatusCode::SERVICE_UNAVAILABLE, format!("unified inbox service unreachable: {e}")))
}

fn json_value(json_str: String) -> serde_json::Value {
    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
}

#[derive(Deserialize)]
pub struct UnifiedInboxQuery {
    limit: Option<i32>,
    offset: Option<i32>,
    channel: Option<String>,
    state: Option<String>,
    search: Option<String>,
}

// ——— Handlers ———

/// GET /unified-inbox — List customers with their conversations grouped
pub async fn list_unified_inbox(
    State(_state): State<AppState>,
    Query(params): Query<UnifiedInboxQuery>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ListInboxRequest {
        limit: params.limit.unwrap_or(20),
        offset: params.offset.unwrap_or(0),
        channel: params.channel,
        state: params.state,
        search: params.search,
    };
    match c.list_inbox(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /unified-inbox/{customer_id} — Get unified view for a specific customer
pub async fn get_unified_customer(
    State(_state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id: customer_id };
    match c.get_customer(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /unified-inbox/resolve — Resolve customer identity from channel + identifier
pub async fn resolve_customer(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let channel = match payload.get("channel").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "channel is required"),
    };
    let identifier = match payload.get("identifier").and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "identifier is required"),
    };
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ResolveCustomerRequest { channel, identifier };
    match c.resolve_customer(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /unified-inbox/mapping — Create manual customer mapping
pub async fn create_customer_mapping(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let customer_id = match payload.get("customer_id").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "customer_id is required"),
    };
    let contact_id = match payload.get("contact_id").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "contact_id is required"),
    };
    let channel = match payload.get("channel").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "channel is required"),
    };
    let channel_identifier = match payload.get("channel_identifier").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "channel_identifier is required"),
    };
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = CreateMappingRequest {
        customer_id,
        contact_id,
        channel,
        channel_identifier,
    };
    match c.create_mapping(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /unified-inbox/settings — Get unified inbox settings
pub async fn get_unified_inbox_settings(
    State(_state): State<AppState>,
    Query(params): Query<UnifiedInboxQuery>,
) -> Response {
    let org_id = params.search.as_deref().unwrap_or("default");
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = GetSettingsRequest {
        organization_id: org_id.to_string(),
    };
    match c.get_settings(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// PUT /unified-inbox/settings — Update unified inbox settings
pub async fn update_unified_inbox_settings(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let org_id = payload.get("organization_id").and_then(|v| v.as_str()).unwrap_or("default");
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = UpdateSettingsRequest {
        organization_id: org_id.to_string(),
        payload_json: payload.to_string(),
    };
    match c.update_settings(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}
