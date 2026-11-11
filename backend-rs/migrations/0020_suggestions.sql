-- Migration 0020: AI-Powered Suggestions (Phase 9)
--
-- Connects the feedback loop (Phase 6) to automatic suggestion generation.
-- When a customer gives negative feedback (thumbs down), the system analyzes
-- the conversation using Claude and creates structured suggestions for:
-- 1. New knowledge base entries for uncovered questions
-- 2. FAQ additions for recurring topics
-- 3. Agent behavior improvements based on patterns
--
-- This closes the self-improvement flywheel: feedback → analysis → suggestion
-- → human review → apply → better AI responses.

CREATE TABLE IF NOT EXISTS suggestions (
    id TEXT PRIMARY KEY,
    suggestion_type TEXT NOT NULL,           -- 'knowledge_gap', 'faq_addition', 'agent_insight'
    title TEXT NOT NULL,                     -- Concise title for the suggestion
    description TEXT NOT NULL,               -- What the issue is and why it matters
    suggested_content TEXT,                  -- JSON: draft KB entry, FAQ Q/A, or insight payload
    source_feedback_id TEXT,                 -- message_feedback.id that triggered this
    source_conversation_id TEXT,             -- conversation context
    source_message_body TEXT,                -- The customer message that triggered it
    confidence REAL NOT NULL DEFAULT 0.5,    -- 0.0-1.0, AI confidence in this suggestion
    category TEXT,                           -- e.g., 'pricing', 'technical', 'policy', 'product'
    status TEXT NOT NULL DEFAULT 'suggested', -- 'suggested', 'approved', 'dismissed', 'applied'
    approved_by TEXT,                        -- Agent name who approved
    approved_at TEXT,                        -- When approved
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_suggestions_status
    ON suggestions(status);

CREATE INDEX IF NOT EXISTS idx_suggestions_type
    ON suggestions(suggestion_type);

CREATE INDEX IF NOT EXISTS idx_suggestions_category
    ON suggestions(category);

CREATE INDEX IF NOT EXISTS idx_suggestions_created
    ON suggestions(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_suggestions_source_feedback
    ON suggestions(source_feedback_id);
