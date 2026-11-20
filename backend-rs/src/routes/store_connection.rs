//! Store Connection — thin gRPC client forwarding to svc-store-connection

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::store_connection::v1::store_connection_service_client::StoreConnectionServiceClient;
use newax_proto::store_connection::v1::{
    IdRequest, BrandRequest, JsonBody,
    CreateConnectionRequest, UpdateConnectionRequest,
    SubmitUiCompatibilityRequest, UpdateBrandIsolationRequest,
    InitiateAccountCreationRequest, VerifyAccountCreationRequest, ResendOtpRequest,
};

#[derive(Deserialize, Default)]
pub struct BrandQuery {
    pub brand_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateIsolationRequest {
    pub isolation_enabled: Option<bool>,
    pub isolation_mode: Option<String>,
    pub rate_limit_per_minute: Option<i64>,
    pub rate_limit_per_hour: Option<i64>,
    pub cooldown_seconds: Option<i64>,
    pub lockout_seconds: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct ResendOtpQuery {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

async fn client(state: &AppState) -> Result<StoreConnectionServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = if state.config.store_connection_grpc_url.is_empty() {
        "http://127.0.0.1:50106".to_string()
    } else {
        state.config.store_connection_grpc_url.clone()
    };
    StoreConnectionServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("gRPC connection failed: {}", e)))
}

fn parse_json_resp(resp: tonic::Response<newax_proto::store_connection::v1::JsonResponse>) -> Value {
    let json_str = resp.into_inner().json;
    serde_json::from_str(&json_str).unwrap_or_else(|_| serde_json::json!({"raw": json_str}))
}

// ---------------------------------------------------------------------------
// Store connection CRUD
// ---------------------------------------------------------------------------

pub async fn create_connection(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let json_str = serde_json::to_string(&payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req = CreateConnectionRequest { body: Some(JsonBody { json: json_str }) };
    let resp = c.create_connection(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn get_connection(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: connection_id };
    let resp = c.get_connection(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn list_connections(
    State(state): State<AppState>,
    Query(params): Query<BrandQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let brand_id = params.brand_id.unwrap_or_default();
    let mut c = client(&state).await?;
    let req = BrandRequest { brand_id };
    let resp = c.list_connections(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn update_connection(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let json_str = serde_json::to_string(&payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req = UpdateConnectionRequest {
        connection_id,
        body: Some(JsonBody { json: json_str }),
    };
    let resp = c.update_connection(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn deactivate_connection(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: connection_id };
    let resp = c.deactivate_connection(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

// ---------------------------------------------------------------------------
// Connection status
// ---------------------------------------------------------------------------

pub async fn get_connection_status(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: connection_id };
    let resp = c.get_connection_status(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn run_health_check(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: connection_id };
    let resp = c.run_health_check(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

// ---------------------------------------------------------------------------
// UI Compatibility
// ---------------------------------------------------------------------------

pub async fn submit_ui_compatibility_report(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let json_str = serde_json::to_string(&payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req = SubmitUiCompatibilityRequest {
        brand_id,
        body: Some(JsonBody { json: json_str }),
    };
    let resp = c.submit_ui_compatibility_report(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn get_ui_compatibility(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: connection_id };
    let resp = c.get_latest_ui_compatibility(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

// ---------------------------------------------------------------------------
// Brand OTP Isolation
// ---------------------------------------------------------------------------

pub async fn get_brand_isolation(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = BrandRequest { brand_id };
    let resp = c.get_brand_isolation(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn update_brand_isolation(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Json(req): Json<UpdateIsolationRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let body = serde_json::json!({
        "isolation_enabled": req.isolation_enabled,
        "isolation_mode": req.isolation_mode,
        "rate_limit_per_minute": req.rate_limit_per_minute,
        "rate_limit_per_hour": req.rate_limit_per_hour,
        "cooldown_seconds": req.cooldown_seconds,
        "lockout_seconds": req.lockout_seconds,
    });
    let json_str = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req_msg = UpdateBrandIsolationRequest {
        brand_id,
        body: Some(JsonBody { json: json_str }),
    };
    let resp = c.update_brand_isolation(tonic::Request::new(req_msg)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

// ---------------------------------------------------------------------------
// Store account creation
// ---------------------------------------------------------------------------

pub async fn initiate_account_creation(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let json_str = serde_json::to_string(&payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req = InitiateAccountCreationRequest {
        connection_id,
        body: Some(JsonBody { json: json_str }),
    };
    let resp = c.initiate_account_creation(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn verify_account_creation(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let json_str = serde_json::to_string(&payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req = VerifyAccountCreationRequest { body: Some(JsonBody { json: json_str }) };
    let resp = c.verify_account_creation(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

pub async fn resend_otp(
    State(state): State<AppState>,
    Path(pending_id): Path<String>,
    Json(req): Json<ResendOtpQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let body = serde_json::json!({
        "ip_address": req.ip_address,
        "user_agent": req.user_agent,
    });
    let json_str = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut c = client(&state).await?;
    let req_msg = ResendOtpRequest {
        pending_id,
        body: Some(JsonBody { json: json_str }),
    };
    let resp = c.resend_otp(tonic::Request::new(req_msg)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}
