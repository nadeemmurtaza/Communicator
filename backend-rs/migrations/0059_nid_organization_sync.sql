-- Migration 0059: nID Organization Sync
-- Mirrors nID organizations into Communicator for real-time sync,
-- redirect flow, and organization management dashboard.

-- Organizations synced from nID. Each nID organization belongs to a tenant,
-- and is mirrored here so the Communicator can manage the connection.
CREATE TABLE IF NOT EXISTS nid_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nid_organization_id TEXT NOT NULL,
    nid_tenant_id TEXT NOT NULL,
    tenant_slug TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    registration_number TEXT,
    country_code TEXT,
    subject_id TEXT NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    verified_level TEXT,
    webhook_url TEXT,
    sync_status TEXT NOT NULL DEFAULT 'synced',
    last_synced_at TIMESTAMPTZ,
    last_sync_error TEXT NOT NULL DEFAULT '',
    linked_brand_id TEXT REFERENCES brands(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(nid_organization_id),
    UNIQUE(nid_tenant_id, legal_name)
);

CREATE INDEX IF NOT EXISTS idx_nid_orgs_tenant ON nid_organizations(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_nid_orgs_sync_status ON nid_organizations(sync_status);

-- Sync log for organization events from nID webhooks
CREATE TABLE IF NOT EXISTS organization_sync_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nid_organization_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB,
    status TEXT NOT NULL DEFAULT 'success',
    error_message TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_org_sync_log_org ON organization_sync_log(nid_organization_id);
CREATE INDEX IF NOT EXISTS idx_org_sync_log_event ON organization_sync_log(event_type);

-- Redirect sessions for organization creation flow
-- Communicator -> nID -> Communicator
CREATE TABLE IF NOT EXISTS organization_redirect_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_token TEXT NOT NULL UNIQUE,
    redirect_type TEXT NOT NULL DEFAULT 'create',
    return_url TEXT NOT NULL,
    nid_tenant_id TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    max_attempts INTEGER NOT NULL DEFAULT 1,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    error_code TEXT NOT NULL DEFAULT '',
    error_message TEXT NOT NULL DEFAULT '',
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_org_redirect_state ON organization_redirect_sessions(state_token);
CREATE INDEX IF NOT EXISTS idx_org_redirect_status ON organization_redirect_sessions(status);

-- nID organization connection settings
CREATE TABLE IF NOT EXISTS nid_organization_connection (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nID_base_url TEXT NOT NULL DEFAULT 'http://localhost:4000',
    webhook_secret TEXT NOT NULL DEFAULT '',
    auto_sync_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    auto_link_new_organizations BOOLEAN NOT NULL DEFAULT TRUE,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 300,
    connection_status TEXT NOT NULL DEFAULT 'disconnected',
    last_error TEXT NOT NULL DEFAULT '',
    last_health_check_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default connection settings
INSERT INTO nid_organization_connection (id, nID_base_url, auto_sync_enabled, auto_link_new_organizations)
VALUES ('00000000-0000-0000-0000-000000000001', 'http://localhost:4000', TRUE, TRUE)
ON CONFLICT (id) DO NOTHING;

-- Comments for documentation
COMMENT ON TABLE nid_organizations IS 'Mirrored nID organizations for real-time sync with Communicator';
COMMENT ON TABLE organization_sync_log IS 'Audit trail for organization webhook events from nID';
COMMENT ON TABLE organization_redirect_sessions IS 'State tokens for organization creation redirect flow';
COMMENT ON TABLE nid_organization_connection IS 'Connection settings for nID organization sync';
