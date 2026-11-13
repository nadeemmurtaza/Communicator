-- WhatsApp Channel: Templates, Customer Profiles, Session Windows, Automation State

-- WhatsApp templates cached from Meta Graph API
CREATE TABLE IF NOT EXISTS whatsapp_templates (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    language TEXT,
    status TEXT DEFAULT 'approved',
    body TEXT NOT NULL,
    components TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_templates_brand ON whatsapp_templates(brand_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_templates_status ON whatsapp_templates(status);

-- Customer profiles synced from Meta
CREATE TABLE IF NOT EXISTS whatsapp_customer_profiles (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    wa_id TEXT NOT NULL,
    profile_name TEXT,
    profile_picture_url TEXT,
    about TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_fetched_at TEXT,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_customer_profiles_brand ON whatsapp_customer_profiles(brand_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_customer_profiles_wa_id ON whatsapp_customer_profiles(wa_id);

-- 24-hour session windows per contact
CREATE TABLE IF NOT EXISTS whatsapp_session_windows (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    message_count BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_session_brand ON whatsapp_session_windows(brand_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_session_contact ON whatsapp_session_windows(contact_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_session_active ON whatsapp_session_windows(is_active, expires_at);

-- Per-conversation automation state
CREATE TABLE IF NOT EXISTS whatsapp_automation_state (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    greeting_sent BOOLEAN DEFAULT false,
    csat_sent BOOLEAN DEFAULT false,
    last_menu TEXT,
    current_flow TEXT,
    metadata TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_automation_brand ON whatsapp_automation_state(brand_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_automation_conversation ON whatsapp_automation_state(conversation_id);

-- Template send history for audit
CREATE TABLE IF NOT EXISTS whatsapp_template_log (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    template_name TEXT NOT NULL,
    template_id TEXT,
    contact_id TEXT,
    conversation_id TEXT,
    message_id TEXT,
    status TEXT,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_template_log_brand ON whatsapp_template_log(brand_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_template_log_contact ON whatsapp_template_log(contact_id);

-- WhatsApp automation rules
CREATE TABLE IF NOT EXISTS whatsapp_automation_rules (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    trigger_type TEXT NOT NULL,
    trigger_value TEXT,
    action_type TEXT NOT NULL,
    action_config TEXT,
    is_active BOOLEAN DEFAULT true,
    priority BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_rules_brand ON whatsapp_automation_rules(brand_id);
CREATE INDEX IF NOT EXISTS idx_whatsapp_rules_active ON whatsapp_automation_rules(is_active, priority);

-- WhatsApp business profile cache
CREATE TABLE IF NOT EXISTS whatsapp_business_profile_cache (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    verified_name TEXT,
    about TEXT,
    address TEXT,
    description TEXT,
    email TEXT,
    websites TEXT,
    vertical TEXT,
    profile_picture_url TEXT,
    messaging_limit_tier TEXT,
    quality_rating TEXT,
    account_mode TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_profile_cache_brand ON whatsapp_business_profile_cache(brand_id);
