//! Organization — standalone crate extracted from backend-rs monolith.
//!
//! Provides organization profile, BYOK settings, stats, and channel verification.
//! The integration_settings SQL is inlined to avoid in-process coupling.

use chrono::Utc;
use serde_json::{json, Value};
use sqlx::{PgPool, Row, postgres::PgRow};
use std::collections::HashMap;
use tracing::error;

/// Shared application state for the organization service.
#[derive(Clone)]
pub struct OrganizationState {
    pub db: PgPool,
    pub default_tenant_key: String,
    pub meta_whatsapp_access_token: String,
    pub meta_whatsapp_phone_number_id: String,
    pub meta_app_secret: String,
    pub meta_webhook_verify_token: String,
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
}

type AppState = OrganizationState;

// ---------------------------------------------------------------------------
// Integration settings helpers (inlined from backend-rs)
// ---------------------------------------------------------------------------

fn usable(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !trimmed.chars().any(|c| c == '•') && !trimmed.starts_with("•")
}

fn text(row: &PgRow, col: &str) -> String {
    row.try_get::<Option<String>, _>(col).ok().flatten().unwrap_or_default()
}

fn flag(row: &PgRow, col: &str) -> bool {
    row.try_get::<Option<bool>, _>(col).ok().flatten().unwrap_or(false)
}

fn int64(row: &PgRow, col: &str) -> i64 {
    row.try_get::<Option<i64>, _>(col).ok().flatten().unwrap_or(-1)
}

const SELECT_COLUMNS: &str = "id, tenant_key, whatsapp_provider, meta_access_token, \
    meta_phone_number_id, meta_waba_id, meta_app_secret, meta_webhook_verify_token, \
    whatsapp_verified_at, whatsapp_verified_name, whatsapp_display_number, \
    business_name, business_description, agent_guidelines, auto_reply_enabled, \
    business_hours_enabled, business_hours_start, business_hours_end, business_timezone, \
    away_message, updated_at, \
    twilio_account_sid, twilio_auth_token, twilio_sms_from, twilio_messaging_service_sid, \
    telegram_bot_token, telegram_bot_id, \
    line_channel_access_token, line_channel_secret, line_channel_id, \
    tiktok_app_id, tiktok_app_secret, tiktok_access_token, tiktok_webhook_verify_token, \
    facebook_page_access_token, facebook_page_id, facebook_app_secret, facebook_webhook_verify_token, \
    instagram_access_token, instagram_account_id, instagram_app_secret, \
    email_smtp_host, email_smtp_port, email_smtp_username, email_smtp_password, \
    email_from_address, postmark_api_token, email_webhook_secret, \
    voice_webhook_url, voice_webhook_secret, \
    anthropic_api_key, anthropic_model, anthropic_base_url, anthropic_max_tokens, \
    openai_api_key, openai_model, openai_base_url, \
    embedding_provider, embedding_api_key, embedding_model, \
    translation_api_url, translation_api_key, \
    organization_logo_url, organization_website, organization_industry, organization_size, \
    organization_address, organization_phone, organization_email, organization_social_links, \
    ai_agent_enabled, ai_agent_tone, ai_agent_language, ai_agent_max_context_messages, \
    ai_agent_auto_reply_confidence, ai_agent_escalation_keywords, ai_agent_system_prompt, \
    ai_memory_enabled, ai_memory_max_entries, ai_rag_enabled, ai_rag_top_k, ai_rag_similarity_threshold, \
    auto_reply_whatsapp, auto_reply_sms, auto_reply_email, auto_reply_telegram, \
    auto_reply_facebook, auto_reply_instagram, auto_reply_line, auto_reply_tiktok, \
    auto_reply_voice, auto_reply_live_chat, \
    whatsapp_connected, whatsapp_last_connected_at, \
    sms_connected, sms_last_connected_at, \
    telegram_connected, telegram_last_connected_at, \
    line_connected, line_last_connected_at, \
    tiktok_connected, tiktok_last_connected_at, \
    facebook_connected, facebook_last_connected_at, \
    instagram_connected, instagram_last_connected_at, \
    email_connected, email_last_connected_at, \
    voice_connected, voice_last_connected_at, \
    ai_connected, ai_last_connected_at";

