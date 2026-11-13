-- Phase 36: Unified Analytics Dashboard
-- Aggregates metrics from all phases into comprehensive reports with export and scheduling

CREATE TABLE IF NOT EXISTS saved_reports (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    report_type TEXT NOT NULL, -- conversation, agent, csat, cx_qa, feedback, topic, validation, autonomous, procedure, workflow, unified
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    time_range TEXT NOT NULL DEFAULT '7d', -- 7d, 30d, 90d, custom
    start_date DATE,
    end_date DATE,
    filters JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_run_at TIMESTAMPTZ,
    last_result JSONB,
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_saved_reports_org ON saved_reports(org_id);
CREATE INDEX IF NOT EXISTS idx_saved_reports_type ON saved_reports(report_type);

CREATE TABLE IF NOT EXISTS scheduled_reports (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    report_id TEXT NOT NULL REFERENCES saved_reports(id) ON DELETE CASCADE,
    schedule TEXT NOT NULL, -- daily, weekly, monthly, custom_cron
    cron_expression TEXT,
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    recipients TEXT[] NOT NULL DEFAULT ARRAY[]::text[],
    format TEXT NOT NULL DEFAULT 'csv', -- csv, json, pdf
    status TEXT NOT NULL DEFAULT 'active', -- active, paused, completed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduled_reports_report ON scheduled_reports(report_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_status ON scheduled_reports(status);

CREATE TABLE IF NOT EXISTS report_templates (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    report_type TEXT NOT NULL,
    description TEXT,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_default BOOLEAN NOT NULL DEFAULT false,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_report_templates_org ON report_templates(org_id);
CREATE INDEX IF NOT EXISTS idx_report_templates_type ON report_templates(report_type);

CREATE TABLE IF NOT EXISTS dashboard_widgets (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    dashboard_id TEXT NOT NULL,
    widget_type TEXT NOT NULL, -- metric_card, line_chart, bar_chart, pie_chart, table, trend
    title TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    position_x INTEGER NOT NULL DEFAULT 0,
    position_y INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 4,
    height INTEGER NOT NULL DEFAULT 3,
    is_visible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboard_widgets_org ON dashboard_widgets(org_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_widgets_dashboard ON dashboard_widgets(dashboard_id);

CREATE TABLE IF NOT EXISTS analytics_snapshots (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    snapshot_type TEXT NOT NULL, -- daily, weekly, monthly
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analytics_snapshots_org ON analytics_snapshots(org_id);
CREATE INDEX IF NOT EXISTS idx_analytics_snapshots_type ON analytics_snapshots(snapshot_type);
CREATE INDEX IF NOT EXISTS idx_analytics_snapshots_period ON analytics_snapshots(period_start, period_end);

CREATE TABLE IF NOT EXISTS unified_analytics_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    auto_snapshot BOOLEAN NOT NULL DEFAULT true,
    snapshot_frequency TEXT NOT NULL DEFAULT 'daily', -- daily, weekly, monthly
    retention_days INTEGER NOT NULL DEFAULT 365,
    default_dashboard TEXT NOT NULL DEFAULT 'overview',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default settings
INSERT INTO unified_analytics_settings (id, org_id, enabled, auto_snapshot)
VALUES ('default-analytics-settings', 'default', true, true)
ON CONFLICT (id) DO NOTHING;

-- Seed default report templates
INSERT INTO report_templates (id, org_id, name, report_type, description, config, is_default)
VALUES
    ('template-conversation-overview', 'default', 'Conversation Overview', 'conversation', 'Overall conversation metrics including volume, resolution rate, and response times', '{"metrics": ["total_conversations", "resolved_conversations", "avg_response_time", "resolution_rate"]}'::jsonb, true),
    ('template-agent-performance', 'default', 'Agent Performance', 'agent', 'Per-agent metrics including conversations handled, resolution rate, and CSAT scores', '{"metrics": ["conversations_handled", "resolution_rate", "avg_response_time", "csat_score"]}'::jsonb, true),
    ('template-cx-summary', 'default', 'CX Score Summary', 'cx_qa', 'Customer experience scores across all dimensions with trends', '{"metrics": ["overall_cx", "accuracy", "empathy", "resolution", "compliance", "timeliness"]}'::jsonb, true),
    ('template-unified-dashboard', 'default', 'Unified Dashboard', 'unified', 'Comprehensive view combining all metrics from all phases', '{"sections": ["conversations", "feedback", "validation", "cx_qa", "csat", "topics", "agents"]}'::jsonb, true)
ON CONFLICT (id) DO NOTHING;

-- Seed default dashboard widgets
INSERT INTO dashboard_widgets (id, org_id, dashboard_id, widget_type, title, config, position_x, position_y, width, height)
VALUES
    ('widget-total-conversations', 'default', 'overview', 'metric_card', 'Total Conversations', '{"metric": "total_conversations", "trend": "7d"}'::jsonb, 0, 0, 4, 3),
    ('widget-resolution-rate', 'default', 'overview', 'metric_card', 'Resolution Rate', '{"metric": "resolution_rate", "trend": "7d"}'::jsonb, 4, 0, 4, 3),
    ('widget-cx-score', 'default', 'overview', 'metric_card', 'CX Score', '{"metric": "cx_score", "trend": "7d"}'::jsonb, 8, 0, 4, 3),
    ('widget-csat-score', 'default', 'overview', 'metric_card', 'CSAT Score', '{"metric": "csat_score", "trend": "7d"}'::jsonb, 0, 3, 4, 3),
    ('widget-feedback-rate', 'default', 'overview', 'metric_card', 'Satisfaction Rate', '{"metric": "feedback_satisfaction_rate", "trend": "7d"}'::jsonb, 4, 3, 4, 3),
    ('widget-validation-pass', 'default', 'overview', 'metric_card', 'Validation Pass Rate', '{"metric": "validation_pass_rate", "trend": "7d"}'::jsonb, 8, 3, 4, 3),
    ('widget-conversation-trend', 'default', 'overview', 'line_chart', 'Conversation Trend', '{"metric": "conversations_over_time", "period": "7d"}'::jsonb, 0, 6, 8, 6),
    ('widget-topic-distribution', 'default', 'overview', 'pie_chart', 'Topic Distribution', '{"metric": "topic_distribution", "limit": 10}'::jsonb, 8, 6, 4, 6),
    ('widget-channel-breakdown', 'default', 'overview', 'bar_chart', 'Channel Breakdown', '{"metric": "conversations_by_channel", "limit": 10}'::jsonb, 0, 12, 6, 5),
    ('widget-agent-leaderboard', 'default', 'overview', 'table', 'Agent Leaderboard', '{"metric": "agent_performance", "limit": 10, "sort_by": "resolution_rate"}'::jsonb, 6, 12, 6, 5)
ON CONFLICT (id) DO NOTHING;
