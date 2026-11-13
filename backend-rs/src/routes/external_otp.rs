//! External OTP Integration API Routes
//!
//! These endpoints are designed for external e-commerce platforms (Shopify, custom stores,
//! WooCommerce, etc.) to connect to the Communicator OTP engine.
//!
//! Authentication: All endpoints require the `Authorization: Bearer {secret_key}` header.
//! The secret key is generated when creating an integration and is only shown once.

use crate::db::AppState;
use crate::services::external_otp_integration::{
    ExternalGenerateRequest, ExternalIntegration, ExternalOtpIntegrationService,
    ExternalVerifyRequest, CreateIntegrationRequest,
};
use axum::{
    extract::{Path, Query, State, ConnectInfo, Json},
    http::{HeaderMap, StatusCode},
    response::Json as JsonResponse,
};
use serde::Deserialize;
use std::net::SocketAddr;

// ---------------------------------------------------------------------------
// Authentication middleware helper
// ---------------------------------------------------------------------------

/// Extract and verify the secret key from the Authorization header.
async fn authenticate_integration(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<ExternalIntegration, (StatusCode, JsonResponse<serde_json::Value>)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": "Missing Authorization header",
            })),
        ))?;

    // Extract Bearer token
    let secret_key = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else if auth_header.starts_with("bearer ") {
        &auth_header[7..]
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": "Invalid Authorization format. Use: Bearer {secret_key}",
            })),
        ));
    };

    // Verify the secret key
    match ExternalOtpIntegrationService::verify_secret_key(&state.db, secret_key).await {
        Ok(integration) => Ok(integration),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        )),
    }
}

// ---------------------------------------------------------------------------
// Integration management (admin endpoints — require internal auth)
// ---------------------------------------------------------------------------

/// POST /api/v1/external-otp/integrations
/// Create a new external integration key pair.
/// This is an admin endpoint — the brand admin creates the integration and gets the keys.
pub async fn create_integration(
    State(state): State<AppState>,
    Json(body): Json<CreateIntegrationRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ExternalOtpIntegrationService::create_integration(&state.db, &body).await {
        Ok(resp) => (
            StatusCode::CREATED,
            JsonResponse(serde_json::json!({
                "success": true,
                "id": resp.id,
                "public_key": resp.public_key,
                "secret_key": resp.secret_key,
                "webhook_secret": resp.webhook_secret,
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

/// GET /api/v1/external-otp/integrations/{brand_id}
/// List all integrations for a brand.
pub async fn list_integrations(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ExternalOtpIntegrationService::list_by_brand(&state.db, &brand_id).await {
        Ok(integrations) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "integrations": integrations.iter().map(|i| serde_json::json!({
                    "id": i.id,
                    "platform_name": i.platform_name,
                    "platform_url": i.platform_url,
                    "display_name": i.display_name,
                    "public_key": i.public_key,
                    "webhook_url": i.webhook_url,
                    "allowed_purposes": i.allowed_purposes,
                    "allowed_channels": i.allowed_channels,
                    "is_active": i.is_active,
                    "rate_limit_per_minute": i.rate_limit_per_minute,
                    "rate_limit_per_hour": i.rate_limit_per_hour,
                    "last_used_at": i.last_used_at,
                    "created_at": i.created_at,
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

/// DELETE /api/v1/external-otp/integrations/{integration_id}
/// Deactivate an integration.
pub async fn deactivate_integration(
    State(state): State<AppState>,
    Path(integration_id): Path<String>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ExternalOtpIntegrationService::deactivate(&state.db, &integration_id).await {
        Ok(()) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "message": "Integration deactivated",
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

// ---------------------------------------------------------------------------
// External OTP endpoints (authenticated by secret key)
// ---------------------------------------------------------------------------

/// POST /api/v1/external-otp/generate
/// Generate and send an OTP via the configured channel.
///
/// Authentication: Authorization: Bearer {secret_key}
///
/// Request body:
/// {
///   "recipient": "+1234567890",
///   "recipient_type": "phone",  // optional: "phone" or "email"
///   "purpose": "sign_up",       // optional: defaults to "sign_up"
///   "channel": "whatsapp",      // optional: uses brand default
///   "metadata": {}              // optional: additional context
/// }
pub async fn external_generate_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ExternalGenerateRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    // Authenticate the integration
    let integration = match authenticate_integration(&state, &headers).await {
        Ok(integ) => integ,
        Err(err) => return err,
    };

    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match ExternalOtpIntegrationService::external_generate(
        &state.db, &integration, &body, &ip_address, user_agent,
    ).await {
        Ok(resp) => (
            if resp.success { StatusCode::OK } else { StatusCode::BAD_REQUEST },
            JsonResponse(serde_json::json!({
                "success": resp.success,
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

/// POST /api/v1/external-otp/verify
/// Verify an OTP code.
///
/// Authentication: Authorization: Bearer {secret_key}
///
/// Request body:
/// {
///   "otp_id": "otp_abc123...",
///   "code": "A7xK9mP2qR5t"
/// }
///
/// Response on success:
/// {
///   "success": true,
///   "verified": true,
///   "attempts_remaining": 4,
///   "session_token": "otp_session_..."  // short-lived token for next step
/// }
pub async fn external_verify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ExternalVerifyRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    // Authenticate the integration
    let integration = match authenticate_integration(&state, &headers).await {
        Ok(integ) => integ,
        Err(err) => return err,
    };

    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match ExternalOtpIntegrationService::external_verify(
        &state.db, &integration, &body, &ip_address, user_agent,
    ).await {
        Ok(resp) => (
            if resp.verified { StatusCode::OK } else { StatusCode::BAD_REQUEST },
            JsonResponse(serde_json::json!({
                "success": resp.success,
                "verified": resp.verified,
                "message": resp.message,
                "attempts_remaining": resp.attempts_remaining,
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

// ---------------------------------------------------------------------------
// Integration info endpoint (public key lookup)
// ---------------------------------------------------------------------------

/// GET /api/v1/external-otp/info?public_key=pk_...
/// Get integration info by public key (safe for frontend).
/// Returns only non-sensitive information.
#[derive(Debug, Deserialize)]
pub struct IntegrationInfoQuery {
    pub public_key: String,
}

pub async fn get_integration_info(
    State(state): State<AppState>,
    Query(query): Query<IntegrationInfoQuery>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    match ExternalOtpIntegrationService::get_by_public_key(&state.db, &query.public_key).await {
        Ok(integration) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "success": true,
                "id": integration.id,
                "platform_name": integration.platform_name,
                "display_name": integration.display_name,
                "public_key": integration.public_key,
                "allowed_purposes": integration.allowed_purposes,
                "allowed_channels": integration.allowed_channels,
                "is_active": integration.is_active,
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            JsonResponse(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}
