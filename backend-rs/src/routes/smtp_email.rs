//! SMTP and email template management routes.

use axum::{Json, Path, Query, State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::AppState;
use crate::services::smtp_email::*;

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TemplatesQuery {
    pub category: Option<String>,
    pub brand_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendLogsQuery {
    pub limit: Option<i64>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn err(status: StatusCode, msg: String) -> impl IntoResponse {
    (status, Json(json!({ "error": msg })))
}

fn db_err(e: String) -> impl IntoResponse {
    err(StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}"))
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(State(state): State<AppState>) -> impl IntoResponse {
    match get_smtp_summary(&state.db).await {
        Ok(summary) => Json(json!({
            "total_smtp_configs": summary.total_smtp_configs,
            "enabled_smtp_configs": summary.enabled_smtp_configs,
            "total_templates": summary.total_templates,
            "enabled_templates": summary.enabled_templates,
            "default_templates": summary.default_templates,
            "total_send_logs": summary.total_send_logs,
            "sent_count": summary.sent_count,
            "failed_count": summary.failed_count,
            "bounced_count": summary.bounced_count,
            "categories": summary.categories,
            "recent_logs": summary.recent_logs,
        })).into_response(),
        Err(e) => {
            tracing::error!("Failed to get SMTP summary: {}", e);
            Json(json!({
                "total_smtp_configs": 0i64,
                "enabled_smtp_configs": 0i64,
                "total_templates": 0i64,
                "enabled_templates": 0i64,
                "default_templates": 0i64,
                "total_send_logs": 0i64,
                "sent_count": 0i64,
                "failed_count": 0i64,
                "bounced_count": 0i64,
                "categories": [],
                "recent_logs": [],
            })).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// SMTP Settings
// ---------------------------------------------------------------------------

pub async fn list_smtp_settings(State(state): State<AppState>) -> impl IntoResponse {
    match SmtpSettings::list_all(&state.db).await {
        Ok(settings) => {
            let list: Vec<Value> = settings.iter().map(|s| {
                json!({
                    "id": s.id,
                    "brand_id": s.brand_id,
                    "host": s.host,
                    "port": s.port,
                    "security": s.security,
                    "auth_method": s.auth_method,
                    "username": s.username,
                    "from_address": s.from_address,
                    "from_name": s.from_name,
                    "reply_to_address": s.reply_to_address,
                    "connection_timeout_seconds": s.connection_timeout_seconds,
                    "enabled": s.enabled,
                    "test_sent_at": s.test_sent_at,
                    "test_status": s.test_status,
                    "test_error": s.test_error,
                    "created_at": s.created_at,
                    "updated_at": s.updated_at,
                })
            }).collect();
            Json(json!({ "smtp_settings": list })).into_response()
        }
        Err(e) => db_err(e),
    }
}

pub async fn get_smtp_settings(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match SmtpSettings::list_all(&state.db).await {
        Ok(settings) => {
            match settings.into_iter().find(|s| s.id == id) {
                Some(s) => {
                    Json(json!({
                        "id": s.id,
                        "brand_id": s.brand_id,
                        "host": s.host,
                        "port": s.port,
                        "security": s.security,
                        "auth_method": s.auth_method,
                        "username": s.username,
                        "from_address": s.from_address,
                        "from_name": s.from_name,
                        "reply_to_address": s.reply_to_address,
                        "connection_timeout_seconds": s.connection_timeout_seconds,
                        "enabled": s.enabled,
                        "test_sent_at": s.test_sent_at,
                        "test_status": s.test_status,
                        "test_error": s.test_error,
                        "created_at": s.created_at,
                        "updated_at": s.updated_at,
                    })).into_response()
                }
                None => err(StatusCode::NOT_FOUND, "SMTP settings not found".to_string()),
            }
        }
        Err(e) => db_err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSmtpPayload {
    pub brand_id: Option<String>,
    pub host: String,
    pub port: i64,
    pub security: String,
    pub auth_method: String,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
    pub reply_to_address: Option<String>,
    pub connection_timeout_seconds: Option<i64>,
}

pub async fn create_smtp_settings(
    State(state): State<AppState>,
    Json(payload): Json<CreateSmtpPayload>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let settings = SmtpSettings {
        id,
        brand_id: payload.brand_id,
        host: payload.host,
        port: payload.port,
        security: payload.security,
        auth_method: payload.auth_method,
        username: payload.username,
        password: payload.password,
        from_address: payload.from_address,
        from_name: payload.from_name,
        reply_to_address: payload.reply_to_address,
        connection_timeout_seconds: payload.connection_timeout_seconds.unwrap_or(30),
        enabled: true,
        test_sent_at: None,
        test_status: None,
        test_error: None,
        created_at: now.clone(),
        updated_at: now,
    };
    match SmtpSettings::create(&state.db, &settings).await {
        Ok(s) => (StatusCode::CREATED, Json(json!({ "id": s.id }))).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSmtpPayload {
    pub host: Option<String>,
    pub port: Option<i64>,
    pub security: Option<String>,
    pub auth_method: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub reply_to_address: Option<String>,
    pub connection_timeout_seconds: Option<i64>,
    pub enabled: Option<bool>,
}

pub async fn update_smtp_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSmtpPayload>,
) -> impl IntoResponse {
    let existing = match SmtpSettings::list_all(&state.db).await {
        Ok(settings) => settings.into_iter().find(|s| s.id == id),
        Err(e) => return db_err(e),
    };
    let mut settings = match existing {
        Some(s) => s,
        None => return err(StatusCode::NOT_FOUND, "SMTP settings not found".to_string()),
    };
    if let Some(v) = payload.host { settings.host = v; }
    if let Some(v) = payload.port { settings.port = v; }
    if let Some(v) = payload.security { settings.security = v; }
    if let Some(v) = payload.auth_method { settings.auth_method = v; }
    if let Some(v) = payload.username { settings.username = v; }
    if let Some(v) = payload.password { settings.password = v; }
    if let Some(v) = payload.from_address { settings.from_address = v; }
    if let Some(v) = payload.from_name { settings.from_name = v; }
    if let Some(v) = payload.reply_to_address { settings.reply_to_address = v; }
    if let Some(v) = payload.connection_timeout_seconds { settings.connection_timeout_seconds = v; }
    if let Some(v) = payload.enabled { settings.enabled = v; }
    settings.updated_at = chrono::Utc::now().to_rfc3339();
    match SmtpSettings::update(&state.db, &id, &settings).await {
        Ok(_) => Json(json!({ "updated": true })).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

pub async fn delete_smtp_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match SmtpSettings::delete(&state.db, &id).await {
        Ok(()) => Json(json!({ "deleted": true })).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct TestSmtpPayload {
    pub to_address: String,
}

pub async fn test_smtp_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<TestSmtpPayload>,
) -> impl IntoResponse {
    match SmtpSettings::test_connection(&state.db, &id, &payload.to_address).await {
        Ok(response) => Json(json!({
            "success": response.success,
            "message": response.message,
            "error": response.error,
        })).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

// ---------------------------------------------------------------------------
// Email Templates
// ---------------------------------------------------------------------------

pub async fn list_email_templates(
    State(state): State<AppState>,
    Query(params): Query<TemplatesQuery>,
) -> impl IntoResponse {
    match EmailTemplate::list(&state.db, params.category.as_deref(), params.brand_id.as_deref()).await {
        Ok(templates) => {
            let list: Vec<Value> = templates.iter().map(|t| {
                json!({
                    "id": t.id,
                    "brand_id": t.brand_id,
                    "name": t.name,
                    "subject": t.subject,
                    "body_html": t.body_html,
                    "body_text": t.body_text,
                    "template_type": t.template_type,
                    "category": t.category,
                    "language": t.language,
                    "enabled": t.enabled,
                    "is_default": t.is_default,
                    "variables": t.variables,
                    "created_at": t.created_at,
                    "updated_at": t.updated_at,
                })
            }).collect();
            Json(json!({ "templates": list })).into_response()
        }
        Err(e) => db_err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplatePayload {
    pub brand_id: Option<String>,
    pub name: String,
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
    pub template_type: String,
    pub category: String,
    pub language: Option<String>,
    pub variables: Option<Value>,
}

pub async fn create_email_template(
    State(state): State<AppState>,
    Json(payload): Json<CreateTemplatePayload>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let template = EmailTemplate {
        id,
        brand_id: payload.brand_id,
        name: payload.name,
        subject: payload.subject,
        body_html: payload.body_html,
        body_text: payload.body_text,
        template_type: payload.template_type,
        category: payload.category,
        language: payload.language.unwrap_or_else(|| "en".to_string()),
        enabled: true,
        is_default: false,
        variables: payload.variables.unwrap_or_else(|| json!({})),
        created_at: now.clone(),
        updated_at: now,
    };
    match EmailTemplate::create(&state.db, &template).await {
        Ok(t) => (StatusCode::CREATED, Json(json!({ "id": t.id }))).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

pub async fn get_email_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match EmailTemplate::list(&state.db, None, None).await {
        Ok(templates) => {
            match templates.into_iter().find(|t| t.id == id) {
                Some(t) => Json(json!({
                    "id": t.id,
                    "brand_id": t.brand_id,
                    "name": t.name,
                    "subject": t.subject,
                    "body_html": t.body_html,
                    "body_text": t.body_text,
                    "template_type": t.template_type,
                    "category": t.category,
                    "language": t.language,
                    "enabled": t.enabled,
                    "is_default": t.is_default,
                    "variables": t.variables,
                    "created_at": t.created_at,
                    "updated_at": t.updated_at,
                })).into_response(),
                None => err(StatusCode::NOT_FOUND, "Template not found".to_string()),
            }
        }
        Err(e) => db_err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplatePayload {
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub template_type: Option<String>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub enabled: Option<bool>,
    pub variables: Option<Value>,
}

pub async fn update_email_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTemplatePayload>,
) -> impl IntoResponse {
    let existing = match EmailTemplate::list(&state.db, None, None).await {
        Ok(templates) => templates.into_iter().find(|t| t.id == id),
        Err(e) => return db_err(e),
    };
    let mut template = match existing {
        Some(t) => t,
        None => return err(StatusCode::NOT_FOUND, "Template not found".to_string()),
    };
    if let Some(v) = payload.subject { template.subject = v; }
    if let Some(v) = payload.body_html { template.body_html = v; }
    if let Some(v) = payload.body_text { template.body_text = v; }
    if let Some(v) = payload.template_type { template.template_type = v; }
    if let Some(v) = payload.category { template.category = v; }
    if let Some(v) = payload.language { template.language = v; }
    if let Some(v) = payload.enabled { template.enabled = v; }
    if let Some(v) = payload.variables { template.variables = v; }
    template.updated_at = chrono::Utc::now().to_rfc3339();
    match EmailTemplate::update(&state.db, &id, &template).await {
        Ok(_) => Json(json!({ "updated": true })).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

pub async fn delete_email_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match EmailTemplate::delete(&state.db, &id).await {
        Ok(()) => Json(json!({ "deleted": true })).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

// ---------------------------------------------------------------------------
// Send Email
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SendEmailPayload {
    pub to_address: String,
    pub template_name: String,
    pub variables: Option<std::collections::HashMap<String, String>>,
    pub brand_id: Option<String>,
}

pub async fn send_email(
    State(state): State<AppState>,
    Json(payload): Json<SendEmailPayload>,
) -> impl IntoResponse {
    let request = SendEmailRequest {
        to_address: payload.to_address,
        template_name: payload.template_name,
        variables: payload.variables.unwrap_or_default(),
        brand_id: payload.brand_id,
    };
    match send_templated_email(&state.db, &request).await {
        Ok(response) => Json(json!({
            "success": response.success,
            "log_id": response.log_id,
            "message_id": response.message_id,
            "error": response.error,
        })).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

// ---------------------------------------------------------------------------
// Send Logs
// ---------------------------------------------------------------------------

pub async fn get_send_logs(
    State(state): State<AppState>,
    Query(params): Query<SendLogsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    match EmailSendLog::recent(&state.db, limit).await {
        Ok(logs) => {
            let list: Vec<Value> = logs.iter().map(|l| {
                json!({
                    "id": l.id,
                    "brand_id": l.brand_id,
                    "template_id": l.template_id,
                    "template_name": l.template_name,
                    "to_address": l.to_address,
                    "from_address": l.from_address,
                    "subject": l.subject,
                    "status": l.status,
                    "error_message": l.error_message,
                    "smtp_response": l.smtp_response,
                    "message_id": l.message_id,
                    "sent_at": l.sent_at,
                    "created_at": l.created_at,
                })
            }).collect();
            Json(json!({ "logs": list })).into_response()
        }
        Err(e) => db_err(e),
    }
}
