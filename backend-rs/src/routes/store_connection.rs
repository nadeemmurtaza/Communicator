//! Platform-Agnostic Store Connection Management Routes
//!
//! API endpoints for managing store connections, checking their status,
//! verifying UI compatibility, and managing brand OTP isolation.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use crate::db::DbPool;
use crate::services::store_connection::{
    StoreConnectionService, CreateStoreConnectionRequest, UpdateStoreConnectionRequest,
    StoreAccountCreationRequest, StoreOtpVerificationRequest, UiCompatibilityReport,
};

#[derive(Debug, Deserialize)]
pub struct BrandQuery {
    pub brand_id: String,
}

// ---------------------------------------------------------------------------
// Store connection CRUD
// ---------------------------------------------------------------------------

/// Create a new store connection for any platform.
pub async fn create_connection(
    State(pool): State<DbPool>,
    Json(req): Json<CreateStoreConnectionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::create_connection(&pool, &req).await {
        Ok(resp) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": resp,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Get a store connection by ID.
pub async fn get_connection(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::get_by_id(&pool, &connection_id).await {
        Ok(conn) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": conn,
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// List all store connections for a brand.
pub async fn list_connections(
    State(pool): State<DbPool>,
    Query(params): Query<BrandQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::list_by_brand(&pool, &params.brand_id).await {
        Ok(conns) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": conns,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Update a store connection.
pub async fn update_connection(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
    Json(req): Json<UpdateStoreConnectionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::update_connection(&pool, &connection_id, &req).await {
        Ok(conn) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": conn,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Deactivate a store connection.
pub async fn deactivate_connection(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::deactivate_connection(&pool, &connection_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Connection deactivated",
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Connection status
// ---------------------------------------------------------------------------

/// Get connection status for a store.
pub async fn get_connection_status(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::get_connection_status(&pool, &connection_id).await {
        Ok(status) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": status,
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Run a health check on a store connection.
pub async fn run_health_check(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::run_health_check(&pool, &connection_id).await {
        Ok(status) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": status,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// UI Compatibility
// ---------------------------------------------------------------------------

/// Submit a UI compatibility report from the widget.
pub async fn submit_ui_compatibility_report(
    State(pool): State<DbPool>,
    Path(brand_id): Path<String>,
    Json(report): Json<UiCompatibilityReport>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::submit_ui_compatibility_report(&pool, &brand_id, &report).await {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": result,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Get the latest UI compatibility result for a store.
pub async fn get_ui_compatibility(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::get_latest_ui_compatibility(&pool, &connection_id).await {
        Ok(Some(result)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": result,
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "No UI compatibility check found",
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Brand OTP Isolation
// ---------------------------------------------------------------------------

/// Get brand OTP isolation config.
pub async fn get_brand_isolation(
    State(pool): State<DbPool>,
    Path(brand_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::get_brand_isolation(&pool, &brand_id).await {
        Ok(iso) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": iso,
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Update brand OTP isolation settings.
pub async fn update_brand_isolation(
    State(pool): State<DbPool>,
    Path(brand_id): Path<String>,
    Json(req): Json<UpdateIsolationRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::update_brand_isolation(
        &pool, &brand_id,
        req.isolation_enabled,
        req.isolation_mode,
        req.rate_limit_per_minute,
        req.rate_limit_per_hour,
        req.cooldown_seconds,
        req.lockout_seconds,
    ).await {
        Ok(iso) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": iso,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
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

// ---------------------------------------------------------------------------
// Store account creation (platform-agnostic)
// ---------------------------------------------------------------------------

/// Initiate account creation for a store customer.
pub async fn initiate_account_creation(
    State(pool): State<DbPool>,
    Path(connection_id): Path<String>,
    Json(req): Json<StoreAccountCreationRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::initiate_account_creation(&pool, &connection_id, &req).await {
        Ok(resp) => {
            let status = if resp.success {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (
                status,
                Json(serde_json::json!({
                    "success": resp.success,
                    "data": resp,
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Verify OTP for store account creation.
pub async fn verify_account_creation(
    State(pool): State<DbPool>,
    Json(req): Json<StoreOtpVerificationRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::verify_account_creation(&pool, &req).await {
        Ok(resp) => {
            let status = if resp.success {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (
                status,
                Json(serde_json::json!({
                    "success": resp.success,
                    "data": resp,
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// Resend OTP for a pending account creation.
pub async fn resend_otp(
    State(pool): State<DbPool>,
    Path(pending_id): Path<String>,
    Json(req): Json<ResendOtpRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match StoreConnectionService::resend_otp(
        &pool, &pending_id,
        req.ip_address.as_deref(),
        req.user_agent.as_deref(),
    ).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": resp.success,
                "data": resp,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

#[derive(serde::Deserialize)]
pub struct ResendOtpRequest {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
