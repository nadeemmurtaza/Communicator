//! Agent Performance Analytics — Phase 30
//!
//! Per-agent metrics, coaching insights, and performance comparison.
//! Complements AI-powered CX scoring (Phase 16) with human agent analytics.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPerformanceSnapshot {
    pub id: String,
    pub organization_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub window_start: String,
    pub window_end: String,
    pub conversations_handled: i32,
    pub conversations_resolved: i32,
    pub avg_response_time_seconds: f64,
    pub avg_first_response_time_seconds: f64,
    pub avg_resolution_time_seconds: f64,
    pub resolution_rate: f64,
    pub customer_satisfaction_avg: Option<f64>,
    pub customer_satisfaction_count: i32,
    pub positive_feedback_count: i32,
    pub negative_feedback_count: i32,
    pub feedback_rate: f64,
    pub messages_sent: i32,
    pub messages_received: i32,
    pub drafts_approved: i32,
    pub drafts_rejected: i32,
    pub draft_approval_rate: f64,
    pub escalations_received: i32,
    pub escalations_sent: i32,
    pub sla_breaches: i32,
    pub sla_compliance_rate: f64,
    pub cx_score_avg: Option<f64>,
    pub qa_score_avg: Option<f64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoachingInsight {
    pub id: String,
    pub organization_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub severity: String,
    pub recommendation: String,
    pub related_conversation_ids: Option<Value>,
    pub related_metrics: Option<Value>,
    pub is_addressed: bool,
    pub addressed_at: Option<String>,
    pub addressed_by: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComparisonMetrics {
    pub id: String,
    pub organization_id: String,
    pub window_start: String,
    pub window_end: String,
    pub team_avg_response_time: f64,
    pub team_avg_resolution_rate: f64,
    pub team_avg_satisfaction: Option<f64>,
    pub team_avg_cx_score: Option<f64>,
    pub team_avg_qa_score: Option<f64>,
    pub top_performer_id: Option<String>,
    pub top_performer_score: Option<f64>,
    pub needs_coaching_count: i32,
    pub exceeds_benchmark_count: i32,
    pub total_agents: i32,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPerformanceSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub auto_generate_insights: bool,
    pub insight_frequency: String,
    pub benchmark_response_time_seconds: i32,
    pub benchmark_resolution_rate: f64,
    pub benchmark_satisfaction: f64,
    pub benchmark_cx_score: f64,
    pub coaching_severity_threshold: String,
    pub show_leaderboard: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPerformanceSummary {
    pub enabled: bool,
    pub total_agents: i32,
    pub team_avg_resolution_rate: f64,
    pub team_avg_response_time: f64,
    pub team_avg_satisfaction: Option<f64>,
    pub needs_coaching: i32,
    pub exceeds_benchmark: i32,
    pub active_insights: i32,
    pub recent_snapshots: Vec<AgentPerformanceSnapshot>,
    pub recent_insights: Vec<CoachingInsight>,
    pub comparison_metrics: Option<ComparisonMetrics>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotResult {
    pub snapshot_id: String,
    pub agent_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsightResult {
    pub insight_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsResult {
    pub settings_id: String,
    pub message: String,
}

// ——— Core Functions ———

/// Get agent performance summary dashboard data
pub async fn get_performance_summary(state: &AppState) -> Result<AgentPerformanceSummary, sqlx::Error> {
    // Settings
    let settings_row = sqlx::query(
        "SELECT enabled FROM agent_performance_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let enabled = settings_row
        .as_ref()
        .map(|r| r.try_get::<bool, _>(0).unwrap_or(true))
        .unwrap_or(true);

    // Total agents (distinct agent_id from snapshots)
    let total_agents: i32 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT agent_id) FROM agent_performance_snapshots"
    )
    .fetch_one(&state.db)
    .await?;

    // Team avg resolution rate (latest snapshots)
    let avg_resolution_row: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(resolution_rate) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_optional(&state.db)
    .await?;

    let team_avg_resolution_rate = avg_resolution_row.map(|r| r.0).unwrap_or(0.0);

    // Team avg response time
    let avg_response_row: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(avg_response_time_seconds) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_optional(&state.db)
    .await?;

    let team_avg_response_time = avg_response_row.map(|r| r.0).unwrap_or(0.0);

    // Team avg satisfaction
    let avg_satisfaction_row: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(customer_satisfaction_avg) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days' AND customer_satisfaction_avg IS NOT NULL"
    )
    .fetch_optional(&state.db)
    .await?;

    let team_avg_satisfaction = avg_satisfaction_row.map(|r| r.0);

    // Needs coaching (below benchmark)
    let needs_coaching: i32 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT agent_id) FROM agent_performance_snapshots \
         WHERE resolution_rate < 0.8 AND window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Exceeds benchmark
    let exceeds_benchmark: i32 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT agent_id) FROM agent_performance_snapshots \
         WHERE resolution_rate >= 0.8 AND window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Active insights
    let active_insights: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_coaching_insights WHERE is_addressed = false"
    )
    .fetch_one(&state.db)
    .await?;

    // Recent snapshots
    let snapshot_rows = sqlx::query(
        "SELECT id, organization_id, agent_id, agent_name, window_start, window_end, \
         conversations_handled, conversations_resolved, avg_response_time_seconds, \
         avg_first_response_time_seconds, avg_resolution_time_seconds, resolution_rate, \
         customer_satisfaction_avg, customer_satisfaction_count, positive_feedback_count, \
         negative_feedback_count, feedback_rate, messages_sent, messages_received, \
         drafts_approved, drafts_rejected, draft_approval_rate, escalations_received, \
         escalations_sent, sla_breaches, sla_compliance_rate, cx_score_avg, qa_score_avg, \
         created_at \
         FROM agent_performance_snapshots ORDER BY window_end DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_snapshots: Vec<AgentPerformanceSnapshot> = snapshot_rows
        .iter()
        .map(|row| AgentPerformanceSnapshot {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            agent_id: row.try_get(2).unwrap_or_default(),
            agent_name: row.try_get(3).unwrap_or_default(),
            window_start: row.try_get(4).unwrap_or_default(),
            window_end: row.try_get(5).unwrap_or_default(),
            conversations_handled: row.try_get(6).unwrap_or(0i32),
            conversations_resolved: row.try_get(7).unwrap_or(0i32),
            avg_response_time_seconds: row.try_get(8).unwrap_or(0.0),
            avg_first_response_time_seconds: row.try_get(9).unwrap_or(0.0),
            avg_resolution_time_seconds: row.try_get(10).unwrap_or(0.0),
            resolution_rate: row.try_get(11).unwrap_or(0.0),
            customer_satisfaction_avg: row.try_get(12).ok(),
            customer_satisfaction_count: row.try_get(13).unwrap_or(0i32),
            positive_feedback_count: row.try_get(14).unwrap_or(0i32),
            negative_feedback_count: row.try_get(15).unwrap_or(0i32),
            feedback_rate: row.try_get(16).unwrap_or(0.0),
            messages_sent: row.try_get(17).unwrap_or(0i32),
            messages_received: row.try_get(18).unwrap_or(0i32),
            drafts_approved: row.try_get(19).unwrap_or(0i32),
            drafts_rejected: row.try_get(20).unwrap_or(0i32),
            draft_approval_rate: row.try_get(21).unwrap_or(0.0),
            escalations_received: row.try_get(22).unwrap_or(0i32),
            escalations_sent: row.try_get(23).unwrap_or(0i32),
            sla_breaches: row.try_get(24).unwrap_or(0i32),
            sla_compliance_rate: row.try_get(25).unwrap_or(1.0),
            cx_score_avg: row.try_get(26).ok(),
            qa_score_avg: row.try_get(27).ok(),
            created_at: row.try_get(28).unwrap_or_default(),
        })
        .collect();

    // Recent insights
    let insight_rows = sqlx::query(
        "SELECT id, organization_id, agent_id, agent_name, insight_type, title, description, \
         category, severity, recommendation, related_conversation_ids, related_metrics, \
         is_addressed, addressed_at, addressed_by, created_at \
         FROM agent_coaching_insights ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_insights: Vec<CoachingInsight> = insight_rows
        .iter()
        .map(|row| CoachingInsight {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            agent_id: row.try_get(2).unwrap_or_default(),
            agent_name: row.try_get(3).unwrap_or_default(),
            insight_type: row.try_get(4).unwrap_or_default(),
            title: row.try_get(5).unwrap_or_default(),
            description: row.try_get(6).unwrap_or_default(),
            category: row.try_get(7).unwrap_or_default(),
            severity: row.try_get(8).unwrap_or_default(),
            recommendation: row.try_get(9).unwrap_or_default(),
            related_conversation_ids: row.try_get(10).ok(),
            related_metrics: row.try_get(11).ok(),
            is_addressed: row.try_get(12).unwrap_or(false),
            addressed_at: row.try_get(13).ok(),
            addressed_by: row.try_get(14).ok(),
            created_at: row.try_get(15).unwrap_or_default(),
        })
        .collect();

    // Latest comparison metrics
    let comparison_row = sqlx::query(
        "SELECT id, organization_id, window_start, window_end, team_avg_response_time, \
         team_avg_resolution_rate, team_avg_satisfaction, team_avg_cx_score, team_avg_qa_score, \
         top_performer_id, top_performer_score, needs_coaching_count, exceeds_benchmark_count, \
         total_agents, created_at \
         FROM agent_comparison_metrics ORDER BY window_end DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let comparison_metrics = comparison_row.map(|row| ComparisonMetrics {
        id: row.try_get(0).unwrap_or_default(),
        organization_id: row.try_get(1).unwrap_or_default(),
        window_start: row.try_get(2).unwrap_or_default(),
        window_end: row.try_get(3).unwrap_or_default(),
        team_avg_response_time: row.try_get(4).unwrap_or(0.0),
        team_avg_resolution_rate: row.try_get(5).unwrap_or(0.0),
        team_avg_satisfaction: row.try_get(6).ok(),
        team_avg_cx_score: row.try_get(7).ok(),
        team_avg_qa_score: row.try_get(8).ok(),
        top_performer_id: row.try_get(9).ok(),
        top_performer_score: row.try_get(10).ok(),
        needs_coaching_count: row.try_get(11).unwrap_or(0i32),
        exceeds_benchmark_count: row.try_get(12).unwrap_or(0i32),
        total_agents: row.try_get(13).unwrap_or(0i32),
        created_at: row.try_get(14).unwrap_or_default(),
    });

    Ok(AgentPerformanceSummary {
        enabled,
        total_agents,
        team_avg_resolution_rate,
        team_avg_response_time,
        team_avg_satisfaction,
        needs_coaching,
        exceeds_benchmark,
        active_insights,
        recent_snapshots,
        recent_insights,
        comparison_metrics,
    })
}

/// Get performance snapshots with optional filters
pub async fn list_snapshots(
    state: &AppState,
    agent_id: Option<&str>,
    days: i32,
) -> Result<Vec<AgentPerformanceSnapshot>, sqlx::Error> {
    let query = if let Some(id) = agent_id {
        format!(
            "SELECT id, organization_id, agent_id, agent_name, window_start, window_end, \
             conversations_handled, conversations_resolved, avg_response_time_seconds, \
             avg_first_response_time_seconds, avg_resolution_time_seconds, resolution_rate, \
             customer_satisfaction_avg, customer_satisfaction_count, positive_feedback_count, \
             negative_feedback_count, feedback_rate, messages_sent, messages_received, \
             drafts_approved, drafts_rejected, draft_approval_rate, escalations_received, \
             escalations_sent, sla_breaches, sla_compliance_rate, cx_score_avg, qa_score_avg, \
             created_at \
             FROM agent_performance_snapshots \
             WHERE agent_id = '{}' AND window_end >= NOW() - INTERVAL '{}' days \
             ORDER BY window_end DESC",
            id, days
        )
    } else {
        format!(
            "SELECT id, organization_id, agent_id, agent_name, window_start, window_end, \
             conversations_handled, conversations_resolved, avg_response_time_seconds, \
             avg_first_response_time_seconds, avg_resolution_time_seconds, resolution_rate, \
             customer_satisfaction_avg, customer_satisfaction_count, positive_feedback_count, \
             negative_feedback_count, feedback_rate, messages_sent, messages_received, \
             drafts_approved, drafts_rejected, draft_approval_rate, escalations_received, \
             escalations_sent, sla_breaches, sla_compliance_rate, cx_score_avg, qa_score_avg, \
             created_at \
             FROM agent_performance_snapshots \
             WHERE window_end >= NOW() - INTERVAL '{}' days \
             ORDER BY window_end DESC",
            days
        )
    };

    let rows = sqlx::query(&query).fetch_all(&state.db).await?;

    let snapshots: Vec<AgentPerformanceSnapshot> = rows
        .iter()
        .map(|row| AgentPerformanceSnapshot {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            agent_id: row.try_get(2).unwrap_or_default(),
            agent_name: row.try_get(3).unwrap_or_default(),
            window_start: row.try_get(4).unwrap_or_default(),
            window_end: row.try_get(5).unwrap_or_default(),
            conversations_handled: row.try_get(6).unwrap_or(0i32),
            conversations_resolved: row.try_get(7).unwrap_or(0i32),
            avg_response_time_seconds: row.try_get(8).unwrap_or(0.0),
            avg_first_response_time_seconds: row.try_get(9).unwrap_or(0.0),
            avg_resolution_time_seconds: row.try_get(10).unwrap_or(0.0),
            resolution_rate: row.try_get(11).unwrap_or(0.0),
            customer_satisfaction_avg: row.try_get(12).ok(),
            customer_satisfaction_count: row.try_get(13).unwrap_or(0i32),
            positive_feedback_count: row.try_get(14).unwrap_or(0i32),
            negative_feedback_count: row.try_get(15).unwrap_or(0i32),
            feedback_rate: row.try_get(16).unwrap_or(0.0),
            messages_sent: row.try_get(17).unwrap_or(0i32),
            messages_received: row.try_get(18).unwrap_or(0i32),
            drafts_approved: row.try_get(19).unwrap_or(0i32),
            drafts_rejected: row.try_get(20).unwrap_or(0i32),
            draft_approval_rate: row.try_get(21).unwrap_or(0.0),
            escalations_received: row.try_get(22).unwrap_or(0i32),
            escalations_sent: row.try_get(23).unwrap_or(0i32),
            sla_breaches: row.try_get(24).unwrap_or(0i32),
            sla_compliance_rate: row.try_get(25).unwrap_or(1.0),
            cx_score_avg: row.try_get(26).ok(),
            qa_score_avg: row.try_get(27).ok(),
            created_at: row.try_get(28).unwrap_or_default(),
        })
        .collect();

    Ok(snapshots)
}

/// Create a performance snapshot for an agent
pub async fn create_snapshot(
    state: &AppState,
    agent_id: &str,
    agent_name: &str,
    data: &Value,
) -> Result<SnapshotResult, sqlx::Error> {
    let snapshot_id = uuid::Uuid::new_v4().to_string();
    let window_start: String = data.get("window_start").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let window_end: String = data.get("window_end").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let conversations_handled: i32 = data.get("conversations_handled").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let conversations_resolved: i32 = data.get("conversations_resolved").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let avg_response_time: f64 = data.get("avg_response_time_seconds").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let resolution_rate: f64 = data.get("resolution_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO agent_performance_snapshots \
         (id, organization_id, agent_id, agent_name, window_start, window_end, \
          conversations_handled, conversations_resolved, avg_response_time_seconds, \
          resolution_rate, created_at) \
         VALUES ($1, 'org-default', $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(&snapshot_id)
    .bind(agent_id)
    .bind(agent_name)
    .bind(&window_start)
    .bind(&window_end)
    .bind(conversations_handled)
    .bind(conversations_resolved)
    .bind(avg_response_time)
    .bind(resolution_rate)
    .bind(&now)
    .execute(&state.db)
    .await?;

    Ok(SnapshotResult {
        snapshot_id,
        agent_id: agent_id.to_string(),
        message: "Performance snapshot created".to_string(),
    })
}

/// List coaching insights
pub async fn list_insights(
    state: &AppState,
    agent_id: Option<&str>,
    addressed: Option<bool>,
) -> Result<Vec<CoachingInsight>, sqlx::Error> {
    let mut conditions = vec![];
    if let Some(id) = agent_id {
        conditions.push(format!("agent_id = '{}'", id));
    }
    if let Some(addr) = addressed {
        conditions.push(format!("is_addressed = {}", addr));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let rows = sqlx::query(
        &format!(
            "SELECT id, organization_id, agent_id, agent_name, insight_type, title, description, \
             category, severity, recommendation, related_conversation_ids, related_metrics, \
             is_addressed, addressed_at, addressed_by, created_at \
             FROM agent_coaching_insights {} ORDER BY created_at DESC LIMIT 50",
            where_clause
        )
    )
    .fetch_all(&state.db)
    .await?;

    let insights: Vec<CoachingInsight> = rows
        .iter()
        .map(|row| CoachingInsight {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            agent_id: row.try_get(2).unwrap_or_default(),
            agent_name: row.try_get(3).unwrap_or_default(),
            insight_type: row.try_get(4).unwrap_or_default(),
            title: row.try_get(5).unwrap_or_default(),
            description: row.try_get(6).unwrap_or_default(),
            category: row.try_get(7).unwrap_or_default(),
            severity: row.try_get(8).unwrap_or_default(),
            recommendation: row.try_get(9).unwrap_or_default(),
            related_conversation_ids: row.try_get(10).ok(),
            related_metrics: row.try_get(11).ok(),
            is_addressed: row.try_get(12).unwrap_or(false),
            addressed_at: row.try_get(13).ok(),
            addressed_by: row.try_get(14).ok(),
            created_at: row.try_get(15).unwrap_or_default(),
        })
        .collect();

    Ok(insights)
}

/// Create a coaching insight
pub async fn create_insight(
    state: &AppState,
    agent_id: &str,
    agent_name: &str,
    data: &Value,
) -> Result<InsightResult, sqlx::Error> {
    let insight_id = uuid::Uuid::new_v4().to_string();
    let title: String = data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description: String = data.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let category: String = data.get("category").and_then(|v| v.as_str()).unwrap_or("response_quality").to_string();
    let severity: String = data.get("severity").and_then(|v| v.as_str()).unwrap_or("low").to_string();
    let recommendation: String = data.get("recommendation").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO agent_coaching_insights \
         (id, organization_id, agent_id, agent_name, title, description, category, severity, recommendation, created_at) \
         VALUES ($1, 'org-default', $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&insight_id)
    .bind(agent_id)
    .bind(agent_name)
    .bind(&title)
    .bind(&description)
    .bind(&category)
    .bind(&severity)
    .bind(&recommendation)
    .bind(&now)
    .execute(&state.db)
    .await?;

    Ok(InsightResult {
        insight_id,
        message: "Coaching insight created".to_string(),
    })
}

/// Mark insight as addressed
pub async fn address_insight(
    state: &AppState,
    insight_id: &str,
    addressed_by: &str,
) -> Result<InsightResult, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE agent_coaching_insights SET is_addressed = true, addressed_at = $1, addressed_by = $2 WHERE id = $3"
    )
    .bind(&now)
    .bind(addressed_by)
    .bind(insight_id)
    .execute(&state.db)
    .await?;

    Ok(InsightResult {
        insight_id: insight_id.to_string(),
        message: "Insight marked as addressed".to_string(),
    })
}

/// Get comparison metrics
pub async fn get_comparison_metrics(
    state: &AppState,
    days: i32,
) -> Result<Vec<ComparisonMetrics>, sqlx::Error> {
    let rows = sqlx::query(
        &format!(
            "SELECT id, organization_id, window_start, window_end, team_avg_response_time, \
             team_avg_resolution_rate, team_avg_satisfaction, team_avg_cx_score, team_avg_qa_score, \
             top_performer_id, top_performer_score, needs_coaching_count, exceeds_benchmark_count, \
             total_agents, created_at \
             FROM agent_comparison_metrics \
             WHERE window_end >= NOW() - INTERVAL '{}' days \
             ORDER BY window_end DESC",
            days
        )
    )
    .fetch_all(&state.db)
    .await?;

    let metrics: Vec<ComparisonMetrics> = rows
        .iter()
        .map(|row| ComparisonMetrics {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            window_start: row.try_get(2).unwrap_or_default(),
            window_end: row.try_get(3).unwrap_or_default(),
            team_avg_response_time: row.try_get(4).unwrap_or(0.0),
            team_avg_resolution_rate: row.try_get(5).unwrap_or(0.0),
            team_avg_satisfaction: row.try_get(6).ok(),
            team_avg_cx_score: row.try_get(7).ok(),
            team_avg_qa_score: row.try_get(8).ok(),
            top_performer_id: row.try_get(9).ok(),
            top_performer_score: row.try_get(10).ok(),
            needs_coaching_count: row.try_get(11).unwrap_or(0i32),
            exceeds_benchmark_count: row.try_get(12).unwrap_or(0i32),
            total_agents: row.try_get(13).unwrap_or(0i32),
            created_at: row.try_get(14).unwrap_or_default(),
        })
        .collect();

    Ok(metrics)
}

/// Generate comparison metrics snapshot
pub async fn generate_comparison_snapshot(state: &AppState) -> Result<ComparisonMetrics, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    let window_start = (now - chrono::Duration::days(7)).to_rfc3339();
    let window_end = now.to_rfc3339();
    let created_at = now.to_rfc3339();

    // Calculate team averages from recent snapshots
    let avg_response: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(avg_response_time_seconds) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_optional(&state.db)
    .await?;

    let avg_resolution: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(resolution_rate) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_optional(&state.db)
    .await?;

    let avg_satisfaction: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(customer_satisfaction_avg) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days' AND customer_satisfaction_avg IS NOT NULL"
    )
    .fetch_optional(&state.db)
    .await?;

    let total_agents: i32 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT agent_id) FROM agent_performance_snapshots \
         WHERE window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    let needs_coaching: i32 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT agent_id) FROM agent_performance_snapshots \
         WHERE resolution_rate < 0.8 AND window_end >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    let exceeds_benchmark: i32 = total_agents.saturating_sub(needs_coaching);

    sqlx::query(
        "INSERT INTO agent_comparison_metrics \
         (id, organization_id, window_start, window_end, team_avg_response_time, \
          team_avg_resolution_rate, team_avg_satisfaction, needs_coaching_count, \
          exceeds_benchmark_count, total_agents, created_at) \
         VALUES ($1, 'org-default', $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(&id)
    .bind(&window_start)
    .bind(&window_end)
    .bind(avg_response.map(|r| r.0).unwrap_or(0.0))
    .bind(avg_resolution.map(|r| r.0).unwrap_or(0.0))
    .bind(avg_satisfaction.map(|r| r.0))
    .bind(needs_coaching)
    .bind(exceeds_benchmark)
    .bind(total_agents)
    .bind(&created_at)
    .execute(&state.db)
    .await?;

    Ok(ComparisonMetrics {
        id,
        organization_id: "org-default".to_string(),
        window_start,
        window_end,
        team_avg_response_time: avg_response.map(|r| r.0).unwrap_or(0.0),
        team_avg_resolution_rate: avg_resolution.map(|r| r.0).unwrap_or(0.0),
        team_avg_satisfaction: avg_satisfaction.map(|r| r.0),
        team_avg_cx_score: None,
        team_avg_qa_score: None,
        top_performer_id: None,
        top_performer_score: None,
        needs_coaching_count: needs_coaching,
        exceeds_benchmark_count: exceeds_benchmark,
        total_agents,
        created_at,
    })
}

