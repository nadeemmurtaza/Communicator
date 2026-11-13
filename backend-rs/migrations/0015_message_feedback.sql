-- Migration 0015: Message Feedback Collection Loop
--
-- Allows customers and agents to rate AI-generated outbound messages with
-- thumbs up / thumbs down, optionally adding a short comment.  The data
-- feeds the self-improvement flywheel: analytics, content-gap suggestions,
-- and model tuning.

CREATE TABLE IF NOT EXISTS message_feedback (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    rating INTEGER NOT NULL,              -- 1 = thumbs up, -1 = thumbs down
    comment TEXT,                         -- optional free-text reason
    author_type TEXT NOT NULL DEFAULT 'customer',  -- 'customer' | 'agent'
    author_id TEXT,                       -- optional: agent id or contact id
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_message_feedback_message_id ON message_feedback(message_id);
CREATE INDEX IF NOT EXISTS idx_message_feedback_conversation_id ON message_feedback(conversation_id);
CREATE INDEX IF NOT EXISTS idx_message_feedback_rating ON message_feedback(rating);
