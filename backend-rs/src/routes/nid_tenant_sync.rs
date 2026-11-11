//! nID Tenant Sync Routes
//!
//! API endpoints for managing the nID tenant connection:
//! - Webhook reception from nID
//! - Tenant listing and status
//! - Redirect flow for tenant creation
//! - Connection settings management

use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::db::DbPool;
use crate::services::nid_tenant_sync::{
    CreateRedirectSessionRequest, NidTenantSyncService, UpdateConnectionSettingsRequest,
};

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct WebhookRequest {
    pub event_type: String,
    pub tenant_id: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub status: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/nid/webhook
/// Receives webhook events from nID for tenant creation/updates
pub async fn nid_webhook(
    State(pool): State<DbPool>,
    headers: axum::headers::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<WebhookResponse>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    // Extract signature from header
    let signature = headers
        .get("X-nID-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let payload_str = payload.to_string();
    match service.process_webhook(&payload_str, signature).await {
        Ok(result) => {
            info!("nID webhook processed successfully");
            Ok(Json(WebhookResponse {
                status: "ok".to_string(),
                message: result.to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("nID webhook processing failed: {}", e);
            Err((StatusCode::BAD_REQUEST, e))
        }
    }
}

/// GET /api/v1/nid/tenants
/// List all synced nID tenants
pub async fn list_nid_tenants(
    State(pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    match service.list_tenants().await {
        Ok(tenants) => Ok(Json(serde_json::json!({
            "tenants": tenants,
            "count": tenants.len()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /api/v1/nid/tenants/{tenant_id}
/// Get a specific nID tenant by ID
pub async fn get_nid_tenant(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    match service.get_tenant(&tenant_id).await {
        Ok(Some(tenant)) => Ok(Json(serde_json::json!(tenant))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Tenant not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /api/v1/nid/connection
/// Get nID connection settings and health
pub async fn get_nid_connection(
    State(pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);
    let health = service.check_connection_health().await;
    Ok(Json(health))
}

/// PUT /api/v1/nid/connection
/// Update nID connection settings
pub async fn update_nid_connection(
    State(pool): State<DbPool>,
    Json(req): Json<UpdateConnectionSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    match service.update_connection_settings(&req).await {
        Ok(settings) => {
            info!("nID connection settings updated");
            Ok(Json(serde_json::json!(settings)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// POST /api/v1/nid/redirect/create
/// Create a redirect session for tenant creation in nID
pub async fn create_redirect_session(
    State(pool): State<DbPool>,
    Json(req): Json<CreateRedirectSessionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    match service.create_redirect_session(&req).await {
        Ok(session) => {
            info!("Redirect session created: {}", session.state_token);
            Ok(Json(serde_json::json!({
                "session": session,
                "redirect_url": format!(
                    "{}/tenant-create?state={}&return_url={}",
                    service.get_connection_settings().await.nID_base_url,
                    session.state_token,
                    urlencoding::encode(&session.return_url)
                )
            })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /api/v1/nid/redirect/callback
/// Handle callback from nID after tenant creation
pub async fn redirect_callback(
    State(pool): State<DbPool>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    let state = params
        .get("state")
        .ok_or(("Invalid state parameter".to_string()))?;
    let tenant_id = params.get("tenant_id").map(|s| s.as_str());
    let error = params.get("error").map(|s| s.as_str());

    match service.handle_redirect_callback(state, tenant_id, error).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// GET /api/v1/nid/redirect/{state}
/// Get redirect session status
pub async fn get_redirect_session(
    State(pool): State<DbPool>,
    Path(state): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = NidTenantSyncService::new(&pool);

    match service.get_redirect_session(&state) {
        Some(session) => Ok(Json(serde_json::json!(session))),
        None => Err((StatusCode::NOT_FOUND, "Redirect session not found".to_string())),
    }
}

/// GET /api/v1/nid/sync-log
/// Get tenant sync log entries
pub async fn get_sync_log(
    State(pool): State<DbPool>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let event_type = params.get("event_type").cloned();

    let pool_ref = pool.get_pg_pool();

    let query = if let Some(ref et) = event_type {
        format!(
            r#"SELECT id, nid_tenant_id, event_type, processing_status, error_message, created_at
               FROM tenant_sync_log
               WHERE event_type = '{}'
               ORDER BY created_at DESC LIMIT {}"#,
            et.replace('\'', "''"),
            limit
        )
    } else {
        format!(
            r#"SELECT id, nid_tenant_id, event_type, processing_status, error_message, created_at
               FROM tenant_sync_log
               ORDER BY created_at DESC LIMIT {}"#,
            limit
        )
    };

    let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(&query)
        .fetch_all(pool_ref)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "logs": rows.iter().map(|(id, nid_tenant_id, event_type, processing_status, error_message, created_at)| {
            serde_json::json!({
                "id": id,
                "nid_tenant_id": nid_tenant_id,
                "event_type": event_type,
                "processing_status": processing_status,
                "error_message": error_message,
                "created_at": created_at,
            })
        }).collect::<Vec<_>>(),
        "count": rows.len()
    })))
}
