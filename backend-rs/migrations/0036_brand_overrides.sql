-- Migration 0036: Brand Overrides in Workflows (PostgreSQL)
-- Enables dynamic brand switching mid-conversation or mid-workflow execution.
-- Mirrors Intercom's "Brand overrides in workflows" capability where a
-- conversation can switch brands mid-flow (e.g., from Coke to Pepsi sub-brand).

-- ============================================================================
-- 1. BRAND OVERRIDE RULES
-- ============================================================================
-- Defines when and how a brand switch occurs. Rules can be triggered by
-- workflow steps, conversation keywords, customer segment, or manually.
CREATE TABLE IF NOT EXISTS brand_override_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    trigger_type TEXT NOT NULL DEFAULT 'workflow',
    trigger_config TEXT NOT NULL DEFAULT '{}',
    from_brand_id TEXT,
    to_brand_id TEXT NOT NULL,
    conditions TEXT NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    auto_switch_kb BOOLEAN NOT NULL DEFAULT true,
    auto_switch_ai_config BOOLEAN NOT NULL DEFAULT true,
    auto_switch_messenger BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (from_brand_id) REFERENCES brands(id) ON DELETE SET NULL,
    FOREIGN KEY (to_brand_id) REFERENCES brands(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_brand_override_org ON brand_override_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_brand_override_from_brand ON brand_override_rules(from_brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_override_to_brand ON brand_override_rules(to_brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_override_active ON brand_override_rules(is_active);

-- ============================================================================
-- 2. BRAND SWITCH LOG
-- ============================================================================
-- Tracks every brand switch event for auditing and analytics.
CREATE TABLE IF NOT EXISTS brand_switch_log (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    conversation_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    from_brand_id TEXT NOT NULL,
    to_brand_id TEXT NOT NULL,
    trigger_type TEXT NOT NULL DEFAULT 'manual',
    trigger_source TEXT DEFAULT '',
    rule_id TEXT,
    switched_by TEXT DEFAULT '',
    switched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (from_brand_id) REFERENCES brands(id) ON DELETE SET NULL,
    FOREIGN KEY (to_brand_id) REFERENCES brands(id) ON DELETE SET NULL,
    FOREIGN KEY (rule_id) REFERENCES brand_override_rules(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_brand_switch_conversation ON brand_switch_log(conversation_id);
CREATE INDEX IF NOT EXISTS idx_brand_switch_contact ON brand_switch_log(contact_id);
CREATE INDEX IF NOT EXISTS idx_brand_switch_switched_at ON brand_switch_log(switched_at);
CREATE INDEX IF NOT EXISTS idx_brand_switch_rule ON brand_switch_log(rule_id);

-- ============================================================================
-- 3. BRAND OVERRIDE SETTINGS
-- ============================================================================
-- Per-organization settings for brand override behavior.
CREATE TABLE IF NOT EXISTS brand_override_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    allow_cross_org_brands BOOLEAN NOT NULL DEFAULT false,
    require_approval BOOLEAN NOT NULL DEFAULT false,
    log_all_switches BOOLEAN NOT NULL DEFAULT true,
    default_fallback_brand_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (default_fallback_brand_id) REFERENCES brands(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_brand_override_settings_org ON brand_override_settings(organization_id);

-- Seed default settings
INSERT INTO brand_override_settings (id, organization_id, enabled, allow_cross_org_brands, require_approval, log_all_switches)
VALUES ('brand-override-default', 'org-default', true, false, false, true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 4. BRAND HIERARCHY (Parent-Sub-brand relationships)
-- ============================================================================
-- Models parent-brand to sub-brand relationships for cascading overrides.
CREATE TABLE IF NOT EXISTS brand_hierarchy (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    parent_brand_id TEXT NOT NULL,
    sub_brand_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL DEFAULT 'sub_brand',
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (parent_brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (sub_brand_id) REFERENCES brands(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_brand_hierarchy_unique ON brand_hierarchy(parent_brand_id, sub_brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_hierarchy_parent ON brand_hierarchy(parent_brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_hierarchy_sub ON brand_hierarchy(sub_brand_id);
