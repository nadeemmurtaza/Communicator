-- Phase 35: Self-Service Embedding Widget
-- Public-facing embeddable help center and live chat widget for external websites

CREATE TABLE IF NOT EXISTS embed_widget_configs (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    widget_type TEXT NOT NULL DEFAULT 'help_center', -- help_center, live_chat, search, combined
    status TEXT NOT NULL DEFAULT 'draft', -- draft, active, paused
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    styles JSONB NOT NULL DEFAULT '{}'::jsonb,
    trigger_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    domain_whitelist TEXT[],
    install_count INTEGER NOT NULL DEFAULT 0,
    last_installed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_embed_widget_configs_org ON embed_widget_configs(org_id);
CREATE INDEX IF NOT EXISTS idx_embed_widget_configs_status ON embed_widget_configs(status);

CREATE TABLE IF NOT EXISTS embed_widget_instances (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    config_id TEXT NOT NULL REFERENCES embed_widget_configs(id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    page_path TEXT,
    visitor_id TEXT,
    session_id TEXT,
    status TEXT NOT NULL DEFAULT 'active', -- active, closed, minimized
    messages_sent INTEGER NOT NULL DEFAULT 0,
    last_interaction_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_embed_widget_instances_config ON embed_widget_instances(config_id);
CREATE INDEX IF NOT EXISTS idx_embed_widget_instances_domain ON embed_widget_instances(domain);
CREATE INDEX IF NOT EXISTS idx_embed_widget_instances_visitor ON embed_widget_instances(visitor_id);

CREATE TABLE IF NOT EXISTS embed_widget_events (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    instance_id TEXT REFERENCES embed_widget_instances(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL, -- opened, closed, minimized, message_sent, message_received, search_performed, article_viewed
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_embed_widget_events_instance ON embed_widget_events(instance_id);
CREATE INDEX IF NOT EXISTS idx_embed_widget_events_type ON embed_widget_events(event_type);
CREATE INDEX IF NOT EXISTS idx_embed_widget_events_created ON embed_widget_events(created_at);

CREATE TABLE IF NOT EXISTS embed_widget_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    allow_custom_domains BOOLEAN NOT NULL DEFAULT true,
    max_instances_per_domain INTEGER NOT NULL DEFAULT 100,
    analytics_enabled BOOLEAN NOT NULL DEFAULT true,
    default_language TEXT NOT NULL DEFAULT 'en',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default settings
INSERT INTO embed_widget_settings (id, org_id, enabled, analytics_enabled)
VALUES ('default-embed-settings', 'default', true, true)
ON CONFLICT (id) DO NOTHING;

-- Seed sample widget config
INSERT INTO embed_widget_configs (id, org_id, name, widget_type, status, settings, styles, trigger_config)
VALUES (
    'default-help-center-widget',
    'default',
    'Help Center Widget',
    'help_center',
    'active',
    '{"show_search": true, "show_categories": true, "max_articles": 10}'::jsonb,
    '{"primary_color": "#4F46E5", "font_family": "system-ui", "position": "bottom-right", "width": 400, "height": 600}'::jsonb,
    '{"type": "button", "button_text": "Help", "button_color": "#4F46E5"}'::jsonb
)
ON CONFLICT (id) DO NOTHING;
