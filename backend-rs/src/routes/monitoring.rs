#![allow(dead_code)]

//! Monitoring & Alerting API routes -- thin gRPC client for
//! svc-monitoring. See crates/monitoring and services/monitoring.
//! REST paths and payload shapes are unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use newax_proto::monitoring::v1::{
    monitoring_service_client::MonitoringServiceClient, AcknowledgeRequest, AlertIdRequest, AlertsRequest, Empty,
    MetricsRequest, RuleIdRequest,
};
use proto::{CreateAlertRuleRequest as ProtoCreateReq, UpdateAlertRuleRequest as ProtoUpdateReq};
use newax_proto::monitoring::v1 as proto;
use serde::Deserialize;
use serde_json::json;
use tonic::transport::Channel;

use crate::db::AppState;

async fn client() -> Result<MonitoringServiceClient<Channel>, (StatusCode, String)> {
    let url = std::env::var("MONITORING_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50066".to_string());
    MonitoringServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("monitoring service unreachable: {e}")))
}

fn grpc_err(status: tonic::Status) -> Response {
    let code = match status.code() {
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, Json(json!({ "error": status.message() }))).into_response()
}

use axum::response::Response;

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub name: Option<String>,
    pub window_hours: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub severity: Option<String>,
    pub acknowledged: Option<bool>,
    pub resolved: Option<bool>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    pub window_minutes: Option<i64>,
    pub severity: Option<String>,
    pub notification_webhook: Option<String>,
    pub cooldown_minutes: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metric_name: Option<String>,
    pub condition: Option<String>,
    pub threshold: Option<f64>,
    pub window_minutes: Option<i64>,
    pub severity: Option<String>,
    pub enabled: Option<i32>,
    pub notification_webhook: Option<String>,
    pub cooldown_minutes: Option<i64>,
}

