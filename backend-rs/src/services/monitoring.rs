//! Monitoring & Alerting Service (Phase 8) — system health, time-series
//! metrics collection, and threshold-based alert evaluation.
//!
//! Backed by `system_metrics`, `alert_rules` and `alert_events`
//! (migration 0019).

use crate::db::AppState;
use serde::{Deserialize, Serialize};
use sqlx::Row;

/// A single time-series metric sample.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemMetric {
    pub id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_type: String,
    pub labels: String,
    pub collected_at: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A configurable alert rule.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    pub window_minutes: i32,
    pub severity: String,
    pub enabled: bool,
    pub notification_webhook: Option<String>,
    pub cooldown_minutes: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A fired alert event.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub alert_name: String,
    pub severity: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub condition: String,
    pub message: String,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<String>,
    pub resolved: bool,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub overall_status: String,
    pub components: Vec<ComponentHealth>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSummary {
    pub metric_name: String,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub latest_value: f64,
    pub sample_count: i64,
}

/// Insert one metric sample.
async fn insert_metric(
    state: &AppState,
    name: &str,
    value: f64,
    metric_type: &str,
    collected_at: &str,
) -> Result<(), String> {
    let id = format!("metric-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO system_metrics (id, metric_name, metric_value, metric_type, labels, collected_at) \
         VALUES ($1, $2, $3, $4, '{}', $5)",
    )
    .bind(&id)
    .bind(name)
    .bind(value)
    .bind(metric_type)
    .bind(collected_at)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Run a quick health check across the components we can actually observe:
/// database connectivity and whether any critical alerts are unresolved.
pub async fn run_health_checks(state: &AppState) -> HealthCheckResult {
    let mut components = Vec::new();

    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    components.push(ComponentHealth {
        name: "database".to_string(),
        status: db_status.to_string(),
        detail: None,
    });

    let critical_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alert_events WHERE severity = 'critical' AND resolved = false",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let alerting_status = if critical_alerts > 0 { "degraded" } else { "healthy" };
    components.push(ComponentHealth {
        name: "alerting".to_string(),
        status: alerting_status.to_string(),
        detail: if critical_alerts > 0 {
            Some(format!("{} unresolved critical alert(s)", critical_alerts))
        } else {
            None
        },
    });

    let overall_status = if components.iter().any(|c| c.status == "unhealthy") {
        "unhealthy"
    } else if components.iter().any(|c| c.status == "degraded") {
        "degraded"
    } else {
        "healthy"
    };

    HealthCheckResult {
        overall_status: overall_status.to_string(),
        components,
        checked_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Collect a fresh snapshot of each tracked metric from live data and
/// persist it to `system_metrics`. Returns how many metrics were recorded.
pub async fn collect_all_metrics(state: &AppState) -> Result<i64, String> {
    let collected_at = chrono::Utc::now().to_rfc3339();
    let mut collected = 0i64;

    let active_conversations: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM conversations WHERE state = 'open'")
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    insert_metric(state, "active_conversations", active_conversations as f64, "gauge", &collected_at).await?;
    collected += 1;

    let avg_confidence: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(confidence) FROM drafts WHERE created_at > NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    insert_metric(state, "validation_score", avg_confidence.unwrap_or(0.0), "gauge", &collected_at).await?;
    collected += 1;

    let total_recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM drafts WHERE created_at > NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let rejected_recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM drafts WHERE status = 'rejected' AND created_at > NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let error_rate = if total_recent > 0 {
        rejected_recent as f64 / total_recent as f64
    } else {
        0.0
    };
    insert_metric(state, "error_rate", error_rate, "gauge", &collected_at).await?;
    collected += 1;

    let avg_latency_ms: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) * 1000) FROM drafts \
         WHERE created_at > NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    insert_metric(state, "draft_generation_latency", avg_latency_ms.unwrap_or(0.0), "gauge", &collected_at).await?;
    collected += 1;

    Ok(collected)
}

/// Aggregate metrics over the trailing `window_hours`, grouped by metric name.
pub async fn get_metric_summaries(state: &AppState, window_hours: i64) -> Result<Vec<MetricSummary>, String> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(window_hours)).to_rfc3339();

    let rows = sqlx::query(
        "SELECT metric_name, AVG(metric_value) as avg_value, MIN(metric_value) as min_value, \
         MAX(metric_value) as max_value, COUNT(*) as sample_count \
         FROM system_metrics WHERE collected_at >= $1 GROUP BY metric_name ORDER BY metric_name",
    )
    .bind(&cutoff)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let mut summaries = Vec::with_capacity(rows.len());
    for row in rows {
        let metric_name: String = row.try_get("metric_name").unwrap_or_default();
        let latest_value: f64 = sqlx::query_scalar(
            "SELECT metric_value FROM system_metrics WHERE metric_name = $1 ORDER BY collected_at DESC LIMIT 1",
        )
        .bind(&metric_name)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0.0);

        summaries.push(MetricSummary {
            metric_name,
            avg_value: row.try_get("avg_value").unwrap_or(0.0),
            min_value: row.try_get("min_value").unwrap_or(0.0),
            max_value: row.try_get("max_value").unwrap_or(0.0),
            latest_value,
            sample_count: row.try_get("sample_count").unwrap_or(0),
        });
    }

    Ok(summaries)
}