fn row_to_json(row: &PgRow) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("id".to_string(), Value::String(text(row, "id")));
    map.insert("tenant_key".to_string(), Value::String(text(row, "tenant_key")));
    map.insert("whatsapp_provider".to_string(), Value::String(text(row, "whatsapp_provider")));
    map.insert("meta_access_token".to_string(), Value::String(text(row, "meta_access_token")));
    map.insert("meta_phone_number_id".to_string(), Value::String(text(row, "meta_phone_number_id")));
    map.insert("meta_waba_id".to_string(), Value::String(text(row, "meta_waba_id")));
    map.insert("meta_app_secret".to_string(), Value::String(text(row, "meta_app_secret")));
    map.insert("meta_webhook_verify_token".to_string(), Value::String(text(row, "meta_webhook_verify_token")));
    map.insert("whatsapp_verified_at".to_string(), Value::String(text(row, "whatsapp_verified_at")));
    map.insert("whatsapp_verified_name".to_string(), Value::String(text(row, "whatsapp_verified_name")));
    map.insert("whatsapp_display_number".to_string(), Value::String(text(row, "whatsapp_display_number")));
    map.insert("business_name".to_string(), Value::String(text(row, "business_name")));
    map.insert("business_description".to_string(), Value::String(text(row, "business_description")));
    map.insert("agent_guidelines".to_string(), Value::String(text(row, "agent_guidelines")));
    map.insert("auto_reply_enabled".to_string(), Value::Bool(flag(row, "auto_reply_enabled")));
    map.insert("business_hours_enabled".to_string(), Value::Bool(flag(row, "business_hours_enabled")));
    map.insert("business_hours_start".to_string(), Value::Number(int64(row, "business_hours_start").into()));
    map.insert("business_hours_end".to_string(), Value::Number(int64(row, "business_hours_end").into()));
    map.insert("business_timezone".to_string(), Value::String(text(row, "business_timezone")));
    map.insert("away_message".to_string(), Value::String(text(row, "away_message")));
    map.insert("updated_at".to_string(), Value::String(text(row, "updated_at")));
    // BYOK channels
    map.insert("twilio_account_sid".to_string(), Value::String(text(row, "twilio_account_sid")));
    map.insert("twilio_auth_token".to_string(), Value::String(text(row, "twilio_auth_token")));
    map.insert("twilio_sms_from".to_string(), Value::String(text(row, "twilio_sms_from")));
    map.insert("twilio_messaging_service_sid".to_string(), Value::String(text(row, "twilio_messaging_service_sid")));
    map.insert("telegram_bot_token".to_string(), Value::String(text(row, "telegram_bot_token")));
    map.insert("telegram_bot_id".to_string(), Value::String(text(row, "telegram_bot_id")));
    map.insert("line_channel_access_token".to_string(), Value::String(text(row, "line_channel_access_token")));
    map.insert("line_channel_secret".to_string(), Value::String(text(row, "line_channel_secret")));
    map.insert("line_channel_id".to_string(), Value::String(text(row, "line_channel_id")));
    map.insert("tiktok_app_id".to_string(), Value::String(text(row, "tiktok_app_id")));
    map.insert("tiktok_app_secret".to_string(), Value::String(text(row, "tiktok_app_secret")));
    map.insert("tiktok_access_token".to_string(), Value::String(text(row, "tiktok_access_token")));
    map.insert("tiktok_webhook_verify_token".to_string(), Value::String(text(row, "tiktok_webhook_verify_token")));
    map.insert("facebook_page_access_token".to_string(), Value::String(text(row, "facebook_page_access_token")));
    map.insert("facebook_page_id".to_string(), Value::String(text(row, "facebook_page_id")));
    map.insert("facebook_app_secret".to_string(), Value::String(text(row, "facebook_app_secret")));
    map.insert("facebook_webhook_verify_token".to_string(), Value::String(text(row, "facebook_webhook_verify_token")));
    map.insert("instagram_access_token".to_string(), Value::String(text(row, "instagram_access_token")));
    map.insert("instagram_account_id".to_string(), Value::String(text(row, "instagram_account_id")));
    map.insert("instagram_app_secret".to_string(), Value::String(text(row, "instagram_app_secret")));
    map.insert("email_smtp_host".to_string(), Value::String(text(row, "email_smtp_host")));
    map.insert("email_smtp_port".to_string(), Value::Number(int64(row, "email_smtp_port").into()));
    map.insert("email_smtp_username".to_string(), Value::String(text(row, "email_smtp_username")));
    map.insert("email_smtp_password".to_string(), Value::String(text(row, "email_smtp_password")));
    map.insert("email_from_address".to_string(), Value::String(text(row, "email_from_address")));
    map.insert("postmark_api_token".to_string(), Value::String(text(row, "postmark_api_token")));
    map.insert("email_webhook_secret".to_string(), Value::String(text(row, "email_webhook_secret")));
    map.insert("voice_webhook_url".to_string(), Value::String(text(row, "voice_webhook_url")));
    map.insert("voice_webhook_secret".to_string(), Value::String(text(row, "voice_webhook_secret")));
    // AI
    map.insert("anthropic_api_key".to_string(), Value::String(text(row, "anthropic_api_key")));
    map.insert("anthropic_model".to_string(), Value::String(text(row, "anthropic_model")));
    map.insert("anthropic_base_url".to_string(), Value::String(text(row, "anthropic_base_url")));
    map.insert("anthropic_max_tokens".to_string(), Value::Number(int64(row, "anthropic_max_tokens").into()));
    map.insert("openai_api_key".to_string(), Value::String(text(row, "openai_api_key")));
    map.insert("openai_model".to_string(), Value::String(text(row, "openai_model")));
    map.insert("openai_base_url".to_string(), Value::String(text(row, "openai_base_url")));
    map.insert("embedding_provider".to_string(), Value::String(text(row, "embedding_provider")));
    map.insert("embedding_api_key".to_string(), Value::String(text(row, "embedding_api_key")));
    map.insert("embedding_model".to_string(), Value::String(text(row, "embedding_model")));
    map.insert("translation_api_url".to_string(), Value::String(text(row, "translation_api_url")));
    map.insert("translation_api_key".to_string(), Value::String(text(row, "translation_api_key")));
    // Organization profile
    map.insert("organization_logo_url".to_string(), Value::String(text(row, "organization_logo_url")));
    map.insert("organization_website".to_string(), Value::String(text(row, "organization_website")));
    map.insert("organization_industry".to_string(), Value::String(text(row, "organization_industry")));
    map.insert("organization_size".to_string(), Value::String(text(row, "organization_size")));
    map.insert("organization_address".to_string(), Value::String(text(row, "organization_address")));
    map.insert("organization_phone".to_string(), Value::String(text(row, "organization_phone")));
    map.insert("organization_email".to_string(), Value::String(text(row, "organization_email")));
    map.insert("organization_social_links".to_string(), Value::String(text(row, "organization_social_links")));
    // AI agent
    map.insert("ai_agent_enabled".to_string(), Value::Bool(flag(row, "ai_agent_enabled")));
    map.insert("ai_agent_tone".to_string(), Value::String(text(row, "ai_agent_tone")));
    map.insert("ai_agent_language".to_string(), Value::String(text(row, "ai_agent_language")));
    map.insert("ai_agent_max_context_messages".to_string(), Value::Number(int64(row, "ai_agent_max_context_messages").into()));
    {
        let v: f64 = row.try_get("ai_agent_auto_reply_confidence").unwrap_or(0.85);
        map.insert("ai_agent_auto_reply_confidence".to_string(), Value::Number(
            serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from_f64(0.85).unwrap())
        ));
    }
    map.insert("ai_agent_escalation_keywords".to_string(), Value::String(text(row, "ai_agent_escalation_keywords")));
    map.insert("ai_agent_system_prompt".to_string(), Value::String(text(row, "ai_agent_system_prompt")));
    map.insert("ai_memory_enabled".to_string(), Value::Bool(flag(row, "ai_memory_enabled")));
    map.insert("ai_memory_max_entries".to_string(), Value::Number(int64(row, "ai_memory_max_entries").into()));
    map.insert("ai_rag_enabled".to_string(), Value::Bool(flag(row, "ai_rag_enabled")));
    map.insert("ai_rag_top_k".to_string(), Value::Number(int64(row, "ai_rag_top_k").into()));
    {
        let v: f64 = row.try_get("ai_rag_similarity_threshold").unwrap_or(0.75);
        map.insert("ai_rag_similarity_threshold".to_string(), Value::Number(
            serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from_f64(0.75).unwrap())
        ));
    }
    // Auto reply per channel
    map.insert("auto_reply_whatsapp".to_string(), Value::Bool(flag(row, "auto_reply_whatsapp")));
    map.insert("auto_reply_sms".to_string(), Value::Bool(flag(row, "auto_reply_sms")));
    map.insert("auto_reply_email".to_string(), Value::Bool(flag(row, "auto_reply_email")));
    map.insert("auto_reply_telegram".to_string(), Value::Bool(flag(row, "auto_reply_telegram")));
    map.insert("auto_reply_facebook".to_string(), Value::Bool(flag(row, "auto_reply_facebook")));
    map.insert("auto_reply_instagram".to_string(), Value::Bool(flag(row, "auto_reply_instagram")));
    map.insert("auto_reply_line".to_string(), Value::Bool(flag(row, "auto_reply_line")));
    map.insert("auto_reply_tiktok".to_string(), Value::Bool(flag(row, "auto_reply_tiktok")));
    map.insert("auto_reply_voice".to_string(), Value::Bool(flag(row, "auto_reply_voice")));
    map.insert("auto_reply_live_chat".to_string(), Value::Bool(flag(row, "auto_reply_live_chat")));
    // Connection status
    map.insert("whatsapp_connected".to_string(), Value::Bool(flag(row, "whatsapp_connected")));
    map.insert("whatsapp_last_connected_at".to_string(), Value::String(text(row, "whatsapp_last_connected_at")));
    map.insert("sms_connected".to_string(), Value::Bool(flag(row, "sms_connected")));
    map.insert("sms_last_connected_at".to_string(), Value::String(text(row, "sms_last_connected_at")));
    map.insert("telegram_connected".to_string(), Value::Bool(flag(row, "telegram_connected")));
    map.insert("telegram_last_connected_at".to_string(), Value::String(text(row, "telegram_last_connected_at")));
    map.insert("line_connected".to_string(), Value::Bool(flag(row, "line_connected")));
    map.insert("line_last_connected_at".to_string(), Value::String(text(row, "line_last_connected_at")));
    map.insert("tiktok_connected".to_string(), Value::Bool(flag(row, "tiktok_connected")));
    map.insert("tiktok_last_connected_at".to_string(), Value::String(text(row, "tiktok_last_connected_at")));
    map.insert("facebook_connected".to_string(), Value::Bool(flag(row, "facebook_connected")));
    map.insert("facebook_last_connected_at".to_string(), Value::String(text(row, "facebook_last_connected_at")));
    map.insert("instagram_connected".to_string(), Value::Bool(flag(row, "instagram_connected")));
    map.insert("instagram_last_connected_at".to_string(), Value::String(text(row, "instagram_last_connected_at")));
    map.insert("email_connected".to_string(), Value::Bool(flag(row, "email_connected")));
    map.insert("email_last_connected_at".to_string(), Value::String(text(row, "email_last_connected_at")));
    map.insert("voice_connected".to_string(), Value::Bool(flag(row, "voice_connected")));
    map.insert("voice_last_connected_at".to_string(), Value::String(text(row, "voice_last_connected_at")));
    map.insert("ai_connected".to_string(), Value::Bool(flag(row, "ai_connected")));
    map.insert("ai_last_connected_at".to_string(), Value::String(text(row, "ai_last_connected_at")));
    Value::Object(map)
}

