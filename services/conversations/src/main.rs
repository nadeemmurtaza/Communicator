use newax_conversations as conversations;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.conversations.v1");
}

use proto::{
    conversations_service_server::{ConversationsService, ConversationsServiceServer},
    JsonResponse,
};

const PORT: u16 = 50120;

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl ConversationsService for Svc {
    async fn list_conversations(
        &self,
        _req: Request<proto::EmptyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.list_conversations().await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn get_conversation(
        &self,
        req: Request<proto::GetConversationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.get_conversation(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn send_conversation_message(
        &self,
        req: Request<proto::SendMessageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.send_conversation_message(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn resolve_conversation(
        &self,
        req: Request<proto::ResolveConversationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.resolve_conversation(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn update_priority(
        &self,
        req: Request<proto::UpdatePriorityRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.update_priority(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn assign_team(
        &self,
        req: Request<proto::AssignTeamRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.assign_team(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn assign_agent(
        &self,
        req: Request<proto::AssignAgentRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.assign_agent(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn snooze_conversation(
        &self,
        req: Request<proto::SnoozeConversationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.snooze_conversation(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn update_custom_attributes(
        &self,
        req: Request<proto::UpdateCustomAttributesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.update_custom_attributes(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn auto_assign(
        &self,
        req: Request<proto::AutoAssignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.auto_assign(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn list_contact_notes(
        &self,
        req: Request<proto::ListContactNotesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.list_contact_notes(&r.contact_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn add_contact_note(
        &self,
        req: Request<proto::AddContactNoteRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.add_contact_note(&r.contact_id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn enroll_sequence(
        &self,
        req: Request<proto::EnrollSequenceRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.enroll_sequence(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn submit_message_feedback(
        &self,
        req: Request<proto::SubmitFeedbackRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: serde_json::Value = serde_json::from_str(&r.json).map_err(to_status)?;
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.submit_message_feedback(&r.message_id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn get_feedback_summary(
        &self,
        _req: Request<proto::EmptyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let service = conversations::ConversationsService::new(self.pool.clone());
        let result = service.get_feedback_summary().await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
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

    let port: u16 = std::env::var("CONVERSATIONS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(PORT);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-conversations starting (gRPC)");

    Server::builder()
        .add_service(ConversationsServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;

    Ok(())
}
