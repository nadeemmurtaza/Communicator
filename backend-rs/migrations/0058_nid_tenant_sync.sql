-- Phase 58: nID Tenant Sync
--
-- Connects the Communicator to nID's tenant system for real-time sync.
-- When a tenant is created/updated in nID, this system receives the webhook
-- and keeps the Communicator's tenant data in sync.
--
-- Key features:
-- 1. nID tenant mirror table — stores tenant data from nID
-- 2. Tenant sync log — tracks all sync events and their status
-- 3. Tenant redirect flow — handles Communicator → nID → Communicator redirects
--    for tenant creation with error handling and state tracking

-- ---------------------------------------------------------------------------
-- 1. nID tenant mirror: stores tenant data synced from nID
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS nid_tenants (
    id TEXT PRIMARY KEY,                          -- nID tenant ID (cuid)
    nid_tenant_id TEXT NOT NULL UNIQUE,           -- same as id, explicit for clarity
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    data_residency_region TEXT NOT NULL DEFAULT 'global',
    webhook_url TEXT,
    webhook_secret_hash TEXT,                     -- HMAC-SHA256 hash of webhook secret
    webhook_secret_salt TEXT,
    billing_webhook_url TEXT,
    billing_webhook_secret_hash TEXT,
    billing_webhook_secret_salt TEXT,
    
    -- Sync status
    sync_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'synced', 'out_of_sync', 'error'
    last_synced_at TIMESTAMPTZ,
    last_sync_error TEXT DEFAULT '',
    
    -- Connection to Communicator brands
    linked_brand_id TEXT,                         -- which Communicator brand this tenant maps to
    auto_link_enabled BOOLEAN NOT NULL DEFAULT true,
    
    -- Metadata
    nID_created_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_nid_tenants_slug ON nid_tenants(slug);
CREATE INDEX IF NOT EXISTS idx_nid_tenants_brand ON nid_tenants(linked_brand_id);
CREATE INDEX IF NOT EXISTS idx_nid_tenants_sync ON nid_tenants(sync_status);

COMMENT ON TABLE nid_tenants IS 'Mirror of nID tenants for real-time sync. Each row corresponds to a tenant in nID, keeping the Communicator in sync with tenant creation, updates, and profile changes.';

-- ---------------------------------------------------------------------------
-- 2. Tenant sync log: tracks all sync events from nID webhooks
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tenant_sync_log (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    nid_tenant_id TEXT NOT NULL,
    event_type TEXT NOT NULL,                     -- 'tenant.created', 'tenant.updated', 'tenant.deleted'
    event_id TEXT,                                -- nID outbound event ID
    payload JSONB NOT NULL DEFAULT '{}',
    signature TEXT DEFAULT '',
    signature_valid BOOLEAN NOT NULL DEFAULT false,
    processing_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'processed', 'failed', 'skipped'
    error_message TEXT DEFAULT '',
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_sync_nid ON tenant_sync_log(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_sync_event ON tenant_sync_log(event_type);
CREATE INDEX IF NOT EXISTS idx_tenant_sync_status ON tenant_sync_log(processing_status);
CREATE INDEX IF NOT EXISTS idx_tenant_sync_created ON tenant_sync_log(created_at);

COMMENT ON TABLE tenant_sync_log IS 'Audit trail for all tenant sync events received from nID webhooks. Tracks event type, signature validation, and processing status.';

-- ---------------------------------------------------------------------------
-- 3. Tenant redirect flow: tracks Communicator → nID → Communicator redirects
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tenant_redirect_sessions (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    state_token TEXT NOT NULL UNIQUE,             -- OAuth-like state token for redirect validation
    redirect_type TEXT NOT NULL DEFAULT 'create', -- 'create', 'manage', 'link'
    brand_id TEXT,                                -- Communicator brand initiating the redirect
    return_url TEXT NOT NULL,                     -- where to redirect back after nID flow completes
    nID_tenant_id TEXT,                           -- set after successful tenant creation in nID
    status TEXT NOT NULL DEFAULT 'pending',       -- 'pending', 'completed', 'failed', 'expired', 'cancelled'
    error_message TEXT DEFAULT '',
    error_code TEXT,                              -- 'timeout', 'user_cancelled', 'nID_error', 'validation_failed'
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    expires_at TIMESTAMPTZ NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_redirect_state ON tenant_redirect_sessions(state_token);
CREATE INDEX IF NOT EXISTS idx_redirect_brand ON tenant_redirect_sessions(brand_id);
CREATE INDEX IF NOT EXISTS idx_redirect_status ON tenant_redirect_sessions(status);
CREATE INDEX IF NOT EXISTS idx_redirect_expires ON tenant_redirect_sessions(expires_at);

COMMENT ON TABLE tenant_redirect_sessions IS 'Tracks the redirect flow when Communicator sends users to nID for tenant creation or management. Handles state validation, error recovery, and duplicate prevention.';

-- ---------------------------------------------------------------------------
-- 4. nID connection settings: stores the connection configuration
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS nid_connection_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    nID_base_url TEXT NOT NULL,                   -- base URL of nID API
    platform_admin_secret_hash TEXT,              -- HMAC hash of platform admin secret
    platform_admin_secret_salt TEXT,
    webhook_receive_secret TEXT,                  -- secret for verifying incoming nID webhooks
    auto_sync_enabled BOOLEAN NOT NULL DEFAULT true,
    auto_link_new_tenants BOOLEAN NOT NULL DEFAULT true,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 300,  -- polling interval as fallback
    last_poll_at TIMESTAMPTZ,
    connection_status TEXT NOT NULL DEFAULT 'disconnected',  -- 'connected', 'disconnected', 'error'
    last_connection_check_at TIMESTAMPTZ,
    last_error TEXT DEFAULT '',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE nid_connection_settings IS 'Stores the connection configuration between Communicator and nID. Includes platform admin credentials, webhook secrets, and sync settings.';

-- Seed default connection settings
INSERT INTO nid_connection_settings (id, nID_base_url, connection_status)
VALUES ('default', 'http://localhost:4000', 'disconnected')
ON CONFLICT (id) DO NOTHING;
