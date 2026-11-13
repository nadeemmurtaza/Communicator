use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::AppState;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize)]
pub struct OrganizationSyncLog {
    pub id: String,
    pub nid_organization_id: String,
    pub event_type: String,
    pub status: String,
    pub error_message: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConnectionHealth {
    pub connection_status: String,
    pub nID_base_url: String,
    pub auto_sync_enabled: bool,
    pub total_organizations: i64,
    pub synced_organizations: i64,
    pub error_organizations: i64,
    pub last_error: String,
}

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub organization: serde_json::Value,
    pub tenant: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateConnectionRequest {
    pub nID_base_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub auto_sync_enabled: Option<bool>,
    pub auto_link_new_organizations: Option<bool>,
    pub sync_interval_seconds: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateRedirectRequest {
    pub redirect_type: Option<String>,
    pub return_url: String,
    pub tenant_id: Option<String>,
}

#[derive(Serialize)]
pub struct RedirectSession {
    pub state_token: String,
    pub redirect_url: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
pub struct RedirectCallbackQuery {
    pub state: String,
    pub organization_id: Option<String>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

async fn get_connection(pool: &PgPool) -> Option<NidOrganizationConnection> {
    sqlx::query_as!(
        NidOrganizationConnection,
        r#"SELECT id, "nID_base_url" as nID_base_url, webhook_secret, auto_sync_enabled,
                  auto_link_new_organizations, sync_interval_seconds, connection_status,
                  last_error, "last_health_check_at" as last_health_check_at
           FROM nid_organization_connection LIMIT 1"#
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

async fn upsert_organization_from_webhook(
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

    // Check KYB verification status
    let is_verified = org_data.get("isVerified")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let verified_level = org_data.get("verifiedLevel")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Log the sync event
    let payload_json = serde_json::to_value(org_data).unwrap_or_default();
    let log_id = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO organization_sync_log (id, nid_organization_id, event_type, payload, status, error_message)
           VALUES ($1, $2, $3, $4, 'success', '')"#,
        log_id.as_str(),
        nid_org_id,
        event_type,
        payload_json.to_string() as &str,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to log sync: {}", e))?;

    // Upsert the organization
    let org_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // First try to find existing
    let existing = sqlx::query!(
        r#"SELECT id::TEXT, sync_status FROM nid_organizations WHERE nid_organization_id = $1"#,
        nid_org_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Query failed: {}", e))?;

    if let Some(_existing) = existing {
        // Update existing
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
            &now,
            &now,
            nid_org_id
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Update failed: {}", e))?;
    } else {
        // Insert new
        sqlx::query!(
            r#"INSERT INTO nid_organizations (id, nid_organization_id, nid_tenant_id, tenant_slug,
                   legal_name, registration_number, country_code, subject_id, is_verified,
                   verified_level, sync_status, last_synced_at, last_sync_error, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'synced', $11, '', $12, $13)"#,
            org_id.as_str(),
            nid_org_id,
            nid_tenant_id,
            tenant_slug,
            legal_name,
            registration_number.as_deref(),
            country_code.as_deref(),
            subject_id,
            is_verified,
            verified_level.as_deref(),
            &now,
            &now,
            &now
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Insert failed: {}", e))?;
    }

    // Fetch and return
    fetch_organization_by_nid_id(pool, nid_org_id).await
}

async fn fetch_organization_by_nid_id(pool: &PgPool, nid_org_id: &str) -> Result<NidOrganization, String> {
    let org = sqlx::query_as!(
        NidOrganization,
        r#"SELECT id::TEXT as id, nid_organization_id, nid_tenant_id, tenant_slug,
                  legal_name, registration_number, country_code, subject_id,
                  is_verified, verified_level, webhook_url, sync_status,
                  "last_synced_at"::TEXT as last_synced_at, last_sync_error,
                  linked_brand_id::TEXT as linked_brand_id,
                  created_at::TEXT as created_at, updated_at::TEXT as updated_at
           FROM nid_organizations WHERE nid_organization_id = $1"#,
        nid_org_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Organization not found: {}", e))?;

    Ok(org)
}

async fn list_organizations(pool: &PgPool) -> Result<Vec<NidOrganization>, String> {
    let orgs = sqlx::query_as!(
        NidOrganization,
        r#"SELECT id::TEXT as id, nid_organization_id, nid_tenant_id, tenant_slug,
                  legal_name, registration_number, country_code, subject_id,
                  is_verified, verified_level, webhook_url, sync_status,
                  "last_synced_at"::TEXT as last_synced_at, last_sync_error,
                  linked_brand_id::TEXT as linked_brand_id,
                  created_at::TEXT as created_at, updated_at::TEXT as updated_at
           FROM nid_organizations ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to list organizations: {}", e))?;

    Ok(orgs)
}

async fn get_sync_log(pool: &PgPool, limit: i64) -> Result<Vec<OrganizationSyncLog>, String> {
    let logs = sqlx::query_as!(
        OrganizationSyncLog,
        r#"SELECT id::TEXT as id, nid_organization_id, event_type, status,
                  error_message, created_at::TEXT as created_at
           FROM organization_sync_log ORDER BY created_at DESC LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get sync log: {}", e))?;

    Ok(logs)
}

async fn get_health(pool: &PgPool) -> Result<ConnectionHealth, String> {
    let conn = get_connection(pool).await;
    let connection_status = conn.as_ref().map(|c| c.connection_status.clone()).unwrap_or_else(|| "disconnected".to_string());
    let nID_base_url = conn.as_ref().map(|c| c.nID_base_url.clone()).unwrap_or_default();
    let auto_sync_enabled = conn.as_ref().map(|c| c.auto_sync_enabled).unwrap_or(false);
    let last_error = conn.as_ref().map(|c| c.last_error.clone()).unwrap_or_default();

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::INTEGER FROM nid_organizations"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let synced: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::INTEGER FROM nid_organizations WHERE sync_status = 'synced'"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let errors: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::INTEGER FROM nid_organizations WHERE sync_status = 'error'"#
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

// ---------------------------------------------------------------------------
// Redirect flow
// ---------------------------------------------------------------------------

fn generate_state_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = std::array::from_fn(|_| rng.gen());
    hex::encode(bytes)
}

pub async fn create_redirect_session(
    pool: &PgPool,
    req: CreateRedirectRequest,
) -> Result<RedirectSession, String> {
    let state_token = generate_state_token();
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_days(chrono::Days::new(1))
        .unwrap()
        .to_rfc3339();

    let redirect_type = req.redirect_type.unwrap_or_else(|| "create".to_string());

    sqlx::query!(
        r#"INSERT INTO organization_redirect_sessions (id, state_token, redirect_type, return_url,
                   nid_tenant_id, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        session_id.as_str(),
        &state_token,
        &redirect_type,
        &req.return_url,
        req.tenant_id.as_deref(),
        &expires_at
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create redirect session: {}", e))?;

    let conn = get_connection(pool).await;
    let nid_base_url = conn.map(|c| c.nID_base_url).unwrap_or_else(|| "http://localhost:4000".to_string());

    // Build the redirect URL to nID dashboard with organization creation context
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
        r#"SELECT id::TEXT, state_token, redirect_type, return_url, nid_tenant_id,
                  expires_at::TEXT, max_attempts, attempt_count, status, error_code, error_message
           FROM organization_redirect_sessions WHERE state_token = $1"#,
        state
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Invalid state token".to_string())?;

    // Check expiry
    let now = chrono::Utc::now();
    let expires_at: chrono::DateTime<chrono::Utc> = session.expires_at.parse()
        .map_err(|_| "Invalid expiry date".to_string())?;

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

    // Check max attempts
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

    // Handle error case
    if let Some(err) = error {
        sqlx::query!(
            r#"UPDATE organization_redirect_sessions SET status = 'error',
                   error_code = 'CALLBACK_ERROR', error_message = $1,
                   attempt_count = attempt_count + 1, completed_at = $2
               WHERE state_token = $3"#,
            err,
            now.to_rfc3339(),
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

    // Success - mark session as completed
    sqlx::query!(
        r#"UPDATE organization_redirect_sessions SET status = 'completed',
               attempt_count = attempt_count + 1, completed_at = $1
           WHERE state_token = $2"#,
        now.to_rfc3339(),
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

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

pub async fn webhook_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> StatusCode {
    let event_type = payload.get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let org_data = payload.get("organization")
        .cloned()
        .unwrap_or_default();

    let tenant_data = payload.get("tenant")
        .cloned()
        .unwrap_or_default();

    match upsert_organization_from_webhook(&state.db, &org_data, &tenant_data, event_type).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Organization webhook error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn list_organizations_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<NidOrganization>>) {
    match list_organizations(&state.db).await {
        Ok(orgs) => (StatusCode::OK, Json(orgs)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn get_organization_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<NidOrganization>) {
    match fetch_organization_by_nid_id(&state.db, &id).await {
        Ok(org) => (StatusCode::OK, Json(org)),
        Err(_) => (StatusCode::NOT_FOUND, Json(NidOrganization {
            id: String::new(),
            nid_organization_id: String::new(),
            nid_tenant_id: String::new(),
            tenant_slug: String::new(),
            legal_name: String::new(),
            registration_number: None,
            country_code: None,
            subject_id: String::new(),
            is_verified: false,
            verified_level: None,
            webhook_url: None,
            sync_status: String::new(),
            last_synced_at: None,
            last_sync_error: String::new(),
            linked_brand_id: None,
            created_at: String::new(),
            updated_at: String::new(),
        })),
    }
}

pub async fn get_connection_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<NidOrganizationConnection>>) {
    let conn = get_connection(&state.db).await;
    (StatusCode::OK, Json(conn))
}

pub async fn update_connection_handler(
    State(state): State<AppState>,
    Json(req): Json<UpdateConnectionRequest>,
) -> StatusCode {
    let mut query_parts = vec![];
    let mut params: Vec<&str> = vec![];
    let mut param_idx = 1;

    if let Some(url) = req.nID_base_url {
        query_parts.push(format!("\"nID_base_url\" = '${}' ", param_idx));
        params.push(&url);
        param_idx += 1;
    }
    if let Some(secret) = req.webhook_secret {
        query_parts.push(format!("webhook_secret = '${}' ", param_idx));
        params.push(&secret);
        param_idx += 1;
    }
    if let Some(enabled) = req.auto_sync_enabled {
        query_parts.push(format!("auto_sync_enabled = ${} ", param_idx));
        param_idx += 1;
    }
    if let Some(link) = req.auto_link_new_organizations {
        query_parts.push(format!("auto_link_new_organizations = ${} ", param_idx));
        param_idx += 1;
    }
    if let Some(interval) = req.sync_interval_seconds {
        query_parts.push(format!("sync_interval_seconds = ${} ", param_idx));
        param_idx += 1;
    }

    if query_parts.is_empty() {
        return StatusCode::OK;
    }

    query_parts.push(format!("connection_status = 'connected' "));
    query_parts.push(format!("updated_at = NOW() "));

    let query = format!(
        "UPDATE nid_organization_connection SET {} WHERE id = '00000000-0000-0000-0000-000000000001'",
        query_parts.join(", ")
    );

    match sqlx::query(&query).execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Failed to update connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_health_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<ConnectionHealth>) {
    match get_health(&state.db).await {
        Ok(health) => (StatusCode::OK, Json(health)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ConnectionHealth {
            connection_status: "error".to_string(),
            nID_base_url: String::new(),
            auto_sync_enabled: false,
            total_organizations: 0,
            synced_organizations: 0,
            error_organizations: 0,
            last_error: e,
        })),
    }
}

pub async fn get_sync_log_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Vec<OrganizationSyncLog>>) {
    let limit = params.get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50);

    match get_sync_log(&state.db, limit).await {
        Ok(logs) => (StatusCode::OK, Json(logs)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn create_redirect_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateRedirectRequest>,
) -> (StatusCode, Json<RedirectSession>) {
    match create_redirect_session(&state.db, req).await {
        Ok(session) => (StatusCode::OK, Json(session)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RedirectSession {
            state_token: String::new(),
            redirect_url: String::new(),
            expires_at: String::new(),
        })),
    }
}

pub async fn redirect_callback_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state_token = params.get("state")
        .map(|s| s.as_str())
        .unwrap_or("");

    let org_id = params.get("organization_id").map(|s| s.as_str());
    let error = params.get("error").map(|s| s.as_str());

    match handle_redirect_callback(&state.db, state_token, org_id, error).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": e,
        }))),
    }
}
