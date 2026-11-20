//! Business Profile standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use gen::business_profile::v1::business_profile_service_server::{BusinessProfileService, BusinessProfileServiceServer};
use gen::business_profile::v1::*;
use newax_business_profile::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct BusinessProfileServer {
    state: BusinessProfileState,
}

#[tonic::async_trait]
impl BusinessProfileService for BusinessProfileServer {
    async fn get_business_profile(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match get_business_profile(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn update_business_profile(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match update_business_profile(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn sync_meta(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match sync_meta(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let meta_whatsapp_access_token = std::env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default();
    let meta_whatsapp_phone_number_id = std::env::var("META_WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default();
    let meta_whatsapp_api_version = std::env::var("META_WHATSAPP_API_VERSION").unwrap_or_else(|_| "v18.0".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let http_client = reqwest::Client::new();
    let state = BusinessProfileState {
        db: pool,
        http_client,
        meta_whatsapp_access_token,
        meta_whatsapp_phone_number_id,
        meta_whatsapp_api_version,
    };
    let server = BusinessProfileServer { state };

    let addr = "[::1]:50115".parse()?;
    info!("business_profile service listening on {}", addr);

    BusinessProfileServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
