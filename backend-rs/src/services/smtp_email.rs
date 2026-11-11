//! SMTP email service for transactional emails.
//!
//! Provides SMTP configuration management, email template rendering with
//! variable substitution, and email sending with audit logging.
//! Supports STARTTLS, TLS, and plaintext connections.

use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::PgRow;
use std::collections::HashMap;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtpSettings {
    pub id: String,
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
    pub connection_timeout_seconds: i64,
    pub enabled: bool,
    pub test_sent_at: Option<String>,
    pub test_status: Option<String>,
    pub test_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub brand_id: Option<String>,
    pub name: String,
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
    pub template_type: String,
    pub category: String,
    pub language: String,
    pub enabled: bool,
    pub is_default: bool,
    pub variables: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailSendLog {
    pub id: String,
    pub brand_id: Option<String>,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub to_address: String,
    pub from_address: String,
    pub subject: String,
    pub status: String,
    pub error_message: Option<String>,
    pub smtp_response: Option<String>,
    pub message_id: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub to_address: String,
    pub template_name: String,
    pub variables: HashMap<String, String>,
    pub brand_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendEmailResponse {
    pub success: bool,
    pub log_id: String,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestSmtpResponse {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtpSummary {
    pub total_smtp_configs: i64,
    pub enabled_smtp_configs: i64,
    pub total_templates: i64,
    pub enabled_templates: i64,
    pub default_templates: i64,
    pub total_send_logs: i64,
    pub sent_count: i64,
    pub failed_count: i64,
    pub bounced_count: i64,
    pub categories: Vec<CategoryCount>,
    pub recent_logs: Vec<EmailSendLog>,
}

// ---------------------------------------------------------------------------
// ID generation
// ---------------------------------------------------------------------------

fn gen_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = std::array::from_fn(|_| rng.gen());
    let mut result = String::with_capacity(36);
    result.push_str(&format!("{:08x}", u32::from_be_bytes(bytes[0..4].try_into().unwrap())));
    result.push('-');
    result.push_str(&format!("{:04x}", u16::from_be_bytes(bytes[4..6].try_into().unwrap())));
    result.push('-');
    result.push_str(&format!("{:04x}", u16::from_be_bytes(bytes[6..8].try_into().unwrap())));
    result.push('-');
    result.push_str(&format!("{:04x}", u16::from_be_bytes(bytes[8..10].try_into().unwrap())));
    result.push('-');
    result.push_str(&format!("{:012x}", u128::from_be_bytes(bytes[10..].try_into().unwrap())));
    result
}

// ---------------------------------------------------------------------------
// Template rendering
// ---------------------------------------------------------------------------

/// Render a template string by replacing `{{variable}}` placeholders with values.
pub fn render_template(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

// ---------------------------------------------------------------------------
// SMTP Settings CRUD
// ---------------------------------------------------------------------------

impl SmtpSettings {
    /// Get SMTP settings, optionally filtered by brand_id. Falls back to global (brand_id IS NULL).
    pub async fn get(pool: &DbPool, brand_id: Option<&str>) -> Result<Option<Self>, String> {
        let query = if brand_id.is_some() {
            r#"SELECT * FROM smtp_settings WHERE brand_id = $1 AND enabled = true LIMIT 1
               UNION ALL
               SELECT * FROM smtp_settings WHERE brand_id IS NULL AND enabled = true LIMIT 1"#
        } else {
            "SELECT * FROM smtp_settings WHERE brand_id IS NULL AND enabled = true LIMIT 1"
        };

        let row = sqlx::query(query)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Failed to fetch SMTP settings: {}", e))?;

        Ok(row.map(Self::from_row))
    }

    /// Get all SMTP settings.
    pub async fn list_all(pool: &DbPool) -> Result<Vec<Self>, String> {
        let rows = sqlx::query("SELECT * FROM smtp_settings ORDER BY updated_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to list SMTP settings: {}", e))?;

        Ok(rows.into_iter().map(Self::from_row).collect())
    }

    /// Create new SMTP settings.
    pub async fn create(pool: &DbPool, settings: &SmtpSettings) -> Result<Self, String> {
        let id = &settings.id;
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO smtp_settings (id, brand_id, host, port, security, auth_method, \
             username, password, from_address, from_name, reply_to_address, \
             connection_timeout_seconds, enabled, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"
        )
        .bind(id)
        .bind(&settings.brand_id)
        .bind(&settings.host)
        .bind(&settings.port)
        .bind(&settings.security)
        .bind(&settings.auth_method)
        .bind(&settings.username)
        .bind(&settings.password)
        .bind(&settings.from_address)
        .bind(&settings.from_name)
        .bind(&settings.reply_to_address)
        .bind(&settings.connection_timeout_seconds)
        .bind(&settings.enabled)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create SMTP settings: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Failed to create SMTP settings".to_string());
        }

        Ok(settings.clone())
    }

    /// Update SMTP settings.
    pub async fn update(pool: &DbPool, id: &str, settings: &SmtpSettings) -> Result<Self, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let row = sqlx::query(
            "UPDATE smtp_settings SET \
             host = $1, port = $2, security = $3, auth_method = $4, \
             username = $5, password = $6, from_address = $7, from_name = $8, \
             reply_to_address = $9, connection_timeout_seconds = $10, enabled = $11, \
             updated_at = $12 \
             WHERE id = $13 RETURNING *"
        )
        .bind(&settings.host)
        .bind(&settings.port)
        .bind(&settings.security)
        .bind(&settings.auth_method)
        .bind(&settings.username)
        .bind(&settings.password)
        .bind(&settings.from_address)
        .bind(&settings.from_name)
        .bind(&settings.reply_to_address)
        .bind(&settings.connection_timeout_seconds)
        .bind(&settings.enabled)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to update SMTP settings: {}", e))?;

        Ok(Self::from_row(row))
    }

    /// Delete SMTP settings.
    pub async fn delete(pool: &DbPool, id: &str) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM smtp_settings WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to delete SMTP settings: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("SMTP settings not found".to_string());
        }

        Ok(())
    }

    /// Test SMTP connection by sending a test email.
    pub async fn test_connection(pool: &DbPool, id: &str, to_address: &str) -> Result<TestSmtpResponse, String> {
        // Fetch settings
        let settings = sqlx::query("SELECT * FROM smtp_settings WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Failed to fetch settings: {}", e))?
            .ok_or_else(|| "SMTP settings not found".to_string())
            .map(Self::from_row)?;

        if !settings.enabled {
            return Ok(TestSmtpResponse {
                success: false,
                message: "SMTP settings are disabled".to_string(),
                error: Some("Settings disabled".to_string()),
            });
        }

        // Build test email
        let subject = "SMTP Connection Test";
        let body_html = format!(
            "<html><body><h1>SMTP Test</h1><p>This is a test email from Communicator.</p><p>Time: {}</p></body></html>",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        let body_text = format!("SMTP Test from Communicator. Time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));

        // Send test email
        let result = send_smtp_email(
            &settings.host,
            settings.port as u16,
            &settings.security,
            &settings.auth_method,
            &settings.username,
            &settings.password,
            &settings.from_address,
            &settings.from_name,
            &settings.reply_to_address,
            settings.connection_timeout_seconds as u64,
            to_address,
            subject,
            &body_html,
            &body_text,
        ).await;

        let now = chrono::Utc::now().to_rfc3339();

        // Update test status
        match &result {
            Ok(_) => {
                let _ = sqlx::query(
                    "UPDATE smtp_settings SET test_sent_at = $1, test_status = 'success', test_error = NULL WHERE id = $2"
                )
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await;
            }
            Err(e) => {
                let _ = sqlx::query(
                    "UPDATE smtp_settings SET test_sent_at = $1, test_status = 'failed', test_error = $2 WHERE id = $3"
                )
                .bind(&now)
                .bind(e.as_str())
                .bind(id)
                .execute(pool)
                .await;
            }
        }

        match result {
            Ok(_) => Ok(TestSmtpResponse {
                success: true,
                message: format!("Test email sent successfully to {}", to_address),
                error: None,
            }),
            Err(e) => Ok(TestSmtpResponse {
                success: false,
                message: "Test email failed".to_string(),
                error: Some(e),
            }),
        }
    }

    fn from_row(row: PgRow) -> Self {
        Self {
            id: row.try_get("id").unwrap_or_default(),
            brand_id: row.try_get("brand_id").ok(),
            host: row.try_get("host").unwrap_or_default(),
            port: row.try_get("port").unwrap_or(587i64),
            security: row.try_get("security").unwrap_or_else(|| "starttls".to_string()),
            auth_method: row.try_get("auth_method").unwrap_or_else(|| "login".to_string()),
            username: row.try_get("username").unwrap_or_default(),
            password: row.try_get("password").unwrap_or_default(),
            from_address: row.try_get("from_address").unwrap_or_default(),
            from_name: row.try_get("from_name").unwrap_or_default(),
            reply_to_address: row.try_get("reply_to_address").ok(),
            connection_timeout_seconds: row.try_get("connection_timeout_seconds").unwrap_or(30i64),
            enabled: row.try_get("enabled").unwrap_or(true),
            test_sent_at: row.try_get("test_sent_at").ok(),
            test_status: row.try_get("test_status").ok(),
            test_error: row.try_get("test_error").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Email Template CRUD
// ---------------------------------------------------------------------------

impl EmailTemplate {
    /// Get template by name.
    pub async fn get_by_name(pool: &DbPool, name: &str, brand_id: Option<&str>) -> Result<Option<Self>, String> {
        let row = if let Some(brand) = brand_id {
            sqlx::query(
                "SELECT * FROM email_templates WHERE name = $1 AND (brand_id = $2 OR brand_id IS NULL) AND enabled = true LIMIT 1"
            )
            .bind(name)
            .bind(brand)
            .fetch_optional(pool)
            .await
        } else {
            sqlx::query(
                "SELECT * FROM email_templates WHERE name = $1 AND brand_id IS NULL AND enabled = true LIMIT 1"
            )
            .bind(name)
            .fetch_optional(pool)
            .await
        }
        .map_err(|e| format!("Failed to fetch template: {}", e))?;

        Ok(row.map(Self::from_row))
    }

    /// List all templates with optional filters.
    pub async fn list(
        pool: &DbPool,
        category: Option<&str>,
        template_type: Option<&str>,
        brand_id: Option<&str>,
    ) -> Result<Vec<Self>, String> {
        let mut query = String::from("SELECT * FROM email_templates WHERE 1=1");

        if let Some(cat) = category {
            query.push_str(" AND category = $1");
        }

        if let Some(ty) = template_type {
            let idx = if category.is_some() { 2 } else { 1 };
            query.push_str(&format!(" AND template_type = ${}", idx));
        }

        if let Some(brand) = brand_id {
            let idx = if category.is_some() && template_type.is_some() { 3 } else if category.is_some() || template_type.is_some() { 2 } else { 1 };
            query.push_str(&format!(" AND (brand_id = ${} OR brand_id IS NULL)", idx));
        }

        query.push_str(" ORDER BY is_default DESC, updated_at DESC");

        // Build dynamic query
        let rows = if let Some(cat) = category {
            if let Some(ty) = template_type {
                if let Some(brand) = brand_id {
                    sqlx::query(&query)
                        .bind(cat)
                        .bind(ty)
                        .bind(brand)
                        .fetch_all(pool)
                        .await?
                } else {
                    sqlx::query(&query)
                        .bind(cat)
                        .bind(ty)
                        .fetch_all(pool)
                        .await?
                }
            } else if let Some(brand) = brand_id {
                sqlx::query(&query)
                    .bind(cat)
                    .bind(brand)
                    .fetch_all(pool)
                    .await?
            } else {
                sqlx::query(&query)
                    .bind(cat)
                    .fetch_all(pool)
                    .await?
            }
        } else if let Some(ty) = template_type {
            if let Some(brand) = brand_id {
                sqlx::query(&query)
                    .bind(ty)
                    .bind(brand)
                    .fetch_all(pool)
                    .await?
            } else {
                sqlx::query(&query)
                    .bind(ty)
                    .fetch_all(pool)
                    .await?
            }
        } else if let Some(brand) = brand_id {
            sqlx::query(&query)
                .bind(brand)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query(&query)
                .fetch_all(pool)
                .await?
        };

        Ok(rows.into_iter().map(Self::from_row).collect())
    }

    /// Create a new email template.
    pub async fn create(pool: &DbPool, template: &EmailTemplate) -> Result<Self, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let variables_json = serde_json::to_string(&template.variables)
            .unwrap_or_else(|_| "[]".to_string());

        let result = sqlx::query(
            "INSERT INTO email_templates (id, brand_id, name, subject, body_html, body_text, \
             template_type, category, language, enabled, is_default, variables, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
        )
        .bind(&template.id)
        .bind(&template.brand_id)
        .bind(&template.name)
        .bind(&template.subject)
        .bind(&template.body_html)
        .bind(&template.body_text)
        .bind(&template.template_type)
        .bind(&template.category)
        .bind(&template.language)
        .bind(&template.enabled)
        .bind(&template.is_default)
        .bind(&variables_json)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create template: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Failed to create template".to_string());
        }

        Ok(template.clone())
    }

    /// Update an email template.
    pub async fn update(pool: &DbPool, id: &str, template: &EmailTemplate) -> Result<Self, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let variables_json = serde_json::to_string(&template.variables)
            .unwrap_or_else(|_| "[]".to_string());

        let row = sqlx::query(
            "UPDATE email_templates SET \
             name = $1, subject = $2, body_html = $3, body_text = $4, \
             template_type = $5, category = $6, language = $7, \
             enabled = $8, variables = $9, updated_at = $10 \
             WHERE id = $11 RETURNING *"
        )
        .bind(&template.name)
        .bind(&template.subject)
        .bind(&template.body_html)
        .bind(&template.body_text)
        .bind(&template.template_type)
        .bind(&template.category)
        .bind(&template.language)
        .bind(&template.enabled)
        .bind(&variables_json)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to update template: {}", e))?;

        Ok(Self::from_row(row))
    }

    /// Delete an email template (cannot delete default templates).
    pub async fn delete(pool: &DbPool, id: &str) -> Result<(), String> {
        // Check if it's a default template
        let is_default = sqlx::query_scalar::<_, bool>("SELECT is_default FROM email_templates WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Failed to check template: {}", e))?
            .unwrap_or(false);

        if is_default {
            return Err("Cannot delete default system templates".to_string());
        }

        let result = sqlx::query("DELETE FROM email_templates WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to delete template: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Template not found".to_string());
        }

        Ok(())
    }

    fn from_row(row: PgRow) -> Self {
        let variables_raw: String = row.try_get("variables").unwrap_or_else(|| "[]".to_string());
        let variables = serde_json::from_str(&variables_raw).unwrap_or_else(|_| serde_json::json!([]));

        Self {
            id: row.try_get("id").unwrap_or_default(),
            brand_id: row.try_get("brand_id").ok(),
            name: row.try_get("name").unwrap_or_default(),
            subject: row.try_get("subject").unwrap_or_default(),
            body_html: row.try_get("body_html").unwrap_or_default(),
            body_text: row.try_get("body_text").unwrap_or_default(),
            template_type: row.try_get("template_type").unwrap_or_else(|| "transactional".to_string()),
            category: row.try_get("category").unwrap_or_else(|| "general".to_string()),
            language: row.try_get("language").unwrap_or_else(|| "en".to_string()),
            enabled: row.try_get("enabled").unwrap_or(true),
            is_default: row.try_get("is_default").unwrap_or(false),
            variables,
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Email Send Log
// ---------------------------------------------------------------------------

impl EmailSendLog {
    /// Log an email send attempt.
    pub async fn log(
        pool: &DbPool,
        brand_id: Option<&str>,
        template_id: Option<&str>,
        template_name: Option<&str>,
        to_address: &str,
        from_address: &str,
        subject: &str,
        status: &str,
        error_message: Option<&str>,
        smtp_response: Option<&str>,
        message_id: Option<&str>,
    ) -> Result<String, String> {
        let id = gen_id();
        let sent_at = if status == "sent" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        let now = chrono::Utc::now().to_rfc3339();

        let _ = sqlx::query(
            "INSERT INTO email_send_log (id, brand_id, template_id, template_name, \
             to_address, from_address, subject, status, error_message, smtp_response, \
             message_id, sent_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
        )
        .bind(&id)
        .bind(brand_id)
        .bind(template_id)
        .bind(template_name)
        .bind(to_address)
        .bind(from_address)
        .bind(subject)
        .bind(status)
        .bind(error_message)
        .bind(smtp_response)
        .bind(message_id)
        .bind(&sent_at)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to log email send: {}", e))?;

        Ok(id)
    }

    /// Get recent send logs.
    pub async fn recent(pool: &DbPool, limit: i64) -> Result<Vec<Self>, String> {
        let rows = sqlx::query(
            "SELECT * FROM email_send_log ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch send logs: {}", e))?;

        Ok(rows.into_iter().map(Self::from_row).collect())
    }

    fn from_row(row: PgRow) -> Self {
        Self {
            id: row.try_get("id").unwrap_or_default(),
            brand_id: row.try_get("brand_id").ok(),
            template_id: row.try_get("template_id").ok(),
            template_name: row.try_get("template_name").ok(),
            to_address: row.try_get("to_address").unwrap_or_default(),
            from_address: row.try_get("from_address").unwrap_or_default(),
            subject: row.try_get("subject").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            error_message: row.try_get("error_message").ok(),
            smtp_response: row.try_get("smtp_response").ok(),
            message_id: row.try_get("message_id").ok(),
            sent_at: row.try_get("sent_at").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_smtp_summary(pool: &DbPool) -> Result<SmtpSummary, String> {
    // SMTP counts
    let total_smtp: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM smtp_settings")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count SMTP configs: {}", e))?;
    let enabled_smtp: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM smtp_settings WHERE enabled = true")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count enabled SMTP: {}", e))?;

    // Template counts
    let total_templates: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_templates")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count templates: {}", e))?;
    let enabled_templates: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_templates WHERE enabled = true")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count enabled templates: {}", e))?;
    let default_templates: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_templates WHERE is_default = true")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count default templates: {}", e))?;

    // Send log counts
    let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_send_log")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count send logs: {}", e))?;
    let sent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_send_log WHERE status = 'sent'")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count sent: {}", e))?;
    let failed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_send_log WHERE status = 'failed'")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count failed: {}", e))?;
    let bounced_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_send_log WHERE status = 'bounced'")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count bounced: {}", e))?;

    // Categories
    let cat_rows = sqlx::query("SELECT category, COUNT(*) as count FROM email_templates GROUP BY category ORDER BY count DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch categories: {}", e))?;
    let categories: Vec<CategoryCount> = cat_rows.iter().map(|row| CategoryCount {
        category: row.try_get(0).unwrap_or_default(),
        count: row.try_get(1).unwrap_or(0),
    }).collect();

    // Recent logs
    let recent_logs = EmailSendLog::recent(pool, 10).await.unwrap_or_default();

    Ok(SmtpSummary {
        total_smtp_configs: total_smtp,
        enabled_smtp_configs: enabled_smtp,
        total_templates,
        enabled_templates,
        default_templates,
        total_send_logs: total_logs,
        sent_count,
        failed_count,
        bounced_count,
        categories,
        recent_logs,
    })
}

