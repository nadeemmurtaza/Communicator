use newax_scheduled_tasks::{
    CleanupResult as LogicCleanupResult, CreateTaskRequest as LogicCreateTaskRequest, ScheduledTask as LogicTask,
    ScheduledTasksService, SchedulerSettings as LogicSettings, SchedulerSummary as LogicSummary,
    TaskQueueEntry as LogicQueueEntry, TaskRun as LogicRun, TaskTypeCount as LogicTypeCount,
    TriggerResult as LogicTriggerResult, UpdateTaskRequest as LogicUpdateTaskRequest,
};
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.scheduled_tasks.v1");
}

use proto::{
    scheduled_tasks_service_server::{ScheduledTasksService as ScheduledTasksRpc, ScheduledTasksServiceServer},
    CleanupRequest, CleanupResult, CreateTaskRequest, Empty, ListQueueRequest, ListRunsRequest, ListTasksRequest,
    ScheduledTask, SchedulerSettings, SchedulerSummary, StringList, TaskIdRequest, TaskList, TaskQueueEntry,
    TaskQueueList, TaskRun, TaskRunList, TaskTypeCount, TriggerResult, UpdateTaskRequest,
};

struct Svc {
    service: ScheduledTasksService,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn task_to_proto(t: LogicTask) -> ScheduledTask {
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

fn run_to_proto(r: LogicRun) -> TaskRun {
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

fn queue_to_proto(q: LogicQueueEntry) -> TaskQueueEntry {
    TaskQueueEntry {
        id: q.id,
        task_id: q.task_id,
        triggered_by: q.triggered_by,
        triggered_at: q.triggered_at,
        status: q.status,
        completed_at: q.completed_at,
        error_message: q.error_message,
    }
}

fn settings_to_proto(s: LogicSettings) -> SchedulerSettings {
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

fn settings_from_proto(s: SchedulerSettings) -> LogicSettings {
    LogicSettings {
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

fn summary_to_proto(s: LogicSummary) -> SchedulerSummary {
    SchedulerSummary {
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
            .map(|t: LogicTypeCount| TaskTypeCount { task_type: t.task_type, count: t.count })
            .collect(),
        recent_runs: s.recent_runs.into_iter().map(run_to_proto).collect(),
    }
}

fn parse_config(s: Option<String>) -> Option<serde_json::Value> {
    s.and_then(|s| serde_json::from_str(&s).ok())
}

#[tonic::async_trait]
impl ScheduledTasksRpc for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<SchedulerSummary>, Status> {
        let s = self.service.get_summary().await.map_err(to_status)?;
        Ok(Response::new(summary_to_proto(s)))
    }

    async fn list_tasks(&self, req: Request<ListTasksRequest>) -> Result<Response<TaskList>, Status> {
        let r = req.into_inner();
        let tasks = self.service.list_tasks(r.active_only, r.task_type.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(TaskList { tasks: tasks.into_iter().map(task_to_proto).collect() }))
    }

    async fn get_task(&self, req: Request<TaskIdRequest>) -> Result<Response<ScheduledTask>, Status> {
        let t = self.service.get_task(&req.into_inner().task_id).await.map_err(Status::not_found)?;
        Ok(Response::new(task_to_proto(t)))
    }

    async fn create_task(&self, req: Request<CreateTaskRequest>) -> Result<Response<ScheduledTask>, Status> {
        let r = req.into_inner();
        let logic_req = LogicCreateTaskRequest {
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            cron_schedule: r.cron_schedule,
            config: parse_config(r.config_json),
        };
        let t = self.service.create_task(&logic_req).await.map_err(to_status)?;
        Ok(Response::new(task_to_proto(t)))
    }

    async fn update_task(&self, req: Request<UpdateTaskRequest>) -> Result<Response<ScheduledTask>, Status> {
        let r = req.into_inner();
        let logic_req = LogicUpdateTaskRequest {
            description: r.description,
            task_type: r.task_type,
            cron_schedule: r.cron_schedule,
            is_active: r.is_active,
            config: parse_config(r.config_json),
        };
        let t = self.service.update_task(&r.task_id, &logic_req).await.map_err(to_status)?;
        Ok(Response::new(task_to_proto(t)))
    }

    async fn delete_task(&self, req: Request<TaskIdRequest>) -> Result<Response<Empty>, Status> {
        self.service.delete_task(&req.into_inner().task_id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn trigger_task(&self, req: Request<TaskIdRequest>) -> Result<Response<TriggerResult>, Status> {
        let r = self.service.trigger_task(&req.into_inner().task_id, "api").await.map_err(to_status)?;
        let r: LogicTriggerResult = r;
        Ok(Response::new(TriggerResult { queue_id: r.queue_id, task_name: r.task_name, message: r.message }))
    }

    async fn process_queue(&self, _req: Request<Empty>) -> Result<Response<StringList>, Status> {
        let values = self.service.process_queue().await.map_err(to_status)?;
        Ok(Response::new(StringList { values }))
    }

    async fn list_runs(&self, req: Request<ListRunsRequest>) -> Result<Response<TaskRunList>, Status> {
        let r = req.into_inner();
        let runs = self
            .service
            .list_runs(r.task_id.as_deref(), r.status.as_deref(), r.limit)
            .await
            .map_err(to_status)?;
        Ok(Response::new(TaskRunList { runs: runs.into_iter().map(run_to_proto).collect() }))
    }

    async fn list_queue(&self, req: Request<ListQueueRequest>) -> Result<Response<TaskQueueList>, Status> {
        let entries = self.service.list_queue(req.into_inner().status.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(TaskQueueList { entries: entries.into_iter().map(queue_to_proto).collect() }))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<SchedulerSettings>, Status> {
        let s = self.service.get_settings().await.map_err(to_status)?;
        Ok(Response::new(settings_to_proto(s)))
    }

    async fn update_settings(&self, req: Request<SchedulerSettings>) -> Result<Response<SchedulerSettings>, Status> {
        let logic_settings = settings_from_proto(req.into_inner());
        let s = self.service.update_settings(&logic_settings).await.map_err(to_status)?;
        Ok(Response::new(settings_to_proto(s)))
    }

    async fn cleanup_old_logs(&self, req: Request<CleanupRequest>) -> Result<Response<CleanupResult>, Status> {
        let r: LogicCleanupResult =
            self.service.cleanup_old_logs(req.into_inner().retention_days).await.map_err(to_status)?;
        Ok(Response::new(CleanupResult {
            deleted_runs: r.deleted_runs,
            deleted_queue: r.deleted_queue,
            message: r.message,
        }))
    }

    async fn get_task_types(&self, _req: Request<Empty>) -> Result<Response<StringList>, Status> {
        Ok(Response::new(StringList { values: ScheduledTasksService::get_available_task_types() }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_scheduled_tasks=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("SCHEDULED_TASKS_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50065);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = Svc { service: ScheduledTasksService::new(pool) };

    info!(port, "svc-scheduled-tasks starting (gRPC)");
    Server::builder()
        .add_service(ScheduledTasksServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
