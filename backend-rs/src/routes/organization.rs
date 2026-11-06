#![allow(dead_code)]

//! Organization profile, BYOK settings, and stats endpoints.
//!
//! Routes:
//!   GET  /organization/profile       - Get organization profile data
//!   PUT  /organization/profile       - Update organization profile
//!   GET  /organization/settings      - Get BYOK settings
//!   PUT  /organization/settings      - Update BYOK settings
//!   GET  /organization/stats         - Get organization statistics
//!   POST /organization/channels/:ch/verify - Verify channel connection

use crate::db::AppState;
use crate::services::integration_settings;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

/// GET /organization/profile - Returns the organization's full profile
pub async fn get_profile(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tk = resolve_tenant(&state);
    
    let settings = match integration_settings::load(&state.db, &tk).await {
        Ok(Some(s)) => s.to_json(),
        Ok(None) => integration_settings::IntegrationSettings::empty(&tk).to_json(),
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load profile: {e}") })),
            ))
        }
    };
    
    Ok(Json(settings))
}

/// PUT /organization/profile - Update organization profile fields
pub async fn update_profile(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tk = resolve_tenant(&state);
    
    // Ensure a row exists
    match integration_settings::load(&state.db, &tk).await {
        Ok(None) => {
            let _ = integration_settings::get_or_create(&state.db, &state.config, &tk).await;
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load settings: {e}") })),
            ))
        }
        _ => {}
    }
    
    let report = match integration_settings::update(&state.db, &tk, &payload).await {
        Ok(r) => r,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to update profile: {e}") })),
            ))
        }
    };
    
    Ok(Json(json!({
        "status": "updated",
        "rowsAffected": report.rows_affected,
        "applied": report.applied,
        "rejected": report.rejected,
    })))
}

/// GET /organization/settings - Returns BYOK settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tk = resolve_tenant(&state);
    
    let settings = match integration_settings::load(&state.db, &tk).await {
        Ok(Some(s)) => s.to_json(),
        Ok(None) => integration_settings::IntegrationSettings::empty(&tk).to_json(),
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load settings: {e}") })),
            ))
        }
    };
    
    Ok(Json(settings))
}

/// PUT /organization/settings - Update BYOK settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tk = resolve_tenant(&state);
    
    // Ensure a row exists
    match integration_settings::load(&state.db, &tk).await {
        Ok(None) => {
            let _ = integration_settings::get_or_create(&state.db, &state.config, &tk).await;
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load settings: {e}") })),
            ))
        }
        _ => {}
    }
    
    let report = match integration_settings::update(&state.db, &tk, &payload).await {
        Ok(r) => r,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to update settings: {e}") })),
            ))
        }
    };
    
    Ok(Json(json!({
        "status": "updated",
        "rowsAffected": report.rows_affected,
        "applied": report.applied,
        "rejected": report.rejected,
    })))
}

/// GET /organization/stats - Returns organization statistics
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tk = resolve_tenant(&state);
    
    let mut stats = json!({});
    
    // Count contacts
    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM contacts WHERE COALESCE(tenant_key, '') = ?"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["totalContacts"] = json!(count);
    }
    
    // Count conversations
    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM conversations WHERE COALESCE(tenant_key, '') = ?"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["totalConversations"] = json!(count);
    }
    
    // Count drafts
    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM drafts"
    )
    .fetch_one(&state.db)
    .await
    {
        stats["totalDrafts"] = json!(count);
    }
    
    // Count knowledge base entries
    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM knowledge_base_entries WHERE COALESCE(tenant_key, '') = ?"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["knowledgeBaseEntries"] = json!(count);
    }
    
    // Count services
    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM services WHERE COALESCE(tenant_key, '') = ?"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["services"] = json!(count);
    }
    
    // Count active inboxes
    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM inboxes WHERE COALESCE(tenant_key, '') = ? AND COALESCE(is_active, 1) = 1"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["activeInboxes"] = json!(count);
    }
    
    Ok(Json(stats))
}

