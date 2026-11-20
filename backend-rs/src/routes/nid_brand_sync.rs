use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use std::collections::HashMap;

use newax_proto::nid_brand_sync::v1::nid_brand_sync_service_client::NidBrandSyncServiceClient;
use newax_proto::nid_brand_sync::v1::{
    Empty, JsonResponse, WebhookRequest, ListBrandsRequest, BrandsResponse, IdRequest,
    BrandConnectionSettings as ProtoBrandConnectionSettings, UpdateConnectionRequest,
    GetSyncLogRequest, SyncLogResponse, CreateRedirectRequest, RedirectCallbackRequest,
};

async fn client() -> Result<NidBrandSyncServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let endpoint = std::env::var("NID_BRAND_SYNC_GRPC_URL")
        .unwrap_or("http://127.0.0.1:50103".to_string());
    NidBrandSyncServiceClient::connect(endpoint)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("nid_brand_sync grpc: {}", e)))
}

pub async fn webhook_handler(
    Json(payload): Json<Value>,
) -> StatusCode {
    let mut c = match client().await {
        Ok(c) => c,
        Err(_) => return StatusCode::BAD_GATEWAY,
    };
    let json_str = serde_json::to_string(&payload).unwrap_or_default();
    let req = WebhookRequest { json: json_str };
    match c.webhook(tonic::Request::new(req)).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("nid_brand_sync webhook error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn list_brands_handler(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let tenant_id = params.get("tenant_id").cloned();
    let req = ListBrandsRequest { tenant_id };
    let resp = c.list_brands(req).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let inner = resp.into_inner();
    let brands: Vec<Value> = inner.brands.into_iter().map(|b| {
        serde_json::json!({
            "id": b.id,
            "nid_brand_id": b.nid_brand_id,
            "nid_org_id": b.nid_org_id,
            "nid_tenant_id": b.nid_tenant_id,
            "name": b.name,
            "slug": b.slug,
            "description": b.description,
            "bio": b.bio,
            "website": b.website,
            "email": b.email,
            "phone": b.phone,
            "primary_color": b.primary_color,
            "secondary_color": b.secondary_color,
            "active": b.active,
            "synced_at": b.synced_at,
            "created_at": b.created_at,
        })
    }).collect();
    Ok(Json(Value::Array(brands)))
}

pub async fn get_brand_handler(Path(id): Path<String>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_brand(IdRequest { id }).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let b = resp.into_inner();
    let val = serde_json::json!({
        "id": b.id,
        "nid_brand_id": b.nid_brand_id,
        "nid_org_id": b.nid_org_id,
        "nid_tenant_id": b.nid_tenant_id,
        "name": b.name,
        "slug": b.slug,
        "description": b.description,
        "bio": b.bio,
        "website": b.website,
        "email": b.email,
        "phone": b.phone,
        "primary_color": b.primary_color,
        "secondary_color": b.secondary_color,
        "active": b.active,
        "synced_at": b.synced_at,
        "created_at": b.created_at,
    });
    Ok(Json(val))
}

pub async fn get_connection_handler() -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_connection(Empty {}).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let s = resp.into_inner();
    let val = serde_json::json!({
        "webhook_url": s.webhook_url,
        "webhook_secret": s.webhook_secret,
        "polling_interval_sec": s.polling_interval_sec,
        "enabled": s.enabled,
        "health_status": s.health_status,
    });
    Ok(Json(val))
}

pub async fn update_connection_handler(Json(req_body): Json<Value>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = c.update_connection(UpdateConnectionRequest { json: json_str }).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let s = resp.into_inner();
    let val = serde_json::json!({
        "webhook_url": s.webhook_url,
        "webhook_secret": s.webhook_secret,
        "polling_interval_sec": s.polling_interval_sec,
        "enabled": s.enabled,
        "health_status": s.health_status,
    });
    Ok(Json(val))
}

pub async fn health_handler() -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.health_check(Empty {}).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_sync_log_handler(Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let limit = params.get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50);
    let resp = c.get_sync_log(GetSyncLogRequest { limit }).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let inner = resp.into_inner();
    let entries: Vec<Value> = inner.entries.into_iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "nid_brand_id": e.nid_brand_id,
            "event_type": e.event_type,
            "status": e.status,
            "error_msg": e.error_msg,
            "created_at": e.created_at,
        })
    }).collect();
    Ok(Json(Value::Array(entries)))
}

pub async fn create_redirect_handler(Json(req_body): Json<Value>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = c.create_redirect(CreateRedirectRequest { json: json_str }).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn redirect_callback_handler(Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let state = params.get("state")
        .map(|s| s.as_str())
        .unwrap_or("");
    let brand_id = params.get("brand_id")
        .map(|s| s.as_str())
        .unwrap_or("");
    let req = RedirectCallbackRequest {
        state: state.to_string(),
        brand_id: brand_id.to_string(),
    };
    let resp = c.redirect_callback(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}
