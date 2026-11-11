-- Migration 0041: In-App Messaging & Proactive Triggers (PostgreSQL)
-- Behavioral event-driven messaging system for triggered messages based on user actions,
-- page views, time on site, and custom events. Enables proactive outreach like
-- "Are you stuck?" popups, onboarding sequences, and re-engagement messages.

-- ============================================================================
-- 1. IN-APP MESSAGES
-- ============================================================================
-- Message definitions that can be triggered by events, time, or behavior.
CREATE TABLE IF NOT EXISTS inapp_messages (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    message_type TEXT NOT NULL DEFAULT 'popup',
    style TEXT NOT NULL DEFAULT 'default',
    target_url TEXT,
    button_text TEXT,
    button_url TEXT,
    dismissible BOOLEAN NOT NULL DEFAULT true,
    auto_close_seconds INTEGER,
    trigger_type TEXT NOT NULL DEFAULT 'event',
    trigger_event TEXT,
    trigger_page TEXT,
    trigger_delay_seconds INTEGER,
    trigger_segment_id TEXT,
    audience_segment_id TEXT,
    frequency TEXT NOT NULL DEFAULT 'once_per_session',
    status TEXT NOT NULL DEFAULT 'draft',
    start_at TIMESTAMPTZ,
    end_at TIMESTAMPTZ,
    views INTEGER NOT NULL DEFAULT 0,
    clicks INTEGER NOT NULL DEFAULT 0,
    dismisses INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_inapp_msg_org ON inapp_messages(organization_id);
CREATE INDEX IF NOT EXISTS idx_inapp_msg_status ON inapp_messages(status);
CREATE INDEX IF NOT EXISTS idx_inapp_msg_trigger ON inapp_messages(trigger_type);

-- ============================================================================
-- 2. IN-APP MESSAGE EVENTS
-- ============================================================================
-- Tracks when messages are shown, clicked, or dismissed.
CREATE TABLE IF NOT EXISTS inapp_message_events (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    contact_id TEXT,
    session_id TEXT,
    event_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (message_id) REFERENCES inapp_messages(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_inapp_evt_msg ON inapp_message_events(message_id);
CREATE INDEX IF NOT EXISTS idx_inapp_evt_contact ON inapp_message_events(contact_id);
CREATE INDEX IF NOT EXISTS idx_inapp_evt_type ON inapp_message_events(event_type);
CREATE INDEX IF NOT EXISTS idx_inapp_evt_created ON inapp_message_events(created_at);

-- ============================================================================
-- 3. PROACTIVE TRIGGERS
-- ============================================================================
-- Configurable trigger rules for proactive messaging.
CREATE TABLE IF NOT EXISTS proactive_triggers (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    condition TEXT NOT NULL DEFAULT '{}',
    action TEXT NOT NULL DEFAULT 'show_message',
    message_id TEXT,
    delay_seconds INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_triggers_per_day INTEGER NOT NULL DEFAULT 100,
    triggers_today INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (message_id) REFERENCES inapp_messages(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_proactive_trig_org ON proactive_triggers(organization_id);
CREATE INDEX IF NOT EXISTS idx_proactive_trig_enabled ON proactive_triggers(enabled);

-- ============================================================================
-- 4. BEHAVIORAL EVENTS
-- ============================================================================
-- Tracks user behavioral events for trigger evaluation.
CREATE TABLE IF NOT EXISTS behavioral_events (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    contact_id TEXT,
    session_id TEXT,
    event_name TEXT NOT NULL,
    event_data TEXT NOT NULL DEFAULT '{}',
    page_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_behavioral_evt_org ON behavioral_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_behavioral_evt_contact ON behavioral_events(contact_id);
CREATE INDEX IF NOT EXISTS idx_behavioral_evt_name ON behavioral_events(event_name);
CREATE INDEX IF NOT EXISTS idx_behavioral_evt_created ON behavioral_events(created_at);

-- ============================================================================
-- 5. IN-APP SETTINGS
-- ============================================================================
-- Per-organization configuration for in-app messaging.
CREATE TABLE IF NOT EXISTS inapp_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_concurrent_messages INTEGER NOT NULL DEFAULT 1,
    default_style TEXT NOT NULL DEFAULT 'default',
    allow_custom_css BOOLEAN NOT NULL DEFAULT true,
    track_behavioral_events BOOLEAN NOT NULL DEFAULT true,
    event_retention_days INTEGER NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_inapp_settings_org ON inapp_settings(organization_id);

-- Seed default settings
INSERT INTO inapp_settings (id, organization_id, enabled)
VALUES ('inapp-settings-default', 'org-default', true)
ON CONFLICT (id) DO NOTHING;

-- Seed sample proactive triggers
INSERT INTO proactive_triggers (id, organization_id, name, trigger_type, condition, action, enabled)
VALUES (
    'proactive-trigger-welcome',
    'org-default',
    'Welcome New Visitors',
    'page_view',
    '{"pages": ["/"], "delay_seconds": 5}',
    'show_message',
    true
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO proactive_triggers (id, organization_id, name, trigger_type, condition, action, enabled)
VALUES (
    'proactive-trigger-exit',
    'org-default',
    'Exit Intent',
    'exit_intent',
    '{"threshold": 0.8}',
    'show_message',
    true
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO proactive_triggers (id, organization_id, name, trigger_type, condition, action, enabled)
VALUES (
    'proactive-trigger-inactive',
    'org-default',
    'Re-engage Inactive Users',
    'inactivity',
    '{"idle_seconds": 30}',
    'show_message',
    true
)
ON CONFLICT (id) DO NOTHING;
