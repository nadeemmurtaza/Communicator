//! Thin gRPC client forwarding to svc-unified-analytics; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::AppState;

use newax_proto::unified_analytics::v1::{
    unified_analytics_service_client::UnifiedAnalyticsServiceClient, CreateReportRequest as GrpcCreateReportRequest,
    CreateScheduledRequest as GrpcCreateScheduledRequest, CreateSnapshotRequest as GrpcCreateSnapshotRequest,
    CreateTemplateRequest as GrpcCreateTemplateRequest, CreateWidgetRequest as GrpcCreateWidgetRequest, Empty,
    ExportRequest as GrpcExportRequest, IdRequest, ListReportsRequest, ListSnapshotsRequest, OptionalIdRequest, OrgIdRequest,
    UpdateByIdRequest, UpdateByOrgRequest, UpdateStatusRequest as GrpcUpdateStatusRequest, WidgetQueryRequest,
};

async fn client() -> Result<UnifiedAnalyticsServiceClient<tonic::transport::Channel>, (StatusCode, Json<serde_json::Value>)> {
    let url = std::env::var("UNIFIED_ANALYTICS_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50078".to_string());
    UnifiedAnalyticsServiceClient::connect(url).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": format!("unified-analytics service unavailable: {e}") })))
    })
}

fn grpc_err(e: tonic::Status) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.message().to_string() })))
}

fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_default()
}

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub org_id: Option<String>,
    pub report_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReportQuery {
    pub org_id: String,
    pub name: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub time_range: String,
    pub filters: serde_json::Value,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduledQuery {
    pub report_id: String,
    pub schedule: String,
    pub recipients: Vec<String>,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWidgetQuery {
    pub org_id: String,
    pub dashboard_id: String,
    pub widget_type: String,
    pub title: String,
    pub config: serde_json::Value,
    pub position_x: i32,
    pub position_y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotQuery {
    pub org_id: String,
    pub snapshot_type: String,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSnapshotQuery {
    pub org_id: String,
    pub snapshot_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct WidgetQuery {
    pub org_id: String,
    pub dashboard_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub report_id: String,
    pub format: String,
}

pub async fn get_summary(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(parse_json(&resp.json)))
}

// Saved Reports
pub async fn list_saved_reports(
    State(_state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    let resp = c
        .list_saved_reports(ListReportsRequest { org_id, report_type: params.report_type })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn create_saved_report(
    State(_state): State<AppState>,
    Json(payload): Json<CreateReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .create_saved_report(GrpcCreateReportRequest {
            org_id: payload.org_id,
            name: payload.name,
            report_type: payload.report_type,
            config_json: payload.config.to_string(),
            time_range: payload.time_range,
            filters_json: payload.filters.to_string(),
            created_by: payload.created_by,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn update_saved_report(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .update_saved_report(UpdateByIdRequest { id, updates_json: updates.to_string() })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn delete_saved_report(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    c.delete_saved_report(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn run_report(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c.run_report(IdRequest { id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(parse_json(&resp.json)))
}

// Scheduled Reports
pub async fn list_scheduled_reports(
    State(_state): State<AppState>,
    Query(_params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c.list_scheduled_reports(OptionalIdRequest { id: None }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn create_scheduled_report(
    State(_state): State<AppState>,
    Json(payload): Json<CreateScheduledQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .create_scheduled_report(GrpcCreateScheduledRequest {
            report_id: payload.report_id,
            schedule: payload.schedule,
            recipients: payload.recipients,
            format: payload.format,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn update_scheduled_report(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("active").to_string();
    let resp = c.update_scheduled_report(GrpcUpdateStatusRequest { id, status }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn delete_scheduled_report(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    c.delete_scheduled_report(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// Report Templates
pub async fn list_templates(
    State(_state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    let resp = c.list_templates(OrgIdRequest { org_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn create_template(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let org_id = payload.get("org_id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let report_type = payload.get("report_type").and_then(|v| v.as_str()).unwrap_or("unified").to_string();
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let config = payload.get("config").cloned().unwrap_or(serde_json::json!({}));
    let resp = c
        .create_template(GrpcCreateTemplateRequest { org_id, name, report_type, description, config_json: config.to_string() })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

// Dashboard Widgets
pub async fn list_widgets(
    State(_state): State<AppState>,
    Query(params): Query<WidgetQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .list_widgets(WidgetQueryRequest { org_id: params.org_id, dashboard_id: params.dashboard_id })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn create_widget(
    State(_state): State<AppState>,
    Json(payload): Json<CreateWidgetQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .create_widget(GrpcCreateWidgetRequest {
            org_id: payload.org_id,
            dashboard_id: payload.dashboard_id,
            widget_type: payload.widget_type,
            title: payload.title,
            config_json: payload.config.to_string(),
            position_x: payload.position_x,
            position_y: payload.position_y,
            width: payload.width,
            height: payload.height,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn update_widget(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .update_widget(UpdateByIdRequest { id, updates_json: updates.to_string() })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn delete_widget(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    c.delete_widget(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// Analytics Snapshots
pub async fn create_snapshot(
    State(_state): State<AppState>,
    Json(payload): Json<SnapshotQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .create_snapshot(GrpcCreateSnapshotRequest {
            org_id: payload.org_id,
            snapshot_type: payload.snapshot_type,
            period_start: payload.period_start,
            period_end: payload.period_end,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn list_snapshots(
    State(_state): State<AppState>,
    Query(params): Query<ListSnapshotQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let limit = params.limit.unwrap_or(50);
    let resp = c
        .list_snapshots(ListSnapshotsRequest { org_id: params.org_id, snapshot_type: params.snapshot_type, limit })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

// Settings
pub async fn get_settings(
    State(_state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    let resp = c.get_settings(OrgIdRequest { org_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(parse_json(&resp.json)))
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Query(params): Query<ReportQuery>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    let resp = c
        .update_settings(UpdateByOrgRequest { org_id, updates_json: updates.to_string() })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}

// Export
pub async fn export_report(
    State(_state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .export_report(GrpcExportRequest { report_id: params.report_id, format: params.format })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(parse_json(&resp.json)))
}
