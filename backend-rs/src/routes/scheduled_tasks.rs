//! Phase 40: Scheduled Tasks & Cron Jobs API Routes -- thin gRPC client
//! for svc-scheduled-tasks. See crates/scheduled-tasks and
//! services/scheduled-tasks. REST paths and payload shapes unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use newax_proto::scheduled_tasks::v1::{
    scheduled_tasks_service_client::ScheduledTasksServiceClient, CleanupRequest, CreateTaskRequest, Empty,
    ListQueueRequest, ListRunsRequest, ListTasksRequest, SchedulerSettings as ProtoSettings, TaskIdRequest,
    UpdateTaskRequest,
};
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;

use crate::AppState;

async fn client() -> Result<ScheduledTasksServiceClient<Channel>, (StatusCode, String)> {
    let url = std::env::var("SCHEDULED_TASKS_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50065".to_string());
    ScheduledTasksServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("scheduled-tasks service unreachable: {e}")))
}

fn grpc_err(status: tonic::Status) -> (StatusCode, String) {
    let code = match status.code() {
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, status.message().to_string())
}

// ---------------------------------------------------------------------------
// Wire types -- mirror the old crate::services::scheduled_tasks types so
// the JSON shape on the wire is unchanged.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct TaskQueueEntry {
    pub id: String,
    pub task_id: String,
    pub triggered_by: String,
    pub triggered_at: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize)]
