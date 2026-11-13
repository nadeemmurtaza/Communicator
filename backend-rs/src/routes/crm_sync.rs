//! CRM Sync API routes (Phase 22).

use crate::db::AppState;
use crate::routes::auth::Tenant;
use crate::services::crm_sync;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn db_err(e: sqlx::Error) -> Response {
    err(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {e}"),
    )
}

// ——— Query params ———

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

// ——— Payloads ———

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

// ——— Routes ———

/// GET /crm-sync/summary
pub async fn get_crm_sync_summary(State(state): State<AppState>) -> impl IntoResponse {
    match crm_sync::get_crm_sync_summary(&state).await {
        Ok(summary) => Json(json!(summary)).into_response(),
        Err(e) => db_err(e),
    }
}

/// GET /crm-sync/providers
pub async fn get_crm_providers() -> impl IntoResponse {
    let providers = crm_sync::get_crm_providers();
    Json(json!(providers)).into_response()
}

/// GET /crm-sync/connections
pub async fn list_crm_connections(
    State(state): State<AppState>,
    Query(query): Query<ConnectionsQuery>,
) -> impl IntoResponse {
    match crm_sync::list_crm_connections(
        &state,
        query.provider.as_deref(),
        query.status.as_deref(),
    )
    .await
    {
        Ok(connections) => Json(json!(connections)).into_response(),
        Err(e) => db_err(e),
    }
}

/// POST /crm-sync/connections
pub async fn create_crm_connection(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(payload): Json<CreateConnectionPayload>,
) -> impl IntoResponse {
    let organization_id = payload.organization_id.unwrap_or_else(|| tenant_key.clone());
    let auth_type = payload.auth_type.unwrap_or_else(|| "oauth".to_string());
    let credentials = payload.credentials.and_then(|v| serde_json::to_string(&v).ok());
    let config = payload.config.and_then(|v| serde_json::to_string(&v).ok());

    match crm_sync::create_crm_connection(
        &state,
        &organization_id,
        &payload.provider,
        &auth_type,
        credentials.as_deref(),
        config.as_deref(),
    )
    .await
    {
        Ok(conn) => (StatusCode::CREATED, Json(json!(conn))).into_response(),
        Err(e) => db_err(e),
    }
}

/// PUT /crm-sync/connections/{id}
pub async fn update_crm_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateConnectionPayload>,
) -> impl IntoResponse {
    let credentials = payload.credentials.and_then(|v| serde_json::to_string(&v).ok());

    match crm_sync::update_crm_connection_status(
        &state,
        &id,
        &payload.status,
        credentials.as_deref(),
        payload.last_error.as_deref(),
    )
    .await
    {
        Ok(conn) => Json(json!(conn)).into_response(),
        Err(e) => db_err(e),
    }
}

/// DELETE /crm-sync/connections/{id}
pub async fn delete_crm_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match crm_sync::delete_crm_connection(&state, &id).await {
        Ok(()) => Json(json!({ "deleted": true })).into_response(),
        Err(e) => db_err(e),
    }
}

/// POST /crm-sync/connections/{id}/sync
pub async fn trigger_sync(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<TriggerSyncPayload>,
) -> impl IntoResponse {
    match crm_sync::trigger_sync(&state, &id, &payload.direction, &payload.object_type).await {
        Ok(result) => Json(json!(result)).into_response(),
        Err(sqlx::Error::RowNotFound) => err(
            StatusCode::BAD_REQUEST,
            "Connection is not in 'connected' status",
        ),
        Err(e) => db_err(e),
    }
}

/// GET /crm-sync/logs
pub async fn list_sync_logs(
    State(state): State<AppState>,
    Query(query): Query<SyncLogsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    match crm_sync::list_sync_logs(&state, query.connection_id.as_deref(), limit).await {
        Ok(logs) => Json(json!(logs)).into_response(),
        Err(e) => db_err(e),
    }
}

/// GET /crm-sync/mappings
pub async fn list_field_mappings(
    State(state): State<AppState>,
    Query(query): Query<FieldMappingsQuery>,
) -> impl IntoResponse {
    match crm_sync::list_field_mappings(
        &state,
        query.connection_id.as_deref(),
        query.object_type.as_deref(),
    )
    .await
    {
        Ok(mappings) => Json(json!(mappings)).into_response(),
        Err(e) => db_err(e),
    }
}

/// GET /crm-sync/refs
pub async fn get_external_refs(
    State(state): State<AppState>,
    Query(query): Query<ExternalRefsQuery>,
) -> impl IntoResponse {
    match crm_sync::get_external_refs(&state, &query.local_type, &query.local_id).await {
        Ok(refs) => Json(json!(refs)).into_response(),
        Err(e) => db_err(e),
    }
}

/// POST /crm-sync/refs
pub async fn create_external_ref(
    State(state): State<AppState>,
    Json(payload): Json<CreateExternalRefPayload>,
) -> impl IntoResponse {
    match crm_sync::create_external_ref(
        &state,
        &payload.connection_id,
        &payload.local_type,
        &payload.local_id,
        &payload.crm_object_type,
        &payload.crm_record_id,
        payload.crm_record_url.as_deref(),
    )
    .await
    {
        Ok(ref_) => (StatusCode::CREATED, Json(json!(ref_))).into_response(),
        Err(e) => db_err(e),
    }
}
