-- Phase 37: Reply Engine Integration
-- Tracks which knowledge sources were used in each reply for transparency and analytics

CREATE TABLE IF NOT EXISTS reply_knowledge_sources (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    draft_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    source_type TEXT NOT NULL, -- knowledge_base, help_center, document_chunk, audience_scoped, service_catalog
    source_id TEXT,
    source_title TEXT,
    relevance_score FLOAT DEFAULT 0.5,
    used_in_reply BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reply_knowledge_draft ON reply_knowledge_sources(draft_id);
CREATE INDEX IF NOT EXISTS idx_reply_knowledge_conversation ON reply_knowledge_sources(conversation_id);
CREATE INDEX IF NOT EXISTS idx_reply_knowledge_type ON reply_knowledge_sources(source_type);

CREATE TABLE IF NOT EXISTS reply_integration_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    use_tone_of_voice BOOLEAN NOT NULL DEFAULT true,
    use_guidance_rules BOOLEAN NOT NULL DEFAULT true,
    use_audience_scoping BOOLEAN NOT NULL DEFAULT true,
    use_help_center_articles BOOLEAN NOT NULL DEFAULT true,
    use_document_chunks BOOLEAN NOT NULL DEFAULT true,
    max_knowledge_entries INTEGER NOT NULL DEFAULT 30,
    max_document_chunks INTEGER NOT NULL DEFAULT 15,
    max_help_center_articles INTEGER NOT NULL DEFAULT 10,
    track_knowledge_sources BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default settings
INSERT INTO reply_integration_settings (id, org_id, use_tone_of_voice, use_guidance_rules, use_audience_scoping, use_help_center_articles, use_document_chunks)
VALUES ('default-reply-integration', 'default', true, true, true, true, true)
ON CONFLICT (id) DO NOTHING;