async fn load_settings(pool: &PgPool, tenant_key: &str) -> Result<Option<Value>, sqlx::Error> {
    let sql = format!("SELECT {} FROM integration_settings WHERE tenant_key = $1 LIMIT 1", SELECT_COLUMNS);
    let row = sqlx::query(&sql).bind(tenant_key).fetch_optional(pool).await?;
    Ok(row.as_ref().map(row_to_json))
}

async fn get_or_create_settings(pool: &PgPool, tenant_key: &str, state: &AppState) -> Result<Value, sqlx::Error> {
    if let Some(existing) = load_settings(pool, tenant_key).await? {
        return Ok(existing);
    }
    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO integration_settings (id, tenant_key, whatsapp_provider, meta_access_token, \
         meta_phone_number_id, meta_app_secret, meta_webhook_verify_token, business_timezone, \
         updated_at) VALUES ($1, $2, 'meta', $3, $4, $5, $6, 'UTC', $7)",
    )
    .bind(&id)
    .bind(tenant_key)
    .bind(&state.meta_whatsapp_access_token)
    .bind(&state.meta_whatsapp_phone_number_id)
    .bind(&state.meta_app_secret)
    .bind(&state.meta_webhook_verify_token)
    .bind(&now)
    .execute(pool)
    .await?;
    load_settings(pool, tenant_key).await?.ok_or_else(|| {
        sqlx::Error::ColumnNotFound("failed to read back created settings".to_string())
    })
}

