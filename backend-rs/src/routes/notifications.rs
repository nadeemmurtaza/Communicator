//! Real-time Notifications & WebSockets Routes — Phase 31
//!
//! Thin gRPC client forwarding to svc-notifications; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;

use newax_proto::notifications::v1::{
    notifications_service_client::NotificationsServiceClient, Empty, IdRequest,
    ListNotificationsRequest, QueueNotificationRequest, BroadcastNotificationRequest,
    UpdatePreferencesRequest, UpdateSettingsRequest,
};

async fn client() -> Result<NotificationsServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("NOTIFICATIONS_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50082".to_string());
    NotificationsServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("notifications service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

// ——— Query Params ———

#[derive(Deserialize)]
pub struct ListNotificationsQuery {
    pub user_id: Option<String>,
    pub notification_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct QueueNotificationQuery {
    pub organization_id: String,
    pub user_id: Option<String>,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub data: Option<String>,
    pub channel: Option<String>,
}

#[derive(Deserialize)]
pub struct BroadcastNotificationQuery {
    pub organization_id: String,
    pub notification_type: String,
    pub title: String,
    pub body: String,
}

// ——— Handlers ———

/// GET /notifications/summary
pub async fn get_summary(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_summary(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// GET /notifications
pub async fn list(
    State(_state): State<AppState>,
    Query(params): Query<ListNotificationsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.list_notifications(ListNotificationsRequest {
        user_id: params.user_id,
        notification_type: params.notification_type,
        status: params.status,
        limit: params.limit.unwrap_or(50),
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// POST /notifications/queue
pub async fn queue(
    State(_state): State<AppState>,
    Json(params): Json<QueueNotificationQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let data = params.data.unwrap_or_default();
    let channel = params.channel.unwrap_or_else(|| "push".to_string());
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.queue_notification(QueueNotificationRequest {
        organization_id: params.organization_id,
        user_id: params.user_id,
        notification_type: params.notification_type,
        title: params.title,
        body: params.body,
        data,
        channel,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// POST /notifications/broadcast
pub async fn broadcast(
    State(_state): State<AppState>,
    Json(params): Json<BroadcastNotificationQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.broadcast_notification(BroadcastNotificationRequest {
        organization_id: params.organization_id,
        notification_type: params.notification_type,
        title: params.title,
        body: params.body,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// PUT /notifications/{id}/read
pub async fn mark_read(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.mark_read(IdRequest { id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// PUT /notifications/{id}/dismiss
pub async fn dismiss(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.dismiss(IdRequest { id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// GET /notifications/preferences/{user_id}
pub async fn get_preferences(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_preferences(IdRequest { id: user_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// PUT /notifications/preferences/{user_id}
pub async fn update_preferences(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(prefs): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let prefs_json = serde_json::to_string(&prefs).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.update_preferences(UpdatePreferencesRequest {
        user_id,
        prefs_json,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// GET /notifications/settings
pub async fn get_settings(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_settings(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// PUT /notifications/settings
pub async fn update_settings(
    State(_state): State<AppState>,
    Json(settings): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings_json = serde_json::to_string(&settings).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.update_settings(UpdateSettingsRequest {
        settings_json,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// GET /notifications/connections
pub async fn list_connections(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.list_connections(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

/// POST /notifications/cleanup
pub async fn cleanup(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.cleanup(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}
