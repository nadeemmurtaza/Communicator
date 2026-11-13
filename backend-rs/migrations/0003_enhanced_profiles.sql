-- Migration 0003: Enhanced Profiles with Data Sources, Live Sync, Conflicts, and Custom Sections
-- Adds unique IDs, source tracking, sync status, conflict resolution, custom sections,
-- pricing packages, promotions, auto-vouchers, and message triggers

-- ============================================================================
-- 1. UNIQUE IDENTIFIERS
-- ============================================================================

-- Add unique external IDs to tenants, organizations, and brands
ALTER TABLE tenants ADD COLUMN external_id TEXT UNIQUE;
ALTER TABLE organizations ADD COLUMN external_id TEXT UNIQUE;
ALTER TABLE brands ADD COLUMN external_id TEXT UNIQUE;

CREATE INDEX IF NOT EXISTS idx_tenants_external_id ON tenants(external_id);
CREATE INDEX IF NOT EXISTS idx_orgs_external_id ON organizations(external_id);
CREATE INDEX IF NOT EXISTS idx_brands_external_id ON brands(external_id);

-- ============================================================================
-- 2. DATA SOURCE TRACKING
-- ============================================================================

-- Track the source of each data field in brand profiles
CREATE TABLE IF NOT EXISTS brand_profile_sources (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    field_name TEXT NOT NULL,
    source_platform TEXT NOT NULL,  -- 'meta', 'linkedin', 'line', 'tiktok', 'telegram', 'user'
    source_user_id TEXT DEFAULT '',  -- user who entered the data if source is 'user'
    source_user_name TEXT DEFAULT '',
    source_record_id TEXT DEFAULT '',  -- external ID from the source platform
    source_url TEXT DEFAULT '',
    synced_at TEXT,
    last_verified_at TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    confidence_score REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_profile_sources_brand ON brand_profile_sources(brand_id);
CREATE INDEX IF NOT EXISTS idx_profile_sources_field ON brand_profile_sources(field_name);
CREATE INDEX IF NOT EXISTS idx_profile_sources_platform ON brand_profile_sources(source_platform);

-- Same for organization profiles
CREATE TABLE IF NOT EXISTS org_profile_sources (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    field_name TEXT NOT NULL,
    source_platform TEXT NOT NULL,
    source_user_id TEXT DEFAULT '',
    source_user_name TEXT DEFAULT '',
    source_record_id TEXT DEFAULT '',
    source_url TEXT DEFAULT '',
    synced_at TEXT,
    last_verified_at TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    confidence_score REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_org_sources_org ON org_profile_sources(organization_id);
CREATE INDEX IF NOT EXISTS idx_org_sources_field ON org_profile_sources(field_name);

-- ============================================================================
-- 3. LIVE SYNC STATUS
-- ============================================================================

-- Track sync status for each brand with external platforms
CREATE TABLE IF NOT EXISTS brand_sync_status (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,  -- 'meta', 'linkedin', 'line', 'tiktok', 'telegram', 'whatsapp'
    is_connected BOOLEAN NOT NULL DEFAULT false,
    last_synced_at TIMESTAMPTZ,
    next_sync_at TEXT,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 3600,
    last_error TEXT DEFAULT '',
    connection_status TEXT NOT NULL DEFAULT 'disconnected',  -- 'connected', 'syncing', 'error', 'disconnected'
    platform_data TEXT DEFAULT '{}',  -- cached platform data (limits, verified name, etc.)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sync_status_brand ON brand_sync_status(brand_id);
CREATE INDEX IF NOT EXISTS idx_sync_status_platform ON brand_sync_status(platform);

-- Organization-level sync status
CREATE TABLE IF NOT EXISTS org_sync_status (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    is_connected BOOLEAN NOT NULL DEFAULT false,
    last_synced_at TIMESTAMPTZ,
    next_sync_at TEXT,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 3600,
    last_error TEXT DEFAULT '',
    connection_status TEXT NOT NULL DEFAULT 'disconnected',
    platform_data TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_org_sync_status_org ON org_sync_status(organization_id);

-- ============================================================================
-- 4. CONFLICT DETECTION AND RESOLUTION
-- ============================================================================

-- Track conflicts between different data sources
CREATE TABLE IF NOT EXISTS profile_conflicts (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'brand' or 'organization'
    entity_id TEXT NOT NULL,
    field_name TEXT NOT NULL,
    conflict_type TEXT NOT NULL,  -- 'value_mismatch', 'missing_source', 'stale_data'
    source_a_platform TEXT NOT NULL,
    source_a_value TEXT NOT NULL,
    source_a_last_updated TEXT,
    source_b_platform TEXT NOT NULL,
    source_b_value TEXT NOT NULL,
    source_b_last_updated TEXT,
    resolution_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'auto_resolved', 'manual_resolved'
    resolved_value TEXT DEFAULT '',
    resolved_by TEXT DEFAULT '',  -- 'system' or user_id
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conflicts_entity ON profile_conflicts(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_conflicts_status ON profile_conflicts(resolution_status);

-- ============================================================================
-- 5. CUSTOM PROFILE SECTIONS
-- ============================================================================

-- User-defined custom sections for profiles
CREATE TABLE IF NOT EXISTS profile_custom_sections (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'brand' or 'organization'
    entity_id TEXT NOT NULL,
    section_name TEXT NOT NULL,
    section_type TEXT NOT NULL,  -- 'table', 'list', 'paragraph', 'key_value', 'rich_text'
    display_order BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by TEXT DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_custom_sections_entity ON profile_custom_sections(entity_type, entity_id);

-- Data for custom sections
CREATE TABLE IF NOT EXISTS profile_custom_section_data (
    id TEXT PRIMARY KEY,
    section_id TEXT NOT NULL REFERENCES profile_custom_sections(id) ON DELETE CASCADE,
    row_key TEXT DEFAULT '',  -- for tables: row identifier
    field_key TEXT NOT NULL,
    field_value TEXT NOT NULL,
    field_type TEXT NOT NULL DEFAULT 'text',  -- 'text', 'number', 'date', 'url', 'boolean', 'json'
    display_order BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_custom_section_data_section ON profile_custom_section_data(section_id);

-- ============================================================================
-- 6. PRICING, PACKAGES, OFFERS, PROMOTIONS
-- ============================================================================

-- Pricing packages for brands
CREATE TABLE IF NOT EXISTS brand_packages (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    price REAL NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'USD',
    duration_type TEXT DEFAULT '',  -- 'one_time', 'monthly', 'yearly'
    duration_value BOOLEAN DEFAULT true,
    features TEXT DEFAULT '[]',  -- JSON array of feature strings
    is_active BOOLEAN NOT NULL DEFAULT true,
    display_order BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_packages_brand ON brand_packages(brand_id);

-- Promotions and offers
CREATE TABLE IF NOT EXISTS brand_promotions (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    discount_type TEXT NOT NULL,  -- 'percentage', 'fixed', 'bogo'
    discount_value REAL NOT NULL DEFAULT 0,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    applicable_packages TEXT DEFAULT '[]',  -- JSON array of package IDs
    is_active BOOLEAN NOT NULL DEFAULT true,
    usage_limit BIGINT DEFAULT 0,  -- 0 = unlimited
    usage_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_promotions_brand ON brand_promotions(brand_id);

-- Auto-voucher generation settings
CREATE TABLE IF NOT EXISTS brand_voucher_settings (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    auto_generate BOOLEAN NOT NULL DEFAULT false,
    voucher_prefix TEXT NOT NULL DEFAULT 'VOUCHER',
    voucher_length INTEGER NOT NULL DEFAULT 10,
    discount_type TEXT NOT NULL DEFAULT 'percentage',
    discount_value REAL NOT NULL DEFAULT 10,
    validity_days INTEGER NOT NULL DEFAULT 30,
    trigger_event TEXT DEFAULT '',  -- 'first_contact', 'purchase', 'birthday', 'anniversary'
    max_uses_per_customer BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_voucher_settings_brand ON brand_voucher_settings(brand_id);

-- ============================================================================
-- 7. TRANSACTIONAL MESSAGE TRIGGERS
-- ============================================================================

-- Trigger definitions for automated messages
CREATE TABLE IF NOT EXISTS message_triggers (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    trigger_type TEXT NOT NULL,  -- 'event', 'schedule', 'condition'
    trigger_event TEXT DEFAULT '',  -- 'new_contact', 'purchase', 'ticket_created', 'sla_breach'
    trigger_schedule TEXT DEFAULT '',  -- cron expression for schedule triggers
    trigger_conditions TEXT DEFAULT '{}',  -- JSON conditions for condition triggers
    preferred_channel TEXT NOT NULL DEFAULT 'whatsapp',  -- 'whatsapp', 'sms', 'email', 'telegram'
    message_template TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_triggers_brand ON message_triggers(brand_id);

-- Trigger execution log
CREATE TABLE IF NOT EXISTS message_trigger_logs (
    id TEXT PRIMARY KEY,
    trigger_id TEXT NOT NULL REFERENCES message_triggers(id) ON DELETE CASCADE,
    contact_id TEXT DEFAULT '',
    executed_at TEXT NOT NULL,
    channel_used TEXT NOT NULL,
    message_sent TEXT NOT NULL,
    status TEXT NOT NULL,  -- 'sent', 'failed', 'skipped'
    error_message TEXT DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trigger_logs_trigger ON message_trigger_logs(trigger_id);
CREATE INDEX IF NOT EXISTS idx_trigger_logs_contact ON message_trigger_logs(contact_id);

-- ============================================================================
-- 8. PLATFORM-SPECIFIC DATA CACHE
-- ============================================================================

-- Cache data from external platforms for real-time sync
CREATE TABLE IF NOT EXISTS platform_data_cache (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'brand' or 'organization'
    entity_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    data_key TEXT NOT NULL,
    data_value TEXT NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_platform_cache_entity ON platform_data_cache(entity_type, entity_id, platform);
CREATE INDEX IF NOT EXISTS idx_platform_cache_key ON platform_data_cache(data_key);

-- ============================================================================
-- 9. RAG CONNECTION TABLES
-- ============================================================================

-- Link custom sections to RAG for AI agent context
CREATE TABLE IF NOT EXISTS rag_section_mapping (
    id TEXT PRIMARY KEY,
    section_id TEXT NOT NULL REFERENCES profile_custom_sections(id) ON DELETE CASCADE,
    is_indexed BOOLEAN NOT NULL DEFAULT true,
    embedding_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'indexed', 'failed'
    last_indexed_at TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rag_section_mapping ON rag_section_mapping(section_id);

-- ============================================================================
-- 10. BACKWARD COMPATIBILITY
-- ============================================================================

-- Set external_id for existing records if not set
UPDATE tenants SET external_id = id WHERE external_id IS NULL;
UPDATE organizations SET external_id = id WHERE external_id IS NULL;
UPDATE brands SET external_id = id WHERE external_id IS NULL;
