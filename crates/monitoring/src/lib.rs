//! Monitoring & Alerting — business logic.
//!
//! Extracted from `backend-rs/src/services/monitoring.rs` *and* the
//! query logic that used to live directly in
//! `backend-rs/src/routes/monitoring.rs` (alert CRUD, metrics listing) --
//! all of it belongs in this crate now, not split between a thin route
//! handler and a service module.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

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

async fn insert_metric(
    pool: &PgPool,
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
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn run_health_checks(pool: &PgPool) -> HealthCheckResult {
    let mut components = Vec::new();

    let db_status = match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    components.push(ComponentHealth { name: "database".to_string(), status: db_status.to_string(), detail: None });

    let critical_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alert_events WHERE severity = 'critical' AND resolved = false",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let alerting_status = if critical_alerts > 0 { "degraded" } else { "healthy" };
    components.push(ComponentHealth {
        name: "alerting".to_string(),
        status: alerting_status.to_string(),
        detail: if critical_alerts > 0 { Some(format!("{} unresolved critical alert(s)", critical_alerts)) } else { None },
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

pub async fn get_metrics(pool: &PgPool, name: Option<&str>, window_hours: i64, limit: i64) -> Vec<SystemMetric> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(window_hours)).to_rfc3339();
    if let Some(name) = name {
        sqlx::query_as::<_, SystemMetric>(
            "SELECT * FROM system_metrics WHERE metric_name = $1 AND collected_at >= $2 ORDER BY collected_at DESC LIMIT $3",
        )
        .bind(name)
        .bind(&cutoff)
        .bind(limit)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, SystemMetric>(
            "SELECT * FROM system_metrics WHERE collected_at >= $1 ORDER BY collected_at DESC LIMIT $2",
        )
        .bind(&cutoff)
        .bind(limit)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    }
}

pub async fn collect_all_metrics(pool: &PgPool) -> Result<i64, String> {
    let collected_at = chrono::Utc::now().to_rfc3339();
    let mut collected = 0i64;

    let active_conversations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations WHERE state = 'open'")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    insert_metric(pool, "active_conversations", active_conversations as f64, "gauge", &collected_at).await?;
    collected += 1;

    let avg_confidence: Option<f64> =
        sqlx::query_scalar("SELECT AVG(confidence) FROM drafts WHERE created_at > NOW() - INTERVAL '1 hour'")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
    insert_metric(pool, "validation_score", avg_confidence.unwrap_or(0.0), "gauge", &collected_at).await?;
    collected += 1;

    let total_recent: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM drafts WHERE created_at > NOW() - INTERVAL '1 hour'")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let rejected_recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM drafts WHERE status = 'rejected' AND created_at > NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    let error_rate = if total_recent > 0 { rejected_recent as f64 / total_recent as f64 } else { 0.0 };
    insert_metric(pool, "error_rate", error_rate, "gauge", &collected_at).await?;
    collected += 1;

    let avg_latency_ms: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) * 1000) FROM drafts WHERE created_at > NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    insert_metric(pool, "draft_generation_latency", avg_latency_ms.unwrap_or(0.0), "gauge", &collected_at).await?;
    collected += 1;

    Ok(collected)
}

pub async fn get_metric_summaries(pool: &PgPool, window_hours: i64) -> Result<Vec<MetricSummary>, String> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(window_hours)).to_rfc3339();

    let rows = sqlx::query(
        "SELECT metric_name, AVG(metric_value) as avg_value, MIN(metric_value) as min_value, \
         MAX(metric_value) as max_value, COUNT(*) as sample_count \
         FROM system_metrics WHERE collected_at >= $1 GROUP BY metric_name ORDER BY metric_name",
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut summaries = Vec::with_capacity(rows.len());
    for row in rows {
        let metric_name: String = row.try_get("metric_name").unwrap_or_default();
        let latest_value: f64 = sqlx::query_scalar(
            "SELECT metric_value FROM system_metrics WHERE metric_name = $1 ORDER BY collected_at DESC LIMIT 1",
        )
        .bind(&metric_name)
        .fetch_one(pool)
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

pub async fn evaluate_alert_rules(pool: &PgPool) -> Result<i64, String> {
    let rules: Vec<AlertRule> =
        sqlx::query_as("SELECT * FROM alert_rules WHERE enabled = true").fetch_all(pool).await.map_err(|e| e.to_string())?;

    let mut fired = 0i64;

    for rule in rules {
        let window_cutoff = (chrono::Utc::now() - chrono::Duration::minutes(rule.window_minutes as i64)).to_rfc3339();
        let avg_value: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(metric_value) FROM system_metrics WHERE metric_name = $1 AND collected_at >= $2",
        )
        .bind(&rule.metric_name)
        .bind(&window_cutoff)
        .fetch_one(pool)
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
        let recent: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_events WHERE rule_id = $1 AND created_at >= $2")
            .bind(&rule.id)
            .bind(cooldown_cutoff)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        if recent > 0 {
            continue;
        }

        let id = format!("alert-{}", uuid::Uuid::new_v4());
        let message =
            format!("{} is {:.3} which is {} the threshold of {:.3}", rule.metric_name, value, rule.condition, rule.threshold);
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        fired += 1;
    }

    Ok(fired)
}

pub async fn get_dashboard_summary(pool: &PgPool) -> Result<serde_json::Value, String> {
    let health = run_health_checks(pool).await;
    let summaries = get_metric_summaries(pool, 24).await?;

    let active_alerts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM alert_events WHERE resolved = false").fetch_one(pool).await.map_err(|e| e.to_string())?;
    let critical_alerts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_events WHERE resolved = false AND severity = 'critical'")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let enabled_rules: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM alert_rules WHERE enabled = true").fetch_one(pool).await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "health": health,
        "metric_summaries": summaries,
        "active_alerts": active_alerts,
        "critical_alerts": critical_alerts,
        "enabled_rules": enabled_rules,
    }))
}