fn writable_columns() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    for col in [
        "whatsapp_provider", "meta_access_token", "meta_phone_number_id", "meta_waba_id",
        "meta_app_secret", "meta_webhook_verify_token", "whatsapp_verified_at",
        "whatsapp_verified_name", "whatsapp_display_number", "business_name",
        "business_description", "agent_guidelines", "business_timezone", "away_message",
        "twilio_account_sid", "twilio_auth_token", "twilio_sms_from", "twilio_messaging_service_sid",
        "telegram_bot_token", "telegram_bot_id",
        "line_channel_access_token", "line_channel_secret", "line_channel_id",
        "tiktok_app_id", "tiktok_app_secret", "tiktok_access_token", "tiktok_webhook_verify_token",
        "facebook_page_access_token", "facebook_page_id", "facebook_app_secret", "facebook_webhook_verify_token",
        "instagram_access_token", "instagram_account_id", "instagram_app_secret",
        "email_smtp_host", "email_smtp_username", "email_smtp_password", "email_from_address",
        "postmark_api_token", "email_webhook_secret",
        "voice_webhook_url", "voice_webhook_secret",
        "anthropic_api_key", "anthropic_model", "anthropic_base_url",
        "openai_api_key", "openai_model", "openai_base_url",
        "embedding_provider", "embedding_api_key", "embedding_model",
        "translation_api_url", "translation_api_key",
        "organization_logo_url", "organization_website", "organization_industry", "organization_size",
        "organization_address", "organization_phone", "organization_email", "organization_social_links",
        "ai_agent_tone", "ai_agent_language", "ai_agent_escalation_keywords", "ai_agent_system_prompt",
        "business_hours_start", "business_hours_end", "email_smtp_port",
        "anthropic_max_tokens", "ai_agent_max_context_messages", "ai_memory_max_entries", "ai_rag_top_k",
        "auto_reply_enabled", "business_hours_enabled", "auto_reply_whatsapp", "auto_reply_sms",
        "auto_reply_email", "auto_reply_telegram", "auto_reply_facebook", "auto_reply_instagram",
        "auto_reply_line", "auto_reply_tiktok", "auto_reply_voice", "auto_reply_live_chat",
        "ai_agent_enabled", "ai_memory_enabled", "ai_rag_enabled",
    ] {
        m.insert(col, "text");
    }
    m
}

