use newax_brand_overrides as brand_overrides;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.brand_overrides.v1");
}

use proto::{
    brand_overrides_service_server::{BrandOverridesService, BrandOverridesServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: brand_overrides::BrandOverridesState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl BrandOverridesService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = brand_overrides::get_brand_override_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_rules(
        &self,
        req: Request<proto::ListRulesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let org = if r.organization_id.is_empty() { None } else { Some(r.organization_id) };
        let active = if r.has_is_active { Some(r.is_active) } else { None };
        let v = brand_overrides::list_brand_override_rules(&self.state, org.as_deref(), active, r.limit, r.offset).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn create_rule(
        &self,
        req: Request<proto::CreateRuleRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let ft = if r.trigger_type.is_empty() { "workflow" } else { &r.trigger_type };
        let fb = if r.from_brand_id.is_empty() { None } else { Some(r.from_brand_id) };
        let v = brand_overrides::create_brand_override_rule(&self.state, &r.organization_id, &r.name, ft, fb.as_deref(), &r.to_brand_id, r.auto_switch_kb, r.auto_switch_ai_config, r.auto_switch_messenger).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn update_rule(
        &self,
        req: Request<proto::UpdateRuleRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let name = if r.name.is_empty() { None } else { Some(r.name) };
        let active = if r.has_is_active { Some(r.is_active) } else { None };
        let tb = if r.to_brand_id.is_empty() { None } else { Some(r.to_brand_id) };
        brand_overrides::update_brand_override_rule(&self.state, &r.rule_id, name.as_deref(), active, tb.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({"message": "Rule updated"}))))
    }

    async fn delete_rule(
        &self,
        req: Request<proto::DeleteRuleRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        brand_overrides::delete_brand_override_rule(&self.state, &r.rule_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({"message": "Rule deleted"}))))
    }

    async fn switch_brand(
        &self,
        req: Request<proto::SwitchBrandRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let tt = if r.trigger_type.is_empty() { "manual" } else { &r.trigger_type };
        let ts = if r.trigger_source.is_empty() { "" } else { &r.trigger_source };
        let rid = if r.rule_id.is_empty() { None } else { Some(r.rule_id) };
        let sb = if r.switched_by.is_empty() { "system" } else { &r.switched_by };
        let v = brand_overrides::switch_brand(&self.state, &r.conversation_id, &r.contact_id, &r.from_brand_id, &r.to_brand_id, tt, ts, rid.as_deref(), sb).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_switch_log(
        &self,
        req: Request<proto::GetSwitchLogRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = brand_overrides::get_switch_log_for_conversation(&self.state, &r.conversation_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = brand_overrides::get_brand_override_settings(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn update_settings(
        &self,
        req: Request<proto::UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let enabled = if r.has_enabled { Some(r.enabled) } else { None };
        let cross = if r.has_allow_cross_org_brands { Some(r.allow_cross_org_brands) } else { None };
        let approval = if r.has_require_approval { Some(r.require_approval) } else { None };
        let log_all = if r.has_log_all_switches { Some(r.log_all_switches) } else { None };
        let v = brand_overrides::update_brand_override_settings(&self.state, enabled, cross, approval, log_all).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_hierarchy(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = brand_overrides::list_brand_hierarchy(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn add_hierarchy(
        &self,
        req: Request<proto::AddHierarchyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let rt = if r.relationship_type.is_empty() { "sub_brand" } else { &r.relationship_type };
        brand_overrides::add_brand_hierarchy(&self.state, &r.parent_brand_id, &r.sub_brand_id, rt, r.priority).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({"message": "Hierarchy added"}))))
    }

    async fn remove_hierarchy(
        &self,
        req: Request<proto::RemoveHierarchyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        brand_overrides::remove_brand_hierarchy(&self.state, &r.parent_brand_id, &r.sub_brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({"message": "Hierarchy removed"}))))
    }

    async fn find_matching_rules(
        &self,
        req: Request<proto::FindMatchingRulesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = brand_overrides::find_matching_rules(&self.state, &r.from_brand_id, &r.organization_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_sub_brands(
        &self,
        req: Request<proto::GetSubBrandsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = brand_overrides::get_sub_brands(&self.state, &r.parent_brand_id).await.map_err(to_status)?;
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

    let state = brand_overrides::BrandOverridesState::new(pool);

    let addr = "[::1]:50090".parse().unwrap();
    info!("brand-overrides listening on {}", addr);

    Server::builder()
        .add_service(BrandOverridesServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
