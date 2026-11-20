//! Live Proof standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use gen::live_proof::v1::live_proof_service_server::{LiveProofService, LiveProofServiceServer};
use gen::live_proof::v1::*;
use newax_live_proof::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct LiveProofServer {
    state: LiveProofState,
}

#[tonic::async_trait]
impl LiveProofService for LiveProofServer {
    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let public_url = std::env::var("PUBLIC_URL").unwrap_or_default();
        match create_session(&self.state, &body_str, &public_url).await {
            Ok((_status, val)) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err((_status, e)) => Err(tonic::Status::invalid_argument(e)),
        }
    }

    async fn get_session_details(
        &self,
        request: Request<GetSessionDetailsRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match get_session_details(&self.state, &inner.session_id, &inner.token).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err((_status, e)) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn record_decision(
        &self,
        request: Request<RecordDecisionRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match record_decision(&self.state, &inner.session_id, &inner.token, &inner.json).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err((_status, e)) => Err(tonic::Status::invalid_argument(e)),
        }
    }

    async fn upload_evidence(
        &self,
        request: Request<UploadEvidenceRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match upload_evidence(&self.state, &inner.session_id, &inner.token, &inner.json).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err((_status, e)) => Err(tonic::Status::invalid_argument(e)),
        }
    }

    async fn get_evidence_media(
        &self,
        request: Request<GetEvidenceMediaRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match get_evidence_media(&self.state, &inner.session_id, &inner.evidence_id, &inner.token).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err((_status, e)) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn get_capture_page(
        &self,
        request: Request<GetCapturePageRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match get_capture_page(&self.state, &inner.session_id, &inner.token).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err((_status, e)) => Err(tonic::Status::not_found(e)),
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

    let public_url = std::env::var("PUBLIC_URL").unwrap_or_default();
    let live_proof_storage_dir = std::env::var("LIVE_PROOF_STORAGE_DIR")
        .unwrap_or_else(|_| "./live_proof_storage".to_string());

    let state = LiveProofState {
        db: pool,
        public_url,
        live_proof_storage_dir,
    };
    let server = LiveProofServer { state };

    let addr = "[::1]:50118".parse()?;
    info!("live-proof service listening on {}", addr);

    LiveProofServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
