#![allow(dead_code)]

//! Monitoring & Alerting API routes — Phase 8.
//!
//! Provides endpoints for:
//! - Extended health checks with all component status
//! - Time-series metrics with filtering
//! - Aggregated metric summaries
//! - Alert events with acknowledgment
//! - Alert rules management (CRUD)
//! - Manual metric collection trigger

use crate::db::AppState;
use crate::services::monitoring;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use tracing::info;

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    /// Metric name filter (optional)
    pub name: Option<String>,
    /// Time window in hours (default: 24)
    pub window_hours: Option<i64>,
    /// Limit number of results (default: 100)
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    /// Filter by severity (optional)
    pub severity: Option<String>,
    /// Filter by acknowledged status (optional)
    pub acknowledged: Option<bool>,
    /// Filter by resolved status (optional)
    pub resolved: Option<bool>,
    /// Limit number of results (default: 50)
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

// ---------------------------------------------------------------------------
// Health Check
// ---------------------------------------------------------------------------

/// Extended health check with all component status.
pub async fn health_check_extended(State(state): State<AppState>) -> impl IntoResponse {
    let health = monitoring::run_health_checks(&state).await;

    let status_code = match health.overall_status.as_str() {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::SERVICE_UNAVAILABLE,
        "unhealthy" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status_code, Json(health)).into_response()
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

/// Get time-series metrics with optional filtering.
pub async fn get_metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricsQuery>,
) -> impl IntoResponse {
    let window_hours = params.window_hours.unwrap_or(24);
    let limit = params.limit.unwrap_or(100);

    let query = if let Some(ref name) = params.name {
        format!(
            "SELECT * FROM system_metrics
             WHERE metric_name = ?
             AND collected_at >= datetime('now', '-{} hours')
             ORDER BY collected_at DESC
             LIMIT {}",
            window_hours, limit
        )
    } else {
        format!(
            "SELECT * FROM system_metrics
             WHERE collected_at >= datetime('now', '-{} hours')
             ORDER BY collected_at DESC
             LIMIT {}",
            window_hours, limit
        )
    };

    let metrics: Vec<monitoring::SystemMetric> = if let Some(ref name) = params.name {
        sqlx::query_as::<_, monitoring::SystemMetric>(&query)
            .bind(name)
            .fetch_all(&state.db)
            .await
    } else {
        sqlx::query_as::<_, monitoring::SystemMetric>(&query)
            .fetch_all(&state.db)
            .await
    }
    .unwrap_or_default();

    Json(json!({
        "metrics": metrics,
        "window_hours": window_hours,
        "count": metrics.len(),
    }))
    .into_response()
}

