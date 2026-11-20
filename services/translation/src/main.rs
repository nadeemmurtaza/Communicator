mod gen {
    include!("gen/mod.rs");
}

use gen::translation_service_server::TranslationService;
use gen::translation_service_server::TranslationServiceServer;
use gen::{
    EmptyRequest, JsonResponse, SettingsRequest, DetectRequest, TranslateRequest,
    TranslateMessageRequest, PrepareOutboundRequest,
};

use newax_translation::{
    TranslationState, list_supported_languages, get_translation_settings,
    update_translation_settings, detect_language_endpoint, translate_text,
    translate_message, prepare_outbound,
};
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct TranslationServer {
    state: TranslationState,
}

#[tonic::async_trait]
impl TranslationService for TranslationServer {
    async fn list_supported_languages(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let result = list_supported_languages(&self.state);
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }

    async fn get_translation_settings(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let result = get_translation_settings(&self.state).await;
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }

    async fn update_translation_settings(
        &self,
        request: Request<SettingsRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let payload = request.into_inner();
        let value: serde_json::Value =
            serde_json::from_str(&payload.json).map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        let result = update_translation_settings(&self.state, &value).await;
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }

    async fn detect_language(
        &self,
        request: Request<DetectRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let payload = request.into_inner();
        let result = detect_language_endpoint(&payload.text);
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }

    async fn translate_text(
        &self,
        request: Request<TranslateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let payload = request.into_inner();
        let source = if payload.source_language.is_empty() {
            None
        } else {
            Some(payload.source_language.as_str())
        };
        let result = translate_text(&self.state, &payload.text, &payload.target_language, source).await;
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }

    async fn translate_message(
        &self,
        request: Request<TranslateMessageRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let payload = request.into_inner();
        let result = translate_message(&self.state, &payload.message_id, &payload.target_language).await;
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }

    async fn prepare_outbound(
        &self,
        request: Request<PrepareOutboundRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let payload = request.into_inner();
        let target = if payload.target_language.is_empty() {
            None
        } else {
            Some(payload.target_language.as_str())
        };
        let result = prepare_outbound(&self.state, &payload.conversation_id, &payload.text, target).await;
        Ok(tonic::Response::new(JsonResponse {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let http_client = reqwest::Client::new();
    let translation_api_url = std::env::var("TRANSLATION_API_URL").unwrap_or_default();
    let translation_api_key = std::env::var("TRANSLATION_API_KEY").unwrap_or_default();

    let state = TranslationState::new(pool, http_client, translation_api_url, translation_api_key);
    let server = TranslationServer { state };

    let addr = "[::1]:50111".parse()?;
    info!("Translation service listening on {}", addr);

    TranslationServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
