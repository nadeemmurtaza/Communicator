//! Fin Operator — Meta-agent for autonomous self-improvement (Phase 17).
//!
//! The operator monitors system metrics, detects anomalies, and generates
//! structured improvement proposals using Claude. It follows a proposal-based
//! review system: proposals are reviewed by humans before deployment, unless
//! auto-deploy is enabled for low-risk changes.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;
use crate::services::claude_client;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperatorProposal {
    pub id: String,
    pub proposal_type: String,
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub expected_impact: Option<Value>,
    pub implementation_plan: Option<Value>,
    pub status: String,
    pub priority: String,
    pub confidence: f64,
    pub source: String,
    pub source_data: Option<Value>,
    pub related_feedback_ids: Option<Value>,
    pub related_conversation_ids: Option<Value>,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub deployed_at: Option<String>,
    pub deployment_result: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperatorConfig {
    pub id: String,
    pub scan_interval_minutes: i32,
    pub auto_deploy_enabled: bool,
    pub auto_deploy_max_risk: String,
    pub max_proposals_per_scan: i32,
    pub min_confidence_threshold: f64,
    pub metrics_to_monitor: Value,
    pub alert_on_failure: bool,
    pub enabled: bool,
    pub last_scan_at: Option<String>,
    pub last_scan_result: Option<Value>,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperatorMetricsSnapshot {
    pub id: String,
    pub snapshot_time: String,
    pub metrics_data: Value,
    pub anomalies: Option<Value>,
    pub trend_data: Option<Value>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperatorPerformanceLog {
    pub id: String,
    pub action_type: String,
    pub action_detail: String,
    pub target: Option<String>,
    pub result: String,
    pub output_data: Option<Value>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperatorSummary {
    pub config: OperatorConfig,
    pub total_proposals: i32,
    pub proposals_by_status: Value,
    pub proposals_by_type: Value,
    pub recent_proposals: Vec<OperatorProposal>,
    pub recent_logs: Vec<OperatorPerformanceLog>,
    pub latest_snapshot: Option<OperatorMetricsSnapshot>,
    pub anomaly_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScanResult {
    pub snapshot: OperatorMetricsSnapshot,
    pub proposals_generated: Vec<OperatorProposal>,
    pub anomalies_detected: Vec<Value>,
    pub duration_ms: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeployResult {
    pub proposal_id: String,
    pub success: bool,
    pub message: String,
    pub metrics_before: Option<Value>,
    pub metrics_after: Option<Value>,
}

// ——— Core Functions ———

/// Log an operator action to the performance log.
pub async fn log_action(
    state: &AppState,
    action_type: &str,
    action_detail: &str,
    target: Option<&str>,
    result: &str,
    output_data: Option<&Value>,
    duration_ms: Option<i64>,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let output_json = output_data.map(|v| v.to_string());

    sqlx::query(
        "INSERT INTO operator_performance_log (id, action_type, action_detail, target, result, output_data, duration_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(action_type)
    .bind(action_detail)
    .bind(target)
    .bind(result)
    .bind(&output_json)
    .bind(duration_ms)
    .execute(&state.db)
    .await?;

    Ok(id)
}

/// Get the operator configuration.
pub async fn get_config(state: &AppState) -> Result<OperatorConfig, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, scan_interval_minutes, auto_deploy_enabled, auto_deploy_max_risk,
         max_proposals_per_scan, min_confidence_threshold, metrics_to_monitor,
         alert_on_failure, enabled, last_scan_at, last_scan_result, updated_at
         FROM operator_config WHERE id = 'default'"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(OperatorConfig {
        id: row.try_get("id").unwrap_or_default(),
        scan_interval_minutes: row.try_get("scan_interval_minutes").unwrap_or(60),
        auto_deploy_enabled: row.try_get("auto_deploy_enabled").unwrap_or(false),
        auto_deploy_max_risk: row.try_get("auto_deploy_max_risk").unwrap_or_else(|| "low".to_string()),
        max_proposals_per_scan: row.try_get("max_proposals_per_scan").unwrap_or(5),
        min_confidence_threshold: row.try_get("min_confidence_threshold").unwrap_or(0.7),
        metrics_to_monitor: row.try_get::<Option<String>, _>("metrics_to_monitor")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!(["validation_score", "feedback_rate", "error_rate"])),
        alert_on_failure: row.try_get("alert_on_failure").unwrap_or(true),
        enabled: row.try_get("enabled").unwrap_or(true),
        last_scan_at: row.try_get("last_scan_at").ok(),
        last_scan_result: row.try_get::<Option<String>, _>("last_scan_result")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok()),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    })
}

/// Update the operator configuration.
pub async fn update_config(
    state: &AppState,
    updates: &Value,
) -> Result<OperatorConfig, (axum::http::StatusCode, String)> {
    let mut sets = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(v) = updates.get("scan_interval_minutes").and_then(|v| v.as_i64()) {
        sets.push("scan_interval_minutes = ?");
        binds.push(json!(v));
    }
    if let Some(v) = updates.get("auto_deploy_enabled").and_then(|v| v.as_bool()) {
        sets.push("auto_deploy_enabled = ?");
        binds.push(json!(v));
    }
    if let Some(v) = updates.get("auto_deploy_max_risk").and_then(|v| v.as_str()) {
        sets.push("auto_deploy_max_risk = ?");
        binds.push(json!(v));
    }
    if let Some(v) = updates.get("max_proposals_per_scan").and_then(|v| v.as_i64()) {
        sets.push("max_proposals_per_scan = ?");
        binds.push(json!(v));
    }
    if let Some(v) = updates.get("min_confidence_threshold").and_then(|v| v.as_f64()) {
        sets.push("min_confidence_threshold = ?");
        binds.push(json!(v));
    }
    if let Some(v) = updates.get("metrics_to_monitor") {
        sets.push("metrics_to_monitor = ?");
        binds.push(json!(v.to_string()));
    }
    if let Some(v) = updates.get("alert_on_failure").and_then(|v| v.as_bool()) {
        sets.push("alert_on_failure = ?");
        binds.push(json!(v));
    }
    if let Some(v) = updates.get("enabled").and_then(|v| v.as_bool()) {
        sets.push("enabled = ?");
        binds.push(json!(v));
    }

    if sets.is_empty() {
        return get_config(state).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    sets.push("updated_at = ?");
    binds.push(json!(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string()));
    binds.push(json!("default"));

    let sql = format!("UPDATE operator_config SET {} WHERE id = ?", sets.join(", "));
    let mut qb = sqlx::query(&sql);
    for b in &binds {
        if let Value::String(s) = b {
            qb = qb.bind(s.as_str());
        } else if let Value::Number(n) = b {
            if let Some(i) = n.as_i64() {
                qb = qb.bind(i);
            } else if let Some(f) = n.as_f64() {
                qb = qb.bind(f);
            }
        } else if let Value::Bool(bl) = b {
            qb = qb.bind(*bl);
        }
    }

    qb.execute(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    get_config(state).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Collect current metrics from existing tables and create a snapshot.
pub async fn collect_metrics_snapshot(state: &AppState) -> Result<OperatorMetricsSnapshot, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    // Collect validation scores
    let avg_validation = sqlx::query_scalar::<_, f64>(
        "SELECT AVG(relevance_score) FROM validation_results WHERE created_at >= datetime('now', '-7 days') AND relevance_score IS NOT NULL"
    )
    .fetch_optional(&state.db)
    .await?;

    // Collect feedback rate
    let feedback_row = sqlx::query(
        "SELECT COUNT(*) as total, SUM(CASE WHEN rating = 1 THEN 1 ELSE 0 END) as positive
         FROM message_feedback WHERE created_at >= datetime('now', '-7 days')"
    )
    .fetch_optional(&state.db)
    .await?;

    let feedback_rate = feedback_row.map(|row| {
        let total: i64 = row.try_get("total").unwrap_or(0);
        let positive: i64 = row.try_get("positive").unwrap_or(0);
        if total > 0 { positive as f64 / total as f64 } else { 0.0 }
    }).unwrap_or(0.0);

    // Collect error rate (failed drafts)
    let draft_row = sqlx::query(
        "SELECT COUNT(*) as total, SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) as rejected
         FROM drafts WHERE created_at >= datetime('now', '-7 days')"
    )
    .fetch_optional(&state.db)
    .await?;

    let error_rate = draft_row.map(|row| {
        let total: i64 = row.try_get("total").unwrap_or(0);
        let rejected: i64 = row.try_get("rejected").unwrap_or(0);
        if total > 0 { rejected as f64 / total as f64 } else { 0.0 }
    }).unwrap_or(0.0);

    // Collect conversation resolution rate
    let conv_row = sqlx::query(
        "SELECT COUNT(*) as total, SUM(CASE WHEN state IN ('CLOSED_WON', 'RESOLVED') THEN 1 ELSE 0 END) as resolved
         FROM conversations WHERE updated_at >= datetime('now', '-7 days')"
    )
    .fetch_optional(&state.db)
    .await?;

    let resolution_rate = conv_row.map(|row| {
        let total: i64 = row.try_get("total").unwrap_or(0);
        let resolved: i64 = row.try_get("resolved").unwrap_or(0);
        if total > 0 { resolved as f64 / total as f64 } else { 0.0 }
    }).unwrap_or(0.0);

    // Collect avg response time (from system_metrics if available)
    let avg_response_time = sqlx::query_scalar::<_, f64>(
        "SELECT AVG(value) FROM system_metrics WHERE metric_name = 'response_time_seconds' AND created_at >= datetime('now', '-7 days')"
    )
    .fetch_optional(&state.db)
    .await?;

    let metrics = json!({
        "validation_score": avg_validation,
        "feedback_rate": feedback_rate,
        "error_rate": error_rate,
        "resolution_rate": resolution_rate,
        "response_time_seconds": avg_response_time,
    });

    // Detect anomalies by comparing to previous snapshot
    let prev_snapshot = sqlx::query(
        "SELECT metrics_data FROM operator_metrics_snapshot ORDER BY snapshot_time DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let mut anomalies = Vec::<Value>::new();
    let mut trends = serde_json::Map::new();

    if let Some(prev) = prev_snapshot {
        let prev_data: Value = serde_json::from_str(&prev.try_get("metrics_data").unwrap_or_default())
            .unwrap_or_else(|_| json!({}));

        for (key, current_val) in metrics.as_object().unwrap().iter() {
            if let Some(current_f) = current_val.as_f64() {
                if let Some(prev_f) = prev_data.get(key).and_then(|v| v.as_f64()) {
                    let change = current_f - prev_f;
                    let trend = if change > 0.05 { "up" } else if change < -0.05 { "down" } else { "stable" };
                    trends.insert(key.clone(), json!({
                        "trend": trend,
                        "change": (change * 100.0).round() / 100.0,
                        "previous": prev_f,
                        "current": current_f
                    }));

                    // Anomaly detection: significant negative change
                    if (key == "validation_score" || key == "feedback_rate" || key == "resolution_rate") && change < -0.1 {
                        anomalies.push(json!({
                            "metric": key,
                            "change": change,
                            "severity": if change < -0.2 { "high" } else { "medium" },
                            "message": format!("{} dropped by {:.1}% over the period", key, change * 100.0)
                        }));
                    }
                    if (key == "error_rate") && change > 0.1 {
                        anomalies.push(json!({
                            "metric": key,
                            "change": change,
                            "severity": if change > 0.2 { "high" } else { "medium" },
                            "message": format!("{} increased by {:.1}% over the period", key, change * 100.0)
                        }));
                    }
                }
            }
        }
    }

    let anomalies_json = if anomalies.is_empty() {
        json!([])
    } else {
        json!(anomalies)
    };
    let trend_json = json!(trends);

    sqlx::query(
        "INSERT INTO operator_metrics_snapshot (id, snapshot_time, metrics_data, anomalies, trend_data)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&now)
    .bind(&metrics.to_string())
    .bind(&anomalies_json.to_string())
    .bind(&trend_json.to_string())
    .execute(&state.db)
    .await?;

    Ok(OperatorMetricsSnapshot {
        id,
        snapshot_time: now,
        metrics_data: metrics,
        anomalies: Some(anomalies_json),
        trend_data: Some(trend_json),
        created_at: now,
    })
}

/// Run a full operator scan: collect metrics, detect anomalies, generate proposals.
pub async fn run_scan(state: &AppState) -> Result<ScanResult, sqlx::Error> {
    let start = std::time::Instant::now();

    // Log scan start
    log_action(state, "scan", "Operator scan initiated", None, "success", None, None).await.ok();

    // Collect metrics snapshot
    let snapshot = collect_metrics_snapshot(state).await?;

    // Get config
    let config = get_config(state).await?;
    if !config.enabled {
        return Err(sqlx::Error::Pool(sqlx::pool::PoolError::Closed));
    }

    // Generate proposals based on anomalies and metrics
    let mut proposals = Vec::new();

    // Use Claude to analyze metrics and generate proposals
    let anomalies = snapshot.anomalies.clone().unwrap_or_else(|| json!([]));
    let metrics_context = snapshot.metrics_data.to_string();

    let prompt = format!(
        r#"You are the Fin Operator — a meta-agent responsible for monitoring and improving the AI customer communication system.

Current system metrics (last 7 days):
{metrics_context}

Detected anomalies:
{anomalies}

Based on these metrics and anomalies, generate up to {} improvement proposals. Each proposal should address a specific issue and include:
1. A clear title
2. A description of the problem and proposed solution
3. The rationale (why this will help)
4. Expected impact (which metric, direction, estimated delta)
5. Implementation plan (steps)
6. Confidence score (0-1)
7. Priority (low/medium/high/critical)
8. Type (knowledge_update, procedure_change, policy_update, prompt_tuning, threshold_adjustment, content_creation, escalation_rule, general)

Return a JSON array of proposals. If no improvements are needed, return an empty array."#,
        config.max_proposals_per_scan
    );

    let claude_result = claude_client::generate_text(&state.http_client, &prompt, 1024).await;

    let proposals_generated = match claude_result {
        Ok(response) => {
            // Parse proposals from Claude response
            if let Ok(parsed) = serde_json::from_str::<Value>(&response) {
                let arr = if parsed.is_array() { parsed.as_array().unwrap() } else { &[] };
                for item in arr.iter().take(config.max_proposals_per_scan as usize) {
                    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if title.is_empty() {
                        continue;
                    }

                    let confidence = item.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
                    if confidence < config.min_confidence_threshold {
                        continue;
                    }

                    let id = Uuid::new_v4().to_string();
                    let proposal_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("general").to_string();
                    let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let rationale = item.get("rationale").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let priority = item.get("priority").and_then(|v| v.as_str()).unwrap_or("medium").to_string();

                    sqlx::query(
                        "INSERT INTO operator_proposals (id, proposal_type, title, description, rationale,
                         expected_impact, implementation_plan, status, priority, confidence, source, source_data)
                         VALUES (?, ?, ?, ?, ?, ?, ?, 'proposed', ?, ?, 'auto', ?)"
                    )
                    .bind(&id)
                    .bind(&proposal_type)
                    .bind(&title)
                    .bind(&description)
                    .bind(&rationale)
                    .bind(&item.get("expected_impact").map(|v| v.to_string()).unwrap_or_default())
                    .bind(&item.get("implementation_plan").map(|v| v.to_string()).unwrap_or_default())
                    .bind(confidence)
                    .bind(&priority)
                    .bind(&json!({"metrics": metrics_context, "anomalies": anomalies}).to_string())
                    .execute(&state.db)
                    .await?;

                    proposals.push(OperatorProposal {
                        id,
                        proposal_type,
                        title,
                        description,
                        rationale,
                        expected_impact: item.get("expected_impact").cloned(),
                        implementation_plan: item.get("implementation_plan").cloned(),
                        status: "proposed".to_string(),
                        priority,
                        confidence,
                        source: "auto".to_string(),
                        source_data: Some(json!({"metrics": metrics_context, "anomalies": anomalies})),
                        related_feedback_ids: None,
                        related_conversation_ids: None,
                        created_by: "operator".to_string(),
                        reviewed_by: None,
                        reviewed_at: None,
                        deployed_at: None,
                        deployment_result: None,
                        created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string(),
                        updated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string(),
                    });
                }
            }
            log_action(state, "propose", &format!("Generated {} proposals via Claude", proposals.len()), None, "success", Some(&json!({})), None).await.ok();
            proposals.len()
        }
        Err(e) => {
            tracing::warn!("Claude unavailable for operator scan: {}", e);
            log_action(state, "propose", "Claude unavailable, skipping proposal generation", None, "skipped", None, None).await.ok();
            0
        }
    };

    // Update config with scan result
    let scan_summary = json!({
        "snapshot_id": snapshot.id,
        "proposals_generated": proposals_generated,
        "anomalies_detected": anomalies.as_array().map(|a| a.len()).unwrap_or(0),
    });

    sqlx::query(
        "UPDATE operator_config SET last_scan_at = ?, last_scan_result = ?, updated_at = ? WHERE id = 'default'"
    )
    .bind(&snapshot.snapshot_time)
    .bind(&scan_summary.to_string())
    .bind(&snapshot.created_at)
    .execute(&state.db)
    .await?;

    let duration = start.elapsed().as_millis() as i64;
    log_action(state, "scan", "Operator scan completed", None, "success", Some(&scan_summary), Some(duration)).await.ok();

    Ok(ScanResult {
        snapshot,
        proposals_generated: proposals,
        anomalies_detected: anomalies.as_array().cloned().unwrap_or_default(),
        duration_ms: duration,
    })
}

/// Get operator summary dashboard data.
pub async fn get_summary(state: &AppState) -> Result<OperatorSummary, sqlx::Error> {
    let config = get_config(state).await?;

    // Count proposals by status
    let status_rows = sqlx::query(
        "SELECT status, COUNT(*) as count FROM operator_proposals GROUP BY status"
    )
    .fetch_all(&state.db)
    .await?;

    let mut by_status = serde_json::Map::new();
    for row in &status_rows {
        let status: String = row.try_get("status").unwrap_or_default();
        let count: i64 = row.try_get("count").unwrap_or(0);
        by_status.insert(status, json!(count));
    }

    // Count proposals by type
    let type_rows = sqlx::query(
        "SELECT proposal_type, COUNT(*) as count FROM operator_proposals GROUP BY proposal_type"
    )
    .fetch_all(&state.db)
    .await?;

    let mut by_type = serde_json::Map::new();
    for row in &type_rows {
        let ptype: String = row.try_get("proposal_type").unwrap_or_default();
        let count: i64 = row.try_get("count").unwrap_or(0);
        by_type.insert(ptype, json!(count));
    }

    let total: i64 = by_status.values().map(|v| v.as_i64().unwrap_or(0)).sum();

    // Recent proposals
    let recent_rows = sqlx::query(
        "SELECT id, proposal_type, title, description, rationale, expected_impact,
         implementation_plan, status, priority, confidence, source, source_data,
         related_feedback_ids, related_conversation_ids, created_by, reviewed_by,
         reviewed_at, deployed_at, deployment_result, created_at, updated_at
         FROM operator_proposals ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_proposals: Vec<OperatorProposal> = recent_rows.iter().map(|row| {
        OperatorProposal {
            id: row.try_get("id").unwrap_or_default(),
            proposal_type: row.try_get("proposal_type").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            description: row.try_get("description").unwrap_or_default(),
            rationale: row.try_get("rationale").unwrap_or_default(),
            expected_impact: row.try_get::<Option<String>, _>("expected_impact").ok().and_then(|s| serde_json::from_str(&s).ok()),
            implementation_plan: row.try_get::<Option<String>, _>("implementation_plan").ok().and_then(|s| serde_json::from_str(&s).ok()),
            status: row.try_get("status").unwrap_or_default(),
            priority: row.try_get("priority").unwrap_or_default(),
            confidence: row.try_get("confidence").unwrap_or(0.0),
            source: row.try_get("source").unwrap_or_default(),
            source_data: row.try_get::<Option<String>, _>("source_data").ok().and_then(|s| serde_json::from_str(&s).ok()),
            related_feedback_ids: row.try_get::<Option<String>, _>("related_feedback_ids").ok().and_then(|s| serde_json::from_str(&s).ok()),
            related_conversation_ids: row.try_get::<Option<String>, _>("related_conversation_ids").ok().and_then(|s| serde_json::from_str(&s).ok()),
            created_by: row.try_get("created_by").unwrap_or_default(),
            reviewed_by: row.try_get("reviewed_by").ok(),
            reviewed_at: row.try_get("reviewed_at").ok(),
            deployed_at: row.try_get("deployed_at").ok(),
            deployment_result: row.try_get::<Option<String>, _>("deployment_result").ok().and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        }
    }).collect();

    // Recent logs
    let log_rows = sqlx::query(
        "SELECT id, action_type, action_detail, target, result, output_data, duration_ms, created_at
         FROM operator_performance_log ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_logs: Vec<OperatorPerformanceLog> = log_rows.iter().map(|row| {
        OperatorPerformanceLog {
            id: row.try_get("id").unwrap_or_default(),
            action_type: row.try_get("action_type").unwrap_or_default(),
            action_detail: row.try_get("action_detail").unwrap_or_default(),
            target: row.try_get("target").ok(),
            result: row.try_get("result").unwrap_or_default(),
            output_data: row.try_get::<Option<String>, _>("output_data").ok().and_then(|s| serde_json::from_str(&s).ok()),
            duration_ms: row.try_get("duration_ms").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        }
    }).collect();

    // Latest snapshot
    let snap_row = sqlx::query(
        "SELECT id, snapshot_time, metrics_data, anomalies, trend_data, created_at
         FROM operator_metrics_snapshot ORDER BY snapshot_time DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let latest_snapshot = snap_row.map(|row| OperatorMetricsSnapshot {
        id: row.try_get("id").unwrap_or_default(),
        snapshot_time: row.try_get("snapshot_time").unwrap_or_default(),
        metrics_data: serde_json::from_str(&row.try_get("metrics_data").unwrap_or_default()).unwrap_or_else(|_| json!({})),
        anomalies: row.try_get::<Option<String>, _>("anomalies").ok().and_then(|s| serde_json::from_str(&s).ok()),
        trend_data: row.try_get::<Option<String>, _>("trend_data").ok().and_then(|s| serde_json::from_str(&s).ok()),
        created_at: row.try_get("created_at").unwrap_or_default(),
    });

    // Anomaly count from latest snapshot
    let anomaly_count = latest_snapshot.as_ref()
        .and_then(|s| s.anomalies.as_ref())
        .and_then(|a| a.as_array())
        .map(|a| a.len() as i32)
        .unwrap_or(0);

    Ok(OperatorSummary {
        config,
        total_proposals: total as i32,
        proposals_by_status: json!(by_status),
        proposals_by_type: json!(by_type),
        recent_proposals,
        recent_logs,
        latest_snapshot,
        anomaly_count,
    })
}

/// List proposals with filters.
pub async fn list_proposals(
    state: &AppState,
    status: Option<&str>,
    proposal_type: Option<&str>,
    limit: i32,
) -> Result<Vec<OperatorProposal>, sqlx::Error> {
    let mut query = String::from(
        "SELECT id, proposal_type, title, description, rationale, expected_impact,
         implementation_plan, status, priority, confidence, source, source_data,
         related_feedback_ids, related_conversation_ids, created_by, reviewed_by,
         reviewed_at, deployed_at, deployment_result, created_at, updated_at
         FROM operator_proposals WHERE 1=1"
    );

    if let Some(s) = status {
        query.push_str(" AND status = ?");
    }
    if let Some(t) = proposal_type {
        query.push_str(" AND proposal_type = ?");
    }
    query.push_str(" ORDER BY created_at DESC LIMIT ?");

    let mut qb = sqlx::query(&query);
    if let Some(s) = status {
        qb = qb.bind(s);
    }
    if let Some(t) = proposal_type {
        qb = qb.bind(t);
    }
    qb = qb.bind(limit);

    let rows = qb.fetch_all(&state.db).await?;

    let proposals: Vec<OperatorProposal> = rows.iter().map(|row| {
        OperatorProposal {
            id: row.try_get("id").unwrap_or_default(),
            proposal_type: row.try_get("proposal_type").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            description: row.try_get("description").unwrap_or_default(),
            rationale: row.try_get("rationale").unwrap_or_default(),
            expected_impact: row.try_get::<Option<String>, _>("expected_impact").ok().and_then(|s| serde_json::from_str(&s).ok()),
            implementation_plan: row.try_get::<Option<String>, _>("implementation_plan").ok().and_then(|s| serde_json::from_str(&s).ok()),
            status: row.try_get("status").unwrap_or_default(),
            priority: row.try_get("priority").unwrap_or_default(),
            confidence: row.try_get("confidence").unwrap_or(0.0),
            source: row.try_get("source").unwrap_or_default(),
            source_data: row.try_get::<Option<String>, _>("source_data").ok().and_then(|s| serde_json::from_str(&s).ok()),
            related_feedback_ids: row.try_get::<Option<String>, _>("related_feedback_ids").ok().and_then(|s| serde_json::from_str(&s).ok()),
            related_conversation_ids: row.try_get::<Option<String>, _>("related_conversation_ids").ok().and_then(|s| serde_json::from_str(&s).ok()),
            created_by: row.try_get("created_by").unwrap_or_default(),
            reviewed_by: row.try_get("reviewed_by").ok(),
            reviewed_at: row.try_get("reviewed_at").ok(),
            deployed_at: row.try_get("deployed_at").ok(),
            deployment_result: row.try_get::<Option<String>, _>("deployment_result").ok().and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        }
    }).collect();

    Ok(proposals)
}

/// Approve a proposal (move to testing/deployed).
pub async fn approve_proposal(
    state: &AppState,
    proposal_id: &str,
    reviewer: &str,
) -> Result<OperatorProposal, (axum::http::StatusCode, String)> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    // Check if auto-deploy is enabled
    let config = get_config(state).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let new_status = if config.auto_deploy_enabled { "deployed" } else { "testing" };

    sqlx::query(
        "UPDATE operator_proposals SET status = ?, reviewed_by = ?, reviewed_at = ?,
         deployed_at = CASE WHEN ? = 'deployed' THEN ? ELSE NULL END,
         updated_at = ? WHERE id = ?"
    )
    .bind(new_status)
    .bind(reviewer)
    .bind(&now)
    .bind(new_status)
    .bind(&now)
    .bind(&now)
    .bind(proposal_id)
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    log_action(state, "deploy", &format!("Proposal {} approved and moved to {}", proposal_id, new_status), Some(proposal_id), "success", None, None).await.ok();

    // Return updated proposal
    list_proposals(state, None, None, 1).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .and_then(|props| {
            props.into_iter().find(|p| p.id == proposal_id)
                .ok_or((axum::http::StatusCode::NOT_FOUND, "Proposal not found".to_string()))
        })
}

/// Dismiss a proposal.
pub async fn dismiss_proposal(
    state: &AppState,
    proposal_id: &str,
    reviewer: &str,
) -> Result<(), (axum::http::StatusCode, String)> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    sqlx::query(
        "UPDATE operator_proposals SET status = 'dismissed', reviewed_by = ?, reviewed_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(reviewer)
    .bind(&now)
    .bind(&now)
    .bind(proposal_id)
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    log_action(state, "propose", &format!("Proposal {} dismissed", proposal_id), Some(proposal_id), "success", None, None).await.ok();

    Ok(())
}

/// Get a single proposal by ID.
pub async fn get_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<OperatorProposal, (axum::http::StatusCode, String)> {
    let proposals = list_proposals(state, None, None, 100).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    proposals.into_iter().find(|p| p.id == proposal_id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Proposal not found".to_string()))
}

/// Get performance logs with filters.
pub async fn list_logs(
    state: &AppState,
    action_type: Option<&str>,
    limit: i32,
) -> Result<Vec<OperatorPerformanceLog>, sqlx::Error> {
    let mut query = String::from(
        "SELECT id, action_type, action_detail, target, result, output_data, duration_ms, created_at
         FROM operator_performance_log WHERE 1=1"
    );

    if let Some(at) = action_type {
        query.push_str(" AND action_type = ?");
    }
    query.push_str(" ORDER BY created_at DESC LIMIT ?");

    let mut qb = sqlx::query(&query);
    if let Some(at) = action_type {
        qb = qb.bind(at);
    }
    qb = qb.bind(limit);

    let rows = qb.fetch_all(&state.db).await?;

    let logs: Vec<OperatorPerformanceLog> = rows.iter().map(|row| {
        OperatorPerformanceLog {
            id: row.try_get("id").unwrap_or_default(),
            action_type: row.try_get("action_type").unwrap_or_default(),
            action_detail: row.try_get("action_detail").unwrap_or_default(),
            target: row.try_get("target").ok(),
            result: row.try_get("result").unwrap_or_default(),
            output_data: row.try_get::<Option<String>, _>("output_data").ok().and_then(|s| serde_json::from_str(&s).ok()),
            duration_ms: row.try_get("duration_ms").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        }
    }).collect();

    Ok(logs)
}

/// Get metrics snapshots with time range filter.
pub async fn list_snapshots(
    state: &AppState,
    limit: i32,
) -> Result<Vec<OperatorMetricsSnapshot>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, snapshot_time, metrics_data, anomalies, trend_data, created_at
         FROM operator_metrics_snapshot ORDER BY snapshot_time DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let snapshots: Vec<OperatorMetricsSnapshot> = rows.iter().map(|row| {
        OperatorMetricsSnapshot {
            id: row.try_get("id").unwrap_or_default(),
            snapshot_time: row.try_get("snapshot_time").unwrap_or_default(),
            metrics_data: serde_json::from_str(&row.try_get("metrics_data").unwrap_or_default()).unwrap_or_else(|_| json!({})),
            anomalies: row.try_get::<Option<String>, _>("anomalies").ok().and_then(|s| serde_json::from_str(&s).ok()),
            trend_data: row.try_get::<Option<String>, _>("trend_data").ok().and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at").unwrap_or_default(),
        }
    }).collect();

    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_config_serialization() {
        let config = OperatorConfig {
            id: "default".to_string(),
            scan_interval_minutes: 60,
            auto_deploy_enabled: false,
            auto_deploy_max_risk: "low".to_string(),
            max_proposals_per_scan: 5,
            min_confidence_threshold: 0.7,
            metrics_to_monitor: json!(["validation_score", "feedback_rate"]),
            alert_on_failure: true,
            enabled: true,
            last_scan_at: None,
            last_scan_result: None,
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("scan_interval_minutes"));
        assert!(json.contains("auto_deploy_enabled"));
    }

    #[test]
    fn test_scan_result_serialization() {
        let result = ScanResult {
            snapshot: OperatorMetricsSnapshot {
                id: "test".to_string(),
                snapshot_time: "2026-01-01T00:00:00Z".to_string(),
                metrics_data: json!({"validation_score": 0.85}),
                anomalies: Some(json!([])),
                trend_data: Some(json!({})),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            },
            proposals_generated: vec![],
            anomalies_detected: vec![],
            duration_ms: 1500,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("proposals_generated"));
    }

    #[test]
    fn test_deploy_result_serialization() {
        let result = DeployResult {
            proposal_id: "test-123".to_string(),
            success: true,
            message: "Deployed successfully".to_string(),
            metrics_before: Some(json!({"score": 0.8})),
            metrics_after: Some(json!({"score": 0.85})),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Deployed successfully"));
    }
}