// ---------------------------------------------------------------------------
// High-level send function
// ---------------------------------------------------------------------------

/// Send a templated email. Resolves SMTP settings, renders the template, sends via SMTP, and logs.
pub async fn send_templated_email(
    pool: &DbPool,
    request: &SendEmailRequest,
) -> Result<SendEmailResponse, String> {
    // 1. Resolve SMTP settings
    let settings = SmtpSettings::get(pool, request.brand_id)
        .await?
        .ok_or_else(|| "No SMTP settings configured. Please configure SMTP first.".to_string())?;

    if !settings.enabled {
        return Err("SMTP is disabled".to_string());
    }

    // 2. Resolve template
    let template = EmailTemplate::get_by_name(pool, &request.template_name, request.brand_id)
        .await?
        .ok_or_else(|| format!("Template '{}' not found", request.template_name))?;

    // 3. Render template
    let subject = render_template(&template.subject, &request.variables);
    let body_html = render_template(&template.body_html, &request.variables);
    let body_text = if template.body_text.is_empty() {
        // Strip HTML tags for plain text fallback
        strip_html_tags(&body_html)
    } else {
        render_template(&template.body_text, &request.variables)
    };

    // 4. Send via SMTP
    let send_result = send_smtp_email(
        &settings.host,
        settings.port as u16,
        &settings.security,
        &settings.auth_method,
        &settings.username,
        &settings.password,
        &settings.from_address,
        &settings.from_name,
        &settings.reply_to_address,
        settings.connection_timeout_seconds as u64,
        &request.to_address,
        &subject,
        &body_html,
        &body_text,
    ).await;

    // 5. Log the send
    let log_id = match &send_result {
        Ok(msg_id) => {
            EmailSendLog::log(
                pool,
                request.brand_id,
                Some(&template.id),
                Some(&template.name),
                &request.to_address,
                &settings.from_address,
                &subject,
                "sent",
                None,
                None,
                Some(msg_id),
            ).await.unwrap_or_else(|e| {
                warn!("Failed to log email send: {}", e);
                gen_id()
            })
        }
        Err(e) => {
            EmailSendLog::log(
                pool,
                request.brand_id,
                Some(&template.id),
                Some(&template.name),
                &request.to_address,
                &settings.from_address,
                &subject,
                "failed",
                Some(e),
                None,
                None,
            ).await.unwrap_or_else(|e| {
                warn!("Failed to log email send: {}", e);
                gen_id()
            })
        }
    };

    match send_result {
        Ok(msg_id) => Ok(SendEmailResponse {
            success: true,
            log_id,
            message_id: Some(msg_id),
            error: None,
        }),
        Err(e) => Ok(SendEmailResponse {
            success: false,
            log_id,
            message_id: None,
            error: Some(e),
        }),
    }
}

