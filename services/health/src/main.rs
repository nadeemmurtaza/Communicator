mod gen {
    include!("gen/mod.rs");
}

use gen::health_service_server::HealthService;
use gen::health_service_server::HealthServiceServer;
use gen::{EmptyRequest, JsonResponse};

use newax_health::{HealthState, health_check};
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct HealthServer {
    state: HealthState,
}

#[tonic::async_trait]
impl HealthService for HealthServer {
    async fn check(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let result = health_check(&self.state).await;
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

    let state = HealthState::new(pool);
    let server = HealthServer { state };

    let addr = "[::1]:50110".parse()?;
    info!("Health service listening on {}", addr);

    HealthServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
