-- Phase 15: CSAT Surveys & Customer Satisfaction Tracking

-- Survey templates define the structure of CSAT surveys
CREATE TABLE IF NOT EXISTS csat_survey_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    question TEXT NOT NULL DEFAULT 'How would you rate this conversation?',
    scale_type TEXT NOT NULL DEFAULT '1-5' CHECK (scale_type IN ('1-5', '1-10', 'emoji', 'binary')),
    min_score FLOAT NOT NULL DEFAULT 1,
    max_score INTEGER NOT NULL DEFAULT 5,
    include_comment BOOLEAN NOT NULL DEFAULT true,
    comment_prompt TEXT DEFAULT 'Please tell us how we can improve:',
    trigger_type TEXT NOT NULL DEFAULT 'manual' CHECK (trigger_type IN ('manual', 'auto_on_close', 'auto_on_send', 'scheduled')),
    trigger_delay_minutes BOOLEAN DEFAULT false,
    channel_filter TEXT, -- JSON array of channels, null means all
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Individual survey instances sent to customers
CREATE TABLE IF NOT EXISTS csat_surveys (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES csat_survey_templates(id),
    conversation_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    channel TEXT NOT NULL,
    contact_identifier TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'responded', 'expired', 'cancelled')),
    score INTEGER,
    comment TEXT,
    sent_at TEXT,
    responded_at TEXT,
    expires_at TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- CSAT score snapshots aggregated over time windows
CREATE TABLE IF NOT EXISTS csat_snapshots (
    id TEXT PRIMARY KEY,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    total_surveys_sent BOOLEAN NOT NULL DEFAULT false,
    total_responses BOOLEAN NOT NULL DEFAULT false,
    response_rate REAL NOT NULL DEFAULT 0,
    average_score REAL,
    csat_percentage REAL, -- % of responses that are 4-5 (for 1-5 scale) or 9-10 (for 1-10 scale)
    score_distribution TEXT, -- JSON object: {"1": 5, "2": 3, "3": 10, "4": 20, "5": 40}
    by_channel TEXT, -- JSON object: {"email": {"avg": 4.2, "count": 30}, "sms": {"avg": 3.8, "count": 15}}
    by_template TEXT, -- JSON object: {"template_id": {"avg": 4.2, "count": 30}}
    top_comments TEXT, -- JSON array of top positive/negative comments
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default survey template
INSERT INTO csat_survey_templates (id, name, description, question, scale_type, min_score, max_score, include_comment, comment_prompt, trigger_type)
VALUES ('default', 'Default CSAT Survey', 'Standard 1-5 star satisfaction survey', 'How would you rate this conversation?', '1-5', 1, 5, 1, 'Please tell us how we can improve:', 'manual');

-- Indexes
CREATE INDEX IF NOT EXISTS idx_csat_surveys_conversation ON csat_surveys(conversation_id);
CREATE INDEX IF NOT EXISTS idx_csat_surveys_contact ON csat_surveys(contact_id);
CREATE INDEX IF NOT EXISTS idx_csat_surveys_status ON csat_surveys(status);
CREATE INDEX IF NOT EXISTS idx_csat_surveys_template ON csat_surveys(template_id);
CREATE INDEX IF NOT EXISTS idx_csat_surveys_created ON csat_surveys(created_at);
CREATE INDEX IF NOT EXISTS idx_csat_snapshots_window ON csat_snapshots(window_start, window_end);
