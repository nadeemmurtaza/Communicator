//! Thin gRPC client forwarding canned responses, macros, and SLA policy requests
//! to `svc-canned-macros-sla` (port 50086).

use crate::db::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use newax_proto::canned_macros_sla::v1::{
    canned_macros_sla_service_client::CannedMacrosSlaServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("CANNED_MACROS_SLA_GRPC").unwrap_or("http://127.0.0.1:50086")
}

async fn client() -> CannedMacrosSlaServiceClient<tonic::transport::Channel> {
    CannedMacrosSlaServiceClient::connect(svc_addr().to_string())
        .await
        .expect("canned-macros-sla gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn err_response(message: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": message }))).into_response()
}

// ── Query/Payload types ──

#[derive(Deserialize)]
pub struct CannedQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCannedPayload {
    pub short_code: String,
    pub content: String,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCannedPayload {
    pub short_code: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct ExpandCannedPayload {
    pub short_code: String,
    pub template: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateMacroPayload {
    pub name: String,
    pub description: Option<String>,
    pub actions: Option<Value>,
}

#[derive(Deserialize)]
pub struct UpdateMacroPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub actions: Option<Value>,
}

#[derive(Deserialize)]
pub struct CreateSlaPolicyPayload {
    pub name: String,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i64>,
    pub resolution_time_threshold: Option<i64>,
    pub only_during_business_hours: Option<bool>,
    pub priorities: Option<Vec<String>>,
    pub inbox_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UpdateSlaPolicyPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i64>,
    pub resolution_time_threshold: Option<i64>,
    pub only_during_business_hours: Option<bool>,
    pub priorities: Option<Vec<String>>,
    pub inbox_ids: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct EvaluateSlaQuery {
    pub policy_id: Option<String>,
}

// ── Canned Responses ──

pub async fn list_canned_responses(
    State(_state): State<AppState>,
    Query(query): Query<CannedQuery>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::ListCannedResponsesRequest::default();
    req.q = query.q.clone().unwrap_or_default();
    req.category = query.category.clone().unwrap_or_default();
    match client().await.list_canned_responses(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_canned_response(
    State(_state): State<AppState>,
    Json(payload): Json<CreateCannedPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::CreateCannedResponseRequest::default();
    req.short_code = payload.short_code;
    req.content = payload.content;
    req.category = payload.category.unwrap_or_default();
    match client().await.create_canned_response(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_canned_response(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCannedPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::UpdateCannedResponseRequest::default();
    req.id = id;
    req.short_code = payload.short_code.unwrap_or_default();
    req.content = payload.content.unwrap_or_default();
    req.category = payload.category.unwrap_or_default();
    match client().await.update_canned_response(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn delete_canned_response(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::DeleteCannedResponseRequest::default();
    req.id = id;
    match client().await.delete_canned_response(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn expand_canned_response(
    State(_state): State<AppState>,
    Json(payload): Json<ExpandCannedPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::ExpandCannedResponseRequest::default();
    req.short_code = payload.short_code;
    req.template = payload.template.unwrap_or_default();
    match client().await.expand_canned_response(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

// ── Macros ──

pub async fn list_macros(State(_state): State<AppState>) -> Response {
    match client().await.list_macros(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_macro(
    State(_state): State<AppState>,
    Json(payload): Json<CreateMacroPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::CreateMacroRequest::default();
    req.name = payload.name;
    req.description = payload.description.unwrap_or_default();
    req.actions = serde_json::to_string(&payload.actions).unwrap_or_else(|_| "[]".to_string());
    match client().await.create_macro(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_macro(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::GetMacroRequest::default();
    req.id = id;
    match client().await.get_macro(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_macro(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateMacroPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::UpdateMacroRequest::default();
    req.id = id;
    req.name = payload.name.unwrap_or_default();
    req.description = payload.description.unwrap_or_default();
    req.actions = serde_json::to_string(&payload.actions).unwrap_or_else(|_| "[]".to_string());
    match client().await.update_macro(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn delete_macro(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::DeleteMacroRequest::default();
    req.id = id;
    match client().await.delete_macro(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn execute_macro(
    State(_state): State<AppState>,
    Path((conversation_id, macro_id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Response {
    let agent_name = payload.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    let mut req = newax_proto::canned_macros_sla::v1::ExecuteMacroRequest::default();
    req.conversation_id = conversation_id;
    req.macro_id = macro_id;
    req.agent_name = agent_name.to_string();
    match client().await.execute_macro(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn execute_macro_direct(
    State(_state): State<AppState>,
    Path(macro_id): Path<String>,
    Json(payload): Json<Value>,
) -> Response {
    let conversation_id = payload.get("conversation_id").and_then(|v| v.as_str()).unwrap_or("");
    let agent_name = payload.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    let mut req = newax_proto::canned_macros_sla::v1::ExecuteMacroDirectRequest::default();
    req.macro_id = macro_id;
    req.conversation_id = conversation_id.to_string();
    req.agent_name = agent_name.to_string();
    match client().await.execute_macro_direct(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

// ── WhatsApp Dispatch ──

/// Real Meta Cloud API text dispatch via gRPC. Reports the provider's own error text.
pub async fn dispatch_whatsapp_text(
    _state: &AppState,
    phone: &str,
    body: &str,
    local_msg_id: &str,
) -> Value {
    let mut req = newax_proto::canned_macros_sla::v1::DispatchWhatsAppTextRequest::default();
    req.phone = phone.to_string();
    req.body = body.to_string();
    req.local_msg_id = local_msg_id.to_string();
    match client().await.dispatch_whats_app_text(req).await {
        Ok(resp) => grpc_json(resp.into_inner().json),
        Err(e) => json!({ "dispatched": false, "transportError": e.to_string() }),
    }
}

// ── SLA Policies ──

pub async fn list_sla_policies(State(_state): State<AppState>) -> Response {
    match client().await.list_sla_policies(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_sla_policy(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::GetSlaPolicyRequest::default();
    req.id = id;
    match client().await.get_sla_policy(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn create_sla_policy(
    State(_state): State<AppState>,
    Json(payload): Json<CreateSlaPolicyPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::CreateSlaPolicyRequest::default();
    req.name = payload.name;
    req.description = payload.description.unwrap_or_default();
    req.first_response_time_threshold = payload.first_response_time_threshold.unwrap_or(60) as i32;
    req.resolution_time_threshold = payload.resolution_time_threshold.unwrap_or(1440) as i32;
    req.only_during_business_hours = payload.only_during_business_hours.unwrap_or(false);
    req.priorities = payload.priorities.unwrap_or_default();
    req.inbox_ids = payload.inbox_ids.unwrap_or_default();
    match client().await.create_sla_policy(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn update_sla_policy(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSlaPolicyPayload>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::UpdateSlaPolicyRequest::default();
    req.id = id;
    req.has_name = payload.name.is_some();
    req.name = payload.name.unwrap_or_default();
    req.has_description = payload.description.is_some();
    req.description = payload.description.unwrap_or_default();
    req.has_first_response_time_threshold = payload.first_response_time_threshold.is_some();
    req.first_response_time_threshold = payload.first_response_time_threshold.unwrap_or(0) as i32;
    req.has_resolution_time_threshold = payload.resolution_time_threshold.is_some();
    req.resolution_time_threshold = payload.resolution_time_threshold.unwrap_or(0) as i32;
    req.has_only_during_business_hours = payload.only_during_business_hours.is_some();
    req.only_during_business_hours = payload.only_during_business_hours.unwrap_or(false);
    req.has_priorities = payload.priorities.is_some();
    req.priorities = payload.priorities.unwrap_or_default();
    req.has_inbox_ids = payload.inbox_ids.is_some();
    req.inbox_ids = payload.inbox_ids.unwrap_or_default();
    req.has_is_active = payload.is_active.is_some();
    req.is_active = payload.is_active.unwrap_or(false);
    match client().await.update_sla_policy(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn delete_sla_policy(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::DeleteSlaPolicyRequest::default();
    req.id = id;
    match client().await.delete_sla_policy(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn evaluate_conversation_sla(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EvaluateSlaQuery>,
) -> Response {
    let mut req = newax_proto::canned_macros_sla::v1::EvaluateConversationSlaRequest::default();
    req.id = id;
    req.policy_id = query.policy_id.clone().unwrap_or_default();
    match client().await.evaluate_conversation_sla(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_sla_metrics(State(_state): State<AppState>) -> Response {
    match client().await.get_sla_metrics(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}

pub async fn get_sla_breaches(State(_state): State<AppState>) -> Response {
    match client().await.get_sla_breaches(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(&e.to_string()),
    }
}
