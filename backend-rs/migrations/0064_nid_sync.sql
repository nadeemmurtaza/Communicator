-- Migration 0032: nID Sync System (PostgreSQL)
-- Both nID and Communicator share the same PostgreSQL database.
-- This migration creates linking tables that join nID identity tables
-- directly with Communicator commerce tables.

-- ============================================================================
-- 1. NID ACCOUNT LINKING
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_account_links (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    team_member_id UUID NOT NULL UNIQUE,
    nid_account_id TEXT NOT NULL UNIQUE,
    nid_phone TEXT NOT NULL UNIQUE,
    nid_email TEXT,
    nid_profile_id TEXT,
    nid_profile_slug TEXT,
    nid_profile_display_name TEXT,
    nid_profile_photo_key TEXT,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (team_member_id) REFERENCES team_members(id)
);
CREATE INDEX IF NOT EXISTS idx_nid_links_team_member ON nid_account_links(team_member_id);
CREATE INDEX IF NOT EXISTS idx_nid_links_phone ON nid_account_links(nid_phone);

-- ============================================================================
-- 2. NID TENANT MAPPING
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_tenant_mapping (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    communicator_tenant_key TEXT NOT NULL UNIQUE,
    nid_tenant_id TEXT NOT NULL UNIQUE,
    nid_tenant_name TEXT NOT NULL,
    nid_tenant_slug TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_nid_tenant_mapping_communicator ON nid_tenant_mapping(communicator_tenant_key);

-- ============================================================================
-- 3. NID ORGANIZATION MAPPING
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_organization_mapping (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    communicator_org_id TEXT NOT NULL UNIQUE,
    nid_org_id TEXT NOT NULL UNIQUE,
    nid_tenant_id TEXT NOT NULL,
    nid_legal_name TEXT NOT NULL,
    nid_registration_number TEXT,
    nid_country_code TEXT,
    nid_subject_id TEXT NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verified_level TEXT,
    verified_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (communicator_org_id) REFERENCES organizations(id)
);
CREATE INDEX IF NOT EXISTS idx_nid_org_mapping_communicator ON nid_organization_mapping(communicator_org_id);
CREATE INDEX IF NOT EXISTS idx_nid_org_mapping_tenant ON nid_organization_mapping(nid_tenant_id);

-- ============================================================================
-- 4. NID MEMBERSHIP TRACKING
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_organization_memberships (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    team_member_id UUID NOT NULL,
    nid_org_id TEXT NOT NULL,
    nid_membership_id TEXT NOT NULL,
    role TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (team_member_id) REFERENCES team_members(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_nid_membership_unique ON nid_organization_memberships(team_member_id, nid_org_id);
CREATE INDEX IF NOT EXISTS idx_nid_membership_team ON nid_organization_memberships(team_member_id);

-- ============================================================================
-- 5. NID BILLING ACCOUNT LINKING
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_billing_account_links (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    nid_billing_account_id TEXT NOT NULL UNIQUE,
    nid_tenant_id TEXT NOT NULL,
    communicator_brand_id TEXT,
    communicator_org_id TEXT,
    currency TEXT NOT NULL DEFAULT 'PKR',
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    credit_balance_minor BIGINT NOT NULL DEFAULT 0,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (communicator_brand_id) REFERENCES brands(id),
    FOREIGN KEY (communicator_org_id) REFERENCES organizations(id)
);
CREATE INDEX IF NOT EXISTS idx_nid_billing_brand ON nid_billing_account_links(communicator_brand_id);
CREATE INDEX IF NOT EXISTS idx_nid_billing_org ON nid_billing_account_links(communicator_org_id);

-- ============================================================================
-- 6. NID SUBSCRIPTION SYNC
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_subscription_sync (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    nid_subscription_id TEXT NOT NULL UNIQUE,
    nid_tenant_id TEXT NOT NULL,
    nid_billing_account_id TEXT NOT NULL,
    nid_plan_id TEXT NOT NULL,
    plan_key TEXT NOT NULL,
    plan_name TEXT NOT NULL,
    plan_mode TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'INCOMPLETE',
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_nid_subscription_tenant ON nid_subscription_sync(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_nid_subscription_status ON nid_subscription_sync(status);

-- ============================================================================
-- 7. NID INVOICE SYNC
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_invoice_sync (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    nid_invoice_id TEXT NOT NULL UNIQUE,
    nid_tenant_id TEXT NOT NULL,
    nid_billing_account_id TEXT NOT NULL,
    invoice_number TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    currency TEXT NOT NULL,
    subtotal_minor BIGINT NOT NULL,
    tax_minor BIGINT NOT NULL DEFAULT 0,
    total_minor BIGINT NOT NULL,
    amount_paid_minor BIGINT NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ,
    period_end TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    issued_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_nid_invoice_tenant ON nid_invoice_sync(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_nid_invoice_number ON nid_invoice_sync(invoice_number);
CREATE INDEX IF NOT EXISTS idx_nid_invoice_status ON nid_invoice_sync(status);

-- ============================================================================
-- 8. NID PAYMENT SYNC
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_payment_sync (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    nid_payment_id TEXT NOT NULL UNIQUE,
    nid_tenant_id TEXT NOT NULL,
    nid_payment_intent_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_transaction_id TEXT NOT NULL,
    amount_minor BIGINT NOT NULL,
    currency TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    method_type TEXT,
    paid_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_nid_payment_tenant ON nid_payment_sync(nid_tenant_id);
CREATE INDEX IF NOT EXISTS idx_nid_payment_status ON nid_payment_sync(status);

-- ============================================================================
-- 9. NID ORDER PLATFORM TRACKING
-- ============================================================================
CREATE TABLE IF NOT EXISTS order_platform_info (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    order_id TEXT NOT NULL UNIQUE,
    platform_name TEXT NOT NULL,
    platform_type TEXT NOT NULL DEFAULT 'brand',
    brand_id TEXT,
    brand_name TEXT NOT NULL,
    shop_name TEXT,
    platform_logo_url TEXT,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id),
    FOREIGN KEY (brand_id) REFERENCES brands(id)
);
CREATE INDEX IF NOT EXISTS idx_order_platform_order ON order_platform_info(order_id);
CREATE INDEX IF NOT EXISTS idx_order_platform_brand ON order_platform_info(brand_id);

-- ============================================================================
-- 10. NID PROFILE SYNC CACHE
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_profile_cache (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    nid_profile_id TEXT NOT NULL UNIQUE,
    nid_account_id TEXT,
    nid_organization_id TEXT,
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    headline TEXT,
    category TEXT NOT NULL DEFAULT 'OTHER',
    bio TEXT,
    photo_key TEXT,
    phone TEXT,
    whatsapp TEXT,
    public_email TEXT,
    city TEXT,
    country_code TEXT,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    is_published BOOLEAN NOT NULL DEFAULT false,
    is_personal BOOLEAN NOT NULL DEFAULT true,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_nid_profile_cache_account ON nid_profile_cache(nid_account_id);
CREATE INDEX IF NOT EXISTS idx_nid_profile_cache_org ON nid_profile_cache(nid_organization_id);
CREATE INDEX IF NOT EXISTS idx_nid_profile_cache_slug ON nid_profile_cache(slug);

-- ============================================================================
-- 11. SYNC STATUS TRACKING
-- ============================================================================
CREATE TABLE IF NOT EXISTS nid_sync_status (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    entity_type TEXT NOT NULL UNIQUE,
    last_synced_at TIMESTAMPTZ,
    last_sync_status TEXT NOT NULL DEFAULT 'idle',
    last_error TEXT,
    records_synced BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO nid_sync_status (id, entity_type, last_sync_status) VALUES
('sync_accounts', 'accounts', 'idle'),
('sync_profiles', 'profiles', 'idle'),
('sync_organizations', 'organizations', 'idle'),
('sync_billing', 'billing', 'idle'),
('sync_subscriptions', 'subscriptions', 'idle'),
('sync_invoices', 'invoices', 'idle'),
('sync_payments', 'payments', 'idle')
ON CONFLICT (entity_type) DO NOTHING;

-- ============================================================================
-- 12. VIEWS: Direct nID table access (shared PostgreSQL)
-- ============================================================================

-- View: Personal profile with nID account data
CREATE OR REPLACE VIEW v_personal_profile AS
SELECT
    p.id AS profile_id,
    p.slug,
    p.display_name,
    p.headline,
    p.category,
    p.bio,
    p.photo_key,
    p.phone,
    p.whatsapp,
    p.public_email,
    p.city,
    p.country_code,
    p.timezone,
    p.is_published,
    a.id AS account_id,
    a.phone AS account_phone,
    a.email AS account_email
FROM profile p
JOIN account a ON p.account_id = a.id
WHERE p.organization_id IS NULL;

-- View: Work profiles (organization brand profiles)
CREATE OR REPLACE VIEW v_work_profile AS
SELECT
    p.id AS profile_id,
    p.slug,
    p.display_name,
    p.headline,
    p.category,
    o.id AS organization_id,
    o.tenant_id,
    o.legal_name,
    o.registration_number,
    o.country_code,
    s.id AS subject_id,
    s.type AS subject_type
FROM profile p
JOIN organization o ON p.organization_id = o.id
JOIN subject s ON o.subject_id = s.id;

-- View: User's organizations via membership
CREATE OR REPLACE VIEW v_user_organizations AS
SELECT DISTINCT
    om.account_id,
    om.organization_id,
    om.role,
    om.status AS membership_status,
    om.is_admin,
    o.legal_name,
    o.registration_number,
    o.country_code,
    o.tenant_id,
    s.id AS subject_id
FROM organization_membership om
JOIN organization o ON om.organization_id = o.id
JOIN subject s ON o.subject_id = s.id
WHERE om.status = 'ACTIVE';

-- View: Invoices with billing details
CREATE OR REPLACE VIEW v_invoice_details AS
SELECT
    i.id,
    i.number AS invoice_number,
    i.status,
    i.currency,
    i.subtotal_minor,
    i.tax_minor,
    i.total_minor,
    i.amount_paid_minor,
    i.period_start,
    i.period_end,
    i.due_at,
    i.issued_at,
    i.paid_at,
    i.tenant_id,
    ba.id AS billing_account_id,
    t.name AS tenant_name
FROM invoice i
JOIN billing_account ba ON i.billing_account_id = ba.id
JOIN tenant t ON i.tenant_id = t.id;

-- View: Payments with intent details
CREATE OR REPLACE VIEW v_payment_details AS
SELECT
    p.id,
    p.provider,
    p.provider_transaction_id,
    p.amount_minor,
    p.currency,
    p.status,
    p.method_type,
    p.paid_at,
    p.tenant_id,
    pi.id AS payment_intent_id,
    pi.purpose,
    pi.provider AS intent_provider,
    ba.id AS billing_account_id,
    t.name AS tenant_name
FROM payment p
JOIN payment_intent pi ON p.payment_intent_id = pi.id
JOIN billing_account ba ON pi.billing_account_id = ba.id
JOIN tenant t ON p.tenant_id = t.id;

-- View: Active subscriptions
CREATE OR REPLACE VIEW v_subscription_details AS
SELECT
    s.id,
    s.status,
    s.current_period_start,
    s.current_period_end,
    s.cancel_at_period_end,
    s.tenant_id,
    ba.id AS billing_account_id,
    pl.id AS plan_id,
    pl.key AS plan_key,
    pl.name AS plan_name,
    pl.mode AS plan_mode,
    pl.currency AS plan_currency,
    pl.unit_amount_minor,
    bs.key AS service_key,
    bs.name AS service_name,
    t.name AS tenant_name
FROM subscription s
JOIN billing_account ba ON s.billing_account_id = ba.id
JOIN plan pl ON s.plan_id = pl.id
JOIN billing_service bs ON pl.service_id = bs.id
JOIN tenant t ON s.tenant_id = t.id;