// ---------------------------------------------------------------------------
// SMTP transport (raw TCP + SMTP protocol)
// ---------------------------------------------------------------------------

/// Send an email via raw SMTP protocol.
/// Returns the Message-ID on success.
async fn send_smtp_email(
    host: &str,
    port: u16,
    security: &str,
    auth_method: &str,
    username: &str,
    password: &str,
    from_address: &str,
    from_name: &str,
    reply_to: &Option<String>,
    timeout_secs: u64,
    to_address: &str,
    subject: &str,
    body_html: &str,
    body_text: &str,
) -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    info!("Sending email via SMTP: {}:{} (security={})", host, port, security);

    // Build MIME message
    let boundary = format!("----=_Part_{}", gen_id().replace('-', ""));
    let message_id = format!("<{}@communicator.local>", gen_id().replace('-', ""));

    let from_display = if from_name.is_empty() {
        from_address.to_string()
    } else {
        format!("{} <{}>", from_name, from_address)
    };

    let reply_to_header = match reply_to {
        Some(addr) if !addr.is_empty() => format!("Reply-To: {}\r\n", addr),
        _ => String::new(),
    };

    let mime_message = format!(
        "From: {}\r\n\
         To: {}\r\n\
         {}\
         Subject: {}\r\n\
         Message-ID: {}\r\n\
         Date: {}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/alternative; boundary=\"{}\"\r\n\
         \r\n\
         This is a MIME-encoded message\r\n\
         --{}\r\n\
         Content-Type: text/plain; charset=\"UTF-8\"\r\n\
         Content-Transfer-Encoding: quoted-printable\r\n\
         \r\n\
         {}\r\n\
         --{}\r\n\
         Content-Type: text/html; charset=\"UTF-8\"\r\n\
         Content-Transfer-Encoding: quoted-printable\r\n\
         \r\n\
         {}\r\n\
         --{}--\r\n\
         .\r\n",
        from_display,
        to_address,
        reply_to_header,
        subject,
        message_id,
        chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000"),
        boundary,
        boundary,
        quoted_printable_encode(body_text),
        boundary,
        quoted_printable_encode(body_html),
        boundary
    );

    // Connect to SMTP server
    let stream = tokio::time::timeout(
        tokio::time::Duration::from_secs(timeout_secs),
        TcpStream::connect(format!("{}:{}", host, port))
    ).await
    .map_err(|_| format!("Connection timeout to {}:{}", host, port))?
    .map_err(|e| format!("Failed to connect to {}: {}", host, e))?;

    // Handle TLS
    let (mut reader, mut writer) = if security == "tls" {
        // Implicit TLS (port 465)
        let tls_stream = tokio_native_tls::TlsConnector::new()
            .connect(host, stream)
            .await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;
        let (read_half, write_half) = tokio::io::split(tls_stream);
        (BufReader::new(read_half), write_half)
    } else {
        let (read_half, write_half) = tokio::io::split(stream);
        (BufReader::new(read_half), write_half)
    };

    // Read greeting
    let greeting = read_smtp_line(&mut reader).await?;
    if !greeting.starts_with("220") {
        return Err(format!("Unexpected SMTP greeting: {}", greeting));
    }

    // EHLO
    write_smtp_line(&mut writer, &format!("EHLO communicator\r\n")).await?;
    let _ehlo_response = read_smtp_lines(&mut reader).await?;

    // STARTTLS (if using port 587)
    if security == "starttls" {
        write_smtp_line(&mut writer, "STARTTLS\r\n").await?;
        let _starttls_response = read_smtp_lines(&mut reader).await?;

        // Upgrade to TLS
        let tcp_stream = writer.into_inner();
        let tls_stream = tokio_native_tls::TlsConnector::new()
            .connect(host, tcp_stream)
            .await
            .map_err(|e| format!("STARTTLS handshake failed: {}", e))?;
        let (read_half, write_half) = tokio::io::split(tls_stream);
        reader = BufReader::new(read_half);
        writer = write_half;

        // EHLO again after TLS
        write_smtp_line(&mut writer, "EHLO communicator\r\n").await?;
        let _ehlo_response = read_smtp_lines(&mut reader).await?;
    }

    // Authentication
    if auth_method != "none" && !username.is_empty() {
        match auth_method {
            "plain" => {
                // PLAIN auth
                let auth_string = format!("\0{}\0{}", username, password);
                let encoded = base64_encode(&auth_string);
                write_smtp_line(&mut writer, &format!("AUTH PLAIN {}\r\n", encoded)).await?;
                let _auth_response = read_smtp_lines(&mut reader).await?;
            }
            "login" | _ => {
                // LOGIN auth
                write_smtp_line(&mut writer, "AUTH LOGIN\r\n").await?;
                let _challenge = read_smtp_lines(&mut reader).await?;

                let encoded_user = base64_encode(username);
                write_smtp_line(&mut writer, &format!("{}\r\n", encoded_user)).await?;
                let _challenge = read_smtp_lines(&mut reader).await?;

                let encoded_pass = base64_encode(password);
                write_smtp_line(&mut writer, &format!("{}\r\n", encoded_pass)).await?;
                let _auth_response = read_smtp_lines(&mut reader).await?;
            }
        }
    }

    // MAIL FROM
    write_smtp_line(&mut writer, &format!("MAIL FROM: <{}>\r\n", from_address)).await?;
    let _mail_response = read_smtp_lines(&mut reader).await?;

    // RCPT TO
    write_smtp_line(&mut writer, &format!("RCPT TO: <{}>\r\n", to_address)).await?;
    let _rcpt_response = read_smtp_lines(&mut reader).await?;

    // DATA
    write_smtp_line(&mut writer, "DATA\r\n").await?;
    let _data_response = read_smtp_lines(&mut reader).await?;

    // Send message body
    write_smtp_line(&mut writer, &mime_message).await?;

    // Read final response
    let final_response = read_smtp_lines(&mut reader).await?;
    info!("SMTP final response: {}", final_response.lines().next().unwrap_or(""));

    // QUIT
    write_smtp_line(&mut writer, "QUIT\r\n").await?;

    Ok(message_id)
}

