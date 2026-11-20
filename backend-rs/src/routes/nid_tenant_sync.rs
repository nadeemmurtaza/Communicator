//! nID Tenant Sync Routes — thin gRPC client forwarding to svc-nid-tenant-sync

use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::db::AppState;

use newax_proto::nid_tenant_sync::v1::nid_tenant_sync_service_client::NidTenantSyncServiceClient;
use newax_proto::nid_tenant_sync::v1::{
    ProcessWebhookRequest, ListTenantsRequest, GetTenantRequest, GetConnectionHealthRequest,
    UpdateConnectionSettingsRequest, CreateRedirectSessionRequest, HandleRedirectCallbackRequest,
    GetRedirectSessionRequest, GetSyncLogRequest, JsonBody,
};

async fn client(state: &AppState) -> Result<NidTenantSyncServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = if state.config.nid_tenant_sync_grpc_url.is_empty() {
        "http://127.0.0.1:50103".to_string()
    } else {
        state.config.nid_tenant_sync_grpc_url.clone()
    };
    NidTenantSyncServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("gRPC connection failed: {}", e)))
}

fn parse_json_resp(resp: tonic::Response<newax_proto::nid_tenant_sync::v1::JsonResponse>) -> serde_json::Value {
    let json_str = resp.into_inner().json;
    serde_json::from_str(&json_str).unwrap_or_else(|_| serde_json::json!({"raw": json_str}))
}

/// POST /api/v1/nid/webhook
pub async fn nid_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let signature = headers.get("X-nID-Signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    let payload_str = payload.to_string();
    let req = ProcessWebhookRequest { payload: payload_str, signature: signature.to_string() };
    let resp = c.process_webhook(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /api/v1/nid/tenants
pub async fn list_nid_tenants(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let resp = c.list_tenants(tonic::Request::new(ListTenantsRequest::default())).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /api/v1/nid/tenants/{tenant_id}
pub async fn get_nid_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = GetTenantRequest { tenant_id };
    let resp = c.get_tenant(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /api/v1/nid/connection
pub async fn get_nid_connection(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let resp = c.get_connection_health(tonic::Request::new(GetConnectionHealthRequest::default())).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// PUT /api/v1/nid/connection
pub async fn update_nid_connection(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = UpdateConnectionSettingsRequest { body: Some(JsonBody { json: json_str }) };
    let resp = c.update_connection_settings(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// POST /api/v1/nid/redirect/create
pub async fn create_redirect_session(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = CreateRedirectSessionRequest { body: Some(JsonBody { json: json_str }) };
    let resp = c.create_redirect_session(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /api/v1/nid/redirect/callback
pub async fn redirect_callback(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let state_token = params.get("state").cloned().ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid state parameter".to_string()))?;
    let tenant_id = params.get("tenant_id").cloned().unwrap_or_default();
    let error = params.get("error").cloned().unwrap_or_default();
    let req = HandleRedirectCallbackRequest { state: state_token, tenant_id, error };
    let resp = c.handle_redirect_callback(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /api/v1/nid/redirect/{state}
pub async fn get_redirect_session(
    State(state): State<AppState>,
    Path(state_token): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = GetRedirectSessionRequest { state: state_token };
    let resp = c.get_redirect_session(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /api/v1/nid/sync-log
pub async fn get_sync_log(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let event_type = params.get("event_type").cloned().unwrap_or_default();
    let req = GetSyncLogRequest { limit, event_type };
    let resp = c.get_sync_log(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}
