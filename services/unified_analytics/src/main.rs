use newax_unified_analytics as ua;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.unified_analytics.v1");
}

use proto::{
    unified_analytics_service_server::{UnifiedAnalyticsService, UnifiedAnalyticsServiceServer},
    CreateReportRequest, CreateScheduledRequest, CreateSnapshotRequest, CreateTemplateRequest, CreateWidgetRequest, Empty,
    ExportRequest, IdRequest, JsonResponse, ListReportsRequest, ListSnapshotsRequest, OptionalIdRequest, OrgIdRequest,
    UpdateByIdRequest, UpdateByOrgRequest, UpdateStatusRequest, WidgetQueryRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn json_ok<T: serde::Serialize>(v: &T) -> Result<Response<JsonResponse>, Status> {
    Ok(Response::new(JsonResponse {
        json: serde_json::to_string(v).map_err(to_status)?,
    }))
}

fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or(serde_json::Value::Object(Default::default()))
}

#[tonic::async_trait]
impl UnifiedAnalyticsService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let summary = ua::UnifiedAnalyticsService::new(&self.pool).get_summary().await.map_err(to_status)?;
        json_ok(&summary)
    }

    async fn list_saved_reports(&self, req: Request<ListReportsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let reports = ua::UnifiedAnalyticsService::new(&self.pool)
            .list_saved_reports(&r.org_id, r.report_type.as_deref())
            .await
            .map_err(to_status)?;
        json_ok(&reports)
    }

    async fn create_saved_report(&self, req: Request<CreateReportRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let report = ua::UnifiedAnalyticsService::new(&self.pool)
            .create_saved_report(
                &r.org_id,
                &r.name,
                &r.report_type,
                &parse_json(&r.config_json),
                &r.time_range,
                &parse_json(&r.filters_json),
                &r.created_by,
            )
            .await
            .map_err(to_status)?;
        json_ok(&report)
    }

    async fn update_saved_report(&self, req: Request<UpdateByIdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let report = ua::UnifiedAnalyticsService::new(&self.pool)
            .update_saved_report(&r.id, &parse_json(&r.updates_json))
            .await
            .map_err(to_status)?;
        json_ok(&report)
    }

    async fn delete_saved_report(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        ua::UnifiedAnalyticsService::new(&self.pool).delete_saved_report(&req.into_inner().id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn run_report(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let result = ua::UnifiedAnalyticsService::new(&self.pool).run_report(&req.into_inner().id).await.map_err(to_status)?;
        json_ok(&result)
    }

    async fn list_scheduled_reports(&self, req: Request<OptionalIdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = req.into_inner().id;
        let reports = ua::UnifiedAnalyticsService::new(&self.pool).list_scheduled_reports(id.as_deref()).await.map_err(to_status)?;
        json_ok(&reports)
    }

    async fn create_scheduled_report(&self, req: Request<CreateScheduledRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let report = ua::UnifiedAnalyticsService::new(&self.pool)
            .create_scheduled_report(&r.report_id, &r.schedule, &r.recipients, &r.format)
            .await
            .map_err(to_status)?;
        json_ok(&report)
    }

    async fn update_scheduled_report(&self, req: Request<UpdateStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let report = ua::UnifiedAnalyticsService::new(&self.pool).update_scheduled_report(&r.id, &r.status).await.map_err(to_status)?;
        json_ok(&report)
    }

    async fn delete_scheduled_report(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        ua::UnifiedAnalyticsService::new(&self.pool).delete_scheduled_report(&req.into_inner().id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn list_templates(&self, req: Request<OrgIdRequest>) -> Result<Response<JsonResponse>, Status> {
        let templates = ua::UnifiedAnalyticsService::new(&self.pool).list_templates(&req.into_inner().org_id).await.map_err(to_status)?;
        json_ok(&templates)
    }

    async fn create_template(&self, req: Request<CreateTemplateRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let template = ua::UnifiedAnalyticsService::new(&self.pool)
            .create_template(&r.org_id, &r.name, &r.report_type, &r.description, &parse_json(&r.config_json))
            .await
            .map_err(to_status)?;
        json_ok(&template)
    }

    async fn list_widgets(&self, req: Request<WidgetQueryRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let widgets = ua::UnifiedAnalyticsService::new(&self.pool).list_widgets(&r.org_id, &r.dashboard_id).await.map_err(to_status)?;
        json_ok(&widgets)
    }

    async fn create_widget(&self, req: Request<CreateWidgetRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let widget = ua::UnifiedAnalyticsService::new(&self.pool)
            .create_widget(
                &r.org_id,
                &r.dashboard_id,
                &r.widget_type,
                &r.title,
                &parse_json(&r.config_json),
                r.position_x,
                r.position_y,
                r.width,
                r.height,
            )
            .await
            .map_err(to_status)?;
        json_ok(&widget)
    }

    async fn update_widget(&self, req: Request<UpdateByIdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let widget = ua::UnifiedAnalyticsService::new(&self.pool)
            .update_widget(&r.id, &parse_json(&r.updates_json))
            .await
            .map_err(to_status)?;
        json_ok(&widget)
    }

    async fn delete_widget(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        ua::UnifiedAnalyticsService::new(&self.pool).delete_widget(&req.into_inner().id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn create_snapshot(&self, req: Request<CreateSnapshotRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let snapshot = ua::UnifiedAnalyticsService::new(&self.pool)
            .create_snapshot(&r.org_id, &r.snapshot_type, &r.period_start, &r.period_end)
            .await
            .map_err(to_status)?;
        json_ok(&snapshot)
    }

    async fn list_snapshots(&self, req: Request<ListSnapshotsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let snapshots = ua::UnifiedAnalyticsService::new(&self.pool)
            .list_snapshots(&r.org_id, r.snapshot_type.as_deref(), r.limit)
            .await
            .map_err(to_status)?;
        json_ok(&snapshots)
    }

    async fn get_settings(&self, req: Request<OrgIdRequest>) -> Result<Response<JsonResponse>, Status> {
        let settings = ua::UnifiedAnalyticsService::new(&self.pool).get_settings(&req.into_inner().org_id).await.map_err(to_status)?;
        json_ok(&settings)
    }

    async fn update_settings(&self, req: Request<UpdateByOrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let settings = ua::UnifiedAnalyticsService::new(&self.pool)
            .update_settings(&r.org_id, &parse_json(&r.updates_json))
            .await
            .map_err(to_status)?;
        json_ok(&settings)
    }

    async fn export_report(&self, req: Request<ExportRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = ua::UnifiedAnalyticsService::new(&self.pool).export_report(&r.report_id, &r.format).await.map_err(to_status)?;
        json_ok(&result)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_unified_analytics=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("UNIFIED_ANALYTICS_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50078);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-unified-analytics starting (gRPC)");
    Server::builder()
        .add_service(UnifiedAnalyticsServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