async fn update_settings(pool: &PgPool, tenant_key: &str, patch: &Value) -> Result<(u64, Vec<String>, Vec<String>), sqlx::Error> {
    let columns = writable_columns();
    let mut applied: Vec<String> = Vec::new();
    let mut rejected: Vec<String> = Vec::new();
    let mut sets: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();

    if let Some(map) = patch.as_object() {
        let mut keys: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
        keys.sort_unstable();
        for (idx, key) in keys.iter().enumerate() {
            if columns.contains_key(*key) {
                let value = map[*key].as_str().unwrap_or_default().to_string();
                sets.push(format!("{} = ${}", key, idx + 1));
                binds.push(value);
                applied.push((*key).to_string());
            } else {
                rejected.push((*key).to_string());
            }
        }
    }

    if sets.is_empty() {
        return Ok((0, applied, rejected));
    }

    let now = Utc::now().to_rfc3339();
    sets.push(format!("updated_at = ${}", binds.len() + 1));
    binds.push(now);

    let sql = format!(
        "UPDATE integration_settings SET {} WHERE tenant_key = ${}",
        sets.join(", "),
        binds.len() + 1
    );

    let mut query = sqlx::query(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(tenant_key);

    let res = query.execute(pool).await?;
    Ok((res.rows_affected(), applied, rejected))
}

// ---------------------------------------------------------------------------
// Public handler functions
// ---------------------------------------------------------------------------

pub async fn get_profile(state: &AppState) -> (u16, String) {
    let tk = if !state.default_tenant_key.is_empty() {
        state.default_tenant_key.clone()
    } else {
        "newax".to_string()
    };

    match load_settings(&state.db, &tk).await {
        Ok(Some(settings)) => (200, settings.to_string()),
        Ok(None) => {
            let empty = json!({ "id": "", "tenant_key": &tk, "error": "no settings found" });
            (200, empty.to_string())
        }
        Err(e) => {
            error!("Failed to load profile: {}", e);
            (500, json!({ "error": format!("Failed to load profile: {}", e) }).to_string())
        }
    }
}

pub async fn update_profile(state: &AppState, body: &str) -> (u16, String) {
    let tk = if !state.default_tenant_key.is_empty() {
        state.default_tenant_key.clone()
    } else {
        "newax".to_string()
    };

    let patch: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return (400, json!({ "error": format!("Invalid JSON: {}", e) }).to_string()),
    };

    // Ensure a row exists
    let _ = get_or_create_settings(&state.db, &tk, state).await;

    match update_settings(&state.db, &tk, &patch).await {
        Ok((rows, applied, rejected)) => (
            200,
            json!({
                "status": "updated",
                "rowsAffected": rows,
                "applied": applied,
                "rejected": rejected,
            })
            .to_string(),
        ),
        Err(e) => {
            error!("Failed to update profile: {}", e);
            (500, json!({ "error": format!("Failed to update profile: {}", e) }).to_string())
        }
    }
}

