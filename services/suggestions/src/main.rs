use newax_suggestions as suggestions;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.suggestions.v1");
}

use proto::{
    suggestions_service_server::{SuggestionsService, SuggestionsServiceServer},
    Empty, IdRequest, JsonResponse, ListSuggestionsRequest,
    ApproveSuggestionRequest, BatchAnalyzeRequest,
};

struct Svc {
    state: suggestions::SuggestionsState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

#[tonic::async_trait]
impl SuggestionsService for Svc {
    async fn list_suggestions(
        &self,
        req: Request<ListSuggestionsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = suggestions::list_suggestions(
            &self.state,
            r.status.as_deref(),
            r.suggestion_type.as_deref(),
            r.limit,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_summary(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = suggestions::get_suggestion_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn approve_suggestion(
        &self,
        req: Request<ApproveSuggestionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = suggestions::approve_suggestion(&self.state, &r.id, &r.approved_by)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn dismiss_suggestion(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = suggestions::dismiss_suggestion(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn batch_analyze(
        &self,
        req: Request<BatchAnalyzeRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = suggestions::batch_analyze_negative_feedback(&self.state, r.hours_back)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn analyze_feedback(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = suggestions::analyze_negative_feedback(&self.state, &r.id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_suggestions=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;
    let http_client = reqwest::Client::new();

    let port: u16 = std::env::var("SUGGESTIONS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50083);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = Svc {
        state: suggestions::SuggestionsState { db, http_client },
    };

    info!(port, "svc-suggestions starting (gRPC)");
    Server::builder()
        .add_service(SuggestionsServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
