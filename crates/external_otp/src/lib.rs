//! External OTP Integration — bounded context for external e-commerce platform OTP access.
//!
//! Enables external platforms (Shopify, WooCommerce, etc.) to connect to the Communicator
//! OTP engine via secret-key authenticated API calls with webhook callbacks.

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ExternalOtpState {
    pub pool: PgPool,
    pub otp_engine_url: String,
}

impl ExternalOtpState {
    pub fn new(pool: PgPool, otp_engine_url: String) -> Self {
        Self { pool, otp_engine_url }
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExternalIntegration {
    pub id: String,
    pub brand_id: String,
    pub platform_name: String,
    pub platform_url: String,
    pub display_name: String,
    pub public_key: String,
    pub secret_key_hash: String,
    pub secret_key_salt: String,
    pub webhook_url: String,
    pub webhook_secret: String,
    pub allowed_purposes: Vec<String>,
    pub allowed_channels: Vec<String>,
    pub is_active: bool,
    pub rate_limit_per_minute: i64,
    pub rate_limit_per_hour: i64,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntegrationRequest {
    pub brand_id: String,
    pub platform_name: String,
    pub platform_url: String,
    pub display_name: String,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub allowed_purposes: Option<Vec<String>>,
    pub allowed_channels: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i64>,
    pub rate_limit_per_hour: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateIntegrationResponse {
    pub id: String,
    pub public_key: String,
    pub secret_key: String,
    pub webhook_secret: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalGenerateRequest {
    pub recipient: String,
    pub recipient_type: Option<String>,
    pub purpose: Option<String>,
    pub channel: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalVerifyRequest {
    pub otp_id: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalGenerateResponse {
    pub success: bool,
    pub otp_id: String,
    pub channel: String,
    pub message: String,
    pub expires_in_seconds: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalVerifyResponse {
    pub success: bool,
    pub verified: bool,
    pub message: String,
    pub attempts_remaining: i64,
    pub session_token: Option<String>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Generate a new integration key pair for an external platform.
pub async fn create_integration(
    state: &ExternalOtpState,
    req: &CreateIntegrationRequest,
) -> Result<CreateIntegrationResponse, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let integration_id = format!("ext_{}", uuid::Uuid::new_v4().as_hyphenated());

    let public_key = format!("pk_{}_{}", &req.brand_id[..8.min(req.brand_id.len())], nanoid::nanoid!(8));

    let mut secret_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret_bytes);
    let secret_key = format!("sk_{}", hex::encode(&secret_bytes));

    let mut salt = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
    let salt_hex = hex::encode(&salt);

    let mut mac = HmacSha256::new_from_slice(&salt)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(secret_key.as_bytes());
    let secret_key_hash = hex::encode(mac.finalize().into_bytes());

    let mut webhook_secret_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut webhook_secret_bytes);
    let webhook_secret = hex::encode(&webhook_secret_bytes);

    let allowed_purposes = req.allowed_purposes.clone().unwrap_or_else(|| {
        vec!["sign_up".to_string(), "sign_in".to_string(), "password_reset".to_string(),
             "phone_verify".to_string(), "email_verify".to_string()]
    });

    let allowed_channels = req.allowed_channels.clone().unwrap_or_else(|| {
        vec!["whatsapp".to_string(), "email".to_string(), "sms".to_string(), "in_app".to_string()]
    });

    let purposes_array: Vec<&str> = allowed_purposes.iter().map(|s| s.as_str()).collect();
    let channels_array: Vec<&str> = allowed_channels.iter().map(|s| s.as_str()).collect();

    sqlx::query(
        r#"INSERT INTO otp_external_integrations
            (id, brand_id, platform_name, platform_url, display_name,
             public_key, secret_key_hash, secret_key_salt,
             webhook_url, webhook_secret,
             allowed_purposes, allowed_channels,
             is_active, rate_limit_per_minute, rate_limit_per_hour)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#
    )
    .bind(&integration_id)
    .bind(&req.brand_id)
    .bind(&req.platform_name)
    .bind(&req.platform_url)
    .bind(&req.display_name)
    .bind(&public_key)
    .bind(&secret_key_hash)
    .bind(&salt_hex)
    .bind(req.webhook_url.as_deref().unwrap_or(""))
    .bind(&webhook_secret)
    .bind(&purposes_array)
    .bind(&channels_array)
    .bind(true)
    .bind(req.rate_limit_per_minute.unwrap_or(10))
    .bind(req.rate_limit_per_hour.unwrap_or(100))
    .execute(&state.pool)
    .await
    .map_err(|e| format!("Failed to create integration: {}", e))?;

    info!("Created external OTP integration {} for brand {} (platform: {})",
          integration_id, req.brand_id, req.platform_name);

    Ok(CreateIntegrationResponse {
        id: integration_id,
        public_key,
        secret_key,
        webhook_secret,
        message: "Integration created successfully. Store the secret key securely — it will not be shown again.".to_string(),
    })
}

/// Verify a secret key against the stored hash.
pub async fn verify_secret_key(
    state: &ExternalOtpState,
    secret_key: &str,
) -> Result<ExternalIntegration, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let integrations: Vec<(
        String, String, String, String, String,
        String, String, String, String, String,
        Vec<String>, Vec<String>, bool, i64, i64,
    )> = sqlx::query_as(
        r#"SELECT id, brand_id, platform_name, platform_url, display_name,
                  public_key, secret_key_hash, secret_key_salt,
                  webhook_url, webhook_secret,
                  allowed_purposes, allowed_channels,
                  is_active, rate_limit_per_minute, rate_limit_per_hour
           FROM otp_external_integrations
           WHERE is_active = true"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("Failed to fetch integrations: {}", e))?;

    for integ in &integrations {
        let salt = hex::decode(integ.7.as_str())
            .map_err(|e| format!("Invalid salt: {}", e))?;

        let mut mac = HmacSha256::new_from_slice(&salt)
            .map_err(|e| format!("Failed to create HMAC: {}", e))?;
        mac.update(secret_key.as_bytes());
        let computed_hash = hex::encode(mac.finalize().into_bytes());

        if computed_hash == integ.6 {
            let _ = sqlx::query(
                "UPDATE otp_external_integrations SET last_used_at = NOW(), updated_at = NOW() WHERE id = $1"
            )
            .bind(&integ.0)
            .execute(&state.pool)
            .await;

            return Ok(ExternalIntegration {
                id: integ.0.clone(),
                brand_id: integ.1.clone(),
                platform_name: integ.2.clone(),
                platform_url: integ.3.clone(),
                display_name: integ.4.clone(),
                public_key: integ.5.clone(),
                secret_key_hash: integ.6.clone(),
                secret_key_salt: integ.7.clone(),
                webhook_url: integ.8.clone(),
                webhook_secret: integ.9.clone(),
                allowed_purposes: integ.10.clone(),
                allowed_channels: integ.11.clone(),
                is_active: integ.12,
                rate_limit_per_minute: integ.13,
                rate_limit_per_hour: integ.14,
                last_used_at: None,
                created_at: "".to_string(),
                updated_at: "".to_string(),
            });
        }
    }

    Err("Invalid secret key".to_string())
}

/// Get integration by public key.
pub async fn get_by_public_key(
    state: &ExternalOtpState,
    public_key: &str,
) -> Result<ExternalIntegration, String> {
    let integ: Option<ExternalIntegration> = sqlx::query_as(
        r#"SELECT id, brand_id, platform_name, platform_url, display_name,
                  public_key, secret_key_hash, secret_key_salt,
                  webhook_url, webhook_secret,
                  allowed_purposes, allowed_channels,
                  is_active, rate_limit_per_minute, rate_limit_per_hour,
                  last_used_at, created_at, updated_at
           FROM otp_external_integrations
           WHERE public_key = $1 AND is_active = true
           LIMIT 1"#
    )
    .bind(public_key)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| format!("Failed to fetch integration: {}", e))?;

    integ.ok_or_else(|| "Integration not found or inactive".to_string())
}

/// List all integrations for a brand.
pub async fn list_by_brand(
    state: &ExternalOtpState,
    brand_id: &str,
) -> Result<Vec<ExternalIntegration>, String> {
    let integrations: Vec<ExternalIntegration> = sqlx::query_as(
        r#"SELECT id, brand_id, platform_name, platform_url, display_name,
                  public_key, secret_key_hash, secret_key_salt,
                  webhook_url, webhook_secret,
                  allowed_purposes, allowed_channels,
                  is_active, rate_limit_per_minute, rate_limit_per_hour,
                  last_used_at, created_at, updated_at
           FROM otp_external_integrations
           WHERE brand_id = $1
           ORDER BY created_at DESC"#
    )
    .bind(brand_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("Failed to fetch integrations: {}", e))?;

    Ok(integrations)
}

/// Deactivate an integration.
pub async fn deactivate(
    state: &ExternalOtpState,
    integration_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE otp_external_integrations SET is_active = false, updated_at = NOW() WHERE id = $1"
    )
    .bind(integration_id)
    .execute(&state.pool)
    .await
    .map_err(|e| format!("Failed to deactivate integration: {}", e))?;

    info!("Deactivated external OTP integration {}", integration_id);
    Ok(())
}

/// Generate OTP via external integration.
pub async fn external_generate(
    state: &ExternalOtpState,
    integration: &ExternalIntegration,
    req: &ExternalGenerateRequest,
    ip_address: &str,
    user_agent: Option<String>,
) -> Result<ExternalGenerateResponse, String> {
    // Validate purpose
    let purpose = req.purpose.as_deref().unwrap_or("sign_up");
    if !integration.allowed_purposes.contains(&purpose.to_string()) {
        return Err(format!("Purpose '{}' is not allowed for this integration", purpose));
    }

    // Validate channel
    if let Some(channel) = &req.channel {
        if !integration.allowed_channels.contains(channel) {
            return Err(format!("Channel '{}' is not allowed for this integration", channel));
        }
    }

    // Check integration rate limit
    check_integration_rate_limit(&state.pool, &integration.id, integration).await?;

    // Call the OTP engine via HTTP
    let metadata_json = serde_json::to_string(&req.metadata).unwrap_or_else(|_| "{}".to_string());
    let resp = reqwest::Client::new()
        .post(&format!("{}/api/v1/otp/generate-and-send", state.otp_engine_url))
        .json(&serde_json::json!({
            "brand_id": integration.brand_id,
            "recipient": req.recipient,
            "recipient_type": req.recipient_type.as_deref().unwrap_or(""),
            "purpose": purpose,
            "channel": req.channel.as_deref().unwrap_or(""),
            "metadata": metadata_json,
            "ip_address": ip_address,
            "user_agent": user_agent.as_deref().unwrap_or(""),
        }))
        .send()
        .await
        .map_err(|e| format!("OTP engine unreachable: {}", e))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("OTP engine response error: {}", e))?;

    let success = resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    let otp_id = resp.get("otp_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let channel = resp.get("channel").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let message = resp.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expires_in_seconds = resp.get("expires_in_seconds").and_then(|v| v.as_i64()).unwrap_or(300);
    let error_msg = resp.get("error").and_then(|v| v.as_str());

    // Log usage
    log_usage(&state.pool, &integration.id, &integration.brand_id, "generate",
              &req.recipient, Some(purpose), success, None,
              ip_address, user_agent.as_deref(), None,
              &serde_json::json!(req), &resp).await;

    // Send webhook callback
    if !integration.webhook_url.is_empty() {
        let _ = send_webhook(&state.pool, integration, &otp_id, "otp_generated",
                             &serde_json::json!({
                                 "recipient": req.recipient,
                                 "channel": channel,
                                 "purpose": purpose,
                                 "expires_in_seconds": expires_in_seconds,
                             })).await;
    }

    Ok(ExternalGenerateResponse {
        success,
        otp_id,
        channel,
        message,
        expires_in_seconds,
        error: error_msg.map(|s| s.to_string()),
    })
}

/// Verify OTP via external integration.
pub async fn external_verify(
    state: &ExternalOtpState,
    integration: &ExternalIntegration,
    req: &ExternalVerifyRequest,
    ip_address: &str,
    user_agent: Option<String>,
) -> Result<ExternalVerifyResponse, String> {
    // Fetch the OTP record for logging
    let otp_record: Option<(String, String)> = sqlx::query_as(
        "SELECT recipient, purpose FROM otp_records WHERE id = $1 LIMIT 1"
    )
    .bind(&req.otp_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    // Call the OTP engine via HTTP
    let resp = reqwest::Client::new()
        .post(&format!("{}/api/v1/otp/verify", state.otp_engine_url))
        .json(&serde_json::json!({
            "otp_id": req.otp_id,
            "code": req.code,
            "ip_address": ip_address,
            "user_agent": user_agent.as_deref().unwrap_or(""),
        }))
        .send()
        .await
        .map_err(|e| format!("OTP engine unreachable: {}", e))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("OTP engine response error: {}", e))?;

    let success = resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    let verified = resp.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
    let message = resp.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let attempts_remaining = resp.get("attempts_remaining").and_then(|v| v.as_i64()).unwrap_or(0);
    let session_token = resp.get("session_token").and_then(|v| v.as_str()).map(|s| s.to_string());
    let error_msg = resp.get("error").and_then(|v| v.as_str());

    // Log usage
    let recipient = otp_record.as_ref().map(|r| r.0.as_str());
    let purpose = otp_record.as_ref().map(|r| r.1.as_str());
    log_usage(&state.pool, &integration.id, &integration.brand_id, "verify",
              recipient.unwrap_or(""), purpose, success, None,
              ip_address, user_agent.as_deref(), None,
              &serde_json::json!({"otp_id": req.otp_id}), &resp).await;

    // Send webhook callback
    if !integration.webhook_url.is_empty() {
        let event_type = if verified { "otp_verified" } else { "otp_failed" };
        let _ = send_webhook(&state.pool, integration, &req.otp_id, event_type,
                             &serde_json::json!({
                                 "verified": verified,
                                 "attempts_remaining": attempts_remaining,
                                 "session_token": session_token,
                                 "error": error_msg,
                             })).await;
    }

    Ok(ExternalVerifyResponse {
        success,
        verified,
        message,
        attempts_remaining,
        session_token,
        error: error_msg.map(|s| s.to_string()),
    })
}

// ---------------------------------------------------------------------------
// Rate limiting
// ---------------------------------------------------------------------------

async fn check_integration_rate_limit(
    pool: &PgPool,
    integration_id: &str,
    integration: &ExternalIntegration,
) -> Result<(), String> {
    let minute_count: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*)::INTEGER FROM otp_external_usage_log
         WHERE integration_id = $1 AND created_at > NOW() - INTERVAL '1 minute' LIMIT 1"
    )
    .bind(integration_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some((count,)) = minute_count {
        if count >= integration.rate_limit_per_minute {
            return Err(format!("Rate limit exceeded: {} requests per minute", integration.rate_limit_per_minute));
        }
    }

    let hour_count: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*)::INTEGER FROM otp_external_usage_log
         WHERE integration_id = $1 AND created_at > NOW() - INTERVAL '1 hour' LIMIT 1"
    )
    .bind(integration_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some((count,)) = hour_count {
        if count >= integration.rate_limit_per_hour {
            return Err(format!("Rate limit exceeded: {} requests per hour", integration.rate_limit_per_hour));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Usage logging
// ---------------------------------------------------------------------------

async fn log_usage(
    pool: &PgPool,
    integration_id: &str,
    brand_id: &str,
    action: &str,
    recipient: &str,
    purpose: Option<&str>,
    success: bool,
    response_code: Option<i64>,
    ip_address: &str,
    user_agent: Option<&str>,
    error_message: Option<&str>,
    request_payload: &serde_json::Value,
    response_payload: &serde_json::Value,
) {
    let log_id = format!("log_{}", uuid::Uuid::new_v4().as_hyphenated());

    let result = sqlx::query(
        r#"INSERT INTO otp_external_usage_log
            (id, integration_id, brand_id, action, recipient, purpose,
             success, response_code, ip_address, user_agent, error_message,
             request_payload, response_payload)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#
    )
    .bind(&log_id)
    .bind(integration_id)
    .bind(brand_id)
    .bind(action)
    .bind(recipient)
    .bind(purpose)
    .bind(success)
    .bind(response_code)
    .bind(ip_address)
    .bind(user_agent)
    .bind(error_message)
    .bind(request_payload)
    .bind(response_payload)
    .execute(pool)
    .await;

    if let Err(e) = result {
        warn!("Failed to log external usage: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Webhook delivery
// ---------------------------------------------------------------------------

pub async fn send_webhook(
    pool: &PgPool,
    integration: &ExternalIntegration,
    otp_id: &str,
    event_type: &str,
    data: &serde_json::Value,
) -> Result<(), String> {
    use sha2::{Sha256, Digest};
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<Sha256>;

    if integration.webhook_url.is_empty() {
        return Ok(());
    }

    let event_id = format!("evt_{}", uuid::Uuid::new_v4().as_hyphenated());
    let timestamp = chrono::Utc::now().to_rfc3339();

    let otp_record: Option<(String, String, String)> = sqlx::query_as(
        "SELECT recipient, channel, purpose FROM otp_records WHERE id = $1 LIMIT 1"
    )
    .bind(otp_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let payload = serde_json::json!({
        "event_id": event_id,
        "event_type": event_type,
        "brand_id": integration.brand_id,
        "otp_id": otp_id,
        "recipient": otp_record.as_ref().map(|r| r.0.as_str()).unwrap_or(""),
        "channel": otp_record.as_ref().map(|r| r.1.as_str()).unwrap_or(""),
        "purpose": otp_record.as_ref().map(|r| r.2.as_str()).unwrap_or(""),
        "timestamp": timestamp,
        "data": data,
    });

    let payload_str = payload.to_string();
    let signature = if !integration.webhook_secret.is_empty() {
        let mut mac = HmacSha256::new_from_slice(integration.webhook_secret.as_bytes())
            .map_err(|e| format!("Failed to create HMAC: {}", e))?;
        mac.update(payload_str.as_bytes());
        format!("sha256{}", hex::encode(mac.finalize().into_bytes()))
    } else {
        "".to_string()
    };

    let delivery_id = format!("wh_{}", uuid::Uuid::new_v4().as_hyphenated());
    sqlx::query(
        r#"INSERT INTO otp_webhook_delivery_log
            (id, integration_id, brand_id, otp_id, event_type,
             webhook_url, payload, status)
           VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')"#
    )
    .bind(&delivery_id)
    .bind(&integration.id)
    .bind(&integration.brand_id)
    .bind(otp_id)
    .bind(event_type)
    .bind(&integration.webhook_url)
    .bind(&payload)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to log webhook delivery: {}", e))?;

    let webhook_url = integration.webhook_url.clone();
    let signature = signature.clone();
    let payload_str = payload_str.clone();
    let delivery_id = delivery_id.clone();
    let pool = pool.clone();
    let event_type = event_type.to_string();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut request = client.post(&webhook_url)
            .header("Content-Type", "application/json")
            .header("X-Communicator-Event", &event_type)
            .header("X-Communicator-Timestamp", &timestamp)
            .body(payload_str);

        if !signature.is_empty() {
            request = request.header("X-Communicator-Signature", &signature);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16() as i32;
                let body = response.text().await.unwrap_or_default();

                let delivery_status = if status >= 200 && status < 300 {
                    "delivered"
                } else {
                    "failed"
                };

                let _ = sqlx::query(
                    r#"UPDATE otp_webhook_delivery_log
                       SET status = $1, http_status = $2, response_body = $3, delivered_at = NOW()
                       WHERE id = $4"#
                )
                .bind(delivery_status)
                .bind(status)
                .bind(&body)
                .bind(&delivery_id)
                .execute(&pool)
                .await;

                if delivery_status == "delivered" {
                    info!("Webhook delivered to {} for event {}", webhook_url, event_type);
                } else {
                    warn!("Webhook failed with status {} for event {}", status, event_type);
                }
            }
            Err(e) => {
                error!("Webhook send failed: {}", e);
                let _ = sqlx::query(
                    r#"UPDATE otp_webhook_delivery_log
                       SET status = 'failed', response_body = $1
                       WHERE id = $2"#
                )
                .bind(&e.to_string())
                .bind(&delivery_id)
                .execute(&pool)
                .await;
            }
        }
    });

    Ok(())
}
