#![allow(dead_code)]

//! Shopify Store OTP Integration Service
//!
//! Connects a brand's Shopify storefront to the Communicator OTP engine.

use std::collections::HashMap;

use sqlx::PgPool;
use newax_otp_engine::{OtpEngine, GenerateOtpRequest};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyStoreConnection {
    pub id: String,
    pub brand_id: String,
    pub store_domain: String,
    pub store_name: String,
    pub shopify_api_key: String,
    pub shopify_api_secret_hash: String,
    pub shopify_api_secret_salt: String,
    pub shopify_access_token_hash: String,
    pub shopify_access_token_salt: String,
    pub shopify_webhook_secret: String,
    pub integration_id: Option<String>,
    pub default_purpose: String,
    pub auto_create_customer: bool,
    pub customer_tag: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for ShopifyStoreConnection {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
        let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at")?;
        Ok(ShopifyStoreConnection {
            id: row.try_get("id")?,
            brand_id: row.try_get("brand_id")?,
            store_domain: row.try_get("store_domain")?,
            store_name: row.try_get("store_name")?,
            shopify_api_key: row.try_get("shopify_api_key")?,
            shopify_api_secret_hash: row.try_get("shopify_api_secret_hash")?,
            shopify_api_secret_salt: row.try_get("shopify_api_secret_salt")?,
            shopify_access_token_hash: row.try_get("shopify_access_token_hash")?,
            shopify_access_token_salt: row.try_get("shopify_access_token_salt")?,
            shopify_webhook_secret: row.try_get("shopify_webhook_secret")?,
            integration_id: row.try_get("integration_id")?,
            default_purpose: row.try_get("default_purpose")?,
            auto_create_customer: row.try_get("auto_create_customer")?,
            customer_tag: row.try_get("customer_tag")?,
            is_active: row.try_get("is_active")?,
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateShopifyConnectionRequest {
    pub brand_id: String,
    pub store_domain: String,
    pub store_name: String,
    pub shopify_api_key: Option<String>,
    pub shopify_api_secret: Option<String>,
    pub shopify_access_token: Option<String>,
    pub shopify_webhook_secret: Option<String>,
    pub auto_create_customer: Option<bool>,
    pub customer_tag: Option<String>,
    pub default_purpose: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateShopifyConnectionResponse {
    pub id: String,
    pub store_domain: String,
    pub public_key: String,
    pub webhook_url: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyPendingCustomer {
    pub id: String,
    pub store_connection_id: String,
    pub brand_id: String,
    pub otp_id: String,
    pub email: String,
    pub phone: String,
    pub first_name: String,
    pub last_name: String,
    pub account_status: String,
    pub shopify_customer_id: Option<String>,
    pub verification_attempts: i64,
    pub expires_at: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShopifyAccountCreationRequest {
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub channel: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopifyAccountCreationResponse {
    pub success: bool,
    pub pending_id: String,
    pub otp_id: String,
    pub channel: String,
    pub message: String,
    pub expires_in_seconds: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShopifyOtpVerificationRequest {
    pub pending_id: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopifyOtpVerificationResponse {
    pub success: bool,
    pub verified: bool,
    pub message: String,
    pub customer_created: bool,
    pub shopify_customer_id: Option<String>,
    pub session_token: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyChannelPreference {
    pub id: String,
    pub store_connection_id: String,
    pub purpose: String,
    pub whatsapp_enabled: bool,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub in_app_enabled: bool,
    pub default_channel: String,
    pub priority_order: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChannelPreferenceRequest {
    pub purpose: Option<String>,
    pub whatsapp_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub default_channel: Option<String>,
    pub priority_order: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct ShopifyStoreState {
    pub db: PgPool,
}

// ---------------------------------------------------------------------------
// Store connection management
// ---------------------------------------------------------------------------

pub async fn create_connection(
    pool: &PgPool,
    req: &CreateShopifyConnectionRequest,
) -> Result<CreateShopifyConnectionResponse, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let connection_id = format!("shop_{}", uuid::Uuid::new_v4().as_hyphenated());

    let (api_secret_hash, api_secret_salt) = if let Some(ref secret) = req.shopify_api_secret {
        let mut salt = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        let salt_hex = hex::encode(&salt);
        let mut mac = HmacSha256::new_from_slice(&salt)
            .map_err(|e| format!("Failed to create HMAC: {}", e))?;
        mac.update(secret.as_bytes());
        (hex::encode(mac.finalize().into_bytes()), salt_hex)
    } else {
        ("".to_string(), "".to_string())
    };

    let (access_token_hash, access_token_salt) = (
        req.shopify_access_token.clone().unwrap_or_default(),
        "".to_string(),
    );

    let public_key = format!(
        "pk_shop_{}_{}",
        &req.brand_id[..8.min(req.brand_id.len())],
        nanoid::nanoid!(8)
    );

    let webhook_url = format!("/api/v1/shopify/webhook/{}", connection_id);

    sqlx::query(
        r#"INSERT INTO shopify_store_connections
            (id, brand_id, store_domain, store_name,
             shopify_api_key, shopify_api_secret_hash, shopify_api_secret_salt,
             shopify_access_token_hash, shopify_access_token_salt,
             shopify_webhook_secret, default_purpose,
             auto_create_customer, customer_tag, is_active)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#
    )
    .bind(&connection_id)
    .bind(&req.brand_id)
    .bind(&req.store_domain)
    .bind(&req.store_name)
    .bind(req.shopify_api_key.as_deref().unwrap_or(""))
    .bind(&api_secret_hash)
    .bind(&api_secret_salt)
    .bind(&access_token_hash)
    .bind(&access_token_salt)
    .bind(req.shopify_webhook_secret.as_deref().unwrap_or(""))
    .bind(req.default_purpose.as_deref().unwrap_or("sign_up"))
    .bind(req.auto_create_customer.unwrap_or(true))
    .bind(req.customer_tag.as_deref().unwrap_or("otp_verified"))
    .bind(true)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create Shopify connection: {}", e))?;

    sqlx::query(
        r#"INSERT INTO shopify_channel_preferences
            (store_connection_id, purpose, whatsapp_enabled, email_enabled,
             default_channel, priority_order)
           VALUES ($1, 'sign_up', true, true, 'whatsapp', ARRAY['whatsapp', 'email', 'sms'])
           ON CONFLICT (store_connection_id, purpose) DO NOTHING"#
    )
    .bind(&connection_id)
    .execute(pool)
    .await
    .ok();

    info!("Created Shopify store connection {} for brand {} (domain: {})",
          connection_id, req.brand_id, req.store_domain);

    Ok(CreateShopifyConnectionResponse {
        id: connection_id,
        store_domain: req.store_domain.clone(),
        public_key,
        webhook_url,
        message: "Shopify store connected successfully. Use the public key to embed the OTP widget on your store.".to_string(),
    })
}

pub async fn get_by_domain(
    pool: &PgPool,
    store_domain: &str,
) -> Result<ShopifyStoreConnection, String> {
    let conn: Option<ShopifyStoreConnection> = sqlx::query_as(
        r#"SELECT id, brand_id, store_domain, store_name,
                  shopify_api_key, shopify_api_secret_hash, shopify_api_secret_salt,
                  shopify_access_token_hash, shopify_access_token_salt,
                  shopify_webhook_secret, integration_id, default_purpose,
                  auto_create_customer, customer_tag,
                  is_active, created_at, updated_at
           FROM shopify_store_connections
           WHERE store_domain = $1 AND is_active = true
           LIMIT 1"#
    )
    .bind(store_domain)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch connection: {}", e))?;

    conn.ok_or_else(|| "Shopify store connection not found or inactive".to_string())
}

pub async fn get_by_id(
    pool: &PgPool,
    connection_id: &str,
) -> Result<ShopifyStoreConnection, String> {
    let conn: Option<ShopifyStoreConnection> = sqlx::query_as(
        r#"SELECT id, brand_id, store_domain, store_name,
                  shopify_api_key, shopify_api_secret_hash, shopify_api_secret_salt,
                  shopify_access_token_hash, shopify_access_token_salt,
                  shopify_webhook_secret, integration_id, default_purpose,
                  auto_create_customer, customer_tag,
                  is_active, created_at, updated_at
           FROM shopify_store_connections
           WHERE id = $1 AND is_active = true
           LIMIT 1"#
    )
    .bind(connection_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch connection: {}", e))?;

    conn.ok_or_else(|| "Shopify store connection not found or inactive".to_string())
}

pub async fn list_by_brand(
    pool: &PgPool,
    brand_id: &str,
) -> Result<Vec<ShopifyStoreConnection>, String> {
    let conns: Vec<ShopifyStoreConnection> = sqlx::query_as(
        r#"SELECT id, brand_id, store_domain, store_name,
                  shopify_api_key, shopify_api_secret_hash, shopify_api_secret_salt,
                  shopify_access_token_hash, shopify_access_token_salt,
                  shopify_webhook_secret, integration_id, default_purpose,
                  auto_create_customer, customer_tag,
                  is_active, created_at, updated_at
           FROM shopify_store_connections
           WHERE brand_id = $1
           ORDER BY created_at DESC"#
    )
    .bind(brand_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch connections: {}", e))?;

    Ok(conns)
}

// ---------------------------------------------------------------------------
// Account creation flow
// ---------------------------------------------------------------------------

pub async fn initiate_account_creation(
    pool: &PgPool,
    store_domain: &str,
    req: &ShopifyAccountCreationRequest,
    ip_address: &str,
    user_agent: Option<String>,
) -> Result<ShopifyAccountCreationResponse, String> {
    let connection = get_by_domain(pool, store_domain).await?;

    let (recipient, recipient_type) = if let Some(ref phone) = req.phone {
        if !phone.is_empty() {
            (phone.clone(), "phone".to_string())
        } else {
            (req.email.clone(), "email".to_string())
        }
    } else {
        (req.email.clone(), "email".to_string())
    };

    let channel_prefs = get_channel_preferences(pool, &connection.id, "sign_up").await;

    let channel = select_best_channel(
        &recipient_type,
        req.channel.as_deref(),
        &channel_prefs,
    );

    let otp_req = GenerateOtpRequest {
        recipient,
        recipient_type: Some(recipient_type),
        purpose: Some("sign_up".to_string()),
        channel: Some(channel.clone()),
        metadata: Some(HashMap::from([
            ("store_domain".to_string(), store_domain.to_string()),
            ("source".to_string(), "shopify_store".to_string()),
        ])),
        ip_address: Some(ip_address.to_string()),
        user_agent,
    };

    let otp_result = OtpEngine::generate_and_send(pool, &connection.brand_id, &otp_req).await?;

    if !otp_result.success {
        return Ok(ShopifyAccountCreationResponse {
            success: false,
            pending_id: "".to_string(),
            otp_id: "".to_string(),
            channel: channel.clone(),
            message: otp_result.message,
            expires_in_seconds: 0,
            error: otp_result.error,
        });
    }

    let pending_id = format!("pending_{}", uuid::Uuid::new_v4().as_hyphenated());
    let ttl_seconds = otp_result.expires_in_seconds;

    sqlx::query(
        r#"INSERT INTO shopify_pending_customers
            (id, store_connection_id, brand_id, otp_id, email, phone,
             first_name, last_name, account_status, expires_at, metadata)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'otp_pending', NOW() + ($9 || ' seconds')::INTERVAL, $10)"#
    )
    .bind(&pending_id)
    .bind(&connection.id)
    .bind(&connection.brand_id)
    .bind(&otp_result.otp_id)
    .bind(&req.email)
    .bind(req.phone.as_deref().unwrap_or(""))
    .bind(req.first_name.as_deref().unwrap_or(""))
    .bind(req.last_name.as_deref().unwrap_or(""))
    .bind(&ttl_seconds.to_string())
    .bind(req.metadata.clone().unwrap_or_else(|| serde_json::json!({})))
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create pending customer: {}", e))?;

    info!("Initiated Shopify account creation for {} via {} (pending: {})",
          req.email, channel, pending_id);

    let message = format!("OTP sent via {}. Please enter the code to complete your account creation.", channel);

    Ok(ShopifyAccountCreationResponse {
        success: true,
        pending_id,
        otp_id: otp_result.otp_id,
        channel,
        message,
        expires_in_seconds: ttl_seconds,
        error: None,
    })
}

pub async fn verify_account_creation(
    pool: &PgPool,
    req: &ShopifyOtpVerificationRequest,
    ip_address: &str,
    user_agent: Option<String>,
) -> Result<ShopifyOtpVerificationResponse, String> {
    use newax_otp_engine::{OtpEngine, VerifyOtpRequest};

    let pending: Option<(
        String, String, String, String, String, String,
        String, String, String, Option<String>, i64,
        String, serde_json::Value, String, Option<String>,
    )> = sqlx::query_as(
        r#"SELECT id, store_connection_id, brand_id, otp_id, email, phone,
                  first_name, last_name, account_status, shopify_customer_id,
                  verification_attempts, expires_at, metadata, created_at, verified_at
           FROM shopify_pending_customers
           WHERE id = $1
           LIMIT 1"#
    )
    .bind(&req.pending_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch pending customer: {}", e))?;

    let pending_data = pending.ok_or("Pending customer session not found".to_string())?;

    if pending_data.8 == "verified" {
        return Ok(ShopifyOtpVerificationResponse {
            success: true,
            verified: true,
            message: "Account already verified.".to_string(),
            customer_created: true,
            shopify_customer_id: pending_data.9.clone(),
            session_token: None,
            error: None,
        });
    }

    let now = chrono::Utc::now();
    let expires_at = chrono::DateTime::parse_from_rfc3339(&pending_data.11)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or(now);

    if now > expires_at {
        let _ = sqlx::query(
            "UPDATE shopify_pending_customers SET account_status = 'expired' WHERE id = $1"
        )
        .bind(&req.pending_id)
        .execute(pool)
        .await;

        return Ok(ShopifyOtpVerificationResponse {
            success: false,
            verified: false,
            message: "OTP has expired. Please request a new code.".to_string(),
            customer_created: false,
            shopify_customer_id: None,
            session_token: None,
            error: Some("expired".to_string()),
        });
    }

    let verify_req = VerifyOtpRequest {
        otp_id: pending_data.3.clone(),
        code: req.code.clone(),
        ip_address: Some(ip_address.to_string()),
        user_agent,
    };

    let verify_result = OtpEngine::verify(pool, &verify_req).await?;

    if !verify_result.verified {
        let _ = sqlx::query(
            "UPDATE shopify_pending_customers SET verification_attempts = verification_attempts + 1 WHERE id = $1"
        )
        .bind(&req.pending_id)
        .execute(pool)
        .await;

        return Ok(ShopifyOtpVerificationResponse {
            success: false,
            verified: false,
            message: verify_result.message,
            customer_created: false,
            shopify_customer_id: None,
            session_token: None,
            error: verify_result.error,
        });
    }

    sqlx::query(
        r#"UPDATE shopify_pending_customers
           SET account_status = 'verified', verified_at = NOW()
           WHERE id = $1"#
    )
    .bind(&req.pending_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update pending customer: {}", e))?;

    let connection = get_by_id(pool, &pending_data.1).await;

    let (customer_created, shopify_customer_id) = if let Ok(conn) = &connection {
        if conn.auto_create_customer {
            match create_shopify_customer(
                pool, conn, &pending_data.0, &pending_data.4, &pending_data.5,
                &pending_data.6, &pending_data.7, &conn.customer_tag,
            ).await {
                Ok(customer_id) => (true, Some(customer_id)),
                Err(e) => {
                    warn!("Failed to auto-create Shopify customer: {}", e);
                    (false, None)
                }
            }
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };

    info!("Shopify account creation verified for {} (pending: {}, customer_created: {})",
          pending_data.4, req.pending_id, customer_created);

    Ok(ShopifyOtpVerificationResponse {
        success: true,
        verified: true,
        message: if customer_created {
            "Account verified and created successfully!".to_string()
        } else {
            "Account verified. Your account is being set up.".to_string()
        },
        customer_created,
        shopify_customer_id,
        session_token: verify_result.session_token,
        error: None,
    })
}

// ---------------------------------------------------------------------------
// Shopify Admin API integration
// ---------------------------------------------------------------------------

async fn create_shopify_customer(
    pool: &PgPool,
    connection: &ShopifyStoreConnection,
    pending_id: &str,
    email: &str,
    phone: &str,
    first_name: &str,
    last_name: &str,
    tag: &str,
) -> Result<String, String> {
    if connection.shopify_access_token_hash.is_empty() {
        return Err("Shopify access token not configured. Customer will need to be created manually.".to_string());
    }
    let access_token = &connection.shopify_access_token_hash;

    let url = format!(
        "https://{}/admin/api/2024-01/customers.json",
        connection.store_domain
    );
    let body = serde_json::json!({
        "customer": {
            "first_name": first_name,
            "last_name": last_name,
            "email": email,
            "phone": if phone.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(phone.to_string()) },
            "tags": tag,
            "verified_email": true,
        }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("X-Shopify-Access-Token", access_token.as_str())
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to reach Shopify Admin API: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Shopify Admin API returned {}: {}",
            status, text
        ));
    }

    let payload: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Shopify customer response: {}", e))?;
    let customer_id = payload
        .get("customer")
        .and_then(|c| c.get("id"))
        .map(|id| id.to_string())
        .ok_or_else(|| "Shopify response did not include a customer id".to_string())?;

    info!(
        "Created Shopify customer {} for {} {} ({}) at {}",
        customer_id, first_name, last_name, email, connection.store_domain
    );

    sqlx::query(
        r#"UPDATE shopify_pending_customers
           SET shopify_customer_id = $1
           WHERE id = $2"#
    )
    .bind(&customer_id)
    .bind(pending_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to record Shopify customer id: {}", e))?;

    Ok(customer_id)
}

// ---------------------------------------------------------------------------
// Channel selection
// ---------------------------------------------------------------------------

fn select_best_channel(
    recipient_type: &str,
    requested_channel: Option<&str>,
    channel_prefs: &Option<ShopifyChannelPreference>,
) -> String {
    if let Some(channel) = requested_channel {
        let channel = channel.to_lowercase();
        if channel_is_compatible(&channel, recipient_type) {
            return channel;
        }
    }

    if let Some(prefs) = channel_prefs {
        for channel in &prefs.priority_order {
            if is_channel_enabled(prefs, channel)
                && channel_is_compatible(channel, recipient_type)
            {
                return channel.clone();
            }
        }
    }

    if recipient_type == "phone" {
        "whatsapp".to_string()
    } else {
        "email".to_string()
    }
}

fn channel_is_compatible(channel: &str, recipient_type: &str) -> bool {
    match channel {
        "whatsapp" | "sms" => recipient_type == "phone",
        "email" => recipient_type == "email",
        "in_app" => true,
        _ => false,
    }
}

fn is_channel_enabled(prefs: &ShopifyChannelPreference, channel: &str) -> bool {
    match channel {
        "whatsapp" => prefs.whatsapp_enabled,
        "email" => prefs.email_enabled,
        "sms" => prefs.sms_enabled,
        "in_app" => prefs.in_app_enabled,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Channel preferences
// ---------------------------------------------------------------------------

pub async fn get_channel_preferences(
    pool: &PgPool,
    store_connection_id: &str,
    purpose: &str,
) -> Option<ShopifyChannelPreference> {
    let pref: Option<(
        String, String, String,
        bool, bool, bool, bool,
        String, Vec<String>,
    )> = sqlx::query_as(
        r#"SELECT id, store_connection_id, purpose,
                  whatsapp_enabled, email_enabled, sms_enabled, in_app_enabled,
                  default_channel, priority_order
           FROM shopify_channel_preferences
           WHERE store_connection_id = $1 AND purpose = $2
           LIMIT 1"#
    )
    .bind(store_connection_id)
    .bind(purpose)
    .fetch_optional(pool)
    .await
    .ok()?;

    pref.map(|p| ShopifyChannelPreference {
        id: p.0,
        store_connection_id: p.1,
        purpose: p.2,
        whatsapp_enabled: p.3,
        email_enabled: p.4,
        sms_enabled: p.5,
        in_app_enabled: p.6,
        default_channel: p.7,
        priority_order: p.8,
    })
}

pub async fn update_channel_preferences(
    pool: &PgPool,
    store_connection_id: &str,
    req: &UpdateChannelPreferenceRequest,
) -> Result<ShopifyChannelPreference, String> {
    let purpose = req.purpose.as_deref().unwrap_or("sign_up");

    let query = "UPDATE shopify_channel_preferences SET
            whatsapp_enabled = COALESCE($1, whatsapp_enabled),
            email_enabled = COALESCE($2, email_enabled),
            sms_enabled = COALESCE($3, sms_enabled),
            in_app_enabled = COALESCE($4, in_app_enabled),
            default_channel = COALESCE($5, default_channel),
            priority_order = COALESCE($6, priority_order),
            updated_at = NOW()
        WHERE store_connection_id = $7 AND purpose = $8
        RETURNING id, store_connection_id, purpose, whatsapp_enabled, email_enabled, sms_enabled, in_app_enabled, default_channel, priority_order";

    let pref: Option<(
        String, String, String,
        bool, bool, bool, bool,
        String, Vec<String>,
    )> = sqlx::query_as(query)
        .bind(&req.whatsapp_enabled)
        .bind(&req.email_enabled)
        .bind(&req.sms_enabled)
        .bind(&req.in_app_enabled)
        .bind(req.default_channel.as_deref())
        .bind(req.priority_order.as_ref().map(|v| v.as_slice()))
        .bind(store_connection_id)
        .bind(purpose)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to update channel preferences: {}", e))?;

    match pref {
        Some(p) => Ok(ShopifyChannelPreference {
            id: p.0,
            store_connection_id: p.1,
            purpose: p.2,
            whatsapp_enabled: p.3,
            email_enabled: p.4,
            sms_enabled: p.5,
            in_app_enabled: p.6,
            default_channel: p.7,
            priority_order: p.8,
        }),
        None => Err("Channel preferences not found".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Shopify webhook handling
// ---------------------------------------------------------------------------

pub fn verify_shopify_signature(
    payload: &[u8],
    signature: &str,
    webhook_secret: &str,
) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    if webhook_secret.is_empty() {
        return false;
    }

    let mut mac = match HmacSha256::new_from_slice(webhook_secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload);
    let computed = hex::encode(mac.finalize().into_bytes());
    computed == signature
}

pub async fn log_webhook_event(
    pool: &PgPool,
    store_connection_id: Option<&str>,
    topic: &str,
    shop_domain: &str,
    webhook_id: &str,
    payload: &serde_json::Value,
    signature: &str,
    verified: bool,
) -> Result<String, String> {
    let event_id = format!("wh_{}", uuid::Uuid::new_v4().as_hyphenated());

    sqlx::query(
        r#"INSERT INTO shopify_webhook_events
            (id, store_connection_id, topic, shop_domain, webhook_id,
             payload, signature, verified, processed)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)"#
    )
    .bind(&event_id)
    .bind(store_connection_id)
    .bind(topic)
    .bind(shop_domain)
    .bind(webhook_id)
    .bind(payload)
    .bind(signature)
    .bind(verified)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to log webhook event: {}", e))?;

    Ok(event_id)
}

// ---------------------------------------------------------------------------
// Resend OTP
// ---------------------------------------------------------------------------

pub async fn resend_otp(
    pool: &PgPool,
    pending_id: &str,
    ip_address: &str,
    user_agent: Option<String>,
) -> Result<ShopifyAccountCreationResponse, String> {
    let pending: Option<(
        String, String, String, String, String, String,
        String, String, String,
    )> = sqlx::query_as(
        r#"SELECT id, store_connection_id, brand_id, otp_id, email, phone,
                  first_name, last_name, account_status
           FROM shopify_pending_customers
           WHERE id = $1
           LIMIT 1"#
    )
    .bind(pending_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch pending customer: {}", e))?;

    let pending_data = pending.ok_or("Pending customer session not found".to_string())?;

    if pending_data.8 == "verified" {
        return Ok(ShopifyAccountCreationResponse {
            success: false,
            pending_id: pending_id.to_string(),
            otp_id: "".to_string(),
            channel: "".to_string(),
            message: "Account already verified.".to_string(),
            expires_in_seconds: 0,
            error: Some("already_verified".to_string()),
        });
    }

    let connection = get_by_id(pool, &pending_data.1).await?;

    let (recipient, recipient_type) = if !pending_data.5.is_empty() {
        (pending_data.5.clone(), "phone".to_string())
    } else {
        (pending_data.4.clone(), "email".to_string())
    };

    let channel_prefs = get_channel_preferences(pool, &connection.id, "sign_up").await;
    let channel = select_best_channel(&recipient_type, None, &channel_prefs);

    let otp_req = GenerateOtpRequest {
        recipient,
        recipient_type: Some(recipient_type),
        purpose: Some("sign_up".to_string()),
        channel: Some(channel.clone()),
        metadata: Some(HashMap::from([
            ("store_domain".to_string(), connection.store_domain.clone()),
            ("source".to_string(), "shopify_store_resend".to_string()),
        ])),
        ip_address: Some(ip_address.to_string()),
        user_agent,
    };

    let otp_result = OtpEngine::generate_and_send(pool, &connection.brand_id, &otp_req).await?;

    if !otp_result.success {
        return Ok(ShopifyAccountCreationResponse {
            success: false,
            pending_id: pending_id.to_string(),
            otp_id: "".to_string(),
            channel: channel.clone(),
            message: otp_result.message,
            expires_in_seconds: 0,
            error: otp_result.error,
        });
    }

    let ttl_seconds = otp_result.expires_in_seconds;
    sqlx::query(
        r#"UPDATE shopify_pending_customers
           SET otp_id = $1, expires_at = NOW() + ($2 || ' seconds')::INTERVAL
           WHERE id = $3"#
    )
    .bind(&otp_result.otp_id)
    .bind(&ttl_seconds.to_string())
    .bind(pending_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update pending customer: {}", e))?;

    info!("Resent OTP for Shopify pending customer {} via {}", pending_id, channel);

    Ok(ShopifyAccountCreationResponse {
        success: true,
        pending_id: pending_id.to_string(),
        otp_id: otp_result.otp_id,
        channel,
        message: "New OTP sent. Please enter the code to complete your account creation.".to_string(),
        expires_in_seconds: ttl_seconds,
        error: None,
    })
}
