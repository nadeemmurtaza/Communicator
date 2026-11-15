use newax_compliance_dashboard as cd;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.compliance_dashboard.v1");
}

use proto::{
    compliance_dashboard_service_server::{ComplianceDashboardService, ComplianceDashboardServiceServer},
    AuditLogEntry as ProtoAuditEntry, AuditLogsResponse, CheckComplianceRequest, CheckSuppressedRequest,
    CompliancePolicy as ProtoPolicy, ComplianceCheck as ProtoCheck, ComplianceCheckResult as ProtoCheckResult,
    ComplianceSettings as ProtoSettings, ComplianceSummary as ProtoSummary, CreateGdprRequestMsg, CreatePolicyRequest,
    CreateSuppressionRequest as GrpcCreateSuppressionRequest, Empty, GdprRequest as ProtoGdprRequest, GdprRequestsResponse,
    GdprTypeCount as ProtoGdprTypeCount, IdRequest, ListAuditLogsRequest, ListGdprRequestsRequest, ListPoliciesRequest,
    ListSuppressionsRequest, PoliciesResponse, ReasonCount as ProtoReasonCount, SuppressionCheckResult as ProtoSuppressionCheck,
    SuppressionEntry as ProtoSuppression, SuppressionsResponse, UpdateGdprRequestMsg, UpdatePolicyRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn suppression_to_proto(s: cd::SuppressionEntry) -> ProtoSuppression {
    ProtoSuppression {
        id: s.id,
        contact_id: s.contact_id,
        channel: s.channel,
        reason: s.reason,
        added_at: s.added_at,
        added_by: s.added_by,
        note: s.note,
        is_active: s.is_active,
    }
}

fn gdpr_to_proto(g: cd::GDPRRequest) -> ProtoGdprRequest {
    ProtoGdprRequest {
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

fn policy_to_proto(p: cd::CompliancePolicy) -> ProtoPolicy {
    ProtoPolicy {
        id: p.id,
        org_id: p.org_id,
        policy_name: p.policy_name,
        regulation: p.regulation,
        channel: p.channel,
        description: p.description,
        rules_json: p.rules,
        is_active: p.is_active,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

fn audit_to_proto(a: cd::AuditLogEntry) -> ProtoAuditEntry {
    ProtoAuditEntry {
        id: a.id,
        action: a.action,
        entity_type: a.entity_type,
        entity_id: a.entity_id,
        details: a.details,
        performed_by: a.performed_by,
        performed_at: a.performed_at,
        severity: a.severity,
    }
}

fn settings_to_proto(s: cd::ComplianceSettings) -> ProtoSettings {
    ProtoSettings {
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

fn settings_from_proto(s: ProtoSettings) -> cd::ComplianceSettings {
    cd::ComplianceSettings {
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

#[tonic::async_trait]
impl ComplianceDashboardService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<ProtoSummary>, Status> {
        let s = cd::ComplianceDashboardService::new(self.pool.clone()).get_summary().await.map_err(to_status)?;
        Ok(Response::new(ProtoSummary {
            total_suppressed: s.total_suppressed,
            active_suppressed: s.active_suppressed,
            pending_gdpr_requests: s.pending_gdpr_requests,
            total_gdpr_requests: s.total_gdpr_requests,
            active_policies: s.active_policies,
            total_audit_entries: s.total_audit_entries,
            critical_violations: s.critical_violations,
            suppression_by_reason: s
                .suppression_by_reason
                .into_iter()
                .map(|r| ProtoReasonCount { reason: r.reason, count: r.count })
                .collect(),
            gdpr_by_type: s
                .gdpr_by_type
                .into_iter()
                .map(|t| ProtoGdprTypeCount { request_type: t.request_type, count: t.count })
                .collect(),
            recent_audit_logs: s.recent_audit_logs.into_iter().map(audit_to_proto).collect(),
            upcoming_gdpr_expiries: s.upcoming_gdpr_expiries,
        }))
    }

    async fn list_suppressions(&self, req: Request<ListSuppressionsRequest>) -> Result<Response<SuppressionsResponse>, Status> {
        let r = req.into_inner();
        let suppressions = cd::ComplianceDashboardService::new(self.pool.clone())
            .list_suppressions(r.active_only, r.channel.as_deref(), r.reason.as_deref())
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(SuppressionsResponse { suppressions: suppressions.into_iter().map(suppression_to_proto).collect() }))
    }

    async fn add_suppression(&self, req: Request<GrpcCreateSuppressionRequest>) -> Result<Response<ProtoSuppression>, Status> {
        let r = req.into_inner();
        let logic_req = cd::CreateSuppressionRequest { contact_id: r.contact_id, channel: r.channel, reason: r.reason, note: r.note };
        let entry = cd::ComplianceDashboardService::new(self.pool.clone())
            .add_suppression(&logic_req, "api")
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(suppression_to_proto(entry)))
    }

    async fn get_suppression(&self, req: Request<IdRequest>) -> Result<Response<ProtoSuppression>, Status> {
        let id = req.into_inner().id;
        let entry = cd::ComplianceDashboardService::new(self.pool.clone()).get_suppression(&id).await.map_err(Status::not_found)?;
        Ok(Response::new(suppression_to_proto(entry)))
    }

    async fn remove_suppression(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = req.into_inner().id;
        cd::ComplianceDashboardService::new(self.pool.clone()).remove_suppression(&id).await.map_err(Status::internal)?;
        Ok(Response::new(Empty {}))
    }

    async fn check_suppressed(&self, req: Request<CheckSuppressedRequest>) -> Result<Response<ProtoSuppressionCheck>, Status> {
        let r = req.into_inner();
        let result = cd::ComplianceDashboardService::new(self.pool.clone())
            .check_suppressed(&r.contact_id, &r.channel)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(ProtoSuppressionCheck {
            is_suppressed: result.is_suppressed,
            entry: result.entry.map(suppression_to_proto),
            message: result.message,
        }))
    }

    async fn list_gdpr_requests(&self, req: Request<ListGdprRequestsRequest>) -> Result<Response<GdprRequestsResponse>, Status> {
        let r = req.into_inner();
        let requests = cd::ComplianceDashboardService::new(self.pool.clone())
            .list_gdpr_requests(r.status.as_deref(), r.request_type.as_deref())
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(GdprRequestsResponse { requests: requests.into_iter().map(gdpr_to_proto).collect() }))
    }

    async fn create_gdpr_request(&self, req: Request<CreateGdprRequestMsg>) -> Result<Response<ProtoGdprRequest>, Status> {
        let r = req.into_inner();
        let logic_req = cd::CreateGDPRRequest { contact_id: r.contact_id, email: r.email, request_type: r.request_type, notes: r.notes };
        let result = cd::ComplianceDashboardService::new(self.pool.clone())
            .create_gdpr_request(&logic_req, "api")
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(gdpr_to_proto(result)))
    }

    async fn get_gdpr_request(&self, req: Request<IdRequest>) -> Result<Response<ProtoGdprRequest>, Status> {
        let id = req.into_inner().id;
        let result = cd::ComplianceDashboardService::new(self.pool.clone()).get_gdpr_request(&id).await.map_err(Status::not_found)?;
        Ok(Response::new(gdpr_to_proto(result)))
    }

    async fn update_gdpr_request(&self, req: Request<UpdateGdprRequestMsg>) -> Result<Response<ProtoGdprRequest>, Status> {
        let r = req.into_inner();
        let logic_req = cd::UpdateGDPRRequest { status: r.status, handled_by: r.handled_by, notes: r.notes };
        let result = cd::ComplianceDashboardService::new(self.pool.clone())
            .update_gdpr_request(&r.id, &logic_req, "api")
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(gdpr_to_proto(result)))
    }

    async fn list_policies(&self, req: Request<ListPoliciesRequest>) -> Result<Response<PoliciesResponse>, Status> {
        let r = req.into_inner();
        let policies = cd::ComplianceDashboardService::new(self.pool.clone())
            .list_policies(r.active_only, r.regulation.as_deref())
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(PoliciesResponse { policies: policies.into_iter().map(policy_to_proto).collect() }))
    }

    async fn create_policy(&self, req: Request<CreatePolicyRequest>) -> Result<Response<ProtoPolicy>, Status> {
        let r = req.into_inner();
        let logic_req = cd::CreatePolicyRequest {
            policy_name: r.policy_name,
            regulation: r.regulation,
            channel: r.channel,
            description: r.description,
            rules: r.rules_json.and_then(|s| serde_json::from_str(&s).ok()),
        };
        let policy = cd::ComplianceDashboardService::new(self.pool.clone())
            .create_policy(&logic_req)
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(policy_to_proto(policy)))
    }

    async fn get_policy(&self, req: Request<IdRequest>) -> Result<Response<ProtoPolicy>, Status> {
        let id = req.into_inner().id;
        let policy = cd::ComplianceDashboardService::new(self.pool.clone()).get_policy(&id).await.map_err(Status::not_found)?;
        Ok(Response::new(policy_to_proto(policy)))
    }

    async fn update_policy(&self, req: Request<UpdatePolicyRequest>) -> Result<Response<ProtoPolicy>, Status> {
        let r = req.into_inner();
        let policy = cd::ComplianceDashboardService::new(self.pool.clone())
            .update_policy(&r.id, r.is_active, "api")
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(policy_to_proto(policy)))
    }

    async fn delete_policy(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = req.into_inner().id;
        cd::ComplianceDashboardService::new(self.pool.clone()).delete_policy(&id).await.map_err(Status::internal)?;
        Ok(Response::new(Empty {}))
    }

    async fn list_audit_logs(&self, req: Request<ListAuditLogsRequest>) -> Result<Response<AuditLogsResponse>, Status> {
        let r = req.into_inner();
        let entries = cd::ComplianceDashboardService::new(self.pool.clone())
            .list_audit_logs(r.action.as_deref(), r.severity.as_deref(), r.limit)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(AuditLogsResponse { entries: entries.into_iter().map(audit_to_proto).collect() }))
    }

    async fn check_compliance(&self, req: Request<CheckComplianceRequest>) -> Result<Response<ProtoCheckResult>, Status> {
        let r = req.into_inner();
        let result = cd::ComplianceDashboardService::new(self.pool.clone())
            .check_compliance(&r.contact_id, &r.channel, &r.body)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(ProtoCheckResult {
            is_compliant: result.is_compliant,
            checks: result
                .checks
                .into_iter()
                .map(|c| ProtoCheck { check_name: c.check_name, regulation: c.regulation, passed: c.passed, message: c.message })
                .collect(),
            message: result.message,
        }))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<ProtoSettings>, Status> {
        let settings = cd::ComplianceDashboardService::new(self.pool.clone()).get_settings().await.map_err(Status::internal)?;
        Ok(Response::new(settings_to_proto(settings)))
    }

    async fn update_settings(&self, req: Request<ProtoSettings>) -> Result<Response<ProtoSettings>, Status> {
        let settings = settings_from_proto(req.into_inner());
        let updated = cd::ComplianceDashboardService::new(self.pool.clone())
            .update_settings(&settings)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(settings_to_proto(updated)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_compliance_dashboard=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("COMPLIANCE_DASHBOARD_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50076);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-compliance-dashboard starting (gRPC)");
    Server::builder()
        .add_service(ComplianceDashboardServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
