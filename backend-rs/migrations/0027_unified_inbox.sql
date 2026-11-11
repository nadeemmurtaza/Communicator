-- Phase 19: Unified Inbox â€” Cross-channel conversation merging
-- Enables viewing all conversations for the same customer across channels
-- as a single unified timeline, mirroring Intercom's core differentiator.

-- Unified customer-conversation mapping
-- Links conversations to their resolved customer_id for cross-channel grouping
CREATE TABLE IF NOT EXISTS unified_customer_mapping (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL, -- Resolved customer ID from client profiles
    contact_id TEXT NOT NULL, -- Original contact ID
    channel TEXT NOT NULL, -- Channel type (whatsapp, email, sms, etc.)
    channel_identifier TEXT NOT NULL, -- Phone, email, username, etc.
    confidence REAL NOT NULL DEFAULT 1.0, -- How confident we are this mapping is correct
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_by TEXT NOT NULL DEFAULT 'auto', -- 'auto' | 'manual' | 'ai'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_unified_customer_customer ON unified_customer_mapping(customer_id);
CREATE INDEX IF NOT EXISTS idx_unified_customer_contact ON unified_customer_mapping(contact_id);
CREATE INDEX IF NOT EXISTS idx_unified_customer_channel ON unified_customer_mapping(channel);
CREATE UNIQUE INDEX IF NOT EXISTS idx_unified_customer_unique ON unified_customer_mapping(contact_id, channel);

-- Unified inbox preferences per organization
CREATE TABLE IF NOT EXISTS unified_inbox_settings (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    group_by TEXT NOT NULL DEFAULT 'customer', -- 'customer' | 'contact' | 'channel'
    merge_timeline BOOLEAN NOT NULL DEFAULT true, -- Merge messages from all channels into single timeline
    show_channel_badges BOOLEAN NOT NULL DEFAULT true, -- Show channel indicator on each message
    auto_resolve BOOLEAN NOT NULL DEFAULT true, -- Auto-resolve customer identity from channel IDs
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_unified_settings_org ON unified_inbox_settings(organization_id);

-- Default settings for organizations
INSERT INTO unified_inbox_settings (id, organization_id, enabled, group_by, merge_timeline, show_channel_badges, auto_resolve)
SELECT 'unified-settings-' || substr(id, 1, 24), id, 1, 'customer', 1, 1, 1
FROM organizations
WHERE id NOT IN (SELECT organization_id FROM unified_inbox_settings);

-- Trigger to auto-populate unified_customer_mapping when a conversation is created
-- and the contact has a known customer_id via customer_channel_ids
CREATE TRIGGER IF NOT EXISTS trg_unified_mapping_after_insert
AFTER INSERT ON conversations
BEGIN
    -- If the contact has a known channel mapping, create unified customer mapping
    INSERT INTO unified_customer_mapping (id, customer_id, contact_id, channel, channel_identifier, confidence, resolved_by)
    SELECT 
        'um-' || substr(changes.id, 1, 24) || '-' || substr(changes.channel, 1, 8),
        ccid.customer_id,
        changes.contact_id,
        changes.channel,
        COALESCE((SELECT value FROM contacts WHERE id = changes.contact_id), ''),
        0.9,
        'auto'
    FROM (VALUES (NEW.id, NEW.contact_id, NEW.channel)) AS changes(id, contact_id, channel)
    LEFT JOIN contacts c ON c.id = changes.contact_id
    LEFT JOIN customer_channel_ids ccid ON ccid.channel_type = changes.channel AND ccid.channel_identifier = c.value
    WHERE ccid.customer_id IS NOT NULL
    AND changes.contact_id IS NOT NULL;
END;
