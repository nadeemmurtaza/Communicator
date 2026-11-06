-- Migration 0002: Brands and Organizations hierarchy
-- Introduces Tenant → Organization → Brand hierarchy with BYOK at brand level

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    industry TEXT DEFAULT '',
    website TEXT DEFAULT '',
    email TEXT DEFAULT '',
    phone TEXT DEFAULT '',
    address TEXT DEFAULT '',
    timezone TEXT DEFAULT 'UTC',
    settings TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_orgs_tenant_key ON organizations(tenant_key);

-- Brands table
CREATE TABLE IF NOT EXISTS brands (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT DEFAULT '',
    logo_url TEXT DEFAULT '',
    tagline TEXT DEFAULT '',
    color_scheme TEXT DEFAULT '{}',
    bio TEXT DEFAULT '',
    website TEXT DEFAULT '',
    email TEXT DEFAULT '',
    phone TEXT DEFAULT '',
    address TEXT DEFAULT '',
    social_links TEXT DEFAULT '{}',
    products TEXT DEFAULT '[]',
    services TEXT DEFAULT '[]',
    prices TEXT DEFAULT '{}',
    policies TEXT DEFAULT '{}',
    rules TEXT DEFAULT '{}',
    tone_guidelines TEXT DEFAULT '',
    keywords TEXT DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_brands_org_id ON brands(organization_id);
CREATE INDEX IF NOT EXISTS idx_brands_slug ON brands(slug);

-- Brand credentials table (BYOK - Bring Your Own Keys)
CREATE TABLE IF NOT EXISTS brand_credentials (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    channel_type TEXT NOT NULL,
    credentials TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    last_verified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_brand_creds_brand_id ON brand_credentials(brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_creds_channel ON brand_credentials(channel_type);

-- Brand AI config table (per-brand AI agent settings)
CREATE TABLE IF NOT EXISTS brand_ai_config (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    ai_provider TEXT NOT NULL DEFAULT 'anthropic',
    api_key TEXT NOT NULL DEFAULT '',
    model_id TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
    base_url TEXT DEFAULT '',
    temperature REAL DEFAULT 0.7,
    max_tokens INTEGER DEFAULT 4096,
    system_prompt TEXT DEFAULT '',
    rag_enabled INTEGER NOT NULL DEFAULT 0,
    embedding_provider TEXT DEFAULT '',
    embedding_api_key TEXT DEFAULT '',
    embedding_model TEXT DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_brand_ai_brand_id ON brand_ai_config(brand_id);

-- Brand knowledge base entries (for RAG at brand level)
CREATE TABLE IF NOT EXISTS brand_knowledge_entries (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source_type TEXT DEFAULT 'manual',
    topic TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',
    embedding_vector BLOB,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_brand_kb_brand_id ON brand_knowledge_entries(brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_kb_topic ON brand_knowledge_entries(topic);

-- Add brand_id column to existing tables for brand scoping
ALTER TABLE inboxes ADD COLUMN brand_id TEXT REFERENCES brands(id);
ALTER TABLE conversations ADD COLUMN brand_id TEXT REFERENCES brands(id);
ALTER TABLE messages ADD COLUMN brand_id TEXT REFERENCES brands(id);
ALTER TABLE integration_settings ADD COLUMN organization_id TEXT REFERENCES organizations(id);

CREATE INDEX IF NOT EXISTS idx_inboxes_brand_id ON inboxes(brand_id);
CREATE INDEX IF NOT EXISTS idx_conversations_brand_id ON conversations(brand_id);
CREATE INDEX IF NOT EXISTS idx_messages_brand_id ON messages(brand_id);

-- Create a default organization and brand for existing tenants for backward compatibility
INSERT OR IGNORE INTO organizations (id, tenant_key, name, description)
SELECT 'org_' || substr(tenant_key, 1, 24), tenant_key, 'Primary Organization', 'Default organization for ' || tenant_key
FROM tenants
WHERE tenant_key NOT IN (SELECT tenant_key FROM organizations);

INSERT OR IGNORE INTO brands (id, organization_id, name, slug, description)
SELECT 'brand_' || substr(o.tenant_key, 1, 22), o.id, 'Primary Brand', 'primary-brand', 'Default brand for ' || o.name
FROM organizations o
WHERE o.id NOT IN (SELECT organization_id FROM brands);
