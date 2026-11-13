//! Phase 41: Compliance & Data Privacy Dashboard Routes

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use crate::state::AppState;
use crate::services::compliance_dashboard::*;

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(State(state): State<AppState>) -> Result<Json<ComplianceSummary>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    service.get_summary().await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ---------------------------------------------------------------------------
// Suppression List
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SuppressionListQuery {
    active_only: Option<bool>,
    channel: Option<String>,
    reason: Option<String>,
}

pub async fn list_suppressions(
    State(state): State<AppState>,
    Query(params): Query<SuppressionListQuery>,
) -> Result<Json<Vec<SuppressionEntry>>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.list_suppressions(
        params.active_only.unwrap_or(true),
        params.channel.as_deref(),
        params.reason.as_deref(),
    ).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn add_suppression(
    State(state): State<AppState>,
    Json(req): Json<CreateSuppressionRequest>,
) -> Result<Json<SuppressionEntry>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.add_suppression(&req, "api").await;
    result.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_suppression(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuppressionEntry>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.get_suppression(&id).await;
    result.map(Json).map_err(|e| (StatusCode::NOT_FOUND, e))
}

pub async fn remove_suppression(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    service.remove_suppression(&id).await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(serde::Deserialize)]
pub struct SuppressionCheckQuery {
    contact_id: String,
    channel: String,
}

pub async fn check_suppressed(
    State(state): State<AppState>,
    Query(params): Query<SuppressionCheckQuery>,
) -> Result<Json<SuppressionCheckResult>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.check_suppressed(&params.contact_id, &params.channel).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// GDPR Requests
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct GDPRListQuery {
    status: Option<String>,
    request_type: Option<String>,
}

pub async fn list_gdpr_requests(
    State(state): State<AppState>,
    Query(params): Query<GDPRListQuery>,
) -> Result<Json<Vec<GDPRRequest>>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.list_gdpr_requests(params.status.as_deref(), params.request_type.as_deref()).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn create_gdpr_request(
    State(state): State<AppState>,
    Json(req): Json<CreateGDPRRequest>,
) -> Result<Json<GDPRRequest>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.create_gdpr_request(&req, "api").await;
    result.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_gdpr_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GDPRRequest>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.get_gdpr_request(&id).await;
    result.map(Json).map_err(|e| (StatusCode::NOT_FOUND, e))
}

pub async fn update_gdpr_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateGDPRRequest>,
) -> Result<Json<GDPRRequest>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.update_gdpr_request(&id, &req, "api").await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Compliance Policies
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct PolicyListQuery {
    active_only: Option<bool>,
    regulation: Option<String>,
}

pub async fn list_policies(
    State(state): State<AppState>,
    Query(params): Query<PolicyListQuery>,
) -> Result<Json<Vec<CompliancePolicy>>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.list_policies(params.active_only.unwrap_or(true), params.regulation.as_deref()).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn create_policy(
    State(state): State<AppState>,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<Json<CompliancePolicy>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.create_policy(&req).await;
    result.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CompliancePolicy>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.get_policy(&id).await;
    result.map(Json).map_err(|e| (StatusCode::NOT_FOUND, e))
}

#[derive(serde::Deserialize)]
pub struct UpdatePolicyQuery {
    is_active: Option<bool>,
}

pub async fn update_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<UpdatePolicyQuery>,
) -> Result<Json<CompliancePolicy>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.update_policy(&id, params.is_active, "api").await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn delete_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    service.delete_policy(&id).await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Audit Log
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct AuditLogQuery {
    action: Option<String>,
    severity: Option<String>,
    limit: Option<i64>,
}

pub async fn list_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.list_audit_logs(
        params.action.as_deref(),
        params.severity.as_deref(),
        params.limit.unwrap_or(100),
    ).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Compliance Check
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct ComplianceCheckQuery {
    contact_id: String,
    channel: String,
    body: String,
}

pub async fn check_compliance(
    State(state): State<AppState>,
    Query(params): Query<ComplianceCheckQuery>,
) -> Result<Json<ComplianceCheckResult>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.check_compliance(&params.contact_id, &params.channel, &params.body).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<ComplianceSettings>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.get_settings().await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<ComplianceSettings>,
) -> Result<Json<ComplianceSettings>, (StatusCode, String)> {
    let service = ComplianceDashboardService::new(state.pool.clone());
    let result = service.update_settings(&settings).await;
    result.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
