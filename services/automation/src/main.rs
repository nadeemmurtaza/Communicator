//! Automation standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use gen::automation::v1::automation_service_server::{AutomationService, AutomationServiceServer};
use gen::automation::v1::*;
use newax_automation::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct AutomationServer {
    state: AutomationState,
}

#[tonic::async_trait]
impl AutomationService for AutomationServer {
    async fn list_automation_rules(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let tenant = "".to_string();
        match list_automation_rules(&self.state, &tenant).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn create_automation_rule(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match create_automation_rule(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn update_automation_rule(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match update_automation_rule(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn delete_automation_rule(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match delete_automation_rule(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn test_automation_rule(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match test_automation_rule(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn apply_automation_rules(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match apply_automation_rules(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn get_automation_stats(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let tenant = "".to_string();
        match get_automation_stats(&self.state, &tenant).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn run_inactivity_nudges(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match run_inactivity_nudges(&self.state).await {
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
    let state = AutomationState::new(pool, http_client);
    let server = AutomationServer { state };

    let addr = "[::1]:50114".parse()?;
    info!("automation service listening on {}", addr);

    AutomationServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
