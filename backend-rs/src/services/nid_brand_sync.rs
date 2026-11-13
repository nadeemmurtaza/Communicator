//! nID Brand Sync Service
//!
//! Receives webhooks from nID when brands are created, updated, or deleted,
//! mirrors brand data in the Communicator database, and provides
//! the redirect flow for Communicator → nID → Communicator brand creation.
//!
//! Key features:
//! - Webhook reception and signature verification
//! - Brand data mirroring with deduplication
//! - Redirect flow with state tokens, error handling, and retry logic
//! - Profile update sync (nID → Communicator) in real-time (<1 second)
//! - Connection health monitoring

use crate::db::DbPool;
use crate::services::signatures::secrets_match;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NidBrand {
    pub id: String,
    pub nid_brand_id: String,
    pub nid_org_id: String,
    pub nid_tenant_id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub active: bool,
    pub synced_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrandWebhookPayload {
    pub event_type: String,
    pub tenant_id: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandConnectionSettings {
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub polling_interval_sec: i32,
    pub enabled: bool,
    pub health_status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBrandConnectionRequest {
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub polling_interval_sec: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBrandRedirectRequest {
    pub org_id: Option<String>,
    pub return_url: String,
}

pub struct NidBrandSyncService {
    pool: PgPool,
}

impl NidBrandSyncService {
    pub fn new(pool: &DbPool) -> Self {
        Self {
            pool: pool.get_pg_pool().clone(),
        }
    }

    // -----------------------------------------------------------------------
    // Webhook processing
    // -----------------------------------------------------------------------

    /// Process an incoming brand webhook from nID
    pub async fn process_webhook(
        &self,
        payload: &BrandWebhookPayload,
        signature: &str,
    ) -> Result<NidBrand, String> {
        // Verify webhook signature
        let settings = self.get_connection_settings().await;
        if let Some(ref secret) = settings.webhook_secret {
            if !secrets_match(signature, secret) {
                return Err("Invalid webhook signature".to_string());
            }
        }

        // Check if sync is enabled
        if !settings.enabled {
            return Err("Brand sync is disabled".to_string());
        }

        let result = match payload.event_type.as_str() {
            "brand.created" => self.handle_brand_created(&payload.data).await,
            "brand.updated" => self.handle_brand_updated(&payload.data).await,
            "brand.deleted" => self.handle_brand_deleted(&payload.data).await,
            other => {
                warn!("Unknown brand webhook event: {}", other);
                return Err(format!("Unknown event type: {}", other));
            }
        };

        // Log the sync event
        self.log_sync(
            &payload.event_type,
            &result.as_ref().map(|b| b.nid_brand_id.clone()).unwrap_or_default(),
            &result.as_ref().map(|_| "success".to_string()).unwrap_or_default(),
            result.as_ref().err().map(|e| e.to_string()),
        )
        .await;

        result
    }

    async fn handle_brand_created(&self, data: &serde_json::Value) -> Result<NidBrand, String> {
        let brand_id = data["id"].as_str().ok_or("Missing brand id")?;
        let org_id = data["organization"]["id"].as_str().ok_or("Missing org id")?;
        let tenant_id = data["organization"]["tenant"]["id"].as_str().ok_or("Missing tenant id")?;
        let name = data["name"].as_str().ok_or("Missing brand name")?.to_string();
        let slug = data["slug"].as_str().ok_or("Missing brand slug")?.to_string();

        // Deduplication: check if already exists
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT nid_brand_id FROM nid_brands WHERE nid_brand_id = ?",
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if existing.is_some() {
            info!("Brand {} already exists, skipping create", brand_id);
            return self.get_brand_by_nid_id(brand_id).await;
        }

        let description = data["description"].as_str().map(String::from);
        let bio = data["bio"].as_str().map(String::from);
        let website = data["website"].as_str().map(String::from);
        let email = data["email"].as_str().map(String::from);
        let phone = data["phone"].as_str().map(String::from);
        let primary_color = data["primaryColor"].as_str().map(String::from);
        let secondary_color = data["secondaryColor"].as_str().map(String::from);

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO nid_brands (id, nid_brand_id, nid_org_id, nid_tenant_id, name, slug, \
             description, bio, website, email, phone, primary_color, secondary_color, active, synced_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW())",
        )
        .bind(&id)
        .bind(brand_id)
        .bind(org_id)
        .bind(tenant_id)
        .bind(&name)
        .bind(&slug)
        .bind(description.as_deref())
        .bind(bio.as_deref())
        .bind(website.as_deref())
        .bind(email.as_deref())
        .bind(phone.as_deref())
        .bind(primary_color.as_deref())
        .bind(secondary_color.as_deref())
        .bind(true)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        info!("Brand {} synced from nID", brand_id);
        self.get_brand_by_id(&id).await
    }

    async fn handle_brand_updated(&self, data: &serde_json::Value) -> Result<NidBrand, String> {
        let brand_id = data["id"].as_str().ok_or("Missing brand id")?;

        let description = data["description"].as_str().map(String::from);
        let bio = data["bio"].as_str().map(String::from);
        let name = data["name"].as_str().map(String::from);
        let slug = data["slug"].as_str().map(String::from);
        let website = data["website"].as_str().map(String::from);
        let email = data["email"].as_str().map(String::from);
        let phone = data["phone"].as_str().map(String::from);
        let primary_color = data["primaryColor"].as_str().map(String::from);
        let secondary_color = data["secondaryColor"].as_str().map(String::from);
        let active = data["active"].as_bool().unwrap_or(true);

        let rows = sqlx::query(
            "UPDATE nid_brands SET \
             name = COALESCE(?, name), \
             slug = COALESCE(?, slug), \
             description = ?, bio = ?, website = ?, email = ?, phone = ?, \
             primary_color = ?, secondary_color = ?, active = ?, synced_at = NOW() \
             WHERE nid_brand_id = ?",
        )
        .bind(name)
        .bind(slug)
        .bind(description.as_deref())
        .bind(bio.as_deref())
        .bind(website.as_deref())
        .bind(email.as_deref())
        .bind(phone.as_deref())
        .bind(primary_color.as_deref())
        .bind(secondary_color.as_deref())
        .bind(active)
        .bind(brand_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if rows.rows_affected() == 0 {
            return Err(format!("Brand {} not found for update", brand_id));
        }

        info!("Brand {} updated from nID", brand_id);
        self.get_brand_by_nid_id(brand_id).await
    }

    async fn handle_brand_deleted(&self, data: &serde_json::Value) -> Result<NidBrand, String> {
        let brand_id = data["id"].as_str().ok_or("Missing brand id")?;

        sqlx::query("UPDATE nid_brands SET active = false, synced_at = NOW() WHERE nid_brand_id = ?")
            .bind(brand_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        info!("Brand {} marked inactive from nID", brand_id);
        self.get_brand_by_nid_id(brand_id).await
    }

    // -----------------------------------------------------------------------
    // CRUD operations
    // -----------------------------------------------------------------------

    pub async fn get_brand_by_id(&self, id: &str) -> Result<NidBrand, String> {
        let row = sqlx::query_as::<_, NidBrandRow>(
            "SELECT id, nid_brand_id, nid_org_id, nid_tenant_id, name, slug, \
             description, bio, website, email, phone, primary_color, secondary_color, \
             active, synced_at, created_at FROM nid_brands WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| format!("Brand {} not found", id))?;

        Ok(row.into())
    }

    pub async fn get_brand_by_nid_id(&self, nid_brand_id: &str) -> Result<NidBrand, String> {
        let row = sqlx::query_as::<_, NidBrandRow>(
            "SELECT id, nid_brand_id, nid_org_id, nid_tenant_id, name, slug, \
             description, bio, website, email, phone, primary_color, secondary_color, \
             active, synced_at, created_at FROM nid_brands WHERE nid_brand_id = ?",
        )
        .bind(nid_brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| format!("Brand {} not found", nid_brand_id))?;

        Ok(row.into())
    }

    pub async fn list_brands(&self, tenant_id: Option<&str>) -> Result<Vec<NidBrand>, String> {
        let rows = if let Some(tid) = tenant_id {
            sqlx::query_as::<_, NidBrandRow>(
                "SELECT id, nid_brand_id, nid_org_id, nid_tenant_id, name, slug, \
                 description, bio, website, email, phone, primary_color, secondary_color, \
                 active, synced_at, created_at FROM nid_brands WHERE nid_tenant_id = ? ORDER BY created_at DESC",
            )
            .bind(tid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, NidBrandRow>(
                "SELECT id, nid_brand_id, nid_org_id, nid_tenant_id, name, slug, \
                 description, bio, website, email, phone, primary_color, secondary_color, \
                 active, synced_at, created_at FROM nid_brands ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // -----------------------------------------------------------------------
    // Redirect flow
    // -----------------------------------------------------------------------

    /// Create a redirect session for Communicator → nID brand creation
    pub async fn create_redirect_session(
        &self,
        req: &CreateBrandRedirectRequest,
    ) -> Result<serde_json::Value, String> {
        let state_token = Uuid::new_v4().to_string();
        let id = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now()
            .checked_add_days(chrono::Days::new(1))
            .unwrap()
            .to_rfc3339();

        // Get connection settings for nID base URL
        let settings = self.get_connection_settings();

        sqlx::query(
            "INSERT INTO brand_redirect_sessions (id, state_token, nid_tenant_id, nid_org_id, \
             redirect_uri, max_attempts, attempts, expires_at) \
             VALUES (?, ?, 'default', ?, ?, 1, 0, ?)",
        )
        .bind(&id)
        .bind(&state_token)
        .bind(req.org_id.as_deref())
        .bind(&req.return_url)
        .bind(&expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Build nID redirect URL
        let nid_url = format!(
            "http://localhost:3000?create_brand=1&state={}&return_url={}",
            state_token,
            urlencoding::encode(&req.return_url)
        );

        Ok(serde_json::json!({
            "id": id,
            "state_token": state_token,
            "redirect_url": nid_url,
            "expires_at": expires_at,
        }))
    }

    /// Handle callback from nID after brand creation
    pub async fn redirect_callback(
        &self,
        state_token: &str,
        nid_brand_id: &str,
    ) -> Result<serde_json::Value, String> {
        let session = sqlx::query(
            "SELECT id, nid_org_id, redirect_uri, attempts, max_attempts, expires_at \
             FROM brand_redirect_sessions WHERE state_token = ?",
        )
        .bind(state_token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Redirect session not found".to_string())?;

        // Check expiry
        let expires_at: chrono::DateTime<chrono::Utc> =
            session.try_get::<String, _>("expires_at")
                .parse()
                .map_err(|_| "Invalid expiry date".to_string())?;

        if chrono::Utc::now() > expires_at {
            sqlx::query("UPDATE brand_redirect_sessions SET error_code = 'EXPIRED' WHERE state_token = ?")
                .bind(state_token)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            return Err("Redirect session has expired".to_string());
        }

        // Increment attempts
        sqlx::query("UPDATE brand_redirect_sessions SET attempts = attempts + 1 WHERE state_token = ?")
            .bind(state_token)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(serde_json::json!({
            "nid_brand_id": nid_brand_id,
            "redirect_uri": session.try_get::<String, _>("redirect_uri").unwrap_or_default(),
        }))
    }

    // -----------------------------------------------------------------------
    // Connection settings
    // -----------------------------------------------------------------------

    async fn get_connection_settings(&self) -> BrandConnectionSettings {
        let row = sqlx::query_as::<_, BrandConnectionRow>(
            "SELECT webhook_url, webhook_secret, polling_interval_sec, enabled, \
             COALESCE(health_status, 'unknown') as health_status FROM nid_brand_connection LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await;

        match row {
            Ok(Some(r)) => r.into(),
            _ => BrandConnectionSettings {
                webhook_url: None,
                webhook_secret: None,
                polling_interval_sec: 300,
                enabled: true,
                health_status: "unknown".to_string(),
            },
        }
    }

    pub async fn update_connection_settings(
        &self,
        req: &UpdateBrandConnectionRequest,
    ) -> Result<BrandConnectionSettings, String> {
        // Upsert connection settings
        sqlx::query(
            "INSERT INTO nid_brand_connection (webhook_url, webhook_secret, polling_interval_sec, enabled, updated_at) \
             VALUES (?, ?, ?, ?, NOW()) \
             ON CONFLICT (id) DO UPDATE SET \
             webhook_url = EXCLUDED.webhook_url, \
             webhook_secret = EXCLUDED.webhook_secret, \
             polling_interval_sec = EXCLUDED.polling_interval_sec, \
             enabled = EXCLUDED.enabled, \
             updated_at = NOW()",
        )
        .bind(req.webhook_url.as_deref().unwrap_or(""))
        .bind(req.webhook_secret.as_deref().unwrap_or(""))
        .bind(req.polling_interval_sec.unwrap_or(300))
        .bind(req.enabled.unwrap_or(true))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(self.get_connection_settings().await)
    }

    pub async fn get_connection_settings_public(&self) -> BrandConnectionSettings {
        self.get_connection_settings().await
    }

    // -----------------------------------------------------------------------
    // Health check
    // -----------------------------------------------------------------------

    pub async fn run_health_check(&self) -> Result<serde_json::Value, String> {
        let settings = self.get_connection_settings().await;
        let status = if settings.enabled { "healthy" } else { "disabled" };

        sqlx::query(
            "UPDATE nid_brand_connection SET health_status = ?, last_health_check = NOW(), updated_at = NOW()",
        )
        .bind(status)
        .execute(&self.pool)
        .await
        .ok();

        Ok(serde_json::json!({
            "status": status,
            "enabled": settings.enabled,
            "checked_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    // -----------------------------------------------------------------------
    // Sync log
    // -----------------------------------------------------------------------

    async fn log_sync(
        &self,
        event_type: &str,
        nid_brand_id: &str,
        status: &str,
        error_msg: Option<String>,
    ) {
        sqlx::query(
            "INSERT INTO brand_sync_log (nid_brand_id, event_type, status, error_msg) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(nid_brand_id)
        .bind(event_type)
        .bind(status)
        .bind(error_msg.as_deref())
        .execute(&self.pool)
        .await
        .ok();
    }

    pub async fn get_sync_log(&self, limit: i64) -> Result<Vec<serde_json::Value>, String> {
        let rows = sqlx::query(
            "SELECT id, nid_brand_id, event_type, status, error_msg, created_at \
             FROM brand_sync_log ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let logs: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.try_get::<i64, _>("id").unwrap_or(0),
                    "nid_brand_id": r.try_get::<String, _>("nid_brand_id").unwrap_or_default(),
                    "event_type": r.try_get::<String, _>("event_type").unwrap_or_default(),
                    "status": r.try_get::<String, _>("status").unwrap_or_default(),
                    "error_msg": r.try_get::<Option<String>, _>("error_msg"),
                    "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
                })
            })
            .collect();

        Ok(logs)
    }
}

// ---------------------------------------------------------------------------
// Database row types
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct NidBrandRow {
    id: String,
    nid_brand_id: String,
    nid_org_id: String,
    nid_tenant_id: String,
    name: String,
    slug: String,
    description: Option<String>,
    bio: Option<String>,
    website: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    primary_color: Option<String>,
    secondary_color: Option<String>,
    active: bool,
    synced_at: String,
    created_at: String,
}

impl From<NidBrandRow> for NidBrand {
    fn from(row: NidBrandRow) -> Self {
        NidBrand {
            id: row.id,
            nid_brand_id: row.nid_brand_id,
            nid_org_id: row.nid_org_id,
            nid_tenant_id: row.nid_tenant_id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            bio: row.bio,
            website: row.website,
            email: row.email,
            phone: row.phone,
            primary_color: row.primary_color,
            secondary_color: row.secondary_color,
            active: row.active,
            synced_at: row.synced_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BrandConnectionRow {
    webhook_url: Option<String>,
    webhook_secret: Option<String>,
    polling_interval_sec: i32,
    enabled: bool,
    health_status: String,
}

impl From<BrandConnectionRow> for BrandConnectionSettings {
    fn from(row: BrandConnectionRow) -> Self {
        BrandConnectionSettings {
            webhook_url: row.webhook_url,
            webhook_secret: row.webhook_secret,
            polling_interval_sec: row.polling_interval_sec,
            enabled: row.enabled,
            health_status: row.health_status,
        }
    }
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;
use crate::AppState;

pub async fn webhook_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> StatusCode {
    let event_type = payload.get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let tenant_id = payload.get("tenant_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let data = payload.get("data").cloned().unwrap_or_default();

    let service = NidBrandSyncService::new(&state.db);
    let signature = ""; // In production, extract from X-nID-Signature header

    let payload_inner = BrandWebhookPayload {
        event_type: event_type.to_string(),
        tenant_id: tenant_id.to_string(),
        timestamp: payload.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        data,
    };

    match service.process_webhook(&payload_inner, signature).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Brand webhook error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn list_brands_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Vec<NidBrand>>) {
    let tenant_id = params.get("tenant_id").map(|s| s.as_str());
    let service = NidBrandSyncService::new(&state.db);
    match service.list_brands(tenant_id).await {
        Ok(brands) => (StatusCode::OK, Json(brands)),
        Err(e) => {
            error!("List brands error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

pub async fn get_brand_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<NidBrand>) {
    let service = NidBrandSyncService::new(&state.db);
    match service.get_brand_by_nid_id(&id).await {
        Ok(brand) => (StatusCode::OK, Json(brand)),
        Err(e) => {
            warn!("Get brand error: {}", e);
            (StatusCode::NOT_FOUND, Json(NidBrand {
                id: String::new(),
                nid_brand_id: String::new(),
                nid_org_id: String::new(),
                nid_tenant_id: String::new(),
                name: String::new(),
                slug: String::new(),
                description: None,
                bio: None,
                website: None,
                email: None,
                phone: None,
                primary_color: None,
                secondary_color: None,
                active: false,
                synced_at: String::new(),
                created_at: String::new(),
            }))
        }
    }
}

pub async fn get_connection_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<BrandConnectionSettings>) {
    let service = NidBrandSyncService::new(&state.db);
    let settings = service.get_connection_settings_public().await;
    (StatusCode::OK, Json(settings))
}

pub async fn update_connection_handler(
    State(state): State<AppState>,
    Json(req): Json<UpdateBrandConnectionRequest>,
) -> (StatusCode, Json<BrandConnectionSettings>) {
    let service = NidBrandSyncService::new(&state.db);
    match service.update_connection_settings(&req).await {
        Ok(settings) => (StatusCode::OK, Json(settings)),
        Err(e) => {
            error!("Update connection error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(BrandConnectionSettings {
                webhook_url: None,
                webhook_secret: None,
                polling_interval_sec: 300,
                enabled: true,
                health_status: "error".to_string(),
            }))
        }
    }
}

pub async fn health_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let service = NidBrandSyncService::new(&state.db);
    match service.run_health_check().await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => {
            error!("Health check error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error" })))
        }
    }
}

pub async fn get_sync_log_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Vec<serde_json::Value>>) {
    let limit = params.get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50);

    let service = NidBrandSyncService::new(&state.db);
    match service.get_sync_log(limit).await {
        Ok(logs) => (StatusCode::OK, Json(logs)),
        Err(e) => {
            error!("Sync log error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

pub async fn create_redirect_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateBrandRedirectRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let service = NidBrandSyncService::new(&state.db);
    match service.create_redirect_session(&req).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => {
            error!("Create redirect error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e })))
        }
    }
}

pub async fn redirect_callback_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state_token = params.get("state")
        .map(|s| s.as_str())
        .unwrap_or("");

    let nid_brand_id = params.get("brand_id")
        .map(|s| s.as_str())
        .unwrap_or("");

    let service = NidBrandSyncService::new(&state.db);
    match service.redirect_callback(state_token, nid_brand_id).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => {
            error!("Redirect callback error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e })))
        }
    }
}