// ---------------------------------------------------------------------------
// Alert events (moved from routes.rs's inline queries)
// ---------------------------------------------------------------------------

pub async fn get_alert_events(
    pool: &PgPool,
    severity: Option<&str>,
    acknowledged: Option<bool>,
    resolved: Option<bool>,
    limit: i64,
) -> Vec<AlertEvent> {
    let mut conditions = Vec::new();
    let mut idx = 1u32;

    if severity.is_some() {
        conditions.push(format!("severity = ${}", idx));
        idx += 1;
    }
    if let Some(acknowledged) = acknowledged {
        conditions.push(format!("acknowledged = {}", acknowledged));
    }
    if let Some(resolved) = resolved {
        conditions.push(format!("resolved = {}", resolved));
    }

    let where_clause = if conditions.is_empty() { String::new() } else { format!(" WHERE {}", conditions.join(" AND ")) };
    let query = format!("SELECT * FROM alert_events{} ORDER BY created_at DESC LIMIT ${}", where_clause, idx);

    let mut q = sqlx::query_as::<_, AlertEvent>(&query);
    if let Some(severity) = severity {
        q = q.bind(severity);
    }
    q = q.bind(limit);

    q.fetch_all(pool).await.unwrap_or_default()
}

/// Returns `(acknowledged_by, acknowledged_at)` on success, `None` if the
/// alert wasn't found or was already acknowledged.
pub async fn acknowledge_alert(
    pool: &PgPool,
    alert_id: &str,
    acknowledged_by: Option<&str>,
) -> Result<Option<(String, String)>, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let acknowledged_by = acknowledged_by.unwrap_or("api").to_string();

    let result = sqlx::query(
        "UPDATE alert_events SET acknowledged = true, acknowledged_by = $1, acknowledged_at = $2 \
         WHERE id = $3 AND acknowledged = false",
    )
    .bind(&acknowledged_by)
    .bind(&now)
    .bind(alert_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() > 0 {
        Ok(Some((acknowledged_by, now)))
    } else {
        Ok(None)
    }
}

/// Returns the resolved-at timestamp on success, `None` if not found/already resolved.
pub async fn resolve_alert(pool: &PgPool, alert_id: &str) -> Result<Option<String>, String> {
    let result = sqlx::query(
        "UPDATE alert_events SET resolved = true, resolved_at = NOW() WHERE id = $1 AND resolved = false RETURNING resolved_at",
    )
    .bind(alert_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.map(|row| {
        let resolved_at: chrono::DateTime<chrono::Utc> = row.try_get("resolved_at").unwrap_or_else(|_| chrono::Utc::now());
        resolved_at.to_rfc3339()
    }))
}

pub async fn get_alert_rules(pool: &PgPool) -> Vec<AlertRule> {
    sqlx::query_as::<_, AlertRule>("SELECT * FROM alert_rules ORDER BY severity DESC, name").fetch_all(pool).await.unwrap_or_default()
}

pub async fn get_alert_rule(pool: &PgPool, rule_id: &str) -> Option<AlertRule> {
    sqlx::query_as::<_, AlertRule>("SELECT * FROM alert_rules WHERE id = $1").bind(rule_id).fetch_optional(pool).await.unwrap_or_default()
}

/// Returns `(id, name, created_at)`.
pub async fn create_alert_rule(pool: &PgPool, req: &CreateAlertRuleRequest) -> Result<(String, String, String), String> {
    let id = format!("rule-{}", uuid::Uuid::new_v4().as_hyphenated());
    let description = req.description.clone().unwrap_or_default();
    let window_minutes = req.window_minutes.unwrap_or(60);
    let severity = req.severity.clone().unwrap_or_else(|| "warning".to_string());
    let cooldown_minutes = req.cooldown_minutes.unwrap_or(30);

    sqlx::query(
        "INSERT INTO alert_rules (id, name, description, metric_name, condition, threshold, \
         window_minutes, severity, enabled, notification_webhook, cooldown_minutes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)",
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
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    Ok((id, req.name.clone(), now))
}

/// Returns the updated-at timestamp on success, `None` if the rule wasn't found.
pub async fn update_alert_rule(pool: &PgPool, rule_id: &str, req: &UpdateAlertRuleRequest) -> Result<Option<String>, String> {
    let enabled = req.enabled.map(|v| v != 0);

    let result = sqlx::query(
        "UPDATE alert_rules SET \
            name = COALESCE($1, name), \
            description = COALESCE($2, description), \
            metric_name = COALESCE($3, metric_name), \
            condition = COALESCE($4, condition), \
            threshold = COALESCE($5, threshold), \
            window_minutes = COALESCE($6, window_minutes), \
            severity = COALESCE($7, severity), \
            enabled = COALESCE($8, enabled), \
            notification_webhook = COALESCE($9, notification_webhook), \
            cooldown_minutes = COALESCE($10, cooldown_minutes), \
            updated_at = NOW() \
         WHERE id = $11 \
         RETURNING updated_at",
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.metric_name)
    .bind(&req.condition)
    .bind(req.threshold)
    .bind(req.window_minutes)
    .bind(&req.severity)
    .bind(enabled)
    .bind(&req.notification_webhook)
    .bind(req.cooldown_minutes)
    .bind(rule_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.map(|row| {
        let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now());
        updated_at.to_rfc3339()
    }))
}

pub async fn delete_alert_rule(pool: &PgPool, rule_id: &str) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM alert_rules WHERE id = $1").bind(rule_id).execute(pool).await.map_err(|e| e.to_string())?;
    Ok(result.rows_affected() > 0)
}
