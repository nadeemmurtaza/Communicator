//! Conversations routes — thin gRPC client forwarding to svc-conversations.

use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use newax_proto::conversations::v1::conversations_service_client::ConversationsServiceClient;
use newax_proto::conversations::v1::{
    EmptyRequest, GetConversationRequest, SendMessageRequest, ResolveConversationRequest,
    UpdatePriorityRequest, AssignTeamRequest, AssignAgentRequest, SnoozeConversationRequest,
    UpdateCustomAttributesRequest, AutoAssignRequest, ListContactNotesRequest,
    AddContactNoteRequest, EnrollSequenceRequest, SubmitFeedbackRequest,
};

fn grpc_url(state: &AppState) -> String {
    if state.config.conversations_grpc_url.is_empty() {
        "http://127.0.0.1:50120".to_string()
    } else {
        state.config.conversations_grpc_url.clone()
    }
}

async fn client(
    state: &AppState,
) -> Result<ConversationsServiceClient<tonic::transport::Channel>, Response> {
    ConversationsServiceClient::connect(grpc_url(state))
        .await
        .map_err(|e| {
            tracing::error!("conversations gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "detail": "conversations service unavailable" })),
            )
                .into_response()
        })
}

fn grpc_error(e: tonic::Status) -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({ "detail": e.to_string() })),
    )
        .into_response()
}

fn parse_json(resp: Result<impl tonic::IntoRequest<newax_proto::conversations::v1::JsonResponse>, impl std::fmt::Debug>) -> Response {
    match resp {
        Ok(_) => unreachable!(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "gRPC error" })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CollisionQuery {
    pub agent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct PresencePayload {
    pub agent_id: String,
    pub active: bool,
}

// ============================================================
// Conversations
// ============================================================

pub async fn list_conversations(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let resp = cli
        .list_conversations(EmptyRequest {})
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let resp = cli
        .get_conversation(GetConversationRequest { id })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn send_conversation_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .send_conversation_message(SendMessageRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn resolve_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .resolve_conversation(ResolveConversationRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn update_priority(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .update_priority(UpdatePriorityRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn assign_team(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .assign_team(AssignTeamRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn assign_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .assign_agent(AssignAgentRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn snooze_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .snooze_conversation(SnoozeConversationRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn update_custom_attributes(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .update_custom_attributes(UpdateCustomAttributesRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn auto_assign(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let resp = cli
        .auto_assign(AutoAssignRequest { id })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

// ============================================================
// Contact notes
// ============================================================

pub async fn list_contact_notes(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let resp = cli
        .list_contact_notes(ListContactNotesRequest { contact_id })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn add_contact_note(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .add_contact_note(AddContactNoteRequest { contact_id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

// ============================================================
// Sequences
// ============================================================

pub async fn enroll_sequence(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .enroll_sequence(EnrollSequenceRequest { id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

// ============================================================
// Presence
// ============================================================

pub async fn update_presence(
    Path(id): Path<String>,
    Json(payload): Json<PresencePayload>,
) -> Response {
    // Presence is ephemeral — no gRPC call needed
    let empty: Vec<serde_json::Value> = Vec::new();
    Json(empty).into_response()
}

pub async fn get_presence(_path: Path<String>) -> Response {
    let empty: Vec<serde_json::Value> = Vec::new();
    Json(empty).into_response()
}

pub async fn check_collision(
    Path(id): Path<String>,
    Query(_query): Query<CollisionQuery>,
) -> Response {
    // Collision is ephemeral — no gRPC call needed
    Json(json!({ "collision": false, "id": id })).into_response()
}

// ============================================================
// Feedback
// ============================================================

pub async fn submit_message_feedback(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let json_str = body.to_string();
    let resp = cli
        .submit_message_feedback(SubmitFeedbackRequest { message_id, json: json_str })
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}

pub async fn get_feedback_summary(State(state): State<AppState>) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let resp = cli
        .get_feedback_summary(EmptyRequest {})
        .await
        .map_err(grpc_error)?;
    let body = resp.into_inner().json;
    match serde_json::from_str(&body) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": "invalid JSON from service" })),
        )
            .into_response(),
    }
}
