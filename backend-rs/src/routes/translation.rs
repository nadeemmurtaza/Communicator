//! Translation API routes — thin gRPC client forwarding to svc-translation.

use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use newax_proto::translation::v1::translation_service_client::TranslationServiceClient;
use newax_proto::translation::v1::{
    EmptyRequest, SettingsRequest, DetectRequest, TranslateRequest,
    TranslateMessageRequest, PrepareOutboundRequest,
};

async fn client(
    state: &AppState,
) -> Result<TranslationServiceClient<tonic::transport::Channel>, (StatusCode, serde_json::Value)> {
    let url = if state.config.translation_grpc_url.is_empty() {
        "http://127.0.0.1:50111".to_string()
    } else {
        state.config.translation_grpc_url.clone()
    };
    TranslationServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("translation gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                serde_json::json!({ "error": "translation service unavailable" }),
            )
        })
}

#[derive(Deserialize)]
pub struct TranslatePayload {
    text: String,
    #[serde(alias = "targetLanguage")]
    target_language: String,
    #[serde(alias = "sourceLanguage")]
    source_language: Option<String>,
}

#[derive(Deserialize)]
pub struct DetectPayload {
    text: String,
}

#[derive(Deserialize)]
pub struct OutboundPayload {
    text: String,
    #[serde(alias = "targetLanguage")]
    target_language: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SettingsUpdate {
    is_enabled: Option<bool>,
    auto_translate_inbound: Option<bool>,
    auto_translate_outbound: Option<bool>,
    agent_primary_language: Option<String>,
    preferred_engine: Option<String>,
    confidence_threshold: Option<f64>,
    enabled_languages: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct TranslateQuery {
    #[serde(alias = "targetLanguage")]
    target_language: Option<String>,
}

pub async fn list_supported_languages(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, Json(json)).into_response(),
    };
    let resp = cli.list_supported_languages(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_translation_settings(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, Json(json)).into_response(),
    };
    let resp = cli.get_translation_settings(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_translation_settings(
    State(state): State<AppState>,
    Json(payload): Json<SettingsUpdate>,
) -> Response {
    let json_str = serde_json::to_string(&payload).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, Json(json)).into_response(),
    };
    let resp = cli.update_translation_settings(SettingsRequest { json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn detect_language_endpoint(
    Json(payload): Json<DetectPayload>,
) -> Json<Value> {
    Json(serde_json::json!({
        "language": "unavailable",
        "detail": "detection is now handled server-side; use the gRPC endpoint directly",
        "text": payload.text,
    }))
}

pub async fn translate_text(
    State(state): State<AppState>,
    Json(payload): Json<TranslatePayload>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, Json(json)).into_response(),
    };
    let resp = cli.translate_text(TranslateRequest {
        text: payload.text,
        target_language: payload.target_language,
        source_language: payload.source_language.unwrap_or_default(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn translate_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(params): Query<TranslateQuery>,
) -> Response {
    let target = params.target_language.unwrap_or_else(|| "en".to_string());
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, Json(json)).into_response(),
    };
    let resp = cli.translate_message(TranslateMessageRequest {
        message_id,
        target_language: target,
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn prepare_outbound(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<OutboundPayload>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, Json(json)).into_response(),
    };
    let resp = cli.prepare_outbound(PrepareOutboundRequest {
        conversation_id,
        text: payload.text,
        target_language: payload.target_language.unwrap_or_default(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}