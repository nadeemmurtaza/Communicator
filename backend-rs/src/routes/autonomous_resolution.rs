//! Autonomous Resolution Routes — Phase 29
//!
//! API endpoints for managing autonomous resolution configuration,
//! escalation rules, agent policies, and viewing resolution analytics.

use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;
use crate::services::autonomous_resolution;

// ——— Query Params ———

#[derive(Deserialize)]
struct EscalationRulesQuery {
    is_active: Option<bool>,
}

#[derive(Deserialize)]
struct PoliciesQuery {
    is_active: Option<bool>,
}

#[derive(Deserialize)]
struct LogsQuery {
    outcome: Option<String>,
    escalated: Option<bool>,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Deserialize)]
struct EvaluateQuery {
    conversation_id: String,
    ai_confidence: f64,
    validation_score: Option<f64>,
    topic: Option<String>,
}

#[derive(Deserialize)]
struct LogAttemptQuery {
    conversation_id: String,
    contact_id: String,
    channel: String,
    ai_confidence: f64,
    validation_score: Option<f64>,
    validation_passed: Option<bool>,
}

#[derive(Deserialize)]
struct SnapshotQuery {
    days: Option<i32>,
}

// ——— Handlers ———

/// GET /autonomous-resolution/summary
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<autonomous_resolution::ResolutionSummary>, (StatusCode, String)> {
    let summary = autonomous_resolution::get_resolution_summary(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(summary))
}

/// GET /autonomous-resolution/config
pub async fn get_config(State(state): State<AppState>) -> Result<Json<autonomous_resolution::ResolutionConfig>, (StatusCode, String)> {
    let config = autonomous_resolution::get_resolution_config(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(config))
}

/// PUT /autonomous-resolution/config
pub async fn update_config(
    State(state): State<AppState>,
    Json(config): Json<autonomous_resolution::ResolutionConfig>,
) -> Result<Json<autonomous_resolution::ConfigResult>, (StatusCode, String)> {
    let result = autonomous_resolution::update_resolution_config(&state, &config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// GET /autonomous-resolution/escalation-rules
pub async fn list_escalation_rules(
    State(state): State<AppState>,
    Query(params): Query<EscalationRulesQuery>,
) -> Result<Json<Vec<autonomous_resolution::EscalationRule>>, (StatusCode, String)> {
    let rules = autonomous_resolution::list_escalation_rules(&state, params.is_active)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rules))
}

/// POST /autonomous-resolution/escalation-rules
pub async fn create_escalation_rule(
    State(state): State<AppState>,
    Json(rule): Json<autonomous_resolution::EscalationRule>,
) -> Result<Json<autonomous_resolution::EscalationResult>, (StatusCode, String)> {
    let result = autonomous_resolution::create_escalation_rule(
        &state,
        &rule.name,
        &rule.condition_type,
        rule.condition_value,
        &rule.action_type,
        rule.description.as_deref(),
        rule.condition_topic.as_deref(),
        rule.condition_keyword.as_deref(),
        rule.assign_to_agent_id.as_deref(),
        rule.assign_to_team_id.as_deref(),
        rule.notify_webhook.as_deref(),
        rule.notification_message.as_deref(),
        rule.priority,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// PUT /autonomous-resolution/escalation-rules/{id}
pub async fn update_escalation_rule(
    State(state): State<AppState>,
    Path(_id): Path<String>,
    Json(rule): Json<autonomous_resolution::EscalationRule>,
) -> Result<Json<autonomous_resolution::EscalationResult>, (StatusCode, String)> {
    let result = autonomous_resolution::update_escalation_rule(&state, &rule)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// DELETE /autonomous-resolution/escalation-rules/{id}
pub async fn delete_escalation_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<autonomous_resolution::EscalationResult>, (StatusCode, String)> {
    let result = autonomous_resolution::delete_escalation_rule(&state, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// GET /autonomous-resolution/policies
pub async fn list_policies(
    State(state): State<AppState>,
    Query(params): Query<PoliciesQuery>,
) -> Result<Json<Vec<autonomous_resolution::AgentPolicy>>, (StatusCode, String)> {
    let policies = autonomous_resolution::list_agent_policies(&state, params.is_active)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(policies))
}

/// POST /autonomous-resolution/evaluate
pub async fn evaluate_decision(
    State(state): State<AppState>,
    Query(params): Query<EvaluateQuery>,
) -> Result<Json<autonomous_resolution::ResolutionDecision>, (StatusCode, String)> {
    let decision = autonomous_resolution::evaluate_resolution_decision(
        &state,
        &params.conversation_id,
        params.ai_confidence,
        params.validation_score,
        params.topic.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(decision))
}

/// POST /autonomous-resolution/log-attempt
pub async fn log_attempt(
    State(state): State<AppState>,
    Json(params): Json<LogAttemptQuery>,
) -> Result<Json<autonomous_resolution::ResolutionLog>, (StatusCode, String)> {
    // First evaluate the decision
    let decision = autonomous_resolution::evaluate_resolution_decision(
        &state,
        &params.conversation_id,
        params.ai_confidence,
        params.validation_score,
        None,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Then log the attempt
    let log = autonomous_resolution::log_resolution_attempt(
        &state,
        &params.conversation_id,
        &params.contact_id,
        &params.channel,
        params.ai_confidence,
        params.validation_score,
        params.validation_passed,
        &decision,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(log))
}

/// GET /autonomous-resolution/logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogsQuery>,
) -> Result<Json<Vec<autonomous_resolution::ResolutionLog>>, (StatusCode, String)> {
    let logs = autonomous_resolution::list_resolution_logs(
        &state,
        params.outcome.as_deref(),
        params.escalated,
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(logs))
}

/// POST /autonomous-resolution/snapshot
pub async fn generate_snapshot(
    State(state): State<AppState>,
    Query(params): Query<SnapshotQuery>,
) -> Result<Json<autonomous_resolution::AnalyticsSnapshot>, (StatusCode, String)> {
    let days = params.days.unwrap_or(7);
    let snapshot = autonomous_resolution::generate_analytics_snapshot(&state, days)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(snapshot))
}
