use newax_copilot_reviewer as copilot_reviewer;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.copilot_reviewer.v1");
}

use proto::{
    copilot_reviewer_service_server::{CopilotReviewerService, CopilotReviewerServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: copilot_reviewer::CopilotReviewerState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl CopilotReviewerService for Svc {
    async fn get_draft_context(
        &self,
        req: Request<proto::GetDraftContextRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let ctx = copilot_reviewer::get_copilot_context_for_draft(&self.state, &r.draft_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(ctx).map_err(to_status)?)))
    }

    async fn get_pending_contexts(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let contexts = copilot_reviewer::get_pending_drafts_context(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(contexts).map_err(to_status)?)))
    }

    async fn refresh_draft_context(
        &self,
        req: Request<proto::RefreshDraftContextRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        copilot_reviewer::delete_cached_context(&self.state, &r.draft_id).await.map_err(to_status)?;
        let ctx = copilot_reviewer::get_copilot_context_for_draft(&self.state, &r.draft_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(ctx).map_err(to_status)?)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Cannot connect to database: {e}"))?;

    let http_client = reqwest::Client::new();

    let state = copilot_reviewer::CopilotReviewerState { db: pool, http_client };

    let addr = "[::1]:50094".parse().unwrap();
    info!("copilot-reviewer listening on {}", addr);

    Server::builder()
        .add_service(CopilotReviewerServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