pub async fn get_settings(state: &AppState) -> (u16, String) {
    get_profile(state).await
}

pub async fn update_settings_handler(state: &AppState, body: &str) -> (u16, String) {
    update_profile(state, body).await
}

pub async fn get_stats(state: &AppState) -> (u16, String) {
    let tk = if !state.default_tenant_key.is_empty() {
        state.default_tenant_key.clone()
    } else {
        "newax".to_string()
    };

    let mut stats = json!({});

    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM contacts WHERE COALESCE(tenant_key, '') = $"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["totalContacts"] = json!(count);
    }

    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM conversations WHERE COALESCE(tenant_key, '') = $"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["totalConversations"] = json!(count);
    }

    if let Ok(count) = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM drafts")
        .fetch_one(&state.db)
        .await
    {
        stats["totalDrafts"] = json!(count);
    }

    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM knowledge_base_entries WHERE COALESCE(tenant_key, '') = $"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["knowledgeBaseEntries"] = json!(count);
    }

    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM services WHERE COALESCE(tenant_key, '') = $"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["services"] = json!(count);
    }

    if let Ok(count) = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM inboxes WHERE COALESCE(tenant_key, '') = $ AND COALESCE(is_active, 1) = 1"
    )
    .bind(&tk)
    .fetch_one(&state.db)
    .await
    {
        stats["activeInboxes"] = json!(count);
    }

    (200, stats.to_string())
}

pub async fn verify_channel(state: &AppState, channel: &str) -> (u16, String) {
    let result = match channel.to_lowercase().as_str() {
        "whatsapp" => verify_whatsapp(state).await,
        "sms" | "twilio" => verify_twilio(state).await,
        "telegram" => verify_telegram(state).await,
        "line" => verify_line(state).await,
        "ai" | "anthropic" => verify_ai(state).await,
        _ => (
            200,
            json!({ "connected": false, "error": format!("Channel '{}' verification not implemented", channel) })
                .to_string(),
        ),
    };
    result
}

