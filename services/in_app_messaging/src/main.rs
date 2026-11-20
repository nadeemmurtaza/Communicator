use newax_in_app_messaging as in_app_messaging;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.in_app_messaging.v1");
}

use proto::{
    in_app_messaging_service_server::{InAppMessagingService, InAppMessagingServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: in_app_messaging::InAppMessagingState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl InAppMessagingService for Svc {
    async fn get_summary(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let summary = in_app_messaging::get_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(summary).map_err(to_status)?)))
    }

    async fn create_message(
        &self,
        req: Request<proto::CreateMessageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::create_message(
            &self.state,
            &r.organization_id,
            &r.title,
            &r.body,
            &r.message_type,
            &r.trigger_type,
            if r.trigger_event.is_empty() { None } else { Some(&r.trigger_event) },
            if r.trigger_page.is_empty() { None } else { Some(&r.trigger_page) },
            &r.frequency,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_messages(
        &self,
        req: Request<proto::ListMessagesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let messages = in_app_messaging::list_messages(
            &self.state,
            if r.status.is_empty() { None } else { Some(&r.status) },
            if r.message_type.is_empty() { None } else { Some(&r.message_type) },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(messages).map_err(to_status)?)))
    }

    async fn get_message(
        &self,
        req: Request<proto::GetMessageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let message = in_app_messaging::get_message(&self.state, &r.message_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(message).map_err(to_status)?)))
    }

    async fn update_message(
        &self,
        req: Request<proto::UpdateMessageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::update_message(
            &self.state,
            &r.message_id,
            if r.title.is_empty() { None } else { Some(&r.title) },
            if r.body.is_empty() { None } else { Some(&r.body) },
            if r.message_type.is_empty() { None } else { Some(&r.message_type) },
            if r.status.is_empty() { None } else { Some(&r.status) },
            if r.button_text.is_empty() { None } else { Some(&r.button_text) },
            if r.button_url.is_empty() { None } else { Some(&r.button_url) },
            if r.trigger_type.is_empty() { None } else { Some(&r.trigger_type) },
            if r.trigger_event.is_empty() { None } else { Some(&r.trigger_event) },
            if r.frequency.is_empty() { None } else { Some(&r.frequency) },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_message(
        &self,
        req: Request<proto::DeleteMessageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::delete_message(&self.state, &r.message_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_trigger(
        &self,
        req: Request<proto::CreateTriggerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::create_trigger(
            &self.state,
            &r.organization_id,
            &r.name,
            &r.trigger_type,
            &r.condition,
            &r.action,
            if r.message_id.is_empty() { None } else { Some(&r.message_id) },
            r.delay_seconds,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_triggers(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let triggers = in_app_messaging::list_triggers(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(triggers).map_err(to_status)?)))
    }

    async fn update_trigger(
        &self,
        req: Request<proto::UpdateTriggerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let enabled: Option<bool> = if r.enabled.is_empty() {
            None
        } else {
            Some(r.enabled == "true")
        };
        let result = in_app_messaging::update_trigger(
            &self.state,
            &r.trigger_id,
            if r.name.is_empty() { None } else { Some(&r.name) },
            if r.trigger_type.is_empty() { None } else { Some(&r.trigger_type) },
            if r.condition.is_empty() { None } else { Some(&r.condition) },
            enabled,
            if r.message_id.is_empty() { None } else { Some(&r.message_id) },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_trigger(
        &self,
        req: Request<proto::DeleteTriggerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::delete_trigger(&self.state, &r.trigger_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn record_behavioral_event(
        &self,
        req: Request<proto::RecordBehavioralEventRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::record_behavioral_event(
            &self.state,
            &r.organization_id,
            if r.contact_id.is_empty() { None } else { Some(&r.contact_id) },
            if r.session_id.is_empty() { None } else { Some(&r.session_id) },
            &r.event_name,
            &r.event_data,
            if r.page_url.is_empty() { None } else { Some(&r.page_url) },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn evaluate_triggers(
        &self,
        req: Request<proto::EvaluateTriggersRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let results = in_app_messaging::evaluate_triggers(
            &self.state,
            &r.organization_id,
            &r.event_name,
            if r.page_url.is_empty() { None } else { Some(&r.page_url) },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(results).map_err(to_status)?)))
    }

    async fn record_message_event(
        &self,
        req: Request<proto::RecordMessageEventRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = in_app_messaging::record_message_event(
            &self.state,
            &r.organization_id,
            &r.message_id,
            if r.contact_id.is_empty() { None } else { Some(&r.contact_id) },
            if r.session_id.is_empty() { None } else { Some(&r.session_id) },
            &r.event_type,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_settings(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let settings = in_app_messaging::get_settings(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(settings).map_err(to_status)?)))
    }

    async fn update_settings(
        &self,
        req: Request<proto::UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let enabled: Option<bool> = if r.enabled.is_empty() {
            None
        } else {
            Some(r.enabled == "true")
        };
        let max_concurrent_messages: Option<i32> = if r.max_concurrent_messages.is_empty() {
            None
        } else {
            r.max_concurrent_messages.parse().ok()
        };
        let event_retention_days: Option<i32> = if r.event_retention_days.is_empty() {
            None
        } else {
            r.event_retention_days.parse().ok()
        };
        let track_behavioral_events: Option<bool> = if r.track_behavioral_events.is_empty() {
            None
        } else {
            Some(r.track_behavioral_events == "true")
        };
        let result = in_app_messaging::update_settings(
            &self.state,
            enabled,
            max_concurrent_messages,
            if r.default_style.is_empty() { None } else { Some(&r.default_style) },
            track_behavioral_events,
            event_retention_days,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_behavioral_events(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let events = in_app_messaging::list_behavioral_events(&self.state, None).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(events).map_err(to_status)?)))
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

    let state = in_app_messaging::InAppMessagingState { db: pool, http_client };

    let addr = "[::1]:50095".parse().unwrap();
    info!("in-app-messaging listening on {}", addr);

    Server::builder()
        .add_service(InAppMessagingServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
