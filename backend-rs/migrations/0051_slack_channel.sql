-- Phase 43: Slack Channel Integration
-- Adds Slack as a supported channel in the unified inbox, enabling
-- two-way messaging via Slack bot (Slack App) integration. Mirrors
-- Intercom's Slack channel support.

-- Slack bot commands (custom slash commands for the Slack app)
CREATE TABLE IF NOT EXISTS slack_commands (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_slack_commands_org ON slack_commands(organization_id);

-- Slack webhook events (track incoming webhook events)
CREATE TABLE IF NOT EXISTS slack_events (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    event_type TEXT NOT NULL, -- 'message', 'app_mention', 'reaction_added', etc.
    channel_id TEXT NOT NULL,
    user_id TEXT,
    username TEXT,
    team_id TEXT,
    team_name TEXT,
    message_content TEXT,
    raw_payload TEXT,
    processed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_slack_events_org ON slack_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_slack_events_type ON slack_events(event_type);
CREATE INDEX IF NOT EXISTS idx_slack_events_processed ON slack_events(processed);

-- Slack settings (per-organization configuration)
CREATE TABLE IF NOT EXISTS slack_settings (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL UNIQUE,
    auto_respond BOOLEAN NOT NULL DEFAULT true,
    greeting_message TEXT NOT NULL DEFAULT 'Hello! How can we help you today?',
    away_message TEXT NOT NULL DEFAULT 'We are currently away. We will respond when we return.',
    allowed_channels TEXT NOT NULL DEFAULT '[]', -- JSON array of Slack channel IDs
    blocked_words TEXT NOT NULL DEFAULT '[]', -- JSON array of blocked words
    log_events BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default settings
INSERT INTO slack_settings (id, organization_id)
VALUES ('slack-settings-default', 'org-default')
ON CONFLICT (id) DO NOTHING;

-- Default commands
INSERT INTO slack_commands (id, organization_id, name, description)
VALUES ('slack-cmd-help', 'org-default', '/help', 'Show available commands and get help')
ON CONFLICT (id) DO NOTHING;

INSERT INTO slack_commands (id, organization_id, name, description)
VALUES ('slack-cmd-agent', 'org-default', '/agent', 'Request to speak with a live agent')
ON CONFLICT (id) DO NOTHING;

INSERT INTO slack_commands (id, organization_id, name, description)
VALUES ('slack-cmd-status', 'org-default', '/status', 'Check your support ticket status')
ON CONFLICT (id) DO NOTHING;
