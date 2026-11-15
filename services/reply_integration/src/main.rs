use newax_reply_integration as ri;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.reply_integration.v1");
}

use proto::{
    reply_integration_service_server::{ReplyIntegrationService, ReplyIntegrationServiceServer},
    BuildContextRequest, Empty, GetSourcesRequest, JsonResponse, UpdateSettingsRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn json_ok<T: serde::Serialize>(v: &T) -> Result<Response<JsonResponse>, Status> {
    Ok(Response::new(JsonResponse {
        json: serde_json::to_string(v).map_err(to_status)?,
    }))
}

#[tonic::async_trait]
impl ReplyIntegrationService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let summary = ri::ReplyIntegrationService::new(&self.pool).get_summary().await.map_err(to_status)?;
        json_ok(&summary)
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let settings = ri::ReplyIntegrationService::new(&self.pool).get_settings().await.map_err(to_status)?;
        json_ok(&settings)
    }

    async fn update_settings(&self, req: Request<UpdateSettingsRequest>) -> Result<Response<JsonResponse>, Status> {
        let settings: ri::ReplyIntegrationSettings =
            serde_json::from_str(&req.into_inner().json).map_err(to_status)?;
        let updated = ri::ReplyIntegrationService::new(&self.pool)
            .update_settings(&settings)
            .await
            .map_err(to_status)?;
        json_ok(&updated)
    }

    async fn build_context(&self, req: Request<BuildContextRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let pkg = ri::ReplyIntegrationService::new(&self.pool)
            .build_context_package(r.contact_id.as_deref(), &r.last_inbound, &r.tenant_key)
            .await
            .map_err(to_status)?;
        json_ok(&pkg)
    }

    async fn get_knowledge_sources(&self, req: Request<GetSourcesRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let sources = ri::ReplyIntegrationService::new(&self.pool)
            .get_knowledge_sources(r.draft_id.as_deref(), r.conversation_id.as_deref(), r.source_type.as_deref())
            .await
            .map_err(to_status)?;
        json_ok(&serde_json::json!({ "sources": sources, "total": sources.len() }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_reply_integration=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("REPLY_INTEGRATION_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50068);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-reply-integration starting (gRPC)");
    Server::builder()
        .add_service(ReplyIntegrationServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
