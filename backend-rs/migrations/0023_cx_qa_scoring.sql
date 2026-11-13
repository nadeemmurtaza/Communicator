-- Phase 16: CX Scoring & Automated QA
-- AI-powered conversation quality scoring that evaluates support quality
-- across every conversation without requiring customer surveys.
-- Provides 5x more coverage than traditional CSAT.

CREATE TABLE IF NOT EXISTS qa_scores (
    id TEXT PRIMARY KEY,
    conversation_id UUID NOT NULL,
    
    -- Overall QA score (0-100)
    overall_score REAL NOT NULL,
    
    -- Dimension scores (0-100)
    accuracy_score REAL NOT NULL,           -- Factual accuracy of responses
    empathy_score REAL NOT NULL,            -- Tone and empathy
    resolution_score REAL NOT NULL,         -- Did it resolve the issue?
    compliance_score REAL NOT NULL,         -- Followed policies and guardrails
    timeliness_score REAL NOT NULL,         -- Response time appropriateness
    
    -- Detailed evaluation
    evaluation_summary TEXT NOT NULL DEFAULT '',
    
    -- Specific issues found
    issues TEXT NOT NULL DEFAULT '[]',      -- JSON array of issue strings
    
    -- Recommendations for improvement
    recommendations TEXT NOT NULL DEFAULT '[]',  -- JSON array of recommendation strings
    
    -- Whether this was auto-scored or manually reviewed
    is_auto_scored BOOLEAN NOT NULL DEFAULT true,
    
    -- Who/what performed the scoring
    scored_by TEXT,
    
    -- When the scoring was performed
    scored_at TEXT NOT NULL DEFAULT (NOW()),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_qa_scores_conversation ON qa_scores(conversation_id);
CREATE INDEX IF NOT EXISTS idx_qa_scores_scored_at ON qa_scores(scored_at);
CREATE INDEX IF NOT EXISTS idx_qa_scores_overall ON qa_scores(overall_score);

-- CX Score aggregates over time windows
CREATE TABLE IF NOT EXISTS cx_score_snapshots (
    id TEXT PRIMARY KEY,
    
    -- Time window for this snapshot
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    
    -- Overall CX score (0-100)
    overall_cx_score REAL NOT NULL,
    
    -- Dimension averages
    avg_accuracy REAL NOT NULL,
    avg_empathy REAL NOT NULL,
    avg_resolution REAL NOT NULL,
    avg_compliance REAL NOT NULL,
    avg_timeliness REAL NOT NULL,
    
    -- Number of conversations scored in this window
    conversations_scored BOOLEAN NOT NULL DEFAULT false,
    
    -- Top issues in this window
    top_issues TEXT NOT NULL DEFAULT '[]',
    
    -- Trend direction compared to previous window
    trend_direction TEXT NOT NULL DEFAULT 'stable',  -- 'improving', 'declining', 'stable'
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cx_snapshots_window ON cx_score_snapshots(window_start, window_end);

-- QA scoring rules configuration
CREATE TABLE IF NOT EXISTS qa_scoring_rules (
    id TEXT PRIMARY KEY,
    
    -- Rule name
    name TEXT NOT NULL,
    
    -- Rule description
    description TEXT NOT NULL DEFAULT '',
    
    -- Dimension this rule evaluates
    dimension TEXT NOT NULL,  -- 'accuracy', 'empathy', 'resolution', 'compliance', 'timeliness'
    
    -- Check condition (JSON)
    condition TEXT NOT NULL DEFAULT '{}',
    
    -- Score impact (-100 to +100)
    score_impact REAL NOT NULL DEFAULT 0,
    
    -- Whether this rule is enabled
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    
    -- Severity level
    severity TEXT NOT NULL DEFAULT 'medium',  -- 'low', 'medium', 'high', 'critical'
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default QA scoring rules
INSERT INTO qa_scoring_rules (id, name, description, dimension, condition, score_impact, severity) VALUES
    (uuid(), 'Factual Accuracy Check', 'Response contains factual claims not supported by knowledge base', 'accuracy', '{"check": "grounding_score", "threshold": 0.7}', -20, 'high'),
    (uuid(), 'Empathy Language', 'Response uses empathetic language when customer expresses frustration', 'empathy', '{"check": "sentiment_mismatch", "threshold": 0.8}', -15, 'medium'),
    (uuid(), 'Resolution Confirmation', 'Conversation ends with clear resolution confirmation', 'resolution', '{"check": "resolution_type", "required": true}', -25, 'high'),
    (uuid(), 'Policy Compliance', 'Response follows brand policies and guardrails', 'compliance', '{"check": "policy_violation", "threshold": 0.9}', -30, 'critical'),
    (uuid(), 'Response Time', 'Response delivered within acceptable time window', 'timeliness', '{"check": "response_seconds", "threshold": 300}', -10, 'low'),
    (uuid(), 'Greeting Quality', 'Conversation begins with appropriate greeting', 'empathy', '{"check": "has_greeting", "required": true}', -5, 'low'),
    (uuid(), 'Closing Quality', 'Conversation ends with appropriate closing', 'empathy', '{"check": "has_closing", "required": true}', -5, 'low');
