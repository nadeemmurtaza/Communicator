#![allow(dead_code)]

//! Typed access to the `integration_settings` table -- the Connections page's
//! backing store.
//!
//! Before this module every caller re-implemented the same SQL and the same
//! "DB value wins, else fall back to the process environment" rule, each with
//! slightly different column lists. The rules live here once:
//!
//! * one row per tenant (`tenant_key`), default tenant is the one the product
//!   ships with and is *not* invented here -- it comes from
//!   `Config::default_tenant_key` when set, otherwise the caller must pass one;
//! * a stored value is only usable when it is non-empty and not a display mask
//!   (`••••1234` is not a credential -- see `meta_sync.rs`);
//! * writes go through a column whitelist, so a webhook payload can never make
//!   `integration_settings` the target of a `SQL`-shaped field name.

use crate::config::Config;
use crate::db::DbPool;
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;

/// Columns this module is allowed to write, mapped to the SQL type each binding
/// needs. Anything else in a patch is reported as rejected instead of ignored
/// silently.
enum ColumnType {
    Text,
    Int,
    Bool,
}

fn writable_columns() -> HashMap<&'static str, ColumnType> {
    let mut m = HashMap::new();
    for (name, kind) in [
        // Original columns
        ("whatsapp_provider", ColumnType::Text),
        ("meta_access_token", ColumnType::Text),
        ("meta_phone_number_id", ColumnType::Text),
        ("meta_waba_id", ColumnType::Text),
        ("meta_app_secret", ColumnType::Text),
        ("meta_webhook_verify_token", ColumnType::Text),
        ("whatsapp_verified_at", ColumnType::Text),
        ("whatsapp_verified_name", ColumnType::Text),
        ("whatsapp_display_number", ColumnType::Text),
        ("business_name", ColumnType::Text),
        ("business_description", ColumnType::Text),
        ("agent_guidelines", ColumnType::Text),
        ("auto_reply_enabled", ColumnType::Bool),
        ("business_hours_enabled", ColumnType::Bool),
        ("business_hours_start", ColumnType::Int),
        ("business_hours_end", ColumnType::Int),
        ("business_timezone", ColumnType::Text),
        ("away_message", ColumnType::Text),
        // BYOK: Twilio
        ("twilio_account_sid", ColumnType::Text),
        ("twilio_auth_token", ColumnType::Text),
        ("twilio_sms_from", ColumnType::Text),
        ("twilio_messaging_service_sid", ColumnType::Text),
        // BYOK: Telegram
        ("telegram_bot_token", ColumnType::Text),
        ("telegram_bot_id", ColumnType::Text),
        // BYOK: LINE
        ("line_channel_access_token", ColumnType::Text),
        ("line_channel_secret", ColumnType::Text),
        ("line_channel_id", ColumnType::Text),
        // BYOK: TikTok
        ("tiktok_app_id", ColumnType::Text),
        ("tiktok_app_secret", ColumnType::Text),
        ("tiktok_access_token", ColumnType::Text),
        ("tiktok_webhook_verify_token", ColumnType::Text),
        // BYOK: Facebook
        ("facebook_page_access_token", ColumnType::Text),
        ("facebook_page_id", ColumnType::Text),
        ("facebook_app_secret", ColumnType::Text),
        ("facebook_webhook_verify_token", ColumnType::Text),
        // BYOK: Instagram
        ("instagram_access_token", ColumnType::Text),
        ("instagram_account_id", ColumnType::Text),
        ("instagram_app_secret", ColumnType::Text),
        // BYOK: Email
        ("email_smtp_host", ColumnType::Text),
        ("email_smtp_port", ColumnType::Int),
        ("email_smtp_username", ColumnType::Text),
        ("email_smtp_password", ColumnType::Text),
        ("email_from_address", ColumnType::Text),
        ("postmark_api_token", ColumnType::Text),
        ("email_webhook_secret", ColumnType::Text),
        // BYOK: Voice
        ("voice_webhook_url", ColumnType::Text),
        ("voice_webhook_secret", ColumnType::Text),
        // BYOK: AI - Anthropic
        ("anthropic_api_key", ColumnType::Text),
        ("anthropic_model", ColumnType::Text),
        ("anthropic_base_url", ColumnType::Text),
        ("anthropic_max_tokens", ColumnType::Int),
        // BYOK: AI - OpenAI
        ("openai_api_key", ColumnType::Text),
        ("openai_model", ColumnType::Text),
        ("openai_base_url", ColumnType::Text),
        // BYOK: AI - Embeddings
        ("embedding_provider", ColumnType::Text),
        ("embedding_api_key", ColumnType::Text),
        ("embedding_model", ColumnType::Text),
        // BYOK: Translation
        ("translation_api_url", ColumnType::Text),
        ("translation_api_key", ColumnType::Text),
        // Organization Profile
        ("organization_logo_url", ColumnType::Text),
        ("organization_website", ColumnType::Text),
        ("organization_industry", ColumnType::Text),
        ("organization_size", ColumnType::Text),
        ("organization_address", ColumnType::Text),
        ("organization_phone", ColumnType::Text),
        ("organization_email", ColumnType::Text),
        ("organization_social_links", ColumnType::Text),
        // AI Agent Config
        ("ai_agent_enabled", ColumnType::Bool),
        ("ai_agent_tone", ColumnType::Text),
        ("ai_agent_language", ColumnType::Text),
        ("ai_agent_max_context_messages", ColumnType::Int),
        ("ai_agent_auto_reply_confidence", ColumnType::Text),
        ("ai_agent_escalation_keywords", ColumnType::Text),
        ("ai_agent_system_prompt", ColumnType::Text),
        // AI Memory & RAG
        ("ai_memory_enabled", ColumnType::Bool),
        ("ai_memory_max_entries", ColumnType::Int),
        ("ai_rag_enabled", ColumnType::Bool),
        ("ai_rag_top_k", ColumnType::Int),
        ("ai_rag_similarity_threshold", ColumnType::Text),
        // Per-channel auto-reply
        ("auto_reply_whatsapp", ColumnType::Bool),
        ("auto_reply_sms", ColumnType::Bool),
        ("auto_reply_email", ColumnType::Bool),
        ("auto_reply_telegram", ColumnType::Bool),
        ("auto_reply_facebook", ColumnType::Bool),
        ("auto_reply_instagram", ColumnType::Bool),
        ("auto_reply_line", ColumnType::Bool),
        ("auto_reply_tiktok", ColumnType::Bool),
        ("auto_reply_voice", ColumnType::Bool),
        ("auto_reply_live_chat", ColumnType::Bool),
        // Connection status
        ("whatsapp_connected", ColumnType::Bool),
        ("whatsapp_last_connected_at", ColumnType::Text),
        ("sms_connected", ColumnType::Bool),
        ("sms_last_connected_at", ColumnType::Text),
        ("telegram_connected", ColumnType::Bool),
        ("telegram_last_connected_at", ColumnType::Text),
        ("line_connected", ColumnType::Bool),
        ("line_last_connected_at", ColumnType::Text),
        ("tiktok_connected", ColumnType::Bool),
        ("tiktok_last_connected_at", ColumnType::Text),
        ("facebook_connected", ColumnType::Bool),
        ("facebook_last_connected_at", ColumnType::Text),
        ("instagram_connected", ColumnType::Bool),
        ("instagram_last_connected_at", ColumnType::Text),
        ("email_connected", ColumnType::Bool),
        ("email_last_connected_at", ColumnType::Text),
        ("voice_connected", ColumnType::Bool),
        ("voice_last_connected_at", ColumnType::Text),
        ("ai_connected", ColumnType::Bool),
        ("ai_last_connected_at", ColumnType::Text),
    ] {
        m.insert(name, kind);
    }
    m
}

