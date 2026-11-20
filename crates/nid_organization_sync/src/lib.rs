use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use rand::Rng;

// ─── Models ───

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NidOrganization {
    pub id: String,
    pub nid_organization_id: String,
    pub nid_tenant_id: String,
    pub tenant_slug: String,
    pub legal_name: String,
    pub registration_number: Option<String>,
    pub country_code: Option<String>,
    pub subject_id: String,
    pub is_verified: bool,
    pub verified_level: Option<String>,
    pub webhook_url: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub last_sync_error: String,
    pub linked_brand_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NidOrganizationConnection {
    pub id: String,
    pub nID_base_url: String,
    pub webhook_secret: String,
    pub auto_sync_enabled: bool,
    pub auto_link_new_organizations: bool,
    pub sync_interval_seconds: i64,
    pub connection_status: String,
    pub last_error: String,
    pub last_health_check_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSyncLog {
    pub id: String,
    pub nid_organization_id: String,
    pub event_type: String,
    pub status: String,
    pub error_message: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionHealth {
    pub connection_status: String,
    pub nID_base_url: String,
    pub auto_sync_enabled: bool,
    pub total_organizations: i64,
    pub synced_organizations: i64,
    pub error_organizations: i64,
    pub last_error: String,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnectionRequestDto {
    pub nID_base_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub auto_sync_enabled: Option<bool>,
    pub auto_link_new_organizations: Option<bool>,
    pub sync_interval_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRedirectRequestDto {
    pub redirect_type: Option<String>,
    pub return_url: String,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedirectSession {
    pub state_token: String,
    pub redirect_url: String,
    pub expires_at: String,
}

// ─── Service functions ───

async fn get_connection(pool: &PgPool) -> Option<NidOrganizationConnection> {
    sqlx::query_as!(
        NidOrganizationConnection,
        r#"SELECT id::TEXT as "id!", nid_base_url as "nID_base_url", webhook_secret, auto_sync_enabled,
                  auto_link_new_organizations, sync_interval_seconds, connection_status,
                  last_error, last_health_check_at::TEXT as last_health_check_at
           FROM nid_organization_connection LIMIT 1"#
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn upsert_organization_from_webhook(
    pool: &PgPool,
    org_data: &serde_json::Value,
    tenant_data: &serde_json::Value,
    event_type: &str,
) -> Result<NidOrganization, String> {
    let nid_org_id = org_data.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing organization id")?;

    let legal_name = org_data.get("legalName")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let nid_tenant_id = tenant_data.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing tenant id")?;

    let tenant_slug = tenant_data.get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let registration_number = org_data.get("registrationNumber")
        .and_then(|v| v.as_str())
        .map(String::from);

    let country_code = org_data.get("countryCode")
        .and_then(|v| v.as_str())
        .map(String::from);

    let subject_id = org_data.get("subjectId")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let is_verified = org_data.get("isVerified")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let verified_level = org_data.get("verifiedLevel")
        .and_then(|v| v.as_str())
        .map(String::from);

    let payload_json = serde_json::to_value(org_data).unwrap_or_default();
    let log_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO organization_sync_log (id, nid_organization_id, event_type, payload, status, error_message)
           VALUES ($1, $2, $3, $4, 'success', '')"#,
        log_id,
        nid_org_id,
        event_type,
        payload_json,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to log sync: {}", e))?;

    let org_id = Uuid::new_v4();
    let now = Utc::now();

    let existing = sqlx::query!(
        r#"SELECT id::TEXT, sync_status FROM nid_organizations WHERE nid_organization_id = $1"#,
        nid_org_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Query failed: {}", e))?;

    if let Some(_existing) = existing {
        sqlx::query!(
            r#"UPDATE nid_organizations SET
                   legal_name = $1,
                   registration_number = $2,
                   country_code = $3,
                   subject_id = $4,
                   is_verified = $5,
                   verified_level = $6,
                   sync_status = 'synced',
                   last_synced_at = $7,
                   last_sync_error = '',
                   updated_at = $8
               WHERE nid_organization_id = $9"#,
            legal_name,
            registration_number.as_deref(),
            country_code.as_deref(),
            subject_id,
            is_verified,
            verified_level.as_deref(),
            now,
            now,
            nid_org_id
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Update failed: {}", e))?;
    } else {
        sqlx::query!(
            r#"INSERT INTO nid_organizations (id, nid_organization_id, nid_tenant_id, tenant_slug,
                   legal_name, registration_number, country_code, subject_id, is_verified,
                   verified_level, sync_status, last_synced_at, last_sync_error, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'synced', $11, '', $12, $13)"#,
            org_id,
            nid_org_id,
            nid_tenant_id,
            tenant_slug,
            legal_name,
            registration_number.as_deref(),
            country_code.as_deref(),
            subject_id,
            is_verified,
            verified_level.as_deref(),
            now,
            now,
            now
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Insert failed: {}", e))?;
    }

    fetch_organization_by_nid_id(pool, nid_org_id).await
}

pub async fn fetch_organization_by_nid_id(pool: &PgPool, nid_org_id: &str) -> Result<NidOrganization, String> {
    let org = sqlx::query_as!(
        NidOrganization,
        r#"SELECT id::TEXT as "id!", nid_organization_id, nid_tenant_id, tenant_slug,
                  legal_name, registration_number, country_code, subject_id,
                  is_verified as "is_verified!", verified_level, webhook_url, sync_status,
                  last_synced_at::TEXT as last_synced_at, last_sync_error,
                  linked_brand_id::TEXT as linked_brand_id,
                  created_at::TEXT as "created_at!", updated_at::TEXT as "updated_at!"
           FROM nid_organizations WHERE nid_organization_id = $1"#,
        nid_org_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Organization not found: {}", e))?;

    Ok(org)
}

pub async fn list_organizations(pool: &PgPool) -> Result<Vec<NidOrganization>, String> {
    let orgs = sqlx::query_as!(
        NidOrganization,
        r#"SELECT id::TEXT as "id!", nid_organization_id, nid_tenant_id, tenant_slug,
                  legal_name, registration_number, country_code, subject_id,
                  is_verified as "is_verified!", verified_level, webhook_url, sync_status,
                  last_synced_at::TEXT as last_synced_at, last_sync_error,
                  linked_brand_id::TEXT as linked_brand_id,
                  created_at::TEXT as "created_at!", updated_at::TEXT as "updated_at!"
           FROM nid_organizations ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to list organizations: {}", e))?;

    Ok(orgs)
}

pub async fn get_sync_log(pool: &PgPool, limit: i64) -> Result<Vec<OrganizationSyncLog>, String> {
    let logs = sqlx::query_as!(
        OrganizationSyncLog,
        r#"SELECT id::TEXT as "id!", nid_organization_id, event_type, status,
                  error_message, created_at::TEXT as "created_at!"
           FROM organization_sync_log ORDER BY created_at DESC LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get sync log: {}", e))?;

    Ok(logs)
}

pub async fn get_health(pool: &PgPool) -> Result<ConnectionHealth, String> {
    let conn = get_connection(pool).await;
    let connection_status = conn.as_ref().map(|c| c.connection_status.clone()).unwrap_or_else(|| "disconnected".to_string());
    let nID_base_url = conn.as_ref().map(|c| c.nID_base_url.clone()).unwrap_or_default();
    let auto_sync_enabled = conn.as_ref().map(|c| c.auto_sync_enabled).unwrap_or(false);
    let last_error = conn.as_ref().map(|c| c.last_error.clone()).unwrap_or_default();

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM nid_organizations"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let synced: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM nid_organizations WHERE sync_status = 'synced'"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let errors: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM nid_organizations WHERE sync_status = 'error'"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ConnectionHealth {
        connection_status,
        nID_base_url,
        auto_sync_enabled,
        total_organizations: total as i64,
        synced_organizations: synced as i64,
        error_organizations: errors as i64,
        last_error,
    })
}

// ─── Redirect flow ───

fn generate_state_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = std::array::from_fn(|_| rng.gen());
    hex::encode(bytes)
}

pub async fn create_redirect_session(
    pool: &PgPool,
    req: CreateRedirectRequestDto,
) -> Result<RedirectSession, String> {
    let state_token = generate_state_token();
    let session_id = Uuid::new_v4();
    let expires_at_dt = chrono::Utc::now()
        .checked_add_days(chrono::Days::new(1))
        .unwrap();
    let expires_at = expires_at_dt.to_rfc3339();

    let redirect_type = req.redirect_type.unwrap_or_else(|| "create".to_string());

    sqlx::query!(
        r#"INSERT INTO organization_redirect_sessions (id, state_token, redirect_type, return_url,
                   nid_tenant_id, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        session_id,
        &state_token,
        &redirect_type,
        &req.return_url,
        req.tenant_id.as_deref(),
        expires_at_dt
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create redirect session: {}", e))?;

    let conn = get_connection(pool).await;
    let nid_base_url = conn.map(|c| c.nID_base_url).unwrap_or_else(|| "http://localhost:4000".to_string());

    let nid_url = format!(
        "{}/dashboard?redirect_state={}&return_url={}&redirect_type=organization",
        nid_base_url,
        state_token,
        urlencoding::encode(&req.return_url)
    );

    Ok(RedirectSession {
        state_token,
        redirect_url: nid_url,
        expires_at,
    })
}

pub async fn handle_redirect_callback(
    pool: &PgPool,
    state: &str,
    organization_id: Option<&str>,
    error: Option<&str>,
) -> Result<serde_json::Value, String> {
    let session = sqlx::query!(
        r#"SELECT id::TEXT as "id!", state_token, redirect_type, return_url, nid_tenant_id,
                  expires_at, max_attempts, attempt_count, status, error_code, error_message
           FROM organization_redirect_sessions WHERE state_token = $1"#,
        state
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Invalid state token".to_string())?;

    let now = chrono::Utc::now();
    let expires_at: chrono::DateTime<chrono::Utc> = session.expires_at;

    if now > expires_at {
        sqlx::query!(
            r#"UPDATE organization_redirect_sessions SET status = 'expired',
                   error_code = 'EXPIRED', error_message = 'Session expired'
               WHERE state_token = $1"#,
            state
        )
        .execute(pool)
        .await
        .ok();

        return Err("Redirect session has expired".to_string());
    }

    if session.attempt_count >= session.max_attempts {
        sqlx::query!(
            r#"UPDATE organization_redirect_sessions SET status = 'max_attempts_reached',
                   error_code = 'MAX_ATTEMPTS', error_message = 'Maximum redirect attempts reached'
               WHERE state_token = $1"#,
            state
        )
        .execute(pool)
        .await
        .ok();

        return Err("Maximum redirect attempts reached".to_string());
    }

    if let Some(err) = error {
        sqlx::query!(
            r#"UPDATE organization_redirect_sessions SET status = 'error',
                   error_code = 'CALLBACK_ERROR', error_message = $1,
                   attempt_count = attempt_count + 1, completed_at = $2
               WHERE state_token = $3"#,
            err,
            now,
            state
        )
        .execute(pool)
        .await
        .ok();

        return Ok(serde_json::json!({
            "success": false,
            "error": err,
            "return_url": session.return_url,
        }));
    }

    sqlx::query!(
        r#"UPDATE organization_redirect_sessions SET status = 'completed',
               attempt_count = attempt_count + 1, completed_at = $1
           WHERE state_token = $2"#,
        now,
        state
    )
    .execute(pool)
    .await
    .ok();

    Ok(serde_json::json!({
        "success": true,
        "organization_id": organization_id,
        "return_url": session.return_url,
    }))
}

pub async fn handle_webhook(
    pool: &PgPool,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let event_type = payload.get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let org_data = payload.get("organization")
        .cloned()
        .unwrap_or_default();

    let tenant_data = payload.get("tenant")
        .cloned()
        .unwrap_or_default();

    upsert_organization_from_webhook(pool, &org_data, &tenant_data, event_type).await?;
    Ok(())
}

pub async fn handle_update_connection(
    pool: &PgPool,
    req: UpdateConnectionRequestDto,
) -> Result<(), String> {
    let mut query_parts = vec![];

    if let Some(url) = &req.nID_base_url {
        query_parts.push(format!("nid_base_url = '{}'", url.replace('\'', "''")));
    }
    if let Some(secret) = &req.webhook_secret {
        query_parts.push(format!("webhook_secret = '{}'", secret.replace('\'', "''")));
    }
    if let Some(enabled) = req.auto_sync_enabled {
        query_parts.push(format!("auto_sync_enabled = {}", enabled));
    }
    if let Some(link) = req.auto_link_new_organizations {
        query_parts.push(format!("auto_link_new_organizations = {}", link));
    }
    if let Some(interval) = req.sync_interval_seconds {
        query_parts.push(format!("sync_interval_seconds = {}", interval));
    }

    if query_parts.is_empty() {
        return Ok(());
    }

    query_parts.push("connection_status = 'connected'".to_string());
    query_parts.push("updated_at = NOW()".to_string());

    let query = format!(
        "UPDATE nid_organization_connection SET {} WHERE id = '00000000-0000-0000-0000-000000000001'",
        query_parts.join(", ")
    );

    sqlx::query(&query).execute(pool).await
        .map_err(|e| format!("Failed to update connection: {}", e))?;
    Ok(())
}
