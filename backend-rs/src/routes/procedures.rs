//! AI Procedures & Actions routes (Phase 18).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use sqlx::Row;

use crate::db::AppState;
use crate::services::procedures;

/// GET /procedures/summary — Get procedure and actions summary
pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let summary = procedures::get_procedure_summary(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

/// GET /procedures — List procedures with optional filters
pub async fn list_procedures(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let status = params.get("status").map(|s| s.as_str());
    let category = params.get("category").map(|s| s.as_str());
    let procs = procedures::list_procedures(&state, status, category)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(procs).unwrap_or_default()))
}

/// POST /procedures — Create a new procedure
pub async fn create_procedure(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let name = body.get("name").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "name is required".to_string()))?;
    let description = body.get("description").and_then(|v| v.as_str());
    let category = body.get("category").and_then(|v| v.as_str()).unwrap_or("general");
    let trigger_keywords = body.get("trigger_keywords").cloned().unwrap_or_else(|| json!([]));
    let steps = body.get("steps").cloned().unwrap_or_else(|| json!([]));
    let max_iterations = body.get("max_iterations").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
    let timeout_seconds = body.get("timeout_seconds").and_then(|v| v.as_i64()).unwrap_or(300) as i32;
    let brand_id = body.get("brand_id").and_then(|v| v.as_str());
    let created_by = body.get("created_by").and_then(|v| v.as_str());

    let proc = procedures::create_procedure(&state, name, description, category, &trigger_keywords, &steps, max_iterations, timeout_seconds, brand_id, created_by)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(proc).unwrap_or_default()))
}

/// GET /procedures/{id} — Get a procedure
pub async fn get_procedure(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let proc = procedures::get_procedure(&state, &id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(serde_json::to_value(proc).unwrap_or_default()))
}

/// PUT /procedures/{id} — Update a procedure
pub async fn update_procedure(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let proc = procedures::update_procedure(&state, &id, &body)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(proc).unwrap_or_default()))
}

/// DELETE /procedures/{id} — Delete a procedure
pub async fn delete_procedure(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    procedures::delete_procedure(&state, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /procedures/match — Match a message against procedures
pub async fn match_procedures(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let message = body.get("message").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "message is required".to_string()))?;
    let conversation_id = body.get("conversation_id").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "conversation_id is required".to_string()))?;
    let matches = procedures::match_procedures(&state, message, conversation_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(matches).unwrap_or_default()))
}

/// POST /procedures/{id}/execute — Start a procedure execution
pub async fn start_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conversation_id = body.get("conversation_id").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "conversation_id is required".to_string()))?;
    let execution = procedures::start_procedure_execution(&state, &id, conversation_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(execution).unwrap_or_default()))
}

/// GET /procedures/executions/{id} — Get execution details
pub async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let execution = procedures::get_procedure_execution(&state, &id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(serde_json::to_value(execution).unwrap_or_default()))
}

/// GET /procedures/executions/conversation/{id} — Get running execution for conversation
pub async fn get_running_execution(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let execution = procedures::get_running_execution(&state, &conversation_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(execution).unwrap_or_default()))
}

/// POST /procedures/executions/{id}/step — Execute current step
pub async fn execute_step(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer_input = body.get("customer_input").and_then(|v| v.as_str());
    let result = procedures::execute_current_step(&state, &id, customer_input)
        .await
        .map_err(|(code, msg)| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", code, msg)))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// POST /procedures/executions/{id}/complete — Complete an execution
pub async fn complete_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = body.get("result").and_then(|v| v.as_str()).unwrap_or("completed");
    procedures::complete_procedure_execution(&state, &id, result)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

/// POST /procedures/executions/{id}/fail — Fail an execution
pub async fn fail_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let error = body.get("error").and_then(|v| v.as_str()).unwrap_or("failed");
    procedures::fail_procedure_execution(&state, &id, error)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

// ——— Actions ———

/// GET /actions — List actions
pub async fn list_actions(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let action_type = params.get("action_type").map(|s| s.as_str());
    let actions = procedures::list_actions(&state, action_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(actions).unwrap_or_default()))
}

/// POST /actions — Create an action
pub async fn create_action(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let name = body.get("name").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "name is required".to_string()))?;
    let description = body.get("description").and_then(|v| v.as_str());
    let action_type = body.get("action_type").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "action_type is required".to_string()))?;
    let endpoint = body.get("endpoint").and_then(|v| v.as_str());
    let method = body.get("method").and_then(|v| v.as_str()).unwrap_or("POST");
    let headers = body.get("headers").cloned().unwrap_or_else(|| json!({}));
    let body_template = body.get("body_template").and_then(|v| v.as_str());
    let auth_config = body.get("auth_config").cloned();
    let timeout_seconds = body.get("timeout_seconds").and_then(|v| v.as_i64()).unwrap_or(30) as i32;
    let retry_count = body.get("retry_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let brand_id = body.get("brand_id").and_then(|v| v.as_str());
    let created_by = body.get("created_by").and_then(|v| v.as_str());

    let action = procedures::create_action(&state, name, description, action_type, endpoint, method, &headers, body_template, auth_config, timeout_seconds, retry_count, brand_id, created_by)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}

/// GET /actions/{id} — Get an action
pub async fn get_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let action = procedures::get_action(&state, &id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}

/// PUT /actions/{id} — Update an action
pub async fn update_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let action = procedures::update_action(&state, &id, &body)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}

/// DELETE /actions/{id} — Delete an action
pub async fn delete_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    procedures::delete_action(&state, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /actions/{id}/test — Test an action
pub async fn test_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let params = body.get("params").cloned().unwrap_or_else(|| json!({}));
    // We need to expose execute_action or create a test endpoint
    // For now, return the action details
    let action = procedures::get_action(&state, &id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}
