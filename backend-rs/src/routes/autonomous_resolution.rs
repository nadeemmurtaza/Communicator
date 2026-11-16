//! Thin gRPC client forwarding autonomous resolution requests to `svc-autonomous-resolution` (port 50091).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::autonomous_resolution::v1::{
    autonomous_resolution_service_client::AutonomousResolutionServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("AUTONOMOUS_RESOLUTION_GRPC").unwrap_or("http://127.0.0.1:50091")
}

async fn client() -> AutonomousResolutionServiceClient<tonic::transport::Channel> {
    AutonomousResolutionServiceClient::connect(svc_addr().to_string())
        .await
        .expect("autonomous-resolution gRPC service unavailable")
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
pub struct EscalationRulesQuery {
    pub is_active: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct PoliciesQuery {
    pub is_active: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct LogsQuery {
    pub outcome: Option<String>,
    pub escalated: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct EvaluateQuery {
    pub conversation_id: String,
    pub ai_confidence: f64,
    pub validation_score: Option<f64>,
    pub topic: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LogAttemptQuery {
    pub conversation_id: String,
    pub contact_id: String,
    pub channel: String,
    pub ai_confidence: f64,
    pub validation_score: Option<f64>,
    pub validation_passed: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct SnapshotQuery {
    pub days: Option<i32>,
}

pub async fn get_summary(State(_state): State<AppState>) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_config(State(_state): State<AppState>) -> Response {
    match client().await.get_config(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn update_config(
    State(_state): State<AppState>,
    Json(config): Json<Value>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::UpdateConfigRequest::default();
    req.config_json = serde_json::to_string(&config).unwrap_or_default();
    match client().await.update_config(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_escalation_rules(
    State(_state): State<AppState>,
    Query(params): Query<EscalationRulesQuery>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::ListEscalationRulesRequest::default();
    req.has_is_active = params.is_active.is_some();
    req.is_active = params.is_active.unwrap_or_default();
    match client().await.list_escalation_rules(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn create_escalation_rule(
    State(_state): State<AppState>,
    Json(rule): Json<Value>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::CreateEscalationRuleRequest::default();
    req.name = rule.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.condition_type = rule.get("condition_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.condition_value = rule.get("condition_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
    req.action_type = rule.get("action_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.description = rule.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.condition_topic = rule.get("condition_topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.condition_keyword = rule.get("condition_keyword").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.assign_to_agent_id = rule.get("assign_to_agent_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.assign_to_team_id = rule.get("assign_to_team_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.notify_webhook = rule.get("notify_webhook").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.notification_message = rule.get("notification_message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    req.priority = rule.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    match client().await.create_escalation_rule(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn update_escalation_rule(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(rule): Json<Value>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::UpdateEscalationRuleRequest::default();
    req.rule_json = serde_json::to_string(&rule).unwrap_or_default();
    match client().await.update_escalation_rule(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn delete_escalation_rule(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::DeleteEscalationRuleRequest::default();
    req.rule_id = id;
    match client().await.delete_escalation_rule(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_policies(
    State(_state): State<AppState>,
    Query(params): Query<PoliciesQuery>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::ListPoliciesRequest::default();
    req.has_is_active = params.is_active.is_some();
    req.is_active = params.is_active.unwrap_or_default();
    match client().await.list_policies(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn evaluate_decision(
    State(_state): State<AppState>,
    Query(params): Query<EvaluateQuery>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::EvaluateDecisionRequest::default();
    req.conversation_id = params.conversation_id;
    req.ai_confidence = params.ai_confidence;
    req.has_validation_score = params.validation_score.is_some();
    req.validation_score = params.validation_score.unwrap_or(0.0);
    req.topic = params.topic.unwrap_or_default();
    match client().await.evaluate_decision(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn log_attempt(
    State(_state): State<AppState>,
    Json(params): Json<LogAttemptQuery>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::LogAttemptRequest::default();
    req.conversation_id = params.conversation_id;
    req.contact_id = params.contact_id;
    req.channel = params.channel;
    req.ai_confidence = params.ai_confidence;
    req.has_validation_score = params.validation_score.is_some();
    req.validation_score = params.validation_score.unwrap_or(0.0);
    req.has_validation_passed = params.validation_passed.is_some();
    req.validation_passed = params.validation_passed.unwrap_or_default();
    match client().await.log_attempt(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_logs(
    State(_state): State<AppState>,
    Query(params): Query<LogsQuery>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::ListLogsRequest::default();
    req.outcome = params.outcome.clone().unwrap_or_default();
    req.has_escalated = params.escalated.is_some();
    req.escalated = params.escalated.unwrap_or_default();
    req.limit = params.limit.unwrap_or(50);
    req.offset = params.offset.unwrap_or(0);
    match client().await.list_logs(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn generate_snapshot(
    State(_state): State<AppState>,
    Query(params): Query<SnapshotQuery>,
) -> Response {
    let mut req = newax_proto::autonomous_resolution::v1::GenerateSnapshotRequest::default();
    req.days = params.days.unwrap_or(7);
    match client().await.generate_snapshot(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}