/// Get settings
pub async fn get_settings(state: &AppState) -> Result<AgentPerformanceSettings, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, auto_generate_insights, insight_frequency, \
         benchmark_response_time_seconds, benchmark_resolution_rate, benchmark_satisfaction, \
         benchmark_cx_score, coaching_severity_threshold, show_leaderboard, created_at, updated_at \
         FROM agent_performance_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(r) => Ok(AgentPerformanceSettings {
            id: r.try_get(0).unwrap_or_default(),
            organization_id: r.try_get(1).unwrap_or_default(),
            enabled: r.try_get(2).unwrap_or(true),
            auto_generate_insights: r.try_get(3).unwrap_or(true),
            insight_frequency: r.try_get(4).unwrap_or_default(),
            benchmark_response_time_seconds: r.try_get(5).unwrap_or(300i32),
            benchmark_resolution_rate: r.try_get(6).unwrap_or(0.8),
            benchmark_satisfaction: r.try_get(7).unwrap_or(4.0),
            benchmark_cx_score: r.try_get(8).unwrap_or(0.8),
            coaching_severity_threshold: r.try_get(9).unwrap_or_default(),
            show_leaderboard: r.try_get(10).unwrap_or(true),
            created_at: r.try_get(11).unwrap_or_default(),
            updated_at: r.try_get(12).unwrap_or_default(),
        }),
        None => Err(sqlx::Error::RowNotFound),
    }
}

