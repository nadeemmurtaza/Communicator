-- Migration 0060: nID Brand Sync
-- Mirrors nID Brand entities in the Communicator for real-time sync.
-- When a brand is created/updated/deleted in nID, a webhook fires to the
-- Communicator which upserts/removes the local mirror row.

-- Mirrored nID brands (one row per nID brand)
CREATE TABLE IF NOT EXISTS nid_brands (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nid_brand_id    TEXT NOT NULL UNIQUE,          -- nID Brand.id (cuid)
    nid_org_id      TEXT NOT NULL,                 -- nID Organization.id
    nid_tenant_id   TEXT NOT NULL,                 -- nID Tenant.id
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL,
    description     TEXT,
    bio             TEXT,
    website         TEXT,
    email           TEXT,
    phone           TEXT,
    primary_color   TEXT,
    secondary_color TEXT,
    active          BOOLEAN NOT NULL DEFAULT true,
    synced_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_nid_brand UNIQUE (nid_brand_id),
    CONSTRAINT uq_tenant_slug UNIQUE (nid_tenant_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_nid_brands_tenant ON nid_brands(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_nid_brands_org ON nid_brands(nid_org_id);

-- Sync audit log
CREATE TABLE IF NOT EXISTS brand_sync_log (
    id          BIGSERIAL PRIMARY KEY,
    nid_brand_id TEXT,
    event_type  TEXT NOT NULL,          -- brand.created, brand.updated, brand.deleted
    status      TEXT NOT NULL DEFAULT 'success',  -- success, error
    error_msg   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_brand_sync_log_brand ON brand_sync_log(nid_brand_id);

-- Redirect sessions for Communicator -> nID -> Communicator brand creation flow
CREATE TABLE IF NOT EXISTS brand_redirect_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_token     TEXT NOT NULL UNIQUE,
    nid_tenant_id   TEXT NOT NULL,
    nid_org_id      TEXT,
    communicator_brand_id UUID,
    redirect_uri    TEXT NOT NULL,
    max_attempts    INT NOT NULL DEFAULT 1,
    attempts        INT NOT NULL DEFAULT 0,
    error_code      TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_brand_redirect_state ON brand_redirect_sessions(state_token);

-- nID brand connection settings (webhook URL, secret, polling interval)
CREATE TABLE IF NOT EXISTS nid_brand_connection (
    id              BIGSERIAL PRIMARY KEY,
    webhook_url     TEXT,
    webhook_secret  TEXT,
    polling_interval_sec INT NOT NULL DEFAULT 300,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    last_health_check TIMESTAMPTZ,
    health_status   TEXT DEFAULT 'unknown',
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
