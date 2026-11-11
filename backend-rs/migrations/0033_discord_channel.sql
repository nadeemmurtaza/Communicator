-- Phase 24: Discord Channel Integration
-- Adds Discord as a supported channel in the unified inbox, enabling
-- two-way messaging via Discord bot integration. Mirrors Intercom's
-- Discord channel support.

-- Discord bot commands (custom slash commands for the bot)
CREATE TABLE IF NOT EXISTS discord_commands (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_discord_commands_org ON discord_commands(organization_id);

-- Discord webhook events (track incoming webhook events)
CREATE TABLE IF NOT EXISTS discord_events (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    event_type TEXT NOT NULL, -- 'MESSAGE_CREATE', 'INTERACTION', 'GUILD_JOIN', etc.
    channel_id TEXT NOT NULL,
    user_id TEXT,
    username TEXT,
    guild_id TEXT,
    guild_name TEXT,
    message_content TEXT,
    raw_payload TEXT,
    processed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_discord_events_org ON discord_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_discord_events_type ON discord_events(event_type);
CREATE INDEX IF NOT EXISTS idx_discord_events_processed ON discord_events(processed);

-- Discord settings (per-organization configuration)
CREATE TABLE IF NOT EXISTS discord_settings (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL UNIQUE,
    auto_respond BOOLEAN NOT NULL DEFAULT true,
    greeting_message TEXT NOT NULL DEFAULT 'Hello! How can we help you today?',
    away_message TEXT NOT NULL DEFAULT 'We are currently away. We will respond when we return.',
    allowed_channels TEXT NOT NULL DEFAULT '[]', -- JSON array of Discord channel IDs
    blocked_words TEXT NOT NULL DEFAULT '[]', -- JSON array of blocked words
    log_events BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default settings
INSERT INTO discord_settings (id, organization_id)
VALUES ('discord-settings-default', 'org-default');

-- Default commands
INSERT INTO discord_commands (id, organization_id, name, description)
VALUES ('discord-cmd-start', 'org-default', 'help', 'Show available commands and get help');
INSERT INTO discord_commands (id, organization_id, name, description)
VALUES ('discord-cmd-agent', 'org-default', 'agent', 'Request to speak with a live agent');
INSERT INTO discord_commands (id, organization_id, name, description)
VALUES ('discord-cmd-status', 'org-default', 'status', 'Check your support ticket status');
