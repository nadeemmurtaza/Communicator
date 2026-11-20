//! WhatsApp Automations standalone gRPC service.

use newax_whatsapp_automations::*;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.whatsapp_automations.v1");
}

use proto::{
    whats_app_automations_service_server::{WhatsAppAutomationsService as WhatsAppAutomationsServiceServerTrait, WhatsAppAutomationsServiceServer},
    JsonResponse,
};

struct Svc {
    db: sqlx::PgPool,
    config: Config,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl WhatsAppAutomationsServiceServerTrait for Svc {
    async fn handle_agent_commands(
        &self,
        req: Request<proto::AgentCommandsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let result = engine.handle_agent_commands(&r.brand_id, &r.conversation_id, &r.contact_id, &r.text).await;
        let json = match result {
            Some(s) => serde_json::json!({ "reply": s }),
            None => serde_json::json!({ "reply": null }),
        };
        Ok(Response::new(json_response(json)))
    }

    async fn handle_csat_response(
        &self,
        req: Request<proto::CsatResponseRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let result = engine.handle_csat_response(&r.brand_id, &r.conversation_id, &r.contact_id, &r.payload).await;
        let json = match result {
            Some(s) => serde_json::json!({ "reply": s }),
            None => serde_json::json!({ "reply": null }),
        };
        Ok(Response::new(json_response(json)))
    }

    async fn execute_handoff(
        &self,
        req: Request<proto::HandoffRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let result = engine.execute_handoff(&r.brand_id, &r.conversation_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "result": result }))))
    }

    async fn handle_navigation_menu(
        &self,
        req: Request<proto::NavigationMenuRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let result = engine.handle_navigation_menu(&r.brand_id, &r.contact_id, &r.payload).await;
        let json = match result {
            Some(s) => serde_json::json!({ "reply": s }),
            None => serde_json::json!({ "reply": null }),
        };
        Ok(Response::new(json_response(json)))
    }

    async fn should_send_welcome_greeting(
        &self,
        req: Request<proto::GreetingCheckRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let result = engine.should_send_welcome_greeting(&r.brand_id, &r.conversation_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "should_send": result }))))
    }

    async fn send_welcome_greeting(
        &self,
        req: Request<proto::GreetingSendRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let result = engine.send_welcome_greeting(&r.brand_id, &r.conversation_id, &r.contact_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "result": result }))))
    }

    async fn match_advanced_rule(
        &self,
        req: Request<proto::RuleMatchRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        match engine.match_advanced_rule(&r.brand_id, &r.text).await.map_err(to_status)? {
            Some((rule_id, action_type, action_config)) => {
                Ok(Response::new(json_response(serde_json::json!({
                    "rule_id": rule_id,
                    "action_type": action_type,
                    "action_config": action_config
                }))))
            }
            None => Ok(Response::new(json_response(serde_json::json!({ "matched": false })))),
        }
    }

    async fn execute_rule_action(
        &self,
        req: Request<proto::RuleActionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let action_config: Option<&str> = if r.action_config.is_empty() { None } else { Some(&r.action_config) };
        match engine.execute_rule_action(&r.brand_id, &r.contact_id, &r.conversation_id, &r.action_type, action_config).await.map_err(to_status)? {
            Some(s) => Ok(Response::new(json_response(serde_json::json!({ "result": s })))),
            None => Ok(Response::new(json_response(serde_json::json!({ "result": null })))),
        }
    }

    async fn run_inactivity_nudges(
        &self,
        req: Request<proto::NudgesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let count = engine.run_inactivity_nudges(&r.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "count": count }))))
    }

    async fn sync_customer_profile_from_meta(
        &self,
        req: Request<proto::ProfileSyncRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let engine = WhatsAppAutomationEngine::new(self.db.clone(), self.config.clone());
        let profile_name: Option<&str> = if r.profile_name.is_empty() { None } else { Some(&r.profile_name) };
        let profile_picture_url: Option<&str> = if r.profile_picture_url.is_empty() { None } else { Some(&r.profile_picture_url) };
        let about: Option<&str> = if r.about.is_empty() { None } else { Some(&r.about) };
        engine.sync_customer_profile_from_meta(&r.brand_id, &r.wa_id, profile_name, profile_picture_url, about).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "synced": true }))))
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

    let config = Config::from_env();
    let server = Svc { db: pool, config };

    let port: u16 = std::env::var("WHATSAPP_AUTOMATIONS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50109);
    let addr = format!("0.0.0.0:{}", port).parse()?;
    info!("whatsapp_automations service listening on {}", addr);

    Server::builder()
        .add_service(WhatsAppAutomationsServiceServer::new(server))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("Shutdown signal received");
}
