-- Migration 0002: Extend integration_settings with BYOK (Bring Your Own Keys)
-- for all channels and AI services, plus organization profile enhancements.
-- Each tenant/organization can now manage their own API keys for all integrations.

-- ============================================================
-- Channel: Twilio (SMS/Voice)
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN twilio_account_sid TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN twilio_auth_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN twilio_sms_from TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN twilio_messaging_service_sid TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: Telegram
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN telegram_bot_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN telegram_bot_id TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: LINE
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN line_channel_access_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN line_channel_secret TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN line_channel_id TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: TikTok
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN tiktok_app_id TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN tiktok_app_secret TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN tiktok_access_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN tiktok_webhook_verify_token TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: Facebook Messenger
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN facebook_page_access_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN facebook_page_id TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN facebook_app_secret TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN facebook_webhook_verify_token TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: Instagram
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN instagram_access_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN instagram_account_id TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN instagram_app_secret TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: Email (Postmark/SMTP)
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN email_smtp_host TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN email_smtp_port INTEGER NOT NULL DEFAULT 587;
ALTER TABLE integration_settings ADD COLUMN email_smtp_username TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN email_smtp_password TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN email_from_address TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN postmark_api_token TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN email_webhook_secret TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Channel: Voice
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN voice_webhook_url TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN voice_webhook_secret TEXT NOT NULL DEFAULT '';

-- ============================================================
-- AI: Anthropic Claude
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN anthropic_api_key TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN anthropic_model TEXT NOT NULL DEFAULT 'claude-3-5-sonnet';
ALTER TABLE integration_settings ADD COLUMN anthropic_base_url TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN anthropic_max_tokens INTEGER NOT NULL DEFAULT 1024;

-- ============================================================
-- AI: OpenAI (optional alternative)
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN openai_api_key TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN openai_model TEXT NOT NULL DEFAULT 'gpt-4';
ALTER TABLE integration_settings ADD COLUMN openai_base_url TEXT NOT NULL DEFAULT '';

-- ============================================================
-- AI: Embedding provider (for RAG)
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN embedding_provider TEXT NOT NULL DEFAULT 'openai';
ALTER TABLE integration_settings ADD COLUMN embedding_api_key TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN embedding_model TEXT NOT NULL DEFAULT 'text-embedding-3-small';

-- ============================================================
-- Translation Service
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN translation_api_url TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN translation_api_key TEXT NOT NULL DEFAULT '';

-- ============================================================
-- Organization Profile Enhancements
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN organization_logo_url TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_website TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_industry TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_size TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_address TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_phone TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_email TEXT NOT NULL DEFAULT '';
ALTER TABLE integration_settings ADD COLUMN organization_social_links TEXT NOT NULL DEFAULT '{}';

-- ============================================================
-- AI Agent Configuration
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN ai_agent_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN ai_agent_tone TEXT NOT NULL DEFAULT 'professional';
ALTER TABLE integration_settings ADD COLUMN ai_agent_language TEXT NOT NULL DEFAULT 'en';
ALTER TABLE integration_settings ADD COLUMN ai_agent_max_context_messages INTEGER NOT NULL DEFAULT 40;
ALTER TABLE integration_settings ADD COLUMN ai_agent_auto_reply_confidence REAL NOT NULL DEFAULT 0.85;
ALTER TABLE integration_settings ADD COLUMN ai_agent_escalation_keywords TEXT NOT NULL DEFAULT '[]';
ALTER TABLE integration_settings ADD COLUMN ai_agent_system_prompt TEXT NOT NULL DEFAULT '';

-- ============================================================
-- AI Memory & RAG Configuration
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN ai_memory_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN ai_memory_max_entries INTEGER NOT NULL DEFAULT 100;
ALTER TABLE integration_settings ADD COLUMN ai_rag_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN ai_rag_top_k INTEGER NOT NULL DEFAULT 5;
ALTER TABLE integration_settings ADD COLUMN ai_rag_similarity_threshold REAL NOT NULL DEFAULT 0.75;

-- ============================================================
-- Auto-Reply Configuration (per-channel overrides)
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN auto_reply_whatsapp BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_sms BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_email BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_telegram BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_facebook BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_instagram BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_line BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_tiktok BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE integration_settings ADD COLUMN auto_reply_voice BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN auto_reply_live_chat BOOLEAN NOT NULL DEFAULT true;

-- ============================================================
-- Connection Status Tracking
-- ============================================================
ALTER TABLE integration_settings ADD COLUMN whatsapp_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN whatsapp_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN sms_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN sms_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN telegram_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN telegram_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN line_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN line_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN tiktok_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN tiktok_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN facebook_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN facebook_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN instagram_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN instagram_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN email_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN email_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN voice_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN voice_last_connected_at TEXT;
ALTER TABLE integration_settings ADD COLUMN ai_connected BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE integration_settings ADD COLUMN ai_last_connected_at TEXT;