/// A stored credential/display value is only real when it is non-empty and not
/// the masked form the UI shows back.
pub fn usable(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !trimmed.chars().any(|c| c == '•') && !trimmed.starts_with("•")
}

fn text(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn flag(row: &sqlx::postgres::PgRow, column: &str) -> bool {
    row.try_get::<Option<bool>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(false)
        || row
            .try_get::<Option<i64>, _>(column)
            .ok()
            .flatten()
            .unwrap_or(0)
            != 0
}

fn number(row: &sqlx::postgres::PgRow, column: &str, default: i64) -> i64 {
    row.try_get::<Option<i64>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(default)
}

const SELECT_COLUMNS: &str = "id, tenant_key, whatsapp_provider, meta_access_token, \
     meta_phone_number_id, meta_waba_id, meta_app_secret, meta_webhook_verify_token, \
     whatsapp_verified_at, whatsapp_verified_name, whatsapp_display_number, business_name, \
     business_description, agent_guidelines, auto_reply_enabled, business_hours_enabled, \
     business_hours_start, business_hours_end, business_timezone, away_message, updated_at, \
     twilio_account_sid, twilio_auth_token, twilio_sms_from, twilio_messaging_service_sid, \
     telegram_bot_token, telegram_bot_id, \
     line_channel_access_token, line_channel_secret, line_channel_id, \
     tiktok_app_id, tiktok_app_secret, tiktok_access_token, tiktok_webhook_verify_token, \
     facebook_page_access_token, facebook_page_id, facebook_app_secret, facebook_webhook_verify_token, \
     instagram_access_token, instagram_account_id, instagram_app_secret, \
     email_smtp_host, email_smtp_port, email_smtp_username, email_smtp_password, email_from_address, \
     postmark_api_token, email_webhook_secret, \
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

#[derive(Debug, Clone)]
pub struct IntegrationSettings {
    pub id: String,
    pub tenant_key: String,
    // Original WhatsApp
    pub whatsapp_provider: String,
    pub meta_access_token: String,
    pub meta_phone_number_id: String,
    pub meta_waba_id: String,
    pub meta_app_secret: String,
    pub meta_webhook_verify_token: String,
    pub whatsapp_verified_at: String,
    pub whatsapp_verified_name: String,
    pub whatsapp_display_number: String,
    // Business info
    pub business_name: String,
    pub business_description: String,
    pub agent_guidelines: String,
    pub auto_reply_enabled: bool,
    pub business_hours_enabled: bool,
    pub business_hours_start: i64,
    pub business_hours_end: i64,
    pub business_timezone: String,
    pub away_message: String,
    pub updated_at: String,
    // BYOK: Twilio
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_sms_from: String,
    pub twilio_messaging_service_sid: String,
    // BYOK: Telegram
    pub telegram_bot_token: String,
    pub telegram_bot_id: String,
    // BYOK: LINE
    pub line_channel_access_token: String,
    pub line_channel_secret: String,
    pub line_channel_id: String,
    // BYOK: TikTok
    pub tiktok_app_id: String,
    pub tiktok_app_secret: String,
    pub tiktok_access_token: String,
    pub tiktok_webhook_verify_token: String,
    // BYOK: Facebook
    pub facebook_page_access_token: String,
    pub facebook_page_id: String,
    pub facebook_app_secret: String,
    pub facebook_webhook_verify_token: String,
    // BYOK: Instagram
    pub instagram_access_token: String,
    pub instagram_account_id: String,
    pub instagram_app_secret: String,
    // BYOK: Email
    pub email_smtp_host: String,
    pub email_smtp_port: i64,
    pub email_smtp_username: String,
    pub email_smtp_password: String,
    pub email_from_address: String,
    pub postmark_api_token: String,
    pub email_webhook_secret: String,
    // BYOK: Voice
    pub voice_webhook_url: String,
    pub voice_webhook_secret: String,
    // BYOK: AI - Anthropic
    pub anthropic_api_key: String,
    pub anthropic_model: String,
    pub anthropic_base_url: String,
    pub anthropic_max_tokens: i64,
    // BYOK: AI - OpenAI
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_base_url: String,
    // BYOK: AI - Embeddings
    pub embedding_provider: String,
    pub embedding_api_key: String,
    pub embedding_model: String,
    // BYOK: Translation
    pub translation_api_url: String,
    pub translation_api_key: String,
    // Organization Profile
    pub organization_logo_url: String,
    pub organization_website: String,
    pub organization_industry: String,
    pub organization_size: String,
    pub organization_address: String,
    pub organization_phone: String,
    pub organization_email: String,
    pub organization_social_links: String,
    // AI Agent Config
    pub ai_agent_enabled: bool,
    pub ai_agent_tone: String,
    pub ai_agent_language: String,
    pub ai_agent_max_context_messages: i64,
    pub ai_agent_auto_reply_confidence: f64,
    pub ai_agent_escalation_keywords: String,
    pub ai_agent_system_prompt: String,
    // AI Memory & RAG
    pub ai_memory_enabled: bool,
    pub ai_memory_max_entries: i64,
    pub ai_rag_enabled: bool,
    pub ai_rag_top_k: i64,
    pub ai_rag_similarity_threshold: f64,
    // Per-channel auto-reply
    pub auto_reply_whatsapp: bool,
    pub auto_reply_sms: bool,
    pub auto_reply_email: bool,
    pub auto_reply_telegram: bool,
    pub auto_reply_facebook: bool,
    pub auto_reply_instagram: bool,
    pub auto_reply_line: bool,
    pub auto_reply_tiktok: bool,
    pub auto_reply_voice: bool,
    pub auto_reply_live_chat: bool,
    // Connection status
    pub whatsapp_connected: bool,
    pub whatsapp_last_connected_at: String,
    pub sms_connected: bool,
    pub sms_last_connected_at: String,
    pub telegram_connected: bool,
    pub telegram_last_connected_at: String,
    pub line_connected: bool,
    pub line_last_connected_at: String,
    pub tiktok_connected: bool,
    pub tiktok_last_connected_at: String,
    pub facebook_connected: bool,
    pub facebook_last_connected_at: String,
    pub instagram_connected: bool,
    pub instagram_last_connected_at: String,
    pub email_connected: bool,
    pub email_last_connected_at: String,
    pub voice_connected: bool,
    pub voice_last_connected_at: String,
    pub ai_connected: bool,
    pub ai_last_connected_at: String,
}

impl IntegrationSettings {
    /// Row with nothing in it: used when the table has no row for the tenant
    /// and creating one would be a write on a read path. `is_default` lets
    /// callers tell "the admin never opened the Connections page" apart from
    /// "the admin configured it", which the reply engine and the business-hours
    /// gate both need.
    pub fn empty(tenant_key: &str) -> Self {
        Self {
            id: String::new(),
            tenant_key: tenant_key.to_string(),
            whatsapp_provider: String::new(),
            meta_access_token: String::new(),
            meta_phone_number_id: String::new(),
            meta_waba_id: String::new(),
            meta_app_secret: String::new(),
            meta_webhook_verify_token: String::new(),
            whatsapp_verified_at: String::new(),
            whatsapp_verified_name: String::new(),
            whatsapp_display_number: String::new(),
            business_name: String::new(),
            business_description: String::new(),
            agent_guidelines: String::new(),
            auto_reply_enabled: false,
            business_hours_enabled: false,
            business_hours_start: -1,
            business_hours_end: -1,
            business_timezone: String::new(),
            away_message: String::new(),
            updated_at: String::new(),
            // BYOK defaults
            twilio_account_sid: String::new(), twilio_auth_token: String::new(),
            twilio_sms_from: String::new(), twilio_messaging_service_sid: String::new(),
            telegram_bot_token: String::new(), telegram_bot_id: String::new(),
            line_channel_access_token: String::new(), line_channel_secret: String::new(), line_channel_id: String::new(),
            tiktok_app_id: String::new(), tiktok_app_secret: String::new(),
            tiktok_access_token: String::new(), tiktok_webhook_verify_token: String::new(),
            facebook_page_access_token: String::new(), facebook_page_id: String::new(),
            facebook_app_secret: String::new(), facebook_webhook_verify_token: String::new(),
            instagram_access_token: String::new(), instagram_account_id: String::new(), instagram_app_secret: String::new(),
            email_smtp_host: String::new(), email_smtp_port: 587,
            email_smtp_username: String::new(), email_smtp_password: String::new(),
            email_from_address: String::new(), postmark_api_token: String::new(), email_webhook_secret: String::new(),
            voice_webhook_url: String::new(), voice_webhook_secret: String::new(),
            anthropic_api_key: String::new(), anthropic_model: "claude-3-5-sonnet".to_string(),
            anthropic_base_url: String::new(), anthropic_max_tokens: 1024,
            openai_api_key: String::new(), openai_model: "gpt-4".to_string(), openai_base_url: String::new(),
            embedding_provider: "openai".to_string(), embedding_api_key: String::new(),
            embedding_model: "text-embedding-3-small".to_string(),
            translation_api_url: String::new(), translation_api_key: String::new(),
            organization_logo_url: String::new(), organization_website: String::new(),
            organization_industry: String::new(), organization_size: String::new(),
            organization_address: String::new(), organization_phone: String::new(),
            organization_email: String::new(), organization_social_links: "{}".to_string(),
            ai_agent_enabled: false, ai_agent_tone: "professional".to_string(),
            ai_agent_language: "en".to_string(), ai_agent_max_context_messages: 40,
            ai_agent_auto_reply_confidence: 0.85, ai_agent_escalation_keywords: "[]".to_string(),
            ai_agent_system_prompt: String::new(),
            ai_memory_enabled: false, ai_memory_max_entries: 100,
            ai_rag_enabled: false, ai_rag_top_k: 5, ai_rag_similarity_threshold: 0.75,
            auto_reply_whatsapp: true, auto_reply_sms: true, auto_reply_email: true,
            auto_reply_telegram: true, auto_reply_facebook: true, auto_reply_instagram: true,
            auto_reply_line: true, auto_reply_tiktok: true, auto_reply_voice: false, auto_reply_live_chat: true,
            whatsapp_connected: false, whatsapp_last_connected_at: String::new(),
            sms_connected: false, sms_last_connected_at: String::new(),
            telegram_connected: false, telegram_last_connected_at: String::new(),
            line_connected: false, line_last_connected_at: String::new(),
            tiktok_connected: false, tiktok_last_connected_at: String::new(),
            facebook_connected: false, facebook_last_connected_at: String::new(),
            instagram_connected: false, instagram_last_connected_at: String::new(),
            email_connected: false, email_last_connected_at: String::new(),
            voice_connected: false, voice_last_connected_at: String::new(),
            ai_connected: false, ai_last_connected_at: String::new(),
        }
    }

    pub fn is_default(&self) -> bool {
        self.id.is_empty()
    }

    /// WhatsApp credentials resolved field-by-field: the row wins when it holds
    /// a usable value, otherwise the process environment is used. Never a
    /// compiled-in token.
    pub fn whatsapp_credentials(&self, config: &Config) -> WhatsAppCredentials {
        WhatsAppCredentials {
            provider: first_usable(&[
                &self.whatsapp_provider,
                "meta", // transport default, not a credential
            ]),
            access_token: first_usable(&[&self.meta_access_token, &config.meta_whatsapp_access_token]),
            phone_number_id: first_usable(&[
                &self.meta_phone_number_id,
                &config.meta_whatsapp_phone_number_id,
            ]),
            waba_id: self.meta_waba_id.clone(),
            app_secret: first_usable(&[&self.meta_app_secret, &config.meta_app_secret]),
            webhook_verify_token: first_usable(&[
                &self.meta_webhook_verify_token,
                &config.meta_webhook_verify_token,
            ]),
            api_version: first_usable(&[&config.meta_whatsapp_api_version]),
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.id,
            "tenantKey": self.tenant_key,
            // WhatsApp
            "whatsappProvider": self.whatsapp_provider,
            "metaAccessTokenPresent": usable(&self.meta_access_token),
            "metaPhoneNumberId": self.meta_phone_number_id,
            "metaWabaId": self.meta_waba_id,
            "whatsappDisplayNumber": self.whatsapp_display_number,
            "whatsappVerifiedName": self.whatsapp_verified_name,
            "whatsappVerifiedAt": self.whatsapp_verified_at,
            // Business
            "businessName": self.business_name,
            "businessDescription": self.business_description,
            "agentGuidelines": self.agent_guidelines,
            "autoReplyEnabled": self.auto_reply_enabled,
            "businessHoursEnabled": self.business_hours_enabled,
            "businessHoursStart": self.business_hours_start,
            "businessHoursEnd": self.business_hours_end,
            "businessTimezone": self.business_timezone,
            "awayMessage": self.away_message,
            // BYOK: Twilio
            "twilioAccountSidPresent": usable(&self.twilio_account_sid),
            "twilioAuthTokenPresent": usable(&self.twilio_auth_token),
            "twilioSmsFrom": self.twilio_sms_from,
            // BYOK: Telegram
            "telegramBotTokenPresent": usable(&self.telegram_bot_token),
            "telegramBotId": self.telegram_bot_id,
            // BYOK: LINE
            "lineChannelAccessTokenPresent": usable(&self.line_channel_access_token),
            "lineChannelId": self.line_channel_id,
            // BYOK: TikTok
            "tiktokAppIdPresent": usable(&self.tiktok_app_id),
            "tiktokAccessTokenPresent": usable(&self.tiktok_access_token),
            // BYOK: Facebook
            "facebookPageAccessTokenPresent": usable(&self.facebook_page_access_token),
            "facebookPageId": self.facebook_page_id,
            // BYOK: Instagram
            "instagramAccessTokenPresent": usable(&self.instagram_access_token),
            "instagramAccountId": self.instagram_account_id,
            // BYOK: Email
            "emailSmtpHost": self.email_smtp_host,
            "emailSmtpPort": self.email_smtp_port,
            "emailFromAddress": self.email_from_address,
            "postmarkApiTokenPresent": usable(&self.postmark_api_token),
            // BYOK: Voice
            "voiceWebhookUrl": self.voice_webhook_url,
            // BYOK: AI
            "anthropicApiKeyPresent": usable(&self.anthropic_api_key),
            "anthropicModel": self.anthropic_model,
            "openaiApiKeyPresent": usable(&self.openai_api_key),
            "openaiModel": self.openai_model,
            "embeddingProvider": self.embedding_provider,
            "embeddingModel": self.embedding_model,
            // Translation
            "translationApiUrl": self.translation_api_url,
            // Organization Profile
            "organizationLogoUrl": self.organization_logo_url,
            "organizationWebsite": self.organization_website,
            "organizationIndustry": self.organization_industry,
            "organizationSize": self.organization_size,
            "organizationAddress": self.organization_address,
            "organizationPhone": self.organization_phone,
            "organizationEmail": self.organization_email,
            "organizationSocialLinks": self.organization_social_links,
            // AI Agent
            "aiAgentEnabled": self.ai_agent_enabled,
            "aiAgentTone": self.ai_agent_tone,
            "aiAgentLanguage": self.ai_agent_language,
            "aiAgentMaxContextMessages": self.ai_agent_max_context_messages,
            "aiAgentAutoReplyConfidence": self.ai_agent_auto_reply_confidence,
            "aiAgentEscalationKeywords": self.ai_agent_escalation_keywords,
            "aiAgentSystemPrompt": self.ai_agent_system_prompt,
            // AI Memory & RAG
            "aiMemoryEnabled": self.ai_memory_enabled,
            "aiMemoryMaxEntries": self.ai_memory_max_entries,
            "aiRagEnabled": self.ai_rag_enabled,
            "aiRagTopK": self.ai_rag_top_k,
            "aiRagSimilarityThreshold": self.ai_rag_similarity_threshold,
            // Per-channel auto-reply
            "autoReply": {
                "whatsapp": self.auto_reply_whatsapp,
                "sms": self.auto_reply_sms,
                "email": self.auto_reply_email,
                "telegram": self.auto_reply_telegram,
                "facebook": self.auto_reply_facebook,
                "instagram": self.auto_reply_instagram,
                "line": self.auto_reply_line,
                "tiktok": self.auto_reply_tiktok,
                "voice": self.auto_reply_voice,
                "liveChat": self.auto_reply_live_chat,
            },
            // Connection status
            "connections": {
                "whatsapp": { "connected": self.whatsapp_connected, "lastConnectedAt": self.whatsapp_last_connected_at },
                "sms": { "connected": self.sms_connected, "lastConnectedAt": self.sms_last_connected_at },
                "telegram": { "connected": self.telegram_connected, "lastConnectedAt": self.telegram_last_connected_at },
                "line": { "connected": self.line_connected, "lastConnectedAt": self.line_last_connected_at },
                "tiktok": { "connected": self.tiktok_connected, "lastConnectedAt": self.tiktok_last_connected_at },
                "facebook": { "connected": self.facebook_connected, "lastConnectedAt": self.facebook_last_connected_at },
                "instagram": { "connected": self.instagram_connected, "lastConnectedAt": self.instagram_last_connected_at },
                "email": { "connected": self.email_connected, "lastConnectedAt": self.email_last_connected_at },
                "voice": { "connected": self.voice_connected, "lastConnectedAt": self.voice_last_connected_at },
                "ai": { "connected": self.ai_connected, "lastConnectedAt": self.ai_last_connected_at },
            },
            "updatedAt": self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WhatsAppCredentials {
    pub provider: String,
    pub access_token: String,
    pub phone_number_id: String,
    pub waba_id: String,
    pub app_secret: String,
    pub webhook_verify_token: String,
    pub api_version: String,
}

fn first_usable(values: &[&str]) -> String {
    for value in values {
        if usable(value) {
            return value.trim().to_string();
        }
    }
    String::new()
}

fn row_to_settings(row: &sqlx::postgres::PgRow) -> IntegrationSettings {
    IntegrationSettings {
        id: text(row, "id"),
        tenant_key: text(row, "tenant_key"),
        whatsapp_provider: text(row, "whatsapp_provider"),
        meta_access_token: text(row, "meta_access_token"),
        meta_phone_number_id: text(row, "meta_phone_number_id"),
        meta_waba_id: text(row, "meta_waba_id"),
        meta_app_secret: text(row, "meta_app_secret"),
        meta_webhook_verify_token: text(row, "meta_webhook_verify_token"),
        whatsapp_verified_at: text(row, "whatsapp_verified_at"),
        whatsapp_verified_name: text(row, "whatsapp_verified_name"),
        whatsapp_display_number: text(row, "whatsapp_display_number"),
        business_name: text(row, "business_name"),
        business_description: text(row, "business_description"),
        agent_guidelines: text(row, "agent_guidelines"),
        auto_reply_enabled: flag(row, "auto_reply_enabled"),
        business_hours_enabled: flag(row, "business_hours_enabled"),
        business_hours_start: number(row, "business_hours_start", -1),
        business_hours_end: number(row, "business_hours_end", -1),
        business_timezone: text(row, "business_timezone"),
        away_message: text(row, "away_message"),
        updated_at: text(row, "updated_at"),
        // BYOK: Twilio
        twilio_account_sid: text(row, "twilio_account_sid"),
        twilio_auth_token: text(row, "twilio_auth_token"),
        twilio_sms_from: text(row, "twilio_sms_from"),
        twilio_messaging_service_sid: text(row, "twilio_messaging_service_sid"),
        // BYOK: Telegram
        telegram_bot_token: text(row, "telegram_bot_token"),
        telegram_bot_id: text(row, "telegram_bot_id"),
        // BYOK: LINE
        line_channel_access_token: text(row, "line_channel_access_token"),
        line_channel_secret: text(row, "line_channel_secret"),
        line_channel_id: text(row, "line_channel_id"),
        // BYOK: TikTok
        tiktok_app_id: text(row, "tiktok_app_id"),
        tiktok_app_secret: text(row, "tiktok_app_secret"),
        tiktok_access_token: text(row, "tiktok_access_token"),
        tiktok_webhook_verify_token: text(row, "tiktok_webhook_verify_token"),
        // BYOK: Facebook
        facebook_page_access_token: text(row, "facebook_page_access_token"),
        facebook_page_id: text(row, "facebook_page_id"),
        facebook_app_secret: text(row, "facebook_app_secret"),
        facebook_webhook_verify_token: text(row, "facebook_webhook_verify_token"),
        // BYOK: Instagram
        instagram_access_token: text(row, "instagram_access_token"),
        instagram_account_id: text(row, "instagram_account_id"),
        instagram_app_secret: text(row, "instagram_app_secret"),
        // BYOK: Email
        email_smtp_host: text(row, "email_smtp_host"),
        email_smtp_port: number(row, "email_smtp_port", 587),
        email_smtp_username: text(row, "email_smtp_username"),
        email_smtp_password: text(row, "email_smtp_password"),
        email_from_address: text(row, "email_from_address"),
        postmark_api_token: text(row, "postmark_api_token"),
        email_webhook_secret: text(row, "email_webhook_secret"),
        // BYOK: Voice
        voice_webhook_url: text(row, "voice_webhook_url"),
        voice_webhook_secret: text(row, "voice_webhook_secret"),
        // BYOK: AI - Anthropic
        anthropic_api_key: text(row, "anthropic_api_key"),
        anthropic_model: text(row, "anthropic_model"),
        anthropic_base_url: text(row, "anthropic_base_url"),
        anthropic_max_tokens: number(row, "anthropic_max_tokens", 1024),
        // BYOK: AI - OpenAI
        openai_api_key: text(row, "openai_api_key"),
        openai_model: text(row, "openai_model"),
        openai_base_url: text(row, "openai_base_url"),
        // BYOK: AI - Embeddings
        embedding_provider: text(row, "embedding_provider"),
        embedding_api_key: text(row, "embedding_api_key"),
        embedding_model: text(row, "embedding_model"),
        // BYOK: Translation
        translation_api_url: text(row, "translation_api_url"),
        translation_api_key: text(row, "translation_api_key"),
        // Organization Profile
        organization_logo_url: text(row, "organization_logo_url"),
        organization_website: text(row, "organization_website"),
        organization_industry: text(row, "organization_industry"),
        organization_size: text(row, "organization_size"),
        organization_address: text(row, "organization_address"),
        organization_phone: text(row, "organization_phone"),
        organization_email: text(row, "organization_email"),
        organization_social_links: text(row, "organization_social_links"),
        // AI Agent Config
        ai_agent_enabled: flag(row, "ai_agent_enabled"),
        ai_agent_tone: text(row, "ai_agent_tone"),
        ai_agent_language: text(row, "ai_agent_language"),
        ai_agent_max_context_messages: number(row, "ai_agent_max_context_messages", 40),
        ai_agent_auto_reply_confidence: {
            let raw: String = text(row, "ai_agent_auto_reply_confidence");
            raw.parse::<f64>().unwrap_or(0.85)
        },
        ai_agent_escalation_keywords: text(row, "ai_agent_escalation_keywords"),
        ai_agent_system_prompt: text(row, "ai_agent_system_prompt"),
        // AI Memory & RAG
        ai_memory_enabled: flag(row, "ai_memory_enabled"),
        ai_memory_max_entries: number(row, "ai_memory_max_entries", 100),
        ai_rag_enabled: flag(row, "ai_rag_enabled"),
        ai_rag_top_k: number(row, "ai_rag_top_k", 5),
        ai_rag_similarity_threshold: {
            let raw: String = text(row, "ai_rag_similarity_threshold");
            raw.parse::<f64>().unwrap_or(0.75)
        },
        // Per-channel auto-reply
        auto_reply_whatsapp: flag(row, "auto_reply_whatsapp"),
        auto_reply_sms: flag(row, "auto_reply_sms"),
        auto_reply_email: flag(row, "auto_reply_email"),
        auto_reply_telegram: flag(row, "auto_reply_telegram"),
        auto_reply_facebook: flag(row, "auto_reply_facebook"),
        auto_reply_instagram: flag(row, "auto_reply_instagram"),
        auto_reply_line: flag(row, "auto_reply_line"),
        auto_reply_tiktok: flag(row, "auto_reply_tiktok"),
        auto_reply_voice: flag(row, "auto_reply_voice"),
        auto_reply_live_chat: flag(row, "auto_reply_live_chat"),
        // Connection status
        whatsapp_connected: flag(row, "whatsapp_connected"),
        whatsapp_last_connected_at: text(row, "whatsapp_last_connected_at"),
        sms_connected: flag(row, "sms_connected"),
        sms_last_connected_at: text(row, "sms_last_connected_at"),
        telegram_connected: flag(row, "telegram_connected"),
        telegram_last_connected_at: text(row, "telegram_last_connected_at"),
        line_connected: flag(row, "line_connected"),
        line_last_connected_at: text(row, "line_last_connected_at"),
        tiktok_connected: flag(row, "tiktok_connected"),
        tiktok_last_connected_at: text(row, "tiktok_last_connected_at"),
        facebook_connected: flag(row, "facebook_connected"),
        facebook_last_connected_at: text(row, "facebook_last_connected_at"),
        instagram_connected: flag(row, "instagram_connected"),
        instagram_last_connected_at: text(row, "instagram_last_connected_at"),
        email_connected: flag(row, "email_connected"),
        email_last_connected_at: text(row, "email_last_connected_at"),
        voice_connected: flag(row, "voice_connected"),
        voice_last_connected_at: text(row, "voice_last_connected_at"),
        ai_connected: flag(row, "ai_connected"),
        ai_last_connected_at: text(row, "ai_last_connected_at"),
    }
}

/// Read one tenant's row. `Ok(None)` means the Connections page has never been
/// saved for this tenant -- callers that must not write use this instead of
/// `get_or_create`.
pub async fn load(pool: &DbPool, tenant_key: &str) -> Result<Option<IntegrationSettings>, sqlx::Error> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM integration_settings WHERE tenant_key = $1 LIMIT 1");
    let row = sqlx::query(&sql)
        .bind(tenant_key)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(row_to_settings))
}

/// Read the only row a single-tenant deployment has, when the caller does not
/// know (or has not configured) a tenant key. Deliberately not a `load`
/// fallback that fabricates anything: `Ok(None)` still means "the Connections
/// page was never saved", which is exactly what the outbound channels report.
pub async fn load_any(pool: &DbPool) -> Result<Option<IntegrationSettings>, sqlx::Error> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM integration_settings ORDER BY updated_at DESC LIMIT 1");
    let row = sqlx::query(&sql).fetch_optional(pool).await?;
    Ok(row.as_ref().map(row_to_settings))
}

/// Read the tenant's row, creating it from the process environment when absent
/// (the behaviour the Python `get_settings_row` has). Created rows carry only
/// credentials the environment actually provides: an unset env var stays empty
/// so a later send reports it as missing rather than as configured.
pub async fn get_or_create(
    pool: &DbPool,
    config: &Config,
    tenant_key: &str,
) -> Result<IntegrationSettings, sqlx::Error> {
    if let Some(existing) = load(pool, tenant_key).await? {
        return Ok(existing);
    }

    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO integration_settings (id, tenant_key, whatsapp_provider, meta_access_token, \
         meta_phone_number_id, meta_app_secret, meta_webhook_verify_token, business_timezone, \
         updated_at) VALUES ($1, $2, 'meta', $3, $4, $5, $6, 'UTC', $7)",
    )
    .bind(&id)
    .bind(tenant_key)
    .bind(&config.meta_whatsapp_access_token)
    .bind(&config.meta_whatsapp_phone_number_id)
    .bind(&config.meta_app_secret)
    .bind(&config.meta_webhook_verify_token)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(load(pool, tenant_key)
        .await?
        .unwrap_or_else(|| IntegrationSettings::empty(tenant_key)))
}

