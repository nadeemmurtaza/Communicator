use newax_topic_categorization as topic_categorization;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.topic_categorization.v1");
}

use proto::{
    topic_categorization_service_server::{TopicCategorizationService, TopicCategorizationServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: topic_categorization::TopicCategorizationState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl TopicCategorizationService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = topic_categorization::get_topic_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn categorize_conversation(
        &self,
        req: Request<proto::CategorizeConversationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = topic_categorization::categorize_conversation(&self.state, &r.conversation_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_conversation_topic(
        &self,
        req: Request<proto::GetConversationTopicRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = topic_categorization::get_conversation_topic(&self.state, &r.conversation_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn override_topic(
        &self,
        req: Request<proto::OverrideTopicRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let subtopic = if r.subtopic.is_empty() { None } else { Some(r.subtopic) };
        let overridden_by = if r.overridden_by.is_empty() { "api_user".to_string() } else { r.overridden_by };
        let v = topic_categorization::override_topic(
            &self.state,
            &r.conversation_id,
            &r.topic,
            subtopic.as_deref(),
            &overridden_by,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn batch_categorize(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = topic_categorization::categorize_uncategorized(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
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

    let state = topic_categorization::TopicCategorizationState::new(pool);

    let addr = "[::1]:50088".parse().unwrap();
    info!("topic-categorization listening on {}", addr);

    Server::builder()
        .add_service(TopicCategorizationServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