pub struct TaskTypeCount {
    pub task_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct TriggerResult {
    pub queue_id: String,
    pub task_name: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CleanupResult {
    pub deleted_runs: i64,
    pub deleted_queue: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskBody {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub cron_schedule: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskBody {
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub cron_schedule: Option<String>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
}

fn run_from_proto(r: newax_proto::scheduled_tasks::v1::TaskRun) -> TaskRun {
    TaskRun {
        id: r.id,
        task_id: r.task_id,
        started_at: r.started_at,
        completed_at: r.completed_at,
        status: r.status,
        duration_ms: r.duration_ms,
        result: r.result,
        error_message: r.error_message,
        records_processed: r.records_processed,
    }
}

fn task_from_proto(t: newax_proto::scheduled_tasks::v1::ScheduledTask) -> ScheduledTask {
    ScheduledTask {
        id: t.id,
        name: t.name,
        description: t.description,
        task_type: t.task_type,
        cron_schedule: t.cron_schedule,
        next_run_at: t.next_run_at,
        is_active: t.is_active,
        config: t.config,
        created_at: t.created_at,
        updated_at: t.updated_at,
    }
}

fn settings_from_proto(s: ProtoSettings) -> SchedulerSettings {
    SchedulerSettings {
        id: s.id,
        org_id: s.org_id,
        enabled: s.enabled,
        max_concurrent_tasks: s.max_concurrent_tasks,
        task_timeout_seconds: s.task_timeout_seconds,
        retry_failed: s.retry_failed,
        retry_count: s.retry_count,
        retry_delay_seconds: s.retry_delay_seconds,
        log_retention_days: s.log_retention_days,
        webhook_url: s.webhook_url,
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

fn settings_to_proto(s: SchedulerSettings) -> ProtoSettings {
    ProtoSettings {
        id: s.id,
        org_id: s.org_id,
        enabled: s.enabled,
        max_concurrent_tasks: s.max_concurrent_tasks,
        task_timeout_seconds: s.task_timeout_seconds,
        retry_failed: s.retry_failed,
        retry_count: s.retry_count,
        retry_delay_seconds: s.retry_delay_seconds,
        log_retention_days: s.log_retention_days,
        webhook_url: s.webhook_url,
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Query Params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub active_only: Option<bool>,
    pub task_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListQueueQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupQuery {
    pub retention_days: Option<i32>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn get_summary(State(_s): State<AppState>) -> Result<Json<SchedulerSummary>, (StatusCode, String)> {
    let mut c = client().await?;
    let s = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(SchedulerSummary {
        total_tasks: s.total_tasks,
        active_tasks: s.active_tasks,
        pending_queue: s.pending_queue,
        total_runs: s.total_runs,
        successful_runs: s.successful_runs,
        failed_runs: s.failed_runs,
        scheduler_enabled: s.scheduler_enabled,
        task_type_breakdown: s
            .task_type_breakdown
            .into_iter()
            .map(|t| TaskTypeCount { task_type: t.task_type, count: t.count })
            .collect(),
        recent_runs: s.recent_runs.into_iter().map(run_from_proto).collect(),
    }))
}

pub async fn list_tasks(
    State(_s): State<AppState>,
    Query(params): Query<ListTasksQuery>,
) -> Result<Json<Vec<ScheduledTask>>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = ListTasksRequest { active_only: params.active_only.unwrap_or(false), task_type: params.task_type };
    let tasks = c.list_tasks(req).await.map_err(grpc_err)?.into_inner().tasks;
    Ok(Json(tasks.into_iter().map(task_from_proto).collect()))
}

pub async fn get_task(
    State(_s): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ScheduledTask>, (StatusCode, String)> {
    let mut c = client().await?;
    let t = c.get_task(TaskIdRequest { task_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(task_from_proto(t)))
}

pub async fn create_task(
    State(_s): State<AppState>,
    Json(req): Json<CreateTaskBody>,
) -> Result<Json<ScheduledTask>, (StatusCode, String)> {
    let mut c = client().await?;
    let proto_req = CreateTaskRequest {
        name: req.name,
        description: req.description,
        task_type: req.task_type,
        cron_schedule: req.cron_schedule,
        config_json: req.config.map(|v| v.to_string()),
    };
    let t = c.create_task(proto_req).await.map_err(grpc_err)?.into_inner();
    Ok(Json(task_from_proto(t)))
}

pub async fn update_task(
    State(_s): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<UpdateTaskBody>,
) -> Result<Json<ScheduledTask>, (StatusCode, String)> {
    let mut c = client().await?;
    let proto_req = UpdateTaskRequest {
        task_id,
        description: req.description,
        task_type: req.task_type,
        cron_schedule: req.cron_schedule,
        is_active: req.is_active,
        config_json: req.config.map(|v| v.to_string()),
    };
    let t = c.update_task(proto_req).await.map_err(grpc_err)?.into_inner();
    Ok(Json(task_from_proto(t)))
}

pub async fn delete_task(
    State(_s): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client().await?;
    c.delete_task(TaskIdRequest { task_id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn trigger_task(
    State(_s): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TriggerResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.trigger_task(TaskIdRequest { task_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(TriggerResult { queue_id: r.queue_id, task_name: r.task_name, message: r.message }))
}

pub async fn process_queue(State(_s): State<AppState>) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.process_queue(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(r.values))
}

pub async fn list_runs(
    State(_s): State<AppState>,
    Query(params): Query<ListRunsQuery>,
) -> Result<Json<Vec<TaskRun>>, (StatusCode, String)> {
    let mut c = client().await?;
    let req = ListRunsRequest { task_id: params.task_id, status: params.status, limit: params.limit.unwrap_or(50) };
    let runs = c.list_runs(req).await.map_err(grpc_err)?.into_inner().runs;
    Ok(Json(runs.into_iter().map(run_from_proto).collect()))
}

pub async fn list_queue(
    State(_s): State<AppState>,
    Query(params): Query<ListQueueQuery>,
) -> Result<Json<Vec<TaskQueueEntry>>, (StatusCode, String)> {
    let mut c = client().await?;
    let entries = c.list_queue(ListQueueRequest { status: params.status }).await.map_err(grpc_err)?.into_inner().entries;
    Ok(Json(
        entries
            .into_iter()
            .map(|q| TaskQueueEntry {
                id: q.id,
                task_id: q.task_id,
                triggered_by: q.triggered_by,
                triggered_at: q.triggered_at,
                status: q.status,
                completed_at: q.completed_at,
                error_message: q.error_message,
            })
            .collect(),
    ))
}

pub async fn get_settings(State(_s): State<AppState>) -> Result<Json<SchedulerSettings>, (StatusCode, String)> {
    let mut c = client().await?;
    let s = c.get_settings(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(settings_from_proto(s)))
}

pub async fn update_settings(
    State(_s): State<AppState>,
    Json(settings): Json<SchedulerSettings>,
) -> Result<Json<SchedulerSettings>, (StatusCode, String)> {
    let mut c = client().await?;
    let s = c.update_settings(settings_to_proto(settings)).await.map_err(grpc_err)?.into_inner();
    Ok(Json(settings_from_proto(s)))
}

pub async fn cleanup_old_logs(
    State(_s): State<AppState>,
    Query(params): Query<CleanupQuery>,
) -> Result<Json<CleanupResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let retention_days = params.retention_days.unwrap_or(30);
    let r = c.cleanup_old_logs(CleanupRequest { retention_days }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(CleanupResult { deleted_runs: r.deleted_runs, deleted_queue: r.deleted_queue, message: r.message }))
}

pub async fn get_task_types() -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_task_types(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(r.values))
}
