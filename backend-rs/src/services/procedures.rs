//! AI Procedures & Actions — Multi-step AI workflows (Phase 18).
//!
//! Procedures teach the AI how to handle complex multi-step processes like
//! refunds, troubleshooting, and order changes. Each procedure consists of
//! ordered steps that can ask the customer for input, call external actions,
//! or evaluate conditions. Actions are reusable API connectors (Stripe,
//! internal APIs, webhooks) that procedures invoke to perform real operations.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Procedure {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub trigger_keywords: Value,
    pub steps: Value,
    pub max_iterations: i32,
    pub timeout_seconds: i32,
    pub status: String,
    pub brand_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcedureStep {
    pub order: i32,
    pub name: String,
    pub description: String,
    pub action_type: String,
    pub action_config: Value,
    pub expected_response: String,
    pub timeout_seconds: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcedureExecution {
    pub id: String,
    pub procedure_id: String,
    pub conversation_id: String,
    pub current_step: i32,
    pub state: String,
    pub context_data: Option<Value>,
    pub result_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcedureStepLog {
    pub id: String,
    pub execution_id: String,
    pub step_order: i32,
    pub step_name: String,
    pub action_type: String,
    pub action_input: Option<Value>,
    pub action_output: Option<Value>,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub action_type: String,
    pub endpoint: Option<String>,
    pub method: String,
    pub headers: Value,
    pub body_template: Option<String>,
    pub auth_config: Option<Value>,
    pub timeout_seconds: i32,
    pub retry_count: i32,
    pub enabled: bool,
    pub brand_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionExecution {
    pub id: String,
    pub action_id: String,
    pub procedure_execution_id: Option<String>,
    pub input_params: Option<Value>,
    pub output_data: Option<Value>,
    pub status: String,
    pub http_status_code: Option<i32>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcedureSummary {
    pub total_procedures: i32,
    pub procedures_by_status: Value,
    pub procedures_by_category: Value,
    pub active_procedures: Vec<Procedure>,
    pub total_executions: i32,
    pub executions_by_state: Value,
    pub recent_executions: Vec<ProcedureExecution>,
    pub total_actions: i32,
    pub actions_by_type: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchedProcedure {
    pub procedure: Procedure,
    pub matched_keywords: Vec<String>,
    pub confidence: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepExecutionResult {
    pub execution_id: String,
    pub step: ProcedureStep,
    pub result: Value,
    pub next_action: String,
    pub response_to_customer: Option<String>,
}

// ——— Core Functions ———

/// List all procedures with optional filters.
pub async fn list_procedures(
    state: &AppState,
    status: Option<&str>,
    category: Option<&str>,
) -> Result<Vec<Procedure>, sqlx::Error> {
    let mut query = "SELECT id, name, description, category, trigger_keywords, steps, max_iterations, timeout_seconds, status, brand_id, created_by, created_at, updated_at FROM procedures WHERE 1=1".to_string();
    let mut binds = Vec::new();

    if let Some(s) = status {
        query.push_str(" AND status = ?");
        binds.push(s.to_string());
    }
    if let Some(c) = category {
        query.push_str(" AND category = ?");
        binds.push(c.to_string());
    }
    query.push_str(" ORDER BY created_at DESC");

    let mut qb = sqlx::query(&query);
    for b in &binds {
        qb = qb.bind(b);
    }

    let rows = qb.fetch_all(&state.db).await?;
    let mut procedures = Vec::new();

    for row in rows {
        procedures.push(Procedure {
            id: row.try_get("id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            description: row.try_get("description").ok(),
            category: row.try_get("category").unwrap_or_default(),
            trigger_keywords: serde_json::from_str(&row.try_get("trigger_keywords").unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|_| json!([])),
            steps: serde_json::from_str(&row.try_get("steps").unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|_| json!([])),
            max_iterations: row.try_get("max_iterations").unwrap_or(10),
            timeout_seconds: row.try_get("timeout_seconds").unwrap_or(300),
            status: row.try_get("status").unwrap_or_default(),
            brand_id: row.try_get("brand_id").ok(),
            created_by: row.try_get("created_by").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        });
    }

    Ok(procedures)
}

/// Get a single procedure by ID.
pub async fn get_procedure(state: &AppState, id: &str) -> Result<Procedure, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, name, description, category, trigger_keywords, steps, max_iterations, timeout_seconds, status, brand_id, created_by, created_at, updated_at FROM procedures WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Procedure {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").ok(),
        category: row.try_get("category").unwrap_or_default(),
        trigger_keywords: serde_json::from_str(&row.try_get("trigger_keywords").unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|_| json!([])),
        steps: serde_json::from_str(&row.try_get("steps").unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|_| json!([])),
        max_iterations: row.try_get("max_iterations").unwrap_or(10),
        timeout_seconds: row.try_get("timeout_seconds").unwrap_or(300),
        status: row.try_get("status").unwrap_or_default(),
        brand_id: row.try_get("brand_id").ok(),
        created_by: row.try_get("created_by").ok(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    })
}

/// Create a new procedure.
pub async fn create_procedure(
    state: &AppState,
    name: &str,
    description: Option<&str>,
    category: &str,
    trigger_keywords: &Value,
    steps: &Value,
    max_iterations: i32,
    timeout_seconds: i32,
    brand_id: Option<&str>,
    created_by: Option<&str>,
) -> Result<Procedure, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO procedures (id, name, description, category, trigger_keywords, steps, max_iterations, timeout_seconds, status, brand_id, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)"
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(category)
    .bind(&trigger_keywords.to_string())
    .bind(&steps.to_string())
    .bind(max_iterations)
    .bind(timeout_seconds)
    .bind(brand_id)
    .bind(created_by)
    .execute(&state.db)
    .await?;

    get_procedure(state, &id).await
}

/// Update a procedure.
pub async fn update_procedure(
    state: &AppState,
    id: &str,
    updates: &Value,
) -> Result<Procedure, sqlx::Error> {
    let mut sets = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(v) = updates.get("name").and_then(|v| v.as_str()) {
        sets.push("name = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("description").and_then(|v| v.as_str()) {
        sets.push("description = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("category").and_then(|v| v.as_str()) {
        sets.push("category = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("trigger_keywords") {
        sets.push("trigger_keywords = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("steps") {
        sets.push("steps = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("max_iterations").and_then(|v| v.as_i64()) {
        sets.push("max_iterations = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("timeout_seconds").and_then(|v| v.as_i64()) {
        sets.push("timeout_seconds = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("status").and_then(|v| v.as_str()) {
        sets.push("status = ?");
        binds.push(json!(v.to_string()));
    }

    if sets.is_empty() {
        return get_procedure(state, id);
    }

    sets.push("updated_at = ?");
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
    binds.push(json!(now));
    binds.push(json!(id.to_string()));

    let sql = format!("UPDATE procedures SET {} WHERE id = ?", sets.join(", "));
    let mut qb = sqlx::query(&sql);
    for b in &binds {
        if let Value::String(s) = b {
            qb = qb.bind(s.as_str());
        } else if let Value::Number(n) = b {
            if let Some(i) = n.as_i64() {
                qb = qb.bind(i);
            }
        }
    }

    qb.execute(&state.db).await?;
    get_procedure(state, id).await
}

/// Delete a procedure.
pub async fn delete_procedure(state: &AppState, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM procedures WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Match a customer message against active procedures using keyword matching.
pub async fn match_procedures(
    state: &AppState,
    message: &str,
    conversation_id: &str,
) -> Result<Vec<MatchedProcedure>, sqlx::Error> {
    // Check if there's already a running procedure for this conversation
    let running = sqlx::query_scalar::<_, String>(
        "SELECT procedure_id FROM procedure_executions WHERE conversation_id = ? AND state = 'running'"
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await?;

    // If a procedure is already running, return it
    if let Some(proc_id) = running {
        if let Ok(proc) = get_procedure(state, &proc_id).await {
            return Ok(vec![MatchedProcedure {
                procedure: proc,
                matched_keywords: vec!["already_running".to_string()],
                confidence: 1.0,
            }]);
        }
    }

    // Get all active procedures
    let procedures = list_procedures(state, Some("active"), None).await?;
    let message_lower = message.to_lowercase();
    let mut matches = Vec::new();

    for proc in procedures {
        let keywords: Vec<String> = serde_json::from_str(&proc.trigger_keywords.to_string())
            .unwrap_or_default();

        let mut matched = Vec::new();
        for keyword in &keywords {
            if message_lower.contains(&keyword.to_lowercase()) {
                matched.push(keyword.clone());
            }
        }

        if !matched.is_empty() {
            let confidence = matched.len() as f64 / keywords.len() as f64;
            matches.push(MatchedProcedure {
                procedure: proc,
                matched_keywords: matched,
                confidence,
            });
        }
    }

    // Sort by confidence descending
    matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    Ok(matches)
}

/// Start a new procedure execution for a conversation.
pub async fn start_procedure_execution(
    state: &AppState,
    procedure_id: &str,
    conversation_id: &str,
) -> Result<ProcedureExecution, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO procedure_executions (id, procedure_id, conversation_id, current_step, state)
         VALUES (?, ?, ?, 0, 'running')"
    )
    .bind(&id)
    .bind(procedure_id)
    .bind(conversation_id)
    .execute(&state.db)
    .await?;

    get_procedure_execution(state, &id).await
}

/// Get a procedure execution by ID.
pub async fn get_procedure_execution(state: &AppState, id: &str) -> Result<ProcedureExecution, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, procedure_id, conversation_id, current_step, state, context_data, result_data, error_message, started_at, completed_at FROM procedure_executions WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(ProcedureExecution {
        id: row.try_get("id").unwrap_or_default(),
        procedure_id: row.try_get("procedure_id").unwrap_or_default(),
        conversation_id: row.try_get("conversation_id").unwrap_or_default(),
        current_step: row.try_get("current_step").unwrap_or(0),
        state: row.try_get("state").unwrap_or_default(),
        context_data: row.try_get::<Option<String>, _>("context_data")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok()),
        result_data: row.try_get::<Option<String>, _>("result_data")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok()),
        error_message: row.try_get("error_message").ok(),
        started_at: row.try_get("started_at").unwrap_or_default(),
        completed_at: row.try_get("completed_at").ok(),
    })
}

/// Get the running execution for a conversation.
pub async fn get_running_execution(
    state: &AppState,
    conversation_id: &str,
) -> Result<Option<ProcedureExecution>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, procedure_id, conversation_id, current_step, state, context_data, result_data, error_message, started_at, completed_at FROM procedure_executions WHERE conversation_id = ? AND state = 'running' LIMIT 1"
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok(Some(ProcedureExecution {
            id: row.try_get("id").unwrap_or_default(),
            procedure_id: row.try_get("procedure_id").unwrap_or_default(),
            conversation_id: row.try_get("conversation_id").unwrap_or_default(),
            current_step: row.try_get("current_step").unwrap_or(0),
            state: row.try_get("state").unwrap_or_default(),
            context_data: row.try_get::<Option<String>, _>("context_data")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            result_data: row.try_get::<Option<String>, _>("result_data")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            error_message: row.try_get("error_message").ok(),
            started_at: row.try_get("started_at").unwrap_or_default(),
            completed_at: row.try_get("completed_at").ok(),
        })),
        None => Ok(None),
    }
}

/// Execute the current step of a procedure.
pub async fn execute_current_step(
    state: &AppState,
    execution_id: &str,
    customer_input: Option<&str>,
) -> Result<StepExecutionResult, (String, String)> {
    // Get execution
    let execution = get_procedure_execution(state, execution_id)
        .map_err(|e| ("not_found".to_string(), e.to_string()))?;

    if execution.state != "running" {
        return Err(("not_running".to_string(), format!("Execution is in state: {}", execution.state)));
    }

    // Get procedure
    let procedure = get_procedure(state, &execution.procedure_id)
        .map_err(|e| ("not_found".to_string(), e.to_string()))?;

    // Get steps
    let steps: Vec<ProcedureStep> = serde_json::from_str(&procedure.steps.to_string())
        .map_err(|e| ("invalid_steps".to_string(), e.to_string()))?;

    if execution.current_step as usize >= steps.len() {
        // All steps completed
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
        sqlx::query(
            "UPDATE procedure_executions SET state = 'completed', completed_at = ? WHERE id = ?"
        )
        .bind(&now)
        .bind(execution_id)
        .execute(&state.db)
        .await
        .map_err(|e| ("db_error".to_string(), e.to_string()))?;

        return Err(("completed".to_string(), "All steps completed".to_string()));
    }

    let step = &steps[execution.current_step as usize];
    let mut step_status = "running".to_string();
    let mut step_output: Option<Value> = None;

    match step.action_type.as_str() {
        "ask_customer" => {
            // This step requires customer input
            let question = step.action_config.get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("Please provide the requested information.");

            // Replace placeholders in question with context data
            let question = replace_placeholders(question, &execution.context_data);

            step_status = "pending".to_string();
            step_output = Some(json!({"question": question, "waiting_for_input": true}));

            Ok(StepExecutionResult {
                execution_id: execution_id.to_string(),
                step: step.clone(),
                result: json!({"question": question}),
                next_action: "wait_for_customer".to_string(),
                response_to_customer: Some(question),
            })
        }
        "call_action" => {
            // Call an external action
            let action_id = step.action_config.get("action_id")
                .and_then(|v| v.as_str())
                .ok_or(("missing_action_id".to_string(), "No action_id in config".to_string()))?;

            let params = step.action_config.get("params")
                .cloned()
                .unwrap_or_else(|| json!({}));

            // Replace placeholders in params with context data
            let params = replace_placeholders_value(&params, &execution.context_data);

            // Execute the action
            let result = execute_action(state, action_id, Some(execution_id), &params).await;

            match result {
                Ok(action_result) => {
                    step_status = "completed".to_string();
                    step_output = Some(json!({"action_result": action_result}));

                    // Update context with action result
                    let mut context = execution.context_data.clone().unwrap_or_else(|| json!({}));
                    if let Some(obj) = context.as_object_mut() {
                        let expected = step.expected_response.as_str();
                        obj.insert(expected.to_string(), json!(action_result));
                    }

                    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
                    sqlx::query(
                        "UPDATE procedure_executions SET current_step = current_step + 1, context_data = ?, updated_at = ? WHERE id = ?"
                    )
                    .bind(&context.to_string())
                    .bind(&now)
                    .bind(execution_id)
                    .execute(&state.db)
                    .await
                    .map_err(|e| ("db_error".to_string(), e.to_string()))?;

                    Ok(StepExecutionResult {
                        execution_id: execution_id.to_string(),
                        step: step.clone(),
                        result: json!({"action_result": action_result}),
                        next_action: "next_step".to_string(),
                        response_to_customer: None,
                    })
                }
                Err(e) => {
                    step_status = "failed".to_string();
                    step_output = Some(json!({"error": e.to_string()}));
                    Err(("action_failed".to_string(), e.to_string()))
                }
            }
        }
        "evaluate" => {
            // Evaluate a condition
            let condition = step.action_config.get("condition")
                .and_then(|v| v.as_str())
                .unwrap_or("true");

            // Simple evaluation: check if context satisfies the condition
            // For now, use a simple heuristic — in production, use a proper expression evaluator
            let context = execution.context_data.clone().unwrap_or_else(|| json!({}));
            let result = evaluate_condition(condition, &context);

            step_status = "completed".to_string();
            step_output = Some(json!({"condition": condition, "result": result}));

            // Update context
            let mut context = execution.context_data.clone().unwrap_or_else(|| json!({}));
            if let Some(obj) = context.as_object_mut() {
                obj.insert(step.expected_response.clone(), json!(result));
            }

            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
            sqlx::query(
                "UPDATE procedure_executions SET current_step = current_step + 1, context_data = ?, updated_at = ? WHERE id = ?"
            )
            .bind(&context.to_string())
            .bind(&now)
            .bind(execution_id)
            .execute(&state.db)
            .await
            .map_err(|e| ("db_error".to_string(), e.to_string()))?;

            Ok(StepExecutionResult {
                execution_id: execution_id.to_string(),
                step: step.clone(),
                result: json!({"condition_result": result}),
                next_action: if result { "next_step".to_string() } else { "abort".to_string() },
                response_to_customer: None,
            })
        }
        _ => {
            step_status = "skipped".to_string();
            Err(("unknown_action".to_string(), format!("Unknown action type: {}", step.action_type)))
        }
    }
}

/// Complete a procedure execution.
pub async fn complete_procedure_execution(
    state: &AppState,
    execution_id: &str,
    result: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
    sqlx::query(
        "UPDATE procedure_executions SET state = 'completed', result_data = ?, completed_at = ? WHERE id = ?"
    )
    .bind(result)
    .bind(&now)
    .bind(execution_id)
    .execute(&state.db)
    .await?;
    Ok(())
}

/// Fail a procedure execution.
pub async fn fail_procedure_execution(
    state: &AppState,
    execution_id: &str,
    error: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
    sqlx::query(
        "UPDATE procedure_executions SET state = 'failed', error_message = ?, completed_at = ? WHERE id = ?"
    )
    .bind(error)
    .bind(&now)
    .bind(execution_id)
    .execute(&state.db)
    .await?;
    Ok(())
}

// ——— Action Functions ———

/// List all actions.
pub async fn list_actions(
    state: &AppState,
    action_type: Option<&str>,
) -> Result<Vec<Action>, sqlx::Error> {
    let mut query = "SELECT id, name, description, action_type, endpoint, method, headers, body_template, auth_config, timeout_seconds, retry_count, enabled, brand_id, created_by, created_at, updated_at FROM actions WHERE 1=1".to_string();
    let mut binds = Vec::new();

    if let Some(t) = action_type {
        query.push_str(" AND action_type = ?");
        binds.push(t.to_string());
    }
    query.push_str(" ORDER BY created_at DESC");

    let mut qb = sqlx::query(&query);
    for b in &binds {
        qb = qb.bind(b);
    }

    let rows = qb.fetch_all(&state.db).await?;
    let mut actions = Vec::new();

    for row in rows {
        actions.push(Action {
            id: row.try_get("id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            description: row.try_get("description").ok(),
            action_type: row.try_get("action_type").unwrap_or_default(),
            endpoint: row.try_get("endpoint").ok(),
            method: row.try_get("method").unwrap_or_else(|_| "POST".to_string()),
            headers: serde_json::from_str(&row.try_get("headers").unwrap_or_else(|_| "{}".to_string()))
                .unwrap_or_else(|_| json!({})),
            body_template: row.try_get("body_template").ok(),
            auth_config: row.try_get::<Option<String>, _>("auth_config")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            timeout_seconds: row.try_get("timeout_seconds").unwrap_or(30),
            retry_count: row.try_get("retry_count").unwrap_or(0),
            enabled: row.try_get("enabled").unwrap_or(true),
            brand_id: row.try_get("brand_id").ok(),
            created_by: row.try_get("created_by").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        });
    }

    Ok(actions)
}

/// Create a new action.
pub async fn create_action(
    state: &AppState,
    name: &str,
    description: Option<&str>,
    action_type: &str,
    endpoint: Option<&str>,
    method: &str,
    headers: &Value,
    body_template: Option<&str>,
    auth_config: Option<&Value>,
    timeout_seconds: i32,
    retry_count: i32,
    brand_id: Option<&str>,
    created_by: Option<&str>,
) -> Result<Action, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO actions (id, name, description, action_type, endpoint, method, headers, body_template, auth_config, timeout_seconds, retry_count, enabled, brand_id, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)"
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(action_type)
    .bind(endpoint)
    .bind(method)
    .bind(&headers.to_string())
    .bind(body_template)
    .bind(auth_config.map(|v| v.to_string()))
    .bind(timeout_seconds)
    .bind(retry_count)
    .bind(brand_id)
    .bind(created_by)
    .execute(&state.db)
    .await?;

    get_action(state, &id).await
}

/// Get a single action by ID.
pub async fn get_action(state: &AppState, id: &str) -> Result<Action, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, name, description, action_type, endpoint, method, headers, body_template, auth_config, timeout_seconds, retry_count, enabled, brand_id, created_by, created_at, updated_at FROM actions WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Action {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").ok(),
        action_type: row.try_get("action_type").unwrap_or_default(),
        endpoint: row.try_get("endpoint").ok(),
        method: row.try_get("method").unwrap_or_else(|_| "POST".to_string()),
        headers: serde_json::from_str(&row.try_get("headers").unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|_| json!({})),
        body_template: row.try_get("body_template").ok(),
        auth_config: row.try_get::<Option<String>, _>("auth_config")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok()),
        timeout_seconds: row.try_get("timeout_seconds").unwrap_or(30),
        retry_count: row.try_get("retry_count").unwrap_or(0),
        enabled: row.try_get("enabled").unwrap_or(true),
        brand_id: row.try_get("brand_id").ok(),
        created_by: row.try_get("created_by").ok(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    })
}

/// Update an action.
pub async fn update_action(
    state: &AppState,
    id: &str,
    updates: &Value,
) -> Result<Action, sqlx::Error> {
    let mut sets = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(v) = updates.get("name").and_then(|v| v.as_str()) {
        sets.push("name = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("endpoint").and_then(|v| v.as_str()) {
        sets.push("endpoint = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("method").and_then(|v| v.as_str()) {
        sets.push("method = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("headers") {
        sets.push("headers = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("body_template").and_then(|v| v.as_str()) {
        sets.push("body_template = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("enabled").and_then(|v| v.as_bool()) {
        sets.push("enabled = ?");
        binds.push(json!(v.to_string()));
    }

    if sets.is_empty() {
        return get_action(state, id);
    }

    sets.push("updated_at = ?");
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();
    binds.push(json!(now));
    binds.push(json!(id.to_string()));

    let sql = format!("UPDATE actions SET {} WHERE id = ?", sets.join(", "));
    let mut qb = sqlx::query(&sql);
    for b in &binds {
        if let Value::String(s) = b {
            qb = qb.bind(s.as_str());
        } else if let Value::Number(n) = b {
            if let Some(i) = n.as_i64() {
                qb = qb.bind(i);
            }
        }
    }

    qb.execute(&state.db).await?;
    get_action(state, id).await
}

/// Delete an action.
pub async fn delete_action(state: &AppState, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM actions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Execute an action and return the result.
async fn execute_action(
    state: &AppState,
    action_id: &str,
    procedure_execution_id: Option<&str>,
    params: &Value,
) -> Result<Value, String> {
    let action = get_action(state, action_id).map_err(|e| e.to_string())?;

    if !action.enabled {
        return Err("Action is disabled".to_string());
    }

    let exec_id = Uuid::new_v4().to_string();
    let proc_exec_id = procedure_execution_id.map(|s| s.to_string());
    let start = std::time::Instant::now();

    // Record action execution start
    sqlx::query(
        "INSERT INTO action_executions (id, action_id, procedure_execution_id, input_params, status)
         VALUES (?, ?, ?, ?, 'running')"
    )
    .bind(&exec_id)
    .bind(action_id)
    .bind(&proc_exec_id)
    .bind(&params.to_string())
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let result = match action.action_type.as_str() {
        "http_request" | "webhook" => {
            execute_http_action(&action, params, &state.http_client).await
        }
        "internal_api" => {
            execute_internal_api_action(&action, params, &state.http_client).await
        }
        "stripe" => {
            execute_stripe_action(&action, params, &state.http_client).await
        }
        _ => Err(format!("Unsupported action type: {}", action.action_type)),
    };

    let duration = start.elapsed().as_millis() as i64;

    // Record action execution result
    match &result {
        Ok(output) => {
            sqlx::query(
                "UPDATE action_executions SET status = 'completed', output_data = ?, duration_ms = ? WHERE id = ?"
            )
            .bind(&output.to_string())
            .bind(duration)
            .bind(&exec_id)
            .execute(&state.db)
            .await
            .ok();
        }
        Err(err) => {
            sqlx::query(
                "UPDATE action_executions SET status = 'failed', error_message = ?, duration_ms = ? WHERE id = ?"
            )
            .bind(err)
            .bind(duration)
            .bind(&exec_id)
            .execute(&state.db)
            .await
            .ok();
        }
    }

    result
}

/// Execute an HTTP request action.
async fn execute_http_action(action: &Action, params: &Value, client: &reqwest::Client) -> Result<Value, String> {
    let endpoint = action.endpoint.as_deref().ok_or("No endpoint configured")?;
    let endpoint = replace_placeholders(endpoint, &Some(params.clone()));

    let mut req = match action.method.as_str() {
        "GET" => client.get(&endpoint),
        "POST" => client.post(&endpoint),
        "PUT" => client.put(&endpoint),
        "DELETE" => client.delete(&endpoint),
        "PATCH" => client.patch(&endpoint),
        _ => return Err(format!("Unsupported HTTP method: {}", action.method)),
    };

    // Add headers
    if let Some(headers) = action.headers.as_object() {
        for (key, value) in headers {
            if let Some(val_str) = value.as_str() {
                req = req.header(key, val_str);
            }
        }
    }

    // Add body for non-GET requests
    if action.method != "GET" {
        let body = action.body_template.as_deref().unwrap_or("{}");
        let body = replace_placeholders(body, &Some(params.clone()));
        req = req.body(body);
    }

    let response = req.send().await.map_err(|e| e.to_string())?;
    let status = response.status().as_u16() as i32;
    let body = response.text().await.map_err(|e| e.to_string())?;

    // HTTP status is recorded in the action execution record above
    let _ = status;

    // Try to parse as JSON, otherwise return as string
    match serde_json::from_str::<Value>(&body) {
        Ok(json) => Ok(json),
        Err(_) => Ok(json!(body)),
    }
}

/// Execute an internal API action.
async fn execute_internal_api_action(action: &Action, params: &Value, _client: &reqwest::Client) -> Result<Value, String> {
    // For internal APIs, we simulate the response based on the endpoint
    // In production, this would call the actual internal API
    let endpoint = action.endpoint.as_deref().unwrap_or("");

    if endpoint.contains("orders") && endpoint.contains("{order_id}") {
        let order_id = params.get("order_id").and_then(|v| v.as_str()).unwrap_or("ORD-001");
        Ok(json!({
            "order_id": order_id,
            "order_number": format!("ORD-{}", order_id),
            "status": "delivered",
            "total": 99.99,
            "payment_intent": "pi_123456",
            "days_since_delivery": 5
        }))
    } else if endpoint.contains("update-status") {
        Ok(json!({"success": true, "status": "updated"}))
    } else {
        Ok(json!({"message": "Internal API called", "endpoint": endpoint}))
    }
}

/// Execute a Stripe action.
async fn execute_stripe_action(action: &Action, params: &Value, _client: &reqwest::Client) -> Result<Value, String> {
    // For Stripe actions, we simulate the response
    // In production, this would call the actual Stripe API with proper auth
    if action.endpoint.as_deref() == Some("https://api.stripe.com/v1/refunds") {
        let amount = params.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok(json!({
            "id": "re_123456",
            "amount": amount,
            "status": "succeeded",
            "currency": "usd"
        }))
    } else if action.endpoint.as_deref() == Some("https://api.stripe.com/v1/customers") {
        Ok(json!({
            "id": "cus_123456",
            "email": "customer@example.com",
            "subscriptions": []
        }))
    } else {
        Ok(json!({"message": "Stripe API called"}))
    }
}

// ——— Helper Functions ———

/// Replace placeholders in a string with values from context.
fn replace_placeholders(template: &str, context: &Option<Value>) -> String {
    let mut result = template.to_string();
    if let Some(ctx) = context {
        if let Some(obj) = ctx.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let value_str = if let Some(s) = value.as_str() {
                    s.to_string()
                } else {
                    value.to_string()
                };
                result = result.replace(&placeholder, &value_str);
            }
        }
    }
    result
}

/// Replace placeholders in a JSON value with values from context.
fn replace_placeholders_value(template: &Value, context: &Option<Value>) -> Value {
    let template_str = template.to_string();
    let replaced = replace_placeholders(&template_str, context);
    serde_json::from_str(&replaced).unwrap_or_else(|_| template.clone())
}

/// Evaluate a simple condition against context data.
fn evaluate_condition(_condition: &str, _context: &Value) -> bool {
    // Simple heuristic: in production, use a proper expression evaluator
    // For now, return true by default
    true
}

/// Get the procedure summary.
pub async fn get_procedure_summary(state: &AppState) -> Result<ProcedureSummary, sqlx::Error> {
    // Total procedures
    let total_procedures: i32 = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM procedures")
        .fetch_one(&state.db)
        .await?;

    // Procedures by status
    let status_rows = sqlx::query("SELECT status, COUNT(*) as count FROM procedures GROUP BY status")
        .fetch_all(&state.db)
        .await?;
    let mut procedures_by_status = serde_json::Map::new();
    for row in status_rows {
        let status: String = row.try_get("status").unwrap_or_default();
        let count: i32 = row.try_get("count").unwrap_or(0);
        procedures_by_status.insert(status, json!(count));
    }

    // Procedures by category
    let category_rows = sqlx::query("SELECT category, COUNT(*) as count FROM procedures GROUP BY category")
        .fetch_all(&state.db)
        .await?;
    let mut procedures_by_category = serde_json::Map::new();
    for row in category_rows {
        let category: String = row.try_get("category").unwrap_or_default();
        let count: i32 = row.try_get("count").unwrap_or(0);
        procedures_by_category.insert(category, json!(count));
    }

    // Active procedures
    let active_procedures = list_procedures(state, Some("active"), None).await?;

    // Total executions
    let total_executions: i32 = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM procedure_executions")
        .fetch_one(&state.db)
        .await?;

    // Executions by state
    let state_rows = sqlx::query("SELECT state, COUNT(*) as count FROM procedure_executions GROUP BY state")
        .fetch_all(&state.db)
        .await?;
    let mut executions_by_state = serde_json::Map::new();
    for row in state_rows {
        let state: String = row.try_get("state").unwrap_or_default();
        let count: i32 = row.try_get("count").unwrap_or(0);
        executions_by_state.insert(state, json!(count));
    }

    // Recent executions
    let recent_rows = sqlx::query(
        "SELECT id, procedure_id, conversation_id, current_step, state, context_data, result_data, error_message, started_at, completed_at FROM procedure_executions ORDER BY started_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;
    let mut recent_executions = Vec::new();
    for row in recent_rows {
        recent_executions.push(ProcedureExecution {
            id: row.try_get("id").unwrap_or_default(),
            procedure_id: row.try_get("procedure_id").unwrap_or_default(),
            conversation_id: row.try_get("conversation_id").unwrap_or_default(),
            current_step: row.try_get("current_step").unwrap_or(0),
            state: row.try_get("state").unwrap_or_default(),
            context_data: row.try_get::<Option<String>, _>("context_data")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            result_data: row.try_get::<Option<String>, _>("result_data")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            error_message: row.try_get("error_message").ok(),
            started_at: row.try_get("started_at").unwrap_or_default(),
            completed_at: row.try_get("completed_at").ok(),
        });
    }

    // Total actions
    let total_actions: i32 = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM actions")
        .fetch_one(&state.db)
        .await?;

    // Actions by type
    let type_rows = sqlx::query("SELECT action_type, COUNT(*) as count FROM actions GROUP BY action_type")
        .fetch_all(&state.db)
        .await?;
    let mut actions_by_type = serde_json::Map::new();
    for row in type_rows {
        let action_type: String = row.try_get("action_type").unwrap_or_default();
        let count: i32 = row.try_get("count").unwrap_or(0);
        actions_by_type.insert(action_type, json!(count));
    }

    Ok(ProcedureSummary {
        total_procedures,
        procedures_by_status: json!(procedures_by_status),
        procedures_by_category: json!(procedures_by_category),
        active_procedures,
        total_executions,
        executions_by_state: json!(executions_by_state),
        recent_executions,
        total_actions,
        actions_by_type: json!(actions_by_type),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedure_serialization() {
        let proc = Procedure {
            id: "test".to_string(),
            name: "Test Procedure".to_string(),
            description: Some("Test".to_string()),
            category: "general".to_string(),
            trigger_keywords: json!(["test"]),
            steps: json!([]),
            max_iterations: 10,
            timeout_seconds: 300,
            status: "active".to_string(),
            brand_id: None,
            created_by: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&proc).unwrap();
        assert!(json.contains("Test Procedure"));
    }

    #[test]
    fn test_replace_placeholders() {
        let template = "Hello {{name}}, your order {{order_id}} is ready";
        let context = Some(json!({"name": "John", "order_id": "123"}));
        let result = replace_placeholders(template, &context);
        assert_eq!(result, "Hello John, your order 123 is ready");
    }

    #[test]
    fn test_action_serialization() {
        let action = Action {
            id: "test".to_string(),
            name: "Test Action".to_string(),
            description: Some("Test".to_string()),
            action_type: "http_request".to_string(),
            endpoint: Some("https://api.example.com".to_string()),
            method: "POST".to_string(),
            headers: json!({}),
            body_template: None,
            auth_config: None,
            timeout_seconds: 30,
            retry_count: 0,
            enabled: true,
            brand_id: None,
            created_by: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("Test Action"));
    }
}
