use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use std::collections::HashMap;

use newax_proto::nid_organization_sync::v1::nid_organization_sync_service_client::NidOrganizationSyncServiceClient;
use newax_proto::nid_organization_sync::v1::{Empty, IdRequest, WebhookRequest, UpdateConnectionRequest, GetSyncLogRequest, CreateRedirectRequest, RedirectCallbackRequest};

async fn client() -> Result<NidOrganizationSyncServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let endpoint = std::env::var("NID_ORGANIZATION_SYNC_GRPC_URL")
        .unwrap_or("http://127.0.0.1:50102".to_string());
    NidOrganizationSyncServiceClient::connect(endpoint)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("nid_organization_sync grpc: {}", e)))
}

pub async fn webhook_handler(
    Json(payload): Json<Value>,
) -> StatusCode {
    let mut c = match client().await {
        Ok(c) => c,
        Err(e) => return e.0,
    };
    let json_str = serde_json::to_string(&payload).unwrap_or_default();
    let req = WebhookRequest { json: json_str };
    match c.webhook(tonic::Request::new(req)).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("nid_organization_sync webhook error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn list_organizations_handler() -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.list_organizations(Empty {}).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_organization_handler(Path(id): Path<String>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_organization(IdRequest { id }).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_connection_handler() -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_connection(Empty {}).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_connection_handler(Json(req_body): Json<Value>) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client().await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    c.update_connection(UpdateConnectionRequest { json: json_str }).await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))
}

pub async fn get_health_handler() -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_health(Empty {}).await
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
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
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
    let state_token = params.get("state")
        .map(|s| s.as_str())
        .unwrap_or("");
    let org_id = params.get("organization_id")
        .map(|s| s.as_str())
        .unwrap_or("");
    let error = params.get("error")
        .map(|s| s.as_str())
        .unwrap_or("");
    let req = RedirectCallbackRequest {
        state: state_token.to_string(),
        organization_id: org_id.to_string(),
        error: error.to_string(),
    };
    let resp = c.redirect_callback(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let json: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}
