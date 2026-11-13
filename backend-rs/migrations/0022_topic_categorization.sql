-- Phase 11: Topic Categorization
-- Auto-categorizes every conversation into topics/subtopics for trend analysis
-- and content gap identification.

CREATE TABLE IF NOT EXISTS conversation_topics (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    
    -- Primary topic assigned to the conversation
    topic TEXT NOT NULL,
    
    -- Subtopic for more granular categorization
    subtopic TEXT,
    
    -- Confidence score from the AI classification (0.0 - 1.0)
    confidence REAL NOT NULL DEFAULT 0.8,
    
    -- Keywords extracted from the conversation that led to this classification
    keywords TEXT NOT NULL DEFAULT '[]',
    
    -- Whether the categorization has been manually overridden
    is_overridden BOOLEAN NOT NULL DEFAULT false,
    
    -- Who/what made the override (null if auto)
    overridden_by TEXT,
    
    -- When the categorization was last updated
    categorized_at TEXT NOT NULL DEFAULT (NOW()),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_conv_topics_conversation ON conversation_topics(conversation_id);
CREATE INDEX IF NOT EXISTS idx_conv_topics_topic ON conversation_topics(topic);
CREATE INDEX IF NOT EXISTS idx_conv_topics_subtopic ON conversation_topics(subtopic);
CREATE INDEX IF NOT EXISTS idx_conv_topics_categorized_at ON conversation_topics(categorized_at);

-- Topic trends table for aggregated analytics over time
CREATE TABLE IF NOT EXISTS topic_trends (
    id TEXT PRIMARY KEY,
    topic TEXT NOT NULL,
    subtopic TEXT,
    
    -- Time window for this snapshot
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    
    -- Count of conversations in this window
    conversation_count BOOLEAN NOT NULL DEFAULT false,
    
    -- Average confidence score for this topic in this window
    avg_confidence REAL NOT NULL DEFAULT 0.8,
    
    -- Average resolution rate for this topic (from feedback data)
    avg_resolution_rate REAL,
    
    -- Average response time in seconds for this topic
    avg_response_time_seconds REAL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_topic_trends_topic ON topic_trends(topic);
CREATE INDEX IF NOT EXISTS idx_topic_trends_window ON topic_trends(window_start, window_end);
