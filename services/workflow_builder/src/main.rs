use newax_workflow_builder as workflow_builder;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.workflow_builder.v1");
}

use proto::{
    workflow_builder_service_server::{WorkflowBuilderService, WorkflowBuilderServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: workflow_builder::WorkflowBuilderState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl WorkflowBuilderService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = workflow_builder::get_workflow_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn list_workflows(
        &self,
        req: Request<proto::ListWorkflowsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let org = if r.organization_id.is_empty() { None } else { Some(r.organization_id) };
        let wt = if r.workflow_type.is_empty() { None } else { Some(r.workflow_type) };
        let st = if r.status.is_empty() { None } else { Some(r.status) };
        let v = workflow_builder::list_workflows(&self.state, org.as_deref(), wt.as_deref(), st.as_deref(), r.limit, r.offset).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "workflows": v, "total": v.len() }))))
    }

    async fn get_workflow(
        &self,
        req: Request<proto::GetWorkflowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = workflow_builder::get_workflow(&self.state, &r.workflow_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(v.unwrap_or_else(|| serde_json::json!(null)))))
    }

    async fn create_workflow(
        &self,
        req: Request<proto::CreateWorkflowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let desc = if r.description.is_empty() { None } else { Some(r.description) };
        let tc = if r.trigger_config.is_empty() { None } else { Some(r.trigger_config) };
        let cb = if r.created_by.is_empty() { None } else { Some(r.created_by) };
        let v = workflow_builder::create_workflow(&self.state, &r.organization_id, &r.name, &r.workflow_type, &r.trigger_type, desc.as_deref(), tc.as_deref(), cb.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn update_workflow(
        &self,
        req: Request<proto::UpdateWorkflowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let updates = parse_json(&r.updates)?;
        let v = workflow_builder::update_workflow(&self.state, &r.workflow_id, &updates).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn delete_workflow(
        &self,
        req: Request<proto::DeleteWorkflowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let deleted = workflow_builder::delete_workflow(&self.state, &r.workflow_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "deleted": deleted }))))
    }

    async fn add_step(
        &self,
        req: Request<proto::AddStepRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let config = if r.config.is_empty() { None } else { Some(r.config) };
        let ps = if r.parent_step_id.is_empty() { None } else { Some(r.parent_step_id) };
        let cb = if r.condition_branch.is_empty() { None } else { Some(r.condition_branch) };
        let v = workflow_builder::add_workflow_step(&self.state, &r.workflow_id, &r.step_type, &r.name, r.step_order, config.as_deref(), ps.as_deref(), cb.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn add_condition(
        &self,
        req: Request<proto::AddConditionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let value = if r.value.is_empty() { None } else { Some(r.value) };
        let v = workflow_builder::add_workflow_condition(&self.state, &r.workflow_id, &r.step_id, &r.condition_type, &r.field, &r.operator, value.as_deref(), &r.logic_operator).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn add_trigger(
        &self,
        req: Request<proto::AddTriggerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let fc = if r.filter_conditions.is_empty() { None } else { Some(r.filter_conditions) };
        let v = workflow_builder::add_workflow_trigger(&self.state, &r.workflow_id, &r.trigger_name, &r.trigger_event, fc.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn trigger_execution(
        &self,
        req: Request<proto::TriggerExecutionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let td = if r.trigger_data.is_empty() { None } else { Some(r.trigger_data) };
        let tc = if r.target_contact_id.is_empty() { None } else { Some(r.target_contact_id) };
        let tcv = if r.target_conversation_id.is_empty() { None } else { Some(r.target_conversation_id) };
        let v = workflow_builder::trigger_workflow_execution(&self.state, &r.workflow_id, &r.trigger_source, td.as_deref(), tc.as_deref(), tcv.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn evaluate_conditions(
        &self,
        req: Request<proto::EvaluateConditionsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let context = parse_json(&r.context)?;
        let v = workflow_builder::evaluate_conditions(&self.state, &r.step_id, &context).await.map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn list_executions(
        &self,
        req: Request<proto::ListExecutionsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let wid = if r.workflow_id.is_empty() { None } else { Some(r.workflow_id) };
        let st = if r.status.is_empty() { None } else { Some(r.status) };
        let v = workflow_builder::list_workflow_executions(&self.state, wid.as_deref(), st.as_deref(), r.limit, r.offset).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "executions": v, "total": v.len() }))))
    }

    async fn available_triggers(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = serde_json::json!({
            "triggers": workflow_builder::get_available_trigger_events(),
            "step_types": workflow_builder::get_available_step_types(),
        });
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

    let state = workflow_builder::WorkflowBuilderState::new(pool);

    let addr = "[::1]:50089".parse().unwrap();
    info!("workflow-builder listening on {}", addr);

    Server::builder()
        .add_service(WorkflowBuilderServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
