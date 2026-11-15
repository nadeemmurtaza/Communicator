//! Phase 41: Compliance & Data Privacy Dashboard Routes
//!
//! Thin gRPC client forwarding to svc-compliance-dashboard; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use crate::db::AppState;

use newax_proto::compliance_dashboard::v1::{
    compliance_dashboard_service_client::ComplianceDashboardServiceClient, CheckComplianceRequest as GrpcCheckComplianceRequest,
    CheckSuppressedRequest as GrpcCheckSuppressedRequest, CreateGdprRequestMsg, CreatePolicyRequest as GrpcCreatePolicyRequest,
    CreateSuppressionRequest as GrpcCreateSuppressionRequest, Empty, IdRequest, ListAuditLogsRequest, ListGdprRequestsRequest,
    ListPoliciesRequest, ListSuppressionsRequest, UpdateGdprRequestMsg, UpdatePolicyRequest as GrpcUpdatePolicyRequest,
};

async fn client() -> Result<ComplianceDashboardServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("COMPLIANCE_DASHBOARD_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50076".to_string());
    ComplianceDashboardServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("compliance-dashboard service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

fn not_found_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, e.message().to_string())
}

fn bad_request_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.message().to_string())
}

// ——— Wire types (mirror old service types) ———

#[derive(Debug, Serialize, Clone, serde::Deserialize)]
pub struct SuppressionEntry {
    pub id: String,
    pub contact_id: String,
    pub channel: String,
    pub reason: String,
    pub added_at: String,
    pub added_by: String,
    pub note: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct GDPRRequest {
    pub id: String,
    pub contact_id: Option<String>,
    pub email: Option<String>,
    pub request_type: String,
    pub status: String,
    pub requested_at: String,
    pub completed_at: Option<String>,
    pub handled_by: Option<String>,
    pub notes: Option<String>,
    pub data_export_url: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CompliancePolicy {
    pub id: String,
    pub org_id: String,
    pub policy_name: String,
    pub regulation: String,
    pub channel: String,
    pub description: Option<String>,
    pub rules: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuditLogEntry {
    pub id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: String,
    pub performed_by: String,
    pub performed_at: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Clone, serde::Deserialize)]
pub struct ComplianceSettings {
    pub id: String,
    pub org_id: String,
    pub gdpr_response_days: i32,
    pub auto_expire_gdpr_requests: bool,
    pub require_consent_before_send: bool,
    pub check_suppression_before_send: bool,
    pub log_all_compliance_actions: bool,
    pub can_spam_unsubscribe_link: bool,
    pub can_spam_physical_address: Option<String>,
    pub tcpa_consent_required: bool,
    pub whatsapp_opt_in_required: bool,
    pub data_retention_days: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComplianceSummary {
    pub total_suppressed: i64,
    pub active_suppressed: i64,
    pub pending_gdpr_requests: i64,
    pub total_gdpr_requests: i64,
    pub active_policies: i64,
    pub total_audit_entries: i64,
    pub critical_violations: i64,
    pub suppression_by_reason: Vec<ReasonCount>,
    pub gdpr_by_type: Vec<GDPRTypeCount>,
    pub recent_audit_logs: Vec<AuditLogEntry>,
    pub upcoming_gdpr_expiries: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReasonCount {
    pub reason: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct GDPRTypeCount {
    pub request_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SuppressionCheckResult {
    pub is_suppressed: bool,
    pub entry: Option<SuppressionEntry>,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComplianceCheckResult {
    pub is_compliant: bool,
    pub checks: Vec<ComplianceCheck>,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComplianceCheck {
    pub check_name: String,
    pub regulation: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CreateSuppressionRequest {
    pub contact_id: String,
    pub channel: String,
    pub reason: String,
    pub note: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CreateGDPRRequest {
    pub contact_id: Option<String>,
    pub email: Option<String>,
    pub request_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct UpdateGDPRRequest {
    pub status: Option<String>,
    pub handled_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CreatePolicyRequest {
    pub policy_name: String,
    pub regulation: String,
    pub channel: String,
    pub description: Option<String>,
    pub rules: Option<serde_json::Value>,
}

// ——— proto <-> wire conversions ———

fn suppression_from_grpc(s: newax_proto::compliance_dashboard::v1::SuppressionEntry) -> SuppressionEntry {
    SuppressionEntry { id: s.id, contact_id: s.contact_id, channel: s.channel, reason: s.reason, added_at: s.added_at, added_by: s.added_by, note: s.note, is_active: s.is_active }
}

fn gdpr_from_grpc(g: newax_proto::compliance_dashboard::v1::GdprRequest) -> GDPRRequest {
    GDPRRequest {
        id: g.id,
        contact_id: g.contact_id,
        email: g.email,
        request_type: g.request_type,
        status: g.status,
        requested_at: g.requested_at,
        completed_at: g.completed_at,
        handled_by: g.handled_by,
        notes: g.notes,
        data_export_url: g.data_export_url,
        expires_at: g.expires_at,
    }
}

fn policy_from_grpc(p: newax_proto::compliance_dashboard::v1::CompliancePolicy) -> CompliancePolicy {
    CompliancePolicy {
        id: p.id,
        org_id: p.org_id,
        policy_name: p.policy_name,
        regulation: p.regulation,
        channel: p.channel,
        description: p.description,
        rules: p.rules_json,
        is_active: p.is_active,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

fn audit_from_grpc(a: newax_proto::compliance_dashboard::v1::AuditLogEntry) -> AuditLogEntry {
    AuditLogEntry { id: a.id, action: a.action, entity_type: a.entity_type, entity_id: a.entity_id, details: a.details, performed_by: a.performed_by, performed_at: a.performed_at, severity: a.severity }
}

fn settings_from_grpc(s: newax_proto::compliance_dashboard::v1::ComplianceSettings) -> ComplianceSettings {
    ComplianceSettings {
        id: s.id,
        org_id: s.org_id,
        gdpr_response_days: s.gdpr_response_days,
        auto_expire_gdpr_requests: s.auto_expire_gdpr_requests,
        require_consent_before_send: s.require_consent_before_send,
        check_suppression_before_send: s.check_suppression_before_send,
        log_all_compliance_actions: s.log_all_compliance_actions,
        can_spam_unsubscribe_link: s.can_spam_unsubscribe_link,
        can_spam_physical_address: s.can_spam_physical_address,
        tcpa_consent_required: s.tcpa_consent_required,
        whatsapp_opt_in_required: s.whatsapp_opt_in_required,
        data_retention_days: s.data_retention_days,
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

fn settings_to_grpc(s: ComplianceSettings) -> newax_proto::compliance_dashboard::v1::ComplianceSettings {
    newax_proto::compliance_dashboard::v1::ComplianceSettings {
        id: s.id,
        org_id: s.org_id,
        gdpr_response_days: s.gdpr_response_days,
        auto_expire_gdpr_requests: s.auto_expire_gdpr_requests,
        require_consent_before_send: s.require_consent_before_send,
        check_suppression_before_send: s.check_suppression_before_send,
        log_all_compliance_actions: s.log_all_compliance_actions,
        can_spam_unsubscribe_link: s.can_spam_unsubscribe_link,
        can_spam_physical_address: s.can_spam_physical_address,
        tcpa_consent_required: s.tcpa_consent_required,
        whatsapp_opt_in_required: s.whatsapp_opt_in_required,
        data_retention_days: s.data_retention_days,
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(State(_state): State<AppState>) -> Result<Json<ComplianceSummary>, (StatusCode, String)> {
    let mut c = client().await?;
    let s = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(ComplianceSummary {
        total_suppressed: s.total_suppressed,
        active_suppressed: s.active_suppressed,
        pending_gdpr_requests: s.pending_gdpr_requests,
        total_gdpr_requests: s.total_gdpr_requests,
        active_policies: s.active_policies,
        total_audit_entries: s.total_audit_entries,
        critical_violations: s.critical_violations,
        suppression_by_reason: s.suppression_by_reason.into_iter().map(|r| ReasonCount { reason: r.reason, count: r.count }).collect(),
        gdpr_by_type: s.gdpr_by_type.into_iter().map(|t| GDPRTypeCount { request_type: t.request_type, count: t.count }).collect(),
        recent_audit_logs: s.recent_audit_logs.into_iter().map(audit_from_grpc).collect(),
        upcoming_gdpr_expiries: s.upcoming_gdpr_expiries,
    }))
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
    State(_state): State<AppState>,
    Query(params): Query<SuppressionListQuery>,
) -> Result<Json<Vec<SuppressionEntry>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_suppressions(ListSuppressionsRequest {
            active_only: params.active_only.unwrap_or(true),
            channel: params.channel,
            reason: params.reason,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(resp.suppressions.into_iter().map(suppression_from_grpc).collect()))
}

pub async fn add_suppression(
    State(_state): State<AppState>,
    Json(req): Json<CreateSuppressionRequest>,
) -> Result<Json<SuppressionEntry>, (StatusCode, String)> {
    let mut c = client().await?;
    let entry = c
        .add_suppression(GrpcCreateSuppressionRequest { contact_id: req.contact_id, channel: req.channel, reason: req.reason, note: req.note })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    Ok(Json(suppression_from_grpc(entry)))
}

pub async fn get_suppression(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuppressionEntry>, (StatusCode, String)> {
    let mut c = client().await?;
    let entry = c.get_suppression(IdRequest { id }).await.map_err(not_found_err)?.into_inner();
    Ok(Json(suppression_from_grpc(entry)))
}

pub async fn remove_suppression(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client().await?;
    c.remove_suppression(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct SuppressionCheckQuery {
    contact_id: String,
    channel: String,
}

pub async fn check_suppressed(
    State(_state): State<AppState>,
    Query(params): Query<SuppressionCheckQuery>,
) -> Result<Json<SuppressionCheckResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .check_suppressed(GrpcCheckSuppressedRequest { contact_id: params.contact_id, channel: params.channel })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(SuppressionCheckResult {
        is_suppressed: result.is_suppressed,
        entry: result.entry.map(suppression_from_grpc),
        message: result.message,
    }))
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
    State(_state): State<AppState>,
    Query(params): Query<GDPRListQuery>,
) -> Result<Json<Vec<GDPRRequest>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_gdpr_requests(ListGdprRequestsRequest { status: params.status, request_type: params.request_type })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(resp.requests.into_iter().map(gdpr_from_grpc).collect()))
}

pub async fn create_gdpr_request(
    State(_state): State<AppState>,
    Json(req): Json<CreateGDPRRequest>,
) -> Result<Json<GDPRRequest>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .create_gdpr_request(CreateGdprRequestMsg { contact_id: req.contact_id, email: req.email, request_type: req.request_type, notes: req.notes })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    Ok(Json(gdpr_from_grpc(result)))
}

pub async fn get_gdpr_request(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GDPRRequest>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c.get_gdpr_request(IdRequest { id }).await.map_err(not_found_err)?.into_inner();
    Ok(Json(gdpr_from_grpc(result)))
}

pub async fn update_gdpr_request(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateGDPRRequest>,
) -> Result<Json<GDPRRequest>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .update_gdpr_request(UpdateGdprRequestMsg { id, status: req.status, handled_by: req.handled_by, notes: req.notes })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(gdpr_from_grpc(result)))
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
    State(_state): State<AppState>,
    Query(params): Query<PolicyListQuery>,
) -> Result<Json<Vec<CompliancePolicy>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_policies(ListPoliciesRequest { active_only: params.active_only.unwrap_or(true), regulation: params.regulation })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(resp.policies.into_iter().map(policy_from_grpc).collect()))
}

pub async fn create_policy(
    State(_state): State<AppState>,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<Json<CompliancePolicy>, (StatusCode, String)> {
    let mut c = client().await?;
    let policy = c
        .create_policy(GrpcCreatePolicyRequest {
            policy_name: req.policy_name,
            regulation: req.regulation,
            channel: req.channel,
            description: req.description,
            rules_json: req.rules.map(|v| v.to_string()),
        })
        .await
        .map_err(bad_request_err)?
        .into_inner();
    Ok(Json(policy_from_grpc(policy)))
}

pub async fn get_policy(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CompliancePolicy>, (StatusCode, String)> {
    let mut c = client().await?;
    let policy = c.get_policy(IdRequest { id }).await.map_err(not_found_err)?.into_inner();
    Ok(Json(policy_from_grpc(policy)))
}

#[derive(serde::Deserialize)]
pub struct UpdatePolicyQuery {
    is_active: Option<bool>,
}

pub async fn update_policy(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<UpdatePolicyQuery>,
) -> Result<Json<CompliancePolicy>, (StatusCode, String)> {
    let mut c = client().await?;
    let policy = c
        .update_policy(GrpcUpdatePolicyRequest { id, is_active: params.is_active })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(policy_from_grpc(policy)))
}

pub async fn delete_policy(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client().await?;
    c.delete_policy(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::OK)
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
    State(_state): State<AppState>,
    Query(params): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_audit_logs(ListAuditLogsRequest { action: params.action, severity: params.severity, limit: params.limit.unwrap_or(100) })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(resp.entries.into_iter().map(audit_from_grpc).collect()))
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
    State(_state): State<AppState>,
    Query(params): Query<ComplianceCheckQuery>,
) -> Result<Json<ComplianceCheckResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .check_compliance(GrpcCheckComplianceRequest { contact_id: params.contact_id, channel: params.channel, body: params.body })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(ComplianceCheckResult {
        is_compliant: result.is_compliant,
        checks: result
            .checks
            .into_iter()
            .map(|c| ComplianceCheck { check_name: c.check_name, regulation: c.regulation, passed: c.passed, message: c.message })
            .collect(),
        message: result.message,
    }))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn get_settings(
    State(_state): State<AppState>,
) -> Result<Json<ComplianceSettings>, (StatusCode, String)> {
    let mut c = client().await?;
    let settings = c.get_settings(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(settings_from_grpc(settings)))
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Json(settings): Json<ComplianceSettings>,
) -> Result<Json<ComplianceSettings>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c.update_settings(settings_to_grpc(settings)).await.map_err(grpc_err)?.into_inner();
    Ok(Json(settings_from_grpc(result)))
}
