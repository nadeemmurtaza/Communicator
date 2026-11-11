//! Shopify Store OTP Integration API Routes
//!
//! These endpoints enable a Shopify store to connect to the Communicator OTP engine.
//! The store owner embeds the Communicator OTP widget on their Shopify account page,
//! and these endpoints handle the OTP generation and verification flow.
//!
//! Flow:
//! 1. POST /api/v1/shopify/connect — Brand admin registers their Shopify store
//! 2. GET  /api/v1/shopify/connections/{brand_id} — List connected stores
//! 3. POST /api/v1/shopify/{store_domain}/account/create — Customer initiates account creation
//! 4. POST /api/v1/shopify/account/verify — Customer verifies OTP
//! 5. POST /api/v1/shopify/account/resend — Resend OTP
//! 6. POST /api/v1/shopify/webhook/{connection_id} — Receive Shopify webhooks
//! 7. PUT  /api/v1/shopify/{connection_id}/channels — Update channel preferences

use crate::db::AppState;
use crate::services::shopify_store_integration::{
    ShopifyStoreIntegrationService, CreateShopifyConnectionRequest,
    ShopifyAccountCreationRequest, ShopifyOtpVerificationRequest,
    UpdateChannelPreferenceRequest,
};
use axum::{
    extract::{Path, State, ConnectInfo, Json},
    http::{HeaderMap, StatusCode},
    response::Json as JsonResponse,
};
use std::net::SocketAddr;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Store connection management (admin endpoints)
// ---------------------------------------------------------------------------

