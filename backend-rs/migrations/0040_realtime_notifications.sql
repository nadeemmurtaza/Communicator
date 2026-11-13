-- Migration 0040: Real-time Notifications & WebSockets (PostgreSQL)
-- Push notifications, WebSocket broadcast, and notification preferences.
-- Enables live updates for new messages, draft approvals, alerts, and system events.

-- ============================================================================
-- 1. NOTIFICATION PREFERENCES
-- ============================================================================
-- Per-user notification settings for different event types.
CREATE TABLE IF NOT EXISTS notification_preferences (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    new_message_push BOOLEAN NOT NULL DEFAULT true,
    new_message_email BOOLEAN NOT NULL DEFAULT false,
    draft_approved_push BOOLEAN NOT NULL DEFAULT true,
    draft_rejected_push BOOLEAN NOT NULL DEFAULT true,
    alert_triggered_push BOOLEAN NOT NULL DEFAULT true,
    alert_triggered_email BOOLEAN NOT NULL DEFAULT true,
    conversation_assigned_push BOOLEAN NOT NULL DEFAULT true,
    conversation_escalated_push BOOLEAN NOT NULL DEFAULT true,
    workflow_completed_push BOOLEAN NOT NULL DEFAULT false,
    csat_response_push BOOLEAN NOT NULL DEFAULT false,
    suggestion_ready_push BOOLEAN NOT NULL DEFAULT true,
    operator_proposal_push BOOLEAN NOT NULL DEFAULT true,
    sound_enabled BOOLEAN NOT NULL DEFAULT true,
    desktop_notifications BOOLEAN NOT NULL DEFAULT true,
    quiet_hours_start TEXT,
    quiet_hours_end TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_notif_pref_org ON notification_preferences(organization_id);
CREATE INDEX IF NOT EXISTS idx_notif_pref_user ON notification_preferences(user_id);

-- ============================================================================
-- 2. NOTIFICATION QUEUE
-- ============================================================================
-- All notifications, whether sent via push, email, or WebSocket.
CREATE TABLE IF NOT EXISTS notification_queue (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    user_id TEXT,
    notification_type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    data TEXT NOT NULL DEFAULT '{}',
    channel TEXT NOT NULL DEFAULT 'push',
    status TEXT NOT NULL DEFAULT 'pending',
    sent_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    dismissed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_notif_queue_org ON notification_queue(organization_id);
CREATE INDEX IF NOT EXISTS idx_notif_queue_user ON notification_queue(user_id);
CREATE INDEX IF NOT EXISTS idx_notif_queue_status ON notification_queue(status);
CREATE INDEX IF NOT EXISTS idx_notif_queue_type ON notification_queue(notification_type);
CREATE INDEX IF NOT EXISTS idx_notif_queue_created ON notification_queue(created_at);

-- ============================================================================
-- 3. WEBSOCKET CONNECTIONS
-- ============================================================================
-- Track active WebSocket connections for broadcast targeting.
CREATE TABLE IF NOT EXISTS websocket_connections (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    path TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    ip_address TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_ws_conn_org ON websocket_connections(organization_id);
CREATE INDEX IF NOT EXISTS idx_ws_conn_user ON websocket_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_ws_conn_session ON websocket_connections(session_id);

-- ============================================================================
-- 4. NOTIFICATION BROADCAST LOG
-- ============================================================================
-- Audit log of WebSocket broadcasts.
CREATE TABLE IF NOT EXISTS notification_broadcast_log (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    notification_type TEXT NOT NULL,
    title TEXT NOT NULL,
    recipient_count INTEGER NOT NULL DEFAULT 0,
    delivered_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    broadcast_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_broadcast_log_org ON notification_broadcast_log(organization_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_log_type ON notification_broadcast_log(notification_type);
CREATE INDEX IF NOT EXISTS idx_broadcast_log_at ON notification_broadcast_log(broadcast_at);

-- ============================================================================
-- 5. NOTIFICATION SETTINGS
-- ============================================================================
-- Per-organization global notification configuration.
CREATE TABLE IF NOT EXISTS notification_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    websocket_enabled BOOLEAN NOT NULL DEFAULT true,
    email_notifications_enabled BOOLEAN NOT NULL DEFAULT false,
    email_from_address TEXT NOT NULL DEFAULT 'notifications@communicator.local',
    max_notifications_per_hour INTEGER NOT NULL DEFAULT 100,
    digest_mode BOOLEAN NOT NULL DEFAULT false,
    digest_interval_minutes INTEGER NOT NULL DEFAULT 60,
    retention_days INTEGER NOT NULL DEFAULT 90,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_notif_settings_org ON notification_settings(organization_id);

-- Seed default settings
INSERT INTO notification_settings (id, organization_id, enabled, websocket_enabled)
VALUES ('notif-settings-default', 'org-default', true, true)
ON CONFLICT (id) DO NOTHING;

-- Seed default user preferences
INSERT INTO notification_preferences (id, organization_id, user_id, enabled)
VALUES ('notif-pref-default', 'org-default', 'user-default', true)
ON CONFLICT (id) DO NOTHING;
