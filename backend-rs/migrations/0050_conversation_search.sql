-- Phase 42: Conversation Search & Discovery
-- Full-text search across conversations, messages, and contacts with saved searches
-- and search analytics.

-- Saved Searches (user-defined search queries for quick access)
CREATE TABLE IF NOT EXISTS saved_searches (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    name TEXT NOT NULL,
    query TEXT NOT NULL,
    search_type TEXT NOT NULL DEFAULT 'conversation', -- conversation, message, contact, all
    filters JSONB NOT NULL DEFAULT '{}', -- channel, date range, status, topic, etc.
    created_by TEXT NOT NULL DEFAULT 'system',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_pinned BOOLEAN NOT NULL DEFAULT false,
    usage_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_saved_search_org ON saved_searches(org_id);
CREATE INDEX IF NOT EXISTS idx_saved_search_pinned ON saved_searches(is_pinned);
CREATE INDEX IF NOT EXISTS idx_saved_search_type ON saved_searches(search_type);
CREATE INDEX IF NOT EXISTS idx_saved_search_created ON saved_searches(created_at);

-- Search History (tracks all searches for analytics)
CREATE TABLE IF NOT EXISTS search_history (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    query TEXT NOT NULL,
    search_type TEXT NOT NULL DEFAULT 'all',
    result_count INTEGER NOT NULL DEFAULT 0,
    filters JSONB NOT NULL DEFAULT '{}',
    performed_by TEXT NOT NULL DEFAULT 'system',
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_search_history_org ON search_history(org_id);
CREATE INDEX IF NOT EXISTS idx_search_history_performed ON search_history(performed_at);
CREATE INDEX IF NOT EXISTS idx_search_history_type ON search_history(search_type);
CREATE INDEX IF NOT EXISTS idx_search_history_query ON search_history(query);

-- Search Analytics (aggregated search metrics over time)
CREATE TABLE IF NOT EXISTS search_analytics (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    total_searches INTEGER NOT NULL DEFAULT 0,
    avg_result_count DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    zero_result_searches INTEGER NOT NULL DEFAULT 0,
    top_queries JSONB NOT NULL DEFAULT '[]',
    top_search_types JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_search_analytics_org ON search_analytics(org_id);
CREATE INDEX IF NOT EXISTS idx_search_analytics_window ON search_analytics(window_start, window_end);

-- Search Settings (per-org configuration)
CREATE TABLE IF NOT EXISTS search_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    max_results INTEGER NOT NULL DEFAULT 50,
    search_message_content BOOLEAN NOT NULL DEFAULT true,
    search_contact_details BOOLEAN NOT NULL DEFAULT true,
    search_conversation_metadata BOOLEAN NOT NULL DEFAULT true,
    track_search_history BOOLEAN NOT NULL DEFAULT true,
    auto_suggest BOOLEAN NOT NULL DEFAULT true,
    min_query_length INTEGER NOT NULL DEFAULT 2,
    highlight_matches BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default settings
INSERT INTO search_settings (id, org_id) VALUES
    ('default', 'default')
ON CONFLICT (id) DO NOTHING;
