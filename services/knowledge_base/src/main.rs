//! Knowledge Base standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use gen::knowledge_base::v1::knowledge_base_service_server::{KnowledgeBaseService, KnowledgeBaseServiceServer};
use gen::knowledge_base::v1::*;
use newax_knowledge_base::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct KnowledgeBaseServer {
    state: KnowledgeBaseState,
}

#[tonic::async_trait]
impl KnowledgeBaseService for KnowledgeBaseServer {
    async fn list_knowledge_base(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let query_json = request.into_inner().json;
        match list_knowledge_base(&self.state, &query_json).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn create_knowledge_base_entry(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match create_knowledge_base_entry(&self.state, &body).await {
            Ok((_status, val)) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn delete_knowledge_base_entry(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match delete_knowledge_base_entry(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn upload_knowledge_document(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match upload_knowledge_document(&self.state, &body).await {
            Ok((_status, val)) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn ingest_url(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match ingest_url(&self.state, &body).await {
            Ok((_status, val)) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn get_knowledge_hub_overview(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let query_json = request.into_inner().json;
        match get_knowledge_hub_overview(&self.state, &query_json).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn list_knowledge_recommendations(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let query_json = request.into_inner().json;
        match list_knowledge_recommendations(&self.state, &query_json).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn apply_knowledge_recommendation(
        &self,
        request: Request<ApplyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match apply_knowledge_recommendation(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn dismiss_knowledge_recommendation(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match dismiss_knowledge_recommendation(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn list_knowledge_gaps(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let query_json = request.into_inner().json;
        match list_knowledge_gaps(&self.state, &query_json).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn scan_knowledge_gaps(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match scan_knowledge_gaps(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn approve_knowledge_gap(
        &self,
        request: Request<ApplyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match approve_knowledge_gap(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
        }
    }

    async fn dismiss_knowledge_gap(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match dismiss_knowledge_gap(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::not_found(e)),
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

    let tenant_key = std::env::var("TENANT_KEY").unwrap_or_default();
    let state = KnowledgeBaseState { db: pool, tenant_key };
    let server = KnowledgeBaseServer { state };

    let addr = "[::1]:50117".parse()?;
    info!("knowledge-base service listening on {}", addr);

    KnowledgeBaseServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
