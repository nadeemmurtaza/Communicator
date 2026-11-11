-- Phase 46: Omni-Channel Broadcast System
--
-- Enables designing broadcast messages with rich media (images, videos, link previews)
-- and sending them to all selected channels in one click. The system automatically
-- formats content per channel and distributes to pre-selected contacts/customers.
--
-- Key features:
-- 1. Broadcast composer with rich media support (text, images, videos, links)
-- 2. Multi-channel distribution (WhatsApp, Email, SMS, Instagram, Facebook, Telegram, Discord, Slack)
-- 3. Per-channel automatic formatting based on channel capabilities
-- 4. Contact/segment targeting for precise audience selection
-- 5. One-click send with progress tracking and analytics
-- 6. Draft/scheduled/paused/completed lifecycle management

-- ---------------------------------------------------------------------------
-- 1. Broadcasts: main broadcast definitions
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS broadcasts (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    organization_id TEXT NOT NULL,
    brand_id TEXT,
    title TEXT NOT NULL,                          -- internal title for the broadcast
    status TEXT NOT NULL DEFAULT 'draft',          -- 'draft', 'scheduled', 'sending', 'sent', 'paused', 'failed', 'cancelled'
    created_by TEXT NOT NULL,                     -- user ID who created
    scheduled_at TIMESTAMPTZ,                     -- null = send immediately
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,

    -- Content
    body_text TEXT NOT NULL DEFAULT '',            -- plain text body
    body_html TEXT DEFAULT '',                    -- HTML version (for email)
    subject_line TEXT DEFAULT '',                 -- email subject
    preview_text TEXT DEFAULT '',                 -- short preview for link previews

    -- Media
    media_type TEXT DEFAULT 'text',               -- 'text', 'image', 'video', 'link', 'mixed'
    primary_media_url TEXT DEFAULT '',            -- main image/video/link URL
    media_caption TEXT DEFAULT '',                -- caption for media

    -- Link preview
    link_url TEXT DEFAULT '',                     -- URL for link preview
    link_title TEXT DEFAULT '',
    link_description TEXT DEFAULT '',
    link_image_url TEXT DEFAULT '',

    -- Targeting
    target_type TEXT NOT NULL DEFAULT 'contacts', -- 'contacts', 'segments', 'all_contacts'
    target_ids JSONB DEFAULT '[]',               -- array of contact IDs or segment IDs

    -- Channel selection
    channels JSONB DEFAULT '[]',                  -- array of channel codes: ['whatsapp', 'email', 'sms', 'instagram', 'facebook', 'telegram', 'discord', 'slack']

    -- Per-channel formatting overrides
    channel_overrides JSONB DEFAULT '{}',         -- per-channel text/media overrides

    -- Analytics
    total_recipients INTEGER NOT NULL DEFAULT 0,
    sent_count INTEGER NOT NULL DEFAULT 0,
    delivered_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    read_count INTEGER NOT NULL DEFAULT 0,
    clicked_count INTEGER NOT NULL DEFAULT 0,
    replied_count INTEGER NOT NULL DEFAULT 0,

    -- Settings
    send_delay_seconds INTEGER DEFAULT 0,         -- delay between sends (rate limiting)
    retry_failed BOOLEAN NOT NULL DEFAULT true,
    max_retries INTEGER DEFAULT 3,
    track_opens BOOLEAN NOT NULL DEFAULT true,
    track_clicks BOOLEAN NOT NULL DEFAULT true,

    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcasts_org ON broadcasts(organization_id);
CREATE INDEX IF NOT EXISTS idx_broadcasts_brand ON broadcasts(brand_id);
CREATE INDEX IF NOT EXISTS idx_broadcasts_status ON broadcasts(status);
CREATE INDEX IF NOT EXISTS idx_broadcasts_scheduled ON broadcasts(scheduled_at) WHERE scheduled_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_broadcasts_created ON broadcasts(created_at);

COMMENT ON TABLE broadcasts IS 'Omni-channel broadcast definitions with rich media support and multi-channel distribution.';

-- ---------------------------------------------------------------------------
-- 2. Broadcast media: uploaded media files for broadcasts
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS broadcast_media (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    broadcast_id TEXT NOT NULL REFERENCES broadcasts(id) ON DELETE CASCADE,
    media_type TEXT NOT NULL,                     -- 'image', 'video', 'document', 'audio'
    file_name TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL,
    file_url TEXT NOT NULL,                      -- CDN or storage URL
    thumbnail_url TEXT DEFAULT '',                -- thumbnail for images/videos
    duration_seconds INTEGER DEFAULT 0,           -- for video/audio
    width INTEGER DEFAULT 0,                     -- for images/videos
    height INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'uploading',     -- 'uploading', 'processing', 'ready', 'failed'
    error_message TEXT DEFAULT '',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcast_media_broadcast ON broadcast_media(broadcast_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_media_type ON broadcast_media(media_type);

COMMENT ON TABLE broadcast_media IS 'Media files (images, videos, documents) attached to broadcasts.';

-- ---------------------------------------------------------------------------
-- 3. Broadcast channels: per-channel formatting and status
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS broadcast_channels (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    broadcast_id TEXT NOT NULL REFERENCES broadcasts(id) ON DELETE CASCADE,
    channel_code TEXT NOT NULL,                  -- 'whatsapp', 'email', 'sms', 'instagram', 'facebook', 'telegram', 'discord', 'slack'
    channel_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',      -- 'pending', 'sending', 'sent', 'failed', 'skipped'
    formatted_body TEXT DEFAULT '',              -- channel-specific formatted message
    formatted_media_url TEXT DEFAULT '',         -- channel-specific media URL
    formatted_subject TEXT DEFAULT '',           -- email subject
    recipient_count INTEGER NOT NULL DEFAULT 0,
    sent_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT DEFAULT '',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcast_channels_broadcast ON broadcast_channels(broadcast_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_channels_channel ON broadcast_channels(channel_code);

COMMENT ON TABLE broadcast_channels IS 'Per-channel formatting and delivery status for each broadcast.';

-- ---------------------------------------------------------------------------
-- 4. Broadcast recipients: individual recipient tracking
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS broadcast_recipients (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    broadcast_id TEXT NOT NULL REFERENCES broadcasts(id) ON DELETE CASCADE,
    contact_id TEXT NOT NULL,                    -- reference to contacts table
    channel_code TEXT NOT NULL,                  -- channel used for this recipient
    phone_number TEXT DEFAULT '',
    email_address TEXT DEFAULT '',
    external_id TEXT DEFAULT '',                 -- WhatsApp JID, Discord user ID, etc.
    status TEXT NOT NULL DEFAULT 'pending',      -- 'pending', 'queued', 'sending', 'sent', 'delivered', 'read', 'failed', 'skipped'
    message_id TEXT DEFAULT '',                  -- channel-specific message ID
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ,
    replied_at TIMESTAMPTZ,
    error_message TEXT DEFAULT '',
    retry_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcast_recipients_broadcast ON broadcast_recipients(broadcast_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_recipients_contact ON broadcast_recipients(contact_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_recipients_status ON broadcast_recipients(status);
CREATE INDEX IF NOT EXISTS idx_broadcast_recipients_channel ON broadcast_recipients(channel_code);

COMMENT ON TABLE broadcast_recipients IS 'Individual recipient tracking for each broadcast.';

-- ---------------------------------------------------------------------------
-- 5. Broadcast logs: audit trail for broadcast operations
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS broadcast_logs (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    broadcast_id TEXT NOT NULL REFERENCES broadcasts(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,                    -- 'created', 'scheduled', 'started', 'sent', 'delivered', 'failed', 'paused', 'cancelled', 'completed'
    channel_code TEXT DEFAULT '',
    recipient_id TEXT DEFAULT '',
    details JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcast_logs_broadcast ON broadcast_logs(broadcast_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_logs_event ON broadcast_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_broadcast_logs_created ON broadcast_logs(created_at);

COMMENT ON TABLE broadcast_logs IS 'Audit trail for all broadcast operations.';

-- ---------------------------------------------------------------------------
-- 6. Broadcast settings: per-org configuration
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS broadcast_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_recipients_per_broadcast INTEGER NOT NULL DEFAULT 10000,
    default_send_delay_seconds INTEGER NOT NULL DEFAULT 2,
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    allow_scheduling BOOLEAN NOT NULL DEFAULT true,
    require_approval BOOLEAN NOT NULL DEFAULT false,
    approved_by TEXT DEFAULT '',
    auto_format_channels BOOLEAN NOT NULL DEFAULT true,
    track_opens BOOLEAN NOT NULL DEFAULT true,
    track_clicks BOOLEAN NOT NULL DEFAULT true,
    default_channels TEXT[] NOT NULL DEFAULT ARRAY['whatsapp', 'email'],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcast_settings_org ON broadcast_settings(organization_id);

COMMENT ON TABLE broadcast_settings IS 'Per-organization broadcast configuration.';

-- Seed default settings
INSERT INTO broadcast_settings (id, organization_id, enabled)
VALUES ('default-broadcast-settings', 'default', true)
ON CONFLICT (id) DO NOTHING;
