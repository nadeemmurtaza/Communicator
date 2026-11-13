//! In-App Messaging & Proactive Triggers Routes — Phase 32

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::AppState;
use crate::services::in_app_messaging;

// ——— Request/Response Types ———

#[derive(Deserialize)]
pub struct CreateMessageRequest {
    pub organization_id: String,
    pub title: String,
    pub body: String,
    pub message_type: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_event: Option<String>,
    pub trigger_page: Option<String>,
    pub frequency: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMessageRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub message_type: Option<String>,
    pub status: Option<String>,
    pub button_text: Option<String>,
    pub button_url: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_event: Option<String>,
    pub frequency: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTriggerRequest {
    pub organization_id: String,
    pub name: String,
    pub trigger_type: String,
    pub condition: Option<String>,
    pub action: Option<String>,
    pub message_id: Option<String>,
    pub delay_seconds: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateTriggerRequest {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub condition: Option<String>,
    pub enabled: Option<bool>,
    pub message_id: Option<String>,
}

#[derive(Deserialize)]
pub struct RecordEventRequest {
    pub organization_id: String,
    pub contact_id: Option<String>,
    pub session_id: Option<String>,
    pub event_name: String,
    pub event_data: Option<String>,
    pub page_url: Option<String>,
}

#[derive(Deserialize)]
pub struct EvaluateTriggersRequest {
    pub organization_id: String,
    pub event_name: String,
    pub page_url: Option<String>,
}

#[derive(Deserialize)]
pub struct RecordMessageEventRequest {
    pub organization_id: String,
    pub message_id: String,
    pub contact_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: String,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub enabled: Option<bool>,
    pub max_concurrent_messages: Option<i32>,
    pub default_style: Option<String>,
    pub track_behavioral_events: Option<bool>,
    pub event_retention_days: Option<i32>,
}

#[derive(Deserialize)]
pub struct ListMessagesQuery {
    pub status: Option<String>,
    pub message_type: Option<String>,
}

// ——— Handlers ———

pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::get_summary(&state).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_message(
    State(state): State<AppState>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let message_type = req.message_type.unwrap_or_else(|| "popup".to_string());
    let trigger_type = req.trigger_type.unwrap_or_else(|| "event".to_string());
    let frequency = req.frequency.unwrap_or_else(|| "once_per_session".to_string());

    match in_app_messaging::create_message(
        &state,
        &req.organization_id,
        &req.title,
        &req.body,
        &message_type,
        &trigger_type,
        req.trigger_event.as_deref(),
        req.trigger_page.as_deref(),
        &frequency,
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "message_id": result.message_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_messages(
    State(state): State<AppState>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::list_messages(
        &state,
        query.status.as_deref(),
        query.message_type.as_deref(),
    )
    .await
    {
        Ok(messages) => Ok(Json(serde_json::to_value(messages).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::get_message(&state, &message_id).await {
        Ok(message) => Ok(Json(serde_json::to_value(message).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<UpdateMessageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::update_message(
        &state,
        &message_id,
        req.title.as_deref(),
        req.body.as_deref(),
        req.message_type.as_deref(),
        req.status.as_deref(),
        req.button_text.as_deref(),
        req.button_url.as_deref(),
        req.trigger_type.as_deref(),
        req.trigger_event.as_deref(),
        req.frequency.as_deref(),
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "message_id": result.message_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::delete_message(&state, &message_id).await {
        Ok(result) => Ok(Json(json!({
            "message_id": result.message_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_trigger(
    State(state): State<AppState>,
    Json(req): Json<CreateTriggerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let condition = req.condition.unwrap_or_else(|| "{}".to_string());
    let action = req.action.unwrap_or_else(|| "show_message".to_string());
    let delay_seconds = req.delay_seconds.unwrap_or(0);

    match in_app_messaging::create_trigger(
        &state,
        &req.organization_id,
        &req.name,
        &req.trigger_type,
        &condition,
        &action,
        req.message_id.as_deref(),
        delay_seconds,
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "trigger_id": result.trigger_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_triggers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::list_triggers(&state).await {
        Ok(triggers) => Ok(Json(serde_json::to_value(triggers).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_trigger(
    State(state): State<AppState>,
    Path(trigger_id): Path<String>,
    Json(req): Json<UpdateTriggerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::update_trigger(
        &state,
        &trigger_id,
        req.name.as_deref(),
        req.trigger_type.as_deref(),
        req.condition.as_deref(),
        req.enabled,
        req.message_id.as_deref(),
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "trigger_id": result.trigger_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_trigger(
    State(state): State<AppState>,
    Path(trigger_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::delete_trigger(&state, &trigger_id).await {
        Ok(result) => Ok(Json(json!({
            "trigger_id": result.trigger_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn record_behavioral_event(
    State(state): State<AppState>,
    Json(req): Json<RecordEventRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let event_data = req.event_data.unwrap_or_default();

    match in_app_messaging::record_behavioral_event(
        &state,
        &req.organization_id,
        req.contact_id.as_deref(),
        req.session_id.as_deref(),
        &req.event_name,
        &event_data,
        req.page_url.as_deref(),
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "event_id": result.event_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn evaluate_triggers(
    State(state): State<AppState>,
    Json(req): Json<EvaluateTriggersRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::evaluate_triggers(
        &state,
        &req.organization_id,
        &req.event_name,
        req.page_url.as_deref(),
    )
    .await
    {
        Ok(results) => Ok(Json(serde_json::to_value(results).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn record_message_event(
    State(state): State<AppState>,
    Json(req): Json<RecordMessageEventRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::record_message_event(
        &state,
        &req.organization_id,
        &req.message_id,
        req.contact_id.as_deref(),
        req.session_id.as_deref(),
        &req.event_type,
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "event_id": result.event_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::get_settings(&state).await {
        Ok(settings) => Ok(Json(serde_json::to_value(settings).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::update_settings(
        &state,
        req.enabled,
        req.max_concurrent_messages,
        req.default_style.as_deref(),
        req.track_behavioral_events,
        req.event_retention_days,
    )
    .await
    {
        Ok(result) => Ok(Json(json!({
            "settings_id": result.settings_id,
            "message": result.message,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_behavioral_events(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match in_app_messaging::list_behavioral_events(&state, None).await {
        Ok(events) => Ok(Json(serde_json::to_value(events).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
