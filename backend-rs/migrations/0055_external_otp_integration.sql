-- Phase 55: External OTP Integration Layer
-- Enables external e-commerce platforms (Shopify, custom stores, sales channels)
-- to connect to the Communicator OTP engine for OTP generation and verification.
--
-- Architecture:
-- 1. Each brand generates an integration key pair (public + secret) per external platform
-- 2. External platform uses the secret key to authenticate API calls
-- 3. Webhook callbacks notify the external platform of verification events
-- 4. The external platform's frontend displays the OTP input form
--    and calls the verify endpoint with the integration key

-- External integration keys: one row per brand + platform combination
CREATE TABLE IF NOT EXISTS otp_external_integrations (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    brand_id TEXT NOT NULL,
    platform_name TEXT NOT NULL,              -- 'shopify', 'custom_store', 'woocommerce', 'magento', 'salesforce', etc.
    platform_url TEXT NOT NULL DEFAULT '',     -- the external platform's base URL (e.g., https://mystore.myshopify.com)
    display_name TEXT NOT NULL DEFAULT '',     -- human-readable name for admin UI
    public_key TEXT NOT NULL,                  -- safe to expose in frontend JS (e.g., pk_live_abc123)
    secret_key_hash TEXT NOT NULL,             -- HMAC-SHA256 hash of the secret key for verification
    secret_key_salt TEXT NOT NULL,             -- per-key salt for hashing
    webhook_url TEXT DEFAULT '',               -- callback URL on the external platform for verification events
    webhook_secret TEXT DEFAULT '',            -- shared secret for signing webhook payloads (HMAC-SHA256)
    allowed_purposes TEXT[] NOT NULL DEFAULT ARRAY['sign_up', 'sign_in', 'password_reset', 'phone_verify', 'email_verify'],
    allowed_channels TEXT[] NOT NULL DEFAULT ARRAY['whatsapp', 'email', 'sms', 'in_app'],
    is_active BOOLEAN NOT NULL DEFAULT true,
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 10,
    rate_limit_per_hour INTEGER NOT NULL DEFAULT 100,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_otp_ext_integrations_brand ON otp_external_integrations(brand_id);
CREATE INDEX IF NOT EXISTS idx_otp_ext_integrations_public_key ON otp_external_integrations(public_key);
CREATE INDEX IF NOT EXISTS idx_otp_ext_integrations_platform ON otp_external_integrations(platform_name);

COMMENT ON TABLE otp_external_integrations IS 'External integration keys for e-commerce platforms to use the OTP engine. secret_key_hash stores HMAC-SHA256 of the real secret — never the plaintext.';
COMMENT ON COLUMN otp_external_integrations.public_key IS 'Exposed in frontend JS for identify-ing the integration. Never used for authentication.';
COMMENT ON COLUMN otp_external_integrations.secret_key_hash IS 'HMAC-SHA256 hash of the secret key. Used to verify API calls from the external platform backend.';
COMMENT ON COLUMN otp_external_integrations.webhook_url IS 'URL on the external platform that receives verification event callbacks (verified, failed, expired).';
COMMENT ON COLUMN otp_external_integrations.webhook_secret IS 'Shared secret for HMAC-SHA256 signing of webhook payloads. The external platform verifies the signature.';

-- External integration usage log: tracks API calls from external platforms
CREATE TABLE IF NOT EXISTS otp_external_usage_log (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    integration_id TEXT NOT NULL REFERENCES otp_external_integrations(id) ON DELETE CASCADE,
    brand_id TEXT NOT NULL,
    action TEXT NOT NULL,                      -- 'generate', 'verify', 'status_check'
    recipient TEXT,
    purpose TEXT,
    success BOOLEAN NOT NULL DEFAULT true,
    response_code INTEGER,
    ip_address TEXT,
    user_agent TEXT,
    error_message TEXT,
    request_payload JSONB DEFAULT '{}',
    response_payload JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_otp_ext_usage_integration ON otp_external_usage_log(integration_id);
CREATE INDEX IF NOT EXISTS idx_otp_ext_usage_brand ON otp_external_usage_log(brand_id);
CREATE INDEX IF NOT EXISTS idx_otp_ext_usage_created ON otp_external_usage_log(created_at);

-- Webhook delivery log: tracks outbound webhook deliveries to external platforms
CREATE TABLE IF NOT EXISTS otp_webhook_delivery_log (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    integration_id TEXT NOT NULL REFERENCES otp_external_integrations(id) ON DELETE CASCADE,
    brand_id TEXT NOT NULL,
    otp_id TEXT,
    event_type TEXT NOT NULL,                 -- 'otp_generated', 'otp_verified', 'otp_failed', 'otp_expired'
    webhook_url TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',   -- 'pending', 'sent', 'delivered', 'failed', 'retrying'
    http_status INTEGER,
    response_body TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_otp_webhook_delivery_integration ON otp_webhook_delivery_log(integration_id);
CREATE INDEX IF NOT EXISTS idx_otp_webhook_delivery_status ON otp_webhook_delivery_log(status);
CREATE INDEX IF NOT EXISTS idx_otp_webhook_delivery_otp ON otp_webhook_delivery_log(otp_id);