pub async fn health_check_extended(State(_s): State<AppState>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response(),
    };
    match c.health_check_extended(Empty {}).await {
        Ok(r) => {
            let health = r.into_inner();
            let status_code = match health.overall_status.as_str() {
                "healthy" => StatusCode::OK,
                "degraded" | "unhealthy" => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status_code, Json(json!({
                "overall_status": health.overall_status,
                "components": health.components.into_iter().map(|c| json!({"name": c.name, "status": c.status, "detail": c.detail})).collect::<Vec<_>>(),
                "checked_at": health.checked_at,
            }))).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn get_metrics(State(_s): State<AppState>, Query(params): Query<MetricsQuery>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    let req = MetricsRequest { name: params.name, window_hours: params.window_hours.unwrap_or(24), limit: params.limit.unwrap_or(100) };
    match c.get_metrics(req).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({
                "metrics": r.metrics.into_iter().map(|x| json!({
                    "id": x.id, "metric_name": x.metric_name, "metric_value": x.metric_value,
                    "metric_type": x.metric_type, "labels": x.labels, "collected_at": x.collected_at, "created_at": x.created_at,
                })).collect::<Vec<_>>(),
                "window_hours": r.window_hours,
                "count": r.count,
            })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn get_metrics_summary(State(_s): State<AppState>, Query(params): Query<MetricsQuery>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    let req = MetricsRequest { name: params.name, window_hours: params.window_hours.unwrap_or(24), limit: params.limit.unwrap_or(100) };
    match c.get_metrics_summary(req).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({
                "summaries": r.summaries.into_iter().map(|x| json!({
                    "metric_name": x.metric_name, "avg_value": x.avg_value, "min_value": x.min_value,
                    "max_value": x.max_value, "latest_value": x.latest_value, "sample_count": x.sample_count,
                })).collect::<Vec<_>>(),
                "window_hours": r.window_hours,
                "count": r.count,
            })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn collect_metrics(State(_s): State<AppState>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.collect_metrics(Empty {}).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({ "collected": r.collected, "alerts_fired": r.alerts_fired, "collected_at": r.collected_at })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn get_alert_events(State(_s): State<AppState>, Query(params): Query<AlertsQuery>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    let req = AlertsRequest { severity: params.severity, acknowledged: params.acknowledged, resolved: params.resolved, limit: params.limit.unwrap_or(50) };
    match c.get_alert_events(req).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({
                "alerts": r.alerts.into_iter().map(|x| json!({
                    "id": x.id, "rule_id": x.rule_id, "alert_name": x.alert_name, "severity": x.severity,
                    "metric_name": x.metric_name, "metric_value": x.metric_value, "threshold": x.threshold,
                    "condition": x.condition, "message": x.message, "acknowledged": x.acknowledged,
                    "acknowledged_by": x.acknowledged_by, "acknowledged_at": x.acknowledged_at,
                    "resolved": x.resolved, "resolved_at": x.resolved_at, "created_at": x.created_at,
                })).collect::<Vec<_>>(),
                "count": r.count,
            })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn acknowledge_alert(State(_s): State<AppState>, Path(alert_id): Path<String>, Json(req): Json<AcknowledgeAlertRequest>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.acknowledge_alert(AcknowledgeRequest { alert_id, acknowledged_by: req.acknowledged_by }).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({ "acknowledged": r.acknowledged, "alert_id": r.alert_id, "acknowledged_by": r.acknowledged_by, "acknowledged_at": r.acknowledged_at })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn resolve_alert(State(_s): State<AppState>, Path(alert_id): Path<String>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.resolve_alert(AlertIdRequest { alert_id }).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({ "resolved": r.resolved, "alert_id": r.alert_id, "resolved_at": r.resolved_at })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn get_alert_rules(State(_s): State<AppState>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.get_alert_rules(Empty {}).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({
                "rules": r.rules.into_iter().map(rule_json).collect::<Vec<_>>(),
                "count": r.count,
            })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

fn rule_json(x: proto::AlertRule) -> serde_json::Value {
    json!({
        "id": x.id, "name": x.name, "description": x.description, "metric_name": x.metric_name,
        "condition": x.condition, "threshold": x.threshold, "window_minutes": x.window_minutes,
        "severity": x.severity, "enabled": x.enabled, "notification_webhook": x.notification_webhook,
        "cooldown_minutes": x.cooldown_minutes, "created_at": x.created_at, "updated_at": x.updated_at,
    })
}

pub async fn get_alert_rule(State(_s): State<AppState>, Path(rule_id): Path<String>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.get_alert_rule(RuleIdRequest { rule_id }).await {
        Ok(r) => Json(rule_json(r.into_inner())).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn create_alert_rule(State(_s): State<AppState>, Json(req): Json<CreateAlertRuleRequest>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    let proto_req = ProtoCreateReq {
        name: req.name, description: req.description, metric_name: req.metric_name, condition: req.condition,
        threshold: req.threshold, window_minutes: req.window_minutes, severity: req.severity,
        notification_webhook: req.notification_webhook, cooldown_minutes: req.cooldown_minutes,
    };
    match c.create_alert_rule(proto_req).await {
        Ok(r) => {
            let r = r.into_inner();
            (StatusCode::CREATED, Json(json!({ "id": r.id, "name": r.name, "created_at": r.created_at }))).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn update_alert_rule(State(_s): State<AppState>, Path(rule_id): Path<String>, Json(req): Json<UpdateAlertRuleRequest>) -> Response {
    if req.name.is_none() && req.description.is_none() && req.metric_name.is_none() && req.condition.is_none()
        && req.threshold.is_none() && req.window_minutes.is_none() && req.severity.is_none()
        && req.enabled.is_none() && req.notification_webhook.is_none() && req.cooldown_minutes.is_none()
    {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "No fields to update" }))).into_response();
    }
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    let proto_req = ProtoUpdateReq {
        rule_id, name: req.name, description: req.description, metric_name: req.metric_name, condition: req.condition,
        threshold: req.threshold, window_minutes: req.window_minutes, severity: req.severity, enabled: req.enabled,
        notification_webhook: req.notification_webhook, cooldown_minutes: req.cooldown_minutes,
    };
    match c.update_alert_rule(proto_req).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({ "updated": r.updated, "rule_id": r.rule_id, "updated_at": r.updated_at })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn delete_alert_rule(State(_s): State<AppState>, Path(rule_id): Path<String>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.delete_alert_rule(RuleIdRequest { rule_id }).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({ "deleted": r.deleted, "rule_id": r.rule_id })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn get_dashboard_summary(State(_s): State<AppState>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.get_dashboard_summary(Empty {}).await {
        Ok(r) => {
            let v: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::Value::Null);
            Json(v).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn evaluate_alerts(State(_s): State<AppState>) -> Response {
    let mut c = match client().await { Ok(c) => c, Err((code, msg)) => return (code, Json(json!({"error": msg}))).into_response() };
    match c.evaluate_alerts(Empty {}).await {
        Ok(r) => {
            let r = r.into_inner();
            Json(json!({ "evaluated": r.evaluated, "alerts_fired": r.alerts_fired, "evaluated_at": r.evaluated_at })).into_response()
        }
        Err(e) => grpc_err(e),
    }
}
