//! Thin gRPC client forwarding workflow builder requests to `svc-workflow-builder` (port 50089).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::workflow_builder::v1::{
    workflow_builder_service_client::WorkflowBuilderServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("WORKFLOW_BUILDER_GRPC").unwrap_or("http://127.0.0.1:50089")
}

async fn client() -> WorkflowBuilderServiceClient<tonic::transport::Channel> {
    WorkflowBuilderServiceClient::connect(svc_addr().to_string())
        .await
        .expect("workflow-builder gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn err_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

#[derive(Deserialize, Debug)]
pub struct WorkflowQueryParams {
    pub organization_id: Option<String>,
    pub workflow_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct ExecutionQueryParams {
    pub workflow_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct CreateWorkflowBody {
    pub organization_id: String,
    pub name: String,
    pub workflow_type: String,
    pub trigger_type: String,
    pub description: Option<String>,
    pub trigger_config: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddStepBody {
    pub step_type: String,
    pub name: String,
    pub step_order: i32,
    pub config: Option<String>,
    pub parent_step_id: Option<String>,
    pub condition_branch: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddConditionBody {
    pub step_id: String,
    pub condition_type: String,
    pub field: String,
    pub operator: String,
    pub value: Option<String>,
    pub logic_operator: String,
}

#[derive(Deserialize, Debug)]
pub struct AddTriggerBody {
    pub trigger_name: String,
    pub trigger_event: String,
    pub filter_conditions: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TriggerExecutionBody {
    pub trigger_source: String,
    pub trigger_data: Option<String>,
    pub target_contact_id: Option<String>,
    pub target_conversation_id: Option<String>,
}

pub async fn workflow_summary(State(_state): State<AppState>) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_workflows(
    State(_state): State<AppState>,
    Query(params): Query<WorkflowQueryParams>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::ListWorkflowsRequest::default();
    req.organization_id = params.organization_id.clone().unwrap_or_default();
    req.workflow_type = params.workflow_type.clone().unwrap_or_default();
    req.status = params.status.clone().unwrap_or_default();
    req.limit = params.limit.unwrap_or(50);
    req.offset = params.offset.unwrap_or(0);
    match client().await.list_workflows(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn available_triggers() -> Response {
    match client().await.available_triggers(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::GetWorkflowRequest::default();
    req.workflow_id = workflow_id;
    match client().await.get_workflow(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

pub async fn create_workflow(
    State(_state): State<AppState>,
    Json(body): Json<CreateWorkflowBody>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::CreateWorkflowRequest::default();
    req.organization_id = body.organization_id;
    req.name = body.name;
    req.workflow_type = body.workflow_type;
    req.trigger_type = body.trigger_type;
    req.description = body.description.unwrap_or_default();
    req.trigger_config = body.trigger_config.unwrap_or_default();
    req.created_by = body.created_by.unwrap_or_default();
    match client().await.create_workflow(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn update_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(updates): Json<Value>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::UpdateWorkflowRequest::default();
    req.workflow_id = workflow_id;
    req.updates = serde_json::to_string(&updates).unwrap_or_default();
    match client().await.update_workflow(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn delete_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::DeleteWorkflowRequest::default();
    req.workflow_id = workflow_id;
    match client().await.delete_workflow(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn add_step(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<AddStepBody>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::AddStepRequest::default();
    req.workflow_id = workflow_id;
    req.step_type = body.step_type;
    req.name = body.name;
    req.step_order = body.step_order;
    req.config = body.config.unwrap_or_default();
    req.parent_step_id = body.parent_step_id.unwrap_or_default();
    req.condition_branch = body.condition_branch.unwrap_or_default();
    match client().await.add_step(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn add_condition(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<AddConditionBody>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::AddConditionRequest::default();
    req.workflow_id = workflow_id;
    req.step_id = body.step_id;
    req.condition_type = body.condition_type;
    req.field = body.field;
    req.operator = body.operator;
    req.value = body.value.unwrap_or_default();
    req.logic_operator = body.logic_operator;
    match client().await.add_condition(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn add_trigger(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<AddTriggerBody>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::AddTriggerRequest::default();
    req.workflow_id = workflow_id;
    req.trigger_name = body.trigger_name;
    req.trigger_event = body.trigger_event;
    req.filter_conditions = body.filter_conditions.unwrap_or_default();
    match client().await.add_trigger(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn trigger_execution(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<TriggerExecutionBody>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::TriggerExecutionRequest::default();
    req.workflow_id = workflow_id;
    req.trigger_source = body.trigger_source;
    req.trigger_data = body.trigger_data.unwrap_or_default();
    req.target_contact_id = body.target_contact_id.unwrap_or_default();
    req.target_conversation_id = body.target_conversation_id.unwrap_or_default();
    match client().await.trigger_execution(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn evaluate_conditions(
    State(_state): State<AppState>,
    Path(_workflow_id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let step_id = body.get("step_id").and_then(|v| v.as_str()).unwrap_or("");
    let context = body.get("context").cloned().unwrap_or(serde_json::json!({}));
    let mut req = newax_proto::workflow_builder::v1::EvaluateConditionsRequest::default();
    req.step_id = step_id.to_string();
    req.context = serde_json::to_string(&context).unwrap_or_default();
    match client().await.evaluate_conditions(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_executions(
    State(_state): State<AppState>,
    Query(params): Query<ExecutionQueryParams>,
) -> Response {
    let mut req = newax_proto::workflow_builder::v1::ListExecutionsRequest::default();
    req.workflow_id = params.workflow_id.clone().unwrap_or_default();
    req.status = params.status.clone().unwrap_or_default();
    req.limit = params.limit.unwrap_or(50);
    req.offset = params.offset.unwrap_or(0);
    match client().await.list_executions(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}
