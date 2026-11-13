-- Migration 0035: Audience-Based Knowledge Scoping (PostgreSQL)
-- Links customer segments to specific knowledge base subsets so different
-- customer groups see different AI knowledge. Mirrors Intercom's "Audiences
-- control what Fin knows" capability.

-- ============================================================================
-- 1. AUDIENCE KB LINKS
-- ============================================================================
-- Maps segments (audiences) to specific KB entries. When a contact belongs
-- to a segment, the AI only draws from the linked KB entries for that segment.
CREATE TABLE IF NOT EXISTS audience_kb_links (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    segment_id TEXT NOT NULL,
    kb_entry_id TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (segment_id) REFERENCES contact_segments(id) ON DELETE CASCADE,
    FOREIGN KEY (kb_entry_id) REFERENCES knowledge_base_entries(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_audience_kb_unique ON audience_kb_links(segment_id, kb_entry_id);
CREATE INDEX IF NOT EXISTS idx_audience_kb_segment ON audience_kb_links(segment_id);
CREATE INDEX IF NOT EXISTS idx_audience_kb_entry ON audience_kb_links(kb_entry_id);

-- ============================================================================
-- 2. AUDIENCE KB SETTINGS
-- ============================================================================
-- Per-organization settings for audience-based knowledge scoping.
CREATE TABLE IF NOT EXISTS audience_kb_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    fallback_to_global_kb BOOLEAN NOT NULL DEFAULT true,
    merge_segments BOOLEAN NOT NULL DEFAULT true,
    default_scope TEXT NOT NULL DEFAULT 'all',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_audience_kb_settings_org ON audience_kb_settings(organization_id);

-- Seed default settings
INSERT INTO audience_kb_settings (id, organization_id, enabled, fallback_to_global_kb, merge_segments, default_scope)
VALUES ('audience-kb-default', 'org-default', true, true, true, 'all')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 3. AUDIENCE KB USAGE LOG
-- ============================================================================
-- Tracks which audience KB entries were used for which conversations to
-- measure the impact of audience-based scoping.
CREATE TABLE IF NOT EXISTS audience_kb_usage_log (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    conversation_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    segment_id TEXT NOT NULL,
    kb_entry_id TEXT NOT NULL,
    query_text TEXT NOT NULL DEFAULT '',
    relevance_score REAL NOT NULL DEFAULT 0.5,
    used_in_response BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (segment_id) REFERENCES contact_segments(id) ON DELETE SET NULL,
    FOREIGN KEY (kb_entry_id) REFERENCES knowledge_base_entries(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_audience_kb_usage_conversation ON audience_kb_usage_log(conversation_id);
CREATE INDEX IF NOT EXISTS idx_audience_kb_usage_segment ON audience_kb_usage_log(segment_id);
CREATE INDEX IF NOT EXISTS idx_audience_kb_usage_created ON audience_kb_usage_log(created_at);

-- ============================================================================
-- 4. SEGMENT CONTACT MEMBERSHIP
-- ============================================================================
-- Explicit membership table linking contacts to segments. Segments can be
-- dynamic (evaluated via filters) or static (explicit membership).
CREATE TABLE IF NOT EXISTS segment_contact_memberships (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    segment_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    membership_type TEXT NOT NULL DEFAULT 'dynamic',
    matched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (segment_id) REFERENCES contact_segments(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_segment_contact_unique ON segment_contact_memberships(segment_id, contact_id);
CREATE INDEX IF NOT EXISTS idx_segment_contact_segment ON segment_contact_memberships(segment_id);
CREATE INDEX IF NOT EXISTS idx_segment_contact_contact ON segment_contact_memberships(contact_id);
