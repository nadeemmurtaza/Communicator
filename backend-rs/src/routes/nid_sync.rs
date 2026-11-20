//! NID Sync Routes
//!
//! Thin gRPC client forwarding to svc-nid-sync; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;

use newax_proto::nid_sync::v1::{
    nid_sync_service_client::NidSyncServiceClient,
    Empty, IdRequest, SetOrderPlatformInfoRequest,
};

async fn client() -> Result<NidSyncServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("NID_SYNC_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50097".to_string());
    NidSyncServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("nid-sync service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

// ——— Query Params ———

#[derive(Deserialize)]
pub struct AccountQuery {
    pub account_id: String,
}

#[derive(Deserialize)]
pub struct OrgQuery {
    pub org_id: String,
}

#[derive(Deserialize)]
pub struct TenantQuery {
    pub tenant_id: String,
}

#[derive(Deserialize)]
pub struct SetPlatformRequest {
    pub order_id: String,
    pub platform_name: String,
    pub platform_type: String,
    pub brand_name: String,
    pub shop_name: Option<String>,
}

// ——— Handlers ———

pub async fn get_launchpad_overview(
    State(_state): State<AppState>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_launchpad_overview(IdRequest { id: query.account_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_personal_profile(
    State(_state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_personal_profile(IdRequest { id: account_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_work_profile(
    State(_state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_work_profile(IdRequest { id: org_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_user_organizations(
    State(_state): State<AppState>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_user_organizations(IdRequest { id: query.account_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_user_memberships(
    State(_state): State<AppState>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_user_memberships(IdRequest { id: query.account_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_invoices(
    State(_state): State<AppState>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_invoices(IdRequest { id: query.tenant_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payments(
    State(_state): State<AppState>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payments(IdRequest { id: query.tenant_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_subscriptions(
    State(_state): State<AppState>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_subscriptions(IdRequest { id: query.tenant_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn set_order_platform_info(
    State(_state): State<AppState>,
    Json(req): Json<SetPlatformRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    cli.set_order_platform_info(SetOrderPlatformInfoRequest {
        order_id: req.order_id,
        platform_name: req.platform_name,
        platform_type: req.platform_type,
        brand_name: req.brand_name,
        shop_name: req.shop_name,
    }).await.map_err(grpc_err)?;
    Ok(StatusCode::OK)
}

pub async fn get_tenants(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_tenants(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}