/// Evaluate every enabled alert rule against the recent average of its
/// metric, firing a new `alert_events` row when the condition is met and
/// the rule isn't still within its cooldown window. Returns how many fired.
pub async fn evaluate_alert_rules(state: &AppState) -> Result<i64, String> {
    let rules: Vec<AlertRule> = sqlx::query_as("SELECT * FROM alert_rules WHERE enabled = true")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let mut fired = 0i64;

    for rule in rules {
        let window_cutoff = (chrono::Utc::now() - chrono::Duration::minutes(rule.window_minutes as i64)).to_rfc3339();
        let avg_value: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(metric_value) FROM system_metrics WHERE metric_name = $1 AND collected_at >= $2",
        )
        .bind(&rule.metric_name)
        .bind(&window_cutoff)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        let Some(value) = avg_value else { continue };

        let triggered = match rule.condition.as_str() {
            "above" => value > rule.threshold,
            "below" => value < rule.threshold,
            "equal" => (value - rule.threshold).abs() < f64::EPSILON,
            "not_equal" => (value - rule.threshold).abs() >= f64::EPSILON,
            _ => false,
        };
        if !triggered {
            continue;
        }

        let cooldown_cutoff = chrono::Utc::now() - chrono::Duration::minutes(rule.cooldown_minutes as i64);
        let recent: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM alert_events WHERE rule_id = $1 AND created_at >= $2",
        )
        .bind(&rule.id)
        .bind(cooldown_cutoff)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
        if recent > 0 {
            continue;
        }

        let id = format!("alert-{}", uuid::Uuid::new_v4());
        let message = format!(
            "{} is {:.3} which is {} the threshold of {:.3}",
            rule.metric_name, value, rule.condition, rule.threshold
        );
        sqlx::query(
            "INSERT INTO alert_events (id, rule_id, alert_name, severity, metric_name, metric_value, threshold, condition, message) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&id)
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.severity)
        .bind(&rule.metric_name)
        .bind(value)
        .bind(rule.threshold)
        .bind(&rule.condition)
        .bind(&message)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        fired += 1;
    }

    Ok(fired)
}

/// Full monitoring dashboard: current health, last 24h metric summaries,
/// and outstanding alert counts.
pub async fn get_dashboard_summary(state: &AppState) -> Result<serde_json::Value, String> {
    let health = run_health_checks(state).await;
    let summaries = get_metric_summaries(state, 24).await?;

    let active_alerts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_events WHERE resolved = false")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let critical_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alert_events WHERE resolved = false AND severity = 'critical'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    let enabled_rules: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_rules WHERE enabled = true")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "health": health,
        "metric_summaries": summaries,
        "active_alerts": active_alerts,
        "critical_alerts": critical_alerts,
        "enabled_rules": enabled_rules,
    }))
}
