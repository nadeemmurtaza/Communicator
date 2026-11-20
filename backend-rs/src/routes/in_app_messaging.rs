//! Thin gRPC client forwarding in-app messaging requests to `svc-in-app-messaging` (port 50095).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::in_app_messaging::v1::{
    in_app_messaging_service_client::InAppMessagingServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("IN_APP_MESSAGING_GRPC").unwrap_or("http://127.0.0.1:50095")
}

async fn client() -> InAppMessagingServiceClient<tonic::transport::Channel> {
    InAppMessagingServiceClient::connect(svc_addr().to_string())
        .await
        .expect("in-app-messaging gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn err_response(message: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": message }))).into_response()
}

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

pub async fn get_summary(
    State(_state): State<AppState>,
) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_message(
    State(_state): State<AppState>,
    Json(req): Json<CreateMessageRequest>,
) -> Response {
    let message_type = req.message_type.unwrap_or_else(|| "popup".to_string());
    let trigger_type = req.trigger_type.unwrap_or_else(|| "event".to_string());
    let frequency = req.frequency.unwrap_or_else(|| "once_per_session".to_string());
    let mut r = newax_proto::in_app_messaging::v1::CreateMessageRequest::default();
    r.organization_id = req.organization_id;
    r.title = req.title;
    r.body = req.body;
    r.message_type = message_type;
    r.trigger_type = trigger_type;
    r.trigger_event = req.trigger_event.unwrap_or_default();
    r.trigger_page = req.trigger_page.unwrap_or_default();
    r.frequency = frequency;
    match client().await.create_message(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn list_messages(
    State(_state): State<AppState>,
    Query(query): Query<ListMessagesQuery>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::ListMessagesRequest::default();
    r.status = query.status.unwrap_or_default();
    r.message_type = query.message_type.unwrap_or_default();
    match client().await.list_messages(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_message(
    State(_state): State<AppState>,
    Path(message_id): Path<String>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::GetMessageRequest::default();
    r.message_id = message_id;
    match client().await.get_message(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_message(
    State(_state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<UpdateMessageRequest>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::UpdateMessageRequest::default();
    r.message_id = message_id;
    r.title = req.title.unwrap_or_default();
    r.body = req.body.unwrap_or_default();
    r.message_type = req.message_type.unwrap_or_default();
    r.status = req.status.unwrap_or_default();
    r.button_text = req.button_text.unwrap_or_default();
    r.button_url = req.button_url.unwrap_or_default();
    r.trigger_type = req.trigger_type.unwrap_or_default();
    r.trigger_event = req.trigger_event.unwrap_or_default();
    r.frequency = req.frequency.unwrap_or_default();
    match client().await.update_message(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn delete_message(
    State(_state): State<AppState>,
    Path(message_id): Path<String>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::DeleteMessageRequest::default();
    r.message_id = message_id;
    match client().await.delete_message(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_trigger(
    State(_state): State<AppState>,
    Json(req): Json<CreateTriggerRequest>,
) -> Response {
    let condition = req.condition.unwrap_or_else(|| "{}".to_string());
    let action = req.action.unwrap_or_else(|| "show_message".to_string());
    let delay_seconds = req.delay_seconds.unwrap_or(0);
    let mut r = newax_proto::in_app_messaging::v1::CreateTriggerRequest::default();
    r.organization_id = req.organization_id;
    r.name = req.name;
    r.trigger_type = req.trigger_type;
    r.condition = condition;
    r.action = action;
    r.message_id = req.message_id.unwrap_or_default();
    r.delay_seconds = delay_seconds;
    match client().await.create_trigger(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn list_triggers(
    State(_state): State<AppState>,
) -> Response {
    match client().await.list_triggers(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_trigger(
    State(_state): State<AppState>,
    Path(trigger_id): Path<String>,
    Json(req): Json<UpdateTriggerRequest>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::UpdateTriggerRequest::default();
    r.trigger_id = trigger_id;
    r.name = req.name.unwrap_or_default();
    r.trigger_type = req.trigger_type.unwrap_or_default();
    r.condition = req.condition.unwrap_or_default();
    r.enabled = req.enabled.map_or(String::new(), |b| b.to_string());
    r.message_id = req.message_id.unwrap_or_default();
    match client().await.update_trigger(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn delete_trigger(
    State(_state): State<AppState>,
    Path(trigger_id): Path<String>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::DeleteTriggerRequest::default();
    r.trigger_id = trigger_id;
    match client().await.delete_trigger(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn record_behavioral_event(
    State(_state): State<AppState>,
    Json(req): Json<RecordEventRequest>,
) -> Response {
    let event_data = req.event_data.unwrap_or_default();
    let mut r = newax_proto::in_app_messaging::v1::RecordBehavioralEventRequest::default();
    r.organization_id = req.organization_id;
    r.contact_id = req.contact_id.unwrap_or_default();
    r.session_id = req.session_id.unwrap_or_default();
    r.event_name = req.event_name;
    r.event_data = event_data;
    r.page_url = req.page_url.unwrap_or_default();
    match client().await.record_behavioral_event(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn evaluate_triggers(
    State(_state): State<AppState>,
    Json(req): Json<EvaluateTriggersRequest>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::EvaluateTriggersRequest::default();
    r.organization_id = req.organization_id;
    r.event_name = req.event_name;
    r.page_url = req.page_url.unwrap_or_default();
    match client().await.evaluate_triggers(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn record_message_event(
    State(_state): State<AppState>,
    Json(req): Json<RecordMessageEventRequest>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::RecordMessageEventRequest::default();
    r.organization_id = req.organization_id;
    r.message_id = req.message_id;
    r.contact_id = req.contact_id.unwrap_or_default();
    r.session_id = req.session_id.unwrap_or_default();
    r.event_type = req.event_type;
    match client().await.record_message_event(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_settings(
    State(_state): State<AppState>,
) -> Response {
    match client().await.get_settings(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Response {
    let mut r = newax_proto::in_app_messaging::v1::UpdateSettingsRequest::default();
    r.enabled = req.enabled.map_or(String::new(), |b| b.to_string());
    r.max_concurrent_messages = req.max_concurrent_messages.map_or(String::new(), |n| n.to_string());
    r.default_style = req.default_style.unwrap_or_default();
    r.track_behavioral_events = req.track_behavioral_events.map_or(String::new(), |b| b.to_string());
    r.event_retention_days = req.event_retention_days.map_or(String::new(), |n| n.to_string());
    match client().await.update_settings(r).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn list_behavioral_events(
    State(_state): State<AppState>,
) -> Response {
    match client().await.list_behavioral_events(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}