/// POST /organization/channels/:channel/verify - Verify a channel connection
pub async fn verify_channel(
    State(state): State<AppState>,
    Path(channel): Path<String>,
) -> Json<Value> {
    let result = match channel.to_lowercase().as_str() {
        "whatsapp" => verify_whatsapp(&state).await,
        "sms" | "twilio" => verify_twilio(&state).await,
        "telegram" => verify_telegram(&state).await,
        "line" => verify_line(&state).await,
        "ai" | "anthropic" => verify_ai(&state).await,
        _ => json!({ "connected": false, "error": format!("Channel '{}' verification not implemented", channel) }),
    };
    Json(result)
}

async fn verify_whatsapp(state: &AppState) -> Value {
    let token = std::env::var("META_WHATSAPP_ACCESS_TOKEN")
        .unwrap_or_else(|_| state.config.meta_whatsapp_access_token.clone());
    
    if token.trim().is_empty() {
        return json!({ "connected": false, "error": "No WhatsApp access token configured" });
    }
    
    let phone_id = std::env::var("META_WHATSAPP_PHONE_NUMBER_ID")
        .unwrap_or_else(|_| state.config.meta_whatsapp_phone_number_id.clone());
    
    if phone_id.is_empty() {
        return json!({ "connected": false, "error": "No WhatsApp phone number ID configured" });
    }
    
    let fields = "name,verified_name".to_string();
    match state.http_client
        .get(format!("https://graph.facebook.com/v21.0/{}", phone_id))
        .query(&[("access_token", &token), ("fields", &fields)])
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body: Result<Value, _> = resp.json().await;
            json!({
                "connected": true,
                "providerInfo": body.ok(),
                "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
            })
        }
        Ok(resp) => json!({
            "connected": false,
            "error": format!("HTTP {}: {}", resp.status(), resp.status()),
        }),
        Err(e) => json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_twilio(state: &AppState) -> Value {
    let sid = std::env::var("TWILIO_ACCOUNT_SID")
        .unwrap_or_else(|_| state.config.twilio_account_sid.clone());
    let token = std::env::var("TWILIO_AUTH_TOKEN")
        .unwrap_or_else(|_| state.config.twilio_auth_token.clone());
    
    if sid.is_empty() || token.is_empty() {
        return json!({ "connected": false, "error": "Twilio credentials not configured" });
    }
    
    match state.http_client
        .get("https://api.twilio.com/2010-04-01/Accounts.json")
        .basic_auth(&sid, Some(&token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            json!({
                "connected": true,
                "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
            })
        }
        Ok(resp) => json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_telegram(state: &AppState) -> Value {
    let token = std::env::var("TELEGRAM_BOT_TOKEN")
        .unwrap_or_default();
    
    if token.is_empty() {
        return json!({ "connected": false, "error": "Telegram bot token not configured" });
    }
    
    match state.http_client
        .get(format!("https://api.telegram.org/bot{}/getMe", token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            json!({
                "connected": true,
                "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
            })
        }
        Ok(resp) => json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_line(state: &AppState) -> Value {
    let token = std::env::var("LINE_CHANNEL_ACCESS_TOKEN")
        .unwrap_or_default();
    
    if token.is_empty() {
        return json!({ "connected": false, "error": "LINE channel access token not configured" });
    }
    
    match state.http_client
        .get("https://api.line.me/v2/bot/info")
        .bearer_auth(&token)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            json!({
                "connected": true,
                "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
            })
        }
        Ok(resp) => json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_ai(state: &AppState) -> Value {
    use crate::services::claude_client::configuration_report;
    let report = configuration_report(state);
    
    if report.get("configured").and_then(Value::as_bool) == Some(true) {
        json!({
            "connected": true,
            "providerInfo": report,
            "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
        })
    } else {
        json!({
            "connected": false,
            "error": "AI provider not configured",
            "detail": report,
        })
    }
}

fn resolve_tenant(state: &AppState) -> String {
    let default = state.config.default_tenant_key.trim();
    if !default.is_empty() {
        return default.to_string();
    }
    "newax".to_string()
}
