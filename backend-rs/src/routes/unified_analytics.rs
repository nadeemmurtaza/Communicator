use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::services::unified_analytics::UnifiedAnalyticsService;

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
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.get_summary().await {
        Ok(summary) => Json(serde_json::to_value(summary).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Saved Reports
pub async fn list_saved_reports(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    match service.list_saved_reports(&org_id, params.report_type.as_deref()).await {
        Ok(reports) => Json(serde_json::to_value(reports).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_saved_report(
    State(state): State<AppState>,
    Json(payload): Json<CreateReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.create_saved_report(
        &payload.org_id,
        &payload.name,
        &payload.report_type,
        &payload.config,
        &payload.time_range,
        &payload.filters,
        &payload.created_by,
    ).await {
        Ok(report) => Json(serde_json::to_value(report).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_saved_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.update_saved_report(&id, &updates).await {
        Ok(report) => Json(serde_json::to_value(report).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_saved_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.delete_saved_report(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn run_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.run_report(&id).await {
        Ok(result) => Json(serde_json::to_value(result).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Scheduled Reports
pub async fn list_scheduled_reports(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.list_scheduled_reports(None).await {
        Ok(reports) => Json(serde_json::to_value(reports).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_scheduled_report(
    State(state): State<AppState>,
    Json(payload): Json<CreateScheduledQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.create_scheduled_report(
        &payload.report_id,
        &payload.schedule,
        &payload.recipients,
        &payload.format,
    ).await {
        Ok(report) => Json(serde_json::to_value(report).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_scheduled_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("active");
    match service.update_scheduled_report(&id, status).await {
        Ok(report) => Json(serde_json::to_value(report).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_scheduled_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.delete_scheduled_report(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Report Templates
pub async fn list_templates(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    match service.list_templates(&org_id).await {
        Ok(templates) => Json(serde_json::to_value(templates).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_template(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let org_id = payload.get("org_id").and_then(|v| v.as_str()).unwrap_or("default");
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let report_type = payload.get("report_type").and_then(|v| v.as_str()).unwrap_or("unified");
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let config = payload.get("config").cloned().unwrap_or(serde_json::json!({}));
    match service.create_template(org_id, name, report_type, description, &config).await {
        Ok(template) => Json(serde_json::to_value(template).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Dashboard Widgets
pub async fn list_widgets(
    State(state): State<AppState>,
    Query(params): Query<WidgetQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.list_widgets(&params.org_id, &params.dashboard_id).await {
        Ok(widgets) => Json(serde_json::to_value(widgets).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_widget(
    State(state): State<AppState>,
    Json(payload): Json<CreateWidgetQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.create_widget(
        &payload.org_id,
        &payload.dashboard_id,
        &payload.widget_type,
        &payload.title,
        &payload.config,
        payload.position_x,
        payload.position_y,
        payload.width,
        payload.height,
    ).await {
        Ok(widget) => Json(serde_json::to_value(widget).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_widget(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.update_widget(&id, &updates).await {
        Ok(widget) => Json(serde_json::to_value(widget).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_widget(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.delete_widget(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Analytics Snapshots
pub async fn create_snapshot(
    State(state): State<AppState>,
    Json(payload): Json<SnapshotQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.create_snapshot(
        &payload.org_id,
        &payload.snapshot_type,
        &payload.period_start,
        &payload.period_end,
    ).await {
        Ok(snapshot) => Json(serde_json::to_value(snapshot).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_snapshots(
    State(state): State<AppState>,
    Query(params): Query<ListSnapshotQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let limit = params.limit.unwrap_or(50);
    match service.list_snapshots(&params.org_id, params.snapshot_type.as_deref(), limit).await {
        Ok(snapshots) => Json(serde_json::to_value(snapshots).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Settings
pub async fn get_settings(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    match service.get_settings(&org_id).await {
        Ok(settings) => Json(serde_json::to_value(settings).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    let org_id = params.org_id.unwrap_or_else(|| "default".to_string());
    match service.update_settings(&org_id, &updates).await {
        Ok(settings) => Json(serde_json::to_value(settings).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

// Export
pub async fn export_report(
    State(state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = UnifiedAnalyticsService::new(&state.pool);
    match service.export_report(&params.report_id, &params.format).await {
        Ok(result) => Json(serde_json::to_value(result).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        })?),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}
