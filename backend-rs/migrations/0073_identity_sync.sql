-- Synced read-model for the identity nID owns (COMMUNICATION.md section 1,
-- async event-bus path; "who owns which ID" table). Populated only by
-- `crates/identity-sync`'s NATS JetStream consumer — never write to these
-- tables from anywhere else, and never treat them as this repo's source of
-- truth for these IDs; nID is.
--
-- Deliberately separate from the pre-existing `nid_tenants` /
-- `nid_tenant_mapping` / `nid_organization_mapping` tables (migrations
-- 0064, 0069): those back a webhook-based (REST+HMAC) design that nothing
-- in nID currently calls, and 0064's views assume a literally shared
-- Postgres instance with nID, which this repo's own `communicator`
-- database is not. Flagged in RULE_VIOLATIONS_AUDIT.md rather than
-- migrated/removed here.

CREATE TABLE IF NOT EXISTS identity_sync_processed_event (
    event_id TEXT PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS synced_tenant (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    data_residency_region TEXT NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT false,
    last_event_id TEXT NOT NULL,
    last_event_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_synced_tenant_slug ON synced_tenant(slug);

CREATE TABLE IF NOT EXISTS synced_organization (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    registration_number TEXT,
    country_code TEXT,
    deleted BOOLEAN NOT NULL DEFAULT false,
    last_event_id TEXT NOT NULL,
    last_event_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_synced_organization_tenant ON synced_organization(tenant_id);

CREATE TABLE IF NOT EXISTS synced_brand (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    deleted BOOLEAN NOT NULL DEFAULT false,
    last_event_id TEXT NOT NULL,
    last_event_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_synced_brand_organization ON synced_brand(organization_id);

-- `customer_id` per COMMUNICATION.md is a person's identity, derived from
-- nID's Account — not a separate nID table.
CREATE TABLE IF NOT EXISTS synced_customer (
    id TEXT PRIMARY KEY,
    phone TEXT,
    email TEXT,
    name TEXT,
    deleted BOOLEAN NOT NULL DEFAULT false,
    last_event_id TEXT NOT NULL,
    last_event_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_synced_customer_phone ON synced_customer(phone);
