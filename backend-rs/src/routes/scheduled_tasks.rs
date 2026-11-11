//! Phase 40: Scheduled Tasks & Cron Jobs API Routes

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::AppState;
use crate::services::scheduled_tasks::{
    ScheduledTasksService, ScheduledTask, SchedulerSummary, TaskRun,
    TaskQueueEntry, SchedulerSettings, CreateTaskRequest, UpdateTaskRequest,
    TriggerResult, CleanupResult,
};

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

/// GET /scheduled-tasks/summary
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<SchedulerSummary>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.get_summary().await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /scheduled-tasks
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<ListTasksQuery>,
) -> Result<Json<Vec<ScheduledTask>>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.list_tasks(params.active_only.unwrap_or(false), params.task_type.as_deref()).await {
        Ok(tasks) => Ok(Json(tasks)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /scheduled-tasks/{id}
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ScheduledTask>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.get_task(&task_id).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

/// POST /scheduled-tasks
pub async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<ScheduledTask>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.create_task(&req).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// PUT /scheduled-tasks/{id}
pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<ScheduledTask>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.update_task(&task_id, &req).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// DELETE /scheduled-tasks/{id}
pub async fn delete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.delete_task(&task_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// POST /scheduled-tasks/{id}/trigger
pub async fn trigger_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TriggerResult>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.trigger_task(&task_id, "api").await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// POST /scheduled-tasks/process-queue
pub async fn process_queue(State(state): State<AppState>) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.process_queue().await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /scheduled-tasks/runs
pub async fn list_runs(
    State(state): State<AppState>,
    Query(params): Query<ListRunsQuery>,
) -> Result<Json<Vec<TaskRun>>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.list_runs(params.task_id.as_deref(), params.status.as_deref(), params.limit.unwrap_or(50)).await {
        Ok(runs) => Ok(Json(runs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /scheduled-tasks/queue
pub async fn list_queue(
    State(state): State<AppState>,
    Query(params): Query<ListQueueQuery>,
) -> Result<Json<Vec<TaskQueueEntry>>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.list_queue(params.status.as_deref()).await {
        Ok(queue) => Ok(Json(queue)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /scheduled-tasks/settings
pub async fn get_settings(State(state): State<AppState>) -> Result<Json<SchedulerSettings>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.get_settings().await {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// PUT /scheduled-tasks/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<SchedulerSettings>,
) -> Result<Json<SchedulerSettings>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    match service.update_settings(&settings).await {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// POST /scheduled-tasks/cleanup
pub async fn cleanup_old_logs(
    State(state): State<AppState>,
    Query(params): Query<CleanupQuery>,
) -> Result<Json<CleanupResult>, (StatusCode, String)> {
    let service = ScheduledTasksService::new(&state.pool);
    let retention = params.retention_days.unwrap_or(30);
    match service.cleanup_old_logs(retention).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// GET /scheduled-tasks/task-types
pub async fn get_task_types() -> Json<Vec<String>> {
    Json(ScheduledTasksService::get_available_task_types())
}