/// Update settings
pub async fn update_settings(
    state: &AppState,
    data: &Value,
) -> Result<SettingsResult, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE agent_performance_settings SET \
         enabled = COALESCE($1, enabled), \
         auto_generate_insights = COALESCE($2, auto_generate_insights), \
         insight_frequency = COALESCE(NULLIF($3, ''), insight_frequency), \
         benchmark_response_time_seconds = COALESCE($4, benchmark_response_time_seconds), \
         benchmark_resolution_rate = COALESCE($5, benchmark_resolution_rate), \
         benchmark_satisfaction = COALESCE($6, benchmark_satisfaction), \
         benchmark_cx_score = COALESCE($7, benchmark_cx_score), \
         coaching_severity_threshold = COALESCE(NULLIF($8, ''), coaching_severity_threshold), \
         show_leaderboard = COALESCE($9, show_leaderboard), \
         updated_at = $10 \
         WHERE organization_id = 'org-default'"
    )
    .bind(data.get("enabled").and_then(|v| v.as_bool()))
    .bind(data.get("auto_generate_insights").and_then(|v| v.as_bool()))
    .bind(data.get("insight_frequency").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("benchmark_response_time_seconds").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(data.get("benchmark_resolution_rate").and_then(|v| v.as_f64()))
    .bind(data.get("benchmark_satisfaction").and_then(|v| v.as_f64()))
    .bind(data.get("benchmark_cx_score").and_then(|v| v.as_f64()))
    .bind(data.get("coaching_severity_threshold").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("show_leaderboard").and_then(|v| v.as_bool()))
    .bind(&now)
    .execute(&state.db)
    .await?;

    Ok(SettingsResult {
        settings_id: "agent-perf-settings-default".to_string(),
        message: "Settings updated".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let summary = AgentPerformanceSummary {
            enabled: true,
            total_agents: 5,
            team_avg_resolution_rate: 0.85,
            team_avg_response_time: 120.5,
            team_avg_satisfaction: Some(4.5),
            needs_coaching: 1,
            exceeds_benchmark: 4,
            active_insights: 3,
            recent_snapshots: vec![],
            recent_insights: vec![],
            comparison_metrics: None,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"total_agents\":5"));
    }

    #[test]
    fn test_snapshot_result() {
        let result = SnapshotResult {
            snapshot_id: "test-123".to_string(),
            agent_id: "agent-1".to_string(),
            message: "Performance snapshot created".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-123"));
    }

    #[test]
    fn test_insight_result() {
        let result = InsightResult {
            insight_id: "insight-456".to_string(),
            message: "Coaching insight created".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("insight-456"));
    }
}