async fn verify_whatsapp(state: &AppState) -> (u16, String) {
    let token = state.meta_whatsapp_access_token.clone();
    if token.trim().is_empty() {
        return (
            200,
            json!({ "connected": false, "error": "No WhatsApp access token configured" }).to_string(),
        );
    }
    let phone_id = state.meta_whatsapp_phone_number_id.clone();
    if phone_id.is_empty() {
        return (
            200,
            json!({ "connected": false, "error": "No WhatsApp phone number ID configured" }).to_string(),
        );
    }

    let client = reqwest::Client::new();
    match client
        .get(format!("https://graph.facebook.com/v21.0/{}", phone_id))
        .query(&[("access_token", token.as_str()), ("fields", "name,verified_name")])
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body: Result<Value, _> = resp.json().await;
            (
                200,
                json!({
                    "connected": true,
                    "providerInfo": body.ok(),
                    "lastConnectedAt": Utc::now().to_rfc3339(),
                })
                .to_string(),
            )
        }
        Ok(resp) => (
            200,
            json!({ "connected": false, "error": format!("HTTP {}: {}", resp.status(), resp.status()) }).to_string(),
        ),
        Err(e) => (
            200,
            json!({ "connected": false, "error": format!("Request failed: {}", e) }).to_string(),
        ),
    }
}

async fn verify_twilio(state: &AppState) -> (u16, String) {
    let sid = state.twilio_account_sid.clone();
    let token = state.twilio_auth_token.clone();
    if sid.is_empty() || token.is_empty() {
        return (
            200,
            json!({ "connected": false, "error": "Twilio credentials not configured" }).to_string(),
        );
    }

    let client = reqwest::Client::new();
    match client
        .get("https://api.twilio.com/2010-04-01/Accounts.json")
        .basic_auth(&sid, Some(&token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => (
            200,
            json!({ "connected": true, "lastConnectedAt": Utc::now().to_rfc3339() }).to_string(),
        ),
        Ok(resp) => (
            200,
            json!({ "connected": false, "error": format!("HTTP {}", resp.status()) }).to_string(),
        ),
        Err(e) => (
            200,
            json!({ "connected": false, "error": format!("Request failed: {}", e) }).to_string(),
        ),
    }
}

async fn verify_telegram(state: &AppState) -> (u16, String) {
    let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    if token.is_empty() {
        return (
            200,
            json!({ "connected": false, "error": "Telegram bot token not configured" }).to_string(),
        );
    }

    let client = reqwest::Client::new();
    match client
        .get(format!("https://api.telegram.org/bot{}/getMe", token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => (
            200,
            json!({ "connected": true, "lastConnectedAt": Utc::now().to_rfc3339() }).to_string(),
        ),
        Ok(resp) => (
            200,
            json!({ "connected": false, "error": format!("HTTP {}", resp.status()) }).to_string(),
        ),
        Err(e) => (
            200,
            json!({ "connected": false, "error": format!("Request failed: {}", e) }).to_string(),
        ),
    }
}

async fn verify_line(state: &AppState) -> (u16, String) {
    let token = std::env::var("LINE_CHANNEL_ACCESS_TOKEN").unwrap_or_default();
    if token.is_empty() {
        return (
            200,
            json!({ "connected": false, "error": "LINE channel access token not configured" }).to_string(),
        );
    }

    let client = reqwest::Client::new();
    match client
        .get("https://api.line.me/v2/bot/info")
        .bearer_auth(&token)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => (
            200,
            json!({ "connected": true, "lastConnectedAt": Utc::now().to_rfc3339() }).to_string(),
        ),
        Ok(resp) => (
            200,
            json!({ "connected": false, "error": format!("HTTP {}", resp.status()) }).to_string(),
        ),
        Err(e) => (
            200,
            json!({ "connected": false, "error": format!("Request failed: {}", e) }).to_string(),
        ),
    }
}

async fn verify_ai(state: &AppState) -> (u16, String) {
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    let configured = !anthropic_key.is_empty() || !openai_key.is_empty();

    if configured {
        (
            200,
            json!({
                "connected": true,
                "providerInfo": {
                    "anthropic_configured": !anthropic_key.is_empty(),
                    "openai_configured": !openai_key.is_empty(),
                },
                "lastConnectedAt": Utc::now().to_rfc3339(),
            })
            .to_string(),
        )
    } else {
        (
            200,
            json!({ "connected": false, "error": "AI provider not configured" }).to_string(),
        )
    }
}
