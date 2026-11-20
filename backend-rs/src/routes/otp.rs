//! OTP Engine API routes — thin gRPC client forwarding to svc-otp-engine.

use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    extract::{Path, Query, State, ConnectInfo},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;

use crate::db::AppState;
use newax_proto::otp_engine::v1::otp_engine_service_client::OtpEngineServiceClient;

async fn client(
    state: &AppState,
) -> Result<OtpEngineServiceClient<tonic::transport::Channel>, (StatusCode, Json<serde_json::Value>)> {
    let url = if state.config.otp_engine_grpc_url.is_empty() {
        "http://127.0.0.1:50099".to_string()
    } else {
        state.config.otp_engine_grpc_url.clone()
    };
    OtpEngineServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("otp_engine gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "otp_engine service unavailable" })),
            )
        })
}

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

#[derive(Debug, Deserialize, Default)]
pub struct UpdateChannelConfigInner {
    pub whatsapp_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub default_channel: Option<String>,
    pub code_length: Option<i64>,
    pub code_charset: Option<String>,
    pub ttl_seconds: Option<i64>,
    pub max_attempts: Option<i64>,
    pub rate_limit_per_minute: Option<i64>,
    pub rate_limit_per_hour: Option<i64>,
    pub cooldown_seconds: Option<i64>,
    pub lockout_seconds: Option<i64>,
    pub template_name: Option<String>,
    pub whatsapp_message_template: Option<String>,
    pub sms_message_template: Option<String>,
}

// ---------------------------------------------------------------------------
// Generate OTP
// ---------------------------------------------------------------------------

pub async fn generate_otp(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<GenerateOtpRequestInner>,
) -> (StatusCode, Json<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let metadata = body
        .metadata
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok())
        .unwrap_or_default();

    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .generate_and_send(newax_proto::otp_engine::v1::GenerateAndSendRequest {
            brand_id,
            recipient: body.recipient,
            recipient_type: body.recipient_type.unwrap_or_default(),
            purpose: body.purpose.unwrap_or_default(),
            channel: body.channel.unwrap_or_default(),
            metadata,
            ip_address: if ip_address.is_empty() { String::new() } else { ip_address },
            user_agent: user_agent.unwrap_or_default(),
        })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
            (
                if success {
                    StatusCode::OK
                } else {
                    StatusCode::TOO_MANY_REQUESTS
                },
                Json(val),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Verify OTP
// ---------------------------------------------------------------------------

pub async fn verify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<VerifyOtpRequestInner>,
) -> (StatusCode, Json<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .verify(newax_proto::otp_engine::v1::VerifyRequest {
            otp_id: body.otp_id,
            code: body.code,
            ip_address: if ip_address.is_empty() { String::new() } else { ip_address },
            user_agent: user_agent.unwrap_or_default(),
        })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            let verified = val.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
            (
                if verified {
                    StatusCode::OK
                } else {
                    StatusCode::BAD_REQUEST
                },
                Json(val),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "verified": false, "error": e.to_string() })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Channel Configuration
// ---------------------------------------------------------------------------

pub async fn get_channel_config(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .get_channel_config(newax_proto::otp_engine::v1::GetChannelConfigRequest { brand_id })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn update_channel_config(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Json(body): Json<UpdateChannelConfigInner>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .update_channel_config(newax_proto::otp_engine::v1::UpdateChannelConfigRequest {
            brand_id,
            whatsapp_enabled: body.whatsapp_enabled.unwrap_or(false).to_string(),
            has_whatsapp_enabled: body.whatsapp_enabled.is_some(),
            email_enabled: body.email_enabled.unwrap_or(false).to_string(),
            has_email_enabled: body.email_enabled.is_some(),
            sms_enabled: body.sms_enabled.unwrap_or(false).to_string(),
            has_sms_enabled: body.sms_enabled.is_some(),
            in_app_enabled: body.in_app_enabled.unwrap_or(false).to_string(),
            has_in_app_enabled: body.in_app_enabled.is_some(),
            default_channel: body.default_channel.clone().unwrap_or_default(),
            has_default_channel: body.default_channel.is_some(),
            code_length: body.code_length.unwrap_or(6).to_string(),
            has_code_length: body.code_length.is_some(),
            code_charset: body.code_charset.clone().unwrap_or_default(),
            has_code_charset: body.code_charset.is_some(),
            ttl_seconds: body.ttl_seconds.unwrap_or(300).to_string(),
            has_ttl_seconds: body.ttl_seconds.is_some(),
            max_attempts: body.max_attempts.unwrap_or(3).to_string(),
            has_max_attempts: body.max_attempts.is_some(),
            rate_limit_per_minute: body.rate_limit_per_minute.unwrap_or(5).to_string(),
            has_rate_limit_per_minute: body.rate_limit_per_minute.is_some(),
            rate_limit_per_hour: body.rate_limit_per_hour.unwrap_or(20).to_string(),
            has_rate_limit_per_hour: body.rate_limit_per_hour.is_some(),
            cooldown_seconds: body.cooldown_seconds.unwrap_or(30).to_string(),
            has_cooldown_seconds: body.cooldown_seconds.is_some(),
            lockout_seconds: body.lockout_seconds.unwrap_or(900).to_string(),
            has_lockout_seconds: body.lockout_seconds.is_some(),
            template_name: body.template_name.clone().unwrap_or_default(),
            has_template_name: body.template_name.is_some(),
            whatsapp_message_template: body.whatsapp_message_template.clone().unwrap_or_default(),
            has_whatsapp_message_template: body.whatsapp_message_template.is_some(),
            sms_message_template: body.sms_message_template.clone().unwrap_or_default(),
            has_sms_message_template: body.sms_message_template.is_some(),
        })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ---------------------------------------------------------------------------
// OTP Records
// ---------------------------------------------------------------------------

pub async fn list_otp_records(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Query(query): Query<OtpRecordsQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = query.limit.unwrap_or(50);

    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .list_records(newax_proto::otp_engine::v1::ListRecordsRequest {
            brand_id,
            limit: limit as i32,
        })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Audit Log
// ---------------------------------------------------------------------------

pub async fn get_audit_log(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Query(query): Query<OtpAuditQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = query.limit.unwrap_or(100);

    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .get_audit_log(newax_proto::otp_engine::v1::GetAuditLogRequest {
            brand_id,
            limit: limit as i32,
        })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

pub async fn get_otp_stats(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json),
    };

    let resp = cli
        .get_stats(newax_proto::otp_engine::v1::GetStatsRequest { brand_id })
        .await;

    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
