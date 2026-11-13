//! Real-time Notifications & WebSockets Routes — Phase 31
//!
//! API endpoints for notification management, preferences, settings, and broadcasts.

use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db::AppState;
use crate::services::notifications;

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
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<notifications::NotificationSummary>, (StatusCode, String)> {
    match notifications::get_notification_summary(&state).await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /notifications
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListNotificationsQuery>,
) -> Result<Json<Vec<notifications::NotificationItem>>, (StatusCode, String)> {
    match notifications::list_notifications(
        &state,
        params.user_id.as_deref(),
        params.notification_type.as_deref(),
        params.status.as_deref(),
        params.limit.unwrap_or(50),
    )
    .await
    {
        Ok(notifications) => Ok(Json(notifications)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// POST /notifications/queue
pub async fn queue(
    State(state): State<AppState>,
    Json(params): Json<QueueNotificationQuery>,
) -> Result<Json<notifications::NotificationResult>, (StatusCode, String)> {
    let data = params.data.unwrap_or_default();
    let channel = params.channel.unwrap_or_else(|| "push".to_string());

    match notifications::queue_notification(
        &state,
        &params.organization_id,
        params.user_id.as_deref(),
        &params.notification_type,
        &params.title,
        &params.body,
        &data,
        &channel,
    )
    .await
    {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// POST /notifications/broadcast
pub async fn broadcast(
    State(state): State<AppState>,
    Json(params): Json<BroadcastNotificationQuery>,
) -> Result<Json<notifications::BroadcastResult>, (StatusCode, String)> {
    match notifications::broadcast_notification(
        &state,
        &params.organization_id,
        &params.notification_type,
        &params.title,
        &params.body,
    )
    .await
    {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// PUT /notifications/{id}/read
pub async fn mark_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<notifications::NotificationResult>, (StatusCode, String)> {
    match notifications::mark_notification_read(&state, &id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// PUT /notifications/{id}/dismiss
pub async fn dismiss(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<notifications::NotificationResult>, (StatusCode, String)> {
    match notifications::dismiss_notification(&state, &id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /notifications/preferences/{user_id}
pub async fn get_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<notifications::NotificationPreferences>, (StatusCode, String)> {
    match notifications::get_preferences(&state, &user_id).await {
        Ok(prefs) => Ok(Json(prefs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// PUT /notifications/preferences/{user_id}
pub async fn update_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(prefs): Json<notifications::NotificationPreferences>,
) -> Result<Json<notifications::PreferencesResult>, (StatusCode, String)> {
    match notifications::update_preferences(&state, &user_id, &prefs).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /notifications/settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<notifications::NotificationSettings>, (StatusCode, String)> {
    match notifications::get_settings(&state).await {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// PUT /notifications/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<notifications::NotificationSettings>,
) -> Result<Json<notifications::SettingsResult>, (StatusCode, String)> {
    match notifications::update_settings(&state, &settings).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /notifications/connections
pub async fn list_connections(
    State(state): State<AppState>,
) -> Result<Json<Vec<notifications::WebSocketConnection>>, (StatusCode, String)> {
    match notifications::list_websocket_connections(&state).await {
        Ok(connections) => Ok(Json(connections)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// POST /notifications/cleanup
pub async fn cleanup(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match notifications::cleanup_old_notifications(&state).await {
        Ok(count) => Ok(Json(serde_json::json!({
            "deleted_count": count,
            "message": format!("Cleaned up {} old notifications", count)
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
