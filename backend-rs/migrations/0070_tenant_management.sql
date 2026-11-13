-- Phase 58: Tenant Management — Communicator side
--
-- Manages the connection between Communicator and nID tenants.
-- When a tenant is created in nID, it appears here via webhook sync.
-- When a user clicks "Create Tenant" in Communicator, it redirects to nID,
-- creates the tenant, and redirects back with the tenant ID.
--
-- Key features:
-- 1. Tenant records synced from nID via webhooks
-- 2. Tenant status monitoring (sync state, health, last update)
-- 3. Redirect flow tracking for tenant creation
-- 4. Profile update sync from nID to Communicator
-- 5. Duplication prevention via nID tenant ID and slug

-- ---------------------------------------------------------------------------
-- 1. Tenants: synced from nID, primary source of truth is nID
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS nid_tenants (
    id TEXT PRIMARY KEY,                          -- nID tenant ID (cuid)
    nid_tenant_id TEXT NOT NULL UNIQUE,           -- same as id, explicit reference
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    data_residency_region TEXT NOT NULL DEFAULT 'global',
    
    -- Webhook configuration (set in nID, synced here)
    webhook_url TEXT,                             -- nID sends events to this URL
    webhook_secret_hash TEXT,                     -- HMAC hash of the webhook signing secret
    webhook_secret_salt TEXT,
    billing_webhook_url TEXT,

    -- Sync status
    sync_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'synced', 'out_of_sync', 'error'
    last_synced_at TEXT,
    last_sync_error TEXT NOT NULL DEFAULT '',

    -- Linking / provenance
    linked_brand_id TEXT,
    nID_created_at TEXT,

    -- Connection status
    is_active BOOLEAN NOT NULL DEFAULT true,
    connected_at TIMESTAMPTZ,

    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE INDEX IF NOT EXISTS idx_nid_tenants_slug ON nid_tenants(slug);
CREATE INDEX IF NOT EXISTS idx_nid_tenants_sync_status ON nid_tenants(sync_status);
CREATE INDEX IF NOT EXISTS idx_nid_tenants_active ON nid_tenants(is_active);

COMMENT ON TABLE nid_tenants IS 'Synced from nID. Primary source of truth is nID. This table tracks the connection status and sync state for each tenant.';

-- ---------------------------------------------------------------------------
-- 2. Tenant sync log: audit trail of all sync events
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tenant_sync_log (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    nid_tenant_id TEXT NOT NULL,
    event_type TEXT NOT NULL,                     -- 'tenant.created', 'tenant.updated', 'tenant.deleted'
    payload JSONB NOT NULL DEFAULT '{}',
    signature_valid BOOLEAN NOT NULL DEFAULT false,
    processed BOOLEAN NOT NULL DEFAULT false,
    error_message TEXT DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_sync_log_tenant ON tenant_sync_log(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_sync_log_event ON tenant_sync_log(event_type);
CREATE INDEX IF NOT EXISTS idx_tenant_sync_log_processed ON tenant_sync_log(processed);

COMMENT ON TABLE tenant_sync_log IS 'Audit trail of all tenant sync events received from nID webhooks.';

-- ---------------------------------------------------------------------------
-- 3. Tenant redirect sessions: tracks in-progress tenant creation redirects
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tenant_redirect_sessions (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    state_token TEXT NOT NULL UNIQUE,             -- OAuth-like state token for redirect flow
    return_url TEXT NOT NULL,                     -- where to redirect back after tenant creation
    initiated_by TEXT,                            -- user/brand that initiated the redirect
    status TEXT NOT NULL DEFAULT 'pending',       -- 'pending', 'completed', 'expired', 'error'
    nid_tenant_id TEXT,                           -- set after successful tenant creation in nID
    error_message TEXT DEFAULT '',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_tenant_redirect_state ON tenant_redirect_sessions(state_token);
CREATE INDEX IF NOT EXISTS idx_tenant_redirect_status ON tenant_redirect_sessions(status);
CREATE INDEX IF NOT EXISTS idx_tenant_redirect_expires ON tenant_redirect_sessions(expires_at);

COMMENT ON TABLE tenant_redirect_sessions IS 'Tracks in-progress tenant creation redirects from Communicator to nID and back.';

-- ---------------------------------------------------------------------------
-- 4. Tenant profile updates: tracks profile changes from nID
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tenant_profile_updates (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    nid_tenant_id TEXT NOT NULL,
    field_name TEXT NOT NULL,                     -- e.g., 'name', 'webhookUrl'
    old_value JSONB,
    new_value JSONB NOT NULL,
    updated_at_source TIMESTAMPTZ NOT NULL,       -- when the update happened in nID
    synced_to_communicator BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_profile_updates_tenant ON tenant_profile_updates(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_profile_updates_synced ON tenant_profile_updates(synced_to_communicator);

COMMENT ON TABLE tenant_profile_updates IS 'Tracks profile changes received from nID for real-time sync to Communicator.';
