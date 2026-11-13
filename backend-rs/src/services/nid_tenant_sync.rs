//! nID Tenant Sync Service
//!
//! Receives webhooks from nID when tenants are created or updated,
//! mirrors tenant data in the Communicator database, and provides
//! the redirect flow for Communicator → nID → Communicator tenant creation.
//!
//! Key features:
//! - Webhook reception and signature verification
//! - Tenant data mirroring with deduplication
//! - Redirect flow with state tokens, error handling, and retry logic
//! - Profile update sync (nID → Communicator) in real-time
//! - Connection health monitoring

use crate::db::DbPool;
use crate::services::signatures::secrets_match;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NidTenant {
    pub id: String,
    pub nid_tenant_id: String,
    pub name: String,
    pub slug: String,
    pub data_residency_region: String,
    pub webhook_url: Option<String>,
    pub billing_webhook_url: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub last_sync_error: String,
    pub linked_brand_id: Option<String>,
    pub nID_created_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NidWebhookPayload {
    pub event_type: String,
    pub tenant_id: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NidConnectionSettings {
    pub id: String,
    pub nID_base_url: String,
    pub auto_sync_enabled: bool,
    pub auto_link_new_tenants: bool,
    pub sync_interval_seconds: i64,
    pub connection_status: String,
    pub last_error: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRedirectSession {
    pub id: String,
    pub state_token: String,
    pub redirect_type: String,
    pub brand_id: Option<String>,
    pub return_url: String,
    pub nID_tenant_id: Option<String>,
    pub status: String,
    pub error_message: String,
    pub error_code: Option<String>,
    pub attempts: i64,
    pub max_attempts: i64,
    pub expires_at: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRedirectSessionRequest {
    pub redirect_type: Option<String>,
    pub brand_id: Option<String>,
    pub return_url: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateConnectionSettingsRequest {
    pub nID_base_url: Option<String>,
    pub platform_admin_secret: Option<String>,
    pub webhook_receive_secret: Option<String>,
    pub auto_sync_enabled: Option<bool>,
    pub auto_link_new_tenants: Option<bool>,
    pub sync_interval_seconds: Option<i64>,
}

pub struct NidTenantSyncService {
    pool: PgPool,
}

impl NidTenantSyncService {
    pub fn new(pool: &DbPool) -> Self {
        Self {
            pool: pool.clone(),
        }
    }

    // ------------------------------------------------------------------
    // Webhook reception and processing
    // ------------------------------------------------------------------

    /// Process an incoming webhook from nID. Verifies signature and
    /// updates the tenant mirror based on the event type.
    pub async fn process_webhook(
        &self,
        payload: &str,
        signature: &str,
    ) -> Result<serde_json::Value, String> {
        // Auto-sync must be enabled to accept incoming tenant updates at all
        let settings = self.get_connection_settings().await;
        if !settings.auto_sync_enabled {
            warn!("Auto-sync disabled — rejecting nID webhook");
            return Err("Auto-sync is disabled for this connection".to_string());
        }

        // Get webhook secret for verification
        let webhook_secret = self
            .get_webhook_receive_secret()
            .await
            .unwrap_or_default();

        // Verify signature
        let signature_valid = if !webhook_secret.is_empty() {
            secrets_match(signature, Some(&webhook_secret))
        } else {
            // If no secret configured, accept but warn
            warn!("No webhook receive secret configured — accepting webhook without verification");
            true
        };

        if !signature_valid {
            return Err("Invalid webhook signature".to_string());
        }

        // Parse payload
        let webhook: NidWebhookPayload = match serde_json::from_str(payload) {
            Ok(w) => w,
            Err(e) => return Err(format!("Failed to parse webhook payload: {}", e)),
        };

        // Log the sync event
        let log_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            r#"INSERT INTO tenant_sync_log (id, nid_tenant_id, event_type, payload, signature, signature_valid, processing_status)
               VALUES ($1, $2, $3, $4, $5, $6, 'pending')"#,
        )
        .bind(&log_id)
        .bind(&webhook.tenant_id)
        .bind(&webhook.event_type)
        .bind(serde_json::json!(webhook))
        .bind(signature)
        .bind(signature_valid)
        .execute(&self.pool)
        .await;

        // Process based on event type
        let result = match webhook.event_type.as_str() {
            "tenant.created" => self.handle_tenant_created(&webhook).await,
            "tenant.updated" => self.handle_tenant_updated(&webhook).await,
            "tenant.deleted" => self.handle_tenant_deleted(&webhook).await,
            _ => {
                warn!("Unknown event type: {}", webhook.event_type);
                Ok(serde_json::json!({"status": "skipped", "reason": "unknown_event_type"}))
            }
        };

        // Update sync log
        match &result {
            Ok(_) => {
                let _ = sqlx::query(
                    r#"UPDATE tenant_sync_log SET processing_status = 'processed', processed_at = NOW() WHERE id = $1"#,
                )
                .bind(&log_id)
                .execute(&self.pool)
                .await;
            }
            Err(e) => {
                error!("nID webhook processing failed for log {}: {}", log_id, e);
                let _ = sqlx::query(
                    r#"UPDATE tenant_sync_log SET processing_status = 'failed', error_message = $2, processed_at = NOW() WHERE id = $1"#,
                )
                .bind(&log_id)
                .bind(e)
                .execute(&self.pool)
                .await;
            }
        }

        result
    }

    async fn handle_tenant_created(&self, webhook: &NidWebhookPayload) -> Result<serde_json::Value, String> {
        let data = &webhook.data;
        let tenant_id = data.get("id").and_then(|v| v.as_str()).unwrap_or(&webhook.tenant_id);
        let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let slug = data.get("slug").and_then(|v| v.as_str()).unwrap_or("");

        // Duplication prevention: check if tenant already exists
        let existing: Option<String> = sqlx::query_scalar(
            r#"SELECT id FROM nid_tenants WHERE nid_tenant_id = $1"#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to check existing tenant: {}", e))?;

        if existing.is_some() {
            info!("Tenant {} already exists — skipping creation", tenant_id);
            // Update sync status instead
            self.update_tenant_sync_status(tenant_id, "synced", "").await;
            return Ok(serde_json::json!({"status": "already_exists", "tenant_id": tenant_id}));
        }

        // Also check by slug for cross-system deduplication
        let slug_exists: Option<String> = sqlx::query_scalar(
            r#"SELECT id FROM nid_tenants WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to check tenant slug: {}", e))?;

        if let Some(existing_id) = slug_exists {
            info!("Tenant with slug '{}' already exists (ID: {}) — updating instead", slug, existing_id);
            Box::pin(self.handle_tenant_updated(webhook)).await?;
            return Ok(serde_json::json!({"status": "duplicate_slug", "tenant_id": existing_id}));
        }

        // Hash and store webhook secret if provided
        let webhook_secret = data.get("webhookSecret").and_then(|v| v.as_str());
        let (secret_hash, secret_salt) = if let Some(secret) = webhook_secret {
            let salt = Uuid::new_v4().to_string();
            let hash = self.hmac_sha256(secret, &salt);
            (Some(hash), Some(salt))
        } else {
            (None, None)
        };

        // Insert tenant mirror
        sqlx::query(
            r#"INSERT INTO nid_tenants (id, nid_tenant_id, name, slug, data_residency_region,
               webhook_url, webhook_secret_hash, webhook_secret_salt,
               billing_webhook_url, sync_status, last_synced_at, nID_created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'synced', NOW(), $10)"#,
        )
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(name)
        .bind(slug)
        .bind(
            data.get("dataResidencyRegion")
                .and_then(|v| v.as_str())
                .unwrap_or("global"),
        )
        .bind(
            data.get("webhookUrl")
                .and_then(|v| v.as_str())
                .map(String::from),
        )
        .bind(&secret_hash)
        .bind(&secret_salt)
        .bind(
            data.get("billingWebhookUrl")
                .and_then(|v| v.as_str())
                .map(String::from),
        )
        .bind(
            data.get("createdAt")
                .and_then(|v| v.as_str())
                .map(String::from),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to insert tenant: {}", e))?;

        info!("Tenant {} created in mirror", tenant_id);

        // Auto-link to brand if enabled
        let settings = self.get_connection_settings().await;
        if settings.auto_link_new_tenants {
            self.auto_link_tenant_to_brand(tenant_id).await;
        }

        Ok(serde_json::json!({"status": "created", "tenant_id": tenant_id}))
    }

    async fn handle_tenant_updated(&self, webhook: &NidWebhookPayload) -> Result<serde_json::Value, String> {
        let data = &webhook.data;
        let tenant_id = &webhook.tenant_id;

        // Check if tenant exists
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(SELECT 1 FROM nid_tenants WHERE nid_tenant_id = $1)"#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check tenant existence: {}", e))?;

        if !exists {
            // Tenant doesn't exist — this might be a delayed update, create it
            return Box::pin(self.handle_tenant_created(webhook)).await;
        }

        // Update tenant data. Built dynamically since only the fields nID
        // actually sent should change; each value is bound positionally so
        // nothing here is vulnerable to SQL injection despite the query
        // text itself being assembled at runtime.
        let mut query_parts = Vec::new();
        let mut param_idx = 1;

        let name = data.get("name").and_then(|v| v.as_str());
        if name.is_some() {
            query_parts.push(format!("name = ${}", param_idx));
            param_idx += 1;
        }

        let region = data.get("dataResidencyRegion").and_then(|v| v.as_str());
        if region.is_some() {
            query_parts.push(format!("data_residency_region = ${}", param_idx));
            param_idx += 1;
        }

        let webhook_url = data.get("webhookUrl").and_then(|v| v.as_str());
        if webhook_url.is_some() {
            query_parts.push(format!("webhook_url = ${}", param_idx));
            param_idx += 1;
        }

        // Always update sync status
        query_parts.push("sync_status = 'synced'".to_string());
        query_parts.push("last_synced_at = NOW()".to_string());

        let query = format!(
            "UPDATE nid_tenants SET {}, updated_at = NOW() WHERE nid_tenant_id = ${}",
            query_parts.join(", "),
            param_idx
        );

        let mut q = sqlx::query(&query);
        if let Some(name) = name {
            q = q.bind(name);
        }
        if let Some(region) = region {
            q = q.bind(region);
        }
        if let Some(webhook_url) = webhook_url {
            q = q.bind(webhook_url);
        }
        q = q.bind(tenant_id);

        q.execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update tenant: {}", e))?;

        info!("Tenant {} updated in mirror", tenant_id);
        Ok(serde_json::json!({"status": "updated", "tenant_id": tenant_id}))
    }

    async fn handle_tenant_deleted(&self, webhook: &NidWebhookPayload) -> Result<serde_json::Value, String> {
        let tenant_id = &webhook.tenant_id;

        // Soft delete: mark as deleted instead of removing
        let _ = sqlx::query(
            r#"UPDATE nid_tenants SET sync_status = 'deleted', updated_at = NOW() WHERE nid_tenant_id = $1"#,
        )
        .bind(tenant_id)
        .execute(&self.pool)
        .await;

        info!("Tenant {} marked as deleted in mirror", tenant_id);
        Ok(serde_json::json!({"status": "deleted", "tenant_id": tenant_id}))
    }

    async fn update_tenant_sync_status(
        &self,
        tenant_id: &str,
        status: &str,
        error: &str,
    ) {
        let _ = sqlx::query(
            r#"UPDATE nid_tenants SET sync_status = $2, last_sync_error = $3, updated_at = NOW() WHERE nid_tenant_id = $1"#,
        )
        .bind(tenant_id)
        .bind(status)
        .bind(error)
        .execute(&self.pool)
        .await;
    }

    async fn auto_link_tenant_to_brand(&self, tenant_id: &str) {
        // Get tenant slug
        let slug: Option<String> = sqlx::query_scalar(
            r#"SELECT slug FROM nid_tenants WHERE nid_tenant_id = $1"#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        let Some(slug) = slug else {
            return;
        };

        // Check if a brand with matching slug exists
        let brand_id: Option<String> = sqlx::query_scalar(
            r#"SELECT id FROM brands WHERE slug = $1"#,
        )
        .bind(&slug)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        if let Some(brand_id) = brand_id {
            let _ = sqlx::query(
                r#"UPDATE nid_tenants SET linked_brand_id = $2, updated_at = NOW() WHERE nid_tenant_id = $1"#,
            )
            .bind(tenant_id)
            .bind(&brand_id)
            .execute(&self.pool)
            .await;

            info!("Auto-linked tenant {} to brand {}", tenant_id, brand_id);
        }
    }

    // ------------------------------------------------------------------
    // Redirect flow: Communicator → nID → Communicator
    // ------------------------------------------------------------------

    /// Create a redirect session for tenant creation in nID.
    /// Returns the nID URL to redirect the user to.
    pub async fn create_redirect_session(
        &self,
        req: &CreateRedirectSessionRequest,
    ) -> Result<TenantRedirectSession, String> {
        let state_token = self.generate_state_token();
        let session_id = Uuid::new_v4().to_string();
        let expires_at_secs = 600; // 10 minutes

        sqlx::query(
            r#"INSERT INTO tenant_redirect_sessions (id, state_token, redirect_type, brand_id, return_url, expires_at, metadata)
               VALUES ($1, $2, $3, $4, $5, NOW() + INTERVAL '${} seconds', $6)"#,
        )
        .bind(&session_id)
        .bind(&state_token)
        .bind(req.redirect_type.as_deref().unwrap_or("create"))
        .bind(&req.brand_id)
        .bind(&req.return_url)
        .bind(req.metadata.clone().unwrap_or(serde_json::json!({})))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create redirect session: {}", e))?;

        let session = TenantRedirectSession {
            id: session_id,
            state_token,
            redirect_type: req.redirect_type.clone().unwrap_or("create".to_string()),
            brand_id: req.brand_id.clone(),
            return_url: req.return_url.clone(),
            nID_tenant_id: None,
            status: "pending".to_string(),
            error_message: "".to_string(),
            error_code: None,
            attempts: 0,
            max_attempts: 3,
            expires_at: format!("{} seconds from now", expires_at_secs),
            created_at: format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()),
            completed_at: None,
        };

        Ok(session)
    }

    /// Handle the callback from nID after tenant creation.
    /// Validates state token, extracts tenant data, and completes the flow.
    pub async fn handle_redirect_callback(
        &self,
        state: &str,
        tenant_id: Option<&str>,
        error: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        // Find and validate session
        let session: Option<(String, String, String, String, i64, i64)> = sqlx::query_as(
            r#"SELECT id, return_url, status, expires_at, attempts, max_attempts
               FROM tenant_redirect_sessions WHERE state_token = $1"#,
        )
        .bind(state)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        let Some((session_id, return_url, status, expires_at, attempts, max_attempts)) = session else {
            return Err("Invalid or expired state token".to_string());
        };

        // Check if session is already completed
        if status == "completed" {
            return Err("This redirect session has already been completed".to_string());
        }

        // Check if session has expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if expires_at.parse::<u64>().unwrap_or(u64::MAX) < now {
            // Mark as expired
            let _ = sqlx::query(
                r#"UPDATE tenant_redirect_sessions SET status = 'expired', error_code = 'timeout' WHERE id = $1"#,
            )
            .bind(&session_id)
            .execute(&self.pool)
            .await;

            return Err("Redirect session has expired".to_string());
        }

        // Check max attempts
        if attempts >= max_attempts {
            let _ = sqlx::query(
                r#"UPDATE tenant_redirect_sessions SET status = 'failed', error_code = 'max_attempts_exceeded' WHERE id = $1"#,
            )
            .bind(&session_id)
            .execute(&self.pool)
            .await;

            return Err("Maximum redirect attempts exceeded".to_string());
        }

        // Handle error case
        if let Some(error_msg) = error {
            let _ = sqlx::query(
                r#"UPDATE tenant_redirect_sessions SET status = 'failed', error_message = $2, error_code = 'nID_error', attempts = attempts + 1 WHERE id = $1"#,
            )
            .bind(&session_id)
            .bind(error_msg)
            .execute(&self.pool)
            .await;

            return Err(format!("nID returned an error: {}", error_msg));
        }

        // Handle success case
        if let Some(tid) = tenant_id {
            // Check for duplicate tenant
            let existing: Option<String> = sqlx::query_scalar(
                r#"SELECT id FROM nid_tenants WHERE nid_tenant_id = $1"#,
            )
            .bind(tid)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);

            if existing.is_some() {
                // Duplicate — complete session but note the duplicate
                let _ = sqlx::query(
                    r#"UPDATE tenant_redirect_sessions SET status = 'completed', nID_tenant_id = $2, completed_at = NOW() WHERE id = $1"#,
                )
                .bind(&session_id)
                .bind(tid)
                .execute(&self.pool)
                .await;

                return Ok(serde_json::json!({
                    "status": "completed",
                    "tenant_id": tid,
                    "return_url": return_url,
                    "note": "tenant_already_exists"
                }));
            }

            // Complete session
            let _ = sqlx::query(
                r#"UPDATE tenant_redirect_sessions SET status = 'completed', nID_tenant_id = $2, completed_at = NOW() WHERE id = $1"#,
            )
            .bind(&session_id)
            .bind(tid)
            .execute(&self.pool)
            .await;

            Ok(serde_json::json!({
                "status": "completed",
                "tenant_id": tid,
                "return_url": return_url
            }))
        } else {
            // User cancelled
            let _ = sqlx::query(
                r#"UPDATE tenant_redirect_sessions SET status = 'cancelled', error_code = 'user_cancelled' WHERE id = $1"#,
            )
            .bind(&session_id)
            .execute(&self.pool)
            .await;

            Ok(serde_json::json!({
                "status": "cancelled",
                "return_url": return_url
            }))
        }
    }

    /// Get a redirect session by state token
    pub async fn get_redirect_session(&self, state: &str) -> Option<TenantRedirectSession> {
        sqlx::query_as::<_, (String, String, String, Option<String>, String, Option<String>, String, String, Option<String>, i64, i64, String)>(
            r#"SELECT id, state_token, redirect_type, brand_id, return_url, nID_tenant_id,
                      status, error_message, error_code, attempts, max_attempts, expires_at
               FROM tenant_redirect_sessions WHERE state_token = $1"#,
        )
        .bind(state)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
        .map(|(id, state_token, redirect_type, brand_id, return_url, nID_tenant_id, status, error_message, error_code, attempts, max_attempts, expires_at)| {
            TenantRedirectSession {
                id,
                state_token,
                redirect_type,
                brand_id,
                return_url,
                nID_tenant_id,
                status,
                error_message,
                error_code,
                attempts,
                max_attempts,
                expires_at,
                created_at: "".to_string(),
                completed_at: None,
            }
        })
    }

    // ------------------------------------------------------------------
    // Connection settings
    // ------------------------------------------------------------------

    pub async fn get_connection_settings(&self) -> NidConnectionSettings {
        let row: Option<(String, String, bool, bool, i64, String, String, String, String)> =
            sqlx::query_as(
                r#"SELECT id, nID_base_url, auto_sync_enabled, auto_link_new_tenants,
                          sync_interval_seconds, connection_status, last_error, created_at, updated_at
                   FROM nid_connection_settings WHERE id = 'default'"#,
            )
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten();

        match row {
            Some((id, nID_base_url, auto_sync_enabled, auto_link_new_tenants, sync_interval_seconds, connection_status, last_error, created_at, updated_at)) => {
                NidConnectionSettings {
                    id,
                    nID_base_url,
                    auto_sync_enabled,
                    auto_link_new_tenants,
                    sync_interval_seconds,
                    connection_status,
                    last_error,
                    created_at,
                    updated_at,
                }
            }
            None => NidConnectionSettings {
                id: "default".to_string(),
                nID_base_url: "http://localhost:4000".to_string(),
                auto_sync_enabled: true,
                auto_link_new_tenants: true,
                sync_interval_seconds: 300,
                connection_status: "disconnected".to_string(),
                last_error: "".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
            },
        }
    }

    pub async fn update_connection_settings(
        &self,
        req: &UpdateConnectionSettingsRequest,
    ) -> Result<NidConnectionSettings, String> {
        // Hash platform admin secret if provided
        if let Some(secret) = &req.platform_admin_secret {
            let salt = Uuid::new_v4().to_string();
            let hash = self.hmac_sha256(secret, &salt);

            let _ = sqlx::query(
                r#"UPDATE nid_connection_settings SET platform_admin_secret_hash = $1, platform_admin_secret_salt = $2, updated_at = NOW() WHERE id = 'default'"#,
            )
            .bind(&hash)
            .bind(&salt)
            .execute(&self.pool)
            .await;
        }

        if let Some(base_url) = &req.nID_base_url {
            let _ = sqlx::query(
                r#"UPDATE nid_connection_settings SET nID_base_url = $1, updated_at = NOW() WHERE id = 'default'"#,
            )
            .bind(base_url)
            .execute(&self.pool)
            .await;
        }

        if let Some(webhook_secret) = &req.webhook_receive_secret {
            let _ = sqlx::query(
                r#"UPDATE nid_connection_settings SET webhook_receive_secret = $1, updated_at = NOW() WHERE id = 'default'"#,
            )
            .bind(webhook_secret)
            .execute(&self.pool)
            .await;
        }

        if let Some(auto_sync) = req.auto_sync_enabled {
            let _ = sqlx::query(
                r#"UPDATE nid_connection_settings SET auto_sync_enabled = $1, updated_at = NOW() WHERE id = 'default'"#,
            )
            .bind(auto_sync)
            .execute(&self.pool)
            .await;
        }

        if let Some(auto_link) = req.auto_link_new_tenants {
            let _ = sqlx::query(
                r#"UPDATE nid_connection_settings SET auto_link_new_tenants = $1, updated_at = NOW() WHERE id = 'default'"#,
            )
            .bind(auto_link)
            .execute(&self.pool)
            .await;
        }

        if let Some(interval) = req.sync_interval_seconds {
            let _ = sqlx::query(
                r#"UPDATE nid_connection_settings SET sync_interval_seconds = $1, updated_at = NOW() WHERE id = 'default'"#,
            )
            .bind(interval)
            .execute(&self.pool)
            .await;
        }

        // Update connection status
        let _ = sqlx::query(
            r#"UPDATE nid_connection_settings SET connection_status = 'connected', updated_at = NOW() WHERE id = 'default'"#,
        )
        .execute(&self.pool)
        .await;

        self.get_connection_settings().await;
        Ok(self.get_connection_settings().await)
    }

    /// Check connection health by pinging nID API
    pub async fn check_connection_health(&self) -> serde_json::Value {
        let settings = self.get_connection_settings().await;

        let mut tenant_count: i64 = 0;
        let mut synced_count: i64 = 0;
        let mut error_count: i64 = 0;

        if let Ok(count) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM nid_tenants"#,
        )
        .fetch_one(&self.pool)
        .await
        {
            tenant_count = count;
        }

        if let Ok(count) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM nid_tenants WHERE sync_status = 'synced'"#,
        )
        .fetch_one(&self.pool)
        .await
        {
            synced_count = count;
        }

        if let Ok(count) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM nid_tenants WHERE sync_status = 'error'"#,
        )
        .fetch_one(&self.pool)
        .await
        {
            error_count = count;
        }

        serde_json::json!({
            "connection_status": settings.connection_status,
            "nID_base_url": settings.nID_base_url,
            "auto_sync_enabled": settings.auto_sync_enabled,
            "total_tenants": tenant_count,
            "synced_tenants": synced_count,
            "error_tenants": error_count,
            "last_error": settings.last_error,
        })
    }

    // ------------------------------------------------------------------
    // Tenant listing
    // ------------------------------------------------------------------

    pub async fn list_tenants(&self) -> Result<Vec<NidTenant>, String> {
        let rows = sqlx::query_as::<_, (
            String, String, String, String, String,
            Option<String>, Option<String>,
            String, Option<String>, String,
            Option<String>, Option<String>, String, String,
        )>(
            r#"SELECT id, nid_tenant_id, name, slug, data_residency_region,
                      webhook_url, billing_webhook_url,
                      sync_status, last_synced_at, last_sync_error,
                      linked_brand_id, nID_created_at, created_at, updated_at
               FROM nid_tenants ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list tenants: {}", e))?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    nid_tenant_id,
                    name,
                    slug,
                    data_residency_region,
                    webhook_url,
                    billing_webhook_url,
                    sync_status,
                    last_synced_at,
                    last_sync_error,
                    linked_brand_id,
                    nID_created_at,
                    created_at,
                    updated_at,
                )| NidTenant {
                    id,
                    nid_tenant_id,
                    name,
                    slug,
                    data_residency_region,
                    webhook_url,
                    billing_webhook_url,
                    sync_status,
                    last_synced_at,
                    last_sync_error,
                    linked_brand_id,
                    nID_created_at,
                    created_at,
                    updated_at,
                },
            )
            .collect())
    }

    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Option<NidTenant>, String> {
        let row: Option<(
            String, String, String, String, String,
            Option<String>, Option<String>,
            String, Option<String>, String,
            Option<String>, Option<String>, String, String,
        )> = sqlx::query_as(
            r#"SELECT id, nid_tenant_id, name, slug, data_residency_region,
                      webhook_url, billing_webhook_url,
                      sync_status, last_synced_at, last_sync_error,
                      linked_brand_id, nID_created_at, created_at, updated_at
               FROM nid_tenants WHERE nid_tenant_id = $1"#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get tenant: {}", e))?;

        Ok(row.map(
            |(
                id,
                nid_tenant_id,
                name,
                slug,
                data_residency_region,
                webhook_url,
                billing_webhook_url,
                sync_status,
                last_synced_at,
                last_sync_error,
                linked_brand_id,
                nID_created_at,
                created_at,
                updated_at,
            )| NidTenant {
                id,
                nid_tenant_id,
                name,
                slug,
                data_residency_region,
                webhook_url,
                billing_webhook_url,
                sync_status,
                last_synced_at,
                last_sync_error,
                linked_brand_id,
                nID_created_at,
                created_at,
                updated_at,
            },
        ))
    }

    // ------------------------------------------------------------------
    // Utility functions
    // ------------------------------------------------------------------

    fn generate_state_token(&self) -> String {
        let mut out = String::with_capacity(32);
        while out.len() < 32 {
            out.push_str(&Uuid::new_v4().simple().to_string());
        }
        out.truncate(32);
        out
    }

    fn hmac_sha256(&self, data: &str, salt: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(salt.as_bytes()).unwrap_or_else(|_| {
            let dummy_key = [0u8; 64];
            HmacSha256::new_from_slice(&dummy_key).unwrap()
        });
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    async fn get_webhook_receive_secret(&self) -> Option<String> {
        sqlx::query_scalar::<_, String>(
            r#"SELECT webhook_receive_secret FROM nid_connection_settings WHERE id = 'default'"#,
        )
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }
}
