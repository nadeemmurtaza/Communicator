use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedAnalyticsSummary {
    pub total_conversations: i64,
    pub resolved_conversations: i64,
    pub resolution_rate: f64,
    pub avg_response_time_seconds: f64,
    pub cx_score: f64,
    pub csat_score: f64,
    pub feedback_satisfaction_rate: f64,
    pub validation_pass_rate: f64,
    pub total_feedback: i64,
    pub positive_feedback: i64,
    pub negative_feedback: i64,
    pub total_suggestions: i64,
    pub approved_suggestions: i64,
    pub active_alerts: i64,
    pub conversations_by_channel: Vec<ChannelCount>,
    pub conversations_by_status: Vec<StatusCount>,
    pub topic_distribution: Vec<TopicCount>,
    pub agent_leaderboard: Vec<AgentMetric>,
    pub autonomous_stats: AutonomousStats,
    pub procedure_stats: ProcedureStats,
    pub workflow_stats: WorkflowStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelCount {
    pub channel: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicCount {
    pub topic: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMetric {
    pub agent_id: String,
    pub agent_name: String,
    pub conversations_handled: i64,
    pub resolution_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutonomousStats {
    pub total_resolutions: i64,
    pub auto_sent: i64,
    pub escalated: i64,
    pub resolution_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcedureStats {
    pub total_procedures: i64,
    pub active_executions: i64,
    pub completed_executions: i64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowStats {
    pub total_workflows: i64,
    pub active_workflows: i64,
    pub total_executions: i64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedReport {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: serde_json::Value,
    pub time_range: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub filters: serde_json::Value,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_result: Option<serde_json::Value>,
    pub created_by: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledReport {
    pub id: String,
    pub report_id: String,
    pub schedule: String,
    pub cron_expression: Option<String>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub recipients: Vec<String>,
    pub format: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportTemplate {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub report_type: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub is_default: bool,
    pub usage_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardWidget {
    pub id: String,
    pub org_id: String,
    pub dashboard_id: String,
    pub widget_type: String,
    pub title: String,
    pub config: serde_json::Value,
    pub position_x: i32,
    pub position_y: i32,
    pub width: i32,
    pub height: i32,
    pub is_visible: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsSnapshot {
    pub id: String,
    pub org_id: String,
    pub snapshot_type: String,
    pub period_start: String,
    pub period_end: String,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedAnalyticsSettings {
    pub id: String,
    pub org_id: String,
    pub enabled: bool,
    pub auto_snapshot: bool,
    pub snapshot_frequency: String,
    pub retention_days: i32,
    pub default_dashboard: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunReportResult {
    pub report_id: String,
    pub result: serde_json::Value,
    pub run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportResult {
    pub format: String,
    pub data: serde_json::Value,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

pub struct UnifiedAnalyticsService {
    pub pool: PgPool,
}

impl UnifiedAnalyticsService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn get_summary(&self) -> Result<UnifiedAnalyticsSummary, anyhow::Error> {
        // Conversations
        let total_conversations: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM conversations"
        ).fetch_one(&self.pool).await?;

        let resolved_conversations: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM conversations WHERE state IN ('resolved', 'closed')"
        ).fetch_one(&self.pool).await?;

        let resolution_rate = if total_conversations.0 > 0 {
            resolved_conversations.0 as f64 / total_conversations.0 as f64 * 100.0
        } else {
            0.0
        };

        // Conversations by channel
        let conversations_by_channel = sqlx::query_as::<_, ChannelCount>(
            "SELECT channel, COUNT(*) as count FROM conversations GROUP BY channel ORDER BY count DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;

        // Conversations by status
        let conversations_by_status = sqlx::query_as::<_, StatusCount>(
            "SELECT state as status, COUNT(*) as count FROM conversations GROUP BY state ORDER BY count DESC"
        ).fetch_all(&self.pool).await?;

        // Feedback
        let total_feedback: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM message_feedback"
        ).fetch_one(&self.pool).await?;

        let positive_feedback: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM message_feedback WHERE rating > 0"
        ).fetch_one(&self.pool).await?;

        let negative_feedback: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM message_feedback WHERE rating < 0"
        ).fetch_one(&self.pool).await?;

        let feedback_satisfaction_rate = if total_feedback.0 > 0 {
            positive_feedback.0 as f64 / total_feedback.0 as f64 * 100.0
        } else {
            0.0
        };

        // Validation
        let validation_pass_rate = self.get_validation_pass_rate().await.unwrap_or(0.0);

        // CX Score
        let cx_score = self.get_cx_score().await.unwrap_or(0.0);

        // CSAT
        let csat_score = self.get_csat_score().await.unwrap_or(0.0);

        // Topics
        let topic_distribution = self.get_topic_distribution().await.unwrap_or_default();

        // Suggestions
        let total_suggestions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM suggestions"
        ).fetch_one(&self.pool).await.unwrap_or((0,));

        let approved_suggestions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM suggestions WHERE status = 'approved'"
        ).fetch_one(&self.pool).await.unwrap_or((0,));

        // Active alerts
        let active_alerts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM alert_events WHERE status IN ('fired', 'acknowledged')"
        ).fetch_one(&self.pool).await.unwrap_or((0,));

        // Agent leaderboard
        let agent_leaderboard = self.get_agent_leaderboard().await.unwrap_or_default();

        // Autonomous resolution stats
        let autonomous_stats = self.get_autonomous_stats().await.unwrap_or(AutonomousStats {
            total_resolutions: 0,
            auto_sent: 0,
            escalated: 0,
            resolution_rate: 0.0,
        });

        // Procedure stats
        let procedure_stats = self.get_procedure_stats().await.unwrap_or(ProcedureStats {
            total_procedures: 0,
            active_executions: 0,
            completed_executions: 0,
            success_rate: 0.0,
        });

        // Workflow stats
        let workflow_stats = self.get_workflow_stats().await.unwrap_or(WorkflowStats {
            total_workflows: 0,
            active_workflows: 0,
            total_executions: 0,
            success_rate: 0.0,
        });

        Ok(UnifiedAnalyticsSummary {
            total_conversations: total_conversations.0,
            resolved_conversations: resolved_conversations.0,
            resolution_rate,
            avg_response_time_seconds: 0.0,
            cx_score,
            csat_score,
            feedback_satisfaction_rate,
            validation_pass_rate,
            total_feedback: total_feedback.0,
            positive_feedback: positive_feedback.0,
            negative_feedback: negative_feedback.0,
            total_suggestions: total_suggestions.0,
            approved_suggestions: approved_suggestions.0,
            active_alerts: active_alerts.0,
            conversations_by_channel,
            conversations_by_status,
            topic_distribution,
            agent_leaderboard,
            autonomous_stats,
            procedure_stats,
            workflow_stats,
        })
    }

    async fn get_validation_pass_rate(&self) -> Result<f64, anyhow::Error> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM drafts WHERE validation_result IS NOT NULL"
        ).fetch_one(&self.pool).await?;

        if total.0 == 0 {
            return Ok(0.0);
        }

        // Count drafts where validation passed (not pending_review due to validation)
        let passed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM drafts WHERE status != 'pending_review'"
        ).fetch_one(&self.pool).await?;

        Ok(passed.0 as f64 / total.0 as f64 * 100.0)
    }

    async fn get_cx_score(&self) -> Result<f64, anyhow::Error> {
        let row: (sqlx::types::Json<serde_json::Value>,) = sqlx::query_as(
            "SELECT data FROM analytics_snapshots WHERE snapshot_type = 'daily' ORDER BY created_at DESC LIMIT 1"
        ).fetch_optional(&self.pool).await?
        .map(|r| r.0)
        .unwrap_or(serde_json::json!({}));

        // Try to get CX score from snapshot data
        if let Some(cx) = row.get("cx_score").and_then(|v| v.as_f64()) {
            return Ok(cx);
        }

        // Fallback: calculate from qa_scores
        let avg: (sqlx::types::Json<serde_json::Value>,) = sqlx::query_as(
            "SELECT AVG(overall_score) as avg_score FROM qa_scores"
        ).fetch_optional(&self.pool).await?
        .unwrap_or(serde_json::json!(null));

        if let Some(avg_score) = avg.0.as_f64() {
            return Ok(avg_score * 100.0);
        }

        Ok(0.0)
    }

    async fn get_csat_score(&self) -> Result<f64, anyhow::Error> {
        let row: (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), COUNT(CASE WHEN score >= 4 THEN 1 END) FROM csat_surveys WHERE status = 'responded'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0, 0));

        if row.0 == 0 {
            return Ok(0.0);
        }

        Ok(row.1 as f64 / row.0 as f64 * 100.0)
    }

    async fn get_topic_distribution(&self) -> Result<Vec<TopicCount>, anyhow::Error> {
        let topics = sqlx::query_as::<_, TopicCount>(
            "SELECT topic, COUNT(*) as count FROM conversation_topics GROUP BY topic ORDER BY count DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;
        Ok(topics)
    }

    async fn get_agent_leaderboard(&self) -> Result<Vec<AgentMetric>, anyhow::Error> {
        // Get team members and their conversation counts
        let members = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM team_members LIMIT 20"
        ).fetch_all(&self.pool).await?;

        let mut leaderboard = Vec::new();
        for (id, name) in members {
            let handled: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM conversations WHERE assigned_to = $1"
            ).bind(&id).fetch_one(&self.pool).await?;

            let resolved: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM conversations WHERE assigned_to = $1 AND state IN ('resolved', 'closed')"
            ).bind(&id).fetch_one(&self.pool).await?;

            let resolution_rate = if handled.0 > 0 {
                resolved.0 as f64 / handled.0 as f64 * 100.0
            } else {
                0.0
            };

            leaderboard.push(AgentMetric {
                agent_id: id,
                agent_name: name,
                conversations_handled: handled.0,
                resolution_rate,
            });
        }

        Ok(leaderboard)
    }

    async fn get_autonomous_stats(&self) -> Result<AutonomousStats, anyhow::Error> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resolution_logs"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let auto_sent: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resolution_logs WHERE outcome = 'auto_sent'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let escalated: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resolution_logs WHERE outcome = 'escalated'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let resolution_rate = if total.0 > 0 {
            auto_sent.0 as f64 / total.0 as f64 * 100.0
        } else {
            0.0
        };

        Ok(AutonomousStats {
            total_resolutions: total.0,
            auto_sent: auto_sent.0,
            escalated: escalated.0,
            resolution_rate,
        })
    }

    async fn get_procedure_stats(&self) -> Result<ProcedureStats, anyhow::Error> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM procedures"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM procedure_executions WHERE status = 'active'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let completed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM procedure_executions WHERE status = 'completed'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let total_executions = active.0 + completed.0;
        let success_rate = if total_executions > 0 {
            completed.0 as f64 / total_executions as f64 * 100.0
        } else {
            0.0
        };

        Ok(ProcedureStats {
            total_procedures: total.0,
            active_executions: active.0,
            completed_executions: completed.0,
            success_rate,
        })
    }

    async fn get_workflow_stats(&self) -> Result<WorkflowStats, anyhow::Error> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM workflows"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM workflows WHERE status = 'active'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let executions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM workflow_executions"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let success: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM workflow_executions WHERE status = 'completed'"
        ).fetch_optional(&self.pool).await?
        .unwrap_or((0,));

        let success_rate = if executions.0 > 0 {
            success.0 as f64 / executions.0 as f64 * 100.0
        } else {
            0.0
        };

        Ok(WorkflowStats {
            total_workflows: total.0,
            active_workflows: active.0,
            total_executions: executions.0,
            success_rate,
        })
    }

    // Saved Reports
    pub async fn list_saved_reports(&self, org_id: &str, report_type: Option<&str>) -> Result<Vec<SavedReport>, anyhow::Error> {
        let reports = if let Some(report_type) = report_type {
            sqlx::query_as::<_, SavedReport>(
                "SELECT * FROM saved_reports WHERE org_id = $1 AND report_type = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(report_type).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, SavedReport>(
                "SELECT * FROM saved_reports WHERE org_id = $1 ORDER BY created_at DESC"
            ).bind(org_id).fetch_all(&self.pool).await?
        };
        Ok(reports)
    }

    pub async fn create_saved_report(&self, org_id: &str, name: &str, report_type: &str, config: &serde_json::Value, time_range: &str, filters: &serde_json::Value, created_by: &str) -> Result<SavedReport, anyhow::Error> {
        let report = sqlx::query_as::<_, SavedReport>(
            "INSERT INTO saved_reports (org_id, name, report_type, config, time_range, filters, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
        ).bind(org_id).bind(name).bind(report_type).bind(config).bind(time_range).bind(filters).bind(created_by).fetch_one(&self.pool).await?;
        Ok(report)
    }

    pub async fn update_saved_report(&self, id: &str, updates: &serde_json::Value) -> Result<SavedReport, anyhow::Error> {
        let mut query = String::from("UPDATE saved_reports SET updated_at = NOW()");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , name = ${}", param_count));
            params.push(name.to_string());
            param_count += 1;
        }
        if let Some(config) = updates.get("config") {
            query.push_str(&format!(" , config = ${}", param_count));
            params.push(config.to_string());
            param_count += 1;
        }
        if let Some(time_range) = updates.get("time_range").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , time_range = ${}", param_count));
            params.push(time_range.to_string());
            param_count += 1;
        }
        if let Some(filters) = updates.get("filters") {
            query.push_str(&format!(" , filters = ${}", param_count));
            params.push(filters.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(id.to_string());

        let report = sqlx::query_as::<_, SavedReport>(&query)
            .bind(&params[..])
            .fetch_one(&self.pool).await?;
        Ok(report)
    }

    pub async fn delete_saved_report(&self, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM saved_reports WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn run_report(&self, report_id: &str) -> Result<RunReportResult, anyhow::Error> {
        let report = sqlx::query_as::<_, SavedReport>(
            "SELECT * FROM saved_reports WHERE id = $1"
        ).bind(report_id).fetch_one(&self.pool).await?;

        // Generate report result based on type
        let result = match report.report_type.as_str() {
            "unified" => {
                let summary = self.get_summary().await?;
                serde_json::to_value(summary)?
            }
            "conversation" => {
                let summary = self.get_summary().await?;
                serde_json::json!({
                    "total_conversations": summary.total_conversations,
                    "resolved_conversations": summary.resolved_conversations,
                    "resolution_rate": summary.resolution_rate,
                    "by_channel": summary.conversations_by_channel,
                    "by_status": summary.conversations_by_status,
                })
            }
            "agent" => {
                let summary = self.get_summary().await?;
                serde_json::json!({ "agents": summary.agent_leaderboard })
            }
            "cx_qa" => {
                let summary = self.get_summary().await?;
                serde_json::json!({ "cx_score": summary.cx_score })
            }
            "csat" => {
                let summary = self.get_summary().await?;
                serde_json::json!({ "csat_score": summary.csat_score })
            }
            "feedback" => {
                let summary = self.get_summary().await?;
                serde_json::json!({
                    "total_feedback": summary.total_feedback,
                    "positive": summary.positive_feedback,
                    "negative": summary.negative_feedback,
                    "satisfaction_rate": summary.feedback_satisfaction_rate,
                })
            }
            "topic" => {
                let summary = self.get_summary().await?;
                serde_json::json!({ "topics": summary.topic_distribution })
            }
            _ => serde_json::json!({ "error": "Unknown report type" }),
        };

        // Update last run
        sqlx::query(
            "UPDATE saved_reports SET last_run_at = NOW(), last_result = $1 WHERE id = $2"
        ).bind(&result).bind(report_id).execute(&self.pool).await?;

        Ok(RunReportResult {
            report_id: report_id.to_string(),
            result,
            run_at: chrono::Utc::now(),
        })
    }

    // Scheduled Reports
    pub async fn list_scheduled_reports(&self, report_id: Option<&str>) -> Result<Vec<ScheduledReport>, anyhow::Error> {
        let reports = if let Some(report_id) = report_id {
            sqlx::query_as::<_, ScheduledReport>(
                "SELECT * FROM scheduled_reports WHERE report_id = $1 ORDER BY created_at DESC"
            ).bind(report_id).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, ScheduledReport>(
                "SELECT * FROM scheduled_reports ORDER BY created_at DESC"
            ).fetch_all(&self.pool).await?
        };
        Ok(reports)
    }

    pub async fn create_scheduled_report(&self, report_id: &str, schedule: &str, recipients: &[String], format: &str) -> Result<ScheduledReport, anyhow::Error> {
        let report = sqlx::query_as::<_, ScheduledReport>(
            "INSERT INTO scheduled_reports (report_id, schedule, recipients, format)
             VALUES ($1, $2, $3, $4) RETURNING *"
        ).bind(report_id).bind(schedule).bind(recipients).bind(format).fetch_one(&self.pool).await?;
        Ok(report)
    }

    pub async fn update_scheduled_report(&self, id: &str, status: &str) -> Result<ScheduledReport, anyhow::Error> {
        let report = sqlx::query_as::<_, ScheduledReport>(
            "UPDATE scheduled_reports SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(report)
    }

    pub async fn delete_scheduled_report(&self, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM scheduled_reports WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    // Report Templates
    pub async fn list_templates(&self, org_id: &str) -> Result<Vec<ReportTemplate>, anyhow::Error> {
        let templates = sqlx::query_as::<_, ReportTemplate>(
            "SELECT * FROM report_templates WHERE org_id = $1 ORDER BY is_default DESC, name ASC"
        ).bind(org_id).fetch_all(&self.pool).await?;
        Ok(templates)
    }

    pub async fn create_template(&self, org_id: &str, name: &str, report_type: &str, description: &str, config: &serde_json::Value) -> Result<ReportTemplate, anyhow::Error> {
        let template = sqlx::query_as::<_, ReportTemplate>(
            "INSERT INTO report_templates (org_id, name, report_type, description, config)
             VALUES ($1, $2, $3, $4, $5) RETURNING *"
        ).bind(org_id).bind(name).bind(report_type).bind(description).bind(config).fetch_one(&self.pool).await?;
        Ok(template)
    }

    pub async fn use_template(&self, template_id: &str) -> Result<(), anyhow::Error> {
        sqlx::query(
            "UPDATE report_templates SET usage_count = usage_count + 1 WHERE id = $1"
        ).bind(template_id).execute(&self.pool).await?;
        Ok(())
    }

    // Dashboard Widgets
    pub async fn list_widgets(&self, org_id: &str, dashboard_id: &str) -> Result<Vec<DashboardWidget>, anyhow::Error> {
        let widgets = sqlx::query_as::<_, DashboardWidget>(
            "SELECT * FROM dashboard_widgets WHERE org_id = $1 AND dashboard_id = $2 AND is_visible = true ORDER BY position_y ASC, position_x ASC"
        ).bind(org_id).bind(dashboard_id).fetch_all(&self.pool).await?;
        Ok(widgets)
    }

    pub async fn create_widget(&self, org_id: &str, dashboard_id: &str, widget_type: &str, title: &str, config: &serde_json::Value, position_x: i32, position_y: i32, width: i32, height: i32) -> Result<DashboardWidget, anyhow::Error> {
        let widget = sqlx::query_as::<_, DashboardWidget>(
            "INSERT INTO dashboard_widgets (org_id, dashboard_id, widget_type, title, config, position_x, position_y, width, height)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"
        ).bind(org_id).bind(dashboard_id).bind(widget_type).bind(title).bind(config)
         .bind(position_x).bind(position_y).bind(width).bind(height).fetch_one(&self.pool).await?;
        Ok(widget)
    }

    pub async fn update_widget(&self, id: &str, updates: &serde_json::Value) -> Result<DashboardWidget, anyhow::Error> {
        let mut query = String::from("UPDATE dashboard_widgets SET updated_at = NOW()");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(title) = updates.get("title").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , title = ${}", param_count));
            params.push(title.to_string());
            param_count += 1;
        }
        if let Some(config) = updates.get("config") {
            query.push_str(&format!(" , config = ${}", param_count));
            params.push(config.to_string());
            param_count += 1;
        }
        if let Some(is_visible) = updates.get("is_visible").and_then(|v| v.as_bool()) {
            query.push_str(&format!(" , is_visible = ${}", param_count));
            params.push(is_visible.to_string());
            param_count += 1;
        }
        if let Some(position_x) = updates.get("position_x").and_then(|v| v.as_i64()) {
            query.push_str(&format!(" , position_x = ${}", param_count));
            params.push(position_x.to_string());
            param_count += 1;
        }
        if let Some(position_y) = updates.get("position_y").and_then(|v| v.as_i64()) {
            query.push_str(&format!(" , position_y = ${}", param_count));
            params.push(position_y.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(id.to_string());

        let widget = sqlx::query_as::<_, DashboardWidget>(&query)
            .bind(&params[..])
            .fetch_one(&self.pool).await?;
        Ok(widget)
    }

    pub async fn delete_widget(&self, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM dashboard_widgets WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    // Analytics Snapshots
    pub async fn create_snapshot(&self, org_id: &str, snapshot_type: &str, period_start: &str, period_end: &str) -> Result<AnalyticsSnapshot, anyhow::Error> {
        let summary = self.get_summary().await?;
        let data = serde_json::to_value(summary)?;

        let snapshot = sqlx::query_as::<_, AnalyticsSnapshot>(
            "INSERT INTO analytics_snapshots (org_id, snapshot_type, period_start, period_end, data)
             VALUES ($1, $2, $3, $4, $5) RETURNING *"
        ).bind(org_id).bind(snapshot_type).bind(period_start).bind(period_end).bind(&data).fetch_one(&self.pool).await?;
        Ok(snapshot)
    }

    pub async fn list_snapshots(&self, org_id: &str, snapshot_type: Option<&str>, limit: i64) -> Result<Vec<AnalyticsSnapshot>, anyhow::Error> {
        let snapshots = if let Some(snapshot_type) = snapshot_type {
            sqlx::query_as::<_, AnalyticsSnapshot>(
                "SELECT * FROM analytics_snapshots WHERE org_id = $1 AND snapshot_type = $2 ORDER BY created_at DESC LIMIT $3"
            ).bind(org_id).bind(snapshot_type).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, AnalyticsSnapshot>(
                "SELECT * FROM analytics_snapshots WHERE org_id = $1 ORDER BY created_at DESC LIMIT $2"
            ).bind(org_id).bind(limit).fetch_all(&self.pool).await?
        };
        Ok(snapshots)
    }

    // Settings
    pub async fn get_settings(&self, org_id: &str) -> Result<UnifiedAnalyticsSettings, anyhow::Error> {
        let settings = sqlx::query_as::<_, UnifiedAnalyticsSettings>(
            "SELECT * FROM unified_analytics_settings WHERE org_id = $1"
        ).bind(org_id).fetch_optional(&self.pool).await?;

        match settings {
            Some(s) => Ok(s),
            None => {
                let settings = sqlx::query_as::<_, UnifiedAnalyticsSettings>(
                    "INSERT INTO unified_analytics_settings (org_id, enabled, auto_snapshot)
                     VALUES ($1, true, true) RETURNING *"
                ).bind(org_id).fetch_one(&self.pool).await?;
                Ok(settings)
            }
        }
    }

    pub async fn update_settings(&self, org_id: &str, updates: &serde_json::Value) -> Result<UnifiedAnalyticsSettings, anyhow::Error> {
        let mut query = String::from("UPDATE unified_analytics_settings SET updated_at = NOW()");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(enabled) = updates.get("enabled").and_then(|v| v.as_bool()) {
            query.push_str(&format!(" , enabled = ${}", param_count));
            params.push(enabled.to_string());
            param_count += 1;
        }
        if let Some(auto_snapshot) = updates.get("auto_snapshot").and_then(|v| v.as_bool()) {
            query.push_str(&format!(" , auto_snapshot = ${}", param_count));
            params.push(auto_snapshot.to_string());
            param_count += 1;
        }
        if let Some(snapshot_frequency) = updates.get("snapshot_frequency").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , snapshot_frequency = ${}", param_count));
            params.push(snapshot_frequency.to_string());
            param_count += 1;
        }
        if let Some(retention_days) = updates.get("retention_days").and_then(|v| v.as_i64()) {
            query.push_str(&format!(" , retention_days = ${}", param_count));
            params.push(retention_days.to_string());
            param_count += 1;
        }
        if let Some(default_dashboard) = updates.get("default_dashboard").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , default_dashboard = ${}", param_count));
            params.push(default_dashboard.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE org_id = ${} RETURNING *", param_count));
        params.push(org_id.to_string());

        let settings = sqlx::query_as::<_, UnifiedAnalyticsSettings>(&query)
            .bind(&params[..])
            .fetch_one(&self.pool).await?;
        Ok(settings)
    }

    // Export
    pub async fn export_report(&self, report_id: &str, format: &str) -> Result<ExportResult, anyhow::Error> {
        let run_result = self.run_report(report_id).await?;

        Ok(ExportResult {
            format: format.to_string(),
            data: run_result.result,
            exported_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_serialization() {
        let summary = UnifiedAnalyticsSummary {
            total_conversations: 0,
            resolved_conversations: 0,
            resolution_rate: 0.0,
            avg_response_time_seconds: 0.0,
            cx_score: 0.0,
            csat_score: 0.0,
            feedback_satisfaction_rate: 0.0,
            validation_pass_rate: 0.0,
            total_feedback: 0,
            positive_feedback: 0,
            negative_feedback: 0,
            total_suggestions: 0,
            approved_suggestions: 0,
            active_alerts: 0,
            conversations_by_channel: vec![],
            conversations_by_status: vec![],
            topic_distribution: vec![],
            agent_leaderboard: vec![],
            autonomous_stats: AutonomousStats {
                total_resolutions: 0,
                auto_sent: 0,
                escalated: 0,
                resolution_rate: 0.0,
            },
            procedure_stats: ProcedureStats {
                total_procedures: 0,
                active_executions: 0,
                completed_executions: 0,
                success_rate: 0.0,
            },
            workflow_stats: WorkflowStats {
                total_workflows: 0,
                active_workflows: 0,
                total_executions: 0,
                success_rate: 0.0,
            },
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_conversations\""));
    }

    #[test]
    fn test_run_report_result_serialization() {
        let result = RunReportResult {
            report_id: "test".to_string(),
            result: serde_json::json!({}),
            run_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"report_id\""));
    }

    #[test]
    fn test_export_result_serialization() {
        let result = ExportResult {
            format: "csv".to_string(),
            data: serde_json::json!({}),
            exported_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"format\""));
    }
}
