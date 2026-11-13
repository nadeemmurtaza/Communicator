//! Workflow Builder API routes (Phase 20).

use crate::db::AppState;
use crate::services::workflow_builder;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

// ——— Endpoints ———

/// GET /workflows/summary
pub async fn workflow_summary(State(state): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match workflow_builder::get_workflow_summary(&state).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_default())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// GET /workflows
pub async fn list_workflows(
    State(state): State<AppState>,
    Query(params): Query<WorkflowQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let workflows = workflow_builder::list_workflows(
        &state,
        params.organization_id.as_deref(),
        params.workflow_type.as_deref(),
        params.status.as_deref(),
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "workflows": workflows,
        "total": workflows.len(),
    })))
}

/// GET /workflows/available-triggers
pub async fn available_triggers() -> Json<Value> {
    Json(json!({
        "triggers": workflow_builder::get_available_trigger_events(),
        "step_types": workflow_builder::get_available_step_types(),
    }))
}

/// GET /workflows/:id
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match workflow_builder::get_workflow(&state, &workflow_id).await {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Workflow not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// POST /workflows
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(body): Json<CreateWorkflowBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let wf = workflow_builder::create_workflow(
        &state,
        &body.organization_id,
        &body.name,
        &body.workflow_type,
        &body.trigger_type,
        body.description.as_deref(),
        body.trigger_config.as_deref(),
        body.created_by.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(wf).unwrap_or_default()))
}

/// PUT /workflows/:id
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(updates): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let wf = workflow_builder::update_workflow(&state, &workflow_id, &updates)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(wf).unwrap_or_default()))
}

/// DELETE /workflows/:id
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let deleted = workflow_builder::delete_workflow(&state, &workflow_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if deleted {
        Ok(Json(json!({"deleted": true})))
    } else {
        Err((StatusCode::NOT_FOUND, Json(json!({"error": "Workflow not found"}))))
    }
}

/// POST /workflows/:id/steps
pub async fn add_step(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<AddStepBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let step = workflow_builder::add_workflow_step(
        &state,
        &workflow_id,
        &body.step_type,
        &body.name,
        body.step_order,
        body.config.as_deref(),
        body.parent_step_id.as_deref(),
        body.condition_branch.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(step).unwrap_or_default()))
}

/// POST /workflows/:id/conditions
pub async fn add_condition(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<AddConditionBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cond = workflow_builder::add_workflow_condition(
        &state,
        &workflow_id,
        &body.step_id,
        &body.condition_type,
        &body.field,
        &body.operator,
        body.value.as_deref(),
        &body.logic_operator,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(cond).unwrap_or_default()))
}

/// POST /workflows/:id/triggers
pub async fn add_trigger(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<AddTriggerBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let trigger = workflow_builder::add_workflow_trigger(
        &state,
        &workflow_id,
        &body.trigger_name,
        &body.trigger_event,
        body.filter_conditions.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(trigger).unwrap_or_default()))
}

/// POST /workflows/:id/execute
pub async fn trigger_execution(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<TriggerExecutionBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = workflow_builder::trigger_workflow_execution(
        &state,
        &workflow_id,
        &body.trigger_source,
        body.trigger_data.as_deref(),
        body.target_contact_id.as_deref(),
        body.target_conversation_id.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// POST /workflows/:id/evaluate-conditions
pub async fn evaluate_conditions(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let step_id = body.get("step_id").and_then(|v| v.as_str()).unwrap_or("");
    let context = body.get("context").cloned().unwrap_or(json!({}));

    let result = workflow_builder::evaluate_conditions(&state, step_id, &context)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /workflows/executions
pub async fn list_executions(
    State(state): State<AppState>,
    Query(params): Query<ExecutionQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let executions = workflow_builder::list_workflow_executions(
        &state,
        params.workflow_id.as_deref(),
        params.status.as_deref(),
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "executions": executions,
        "total": executions.len(),
    })))
}
