//! Phase 40: Scheduled Tasks & Cron Jobs Service
//! Automates recurring operations: metric collection, alert evaluation,
//! CSAT survey sending, KB URL re-crawling, analytics snapshots, etc.

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub cron_schedule: String,
    pub next_run_at: Option<String>,
    pub is_active: bool,
    pub config: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskRun {
    pub id: String,
    pub task_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub result: Option<String>,
    pub error_message: Option<String>,
    pub records_processed: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskQueueEntry {
    pub id: String,
    pub task_id: String,
    pub triggered_by: String,
    pub triggered_at: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchedulerSettings {
    pub id: String,
    pub org_id: String,
    pub enabled: bool,
    pub max_concurrent_tasks: i32,
    pub task_timeout_seconds: i32,
    pub retry_failed: bool,
    pub retry_count: i32,
    pub retry_delay_seconds: i32,
    pub log_retention_days: i32,
    pub webhook_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub cron_schedule: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTaskRequest {
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub cron_schedule: Option<String>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchedulerSummary {
    pub total_tasks: i64,
    pub active_tasks: i64,
    pub pending_queue: i64,
    pub total_runs: i64,
    pub successful_runs: i64,
    pub failed_runs: i64,
    pub scheduler_enabled: bool,
    pub task_type_breakdown: Vec<TaskTypeCount>,
    pub recent_runs: Vec<TaskRun>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskTypeCount {
    pub task_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TriggerResult {
    pub queue_id: String,
    pub task_name: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleanupResult {
    pub deleted_runs: i64,
    pub deleted_queue: i64,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct ScheduledTasksService {
    pool: Pool<Postgres>,
}

impl ScheduledTasksService {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get overall scheduler summary
    pub async fn get_summary(&self) -> Result<SchedulerSummary, anyhow::Error> {
        let total_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_tasks")
            .fetch_one(&self.pool).await?;
        let active_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_tasks WHERE is_active = true")
            .fetch_one(&self.pool).await?;
        let pending_queue: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_task_queue WHERE status = 'pending'")
            .fetch_one(&self.pool).await?;
        let total_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_task_runs")
            .fetch_one(&self.pool).await?;
        let successful_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_task_runs WHERE status = 'success'")
            .fetch_one(&self.pool).await?;
        let failed_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_task_runs WHERE status = 'failed'")
            .fetch_one(&self.pool).await?;

        let settings_row = sqlx::query("SELECT enabled FROM scheduler_settings LIMIT 1")
            .fetch_optional(&self.pool).await?;
        let scheduler_enabled = settings_row.map(|r| r.get::<bool, _>("enabled")).unwrap_or(true);

        let type_rows = sqlx::query(
            "SELECT task_type, COUNT(*) as count FROM scheduled_tasks GROUP BY task_type ORDER BY count DESC"
        ).fetch_all(&self.pool).await?;
        let task_type_breakdown: Vec<TaskTypeCount> = type_rows.iter().map(|r| TaskTypeCount {
            task_type: r.get("task_type"),
            count: r.get("count"),
        }).collect();

        let recent_rows = sqlx::query(
            "SELECT id, task_id, started_at, completed_at, status, duration_ms, \
             result, error_message, records_processed \
             FROM scheduled_task_runs ORDER BY started_at DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;
        let recent_runs: Vec<TaskRun> = recent_rows.iter().map(|r| TaskRun {
            id: r.get("id"),
            task_id: r.get("task_id"),
            started_at: r.get("started_at"),
            completed_at: r.get("completed_at"),
            status: r.get("status"),
            duration_ms: r.get("duration_ms"),
            result: r.get("result"),
            error_message: r.get("error_message"),
            records_processed: r.get("records_processed"),
        }).collect();

        Ok(SchedulerSummary {
            total_tasks, active_tasks, pending_queue, total_runs,
            successful_runs, failed_runs, scheduler_enabled,
            task_type_breakdown, recent_runs,
        })
    }

    // ---------------------------------------------------------------------------
    // Task CRUD
    // ---------------------------------------------------------------------------

    pub async fn list_tasks(&self, active_only: bool, task_type: Option<&str>) -> Result<Vec<ScheduledTask>, String> {
        let mut query = String::from("SELECT id, name, description, task_type, cron_schedule, next_run_at, is_active, config, created_at, updated_at FROM scheduled_tasks WHERE 1=1");
        let mut binds: Vec<Box<dyn sqlx::types::Type<sqlx::Postgres> + Send + Sync>> = Vec::new();

        if active_only {
            query.push_str(" AND is_active = true");
        }

        if let Some(tt) = task_type {
            query.push_str(" AND task_type = $1");
            binds.push(Box::new(tt.to_string()));
        }

        query.push_str(" ORDER BY name");

        // Build query with binds
        let rows = if binds.is_empty() {
            sqlx::query(&query).fetch_all(&self.pool).await
        } else {
            // For simplicity, use a single bind approach
            sqlx::query(&query).bind(&*binds[0]).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list tasks: {e}"))?;

        rows.iter().map(|r| Ok(ScheduledTask {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            task_type: r.get("task_type"),
            cron_schedule: r.get("cron_schedule"),
            next_run_at: r.get("next_run_at"),
            is_active: r.get("is_active"),
            config: r.get("config"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn get_task(&self, task_id: &str) -> Result<ScheduledTask, String> {
        let row = sqlx::query(
            "SELECT id, name, description, task_type, cron_schedule, next_run_at, is_active, config, created_at, updated_at \
             FROM scheduled_tasks WHERE id = $1",
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Task '{}' not found", task_id))?;
        Ok(ScheduledTask {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            task_type: row.get("task_type"),
            cron_schedule: row.get("cron_schedule"),
            next_run_at: row.get("next_run_at"),
            is_active: row.get("is_active"),
            config: row.get("config"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn create_task(&self, req: &CreateTaskRequest) -> Result<ScheduledTask, String> {
        let id = Uuid::new_v4().to_string();
        let config = req.config.clone().unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
        let config_str = serde_json::to_string(&config).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            "INSERT INTO scheduled_tasks (id, name, description, task_type, cron_schedule, config) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.task_type)
        .bind(&req.cron_schedule)
        .bind(&config_str)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create task: {e}"))?;

        self.get_task(&id).await
    }

    pub async fn update_task(&self, task_id: &str, req: &UpdateTaskRequest) -> Result<ScheduledTask, String> {
        let mut sets = Vec::new();
        let mut bind_count = 1;
        let mut binds: Vec<String> = Vec::new();

        if let Some(ref desc) = req.description {
            sets.push(format!("description = ${}", bind_count));
            binds.push(desc.clone());
            bind_count += 1;
        }
        if let Some(ref tt) = req.task_type {
            sets.push(format!("task_type = ${}", bind_count));
            binds.push(tt.clone());
            bind_count += 1;
        }
        if let Some(ref cron) = req.cron_schedule {
            sets.push(format!("cron_schedule = ${}", bind_count));
            binds.push(cron.clone());
            bind_count += 1;
        }
        if let Some(active) = req.is_active {
            sets.push(format!("is_active = ${}", bind_count));
            binds.push(active.to_string());
            bind_count += 1;
        }
        if let Some(ref config) = req.config {
            let config_str = serde_json::to_string(config).unwrap_or_else(|_| "{}".to_string());
            sets.push(format!("config = ${}", bind_count));
            binds.push(config_str);
            bind_count += 1;
        }

        if sets.is_empty() {
            return self.get_task(task_id).await;
        }

        sets.push(format!("updated_at = NOW()"));
        let task_bind = bind_count;
        binds.push(task_id.to_string());

        let query = format!("UPDATE scheduled_tasks SET {} WHERE id = ${}", sets.join(", "), task_bind);

        sqlx::query(&query)
            .bind(&binds[0])
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update task: {e}"))?;

        self.get_task(task_id).await
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM scheduled_tasks WHERE id = $1")
            .bind(task_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete task: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Task Execution
    // ---------------------------------------------------------------------------

    /// Trigger a task manually (adds to queue)
    pub async fn trigger_task(&self, task_id: &str, triggered_by: &str) -> Result<TriggerResult, String> {
        let task = self.get_task(task_id).await?;

        if !task.is_active {
            return Err(format!("Task '{}' is not active", task.name));
        }

        let queue_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO scheduled_task_queue (id, task_id, triggered_by) VALUES ($1, $2, $3)",
        )
        .bind(&queue_id)
        .bind(task_id)
        .bind(triggered_by)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to queue task: {e}"))?;

        Ok(TriggerResult {
            queue_id,
            task_name: task.name,
            message: format!("Task '{}' queued for execution", task.name),
        })
    }

    /// Process pending queue entries (called by scheduler or manually)
    pub async fn process_queue(&self) -> Result<Vec<String>, String> {
        let pending = sqlx::query(
            "SELECT id, task_id FROM scheduled_task_queue WHERE status = 'pending' ORDER BY triggered_at LIMIT 5"
        ).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to fetch pending queue: {e}"))?;

        let mut processed = Vec::new();

        for row in pending {
            let queue_id: String = row.get("id");
            let task_id: String = row.get("task_id");

            // Mark as running
            sqlx::query("UPDATE scheduled_task_queue SET status = 'running' WHERE id = $1")
                .bind(&queue_id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to update queue status: {e}"))?;

            // Create a run record
            let run_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO scheduled_task_runs (id, task_id, status) VALUES ($1, $2, 'running')",
            )
            .bind(&run_id)
            .bind(&task_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create run record: {e}"))?;

            // Simulate task execution (production would dispatch to actual handlers)
            let task = match self.get_task(&task_id).await {
                Ok(t) => t,
                Err(_) => {
                    // Task not found, mark as failed
                    sqlx::query(
                        "UPDATE scheduled_task_runs SET status = 'failed', completed_at = NOW(), \
                         error_message = 'Task not found' WHERE id = $1",
                    ).bind(&run_id).execute(&self.pool).await.ok();
                    sqlx::query(
                        "UPDATE scheduled_task_queue SET status = 'failed', completed_at = NOW(), \
                         error_message = 'Task not found' WHERE id = $1",
                    ).bind(&queue_id).execute(&self.pool).await.ok();
                    continue;
                }
            };

            // Simulate processing based on task type
            let records_processed = match task.task_type.as_str() {
                "metric_collection" => 10,
                "alert_evaluation" => 5,
                "csat_send" => 3,
                "kb_crawl" => 2,
                "analytics_snapshot" => 1,
                "operator_scan" => 1,
                "cleanup" => 0,
                _ => 0,
            };

            // Mark as success
            sqlx::query(
                "UPDATE scheduled_task_runs SET status = 'success', completed_at = NOW(), \
                 duration_ms = $1, records_processed = $2, result = $3 WHERE id = $4",
            )
            .bind(100i32)
            .bind(records_processed as i64)
            .bind(format!("{{\"task_type\": \"{}\", \"records\": {}}}", task.task_type, records_processed))
            .bind(&run_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update run: {e}"))?;

            sqlx::query(
                "UPDATE scheduled_task_queue SET status = 'completed', completed_at = NOW() WHERE id = $1",
            )
            .bind(&queue_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update queue: {e}"))?;

            // Update next_run_at on the task
            sqlx::query(
                "UPDATE scheduled_tasks SET next_run_at = NOW(), updated_at = NOW() WHERE id = $1",
            )
            .bind(&task_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update task: {e}"))?;

            processed.push(format!("Processed task '{}'", task.name));
        }

        Ok(processed)
    }

    // ---------------------------------------------------------------------------
    // Task Runs
    // ---------------------------------------------------------------------------

    pub async fn list_runs(&self, task_id: Option<&str>, status: Option<&str>, limit: i64) -> Result<Vec<TaskRun>, String> {
        let mut query = String::from("SELECT id, task_id, started_at, completed_at, status, duration_ms, \
             result, error_message, records_processed FROM scheduled_task_runs WHERE 1=1");

        let mut bind_count = 1;
        let mut binds = Vec::new();

        if let Some(tid) = task_id {
            query.push_str(format!(" AND task_id = ${}", bind_count).as_str());
            binds.push(tid.to_string());
            bind_count += 1;
        }
        if let Some(st) = status {
            query.push_str(format!(" AND status = ${}", bind_count).as_str());
            binds.push(st.to_string());
            bind_count += 1;
        }

        query.push_str(format!(" ORDER BY started_at DESC LIMIT ${}", bind_count).as_str());
        binds.push(limit.to_string());

        let rows = if binds.is_empty() {
            sqlx::query(&query).fetch_all(&self.pool).await
        } else {
            let mut q = sqlx::query(&query);
            for b in &binds {
                q = q.bind(b);
            }
            q.fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list runs: {e}"))?;

        rows.iter().map(|r| Ok(TaskRun {
            id: r.get("id"),
            task_id: r.get("task_id"),
            started_at: r.get("started_at"),
            completed_at: r.get("completed_at"),
            status: r.get("status"),
            duration_ms: r.get("duration_ms"),
            result: r.get("result"),
            error_message: r.get("error_message"),
            records_processed: r.get("records_processed"),
        })).collect()
    }

    // ---------------------------------------------------------------------------
    // Queue
    // ---------------------------------------------------------------------------

    pub async fn list_queue(&self, status: Option<&str>) -> Result<Vec<TaskQueueEntry>, String> {
        let mut query = String::from("SELECT id, task_id, triggered_by, triggered_at, status, completed_at, error_message \
             FROM scheduled_task_queue WHERE 1=1");

        if let Some(st) = status {
            query.push_str(&format!(" AND status = ${}", 1));
        }

        query.push_str(" ORDER BY triggered_at DESC LIMIT 50");

        let rows = if let Some(_) = status {
            sqlx::query(&query).bind(status.unwrap()).fetch_all(&self.pool).await
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list queue: {e}"))?;

        rows.iter().map(|r| Ok(TaskQueueEntry {
            id: r.get("id"),
            task_id: r.get("task_id"),
            triggered_by: r.get("triggered_by"),
            triggered_at: r.get("triggered_at"),
            status: r.get("status"),
            completed_at: r.get("completed_at"),
            error_message: r.get("error_message"),
        })).collect()
    }

    // ---------------------------------------------------------------------------
    // Settings
    // ---------------------------------------------------------------------------

    pub async fn get_settings(&self) -> Result<SchedulerSettings, String> {
        let row = sqlx::query(
            "SELECT id, org_id, enabled, max_concurrent_tasks, task_timeout_seconds, \
             retry_failed, retry_count, retry_delay_seconds, log_retention_days, webhook_url, \
             created_at, updated_at FROM scheduler_settings LIMIT 1",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| "Settings not found".to_string())?;

        Ok(SchedulerSettings {
            id: row.get("id"),
            org_id: row.get("org_id"),
            enabled: row.get("enabled"),
            max_concurrent_tasks: row.get("max_concurrent_tasks"),
            task_timeout_seconds: row.get("task_timeout_seconds"),
            retry_failed: row.get("retry_failed"),
            retry_count: row.get("retry_count"),
            retry_delay_seconds: row.get("retry_delay_seconds"),
            log_retention_days: row.get("log_retention_days"),
            webhook_url: row.get("webhook_url"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn update_settings(&self, settings: &SchedulerSettings) -> Result<SchedulerSettings, String> {
        sqlx::query(
            "UPDATE scheduler_settings SET enabled = $1, max_concurrent_tasks = $2, \
             task_timeout_seconds = $3, retry_failed = $4, retry_count = $5, \
             retry_delay_seconds = $6, log_retention_days = $7, webhook_url = $8, \
             updated_at = NOW() WHERE id = $9",
        )
        .bind(settings.enabled)
        .bind(settings.max_concurrent_tasks)
        .bind(settings.task_timeout_seconds)
        .bind(settings.retry_failed)
        .bind(settings.retry_count)
        .bind(settings.retry_delay_seconds)
        .bind(settings.log_retention_days)
        .bind(&settings.webhook_url)
        .bind(&settings.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update settings: {e}"))?;

        self.get_settings().await
    }

    // ---------------------------------------------------------------------------
    // Cleanup
    // ---------------------------------------------------------------------------

    /// Clean up old task logs
    pub async fn cleanup_old_logs(&self, retention_days: i32) -> Result<CleanupResult, String> {
        let deleted_runs: i64 = sqlx::query_scalar(
            "DELETE FROM scheduled_task_runs WHERE started_at < NOW() - INTERVAL '${1} days'",
        )
        .bind(retention_days.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to clean up runs: {e}"))?
        .rows_affected();

        let deleted_queue: i64 = sqlx::query_scalar(
            "DELETE FROM scheduled_task_queue WHERE triggered_at < NOW() - INTERVAL '${1} days'",
        )
        .bind(retention_days.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to clean up queue: {e}"))?
        .rows_affected();

        Ok(CleanupResult {
            deleted_runs,
            deleted_queue,
            message: format!("Deleted {} runs and {} queue entries older than {} days", deleted_runs, deleted_queue, retention_days),
        })
    }

    /// Get available task types
    pub fn get_available_task_types() -> Vec<String> {
        vec![
            "metric_collection".to_string(),
            "alert_evaluation".to_string(),
            "csat_send".to_string(),
            "kb_crawl".to_string(),
            "analytics_snapshot".to_string(),
            "operator_scan".to_string(),
            "cleanup".to_string(),
            "custom_webhook".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_serialization() {
        let summary = SchedulerSummary {
            total_tasks: 7,
            active_tasks: 6,
            pending_queue: 0,
            total_runs: 100,
            successful_runs: 95,
            failed_runs: 5,
            scheduler_enabled: true,
            task_type_breakdown: vec![],
            recent_runs: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_tasks\":7"));
        assert!(json.contains("\"scheduler_enabled\":true"));
    }

    #[test]
    fn test_trigger_result_serialization() {
        let result = TriggerResult {
            queue_id: "test-queue-id".to_string(),
            task_name: "Test Task".to_string(),
            message: "Task queued".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"task_name\":\"Test Task\""));
    }

    #[test]
    fn test_available_task_types() {
        let types = ScheduledTasksService::get_available_task_types();
        assert!(types.contains(&"metric_collection".to_string()));
        assert!(types.contains(&"alert_evaluation".to_string()));
        assert!(types.len() == 8);
    }
}
