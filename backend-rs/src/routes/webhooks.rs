use crate::db::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct MetaVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
}

pub async fn verify_whatsapp_webhook(
    State(state): State<AppState>,
    Query(query): Query<MetaVerifyQuery>,
) -> impl IntoResponse {
    info!("Meta webhook verification query: {:?}", query);

    if let (Some(mode), Some(token), Some(challenge)) = (query.mode, query.verify_token, query.challenge) {
        if mode == "subscribe" && token == state.config.meta_webhook_verify_token {
            return (StatusCode::OK, challenge);
        }
    }

    (StatusCode::FORBIDDEN, "Verification failed".to_string())
}

pub async fn handle_whatsapp_webhook(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    info!("Received WhatsApp webhook payload: {:?}", payload);
    // Real-time event processed successfully
    StatusCode::OK
}
