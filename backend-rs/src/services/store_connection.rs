//! Platform-Agnostic Store Connection Management Service
//!
//! Manages connections between brands and their e-commerce stores/websites
//! regardless of platform (Shopify, Wix, WordPress, WooCommerce, Square,
//! Magento, BigCommerce, custom stores, etc.).
//!
//! Key features:
//! - Platform-agnostic store connections with CRUD operations
//! - Connection health monitoring and status checks
//! - UI compatibility verification (input boxes, paste support, accessibility)
//! - Brand OTP isolation — each brand has its own isolated OTP engine
//! - No mixing of OTPs, credentials, or rate limits between brands

use crate::db::DbPool;
use crate::services::otp_engine::{OtpEngine, GenerateOtpRequest};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConnection {
    pub id: String,
    pub brand_id: String,
    pub organization_id: Option<String>,
    pub platform_name: String,
    pub platform_version: String,
    pub store_url: String,
    pub store_domain: String,
    pub store_name: String,
    pub display_name: String,
    pub api_key: String,
    pub api_secret_hash: String,
    pub api_secret_salt: String,
    pub access_token_hash: String,
    pub access_token_salt: String,
    pub webhook_secret: String,
    pub external_integration_id: Option<String>,
    pub public_key: String,
    pub webhook_url: Option<String>,
    pub default_purpose: String,
    pub auto_create_customer: bool,
    pub customer_tag: String,
    pub whatsapp_enabled: bool,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub in_app_enabled: bool,
    pub default_channel: String,
    pub priority_order: Vec<String>,
    pub is_active: bool,
    pub connection_status: String,
    pub last_status_check: Option<String>,
    pub last_error: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStoreConnectionRequest {
    pub brand_id: String,
    pub organization_id: Option<String>,
    pub platform_name: String,
    pub platform_version: Option<String>,
    pub store_url: String,
    pub store_domain: String,
    pub store_name: String,
    pub display_name: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub access_token: Option<String>,
    pub webhook_secret: Option<String>,
    pub default_purpose: Option<String>,
    pub auto_create_customer: Option<bool>,
    pub customer_tag: Option<String>,
    pub whatsapp_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub default_channel: Option<String>,
    pub priority_order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStoreConnectionResponse {
    pub id: String,
    pub store_domain: String,
    pub public_key: String,
    pub webhook_url: String,
    pub widget_embed_code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConnectionStatus {
    pub id: String,
    pub store_connection_id: String,
    pub brand_id: String,
    pub heartbeat_status: String,
    pub last_heartbeat_at: Option<String>,
    pub response_time_ms: i64,
    pub uptime_percentage: f64,
    pub api_reachable: bool,
    pub last_api_check_at: Option<String>,
    pub api_version: String,
    pub api_rate_limit_remaining: i64,
    pub webhook_reachable: bool,
    pub last_webhook_check_at: Option<String>,
    pub webhook_events_received: i64,
    pub error_count_24h: i64,
    pub last_error_message: String,
    pub consecutive_errors: i64,
    pub last_full_check_at: Option<String>,
    pub next_check_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCompatibilityResult {
    pub id: String,
    pub store_connection_id: String,
    pub brand_id: String,
    pub check_status: String,
    pub check_score: i64,
    pub checked_at: String,
    pub input_boxes_count: i64,
    pub expected_boxes_count: i64,
    pub input_boxes_match: bool,
    pub input_boxes_error: String,
    pub paste_supported: bool,
    pub paste_event_handler_found: bool,
    pub paste_error: String,
    pub individual_input_paste: bool,
    pub individual_input_paste_error: String,
    pub aria_labels_present: bool,
    pub keyboard_navigation: bool,
    pub screen_reader_compatible: bool,
    pub form_validation_present: bool,
    pub error_messages_visible: bool,
    pub countdown_timer_present: bool,
    pub resend_button_present: bool,
    pub resend_cooldown_enforced: bool,
    pub https_enforced: bool,
    pub csrf_protection_present: bool,
    pub mobile_responsive: bool,
    pub mobile_touch_targets_ok: bool,
    pub overall_compatible: bool,
    pub issues_found: Vec<String>,
    pub recommendations: Vec<String>,
    pub check_method: String,
    pub page_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiCompatibilityReport {
    /// Report submitted by the widget running on the store frontend
    pub store_connection_id: String,
    pub page_url: String,
    pub user_agent: Option<String>,
    
    // Input field checks
    pub input_boxes_count: Option<i64>,
    pub expected_boxes_count: Option<i64>,
    
    // Paste functionality
    pub paste_supported: Option<bool>,
    pub paste_event_handler_found: Option<bool>,
    pub individual_input_paste: Option<bool>,
    
    // Accessibility
    pub aria_labels_present: Option<bool>,
    pub keyboard_navigation: Option<bool>,
    
    // Form validation
    pub form_validation_present: Option<bool>,
    pub error_messages_visible: Option<bool>,
    
    // Timer and resend
    pub countdown_timer_present: Option<bool>,
    pub resend_button_present: Option<bool>,
    
    // Security
    pub https_enforced: Option<bool>,
    pub csrf_protection_present: Option<bool>,
    
    // Mobile
    pub mobile_responsive: Option<bool>,
    pub mobile_touch_targets_ok: Option<bool>,
    
    // Issues found
    pub issues_found: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandOtpIsolation {
    pub id: String,
    pub brand_id: String,
    pub organization_id: Option<String>,
    pub isolation_enabled: bool,
    pub isolation_mode: String,
    pub otp_prefix: String,
    pub otp_namespace: String,
    pub isolated_whatsapp: bool,
    pub isolated_email: bool,
    pub isolated_sms: bool,
    pub rate_limit_per_minute: i64,
    pub rate_limit_per_hour: i64,
    pub cooldown_seconds: i64,
    pub lockout_seconds: i64,
    pub block_cross_brand_otp: bool,
    pub block_shared_recipient: bool,
    pub total_otp_generated: i64,
    pub total_otp_verified: i64,
    pub total_otp_failed: i64,
    pub last_otp_generated_at: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStoreConnectionRequest {
    pub store_name: Option<String>,
    pub display_name: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub access_token: Option<String>,
    pub webhook_secret: Option<String>,
    pub default_purpose: Option<String>,
    pub auto_create_customer: Option<bool>,
    pub whatsapp_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub default_channel: Option<String>,
    pub priority_order: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StoreAccountCreationRequest {
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub channel: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreAccountCreationResponse {
    pub success: bool,
    pub pending_id: String,
    pub otp_id: String,
    pub channel: String,
    pub message: String,
    pub expires_in_seconds: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StoreOtpVerificationRequest {
    pub pending_id: String,
    pub code: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreOtpVerificationResponse {
    pub success: bool,
    pub verified: bool,
    pub message: String,
    pub customer_created: bool,
    pub platform_customer_id: Option<String>,
    pub session_token: Option<String>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// StoreConnectionService
// ---------------------------------------------------------------------------

pub struct StoreConnectionService;

impl StoreConnectionService {
    // -----------------------------------------------------------------------
    // Store connection CRUD
    // -----------------------------------------------------------------------

    /// Create a new platform-agnostic store connection.
    pub async fn create_connection(
        pool: &DbPool,
        req: &CreateStoreConnectionRequest,
    ) -> Result<CreateStoreConnectionResponse, String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let connection_id = format!("store_{}", uuid::Uuid::new_v4().as_hyphenated());

        // Hash the API secret if provided
        let (api_secret_hash, api_secret_salt) = if let Some(ref secret) = req.api_secret {
            let salt = rand::RngCore::next_bytes(&mut rand::thread_rng(), &mut [0u8; 32]);
            let salt_hex = hex::encode(&salt);
            let mut mac = HmacSha256::new_from_slice(&salt)
                .map_err(|e| format!("Failed to create HMAC: {}", e))?;
            mac.update(secret.as_bytes());
            (hex::encode(mac.finalize().into_bytes()), salt_hex)
        } else {
            ("".to_string(), "".to_string())
        };

        // Hash the access token if provided
        let (access_token_hash, access_token_salt) = if let Some(ref token) = req.access_token {
            let salt = rand::RngCore::next_bytes(&mut rand::thread_rng(), &mut [0u8; 32]);
            let salt_hex = hex::encode(&salt);
            let mut mac = HmacSha256::new_from_slice(&salt)
                .map_err(|e| format!("Failed to create HMAC: {}", e))?;
            mac.update(token.as_bytes());
            (hex::encode(mac.finalize().into_bytes()), salt_hex)
        } else {
            ("".to_string(), "".to_string())
        };

        // Generate public key for the store widget
        let public_key = format!(
            "pk_store_{}_{}",
            req.brand_id[..8.min(req.brand_id.len())],
            nanoid::nanoid!(8)
        );

        // Generate webhook URL
        let webhook_url = format!("/api/v1/store/webhook/{}", connection_id);

        // Generate widget embed code
        let widget_embed_code = format!(
            r#"<script src="https://cdn.communicator.app/store-otp-widget.js" defer></script>
<script>
  CommunicatorOTP.init({{
    publicKey: '{}',
    storeId: '{}',
    purpose: '{}'
  }});
</script>"#,
            public_key, connection_id, req.default_purpose.as_deref().unwrap_or("sign_up")
        );

        // Insert the store connection
        let priority_order: Vec<String> = req.priority_order.clone()
            .unwrap_or_else(|| vec!["whatsapp".to_string(), "email".to_string(), "sms".to_string()]);
        let priority_order_str = format!("ARRAY[{}]", priority_order.iter()
            .map(|s| format!("'{}'", s.replace("'", "''")))
            .collect::<Vec<_>>().join(", "));

        sqlx::query(
            r#"INSERT INTO store_connections
                (id, brand_id, organization_id, platform_name, platform_version,
                 store_url, store_domain, store_name, display_name,
                 api_key, api_secret_hash, api_secret_salt,
                 access_token_hash, access_token_salt, webhook_secret,
                 public_key, webhook_url, default_purpose,
                 auto_create_customer, customer_tag,
                 whatsapp_enabled, email_enabled, sms_enabled, in_app_enabled,
                 default_channel, priority_order,
                 is_active, connection_status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)"#
        )
        .bind(&connection_id)
        .bind(&req.brand_id)
        .bind(&req.organization_id)
        .bind(&req.platform_name)
        .bind(req.platform_version.as_deref().unwrap_or(""))
        .bind(&req.store_url)
        .bind(&req.store_domain)
        .bind(&req.store_name)
        .bind(req.display_name.as_deref().unwrap_or(&req.store_name))
        .bind(req.api_key.as_deref().unwrap_or(""))
        .bind(&api_secret_hash)
        .bind(&api_secret_salt)
        .bind(&access_token_hash)
        .bind(&access_token_salt)
        .bind(req.webhook_secret.as_deref().unwrap_or(""))
        .bind(&public_key)
        .bind(&webhook_url)
        .bind(req.default_purpose.as_deref().unwrap_or("sign_up"))
        .bind(req.auto_create_customer.unwrap_or(true))
        .bind(req.customer_tag.as_deref().unwrap_or("otp_verified"))
        .bind(req.whatsapp_enabled.unwrap_or(true))
        .bind(req.email_enabled.unwrap_or(true))
        .bind(req.sms_enabled.unwrap_or(false))
        .bind(req.in_app_enabled.unwrap_or(false))
        .bind(req.default_channel.as_deref().unwrap_or("whatsapp"))
        .bind::<&str>(&priority_order_str)
        .bind(true)
        .bind("pending")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create store connection: {}", e))?;

        // Create initial connection status record
        sqlx::query(
            r#"INSERT INTO store_connection_status
                (store_connection_id, brand_id, check_interval_minutes)
               VALUES ($1, $2, 30)
               ON CONFLICT (store_connection_id) DO NOTHING"#
        )
        .bind(&connection_id)
        .bind(&req.brand_id)
        .execute(pool)
        .await
        .ok();

        // Ensure brand OTP isolation exists
        Self::ensure_brand_isolation(pool, &req.brand_id, &req.organization_id).await.ok();

        info!("Created store connection {} for brand {} (platform: {}, domain: {})",
              connection_id, req.brand_id, req.platform_name, req.store_domain);

        Ok(CreateStoreConnectionResponse {
            id: connection_id,
            store_domain: req.store_domain.clone(),
            public_key,
            webhook_url,
            widget_embed_code,
            message: "Store connected successfully. Embed the widget code on your store's account registration page.".to_string(),
        })
    }

    /// Get store connection by ID.
    pub async fn get_by_id(
        pool: &DbPool,
        connection_id: &str,
    ) -> Result<StoreConnection, String> {
        Self::fetch_connection(pool, "id", connection_id).await
    }

    /// Get store connection by domain.
    pub async fn get_by_domain(
        pool: &DbPool,
        store_domain: &str,
    ) -> Result<StoreConnection, String> {
        Self::fetch_connection(pool, "store_domain", store_domain).await
    }

    /// Get store connection by public key.
    pub async fn get_by_public_key(
        pool: &DbPool,
        public_key: &str,
    ) -> Result<StoreConnection, String> {
        Self::fetch_connection(pool, "public_key", public_key).await
    }

    /// List all store connections for a brand.
    pub async fn list_by_brand(
        pool: &DbPool,
        brand_id: &str,
    ) -> Result<Vec<StoreConnection>, String> {
        let rows: Vec<(
            String, String, Option<String>, String, String,
            String, String, String, String,
            String, String, String,
            String, String, String,
            Option<String>, String, Option<String>, String,
            bool, String,
            bool, bool, bool, bool,
            String, String,
            bool, String, Option<String>, String,
            serde_json::Value, String, String,
        )> = sqlx::query_as(
            r#"SELECT id, brand_id, organization_id, platform_name, platform_version,
                      store_url, store_domain, store_name, display_name,
                      api_key, api_secret_hash, api_secret_salt,
                      access_token_hash, access_token_salt, webhook_secret,
                      external_integration_id, public_key, webhook_url, default_purpose,
                      auto_create_customer, customer_tag,
                      whatsapp_enabled, email_enabled, sms_enabled, in_app_enabled,
                      default_channel, priority_order,
                      is_active, connection_status, last_status_check, last_error,
                      metadata, created_at, updated_at
               FROM store_connections
               WHERE brand_id = $1
               ORDER BY created_at DESC"#
        )
        .bind(brand_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch connections: {}", e))?;

        rows.into_iter().map(Self::row_to_connection).collect()
    }

    /// Update store connection.
    pub async fn update_connection(
        pool: &DbPool,
        connection_id: &str,
        req: &UpdateStoreConnectionRequest,
    ) -> Result<StoreConnection, String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut sets = Vec::new();
        let mut binds: Vec<Box<dyn sqlx::types::Type<sqlx::Postgres> + sqlx::Encode<'_, sqlx::Postgres>>> = Vec::new();
        let mut param_count = 1;

        // Build dynamic SET clause
        let mut parts = Vec::new();
        
        if let Some(ref name) = req.store_name {
            parts.push(format!("store_name = ${}", param_count));
            binds.push(Box::new(name.clone()));
            param_count += 1;
        }
        if let Some(ref name) = req.display_name {
            parts.push(format!("display_name = ${}", param_count));
            binds.push(Box::new(name.clone()));
            param_count += 1;
        }
        if let Some(ref key) = req.api_key {
            parts.push(format!("api_key = ${}", param_count));
            binds.push(Box::new(key.clone()));
            param_count += 1;
        }
        if let Some(ref secret) = req.api_secret {
            let salt = rand::RngCore::next_bytes(&mut rand::thread_rng(), &mut [0u8; 32]);
            let salt_hex = hex::encode(&salt);
            let mut mac = HmacSha256::new_from_slice(&salt)
                .map_err(|e| format!("Failed to create HMAC: {}", e))?;
            mac.update(secret.as_bytes());
            let hash = hex::encode(mac.finalize().into_bytes());
            parts.push(format!("api_secret_hash = ${}, api_secret_salt = ${}", param_count, param_count + 1));
            binds.push(Box::new(hash));
            binds.push(Box::new(salt_hex));
            param_count += 2;
        }
        if let Some(ref token) = req.access_token {
            let salt = rand::RngCore::next_bytes(&mut rand::thread_rng(), &mut [0u8; 32]);
            let salt_hex = hex::encode(&salt);
            let mut mac = HmacSha256::new_from_slice(&salt)
                .map_err(|e| format!("Failed to create HMAC: {}", e))?;
            mac.update(token.as_bytes());
            let hash = hex::encode(mac.finalize().into_bytes());
            parts.push(format!("access_token_hash = ${}, access_token_salt = ${}", param_count, param_count + 1));
            binds.push(Box::new(hash));
            binds.push(Box::new(salt_hex));
            param_count += 2;
        }
        if let Some(ref secret) = req.webhook_secret {
            parts.push(format!("webhook_secret = ${}", param_count));
            binds.push(Box::new(secret.clone()));
            param_count += 1;
        }
        if let Some(ref purpose) = req.default_purpose {
            parts.push(format!("default_purpose = ${}", param_count));
            binds.push(Box::new(purpose.clone()));
            param_count += 1;
        }
        if let Some(auto) = req.auto_create_customer {
            parts.push(format!("auto_create_customer = ${}", param_count));
            binds.push(Box::new(auto));
            param_count += 1;
        }
        if let Some(enabled) = req.whatsapp_enabled {
            parts.push(format!("whatsapp_enabled = ${}", param_count));
            binds.push(Box::new(enabled));
            param_count += 1;
        }
        if let Some(enabled) = req.email_enabled {
            parts.push(format!("email_enabled = ${}", param_count));
            binds.push(Box::new(enabled));
            param_count += 1;
        }
        if let Some(enabled) = req.sms_enabled {
            parts.push(format!("sms_enabled = ${}", param_count));
            binds.push(Box::new(enabled));
            param_count += 1;
        }
        if let Some(enabled) = req.in_app_enabled {
            parts.push(format!("in_app_enabled = ${}", param_count));
            binds.push(Box::new(enabled));
            param_count += 1;
        }
        if let Some(ref channel) = req.default_channel {
            parts.push(format!("default_channel = ${}", param_count));
            binds.push(Box::new(channel.clone()));
            param_count += 1;
        }
        if let Some(ref order) = req.priority_order {
            let order_str = format!("ARRAY[{}]", order.iter()
                .map(|s| format!("'{}'", s.replace("'", "''")))
                .collect::<Vec<_>>().join(", "));
            parts.push(format!("priority_order = ${}", param_count));
            binds.push(Box::new(order_str));
            param_count += 1;
        }
        if let Some(active) = req.is_active {
            parts.push(format!("is_active = ${}", param_count));
            binds.push(Box::new(active));
            param_count += 1;
        }
        
        // Always update updated_at
        parts.push(format!("updated_at = NOW()"));

        if parts.is_empty() {
            return Self::get_by_id(pool, connection_id).await;
        }

        // Use a simpler approach: fetch current, then update with all fields
        let current = Self::get_by_id(pool, connection_id).await?;
        
        let updated = StoreConnection {
            store_name: req.store_name.clone().unwrap_or(current.store_name),
            display_name: req.display_name.clone().unwrap_or(current.display_name),
            default_purpose: req.default_purpose.clone().unwrap_or(current.default_purpose),
            auto_create_customer: req.auto_create_customer.unwrap_or(current.auto_create_customer),
            whatsapp_enabled: req.whatsapp_enabled.unwrap_or(current.whatsapp_enabled),
            email_enabled: req.email_enabled.unwrap_or(current.email_enabled),
            sms_enabled: req.sms_enabled.unwrap_or(current.sms_enabled),
            in_app_enabled: req.in_app_enabled.unwrap_or(current.in_app_enabled),
            default_channel: req.default_channel.clone().unwrap_or(current.default_channel),
            priority_order: req.priority_order.clone().unwrap_or(current.priority_order),
            is_active: req.is_active.unwrap_or(current.is_active),
            ..current
        };

        // Update with all fields
        let priority_order_str = format!("ARRAY[{}]", updated.priority_order.iter()
            .map(|s| format!("'{}'", s.replace("'", "''")))
            .collect::<Vec<_>>().join(", "));

        sqlx::query(
            r#"UPDATE store_connections SET
                store_name = $2, display_name = $3, default_purpose = $4,
                auto_create_customer = $5,
                whatsapp_enabled = $6, email_enabled = $7, sms_enabled = $8, in_app_enabled = $9,
                default_channel = $10, priority_order = $11,
                is_active = $12, updated_at = NOW()
               WHERE id = $1"#
        )
        .bind(&updated.id)
        .bind(&updated.store_name)
        .bind(&updated.display_name)
        .bind(&updated.default_purpose)
        .bind(&updated.auto_create_customer)
        .bind(&updated.whatsapp_enabled)
        .bind(&updated.email_enabled)
        .bind(&updated.sms_enabled)
        .bind(&updated.in_app_enabled)
        .bind(&updated.default_channel)
        .bind::<&str>(&priority_order_str)
        .bind(&updated.is_active)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update store connection: {}", e))?;

        info!("Updated store connection {}", connection_id);
        Ok(updated)
    }

    /// Deactivate a store connection.
    pub async fn deactivate_connection(
        pool: &DbPool,
        connection_id: &str,
    ) -> Result<(), String> {
        sqlx::query(
            r#"UPDATE store_connections SET is_active = false, connection_status = 'disconnected', updated_at = NOW()
               WHERE id = $1"#
        )
        .bind(connection_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to deactivate connection: {}", e))?;

        info!("Deactivated store connection {}", connection_id);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Connection status monitoring
    // -----------------------------------------------------------------------

    /// Get connection status for a store.
    pub async fn get_connection_status(
        pool: &DbPool,
        connection_id: &str,
    ) -> Result<StoreConnectionStatus, String> {
        let row: Option<(
            String, String, String,
            String, Option<String>, i64, f64,
            bool, Option<String>, String, i64,
            bool, Option<String>, i64,
            i64, Option<String>, String, i64,
            Option<String>, Option<String>,
        )> = sqlx::query_as(
            r#"SELECT id, store_connection_id, brand_id,
                      heartbeat_status, last_heartbeat_at, response_time_ms, uptime_percentage,
                      api_reachable, last_api_check_at, api_version, api_rate_limit_remaining,
                      webhook_reachable, last_webhook_check_at, webhook_events_received,
                      error_count_24h, last_error_at, last_error_message, consecutive_errors,
                      last_full_check_at, next_check_at
               FROM store_connection_status
               WHERE store_connection_id = $1
               LIMIT 1"#
        )
        .bind(connection_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch connection status: {}", e))?;

        match row {
            Some(r) => Ok(StoreConnectionStatus {
                id: r.0,
                store_connection_id: r.1,
                brand_id: r.2,
                heartbeat_status: r.3,
                last_heartbeat_at: r.4,
                response_time_ms: r.5,
                uptime_percentage: r.6,
                api_reachable: r.7,
                last_api_check_at: r.8,
                api_version: r.9,
                api_rate_limit_remaining: r.10,
                webhook_reachable: r.11,
                last_webhook_check_at: r.12,
                webhook_events_received: r.13,
                error_count_24h: r.14,
                last_error_message: r.16,
                consecutive_errors: r.17,
                last_full_check_at: r.18,
                next_check_at: r.19,
            }),
            None => Err("Connection status not found".to_string()),
        }
    }

    /// Run a health check on a store connection.
    pub async fn run_health_check(
        pool: &DbPool,
        connection_id: &str,
    ) -> Result<StoreConnectionStatus, String> {
        let conn = Self::get_by_id(pool, connection_id).await?;
        
        let mut api_reachable = false;
        let mut webhook_reachable = false;
        let mut response_time_ms = 0;
        let mut heartbeat_status = "unhealthy".to_string();
        let mut last_error_message = String::new();

        // Check if the store URL is reachable
        let start = std::time::Instant::now();
        match reqwest::get(&conn.store_url).await {
            Ok(resp) => {
                response_time_ms = start.elapsed().as_millis() as i64;
                if resp.status().is_success() {
                    api_reachable = true;
                    heartbeat_status = "healthy".to_string();
                } else {
                    heartbeat_status = "degraded".to_string();
                    last_error_message = format!("HTTP {}", resp.status());
                }
            }
            Err(e) => {
                last_error_message = format!("Connection failed: {}", e);
                warn!("Health check failed for store {}: {}", connection_id, e);
            }
        }

        // Update status record
        let (status_id,) = sqlx::query_as(
            r#"SELECT id FROM store_connection_status WHERE store_connection_id = $1 LIMIT 1"#
        )
        .bind(connection_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch status: {}", e))?
        .unwrap_or_default();

        let status_id = status_id.unwrap_or_else(|| format!("status_{}", uuid::Uuid::new_v4()));

        if status_id == "status_" {
            // Create new status record
            sqlx::query(
                r#"INSERT INTO store_connection_status
                    (id, store_connection_id, brand_id, heartbeat_status, last_heartbeat_at,
                     response_time_ms, api_reachable, last_api_check_at, webhook_reachable,
                     last_error_message, last_full_check_at)
                   VALUES ($1, $2, $3, $4, NOW(), $5, $6, NOW(), $7, $8, NOW())
                   ON CONFLICT (store_connection_id) DO UPDATE SET
                    heartbeat_status = EXCLUDED.heartbeat_status,
                    last_heartbeat_at = NOW(),
                    response_time_ms = EXCLUDED.response_time_ms,
                    api_reachable = EXCLUDED.api_reachable,
                    last_api_check_at = NOW(),
                    webhook_reachable = EXCLUDED.webhook_reachable,
                    last_error_message = EXCLUDED.last_error_message,
                    last_full_check_at = NOW(),
                    updated_at = NOW()"#
            )
            .bind(&status_id)
            .bind(&connection_id)
            .bind(&conn.brand_id)
            .bind(&heartbeat_status)
            .bind(&response_time_ms)
            .bind(&api_reachable)
            .bind(&webhook_reachable)
            .bind(&last_error_message)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update status: {}", e))?;
        } else {
            sqlx::query(
                r#"UPDATE store_connection_status SET
                    heartbeat_status = $2, last_heartbeat_at = NOW(),
                    response_time_ms = $3, api_reachable = $4, last_api_check_at = NOW(),
                    webhook_reachable = $5, last_error_message = $6,
                    last_full_check_at = NOW(), updated_at = NOW()
                   WHERE id = $1"#
            )
            .bind(&status_id)
            .bind(&heartbeat_status)
            .bind(&response_time_ms)
            .bind(&api_reachable)
            .bind(&webhook_reachable)
            .bind(&last_error_message)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update status: {}", e))?;
        }

        // Update connection status
        let conn_status = if heartbeat_status == "healthy" {
            "connected"
        } else if heartbeat_status == "degraded" {
            "error"
        } else {
            "disconnected"
        };

        sqlx::query(
            r#"UPDATE store_connections SET
                connection_status = $2, last_status_check = NOW(), last_error = $3, updated_at = NOW()
               WHERE id = $1"#
        )
        .bind(&connection_id)
        .bind(conn_status)
        .bind(&last_error_message)
        .execute(pool)
        .await
        .ok();

        info!("Health check for store {}: {} ({}ms)", connection_id, heartbeat_status, response_time_ms);

        Self::get_connection_status(pool, connection_id).await
    }

    // -----------------------------------------------------------------------
    // UI Compatibility checks
    // -----------------------------------------------------------------------

    /// Submit a UI compatibility report from the widget.
    pub async fn submit_ui_compatibility_report(
        pool: &DbPool,
        brand_id: &str,
        report: &UiCompatibilityReport,
    ) -> Result<UiCompatibilityResult, String> {
        let conn = Self::get_by_id(pool, &report.store_connection_id).await?;
        
        // Calculate score based on checks
        let mut score = 0;
        let mut max_score = 0;
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Input boxes check (20 points)
        max_score += 20;
        let input_boxes_count = report.input_boxes_count.unwrap_or(0);
        let expected_boxes_count = report.expected_boxes_count.unwrap_or(12);
        let input_boxes_match = input_boxes_count == expected_boxes_count;
        if input_boxes_match {
            score += 20;
        } else {
            issues.push(format!("Input boxes mismatch: found {}, expected {}", input_boxes_count, expected_boxes_count));
            recommendations.push("Ensure the number of OTP input boxes matches the OTP code length".to_string());
        }

        // Paste support (20 points)
        max_score += 20;
        let paste_supported = report.paste_supported.unwrap_or(false);
        if paste_supported {
            score += 20;
        } else {
            issues.push("Paste functionality not detected".to_string());
            recommendations.push("Add paste event handler to OTP input field(s)".to_string());
        }

        // Individual input paste (10 points)
        max_score += 10;
        let individual_input_paste = report.individual_input_paste.unwrap_or(false);
        if individual_input_paste {
            score += 10;
        } else {
            recommendations.push("Add paste support for individual input boxes".to_string());
        }

        // Accessibility (15 points)
        max_score += 15;
        let aria_labels_present = report.aria_labels_present.unwrap_or(false);
        let keyboard_navigation = report.keyboard_navigation.unwrap_or(false);
        if aria_labels_present {
            score += 5;
        } else {
            issues.push("ARIA labels not found on OTP inputs".to_string());
            recommendations.push("Add aria-label attributes to OTP input fields".to_string());
        }
        if keyboard_navigation {
            score += 10;
        } else {
            issues.push("Keyboard navigation not working".to_string());
            recommendations.push("Ensure OTP inputs are keyboard navigable with Tab key".to_string());
        }

        // Form validation (10 points)
        max_score += 10;
        let form_validation_present = report.form_validation_present.unwrap_or(false);
        let error_messages_visible = report.error_messages_visible.unwrap_or(false);
        if form_validation_present {
            score += 5;
        }
        if error_messages_visible {
            score += 5;
        } else {
            recommendations.push("Add visible error messages for invalid OTP".to_string());
        }

        // Timer and resend (10 points)
        max_score += 10;
        let countdown_timer_present = report.countdown_timer_present.unwrap_or(false);
        let resend_button_present = report.resend_button_present.unwrap_or(false);
        if countdown_timer_present {
            score += 5;
        }
        if resend_button_present {
            score += 5;
        } else {
            recommendations.push("Add a resend OTP button with cooldown timer".to_string());
        }

        // Security (10 points)
        max_score += 10;
        let https_enforced = report.https_enforced.unwrap_or(false);
        let csrf_protection_present = report.csrf_protection_present.unwrap_or(false);
        if https_enforced {
            score += 5;
        } else {
            issues.push("HTTPS not enforced".to_string());
            recommendations.push("Enforce HTTPS for all OTP-related pages".to_string());
        }
        if csrf_protection_present {
            score += 5;
        }

        // Mobile (5 points)
        max_score += 5;
        let mobile_responsive = report.mobile_responsive.unwrap_or(false);
        let mobile_touch_targets_ok = report.mobile_touch_targets_ok.unwrap_or(false);
        if mobile_responsive && mobile_touch_targets_ok {
            score += 5;
        } else {
            recommendations.push("Ensure OTP inputs are mobile-friendly with adequate touch targets".to_string());
        }

        // Calculate percentage
        let check_score = if max_score > 0 { (score * 100) / max_score } else { 0 };
        let overall_compatible = check_score >= 70;
        
        let check_status = if check_score >= 90 {
            "passed"
        } else if check_score >= 70 {
            "partial"
        } else {
            "failed"
        }.to_string();

        // Merge with user-provided issues
        if let Some(user_issues) = &report.issues_found {
            for issue in user_issues {
                if !issues.contains(issue) {
                    issues.push(issue.clone());
                }
            }
        }

        // Store the result
        let issues_array = format!("ARRAY[{}]", issues.iter()
            .map(|s| format!("'{}'", s.replace("'", "''")))
            .collect::<Vec<_>>().join(", "));
        let recommendations_array = format!("ARRAY[{}]", recommendations.iter()
            .map(|s| format!("'{}'", s.replace("'", "''")))
            .collect::<Vec<_>>().join(", "));

        sqlx::query(
            r#"INSERT INTO store_ui_compatibility
                (store_connection_id, brand_id, check_status, check_score,
                 input_boxes_count, expected_boxes_count, input_boxes_match,
                 paste_supported, paste_event_handler_found,
                 individual_input_paste,
                 aria_labels_present, keyboard_navigation,
                 form_validation_present, error_messages_visible,
                 countdown_timer_present, resend_button_present,
                 https_enforced, csrf_protection_present,
                 mobile_responsive, mobile_touch_targets_ok,
                 overall_compatible, issues_found, recommendations,
                 check_method, user_agent, page_url)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)"#
        )
        .bind(&report.store_connection_id)
        .bind(brand_id)
        .bind(&check_status)
        .bind(&check_score)
        .bind(&input_boxes_count)
        .bind(&expected_boxes_count)
        .bind(&input_boxes_match)
        .bind(&paste_supported)
        .bind(&report.paste_event_handler_found.unwrap_or(false))
        .bind(&individual_input_paste)
        .bind(&aria_labels_present)
        .bind(&keyboard_navigation)
        .bind(&form_validation_present)
        .bind(&error_messages_visible)
        .bind(&countdown_timer_present)
        .bind(&resend_button_present)
        .bind(&https_enforced)
        .bind(&csrf_protection_present)
        .bind(&mobile_responsive)
        .bind(&mobile_touch_targets_ok)
        .bind(&overall_compatible)
        .bind::<&str>(&issues_array)
        .bind::<&str>(&recommendations_array)
        .bind("widget_report")
        .bind(report.user_agent.as_deref().unwrap_or(""))
        .bind(&report.page_url)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to store UI compatibility report: {}", e))?;

        // Update connection status based on compatibility
        if !overall_compatible {
            sqlx::query(
                r#"UPDATE store_connections SET
                    connection_status = 'ui_incompatible', updated_at = NOW()
                   WHERE id = $1"#
            )
            .bind(&report.store_connection_id)
            .execute(pool)
            .await
            .ok();
        }

        info!("UI compatibility report for store {}: score={}/100, status={}", 
              report.store_connection_id, check_score, check_status);

        Ok(UiCompatibilityResult {
            id: format!("ui_{}", uuid::Uuid::new_v4()),
            store_connection_id: report.store_connection_id.clone(),
            brand_id: brand_id.to_string(),
            check_status,
            check_score,
            checked_at: chrono::Utc::now().to_rfc3339(),
            input_boxes_count,
            expected_boxes_count,
            input_boxes_match,
            input_boxes_error: if input_boxes_match { "".to_string() } else { format!("Found {} boxes, expected {}", input_boxes_count, expected_boxes_count) },
            paste_supported,
            paste_event_handler_found: report.paste_event_handler_found.unwrap_or(false),
            paste_error: if paste_supported { "".to_string() } else { "Paste not supported".to_string() },
            individual_input_paste,
            individual_input_paste_error: if individual_input_paste { "".to_string() } else { "Individual input paste not detected".to_string() },
            aria_labels_present,
            keyboard_navigation,
            screen_reader_compatible: aria_labels_present && keyboard_navigation,
            form_validation_present,
            error_messages_visible,
            countdown_timer_present,
            resend_button_present,
            resend_cooldown_enforced: countdown_timer_present && resend_button_present,
            https_enforced,
            csrf_protection_present,
            rate_limit_ui_feedback: countdown_timer_present,
            mobile_responsive,
            mobile_touch_targets_ok,
            overall_compatible,
            issues_found: issues,
            recommendations,
            check_method: "widget_report".to_string(),
            page_url: report.page_url.clone(),
        })
    }

    /// Get the latest UI compatibility result for a store.
    pub async fn get_latest_ui_compatibility(
        pool: &DbPool,
        connection_id: &str,
    ) -> Result<Option<UiCompatibilityResult>, String> {
        let row: Option<(
            String, String, String, String, i64,
            i64, i64, bool, String,
            bool, bool, String,
            bool, String,
            bool, String, bool,
            bool, bool, bool,
            bool, bool,
            bool, bool, bool,
            bool, bool, bool,
            bool, bool,
            bool, Vec<String>, Vec<String>,
            String, String,
        )> = sqlx::query_as(
            r#"SELECT id, store_connection_id, brand_id, check_status, check_score,
                      input_boxes_count, expected_boxes_count, input_boxes_match, input_boxes_error,
                      paste_supported, paste_event_handler_found, paste_error,
                      individual_input_paste, individual_input_paste_error,
                      aria_labels_present, aria_labels_error, keyboard_navigation,
                      keyboard_navigation_error, screen_reader_compatible,
                      form_validation_present, form_validation_error,
                      error_messages_visible,
                      countdown_timer_present, resend_button_present, resend_cooldown_enforced,
                      https_enforced, csrf_protection_present, rate_limit_ui_feedback,
                      mobile_responsive, mobile_touch_targets_ok,
                      overall_compatible, issues_found, recommendations,
                      check_method, page_url
               FROM store_ui_compatibility
               WHERE store_connection_id = $1
               ORDER BY checked_at DESC
               LIMIT 1"#
        )
        .bind(connection_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch UI compatibility: {}", e))?;

        match row {
            Some(r) => Ok(Some(UiCompatibilityResult {
                id: r.0,
                store_connection_id: r.1,
                brand_id: r.2,
                check_status: r.3,
                check_score: r.4,
                checked_at: chrono::Utc::now().to_rfc3339(),
                input_boxes_count: r.5,
                expected_boxes_count: r.6,
                input_boxes_match: r.7,
                input_boxes_error: r.8,
                paste_supported: r.9,
                paste_event_handler_found: r.10,
                paste_error: r.11,
                individual_input_paste: r.12,
                individual_input_paste_error: r.13,
                aria_labels_present: r.14,
                keyboard_navigation: r.16,
                screen_reader_compatible: r.18,
                form_validation_present: r.19,
                error_messages_visible: r.22,
                countdown_timer_present: r.23,
                resend_button_present: r.24,
                resend_cooldown_enforced: r.25,
                https_enforced: r.26,
                csrf_protection_present: r.27,
                rate_limit_ui_feedback: r.28,
                mobile_responsive: r.29,
                mobile_touch_targets_ok: r.30,
                overall_compatible: r.31,
                issues_found: r.32,
                recommendations: r.33,
                check_method: r.34,
                page_url: r.35,
            })),
            None => Ok(None),
        }
    }

    // -----------------------------------------------------------------------
    // Brand OTP Isolation
    // -----------------------------------------------------------------------

    /// Ensure brand OTP isolation record exists.
    async fn ensure_brand_isolation(
        pool: &DbPool,
        brand_id: &str,
        organization_id: &Option<String>,
    ) -> Result<(), String> {
        // Generate unique prefix and namespace for this brand
        let otp_prefix = format!("BR{}", nanoid::nanoid!(6));
        let otp_namespace = format!("ns_{}", nanoid::nanoid!(8));

        sqlx::query(
            r#"INSERT INTO brand_otp_isolation
                (brand_id, organization_id, otp_prefix, otp_namespace)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (brand_id) DO NOTHING"#
        )
        .bind(brand_id)
        .bind(organization_id)
        .bind(&otp_prefix)
        .bind(&otp_namespace)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to ensure brand isolation: {}", e))?;

        Ok(())
    }

    /// Get brand OTP isolation config.
    pub async fn get_brand_isolation(
        pool: &DbPool,
        brand_id: &str,
    ) -> Result<BrandOtpIsolation, String> {
        // Ensure it exists
        Self::ensure_brand_isolation(pool, brand_id, &None).await.ok();

        let row: Option<(
            String, String, Option<String>,
            bool, String, String, String,
            bool, bool, bool,
            i64, i64, i64, i64,
            bool, bool,
            i64, i64, i64, Option<String>,
            serde_json::Value, String, String,
        )> = sqlx::query_as(
            r#"SELECT id, brand_id, organization_id,
                      isolation_enabled, isolation_mode, otp_prefix, otp_namespace,
                      isolated_whatsapp, isolated_email, isolated_sms,
                      rate_limit_per_minute, rate_limit_per_hour, cooldown_seconds, lockout_seconds,
                      block_cross_brand_otp, block_shared_recipient,
                      total_otp_generated, total_otp_verified, total_otp_failed, last_otp_generated_at,
                      metadata, created_at, updated_at
               FROM brand_otp_isolation
               WHERE brand_id = $1
               LIMIT 1"#
        )
        .bind(brand_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch brand isolation: {}", e))?;

        match row {
            Some(r) => Ok(BrandOtpIsolation {
                id: r.0,
                brand_id: r.1,
                organization_id: r.2,
                isolation_enabled: r.3,
                isolation_mode: r.4,
                otp_prefix: r.5,
                otp_namespace: r.6,
                isolated_whatsapp: r.7,
                isolated_email: r.8,
                isolated_sms: r.9,
                rate_limit_per_minute: r.10,
                rate_limit_per_hour: r.11,
                cooldown_seconds: r.12,
                lockout_seconds: r.13,
                block_cross_brand_otp: r.14,
                block_shared_recipient: r.15,
                total_otp_generated: r.16,
                total_otp_verified: r.17,
                total_otp_failed: r.18,
                last_otp_generated_at: r.19,
                metadata: r.20,
                created_at: r.21,
                updated_at: r.22,
            }),
            None => Err("Brand isolation config not found".to_string()),
        }
    }

    /// Update brand OTP isolation settings.
    pub async fn update_brand_isolation(
        pool: &DbPool,
        brand_id: &str,
        isolation_enabled: Option<bool>,
        isolation_mode: Option<String>,
        rate_limit_per_minute: Option<i64>,
        rate_limit_per_hour: Option<i64>,
        cooldown_seconds: Option<i64>,
        lockout_seconds: Option<i64>,
    ) -> Result<BrandOtpIsolation, String> {
        // Ensure it exists first
        Self::ensure_brand_isolation(pool, brand_id, &None).await?;

        let mut parts = Vec::new();
        let mut param_count = 1;
        let mut binds: Vec<String> = Vec::new();

        if let Some(enabled) = isolation_enabled {
            parts.push(format!("isolation_enabled = ${}", param_count));
            binds.push(enabled.to_string());
            param_count += 1;
        }
        if let Some(ref mode) = isolation_mode {
            parts.push(format!("isolation_mode = ${}", param_count));
            binds.push(mode.clone());
            param_count += 1;
        }
        if let Some(limit) = rate_limit_per_minute {
            parts.push(format!("rate_limit_per_minute = ${}", param_count));
            binds.push(limit.to_string());
            param_count += 1;
        }
        if let Some(limit) = rate_limit_per_hour {
            parts.push(format!("rate_limit_per_hour = ${}", param_count));
            binds.push(limit.to_string());
            param_count += 1;
        }
        if let Some(cooldown) = cooldown_seconds {
            parts.push(format!("cooldown_seconds = ${}", param_count));
            binds.push(cooldown.to_string());
            param_count += 1;
        }
        if let Some(lockout) = lockout_seconds {
            parts.push(format!("lockout_seconds = ${}", param_count));
            binds.push(lockout.to_string());
            param_count += 1;
        }
        
        parts.push(format!("updated_at = NOW()"));

        // Use a simpler approach: fetch, modify, update all
        let mut current = Self::get_brand_isolation(pool, brand_id).await?;
        
        if let Some(enabled) = isolation_enabled {
            current.isolation_enabled = enabled;
        }
        if let Some(ref mode) = isolation_mode {
            current.isolation_mode = mode.clone();
        }
        if let Some(limit) = rate_limit_per_minute {
            current.rate_limit_per_minute = limit;
        }
        if let Some(limit) = rate_limit_per_hour {
            current.rate_limit_per_hour = limit;
        }
        if let Some(cooldown) = cooldown_seconds {
            current.cooldown_seconds = cooldown;
        }
        if let Some(lockout) = lockout_seconds {
            current.lockout_seconds = lockout;
        }

        sqlx::query(
            r#"UPDATE brand_otp_isolation SET
                isolation_enabled = $2, isolation_mode = $3,
                rate_limit_per_minute = $4, rate_limit_per_hour = $5,
                cooldown_seconds = $6, lockout_seconds = $7,
                updated_at = NOW()
               WHERE brand_id = $1"#
        )
        .bind(&current.brand_id)
        .bind(&current.isolation_enabled)
        .bind(&current.isolation_mode)
        .bind(&current.rate_limit_per_minute)
        .bind(&current.rate_limit_per_hour)
        .bind(&current.cooldown_seconds)
        .bind(&current.lockout_seconds)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update brand isolation: {}", e))?;

        info!("Updated brand isolation for {}", brand_id);
        Ok(current)
    }

    /// Verify that an OTP belongs to the correct brand (cross-brand protection).
    pub async fn verify_brand_otp_isolation(
        pool: &DbPool,
        otp_id: &str,
        expected_brand_id: &str,
    ) -> Result<bool, String> {
        // Check the OTP record's brand_id
        let (otp_brand_id,): (String,) = sqlx::query_as(
            r#"SELECT brand_id FROM otp_records WHERE id = $1 LIMIT 1"#
        )
        .bind(otp_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check OTP brand: {}", e))?
        .ok_or("OTP not found".to_string())?;

        // Check brand isolation settings
        let isolation = Self::get_brand_isolation(pool, expected_brand_id).await.ok();
        
        if let Some(iso) = &isolation {
            if iso.block_cross_brand_otp && otp_brand_id != expected_brand_id {
                warn!("Cross-brand OTP verification blocked: OTP {} belongs to brand {}, requested by brand {}",
                      otp_id, otp_brand_id, expected_brand_id);
                return Ok(false);
            }
        }

        Ok(otp_brand_id == expected_brand_id)
    }

    // -----------------------------------------------------------------------
    // Store account creation (platform-agnostic)
    // -----------------------------------------------------------------------

    /// Initiate account creation for a store customer.
    pub async fn initiate_account_creation(
        pool: &DbPool,
        connection_id: &str,
        req: &StoreAccountCreationRequest,
    ) -> Result<StoreAccountCreationResponse, String> {
        let conn = Self::get_by_id(pool, connection_id).await?;

        if !conn.is_active {
            return Err("Store connection is not active".to_string());
        }

        // Determine recipient and channel
        let recipient = if let Some(ref phone) = req.phone {
            phone.clone()
        } else {
            req.email.clone()
        };

        let recipient_type = if req.phone.is_some() {
            "phone"
        } else {
            "email"
        };

        // Select channel based on store preferences and recipient type
        let channel = Self::select_channel(&conn, recipient_type, req.channel.as_deref());

        // Generate OTP via the OTP engine
        let metadata = HashMap::from([
            ("store_connection_id".to_string(), connection_id.to_string()),
            ("platform".to_string(), conn.platform_name.clone()),
            ("store_domain".to_string(), conn.store_domain.clone()),
        ]);

        let otp_req = GenerateOtpRequest {
            recipient: recipient.clone(),
            recipient_type: Some(recipient_type.to_string()),
            purpose: Some(conn.default_purpose.clone()),
            channel: Some(channel.clone()),
            metadata: Some(metadata),
            ip_address: req.ip_address.clone(),
            user_agent: req.user_agent.clone(),
        };

        let otp_result = OtpEngine::generate_and_send(pool, &conn.brand_id, &otp_req).await;
        
        match otp_result {
            Ok(otp_resp) => {
                // Create pending customer record
                let pending_id = format!("pending_{}", uuid::Uuid::new_v4().as_hyphenated());
                let ttl_seconds = otp_resp.expires_in_seconds;
                let expires_at = chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::seconds(ttl_seconds))
                    .unwrap_or(chrono::Utc::now())
                    .to_rfc3339();

                sqlx::query(
                    r#"INSERT INTO store_pending_customers
                        (id, store_connection_id, brand_id, otp_id, email, phone,
                         first_name, last_name, account_status, expires_at, metadata)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'otp_pending', $9, $10)"#
                )
                .bind(&pending_id)
                .bind(&connection_id)
                .bind(&conn.brand_id)
                .bind(&otp_resp.otp_id)
                .bind(&req.email)
                .bind(req.phone.as_deref().unwrap_or(""))
                .bind(req.first_name.as_deref().unwrap_or(""))
                .bind(req.last_name.as_deref().unwrap_or(""))
                .bind(&expires_at)
                .bind(serde_json::to_value(req.metadata.as_ref().unwrap_or(&serde_json::json!({}))).unwrap_or(serde_json::json!({})))
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to create pending customer: {}", e))?;

                info!("Initiated account creation for store {}: pending_id={}, channel={}",
                      connection_id, pending_id, channel);

                Ok(StoreAccountCreationResponse {
                    success: true,
                    pending_id,
                    otp_id: otp_resp.otp_id,
                    channel,
                    message: format!("OTP sent via {}. Please enter the code to verify your account.", channel),
                    expires_in_seconds: ttl_seconds,
                    error: None,
                })
            }
            Err(e) => {
                warn!("Failed to generate OTP for store {}: {}", connection_id, e);
                Ok(StoreAccountCreationResponse {
                    success: false,
                    pending_id: "".to_string(),
                    otp_id: "".to_string(),
                    channel: channel.clone(),
                    message: "Failed to send OTP".to_string(),
                    expires_in_seconds: 0,
                    error: Some(e),
                })
            }
        }
    }

    /// Verify OTP for store account creation.
    pub async fn verify_account_creation(
        pool: &DbPool,
        req: &StoreOtpVerificationRequest,
    ) -> Result<StoreOtpVerificationResponse, String> {
        // Get pending customer record
        let pending: Option<(
            String, String, String, String, String, String,
            String, String, String, String, i64, String, serde_json::Value,
        )> = sqlx::query_as(
            r#"SELECT id, store_connection_id, brand_id, otp_id, email, phone,
                      first_name, last_name, account_status, expires_at,
                      verification_attempts, metadata
               FROM store_pending_customers
               WHERE id = $1
               LIMIT 1"#
        )
        .bind(&req.pending_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch pending customer: {}", e))?;

        let pending = match pending {
            Some(p) => p,
            None => return Ok(StoreOtpVerificationResponse {
                success: false,
                verified: false,
                message: "Pending session not found".to_string(),
                customer_created: false,
                platform_customer_id: None,
                session_token: None,
                error: Some("Invalid or expired session".to_string()),
            }),
        };

        // Check if already verified
        if pending.8 == "verified" {
            return Ok(StoreOtpVerificationResponse {
                success: true,
                verified: true,
                message: "Already verified".to_string(),
                customer_created: true,
                platform_customer_id: None,
                session_token: None,
                error: None,
            });
        }

        // Check expiration
        let now = chrono::Utc::now();
        let expires_at = chrono::DateTime::parse_from_rfc3339(&pending.9)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(now);
        
        if now > expires_at {
            // Mark as expired
            sqlx::query(
                r#"UPDATE store_pending_customers SET account_status = 'expired' WHERE id = $1"#
            )
            .bind(&req.pending_id)
            .execute(pool)
            .await
            .ok();

            return Ok(StoreOtpVerificationResponse {
                success: false,
                verified: false,
                message: "OTP has expired".to_string(),
                customer_created: false,
                platform_customer_id: None,
                session_token: None,
                error: Some("Session expired".to_string()),
            });
        }

        // Verify brand OTP isolation
        let isolation_ok = Self::verify_brand_otp_isolation(pool, &pending.3, &pending.2).await
            .unwrap_or(true);
        
        if !isolation_ok {
            return Ok(StoreOtpVerificationResponse {
                success: false,
                verified: false,
                message: "OTP verification failed".to_string(),
                customer_created: false,
                platform_customer_id: None,
                session_token: None,
                error: Some("Cross-brand OTP verification blocked".to_string()),
            });
        }

        // Verify OTP via the OTP engine
        let verify_result = OtpEngine::verify(
            pool,
            &pending.3,
            &req.code,
            req.ip_address.as_deref(),
            req.user_agent.as_deref(),
        ).await;

        match verify_result {
            Ok(verify_resp) => {
                if verify_resp.verified {
                    // Mark as verified
                    sqlx::query(
                        r#"UPDATE store_pending_customers SET
                            account_status = 'verified', verified_at = NOW(),
                            verification_attempts = verification_attempts + 1
                           WHERE id = $1"#
                    )
                    .bind(&req.pending_id)
                    .execute(pool)
                    .await
                    .ok();

                    // Update brand isolation counters
                    sqlx::query(
                        r#"UPDATE brand_otp_isolation SET
                            total_otp_verified = total_otp_verified + 1,
                            updated_at = NOW()
                           WHERE brand_id = $1"#
                    )
                    .bind(&pending.2)
                    .execute(pool)
                    .await
                    .ok();

                    info!("Account creation verified for store {}: pending_id={}",
                          pending.1, req.pending_id);

                    Ok(StoreOtpVerificationResponse {
                        success: true,
                        verified: true,
                        message: "Account verified successfully".to_string(),
                        customer_created: true,
                        platform_customer_id: None,
                        session_token: verify_resp.session_token,
                        error: None,
                    })
                } else {
                    // Increment attempts
                    let new_attempts: (i64,) = sqlx::query_as(
                        r#"SELECT verification_attempts + 1 FROM store_pending_customers WHERE id = $1"#
                    )
                    .bind(&req.pending_id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or((pending.10 + 1,));

                    sqlx::query(
                        r#"UPDATE store_pending_customers SET verification_attempts = $2 WHERE id = $1"#
                    )
                    .bind(&req.pending_id)
                    .bind(&new_attempts.0)
                    .execute(pool)
                    .await
                    .ok();

                    // Check if max attempts exceeded
                    let max_attempts: (i64,) = sqlx::query_as(
                        r#"SELECT max_attempts FROM otp_records WHERE id = $1 LIMIT 1"#
                    )
                    .bind(&pending.3)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or((5,));

                    if new_attempts.0 >= max_attempts.0 {
                        sqlx::query(
                            r#"UPDATE store_pending_customers SET account_status = 'failed' WHERE id = $1"#
                        )
                        .bind(&req.pending_id)
                        .execute(pool)
                        .await
                        .ok();
                    }

                    Ok(StoreOtpVerificationResponse {
                        success: false,
                        verified: false,
                        message: "Invalid OTP code".to_string(),
                        customer_created: false,
                        platform_customer_id: None,
                        session_token: None,
                        error: Some(format!("Invalid code. {} attempts remaining.", max_attempts.0 - new_attempts.0)),
                    })
                }
            }
            Err(e) => {
                warn!("OTP verification failed for pending {}: {}", req.pending_id, e);
                Ok(StoreOtpVerificationResponse {
                    success: false,
                    verified: false,
                    message: "Verification failed".to_string(),
                    customer_created: false,
                    platform_customer_id: None,
                    session_token: None,
                    error: Some(e),
                })
            }
        }
    }

    /// Resend OTP for a pending account creation.
    pub async fn resend_otp(
        pool: &DbPool,
        pending_id: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<StoreAccountCreationResponse, String> {
        // Get pending customer
        let pending: Option<(
            String, String, String, String, String, String,
            String, String, String,
        )> = sqlx::query_as(
            r#"SELECT id, store_connection_id, brand_id, otp_id, email, phone,
                      first_name, last_name, account_status
               FROM store_pending_customers
               WHERE id = $1
               LIMIT 1"#
        )
        .bind(pending_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch pending customer: {}", e))?;

        let pending = match pending {
            Some(p) => p,
            None => return Err("Pending session not found".to_string()),
        };

        if pending.8 != "otp_pending" {
            return Err("Cannot resend OTP: session is not in pending state".to_string());
        }

        // Get store connection
        let conn = Self::get_by_id(pool, &pending.1).await?;

        // Determine recipient and channel
        let recipient = if pending.5.is_empty() {
            pending.4.clone()
        } else {
            pending.5.clone()
        };

        let recipient_type = if pending.5.is_empty() {
            "email"
        } else {
            "phone"
        };

        let channel = Self::select_channel(&conn, recipient_type, None);

        // Generate new OTP
        let metadata = HashMap::from([
            ("store_connection_id".to_string(), pending.1.clone()),
            ("platform".to_string(), conn.platform_name.clone()),
            ("store_domain".to_string(), conn.store_domain.clone()),
            ("resend".to_string(), "true".to_string()),
        ]);

        let otp_req = GenerateOtpRequest {
            recipient,
            recipient_type: Some(recipient_type.to_string()),
            purpose: Some(conn.default_purpose.clone()),
            channel: Some(channel.clone()),
            metadata: Some(metadata),
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
        };

        let otp_resp = OtpEngine::generate_and_send(pool, &pending.2, &otp_req).await
            .map_err(|e| format!("Failed to resend OTP: {}", e))?;

        // Update pending customer with new OTP
        sqlx::query(
            r#"UPDATE store_pending_customers SET otp_id = $2 WHERE id = $1"#
        )
        .bind(pending_id)
        .bind(&otp_resp.otp_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update pending customer: {}", e))?;

        info!("Resent OTP for pending {}: new otp_id={}", pending_id, otp_resp.otp_id);

        Ok(StoreAccountCreationResponse {
            success: true,
            pending_id: pending_id.to_string(),
            otp_id: otp_resp.otp_id,
            channel,
            message: format!("New OTP sent via {}", channel),
            expires_in_seconds: otp_resp.expires_in_seconds,
            error: None,
        })
    }

    // -----------------------------------------------------------------------
    // Helper methods
    // -----------------------------------------------------------------------

    /// Fetch a store connection by a given column.
    async fn fetch_connection(
        pool: &DbPool,
        column: &str,
        value: &str,
    ) -> Result<StoreConnection, String> {
        let row: Option<(
            String, String, Option<String>, String, String,
            String, String, String, String,
            String, String, String,
            String, String, String,
            Option<String>, String, Option<String>, String,
            bool, String,
            bool, bool, bool, bool,
            String, String,
            bool, String, Option<String>, String,
            serde_json::Value, String, String,
        )> = sqlx::query_as(
            &format!(
                r#"SELECT id, brand_id, organization_id, platform_name, platform_version,
                          store_url, store_domain, store_name, display_name,
                          api_key, api_secret_hash, api_secret_salt,
                          access_token_hash, access_token_salt, webhook_secret,
                          external_integration_id, public_key, webhook_url, default_purpose,
                          auto_create_customer, customer_tag,
                          whatsapp_enabled, email_enabled, sms_enabled, in_app_enabled,
                          default_channel, priority_order,
                          is_active, connection_status, last_status_check, last_error,
                          metadata, created_at, updated_at
                   FROM store_connections
                   WHERE {} = $1 AND is_active = true
                   LIMIT 1"#,
                column
            )
        )
        .bind(value)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch connection: {}", e))?;

        match row {
            Some(r) => Ok(Self::row_to_connection(r)),
            None => Err("Store connection not found or inactive".to_string()),
        }
    }

    /// Convert a database row to a StoreConnection.
    fn row_to_connection(r: (
        String, String, Option<String>, String, String,
        String, String, String, String,
        String, String, String,
        String, String, String,
        Option<String>, String, Option<String>, String,
        bool, String,
        bool, bool, bool, bool,
        String, String,
        bool, String, Option<String>, String,
        serde_json::Value, String, String,
    )) -> StoreConnection {
        let priority_order: Vec<String> = r.25
            .split(',')
            .map(|s| s.trim().trim_matches('[').trim_matches(']').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect();

        StoreConnection {
            id: r.0,
            brand_id: r.1,
            organization_id: r.2,
            platform_name: r.3,
            platform_version: r.4,
            store_url: r.5,
            store_domain: r.6,
            store_name: r.7,
            display_name: r.8,
            api_key: r.9,
            api_secret_hash: r.10,
            api_secret_salt: r.11,
            access_token_hash: r.12,
            access_token_salt: r.13,
            webhook_secret: r.14,
            external_integration_id: r.15,
            public_key: r.16,
            webhook_url: r.17,
            default_purpose: r.18,
            auto_create_customer: r.19,
            customer_tag: r.20,
            whatsapp_enabled: r.21,
            email_enabled: r.22,
            sms_enabled: r.23,
            in_app_enabled: r.24,
            default_channel: r.25,
            priority_order,
            is_active: r.26,
            connection_status: r.27,
            last_status_check: r.28,
            last_error: r.29,
            metadata: r.30,
            created_at: r.31,
            updated_at: r.32,
        }
    }

    /// Select the best channel based on store preferences and recipient type.
    fn select_channel(conn: &StoreConnection, recipient_type: &str, requested: Option<&str>) -> String {
        // If a specific channel is requested and enabled, use it
        if let Some(channel) = requested {
            match channel {
                "whatsapp" => {
                    if conn.whatsapp_enabled {
                        return "whatsapp".to_string();
                    }
                }
                "email" => {
                    if conn.email_enabled {
                        return "email".to_string();
                    }
                }
                "sms" => {
                    if conn.sms_enabled {
                        return "sms".to_string();
                    }
                }
                "in_app" => {
                    if conn.in_app_enabled {
                        return "in_app".to_string();
                    }
                }
                _ => {}
            }
        }

        // Try priority order
        for channel in &conn.priority_order {
            match channel.as_str() {
                "whatsapp" => {
                    if conn.whatsapp_enabled && recipient_type == "phone" {
                        return "whatsapp".to_string();
                    }
                }
                "email" => {
                    if conn.email_enabled {
                        return "email".to_string();
                    }
                }
                "sms" => {
                    if conn.sms_enabled && recipient_type == "phone" {
                        return "sms".to_string();
                    }
                }
                "in_app" => {
                    if conn.in_app_enabled {
                        return "in_app".to_string();
                    }
                }
                _ => {}
            }
        }

        // Fallback to default channel
        match conn.default_channel.as_str() {
            "whatsapp" => {
                if conn.whatsapp_enabled {
                    return "whatsapp".to_string();
                }
            }
            "email" => {
                if conn.email_enabled {
                    return "email".to_string();
                }
            }
            "sms" => {
                if conn.sms_enabled {
                    return "sms".to_string();
                }
            }
            _ => {}
        }

        // Last resort: first enabled channel
        if conn.whatsapp_enabled {
            return "whatsapp".to_string();
        }
        if conn.email_enabled {
            return "email".to_string();
        }
        if conn.sms_enabled {
            return "sms".to_string();
        }
        if conn.in_app_enabled {
            return "in_app".to_string();
        }

        "whatsapp".to_string()
    }
}
