//! Services & Sequences standalone gRPC service

use newax_services_sequences as services_sequences;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.services_sequences.v1");
}

use proto::services_sequences_service_server::{ServicesSequencesService as ServicesSequencesTrait, ServicesSequencesServiceServer};
use proto::{JsonResponse, JsonBody, Empty, IdRequest, ListServicesRequest, CreateServiceRequest, CreateSequenceRequest};
use serde_json::json;

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn ok_json(value: serde_json::Value) -> Result<Response<JsonResponse>, Status> {
    Ok(Response::new(JsonResponse {
        json: serde_json::to_string(&value).unwrap_or_default(),
    }))
}

fn parse_body(msg: Option<JsonBody>) -> serde_json::Value {
    match msg {
        Some(b) => serde_json::from_str(&b.json).unwrap_or(json!({})),
        None => json!({}),
    }
}

#[tonic::async_trait]
impl ServicesSequencesTrait for Svc {
    async fn list_services(
        &self,
        request: Request<ListServicesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let state = services_sequences::ServicesSequencesState::new(self.pool.clone());
        match services_sequences::list_services(&state, &msg.tenant_key).await {
            Ok(resp) => ok_json(resp),
            Err(e) => Err(to_status(e)),
        }
    }

    async fn get_service(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let state = services_sequences::ServicesSequencesState::new(self.pool.clone());
        match services_sequences::get_service(&state, &msg.id).await {
            Ok(resp) => ok_json(resp),
            Err(e) => Err(to_status(e)),
        }
    }

    async fn create_service(
        &self,
        request: Request<CreateServiceRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let body = parse_body(msg.body);
        let payload: services_sequences::CreateServicePayload =
            serde_json::from_value(body.clone()).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let state = services_sequences::ServicesSequencesState::new(self.pool.clone());
        match services_sequences::create_service(&state, &payload, &msg.tenant_key).await {
            Ok(resp) => ok_json(resp),
            Err(e) => Err(to_status(e)),
        }
    }

    async fn list_sequences(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let state = services_sequences::ServicesSequencesState::new(self.pool.clone());
        match services_sequences::list_sequences(&state).await {
            Ok(resp) => ok_json(resp),
            Err(e) => Err(to_status(e)),
        }
    }

    async fn create_sequence(
        &self,
        request: Request<CreateSequenceRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let body = parse_body(msg.body);
        let payload: services_sequences::CreateSequencePayload =
            serde_json::from_value(body.clone()).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let state = services_sequences::ServicesSequencesState::new(self.pool.clone());
        match services_sequences::create_sequence(&state, &payload).await {
            Ok(resp) => ok_json(resp),
            Err(e) => Err(to_status(e)),
        }
    }

    async fn sequence_variant_stats(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let state = services_sequences::ServicesSequencesState::new(self.pool.clone());
        match services_sequences::sequence_variant_stats(&state, &msg.id).await {
            Ok(resp) => ok_json(resp),
            Err(e) => Err(to_status(e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "svc-services-sequences=debug,tonic=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let port: u16 = std::env::var("SERVICES_SEQUENCES_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50107);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!("Services & Sequences service listening on {}", addr);

    Server::builder()
        .add_service(ServicesSequencesServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;

    Ok(())
}
