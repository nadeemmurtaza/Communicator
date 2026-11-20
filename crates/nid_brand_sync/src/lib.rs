//! nID Brand Sync — business logic extracted from backend-rs monolith.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

// ─── Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandConnectionSettings {
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub polling_interval_sec: i32,
    pub enabled: bool,
    pub health_status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnectionRequestDto {
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub polling_interval_sec: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRedirectRequestDto {
    pub org_id: Option<String>,
    pub return_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandWebhookPayload {
    pub event_type: String,
    pub tenant_id: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

// ─── Database row types ───

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

// ─── Service ───

pub struct NidBrandSyncService {
    pool: PgPool,
}

impl NidBrandSyncService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    // -----------------------------------------------------------------------
    // Webhook processing
    // -----------------------------------------------------------------------

    pub async fn process_webhook(
        &self,
        payload: &BrandWebhookPayload,
        signature: &str,
    ) -> Result<NidBrand, String> {
        // Verify webhook signature
        let settings = self.get_connection_settings().await;
        if let Some(ref secret) = settings.webhook_secret {
            if !secrets_match(signature, Some(secret.as_str())) {
                return Err("Invalid webhook signature".to_string());
            }
        }

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

        let existing = sqlx::query_scalar::<_, String>(
            "SELECT nid_brand_id FROM nid_brands WHERE nid_brand_id = $1",
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
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW())",
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
             name = COALESCE($1, name), \
             slug = COALESCE($2, slug), \
             description = $3, bio = $4, website = $5, email = $6, phone = $7, \
             primary_color = $8, secondary_color = $9, active = $10, synced_at = NOW() \
             WHERE nid_brand_id = $11",
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

        sqlx::query("UPDATE nid_brands SET active = false, synced_at = NOW() WHERE nid_brand_id = $1")
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
             active, synced_at, created_at FROM nid_brands WHERE id = $1",
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
             active, synced_at, created_at FROM nid_brands WHERE nid_brand_id = $1",
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
                 active, synced_at, created_at FROM nid_brands WHERE nid_tenant_id = $1 ORDER BY created_at DESC",
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

    pub async fn create_redirect_session(
        &self,
        req: &CreateRedirectRequestDto,
    ) -> Result<serde_json::Value, String> {
        let state_token = Uuid::new_v4().to_string();
        let id = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now()
            .checked_add_days(chrono::Days::new(1))
            .unwrap()
            .to_rfc3339();

        let settings = self.get_connection_settings().await;
        if !settings.enabled {
            return Err("nID brand connection is disabled".to_string());
        }

        sqlx::query(
            "INSERT INTO brand_redirect_sessions (id, state_token, nid_tenant_id, nid_org_id, \
             redirect_uri, max_attempts, attempts, expires_at) \
             VALUES ($1, $2, 'default', $3, $4, 1, 0, $5)",
        )
        .bind(&id)
        .bind(&state_token)
        .bind(req.org_id.as_deref())
        .bind(&req.return_url)
        .bind(&expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

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

    pub async fn redirect_callback(
        &self,
        state_token: &str,
        nid_brand_id: &str,
    ) -> Result<serde_json::Value, String> {
        let session = sqlx::query(
            "SELECT id, nid_org_id, redirect_uri, attempts, max_attempts, expires_at \
             FROM brand_redirect_sessions WHERE state_token = $1",
        )
        .bind(state_token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Redirect session not found".to_string())?;

        let expires_at: chrono::DateTime<chrono::Utc> = session
            .try_get("expires_at")
            .map_err(|_| "Invalid expiry date".to_string())?;

        if chrono::Utc::now() > expires_at {
            sqlx::query("UPDATE brand_redirect_sessions SET error_code = 'EXPIRED' WHERE state_token = $1")
                .bind(state_token)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            return Err("Redirect session has expired".to_string());
        }

        sqlx::query("UPDATE brand_redirect_sessions SET attempts = attempts + 1 WHERE state_token = $1")
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
        req: &UpdateConnectionRequestDto,
    ) -> Result<BrandConnectionSettings, String> {
        sqlx::query(
            "INSERT INTO nid_brand_connection (id, webhook_url, webhook_secret, polling_interval_sec, enabled, updated_at) \
             VALUES (1, $1, $2, $3, $4, NOW()) \
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
            "UPDATE nid_brand_connection SET health_status = $1, last_health_check = NOW(), updated_at = NOW()",
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
             VALUES ($1, $2, $3, $4)",
        )
        .bind(nid_brand_id)
        .bind(event_type)
        .bind(status)
        .bind(error_msg.as_deref())
        .execute(&self.pool)
        .await
        .ok();
    }

    pub async fn get_sync_log(&self, limit: i64) -> Result<Vec<SyncLogEntry>, String> {
        let rows = sqlx::query(
            "SELECT id, nid_brand_id, event_type, status, error_msg, created_at \
             FROM brand_sync_log ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let logs: Vec<SyncLogEntry> = rows
            .into_iter()
            .map(|r| {
                SyncLogEntry {
                    id: r.try_get::<i64, _>("id").unwrap_or(0).to_string(),
                    nid_brand_id: r.try_get::<String, _>("nid_brand_id").unwrap_or_default(),
                    event_type: r.try_get::<String, _>("event_type").unwrap_or_default(),
                    status: r.try_get::<String, _>("status").unwrap_or_default(),
                    error_msg: r.try_get::<Option<String>, _>("error_msg").unwrap_or(None),
                    created_at: r.try_get::<String, _>("created_at").unwrap_or_default(),
                }
            })
            .collect();

        Ok(logs)
    }
}

// ─── Sync log entry for proto ───

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncLogEntry {
    pub id: String,
    pub nid_brand_id: String,
    pub event_type: String,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: String,
}

// ─── Signature verification (extracted from signatures module) ───

fn secrets_match(input: &str, expected: Option<&str>) -> bool {
    match expected {
        Some(exp) => {
            if exp.is_empty() || input.is_empty() {
                return true;
            }
            input == exp
        }
        None => true,
    }
}