async fn read_smtp_line<R>(reader: &mut BufReader<R>) -> Result<String, String>
where
    R: tokio::io::AsyncRead + Unpin + Send,
{
    let mut line = String::new();
    reader.read_line(&mut line).await
        .map_err(|e| format!("Failed to read SMTP response: {}", e))?;
    Ok(line.trim().to_string())
}

async fn read_smtp_lines<R>(reader: &mut BufReader<R>) -> Result<String, String>
where
    R: tokio::io::AsyncRead + Unpin + Send,
{
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await
            .map_err(|e| format!("Failed to read SMTP response: {}", e))?;

        let trimmed = line.trim();
        lines.push(trimmed.to_string());

        // SMTP multi-line responses use "code-continue" format
        // e.g., "250-Feature1\r\n250-Feature2\r\n250 OK\r\n"
        if trimmed.len() >= 4 && trimmed.as_bytes()[3] == b' ' {
            break; // Final line has a space after the 3-digit code
        }
    }
    Ok(lines.join("\n"))
}

async fn write_smtp_line<W>(writer: &mut W, data: &str) -> Result<(), String>
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    writer.write_all(data.as_bytes()).await
        .map_err(|e| format!("Failed to write to SMTP: {}", e))?;
    writer.flush().await
        .map_err(|e| format!("Failed to flush SMTP: {}", e))?;
    Ok(())
}

fn base64_encode(input: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(input.as_bytes())
}

fn quoted_printable_encode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            33..=60 | 62..=126 => result.push(byte as char),
            b' ' => result.push(' '),
            b'\n' => result.push('\n'),
            b'\r' => result.push('\r'),
            _ => {
                result.push_str(&format!("={:02X}", byte));
            }
        }
    }
    result
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(ch);
        }
    }
    // Collapse whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Public helper for OTP engine: send a raw email without template resolution
// ---------------------------------------------------------------------------

/// Send a raw email using the given SMTP settings. Used by the OTP engine
/// to dispatch verification codes via email.
pub async fn send_raw_email(
    _pool: &DbPool,
    host: &str,
    port: u16,
    security: &str,
    auth_method: &str,
    username: &str,
    password: &str,
    from_address: &str,
    from_name: &str,
    reply_to: Option<&str>,
    to_address: &str,
    subject: &str,
    body_html: &str,
) -> Result<String, String> {
    let body_text = strip_html_tags(body_html);
    let reply_to_opt = reply_to.map(|s| s.to_string());
    send_smtp_email(
        host, port, security, auth_method, username, password,
        from_address, from_name, &reply_to_opt, 30,
        to_address, subject, body_html, &body_text,
    ).await
}
