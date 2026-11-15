//! Thin gRPC client forwarding topic categorization requests to `svc-topic-categorization` (port 50088).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::topic_categorization::v1::{
    topic_categorization_service_client::TopicCategorizationServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("TOPIC_CATEGORIZATION_GRPC").unwrap_or("http://127.0.0.1:50088")
}

async fn client() -> TopicCategorizationServiceClient<tonic::transport::Channel> {
    TopicCategorizationServiceClient::connect(svc_addr().to_string())
        .await
        .expect("topic-categorization gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn created_response(v: Value) -> Response {
    (StatusCode::CREATED, Json(v)).into_response()
}

fn err_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

#[derive(Deserialize)]
pub struct OverridePayload {
    pub topic: String,
    pub subtopic: Option<String>,
    pub overridden_by: Option<String>,
}

pub async fn get_topic_summary(State(_state): State<AppState>) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_conversation_topic(
    State(_state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Response {
    let mut req = newax_proto::topic_categorization::v1::GetConversationTopicRequest::default();
    req.conversation_id = conversation_id;
    match client().await.get_conversation_topic(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

pub async fn categorize_conversation(
    State(_state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Response {
    let mut req = newax_proto::topic_categorization::v1::CategorizeConversationRequest::default();
    req.conversation_id = conversation_id;
    match client().await.categorize_conversation(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn override_conversation_topic(
    State(_state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<OverridePayload>,
) -> Response {
    let overridden_by = payload.overridden_by.unwrap_or_else(|| "api_user".to_string());
    let mut req = newax_proto::topic_categorization::v1::OverrideTopicRequest::default();
    req.conversation_id = conversation_id;
    req.topic = payload.topic;
    req.subtopic = payload.subtopic.unwrap_or_default();
    req.overridden_by = overridden_by;
    match client().await.override_topic(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn batch_categorize(State(_state): State<AppState>) -> Response {
    match client().await.batch_categorize(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}
