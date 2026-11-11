-- Phase 22: CRM Sync â€” Salesforce/HubSpot bidirectional sync
-- Provides integration with major CRM platforms to sync leads, contacts,
-- and accounts/companies bidirectionally. Mirrors Intercom's CRM sync feature.

-- CRM connection configurations (per-organization CRM integrations)
CREATE TABLE IF NOT EXISTS crm_connections (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    provider TEXT NOT NULL, -- 'salesforce' | 'hubspot' | 'pipedrive'
    status TEXT NOT NULL DEFAULT 'disconnected', -- 'disconnected' | 'connecting' | 'connected' | 'error'
    auth_type TEXT NOT NULL DEFAULT 'oauth', -- 'oauth' | 'api_key' | 'oauth2'
    credentials TEXT, -- encrypted JSON: access_token, refresh_token, instance_url, etc.
    config TEXT, -- JSON: sync directions, field mappings, object types
    last_sync_at TEXT,
    last_error TEXT,
    connected_at TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_crm_connections_org ON crm_connections(organization_id);
CREATE INDEX IF NOT EXISTS idx_crm_connections_provider ON crm_connections(provider);
CREATE INDEX IF NOT EXISTS idx_crm_connections_status ON crm_connections(status);

-- CRM sync logs (track each sync operation)
CREATE TABLE IF NOT EXISTS crm_sync_logs (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    direction TEXT NOT NULL, -- 'inbound' | 'outbound'
    object_type TEXT NOT NULL, -- 'lead' | 'contact' | 'account' | 'deal'
    record_count BOOLEAN NOT NULL DEFAULT false,
    success_count BOOLEAN NOT NULL DEFAULT false,
    error_count BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'running', -- 'running' | 'completed' | 'partial' | 'failed'
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (NOW()),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_crm_sync_logs_connection ON crm_sync_logs(connection_id);
CREATE INDEX IF NOT EXISTS idx_crm_sync_logs_started ON crm_sync_logs(started_at);

-- CRM object mappings (field-level mapping between Communicator and CRM)
CREATE TABLE IF NOT EXISTS crm_field_mappings (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    object_type TEXT NOT NULL, -- 'lead' | 'contact' | 'account'
    local_field TEXT NOT NULL,
    crm_field TEXT NOT NULL,
    direction TEXT NOT NULL DEFAULT 'both', -- 'inbound' | 'outbound' | 'both'
    transform TEXT, -- JSON: transformation rules
    is_required BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_crm_field_mappings_connection ON crm_field_mappings(connection_id);
CREATE INDEX IF NOT EXISTS idx_crm_field_mappings_object ON crm_field_mappings(object_type);

-- CRM external references (link local records to CRM records)
CREATE TABLE IF NOT EXISTS crm_external_refs (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    local_type TEXT NOT NULL, -- 'lead' | 'contact' | 'customer'
    local_id TEXT NOT NULL,
    crm_object_type TEXT NOT NULL, -- 'Lead' | 'Contact' | 'Account' | 'Company'
    crm_record_id TEXT NOT NULL,
    crm_record_url TEXT,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sync_status TEXT NOT NULL DEFAULT 'synced', -- 'synced' | 'pending' | 'conflict' | 'error'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_crm_external_refs_connection ON crm_external_refs(connection_id);
CREATE INDEX IF NOT EXISTS idx_crm_external_refs_local ON crm_external_refs(local_type, local_id);
CREATE INDEX IF NOT EXISTS idx_crm_external_refs_crm ON crm_external_refs(crm_object_type, crm_record_id);

-- Default field mappings for Salesforce
INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-sf-lead-name', 'conn-default', 'lead', 'name', 'Company', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-sf-lead-name');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-sf-lead-email', 'conn-default', 'lead', 'email', 'Email', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-sf-lead-email');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-sf-lead-phone', 'conn-default', 'lead', 'phone', 'Phone', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-sf-lead-phone');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-sf-contact-first', 'conn-default', 'contact', 'first_name', 'FirstName', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-sf-contact-first');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-sf-contact-last', 'conn-default', 'contact', 'last_name', 'LastName', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-sf-contact-last');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-sf-contact-email', 'conn-default', 'contact', 'email', 'Email', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-sf-contact-email');

-- Default field mappings for HubSpot
INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-hs-lead-name', 'conn-default', 'lead', 'name', 'company', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-hs-lead-name');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-hs-lead-email', 'conn-default', 'lead', 'email', 'email', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-hs-lead-email');

INSERT INTO crm_field_mappings (id, connection_id, object_type, local_field, crm_field, direction)
SELECT 'crm-map-hs-contact-email', 'conn-default', 'contact', 'email', 'email', 'both'
WHERE NOT EXISTS (SELECT 1 FROM crm_field_mappings WHERE id = 'crm-map-hs-contact-email');
