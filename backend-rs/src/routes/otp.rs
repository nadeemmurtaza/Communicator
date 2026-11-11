//! OTP Engine API routes.
//!
//! Provides endpoints for generating, verifying, and managing OTPs across
//! all channels (WhatsApp, Email, SMS, In-App) with per-brand channel toggles.

use crate::db::AppState;
use crate::services::otp_engine::{
    GenerateOtpRequest, OtpEngine, UpdateChannelConfigRequest, VerifyOtpRequest,
};
use axum::{
    extract::{Path, Query, State, ConnectInfo},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OtpRecordsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct OtpAuditQuery {
    pub limit: Option<i64>,
}

// ---------------------------------------------------------------------------
// Inner request types (without IP/user_agent — extracted from connection)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GenerateOtpRequestInner {
    pub recipient: String,
    pub recipient_type: Option<String>,
    pub purpose: Option<String>,
    pub channel: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequestInner {
    pub otp_id: String,
    pub code: String,
}

// ---------------------------------------------------------------------------
// Generate OTP
// ---------------------------------------------------------------------------

/// POST /api/v1/otp/generate
/// Generate and send an OTP via the configured channel.
pub async fn generate_otp(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<GenerateOtpRequestInner>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Extract IP and user agent for banking-grade binding
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let body = GenerateOtpRequest {
        recipient: body.recipient,
        recipient_type: body.recipient_type,
        purpose: body.purpose,
        channel: body.channel,
        metadata: body.metadata,
        ip_address: Some(ip_address),
        user_agent,
    };

    match OtpEngine::generate_and_send(&state.db, &brand_id, &body).await {
        Ok(resp) => (
            if resp.success { StatusCode::OK } else { StatusCode::TOO_MANY_REQUESTS },
            Json(serde_json::json!({
                "success": resp.success,
                "otp_id": resp.otp_id,
                "channel": resp.channel,
                "message": resp.message,
                "expires_in_seconds": resp.expires_in_seconds,
                "error": resp.error,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Verify OTP
// ---------------------------------------------------------------------------

/// POST /api/v1/otp/verify
/// Verify an OTP code.
pub async fn verify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<VerifyOtpRequestInner>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Extract IP and user agent for banking-grade binding
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let body = VerifyOtpRequest {
        otp_id: body.otp_id,
        code: body.code,
        ip_address: Some(ip_address),
        user_agent,
    };

    match OtpEngine::verify(&state.db, &body).await {
        Ok(resp) => (
            if resp.verified { StatusCode::OK } else { StatusCode::BAD_REQUEST },
            Json(serde_json::json!({
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
            Json(serde_json::json!({
                "success": false,
                "verified": false,
                "error": e,
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Channel Configuration
// ---------------------------------------------------------------------------

/// GET /api/v1/otp/config/{brand_id}
/// Get the OTP channel configuration for a brand.
pub async fn get_channel_config(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match OtpEngine::get_channel_config(&state.db, &brand_id).await {
        Ok(config) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": config.id,
                "brand_id": config.brand_id,
                "whatsapp_enabled": config.whatsapp_enabled,
                "email_enabled": config.email_enabled,
                "sms_enabled": config.sms_enabled,
                "in_app_enabled": config.in_app_enabled,
                "default_channel": config.default_channel,
                "code_length": config.code_length,
                "code_charset": config.code_charset,
                "ttl_seconds": config.ttl_seconds,
                "max_attempts": config.max_attempts,
                "rate_limit_per_minute": config.rate_limit_per_minute,
                "rate_limit_per_hour": config.rate_limit_per_hour,
                "cooldown_seconds": config.cooldown_seconds,
                "lockout_seconds": config.lockout_seconds,
                "template_name": config.template_name,
                "whatsapp_message_template": config.whatsapp_message_template,
                "sms_message_template": config.sms_message_template,
                "created_at": config.created_at,
                "updated_at": config.updated_at,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

/// PUT /api/v1/otp/config/{brand_id}
/// Update the OTP channel configuration for a brand.
/// This is where the user toggles which channels are enabled for OTP delivery.
pub async fn update_channel_config(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Json(body): Json<UpdateChannelConfigRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match OtpEngine::update_channel_config(&state.db, &brand_id, &body).await {
        Ok(config) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "OTP channel configuration updated",
                "id": config.id,
                "brand_id": config.brand_id,
                "whatsapp_enabled": config.whatsapp_enabled,
                "email_enabled": config.email_enabled,
                "sms_enabled": config.sms_enabled,
                "in_app_enabled": config.in_app_enabled,
                "default_channel": config.default_channel,
                "code_length": config.code_length,
                "code_charset": config.code_charset,
                "ttl_seconds": config.ttl_seconds,
                "max_attempts": config.max_attempts,
                "rate_limit_per_minute": config.rate_limit_per_minute,
                "rate_limit_per_hour": config.rate_limit_per_hour,
                "cooldown_seconds": config.cooldown_seconds,
                "lockout_seconds": config.lockout_seconds,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

// ---------------------------------------------------------------------------
// OTP Records
// ---------------------------------------------------------------------------

/// GET /api/v1/otp/records/{brand_id}
/// List recent OTP records for a brand.
pub async fn list_otp_records(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Query(query): Query<OtpRecordsQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = query.limit.unwrap_or(50);
    match OtpEngine::list_records(&state.db, &brand_id, limit).await {
        Ok(records) => {
            let list: Vec<serde_json::Value> = records.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "purpose": r.purpose,
                    "channel": r.channel,
                    "recipient": r.recipient,
                    "recipient_type": r.recipient_type,
                    "status": r.status,
                    "attempts": r.attempts,
                    "max_attempts": r.max_attempts,
                    "generated_at": r.generated_at,
                    "expires_at": r.expires_at,
                    "sent_at": r.sent_at,
                    "verified_at": r.verified_at,
                })
            }).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "records": list, "count": list.len() })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Audit Log
// ---------------------------------------------------------------------------

/// GET /api/v1/otp/audit/{brand_id}
/// Get the OTP audit log for a brand.
pub async fn get_audit_log(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Query(query): Query<OtpAuditQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = query.limit.unwrap_or(100);
    match OtpEngine::get_audit_log(&state.db, &brand_id, limit).await {
        Ok(logs) => {
            let list: Vec<serde_json::Value> = logs.iter().map(|l| {
                serde_json::json!({
                    "id": l.id,
                    "event_type": l.event_type,
                    "otp_id": l.otp_id,
                    "channel": l.channel,
                    "recipient": l.recipient,
                    "ip_address": l.ip_address,
                    "details": l.details,
                    "created_at": l.created_at,
                })
            }).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "audit_log": list, "count": list.len() })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

/// GET /api/v1/otp/stats/{brand_id}
/// Get OTP statistics for a brand.
pub async fn get_otp_stats(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match OtpEngine::get_stats(&state.db, &brand_id).await {
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}
