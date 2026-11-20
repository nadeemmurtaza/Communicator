//! CSAT Surveys standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use newax_csat_surveys::*;
use gen::csat_surveys_service_server::{CsatSurveysService, CsatSurveysServiceServer};
use gen::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct CsatserviceServer {
    state: CSATState,
}

#[tonic::async_trait]
impl CsatSurveysService for CsatserviceServer {
    async fn list_templates(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let params = CSATQueryParams::default();
        match list_templates(&self.state, &params).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn get_template(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match get_template(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn create_template(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match create_template(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn update_template(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json).map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match update_template(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn delete_template(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match delete_template(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn list_surveys(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let params = CSATQueryParams::default();
        match list_surveys(&self.state, &params).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn create_survey(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match create_survey(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn respond_to_survey(
        &self,
        request: Request<RespondRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json).map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match respond_to_survey(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn send_survey(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match send_survey(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn get_csat_summary(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let params = CSATQueryParams::default();
        match get_csat_summary(&self.state, &params).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn generate_snapshot(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match generate_snapshot(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn list_snapshots(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match list_snapshots(&self.state).await {
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

    let state = CSATState::new(pool);
    let server = CsatserviceServer { state };

    let addr = "[::1]:50112".parse()?;
    info!("csat_surveys service listening on {}", addr);

    CsatSurveysServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}