-- Phase 53: Advanced OTP Engine
-- Cryptographically strong OTP generation shared across all channels.
-- Per-channel toggles allow brands to enable/disable OTP delivery per channel.

-- OTP records: every generated OTP is stored here for verification and audit.
CREATE TABLE IF NOT EXISTS otp_records (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    otp_code TEXT NOT NULL,                          -- the actual OTP code (hashed for storage)
    otp_hash TEXT NOT NULL,                          -- HMAC-SHA256 hash of the code for secure verification
    otp_salt TEXT NOT NULL,                          -- per-OTP salt for hashing
    purpose TEXT NOT NULL,                           -- 'sign_up', 'sign_in', 'password_reset', 'phone_verify', 'email_verify', 'custom'
    channel TEXT NOT NULL,                           -- 'whatsapp', 'email', 'sms', 'in_app'
    recipient TEXT NOT NULL,                         -- phone number or email address
    recipient_type TEXT NOT NULL,                    -- 'phone' or 'email'
    status TEXT NOT NULL DEFAULT 'pending',          -- 'pending', 'sent', 'verified', 'expired', 'cancelled'
    attempts INTEGER NOT NULL DEFAULT 0,             -- number of verification attempts
    max_attempts INTEGER NOT NULL DEFAULT 5,         -- max verification attempts before invalidation
    ttl_seconds INTEGER NOT NULL DEFAULT 300,        -- time-to-live in seconds (default 5 minutes)
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',                     -- additional context (e.g., session_id, ip_address)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_otp_records_brand ON otp_records(brand_id);
CREATE INDEX IF NOT EXISTS idx_otp_records_recipient ON otp_records(recipient);
CREATE INDEX IF NOT EXISTS idx_otp_records_purpose ON otp_records(purpose);
CREATE INDEX IF NOT EXISTS idx_otp_records_channel ON otp_records(channel);
CREATE INDEX IF NOT EXISTS idx_otp_records_status ON otp_records(status);
CREATE INDEX IF NOT EXISTS idx_otp_records_expires ON otp_records(expires_at);

-- OTP channel configuration: per-brand toggles for which channels deliver OTP.
CREATE TABLE IF NOT EXISTS otp_channel_config (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL UNIQUE,
    whatsapp_enabled BOOLEAN NOT NULL DEFAULT true,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    sms_enabled BOOLEAN NOT NULL DEFAULT false,
    in_app_enabled BOOLEAN NOT NULL DEFAULT true,
    default_channel TEXT NOT NULL DEFAULT 'whatsapp',  -- fallback channel when multiple are enabled
    code_length INTEGER NOT NULL DEFAULT 12,            -- length of OTP code (min 9, max 20)
    code_charset TEXT NOT NULL DEFAULT 'alphanumeric',  -- 'numeric', 'alphanumeric', 'alphanumeric_special'
    ttl_seconds INTEGER NOT NULL DEFAULT 300,           -- default TTL in seconds
    max_attempts INTEGER NOT NULL DEFAULT 5,            -- max verification attempts
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 3,   -- max OTP requests per minute per recipient
    rate_limit_per_hour INTEGER NOT NULL DEFAULT 10,    -- max OTP requests per hour per recipient
    cooldown_seconds INTEGER NOT NULL DEFAULT 60,       -- cooldown between OTP requests for same recipient
    lockout_seconds INTEGER NOT NULL DEFAULT 900,       -- lockout after max attempts exceeded
    template_name TEXT DEFAULT 'OTP Verification',      -- email template name for email channel
    whatsapp_message_template TEXT,                     -- custom WhatsApp message template
    sms_message_template TEXT,                          -- custom SMS message template
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_otp_channel_config_brand ON otp_channel_config(brand_id);

-- OTP rate limiting: tracks request frequency per recipient.
CREATE TABLE IF NOT EXISTS otp_rate_limit (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    recipient TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    window_type TEXT NOT NULL DEFAULT 'minute',  -- 'minute' or 'hour'
    locked_until TIMESTAMPTZ,                    -- if locked out due to too many attempts
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(brand_id, recipient, window_type)
);

CREATE INDEX IF NOT EXISTS idx_otp_rate_limit_recipient ON otp_rate_limit(recipient);
CREATE INDEX IF NOT EXISTS idx_otp_rate_limit_window ON otp_rate_limit(window_start);

-- OTP audit log: immutable record of all OTP events.
CREATE TABLE IF NOT EXISTS otp_audit_log (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    otp_id TEXT,
    event_type TEXT NOT NULL,                     -- 'generated', 'sent', 'verified', 'failed_attempt', 'expired', 'cancelled', 'rate_limited', 'locked_out'
    channel TEXT,
    recipient TEXT,
    ip_address TEXT,
    user_agent TEXT,
    details JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_otp_audit_brand ON otp_audit_log(brand_id);
CREATE INDEX IF NOT EXISTS idx_otp_audit_otp_id ON otp_audit_log(otp_id);
CREATE INDEX IF NOT EXISTS idx_otp_audit_event ON otp_audit_log(event_type);
