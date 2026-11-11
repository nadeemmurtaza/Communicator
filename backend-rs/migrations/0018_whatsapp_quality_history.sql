-- WhatsApp Quality Dashboard: Track quality rating history over 30 days

-- Quality history snapshots â€” one row per health check
CREATE TABLE IF NOT EXISTS whatsapp_quality_history (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    quality_rating TEXT,              -- "HIGH", "MEDIUM", "LOW"
    quality_score INTEGER,            -- 0-100 numeric score
    messaging_limit_tier TEXT,        -- "100", "1000", "10000", "100000"
    messaging_limit_value INTEGER,    -- numeric tier value
    block_reasons TEXT,               -- JSON array of block reason strings
    spam_flags BOOLEAN DEFAULT false,     -- number of spam flags in last 14 days
    matched_conversations BOOLEAN DEFAULT false,  -- conversations in last 14 days
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_quality_history_brand ON whatsapp_quality_history(brand_id, created_at DESC);

-- Quality improvement recommendations cache
CREATE TABLE IF NOT EXISTS whatsapp_quality_recommendations (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    recommendation_type TEXT NOT NULL,  -- "quality", "messaging_limit", "spam", "response_time"
    severity TEXT NOT NULL,             -- "critical", "warning", "info"
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    is_resolved BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_recommendations_brand ON whatsapp_quality_recommendations(brand_id, is_resolved);
