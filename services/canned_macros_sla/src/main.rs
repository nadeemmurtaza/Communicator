mod handlers;

pub mod proto {
    tonic::include_proto!("newax.canned_macros_sla.v1");
}

use proto::{
    canned_macros_sla_service_server::{CannedMacrosSlaService, CannedMacrosSlaServiceServer},
    Empty, JsonResponse,
};
use sqlx::postgres::PgPoolOptions;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use tracing::info;

struct Svc {
    state: handlers::AppState,
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl CannedMacrosSlaService for Svc {
    async fn list_canned_responses(
        &self,
        req: Request<proto::ListCannedResponsesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let query = serde_json::json!({ "q": r.q, "category": r.category });
        let v = handlers::list_canned_responses(&self.state, &query).await;
        Ok(Response::new(json_response(v)))
    }

    async fn create_canned_response(
        &self,
        req: Request<proto::CreateCannedResponseRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "short_code": r.short_code, "content": r.content, "category": r.category });
        let v = handlers::create_canned_response(&self.state, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn update_canned_response(
        &self,
        req: Request<proto::UpdateCannedResponseRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "short_code": r.short_code, "content": r.content, "category": r.category });
        let v = handlers::update_canned_response(&self.state, r.id, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn delete_canned_response(
        &self,
        req: Request<proto::DeleteCannedResponseRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::delete_canned_response(&self.state, r.id).await;
        Ok(Response::new(json_response(v)))
    }

    async fn expand_canned_response(
        &self,
        req: Request<proto::ExpandCannedResponseRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "short_code": r.short_code, "template": r.template });
        let v = handlers::expand_canned_response(&self.state, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn list_macros(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = handlers::list_macros(&self.state).await;
        Ok(Response::new(json_response(v)))
    }

    async fn create_macro(
        &self,
        req: Request<proto::CreateMacroRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "name": r.name, "description": r.description, "actions": r.actions });
        let v = handlers::create_macro(&self.state, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn get_macro(
        &self,
        req: Request<proto::GetMacroRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::get_macro(&self.state, r.id).await;
        Ok(Response::new(json_response(v)))
    }

    async fn update_macro(
        &self,
        req: Request<proto::UpdateMacroRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "name": r.name, "description": r.description, "actions": r.actions });
        let v = handlers::update_macro(&self.state, r.id, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn delete_macro(
        &self,
        req: Request<proto::DeleteMacroRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::delete_macro(&self.state, r.id).await;
        Ok(Response::new(json_response(v)))
    }

    async fn run_macro(
        &self,
        req: Request<proto::RunMacroRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::run_macro(&self.state, r.conversation_id, r.macro_id, r.agent_name).await;
        Ok(Response::new(json_response(v)))
    }

    async fn execute_macro(
        &self,
        req: Request<proto::ExecuteMacroRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "agent_name": r.agent_name });
        let v = handlers::execute_macro(&self.state, r.conversation_id, r.macro_id, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn execute_macro_direct(
        &self,
        req: Request<proto::ExecuteMacroDirectRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({ "conversation_id": r.conversation_id, "agent_name": r.agent_name });
        let v = handlers::execute_macro_direct(&self.state, r.macro_id, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn dispatch_whats_app_text(
        &self,
        req: Request<proto::DispatchWhatsAppTextRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::dispatch_whatsapp_text(&self.state, &r.phone, &r.body, &r.local_msg_id).await;
        Ok(Response::new(json_response(v)))
    }

    async fn list_sla_policies(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = handlers::list_sla_policies(&self.state).await;
        Ok(Response::new(json_response(v)))
    }

    async fn get_sla_policy(
        &self,
        req: Request<proto::GetSlaPolicyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::get_sla_policy(&self.state, r.id).await;
        Ok(Response::new(json_response(v)))
    }

    async fn create_sla_policy(
        &self,
        req: Request<proto::CreateSlaPolicyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = serde_json::json!({
            "name": r.name,
            "description": if r.description.is_empty() { serde_json::Value::Null } else { serde_json::json!(r.description) },
            "first_response_time_threshold": if r.first_response_time_threshold == 0 { serde_json::Value::Null } else { serde_json::json!(r.first_response_time_threshold) },
            "resolution_time_threshold": if r.resolution_time_threshold == 0 { serde_json::Value::Null } else { serde_json::json!(r.resolution_time_threshold) },
            "only_during_business_hours": if r.only_during_business_hours { serde_json::json!(true) } else { serde_json::Value::Null },
            "priorities": r.priorities,
            "inbox_ids": r.inbox_ids,
        });
        let v = handlers::create_sla_policy(&self.state, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn update_sla_policy(
        &self,
        req: Request<proto::UpdateSlaPolicyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let mut payload = serde_json::json!({});
        let obj = payload.as_object_mut().unwrap();
        if r.has_name && !r.name.is_empty() { obj.insert("name".into(), serde_json::json!(r.name)); }
        if r.has_description { obj.insert("description".into(), if r.description.is_empty() { serde_json::Value::Null } else { serde_json::json!(r.description) }); }
        if r.has_first_response_time_threshold { obj.insert("first_response_time_threshold".into(), serde_json::json!(r.first_response_time_threshold)); }
        if r.has_resolution_time_threshold { obj.insert("resolution_time_threshold".into(), serde_json::json!(r.resolution_time_threshold)); }
        if r.has_only_during_business_hours { obj.insert("only_during_business_hours".into(), serde_json::json!(r.only_during_business_hours)); }
        if r.has_priorities { obj.insert("priorities".into(), serde_json::json!(r.priorities)); }
        if r.has_inbox_ids { obj.insert("inbox_ids".into(), serde_json::json!(r.inbox_ids)); }
        if r.has_is_active { obj.insert("is_active".into(), serde_json::json!(r.is_active)); }
        let v = handlers::update_sla_policy(&self.state, r.id, &payload).await;
        Ok(Response::new(json_response(v)))
    }

    async fn delete_sla_policy(
        &self,
        req: Request<proto::DeleteSlaPolicyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = handlers::delete_sla_policy(&self.state, r.id).await;
        Ok(Response::new(json_response(v)))
    }

    async fn evaluate_conversation_sla(
        &self,
        req: Request<proto::EvaluateConversationSlaRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let query = serde_json::json!({ "policy_id": if r.policy_id.is_empty() { serde_json::Value::Null } else { serde_json::json!(r.policy_id) } });
        let v = handlers::evaluate_conversation_sla(&self.state, r.id, &query).await;
        Ok(Response::new(json_response(v)))
    }

    async fn get_sla_metrics(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = handlers::get_sla_metrics(&self.state).await;
        Ok(Response::new(json_response(v)))
    }

    async fn get_sla_breaches(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = handlers::get_sla_breaches(&self.state).await;
        Ok(Response::new(json_response(v)))
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

    let state = handlers::AppState {
        db: pool,
        http_client: reqwest::Client::new(),
        meta_whatsapp_access_token: std::env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default(),
        meta_whatsapp_phone_number_id: std::env::var("META_WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default(),
        meta_whatsapp_api_version: std::env::var("META_WHATSAPP_API_VERSION").unwrap_or_default(),
    };

    let addr = "[::1]:50086".parse().unwrap();
    info!("canned-macros-sla listening on {}", addr);

    Server::builder()
        .add_service(CannedMacrosSlaServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