/// POST /api/v1/shopify/connect
/// Register a new Shopify store connection.
///
/// Request body:
/// {
///   "brand_id": "brand_abc123",
///   "store_domain": "mystore.myshopify.com",
///   "store_name": "My Clothing Store",
///   "shopify_api_key": "shpat_...",        // optional: for Admin API access
///   "shopify_api_secret": "shpss_...",     // optional: for Admin API access
///   "shopify_access_token": "shpat_...",   // optional: for auto customer creation
///   "auto_create_customer": true,          // optional: default true
///   "customer_tag": "otp_verified"         // optional: tag for verified customers
/// }
pub async fn create_shopify_connection(
    State(state): State<AppState>,
    Json(body): Json<CreateShopifyConnectionRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ShopifyStoreIntegrationService::create_connection(&state.db, &body).await {
        Ok(resp) => (
            StatusCode::CREATED,
            JsonResponse(serde_json::json!({
                "success": true,
                "id": resp.id,
                "store_domain": resp.store_domain,
                "public_key": resp.public_key,
                "webhook_url": resp.webhook_url,
                "message": resp.message,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// GET /api/v1/shopify/connections/{brand_id}
/// List all Shopify store connections for a brand.
pub async fn list_shopify_connections(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ShopifyStoreIntegrationService::list_by_brand(&state.db, &brand_id).await {
        Ok(connections) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "connections": connections.iter().map(|c| serde_json::json!({
                    "id": c.id,
                    "store_domain": c.store_domain,
                    "store_name": c.store_name,
                    "auto_create_customer": c.auto_create_customer,
                    "customer_tag": c.customer_tag,
                    "is_active": c.is_active,
                    "created_at": c.created_at,
                })).collect::<Vec<_>>(),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// GET /api/v1/shopify/{connection_id}/channels
/// Get channel preferences for a store connection.
pub async fn get_channel_preferences(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let prefs = ShopifyStoreIntegrationService::get_channel_preferences(
        &state.db, &connection_id, "sign_up",
    ).await;

    match prefs {
        Some(pref) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "channel_preferences": {
                    "purpose": pref.purpose,
                    "whatsapp_enabled": pref.whatsapp_enabled,
                    "email_enabled": pref.email_enabled,
                    "sms_enabled": pref.sms_enabled,
                    "in_app_enabled": pref.in_app_enabled,
                    "default_channel": pref.default_channel,
                    "priority_order": pref.priority_order,
                }
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": "Channel preferences not found",
            })),
        ),
    }
}

/// PUT /api/v1/shopify/{connection_id}/channels
/// Update channel preferences for a store connection.
///
/// Request body:
/// {
///   "whatsapp_enabled": true,
///   "email_enabled": true,
///   "sms_enabled": false,
///   "default_channel": "whatsapp",
///   "priority_order": ["whatsapp", "email", "sms"]
/// }
pub async fn update_channel_preferences(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(body): Json<UpdateChannelPreferenceRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ShopifyStoreIntegrationService::update_channel_preferences(
        &state.db, &connection_id, &body,
    ).await {
        Ok(pref) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "channel_preferences": {
                    "purpose": pref.purpose,
                    "whatsapp_enabled": pref.whatsapp_enabled,
                    "email_enabled": pref.email_enabled,
                    "sms_enabled": pref.sms_enabled,
                    "in_app_enabled": pref.in_app_enabled,
                    "default_channel": pref.default_channel,
                    "priority_order": pref.priority_order,
                }
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Account creation flow (store-facing endpoints)
// ---------------------------------------------------------------------------

/// POST /api/v1/shopify/{store_domain}/account/create
/// Initiate account creation for a Shopify store customer.
///
/// This is called by the Shopify store when a customer fills the account creation form.
/// The OTP is sent via the brand admin's preferred channel (WhatsApp, SMS, Email).
///
/// Request body:
/// {
///   "email": "customer@example.com",
///   "phone": "+1234567890",          // optional: for WhatsApp/SMS OTP
///   "first_name": "John",            // optional
///   "last_name": "Doe",              // optional
///   "channel": "whatsapp"            // optional: override default channel
/// }
///
/// Response:
/// {
///   "success": true,
///   "pending_id": "pending_abc123...",
///   "otp_id": "otp_abc123...",
///   "channel": "whatsapp",
///   "message": "OTP sent via WhatsApp. Please enter the code to complete your account creation.",
///   "expires_in_seconds": 300
/// }
pub async fn create_shopify_account(
    State(state): State<AppState>,
    Path(store_domain): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ShopifyAccountCreationRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match ShopifyStoreIntegrationService::initiate_account_creation(
        &state.db, &store_domain, &body, &ip_address, user_agent,
    ).await {
        Ok(resp) => (
            if resp.success { StatusCode::OK } else { StatusCode::BAD_REQUEST },
            JsonResponse(serde_json::json!({
                "success": resp.success,
                "pending_id": resp.pending_id,
                "otp_id": resp.otp_id,
                "channel": resp.channel,
                "message": resp.message,
                "expires_in_seconds": resp.expires_in_seconds,
                "error": resp.error,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

/// POST /api/v1/shopify/account/verify
/// Verify OTP for a Shopify store account creation.
///
/// After the customer enters the OTP code, the store calls this endpoint.
/// On success, the customer account is created in Shopify (if auto_create is enabled).
///
/// Request body:
/// {
///   "pending_id": "pending_abc123...",
///   "code": "A7xK9mP2qR5t"
/// }
///
/// Response:
/// {
///   "success": true,
///   "verified": true,
///   "message": "Account verified and created successfully!",
///   "customer_created": true,
///   "session_token": "otp_session_..."
/// }
pub async fn verify_shopify_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ShopifyOtpVerificationRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match ShopifyStoreIntegrationService::verify_account_creation(
        &state.db, &body, &ip_address, user_agent,
    ).await {
        Ok(resp) => (
            if resp.verified { StatusCode::OK } else { StatusCode::BAD_REQUEST },
            JsonResponse(serde_json::json!({
                "success": resp.success,
                "verified": resp.verified,
                "message": resp.message,
                "customer_created": resp.customer_created,
                "shopify_customer_id": resp.shopify_customer_id,
                "session_token": resp.session_token,
                "error": resp.error,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({
                "success": false,
                "verified": false,
                "error": e,
            })),
        ),
    }
}

/// POST /api/v1/shopify/account/resend
/// Resend OTP for a pending Shopify account creation.
///
/// Request body:
/// {
///   "pending_id": "pending_abc123..."
/// }
#[derive(Debug, Deserialize)]
pub struct ResendOtpRequest {
    pub pending_id: String,
}

pub async fn resend_shopify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ResendOtpRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match ShopifyStoreIntegrationService::resend_otp(
        &state.db, &body.pending_id, &ip_address, user_agent,
    ).await {
        Ok(resp) => (
            if resp.success { StatusCode::OK } else { StatusCode::BAD_REQUEST },
            JsonResponse(serde_json::json!({
                "success": resp.success,
                "pending_id": resp.pending_id,
                "otp_id": resp.otp_id,
                "channel": resp.channel,
                "message": resp.message,
                "expires_in_seconds": resp.expires_in_seconds,
                "error": resp.error,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Shopify webhook receiver
// ---------------------------------------------------------------------------

/// POST /api/v1/shopify/webhook/{connection_id}
/// Receive webhook events from Shopify stores.
///
/// Shopify sends webhooks for events like customers/create, orders/create, etc.
/// The webhook signature is verified using the stored webhook secret.
pub async fn receive_shopify_webhook(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    // Extract Shopify signature
    let signature = headers
        .get("X-Shopify-Hmac-Sha256")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let topic = headers
        .get("X-Shopify-Topic")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let shop_domain = headers
        .get("X-Shopify-Shop-Domain")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let webhook_id = headers
        .get("X-Shopify-Webhook-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Parse payload
    let payload: serde_json::Value = serde_json::from_slice(&body)
        .unwrap_or_else(|_| serde_json::json!({}));

    // Get store connection to verify signature
    let connection = match ShopifyStoreIntegrationService::get_by_id(&state.db, &connection_id).await {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                JsonResponse(serde_json::json!({
                    "success": false,
                    "error": "Store connection not found",
                })),
            );
        }
    };

    // Verify signature
    let verified = if let Some(sig) = &signature {
        ShopifyStoreIntegrationService::verify_shopify_signature(
            &body, sig, &connection.shopify_webhook_secret,
        )
    } else {
        false
    };

    // Log the webhook event
    match ShopifyStoreIntegrationService::log_webhook_event(
        &state.db,
        Some(&connection_id),
        topic,
        shop_domain,
        webhook_id,
        &payload,
        signature.as_deref().unwrap_or(""),
        verified,
    ).await {
        Ok(_) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "message": "Webhook received",
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}
