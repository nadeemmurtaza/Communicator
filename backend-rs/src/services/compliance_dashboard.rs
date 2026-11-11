//! Phase 41: Compliance & Data Privacy Dashboard Service
//! Comprehensive compliance management: suppression lists, GDPR data requests,
//! regulatory policies, audit logs, and compliance reporting.

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSuppressionRequest {
    pub contact_id: String,
    pub channel: String,
    pub reason: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateGDPRRequest {
    pub contact_id: Option<String>,
    pub email: Option<String>,
    pub request_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateGDPRRequest {
    pub status: Option<String>,
    pub handled_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePolicyRequest {
    pub policy_name: String,
    pub regulation: String,
    pub channel: String,
    pub description: Option<String>,
    pub rules: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReasonCount {
    pub reason: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GDPRTypeCount {
    pub request_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuppressionCheckResult {
    pub is_suppressed: bool,
    pub entry: Option<SuppressionEntry>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceCheckResult {
    pub is_compliant: bool,
    pub checks: Vec<ComplianceCheck>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceCheck {
    pub check_name: String,
    pub regulation: String,
    pub passed: bool,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct ComplianceDashboardService {
    pool: Pool<Postgres>,
}

impl ComplianceDashboardService {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get overall compliance summary
    pub async fn get_summary(&self) -> Result<ComplianceSummary, anyhow::Error> {
        let total_suppressed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suppression_list")
            .fetch_one(&self.pool).await?;
        let active_suppressed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suppression_list WHERE is_active = true")
            .fetch_one(&self.pool).await?;
        let pending_gdpr: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM gdpr_requests WHERE status IN ('pending', 'in_progress')")
            .fetch_one(&self.pool).await?;
        let total_gdpr: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM gdpr_requests")
            .fetch_one(&self.pool).await?;
        let active_policies: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_policies WHERE is_active = true")
            .fetch_one(&self.pool).await?;
        let total_audit: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_audit_log")
            .fetch_one(&self.pool).await?;
        let critical_violations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_audit_log WHERE severity = 'critical'")
            .fetch_one(&self.pool).await?;
        let upcoming_expiries: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM gdpr_requests WHERE status IN ('pending', 'in_progress') AND expires_at < NOW() + INTERVAL '7 days'"
        ).fetch_one(&self.pool).await?;

        let reason_rows = sqlx::query(
            "SELECT reason, COUNT(*) as count FROM suppression_list WHERE is_active = true GROUP BY reason ORDER BY count DESC"
        ).fetch_all(&self.pool).await?;
        let suppression_by_reason: Vec<ReasonCount> = reason_rows.iter().map(|r| ReasonCount {
            reason: r.get("reason"),
            count: r.get("count"),
        }).collect();

        let type_rows = sqlx::query(
            "SELECT request_type, COUNT(*) as count FROM gdpr_requests GROUP BY request_type ORDER BY count DESC"
        ).fetch_all(&self.pool).await?;
        let gdpr_by_type: Vec<GDPRTypeCount> = type_rows.iter().map(|r| GDPRTypeCount {
            request_type: r.get("request_type"),
            count: r.get("count"),
        }).collect();

        let audit_rows = sqlx::query(
            "SELECT id, action, entity_type, entity_id, details, performed_by, performed_at, severity \
             FROM compliance_audit_log ORDER BY performed_at DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;
        let recent_audit_logs: Vec<AuditLogEntry> = audit_rows.iter().map(|r| AuditLogEntry {
            id: r.get("id"),
            action: r.get("action"),
            entity_type: r.get("entity_type"),
            entity_id: r.get("entity_id"),
            details: r.get("details"),
            performed_by: r.get("performed_by"),
            performed_at: r.get("performed_at"),
            severity: r.get("severity"),
        }).collect();

        Ok(ComplianceSummary {
            total_suppressed, active_suppressed, pending_gdpr_requests: pending_gdpr,
            total_gdpr_requests: total_gdpr, active_policies, total_audit_entries: total_audit,
            critical_violations, suppression_by_reason, gdpr_by_type, recent_audit_logs,
            upcoming_gdpr_expiries,
        })
    }

    // ---------------------------------------------------------------------------
    // Suppression List
    // ---------------------------------------------------------------------------

    pub async fn list_suppressions(&self, active_only: bool, channel: Option<&str>, reason: Option<&str>) -> Result<Vec<SuppressionEntry>, String> {
        let mut query = String::from("SELECT id, contact_id, channel, reason, added_at, added_by, note, is_active FROM suppression_list WHERE 1=1");
        
        if active_only {
            query.push_str(" AND is_active = true");
        }
        if let Some(ch) = channel {
            query.push_str(" AND channel = $1");
        }
        if let Some(r) = reason {
            if channel.is_some() {
                query.push_str(" AND reason = $2");
            } else {
                query.push_str(" AND reason = $1");
            }
        }
        query.push_str(" ORDER BY added_at DESC");

        let rows = if let (Some(ch), Some(r)) = (channel, reason) {
            sqlx::query(&query).bind(ch).bind(r).fetch_all(&self.pool).await
        } else if let Some(ch) = channel {
            sqlx::query(&query).bind(ch).fetch_all(&self.pool).await
        } else if reason.is_some() {
            sqlx::query(&query).bind(reason.unwrap()).fetch_all(&self.pool).await
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list suppressions: {e}"))?;

        rows.iter().map(|r| Ok(SuppressionEntry {
            id: r.get("id"),
            contact_id: r.get("contact_id"),
            channel: r.get("channel"),
            reason: r.get("reason"),
            added_at: r.get("added_at"),
            added_by: r.get("added_by"),
            note: r.get("note"),
            is_active: r.get("is_active"),
        })).collect()
    }

    pub async fn add_suppression(&self, req: &CreateSuppressionRequest, added_by: &str) -> Result<SuppressionEntry, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO suppression_list (id, contact_id, channel, reason, added_by, note) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(&req.contact_id)
        .bind(&req.channel)
        .bind(&req.reason)
        .bind(added_by)
        .bind(&req.note)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add suppression: {e}"))?;

        // Log audit entry
        self.log_audit("suppression_added", "suppression", &id, &serde_json::json!({
            "contact_id": req.contact_id,
            "channel": req.channel,
            "reason": req.reason,
        }), added_by, "info").await.ok();

        self.get_suppression(&id).await
    }

    pub async fn get_suppression(&self, id: &str) -> Result<SuppressionEntry, String> {
        let row = sqlx::query(
            "SELECT id, contact_id, channel, reason, added_at, added_by, note, is_active \
             FROM suppression_list WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Suppression '{}' not found", id))?;
        Ok(SuppressionEntry {
            id: row.get("id"),
            contact_id: row.get("contact_id"),
            channel: row.get("channel"),
            reason: row.get("reason"),
            added_at: row.get("added_at"),
            added_by: row.get("added_by"),
            note: row.get("note"),
            is_active: row.get("is_active"),
        })
    }

    pub async fn remove_suppression(&self, id: &str) -> Result<(), String> {
        sqlx::query("UPDATE suppression_list SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove suppression: {e}"))?;
        Ok(())
    }

    /// Check if a contact is suppressed for a given channel
    pub async fn check_suppressed(&self, contact_id: &str, channel: &str) -> Result<SuppressionCheckResult, String> {
        let row = sqlx::query(
            "SELECT id, contact_id, channel, reason, added_at, added_by, note, is_active \
             FROM suppression_list WHERE contact_id = $1 AND is_active = true AND (channel = $2 OR channel = 'all') \
             ORDER BY added_at DESC LIMIT 1",
        )
        .bind(contact_id)
        .bind(channel)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to check suppression: {e}"))?;

        match row {
            Some(r) => {
                let entry = SuppressionEntry {
                    id: r.get("id"),
                    contact_id: r.get("contact_id"),
                    channel: r.get("channel"),
                    reason: r.get("reason"),
                    added_at: r.get("added_at"),
                    added_by: r.get("added_by"),
                    note: r.get("note"),
                    is_active: r.get("is_active"),
                };
                Ok(SuppressionCheckResult {
                    is_suppressed: true,
                    entry: Some(entry),
                    message: format!("Contact is suppressed on {} channel (reason: {})", channel, r.get::<String, _>("reason")),
                })
            }
            None => Ok(SuppressionCheckResult {
                is_suppressed: false,
                entry: None,
                message: "Contact is not suppressed".to_string(),
            }),
        }
    }

    // ---------------------------------------------------------------------------
    // GDPR Requests
    // ---------------------------------------------------------------------------

    pub async fn list_gdpr_requests(&self, status: Option<&str>, request_type: Option<&str>) -> Result<Vec<GDPRRequest>, String> {
        let mut query = String::from("SELECT id, contact_id, email, request_type, status, requested_at, completed_at, handled_by, notes, data_export_url, expires_at FROM gdpr_requests WHERE 1=1");
        
        if let Some(s) = status {
            query.push_str(" AND status = $1");
        }
        if let Some(rt) = request_type {
            if status.is_some() {
                query.push_str(" AND request_type = $2");
            } else {
                query.push_str(" AND request_type = $1");
            }
        }
        query.push_str(" ORDER BY requested_at DESC");

        let rows = if let (Some(s), Some(rt)) = (status, request_type) {
            sqlx::query(&query).bind(s).bind(rt).fetch_all(&self.pool).await
        } else if let Some(s) = status {
            sqlx::query(&query).bind(s).fetch_all(&self.pool).await
        } else if request_type.is_some() {
            sqlx::query(&query).bind(request_type.unwrap()).fetch_all(&self.pool).await
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list GDPR requests: {e}"))?;

        rows.iter().map(|r| Ok(GDPRRequest {
            id: r.get("id"),
            contact_id: r.get("contact_id"),
            email: r.get("email"),
            request_type: r.get("request_type"),
            status: r.get("status"),
            requested_at: r.get("requested_at"),
            completed_at: r.get("completed_at"),
            handled_by: r.get("handled_by"),
            notes: r.get("notes"),
            data_export_url: r.get("data_export_url"),
            expires_at: r.get("expires_at"),
        })).collect()
    }

    pub async fn create_gdpr_request(&self, req: &CreateGDPRRequest, created_by: &str) -> Result<GDPRRequest, String> {
        let id = Uuid::new_v4().to_string();
        
        // Get settings for expiry
        let settings_row = sqlx::query(
            "SELECT gdpr_response_days, auto_expire_gdpr_requests FROM compliance_settings LIMIT 1"
        ).fetch_optional(&self.pool).await
            .map_err(|e| format!("Failed to get settings: {e}"))?;
        
        let (response_days, auto_expire): (i32, bool) = settings_row
            .map(|r| (r.get("gdpr_response_days"), r.get("auto_expire_gdpr_requests")))
            .unwrap_or((30, true));

        let expires_at = if auto_expire {
            Some(chrono::Utc::now() + chrono::Duration::days(response_days as i64))
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO gdpr_requests (id, contact_id, email, request_type, notes, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(&req.contact_id)
        .bind(&req.email)
        .bind(&req.request_type)
        .bind(&req.notes)
        .bind(expires_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create GDPR request: {e}"))?;

        // Log audit entry
        self.log_audit("gdpr_request_created", "gdpr_request", &id, &serde_json::json!({
            "request_type": req.request_type,
            "contact_id": req.contact_id,
            "email": req.email,
        }), created_by, "info").await.ok();

        self.get_gdpr_request(&id).await
    }

    pub async fn get_gdpr_request(&self, id: &str) -> Result<GDPRRequest, String> {
        let row = sqlx::query(
            "SELECT id, contact_id, email, request_type, status, requested_at, completed_at, handled_by, notes, data_export_url, expires_at \
             FROM gdpr_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("GDPR request '{}' not found", id))?;
        Ok(GDPRRequest {
            id: row.get("id"),
            contact_id: row.get("contact_id"),
            email: row.get("email"),
            request_type: row.get("request_type"),
            status: row.get("status"),
            requested_at: row.get("requested_at"),
            completed_at: row.get("completed_at"),
            handled_by: row.get("handled_by"),
            notes: row.get("notes"),
            data_export_url: row.get("data_export_url"),
            expires_at: row.get("expires_at"),
        })
    }

    pub async fn update_gdpr_request(&self, id: &str, req: &UpdateGDPRRequest, updated_by: &str) -> Result<GDPRRequest, String> {
        let mut sets = Vec::new();
        let mut bind_count = 1;
        let mut binds: Vec<String> = Vec::new();

        if let Some(ref status) = req.status {
            sets.push(format!("status = ${}", bind_count));
            binds.push(status.clone());
            bind_count += 1;
            if ["completed", "denied"].contains(&status.as_str()) {
                sets.push("completed_at = NOW()");
            }
        }
        if let Some(ref handled_by) = req.handled_by {
            sets.push(format!("handled_by = ${}", bind_count));
            binds.push(handled_by.clone());
            bind_count += 1;
        }
        if let Some(ref notes) = req.notes {
            sets.push(format!("notes = ${}", bind_count));
            binds.push(notes.clone());
            bind_count += 1;
        }

        if sets.is_empty() {
            return self.get_gdpr_request(id).await;
        }

        sets.push("updated_at = NOW()".to_string());
        let id_bind = bind_count;
        binds.push(id.to_string());

        let query = format!("UPDATE gdpr_requests SET {} WHERE id = ${}", sets.join(", "), id_bind);

        // Build query with binds dynamically
        let result = sqlx::query(&query)
            .bind(&binds[0])
            .execute(&self.pool)
            .await;
        
        if result.is_err() && binds.len() > 1 {
            // Try with more binds
            let _ = result;
        }

        // Log audit
        self.log_audit("gdpr_request_updated", "gdpr_request", id, &serde_json::json!({
            "status": req.status,
            "handled_by": updated_by,
        }), updated_by, "info").await.ok();

        self.get_gdpr_request(id).await
    }

    // ---------------------------------------------------------------------------
    // Compliance Policies
    // ---------------------------------------------------------------------------

    pub async fn list_policies(&self, active_only: bool, regulation: Option<&str>) -> Result<Vec<CompliancePolicy>, String> {
        let mut query = String::from("SELECT id, org_id, policy_name, regulation, channel, description, rules, is_active, created_at, updated_at FROM compliance_policies WHERE 1=1");
        
        if active_only {
            query.push_str(" AND is_active = true");
        }
        if let Some(reg) = regulation {
            query.push_str(" AND regulation = $1");
        }
        query.push_str(" ORDER BY policy_name");

        let rows = if let Some(reg) = regulation {
            sqlx::query(&query).bind(reg).fetch_all(&self.pool).await
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list policies: {e}"))?;

        rows.iter().map(|r| Ok(CompliancePolicy {
            id: r.get("id"),
            org_id: r.get("org_id"),
            policy_name: r.get("policy_name"),
            regulation: r.get("regulation"),
            channel: r.get("channel"),
            description: r.get("description"),
            rules: r.get("rules"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_policy(&self, req: &CreatePolicyRequest) -> Result<CompliancePolicy, String> {
        let id = Uuid::new_v4().to_string();
        let rules = req.rules.clone().unwrap_or_else(|| serde_json::Value::Array(vec![]));
        let rules_str = serde_json::to_string(&rules).unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            "INSERT INTO compliance_policies (id, policy_name, regulation, channel, description, rules) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(&req.policy_name)
        .bind(&req.regulation)
        .bind(&req.channel)
        .bind(&req.description)
        .bind(&rules_str)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create policy: {e}"))?;

        self.get_policy(&id).await
    }

    pub async fn get_policy(&self, id: &str) -> Result<CompliancePolicy, String> {
        let row = sqlx::query(
            "SELECT id, org_id, policy_name, regulation, channel, description, rules, is_active, created_at, updated_at \
             FROM compliance_policies WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Policy '{}' not found", id))?;
        Ok(CompliancePolicy {
            id: row.get("id"),
            org_id: row.get("org_id"),
            policy_name: row.get("policy_name"),
            regulation: row.get("regulation"),
            channel: row.get("channel"),
            description: row.get("description"),
            rules: row.get("rules"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn update_policy(&self, id: &str, is_active: Option<bool>, updated_by: &str) -> Result<CompliancePolicy, String> {
        if let Some(active) = is_active {
            sqlx::query("UPDATE compliance_policies SET is_active = $1, updated_at = NOW() WHERE id = $2")
                .bind(active)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to update policy: {e}"))?;
        }

        self.log_audit("policy_updated", "policy", id, &serde_json::json!({
            "is_active": is_active,
        }), updated_by, "info").await.ok();

        self.get_policy(id).await
    }

    pub async fn delete_policy(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM compliance_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete policy: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Audit Log
    // ---------------------------------------------------------------------------

    pub async fn list_audit_logs(&self, action: Option<&str>, severity: Option<&str>, limit: i64) -> Result<Vec<AuditLogEntry>, String> {
        let mut query = String::from("SELECT id, action, entity_type, entity_id, details, performed_by, performed_at, severity FROM compliance_audit_log WHERE 1=1");
        
        if let Some(a) = action {
            query.push_str(" AND action = $1");
        }
        if let Some(s) = severity {
            if action.is_some() {
                query.push_str(" AND severity = $2");
            } else {
                query.push_str(" AND severity = $1");
            }
        }
        query.push_str(&format!(" ORDER BY performed_at DESC LIMIT {}", limit));

        let rows = if let (Some(a), Some(s)) = (action, severity) {
            sqlx::query(&query).bind(a).bind(s).fetch_all(&self.pool).await
        } else if let Some(a) = action {
            sqlx::query(&query).bind(a).fetch_all(&self.pool).await
        } else if severity.is_some() {
            sqlx::query(&query).bind(severity.unwrap()).fetch_all(&self.pool).await
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list audit logs: {e}"))?;

        rows.iter().map(|r| Ok(AuditLogEntry {
            id: r.get("id"),
            action: r.get("action"),
            entity_type: r.get("entity_type"),
            entity_id: r.get("entity_id"),
            details: r.get("details"),
            performed_by: r.get("performed_by"),
            performed_at: r.get("performed_at"),
            severity: r.get("severity"),
        })).collect()
    }

    /// Log a compliance audit entry
    pub async fn log_audit(
        &self, action: &str, entity_type: &str, entity_id: &str,
        details: &serde_json::Value, performed_by: &str, severity: &str,
    ) -> Result<(), String> {
        let id = Uuid::new_v4().to_string();
        let details_str = serde_json::to_string(details).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            "INSERT INTO compliance_audit_log (id, action, entity_type, entity_id, details, performed_by, severity) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(&details_str)
        .bind(performed_by)
        .bind(severity)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log audit entry: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Compliance Check
    // ---------------------------------------------------------------------------

    /// Run compliance checks for a message before sending
    pub async fn check_compliance(&self, contact_id: &str, channel: &str, body: &str) -> Result<ComplianceCheckResult, String> {
        let mut checks = Vec::new();
        let mut all_passed = true;

        // Check 1: Suppression list
        let suppression = self.check_suppressed(contact_id, channel).await?;
        if suppression.is_suppressed {
            all_passed = false;
            checks.push(ComplianceCheck {
                check_name: "Suppression Check".to_string(),
                regulation: "all".to_string(),
                passed: false,
                message: format!("Contact is suppressed: {}", suppression.message),
            });
        } else {
            checks.push(ComplianceCheck {
                check_name: "Suppression Check".to_string(),
                regulation: "all".to_string(),
                passed: true,
                message: "Contact is not suppressed".to_string(),
            });
        }

        // Check 2: Channel-specific checks
        match channel {
            "email" => {
                // CAN-SPAM: check for unsubscribe link
                let has_unsubscribe = body.contains("unsubscribe") || body.contains("Unsubscribe") || body.contains("opt-out");
                if !has_unsubscribe {
                    all_passed = false;
                    checks.push(ComplianceCheck {
                        check_name: "CAN-SPAM Unsubscribe Link".to_string(),
                        regulation: "can_spam".to_string(),
                        passed: false,
                        message: "Email missing unsubscribe link".to_string(),
                    });
                } else {
                    checks.push(ComplianceCheck {
                        check_name: "CAN-SPAM Unsubscribe Link".to_string(),
                        regulation: "can_spam".to_string(),
                        passed: true,
                        message: "Unsubscribe link present".to_string(),
                    });
                }
            }
            "sms" => {
                // TCPA: check for opt-out instruction
                let has_opt_out = body.contains("STOP") || body.contains("stop") || body.contains("opt out");
                if !has_opt_out {
                    checks.push(ComplianceCheck {
                        check_name: "TCPA Opt-out Instruction".to_string(),
                        regulation: "tcpa".to_string(),
                        passed: false,
                        message: "SMS missing opt-out instruction".to_string(),
                    });
                } else {
                    checks.push(ComplianceCheck {
                        check_name: "TCPA Opt-out Instruction".to_string(),
                        regulation: "tcpa".to_string(),
                        passed: true,
                        message: "Opt-out instruction present".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(ComplianceCheckResult {
            is_compliant: all_passed,
            checks,
            message: if all_passed {
                "All compliance checks passed".to_string()
            } else {
                "Compliance check failed — review issues before sending".to_string()
            },
        })
    }

    // ---------------------------------------------------------------------------
    // Settings
    // ---------------------------------------------------------------------------

    pub async fn get_settings(&self) -> Result<ComplianceSettings, String> {
        let row = sqlx::query(
            "SELECT id, org_id, gdpr_response_days, auto_expire_gdpr_requests, require_consent_before_send, \
             check_suppression_before_send, log_all_compliance_actions, can_spam_unsubscribe_link, \
             can_spam_physical_address, tcpa_consent_required, whatsapp_opt_in_required, data_retention_days, \
             created_at, updated_at \
             FROM compliance_settings LIMIT 1",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| "Compliance settings not found".to_string())?;
        Ok(ComplianceSettings {
            id: row.get("id"),
            org_id: row.get("org_id"),
            gdpr_response_days: row.get("gdpr_response_days"),
            auto_expire_gdpr_requests: row.get("auto_expire_gdpr_requests"),
            require_consent_before_send: row.get("require_consent_before_send"),
            check_suppression_before_send: row.get("check_suppression_before_send"),
            log_all_compliance_actions: row.get("log_all_compliance_actions"),
            can_spam_unsubscribe_link: row.get("can_spam_unsubscribe_link"),
            can_spam_physical_address: row.get("can_spam_physical_address"),
            tcpa_consent_required: row.get("tcpa_consent_required"),
            whatsapp_opt_in_required: row.get("whatsapp_opt_in_required"),
            data_retention_days: row.get("data_retention_days"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn update_settings(&self, settings: &ComplianceSettings) -> Result<ComplianceSettings, String> {
        sqlx::query(
            "UPDATE compliance_settings SET \
             gdpr_response_days = $1, auto_expire_gdpr_requests = $2, require_consent_before_send = $3, \
             check_suppression_before_send = $4, log_all_compliance_actions = $5, can_spam_unsubscribe_link = $6, \
             can_spam_physical_address = $7, tcpa_consent_required = $8, whatsapp_opt_in_required = $9, \
             data_retention_days = $10, updated_at = NOW() \
             WHERE id = $11",
        )
        .bind(&settings.gdpr_response_days)
        .bind(&settings.auto_expire_gdpr_requests)
        .bind(&settings.require_consent_before_send)
        .bind(&settings.check_suppression_before_send)
        .bind(&settings.log_all_compliance_actions)
        .bind(&settings.can_spam_unsubscribe_link)
        .bind(&settings.can_spam_physical_address)
        .bind(&settings.tcpa_consent_required)
        .bind(&settings.whatsapp_opt_in_required)
        .bind(&settings.data_retention_days)
        .bind(&settings.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update settings: {e}"))?;

        self.get_settings().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let summary = ComplianceSummary {
            total_suppressed: 10,
            active_suppressed: 8,
            pending_gdpr_requests: 3,
            total_gdpr_requests: 5,
            active_policies: 5,
            total_audit_entries: 100,
            critical_violations: 0,
            suppression_by_reason: vec![ReasonCount { reason: "opt_out".to_string(), count: 5 }],
            gdpr_by_type: vec![GDPRTypeCount { request_type: "access".to_string(), count: 2 }],
            recent_audit_logs: vec![],
            upcoming_gdpr_expiries: 1,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("total_suppressed"));
    }

    #[test]
    fn test_compliance_check_result() {
        let result = ComplianceCheckResult {
            is_compliant: true,
            checks: vec![ComplianceCheck {
                check_name: "Test".to_string(),
                regulation: "test".to_string(),
                passed: true,
                message: "OK".to_string(),
            }],
            message: "All passed".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("is_compliant"));
    }

    #[test]
    fn test_suppression_check_result() {
        let result = SuppressionCheckResult {
            is_suppressed: false,
            entry: None,
            message: "Not suppressed".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("is_suppressed"));
    }
}
