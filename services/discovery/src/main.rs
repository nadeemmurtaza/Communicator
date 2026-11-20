//! Discovery standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use newax_discovery::*;
use gen::discovery::v1::discovery_service_server::{DiscoveryService, DiscoveryServiceServer};
use gen::discovery::v1::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct DiscoveryServer {
    state: DiscoveryState,
}

#[tonic::async_trait]
impl DiscoveryService for DiscoveryServer {
    async fn list_discovery_templates(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let params = newax_discovery::DiscoveryQueryParams::default();
        match list_discovery_templates(&self.state, &params).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn list_inquiries(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let json_str = request.into_inner().json;
        let params = newax_discovery::parse_query_params(&json_str);
        match list_inquiries(&self.state, &params).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn get_inquiry(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let mut params = newax_discovery::DiscoveryQueryParams::default();
        params.tenant_key = inner.id.clone();
        match get_inquiry(&self.state, &params, &inner.id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn list_campaign_templates(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let json_str = request.into_inner().json;
        let params = newax_discovery::parse_query_params(&json_str);
        match list_campaign_templates(&self.state, &params).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn sync_campaign_template_status(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let json_str = request.into_inner().json;
        let params = newax_discovery::parse_query_params(&json_str);
        match sync_campaign_template_status(&self.state, &params).await {
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

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let http_client = reqwest::Client::new();
    let meta_token = std::env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default();
    let meta_version = std::env::var("META_WHATSAPP_API_VERSION").unwrap_or_default();

    let state = DiscoveryState::new(pool, http_client, meta_token, meta_version);
    let server = DiscoveryServer { state };

    let addr = "[::1]:50113".parse()?;
    info!("discovery service listening on {}", addr);

    DiscoveryServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