/// Column-whitelisted patch write. Returns the columns actually updated plus
/// the keys that were refused, so the route can tell the operator which fields
/// it ignored instead of dropping them silently.
pub async fn update(
    pool: &DbPool,
    tenant_key: &str,
    patch: &Value,
) -> Result<UpdateReport, sqlx::Error> {
    let columns = writable_columns();
    // Built per-type so the SQL fragment order matches the order values are
    // actually bound below (binds_text, then binds_int, then binds_bool) --
    // not the sorted-key iteration order, which would otherwise interleave
    // types and misalign positional placeholders with their bound values.
    let mut sets_text: Vec<String> = Vec::new();
    let mut sets_int: Vec<String> = Vec::new();
    let mut sets_bool: Vec<String> = Vec::new();
    let mut applied: Vec<String> = Vec::new();
    let mut rejected: Vec<String> = Vec::new();
    let mut object: HashMap<&str, &Value> = HashMap::new();

    if let Some(map) = patch.as_object() {
        for (key, value) in map {
            object.insert(key.as_str(), value);
        }
    }

    // Sorted so the built SQL (and its binding order) is deterministic.
    let mut keys: Vec<&str> = object.keys().copied().collect();
    keys.sort_unstable();

    let mut binds_text: Vec<String> = Vec::new();
    let mut binds_int: Vec<i64> = Vec::new();
    let mut binds_bool: Vec<bool> = Vec::new();

    for key in keys {
        match columns.get(key) {
            Some(ColumnType::Text) => {
                let value = object[key].as_str().unwrap_or_default().to_string();
                sets_text.push(format!("{key} = ?"));
                binds_text.push(value);
                applied.push(key.to_string());
            }
            Some(ColumnType::Int) => {
                let value = object[key]
                    .as_i64()
                    .or_else(|| object[key].as_str().and_then(|s| s.parse().ok()))
                    .unwrap_or(0);
                sets_int.push(format!("{key} = ?"));
                binds_int.push(value);
                applied.push(key.to_string());
            }
            Some(ColumnType::Bool) => {
                let raw = object[key];
                let value = match raw {
                    Value::Bool(flag) => *flag,
                    Value::Number(num) => num.as_i64().map(|v| v != 0).unwrap_or(false),
                    Value::String(text) => matches!(
                        text.to_ascii_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    ),
                    _ => false,
                };
                sets_bool.push(format!("{key} = ?"));
                binds_bool.push(value);
                applied.push(key.to_string());
            }
            None => rejected.push(key.to_string()),
        }
    }

    if sets_text.is_empty() && sets_int.is_empty() && sets_bool.is_empty() {
        return Ok(UpdateReport {
            rows_affected: 0,
            applied,
            rejected,
        });
    }

    // Concatenate in the same order as the binds applied below, then number
    // the placeholders left-to-right over that order.
    let mut sets: Vec<String> = Vec::new();
    sets.extend(sets_text);
    sets.extend(sets_int);
    sets.extend(sets_bool);
    sets.push("updated_at = ?".to_string());

    let mut placeholder = 0;
    for s in sets.iter_mut() {
        placeholder += 1;
        *s = s.replacen('?', &format!("${placeholder}"), 1);
    }
    let tenant_placeholder = placeholder + 1;

    let now = chrono::Utc::now().to_rfc3339();
    let sql = format!(
        "UPDATE integration_settings SET {} WHERE tenant_key = ${tenant_placeholder}",
        sets.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for value in binds_text {
        query = query.bind(value);
    }
    for value in binds_int {
        query = query.bind(value);
    }
    for value in binds_bool {
        query = query.bind(value);
    }
    query = query.bind(now).bind(tenant_key);

    let result = query.execute(pool).await?;
    Ok(UpdateReport {
        rows_affected: result.rows_affected(),
        applied,
        rejected,
    })
}

#[derive(Debug, Clone)]
pub struct UpdateReport {
    pub rows_affected: u64,
    pub applied: Vec<String>,
    pub rejected: Vec<String>,
}

/// Business-hours / away-message inputs are integers and strings the operator
/// typed; validate them here so every write path shares the same rules.
pub fn validate_business_hours(patch: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let get = |key: &str| patch.get(key).and_then(|v| v.as_i64());
    if let Some(start) = get("business_hours_start") {
        if !(0..=23).contains(&start) {
            problems.push(format!("business_hours_start must be 0-23, got {start}"));
        }
    }
    if let Some(end) = get("business_hours_end") {
        if !(0..=24).contains(&end) {
            problems.push(format!("business_hours_end must be 0-24, got {end}"));
        }
    }
    if let (Some(start), Some(end)) = (get("business_hours_start"), get("business_hours_end")) {
        if start == end {
            problems.push(format!(
                "business hours start and end are both {start}: the window is empty, \
                 every message would be treated as out-of-hours"
            ));
        }
    }
    problems
}