/// Get aggregated metric summaries.
pub async fn get_metrics_summary(
    State(state): State<AppState>,
    Query(params): Query<MetricsQuery>,
) -> impl IntoResponse {
    let window_hours = params.window_hours.unwrap_or(24);

    match monitoring::get_metric_summaries(&state, window_hours).await {
        Ok(summaries) => {
            Json(json!({
                "summaries": summaries,
                "window_hours": window_hours,
                "count": summaries.len(),
            }))
            .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// Trigger metric collection.
pub async fn collect_metrics(State(state): State<AppState>) -> impl IntoResponse {
    match monitoring::collect_all_metrics(&state).await {
        Ok(count) => {
            // Also evaluate alert rules after collecting metrics
            let alerts_fired = monitoring::evaluate_alert_rules(&state).await.unwrap_or(0);

            Json(json!({
                "collected": count,
                "alerts_fired": alerts_fired,
                "collected_at": Utc::now().to_rfc3339(),
            }))
            .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Alert Events
// ---------------------------------------------------------------------------

/// List alert events with optional filtering.
pub async fn get_alert_events(
    State(state): State<AppState>,
    Query(params): Query<AlertsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);

    let mut conditions = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(ref severity) = params.severity {
        conditions.push("severity = ?".to_string());
        binds.push(json!(severity));
    }

    if let Some(acknowledged) = params.acknowledged {
        conditions.push(format!("acknowledged = {}", if acknowledged { 1 } else { 0 }));
    }

    if let Some(resolved) = params.resolved {
        conditions.push(format!("resolved = {}", if resolved { 1 } else { 0 }));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT * FROM alert_events{} ORDER BY created_at DESC LIMIT {}",
        where_clause, limit
    );

    let events: Vec<monitoring::AlertEvent> = sqlx::query_as::<_, monitoring::AlertEvent>(&query)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    Json(json!({
        "alerts": events,
        "count": events.len(),
    }))
    .into_response()
}

/// Acknowledge an alert event.
pub async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<String>,
    Json(req): Json<AcknowledgeAlertRequest>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let acknowledged_by = req.acknowledged_by.unwrap_or_else(|| "api".to_string());

    let result = sqlx::query(
        "UPDATE alert_events SET acknowledged = 1, acknowledged_by = ?, acknowledged_at = ?
         WHERE id = ? AND acknowledged = 0"
    )
    .bind(&acknowledged_by)
    .bind(&now)
    .bind(&alert_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            info!("Alert {} acknowledged by {}", alert_id, acknowledged_by);
            Json(json!({
                "acknowledged": true,
                "alert_id": alert_id,
                "acknowledged_by": acknowledged_by,
                "acknowledged_at": now,
            }))
            .into_response()
        }
        Ok(_) => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Alert {} not found or already acknowledged", alert_id) })),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

/// Resolve an alert event.
pub async fn resolve_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "UPDATE alert_events SET resolved = 1, resolved_at = ?
         WHERE id = ? AND resolved = 0"
    )
    .bind(&now)
    .bind(&alert_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            info!("Alert {} resolved", alert_id);
            Json(json!({
                "resolved": true,
                "alert_id": alert_id,
                "resolved_at": now,
            }))
            .into_response()
        }
        Ok(_) => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Alert {} not found or already resolved", alert_id) })),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Alert Rules
// ---------------------------------------------------------------------------

/// List all alert rules.
pub async fn get_alert_rules(State(state): State<AppState>) -> impl IntoResponse {
    let rules: Vec<monitoring::AlertRule> = sqlx::query_as::<_, monitoring::AlertRule>(
        "SELECT * FROM alert_rules ORDER BY severity DESC, name"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(json!({
        "rules": rules,
        "count": rules.len(),
    }))
    .into_response()
}

/// Get a single alert rule.
pub async fn get_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    let rule: Option<monitoring::AlertRule> = sqlx::query_as::<_, monitoring::AlertRule>(
        "SELECT * FROM alert_rules WHERE id = ?"
    )
    .bind(&rule_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or_default();

    match rule {
        Some(rule) => Json(rule).into_response(),
        None => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Alert rule {} not found", rule_id) })),
            )
                .into_response()
        }
    }
}

/// Create a new alert rule.
pub async fn create_alert_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateAlertRuleRequest>,
) -> impl IntoResponse {
    let id = format!("rule-{}", uuid::Uuid::new_v4().as_hyphenated());
    let now = Utc::now().to_rfc3339();
    let description = req.description.unwrap_or_default();
    let window_minutes = req.window_minutes.unwrap_or(60);
    let severity = req.severity.unwrap_or_else(|| "warning".to_string());
    let cooldown_minutes = req.cooldown_minutes.unwrap_or(30);

    let result = sqlx::query(
        "INSERT INTO alert_rules (id, name, description, metric_name, condition, threshold,
         window_minutes, severity, enabled, notification_webhook, cooldown_minutes, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&description)
    .bind(&req.metric_name)
    .bind(&req.condition)
    .bind(req.threshold)
    .bind(window_minutes)
    .bind(&severity)
    .bind(&req.notification_webhook)
    .bind(cooldown_minutes)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            info!("Alert rule created: {} ({})", id, req.name);
            (
                StatusCode::CREATED,
                Json(json!({
                    "id": id,
                    "name": req.name,
                    "created_at": now,
                })),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

/// Update an existing alert rule.
pub async fn update_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateAlertRuleRequest>,
) -> impl IntoResponse {
    let mut sets = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(ref name) = req.name {
        sets.push("name = ?");
        binds.push(json!(name));
    }
    if let Some(ref description) = req.description {
        sets.push("description = ?");
        binds.push(json!(description));
    }
    if let Some(ref metric_name) = req.metric_name {
        sets.push("metric_name = ?");
        binds.push(json!(metric_name));
    }
    if let Some(ref condition) = req.condition {
        sets.push("condition = ?");
        binds.push(json!(condition));
    }
    if let Some(threshold) = req.threshold {
        sets.push("threshold = ?");
        binds.push(json!(threshold));
    }
    if let Some(window_minutes) = req.window_minutes {
        sets.push("window_minutes = ?");
        binds.push(json!(window_minutes));
    }
    if let Some(ref severity) = req.severity {
        sets.push("severity = ?");
        binds.push(json!(severity));
    }
    if let Some(enabled) = req.enabled {
        sets.push("enabled = ?");
        binds.push(json!(enabled));
    }
    if let Some(ref webhook) = req.notification_webhook {
        sets.push("notification_webhook = ?");
        binds.push(json!(webhook));
    }
    if let Some(cooldown) = req.cooldown_minutes {
        sets.push("cooldown_minutes = ?");
        binds.push(json!(cooldown));
    }

    if sets.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No fields to update" })),
        )
            .into_response();
    }

    sets.push("updated_at = ?");
    let now = Utc::now().to_rfc3339();
    binds.push(json!(now));
    binds.push(json!(rule_id));

    let query = format!("UPDATE alert_rules SET {} WHERE id = ?", sets.join(", "));

    // Build query with manual binds
    let mut q = sqlx::query(&query);
    for b in &binds {
        q = q.bind(b);
    }
    let result = q.execute(&state.db).await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            Json(json!({
                "updated": true,
                "rule_id": rule_id,
                "updated_at": now,
            }))
            .into_response()
        }
        Ok(_) => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Alert rule {} not found", rule_id) })),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

/// Delete an alert rule.
pub async fn delete_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM alert_rules WHERE id = ?")
        .bind(&rule_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            info!("Alert rule deleted: {}", rule_id);
            Json(json!({ "deleted": true, "rule_id": rule_id })).into_response()
        }
        Ok(_) => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Alert rule {} not found", rule_id) })),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

/// Get the full monitoring dashboard summary.
pub async fn get_dashboard_summary(State(state): State<AppState>) -> impl IntoResponse {
    match monitoring::get_dashboard_summary(&state).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Evaluate Alerts
// ---------------------------------------------------------------------------

/// Manually trigger alert rule evaluation.
pub async fn evaluate_alerts(State(state): State<AppState>) -> impl IntoResponse {
    match monitoring::evaluate_alert_rules(&state).await {
        Ok(count) => {
            Json(json!({
                "evaluated": true,
                "alerts_fired": count,
                "evaluated_at": Utc::now().to_rfc3339(),
            }))
            .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response()
        }
    }
}
