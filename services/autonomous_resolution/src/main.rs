use newax_autonomous_resolution as autonomous_resolution;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.autonomous_resolution.v1");
}

use proto::{
    autonomous_resolution_service_server::{AutonomousResolutionService, AutonomousResolutionServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: autonomous_resolution::AutonomousResolutionState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl AutonomousResolutionService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = autonomous_resolution::get_resolution_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_config(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = autonomous_resolution::get_resolution_config(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn update_config(
        &self,
        req: Request<proto::UpdateConfigRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let config: autonomous_resolution::ResolutionConfig = serde_json::from_str(&r.config_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let v = autonomous_resolution::update_resolution_config(&self.state, &config).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_escalation_rules(
        &self,
        req: Request<proto::ListEscalationRulesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let active = if r.has_is_active { Some(r.is_active) } else { None };
        let v = autonomous_resolution::list_escalation_rules(&self.state, active).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn create_escalation_rule(
        &self,
        req: Request<proto::CreateEscalationRuleRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let desc = if r.description.is_empty() { None } else { Some(r.description) };
        let topic = if r.condition_topic.is_empty() { None } else { Some(r.condition_topic) };
        let kw = if r.condition_keyword.is_empty() { None } else { Some(r.condition_keyword) };
        let agent = if r.assign_to_agent_id.is_empty() { None } else { Some(r.assign_to_agent_id) };
        let team = if r.assign_to_team_id.is_empty() { None } else { Some(r.assign_to_team_id) };
        let webhook = if r.notify_webhook.is_empty() { None } else { Some(r.notify_webhook) };
        let msg = if r.notification_message.is_empty() { None } else { Some(r.notification_message) };
        let v = autonomous_resolution::create_escalation_rule(
            &self.state, &r.name, &r.condition_type, r.condition_value, &r.action_type,
            desc.as_deref(), topic.as_deref(), kw.as_deref(), agent.as_deref(),
            team.as_deref(), webhook.as_deref(), msg.as_deref(), r.priority,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn update_escalation_rule(
        &self,
        req: Request<proto::UpdateEscalationRuleRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let rule: autonomous_resolution::EscalationRule = serde_json::from_str(&r.rule_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let v = autonomous_resolution::update_escalation_rule(&self.state, &rule).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn delete_escalation_rule(
        &self,
        req: Request<proto::DeleteEscalationRuleRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = autonomous_resolution::delete_escalation_rule(&self.state, &r.rule_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_policies(
        &self,
        req: Request<proto::ListPoliciesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let active = if r.has_is_active { Some(r.is_active) } else { None };
        let v = autonomous_resolution::list_agent_policies(&self.state, active).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn evaluate_decision(
        &self,
        req: Request<proto::EvaluateDecisionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let vs = if r.has_validation_score { Some(r.validation_score) } else { None };
        let topic = if r.topic.is_empty() { None } else { Some(r.topic) };
        let v = autonomous_resolution::evaluate_resolution_decision(
            &self.state, &r.conversation_id, r.ai_confidence, vs, topic.as_deref(),
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn log_attempt(
        &self,
        req: Request<proto::LogAttemptRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let vs = if r.has_validation_score { Some(r.validation_score) } else { None };
        let vp = if r.has_validation_passed { Some(r.validation_passed) } else { None };
        // First evaluate the decision
        let decision = autonomous_resolution::evaluate_resolution_decision(
            &self.state, &r.conversation_id, r.ai_confidence, vs, None,
        ).await.map_err(to_status)?;
        // Then log the attempt
        let v = autonomous_resolution::log_resolution_attempt(
            &self.state, &r.conversation_id, &r.contact_id, &r.channel,
            r.ai_confidence, vs, vp, &decision,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_logs(
        &self,
        req: Request<proto::ListLogsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let outcome = if r.outcome.is_empty() { None } else { Some(r.outcome) };
        let escalated = if r.has_escalated { Some(r.escalated) } else { None };
        let v = autonomous_resolution::list_resolution_logs(
            &self.state, outcome.as_deref(), escalated, r.limit, r.offset,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn generate_snapshot(
        &self,
        req: Request<proto::GenerateSnapshotRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let days = if r.days == 0 { 7 } else { r.days };
        let v = autonomous_resolution::generate_analytics_snapshot(&self.state, days).await.map_err(to_status)?;
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

    let state = autonomous_resolution::AutonomousResolutionState::new(pool);

    let addr = "[::1]:50091".parse().unwrap();
    info!("autonomous-resolution listening on {}", addr);

    Server::builder()
        .add_service(AutonomousResolutionServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
