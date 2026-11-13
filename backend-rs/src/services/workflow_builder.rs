//! Workflow Builder Service (Phase 20) — no-code automation engine:
//! multi-step workflows with conditional branching and trigger-based
//! execution.
//!
//! Backed by `workflows`, `workflow_steps`, `workflow_conditions`,
//! `workflow_executions` and `workflow_triggers` (migration 0028).

use crate::db::AppState;
use serde::Serialize;
use sqlx::Row;

fn datetime_string(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get::<chrono::DateTime<chrono::Utc>, _>(column)
        .map(|d| d.to_rfc3339())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub struct Workflow {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: String,
    pub status: String,
    pub trigger_type: String,
    pub trigger_config: Option<String>,
    pub steps_json: Option<String>,
    pub execution_count: i32,
    pub last_executed_at: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_workflow(row: &sqlx::postgres::PgRow) -> Workflow {
    Workflow {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").ok(),
        workflow_type: row.try_get("workflow_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        trigger_type: row.try_get("trigger_type").unwrap_or_default(),
        trigger_config: row.try_get("trigger_config").ok(),
        steps_json: row.try_get("steps_json").ok(),
        execution_count: row.try_get("execution_count").unwrap_or(0),
        last_executed_at: row.try_get("last_executed_at").ok(),
        created_by: row.try_get("created_by").ok(),
        created_at: datetime_string(row, "created_at"),
        updated_at: datetime_string(row, "updated_at"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowStep {
    pub id: String,
    pub workflow_id: String,
    pub step_order: i32,
    pub step_type: String,
    pub name: String,
    pub config: Option<String>,
    pub condition_branch: Option<String>,
    pub parent_step_id: Option<String>,
    pub on_success_next: Option<String>,
    pub on_failure_next: Option<String>,
    pub created_at: String,
}

fn row_to_step(row: &sqlx::postgres::PgRow) -> WorkflowStep {
    WorkflowStep {
        id: row.try_get("id").unwrap_or_default(),
        workflow_id: row.try_get("workflow_id").unwrap_or_default(),
        step_order: row.try_get("step_order").unwrap_or(0),
        step_type: row.try_get("step_type").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        config: row.try_get("config").ok(),
        condition_branch: row.try_get("condition_branch").ok(),
        parent_step_id: row.try_get("parent_step_id").ok(),
        on_success_next: row.try_get("on_success_next").ok(),
        on_failure_next: row.try_get("on_failure_next").ok(),
        created_at: datetime_string(row, "created_at"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowCondition {
    pub id: String,
    pub workflow_id: String,
    pub step_id: String,
    pub condition_type: String,
    pub field: String,
    pub operator: String,
    pub value: Option<String>,
    pub logic_operator: String,
    pub branch_on_match: String,
    pub created_at: String,
}

fn row_to_condition(row: &sqlx::postgres::PgRow) -> WorkflowCondition {
    WorkflowCondition {
        id: row.try_get("id").unwrap_or_default(),
        workflow_id: row.try_get("workflow_id").unwrap_or_default(),
        step_id: row.try_get("step_id").unwrap_or_default(),
        condition_type: row.try_get("condition_type").unwrap_or_default(),
        field: row.try_get("field").unwrap_or_default(),
        operator: row.try_get("operator").unwrap_or_default(),
        value: row.try_get("value").ok(),
        logic_operator: row.try_get("logic_operator").unwrap_or_default(),
        branch_on_match: row.try_get("branch_on_match").unwrap_or_default(),
        created_at: datetime_string(row, "created_at"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowTrigger {
    pub id: String,
    pub workflow_id: String,
    pub trigger_name: String,
    pub trigger_event: String,
    pub filter_conditions: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

fn row_to_trigger(row: &sqlx::postgres::PgRow) -> WorkflowTrigger {
    WorkflowTrigger {
        id: row.try_get("id").unwrap_or_default(),
        workflow_id: row.try_get("workflow_id").unwrap_or_default(),
        trigger_name: row.try_get("trigger_name").unwrap_or_default(),
        trigger_event: row.try_get("trigger_event").unwrap_or_default(),
        filter_conditions: row.try_get("filter_conditions").ok(),
        is_active: row.try_get("is_active").unwrap_or(true),
        created_at: datetime_string(row, "created_at"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,
    pub trigger_source: Option<String>,
    pub trigger_data: Option<String>,
    pub target_contact_id: Option<String>,
    pub target_conversation_id: Option<String>,
    pub current_step_id: Option<String>,
    pub status: String,
    pub step_results: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

fn row_to_execution(row: &sqlx::postgres::PgRow) -> WorkflowExecution {
    WorkflowExecution {
        id: row.try_get("id").unwrap_or_default(),
        workflow_id: row.try_get("workflow_id").unwrap_or_default(),
        trigger_source: row.try_get("trigger_source").ok(),
        trigger_data: row.try_get("trigger_data").ok(),
        target_contact_id: row.try_get("target_contact_id").ok(),
        target_conversation_id: row.try_get("target_conversation_id").ok(),
        current_step_id: row.try_get("current_step_id").ok(),
        status: row.try_get("status").unwrap_or_default(),
        step_results: row.try_get("step_results").ok(),
        error_message: row.try_get("error_message").ok(),
        started_at: row.try_get("started_at").unwrap_or_default(),
        completed_at: row.try_get("completed_at").ok(),
    }
}

const WORKFLOW_COLUMNS: &str = "id, organization_id, name, description, workflow_type, status, trigger_type, \
    trigger_config, steps_json, execution_count, last_executed_at, created_by, created_at, updated_at";

async fn get_workflow_row(state: &AppState, id: &str) -> Result<Option<Workflow>, String> {
    let row = sqlx::query(&format!("SELECT {} FROM workflows WHERE id = $1", WORKFLOW_COLUMNS))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|r| row_to_workflow(&r)))
}

/// Dashboard summary: workflow counts by status/type and recent execution stats.
pub async fn get_workflow_summary(state: &AppState) -> Result<serde_json::Value, String> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflows")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflows WHERE status = 'active'")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let draft: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflows WHERE status = 'draft'")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let total_executions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflow_executions")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let running_executions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM workflow_executions WHERE status = 'running'")
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    let failed_executions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM workflow_executions WHERE status = 'failed'")
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total_workflows": total,
        "active_workflows": active,
        "draft_workflows": draft,
        "total_executions": total_executions,
        "running_executions": running_executions,
        "failed_executions": failed_executions,
    }))
}

/// The trigger events a workflow can be started from.
pub fn get_available_trigger_events() -> Vec<&'static str> {
    vec![
        "conversation_created",
        "message_received",
        "message_sent",
        "contact_updated",
        "tag_added",
        "state_changed",
        "feedback_received",
        "scheduled",
        "webhook",
    ]
}

/// The step types available in the visual editor.
pub fn get_available_step_types() -> Vec<&'static str> {
    vec![
        "wait",
        "send_message",
        "assign",
        "tag",
        "condition",
        "action",
        "notify",
        "update_field",
        "create_ticket",
        "custom",
    ]
}

/// List workflows with optional filtering.
pub async fn list_workflows(
    state: &AppState,
    organization_id: Option<&str>,
    workflow_type: Option<&str>,
    status: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<Workflow>, String> {
    let mut query = format!("SELECT {} FROM workflows WHERE 1=1", WORKFLOW_COLUMNS);
    let mut string_binds: Vec<String> = Vec::new();
    let mut idx = 1;

    if let Some(org) = organization_id {
        query.push_str(&format!(" AND organization_id = ${}", idx));
        string_binds.push(org.to_string());
        idx += 1;
    }
    if let Some(t) = workflow_type {
        query.push_str(&format!(" AND workflow_type = ${}", idx));
        string_binds.push(t.to_string());
        idx += 1;
    }
    if let Some(s) = status {
        query.push_str(&format!(" AND status = ${}", idx));
        string_binds.push(s.to_string());
        idx += 1;
    }
    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", idx, idx + 1));

    let mut q = sqlx::query(&query);
    for b in &string_binds {
        q = q.bind(b);
    }
    q = q.bind(limit as i64).bind(offset as i64);

    let rows = q.fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_workflow).collect())
}

/// Fetch a workflow with its steps, conditions and triggers, as one JSON blob.
pub async fn get_workflow(state: &AppState, workflow_id: &str) -> Result<Option<serde_json::Value>, String> {
    let Some(workflow) = get_workflow_row(state, workflow_id).await? else {
        return Ok(None);
    };

    let step_rows = sqlx::query(
        "SELECT id, workflow_id, step_order, step_type, name, config, condition_branch, \
         parent_step_id, on_success_next, on_failure_next, created_at \
         FROM workflow_steps WHERE workflow_id = $1 ORDER BY step_order ASC",
    )
    .bind(workflow_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let steps: Vec<WorkflowStep> = step_rows.iter().map(row_to_step).collect();

    let condition_rows = sqlx::query(
        "SELECT id, workflow_id, step_id, condition_type, field, operator, value, \
         logic_operator, branch_on_match, created_at \
         FROM workflow_conditions WHERE workflow_id = $1",
    )
    .bind(workflow_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let conditions: Vec<WorkflowCondition> = condition_rows.iter().map(row_to_condition).collect();

    let trigger_rows = sqlx::query(
        "SELECT id, workflow_id, trigger_name, trigger_event, filter_conditions, is_active, created_at \
         FROM workflow_triggers WHERE workflow_id = $1",
    )
    .bind(workflow_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let triggers: Vec<WorkflowTrigger> = trigger_rows.iter().map(row_to_trigger).collect();

    Ok(Some(serde_json::json!({
        "workflow": workflow,
        "steps": steps,
        "conditions": conditions,
        "triggers": triggers,
    })))
}

/// Create a new workflow (starts as a draft).
pub async fn create_workflow(
    state: &AppState,
    organization_id: &str,
    name: &str,
    workflow_type: &str,
    trigger_type: &str,
    description: Option<&str>,
    trigger_config: Option<&str>,
    created_by: Option<&str>,
) -> Result<Workflow, String> {
    let id = format!("wf-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO workflows (id, organization_id, name, description, workflow_type, trigger_type, trigger_config, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&id)
    .bind(organization_id)
    .bind(name)
    .bind(description)
    .bind(workflow_type)
    .bind(trigger_type)
    .bind(trigger_config)
    .bind(created_by)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to create workflow: {}", e))?;

    get_workflow_row(state, &id)
        .await?
        .ok_or_else(|| "Workflow not found after insert".to_string())
}

/// Update a workflow's editable fields from a loosely-typed JSON patch.
pub async fn update_workflow(
    state: &AppState,
    workflow_id: &str,
    updates: &serde_json::Value,
) -> Result<Workflow, String> {
    let name = updates.get("name").and_then(|v| v.as_str());
    let description = updates.get("description").and_then(|v| v.as_str());
    let workflow_type = updates.get("workflow_type").and_then(|v| v.as_str());
    let status = updates.get("status").and_then(|v| v.as_str());
    let trigger_type = updates.get("trigger_type").and_then(|v| v.as_str());
    let trigger_config = updates.get("trigger_config").and_then(|v| v.as_str());
    let steps_json = updates.get("steps_json").and_then(|v| v.as_str());

    let row = sqlx::query(&format!(
        "UPDATE workflows SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            workflow_type = COALESCE($3, workflow_type),
            status = COALESCE($4, status),
            trigger_type = COALESCE($5, trigger_type),
            trigger_config = COALESCE($6, trigger_config),
            steps_json = COALESCE($7, steps_json),
            updated_at = NOW()
         WHERE id = $8
         RETURNING {}",
        WORKFLOW_COLUMNS
    ))
    .bind(name)
    .bind(description)
    .bind(workflow_type)
    .bind(status)
    .bind(trigger_type)
    .bind(trigger_config)
    .bind(steps_json)
    .bind(workflow_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("Failed to update workflow: {}", e))?;

    row.map(|r| row_to_workflow(&r)).ok_or_else(|| "Workflow not found".to_string())
}

/// Delete a workflow (cascades to its steps/conditions/triggers/executions
/// only where the schema defines a cascading FK — here we clean up manually).
pub async fn delete_workflow(state: &AppState, workflow_id: &str) -> Result<bool, String> {
    sqlx::query("DELETE FROM workflow_steps WHERE workflow_id = $1")
        .bind(workflow_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM workflow_conditions WHERE workflow_id = $1")
        .bind(workflow_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM workflow_triggers WHERE workflow_id = $1")
        .bind(workflow_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM workflow_executions WHERE workflow_id = $1")
        .bind(workflow_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let result = sqlx::query("DELETE FROM workflows WHERE id = $1")
        .bind(workflow_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete workflow: {}", e))?;

    Ok(result.rows_affected() > 0)
}

/// Add a step to a workflow.
pub async fn add_workflow_step(
    state: &AppState,
    workflow_id: &str,
    step_type: &str,
    name: &str,
    step_order: i32,
    config: Option<&str>,
    parent_step_id: Option<&str>,
    condition_branch: Option<&str>,
) -> Result<WorkflowStep, String> {
    let id = format!("step-{}", uuid::Uuid::new_v4());
    let row = sqlx::query(
        "INSERT INTO workflow_steps (id, workflow_id, step_order, step_type, name, config, parent_step_id, condition_branch) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, workflow_id, step_order, step_type, name, config, condition_branch, \
                   parent_step_id, on_success_next, on_failure_next, created_at",
    )
    .bind(&id)
    .bind(workflow_id)
    .bind(step_order)
    .bind(step_type)
    .bind(name)
    .bind(config)
    .bind(parent_step_id)
    .bind(condition_branch)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to add workflow step: {}", e))?;

    Ok(row_to_step(&row))
}

/// Add a condition to a workflow step.
pub async fn add_workflow_condition(
    state: &AppState,
    workflow_id: &str,
    step_id: &str,
    condition_type: &str,
    field: &str,
    operator: &str,
    value: Option<&str>,
    logic_operator: &str,
) -> Result<WorkflowCondition, String> {
    let id = format!("cond-{}", uuid::Uuid::new_v4());
    let row = sqlx::query(
        "INSERT INTO workflow_conditions (id, workflow_id, step_id, condition_type, field, operator, value, logic_operator) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, workflow_id, step_id, condition_type, field, operator, value, \
                   logic_operator, branch_on_match, created_at",
    )
    .bind(&id)
    .bind(workflow_id)
    .bind(step_id)
    .bind(condition_type)
    .bind(field)
    .bind(operator)
    .bind(value)
    .bind(logic_operator)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to add workflow condition: {}", e))?;

    Ok(row_to_condition(&row))
}

/// Add an event trigger to a workflow.
pub async fn add_workflow_trigger(
    state: &AppState,
    workflow_id: &str,
    trigger_name: &str,
    trigger_event: &str,
    filter_conditions: Option<&str>,
) -> Result<WorkflowTrigger, String> {
    let id = format!("trig-{}", uuid::Uuid::new_v4());
    let row = sqlx::query(
        "INSERT INTO workflow_triggers (id, workflow_id, trigger_name, trigger_event, filter_conditions) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, workflow_id, trigger_name, trigger_event, filter_conditions, is_active, created_at",
    )
    .bind(&id)
    .bind(workflow_id)
    .bind(trigger_name)
    .bind(trigger_event)
    .bind(filter_conditions)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to add workflow trigger: {}", e))?;

    Ok(row_to_trigger(&row))
}

/// Start a new execution of a workflow and bump its execution counter.
pub async fn trigger_workflow_execution(
    state: &AppState,
    workflow_id: &str,
    trigger_source: &str,
    trigger_data: Option<&str>,
    target_contact_id: Option<&str>,
    target_conversation_id: Option<&str>,
) -> Result<WorkflowExecution, String> {
    get_workflow_row(state, workflow_id)
        .await?
        .ok_or_else(|| "Workflow not found".to_string())?;

    let id = format!("exec-{}", uuid::Uuid::new_v4());
    let row = sqlx::query(
        "INSERT INTO workflow_executions (id, workflow_id, trigger_source, trigger_data, target_contact_id, target_conversation_id) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, workflow_id, trigger_source, trigger_data, target_contact_id, target_conversation_id, \
                   current_step_id, status, step_results, error_message, started_at, completed_at",
    )
    .bind(&id)
    .bind(workflow_id)
    .bind(trigger_source)
    .bind(trigger_data)
    .bind(target_contact_id)
    .bind(target_conversation_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to start workflow execution: {}", e))?;

    sqlx::query(
        "UPDATE workflows SET execution_count = execution_count + 1, last_executed_at = NOW()::TEXT, updated_at = NOW() \
         WHERE id = $1",
    )
    .bind(workflow_id)
    .execute(&state.db)
    .await
    .ok();

    Ok(row_to_execution(&row))
}

/// Evaluate every condition attached to a step against a JSON context,
/// combining them with each condition's `logic_operator` relative to the
/// previous one, and report which branch (`true`/`false`) matched.
pub async fn evaluate_conditions(
    state: &AppState,
    step_id: &str,
    context: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let rows = sqlx::query(
        "SELECT id, workflow_id, step_id, condition_type, field, operator, value, \
         logic_operator, branch_on_match, created_at \
         FROM workflow_conditions WHERE step_id = $1 ORDER BY created_at ASC",
    )
    .bind(step_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let conditions: Vec<WorkflowCondition> = rows.iter().map(row_to_condition).collect();

    if conditions.is_empty() {
        return Ok(serde_json::json!({
            "step_id": step_id,
            "matched": false,
            "branch": "false",
            "evaluations": [],
        }));
    }

    let mut evaluations = Vec::with_capacity(conditions.len());
    let mut result = evaluate_condition(&conditions[0], context);
    evaluations.push(serde_json::json!({
        "field": conditions[0].field,
        "operator": conditions[0].operator,
        "matched": result,
    }));

    for cond in &conditions[1..] {
        let this_result = evaluate_condition(cond, context);
        evaluations.push(serde_json::json!({
            "field": cond.field,
            "operator": cond.operator,
            "matched": this_result,
        }));
        result = match cond.logic_operator.as_str() {
            "or" => result || this_result,
            _ => result && this_result,
        };
    }

    let branch = if result {
        conditions[0].branch_on_match.clone()
    } else if conditions[0].branch_on_match == "true" {
        "false".to_string()
    } else {
        "true".to_string()
    };

    Ok(serde_json::json!({
        "step_id": step_id,
        "matched": result,
        "branch": branch,
        "evaluations": evaluations,
    }))
}

fn evaluate_condition(condition: &WorkflowCondition, context: &serde_json::Value) -> bool {
    let actual = context.get(&condition.field);

    match condition.operator.as_str() {
        "is_empty" => actual.map(|v| is_empty_value(v)).unwrap_or(true),
        "is_not_empty" => actual.map(|v| !is_empty_value(v)).unwrap_or(false),
        _ => {
            let Some(actual) = actual else { return false };
            let Some(expected) = condition.value.as_deref() else { return false };

            match condition.operator.as_str() {
                "equals" => value_as_string(actual) == expected,
                "not_equals" => value_as_string(actual) != expected,
                "contains" => value_as_string(actual).contains(expected),
                "greater_than" => actual
                    .as_f64()
                    .zip(expected.parse::<f64>().ok())
                    .map(|(a, e)| a > e)
                    .unwrap_or(false),
                "less_than" => actual
                    .as_f64()
                    .zip(expected.parse::<f64>().ok())
                    .map(|(a, e)| a < e)
                    .unwrap_or(false),
                _ => false,
            }
        }
    }
}

fn is_empty_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

fn value_as_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// List workflow executions, optionally filtered by workflow and/or status.
pub async fn list_workflow_executions(
    state: &AppState,
    workflow_id: Option<&str>,
    status: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<WorkflowExecution>, String> {
    let mut query = String::from(
        "SELECT id, workflow_id, trigger_source, trigger_data, target_contact_id, target_conversation_id, \
         current_step_id, status, step_results, error_message, started_at, completed_at \
         FROM workflow_executions WHERE 1=1",
    );
    let mut string_binds: Vec<String> = Vec::new();
    let mut idx = 1;

    if let Some(w) = workflow_id {
        query.push_str(&format!(" AND workflow_id = ${}", idx));
        string_binds.push(w.to_string());
        idx += 1;
    }
    if let Some(s) = status {
        query.push_str(&format!(" AND status = ${}", idx));
        string_binds.push(s.to_string());
        idx += 1;
    }
    query.push_str(&format!(" ORDER BY started_at DESC LIMIT ${} OFFSET ${}", idx, idx + 1));

    let mut q = sqlx::query(&query);
    for b in &string_binds {
        q = q.bind(b);
    }
    q = q.bind(limit as i64).bind(offset as i64);

    let rows = q.fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_execution).collect())
}
