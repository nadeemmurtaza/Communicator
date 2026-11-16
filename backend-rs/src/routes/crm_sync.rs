//! CRM Sync API — Phase 22: CRM bidirectional sync.
//!
//! Thin gRPC client forwarding to svc-crm-sync; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::AppState;

use newax_proto::crm_sync::v1::{
    crm_sync_service_client::CrmSyncServiceClient,
    Empty, IdRequest, JsonResponse, ListConnectionsRequest,
    CreateConnectionRequest, UpdateConnectionRequest, TriggerSyncRequest,
    ListSyncLogsRequest, ListFieldMappingsRequest, GetExternalRefsRequest,
    CreateExternalRefRequest,
};

async fn client() -> Result<CrmSyncServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("CRM_SYNC_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50084".to_string());
    CrmSyncServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("crm-sync service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

fn bad_request_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.message().to_string())
}

fn parse_json(s: &str) -> Result<serde_json::Value, (StatusCode, String)> {
    serde_json::from_str(s).map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid json: {e}")))
}

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ConnectionsQuery {
    pub provider: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct SyncLogsQuery {
    pub connection_id: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct FieldMappingsQuery {
    pub connection_id: Option<String>,
    pub object_type: Option<String>,
}

#[derive(Deserialize)]
pub struct ExternalRefsQuery {
    pub local_type: String,
    pub local_id: String,
}

// ---------------------------------------------------------------------------
// Payloads
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateConnectionPayload {
    pub organization_id: Option<String>,
    pub provider: String,
    pub auth_type: Option<String>,
    pub credentials: Option<Value>,
    pub config: Option<Value>,
}

#[derive(Deserialize)]
pub struct UpdateConnectionPayload {
    pub status: String,
    pub credentials: Option<Value>,
    pub last_error: Option<String>,
}

#[derive(Deserialize)]
pub struct TriggerSyncPayload {
    pub direction: String,
    pub object_type: String,
}

#[derive(Deserialize)]
pub struct CreateExternalRefPayload {
    pub connection_id: String,
    pub local_type: String,
    pub local_id: String,
    pub crm_object_type: String,
    pub crm_record_id: String,
    pub crm_record_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

pub async fn get_crm_sync_summary(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    let summary = parse_json(&resp.json)?;
    Ok(Json(summary))
}

pub async fn get_crm_providers(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_providers(Empty {}).await.map_err(grpc_err)?.into_inner();
    let providers = parse_json(&resp.json)?;
    Ok(Json(providers))
}

pub async fn list_crm_connections(
    State(_state): State<AppState>,
    Query(query): Query<ConnectionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_connections(ListConnectionsRequest {
            provider: query.provider,
            status: query.status,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let connections = parse_json(&resp.json)?;
    Ok(Json(connections))
}

pub async fn create_crm_connection(
    State(_state): State<AppState>,
    Json(payload): Json<CreateConnectionPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let organization_id = payload.organization_id.unwrap_or_else(|| "default".to_string());
    let auth_type = payload.auth_type.unwrap_or_else(|| "oauth".to_string());
    let credentials = payload.credentials.and_then(|v| serde_json::to_string(&v).ok());
    let config = payload.config.and_then(|v| serde_json::to_string(&v).ok());

    let mut c = client().await?;
    let resp = c
        .create_connection(CreateConnectionRequest {
            organization_id,
            provider: payload.provider,
            auth_type,
            credentials,
            config,
        })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    let conn = parse_json(&resp.json)?;
    Ok((StatusCode::CREATED, Json(conn)))
}

pub async fn update_crm_connection(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateConnectionPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let credentials = payload.credentials.and_then(|v| serde_json::to_string(&v).ok());

    let mut c = client().await?;
    let resp = c
        .update_connection(UpdateConnectionRequest {
            id,
            status: payload.status,
            credentials,
            last_error: payload.last_error,
        })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    let conn = parse_json(&resp.json)?;
    Ok(Json(conn))
}

pub async fn delete_crm_connection(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    c.delete_connection(IdRequest { id })
        .await
        .map_err(grpc_err)?;
    Ok(Json(json!({ "deleted": true })))
}

pub async fn trigger_sync(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<TriggerSyncPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .trigger_sync(TriggerSyncRequest {
            connection_id: id,
            direction: payload.direction,
            object_type: payload.object_type,
        })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    let result = parse_json(&resp.json)?;
    Ok(Json(result))
}

pub async fn list_sync_logs(
    State(_state): State<AppState>,
    Query(query): Query<SyncLogsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50);
    let mut c = client().await?;
    let resp = c
        .list_sync_logs(ListSyncLogsRequest {
            connection_id: query.connection_id,
            limit,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let logs = parse_json(&resp.json)?;
    Ok(Json(logs))
}

pub async fn list_field_mappings(
    State(_state): State<AppState>,
    Query(query): Query<FieldMappingsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_field_mappings(ListFieldMappingsRequest {
            connection_id: query.connection_id,
            object_type: query.object_type,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let mappings = parse_json(&resp.json)?;
    Ok(Json(mappings))
}

pub async fn get_external_refs(
    State(_state): State<AppState>,
    Query(query): Query<ExternalRefsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .get_external_refs(GetExternalRefsRequest {
            local_type: query.local_type,
            local_id: query.local_id,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let refs = parse_json(&resp.json)?;
    Ok(Json(refs))
}

pub async fn create_external_ref(
    State(_state): State<AppState>,
    Json(payload): Json<CreateExternalRefPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .create_external_ref(CreateExternalRefRequest {
            connection_id: payload.connection_id,
            local_type: payload.local_type,
            local_id: payload.local_id,
            crm_object_type: payload.crm_object_type,
            crm_record_id: payload.crm_record_id,
            crm_record_url: payload.crm_record_url,
        })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    let ref_ = parse_json(&resp.json)?;
    Ok((StatusCode::CREATED, Json(ref_)))
}
