-- Migration 0019: Monitoring & Alerting (Phase 8)
--
-- Provides the storage layer for system metrics collection, configurable
-- alert rules, and fired alert events with acknowledgment tracking.
-- This closes the monitoring gap identified in the Intercom research:
-- every component (database, AI provider, channels) is observable, and
-- anomalies trigger alerts before they become outages.

-- Time-series metrics for system health and AI performance.
-- Each row records a single metric snapshot at a point in time.
CREATE TABLE IF NOT EXISTS system_metrics (
    id TEXT PRIMARY KEY,
    metric_name TEXT NOT NULL,           -- 'response_time_ms', 'error_rate', 'validation_score', 'feedback_rate', 'draft_generation_latency', 'api_calls_total', 'active_conversations'
    metric_value REAL NOT NULL,          -- numeric value of the metric
    metric_type TEXT NOT NULL DEFAULT 'gauge',  -- 'gauge', 'counter', 'histogram'
    labels TEXT NOT NULL DEFAULT '{}',   -- JSON object with additional dimensions (e.g., {"channel": "whatsapp", "brand": "main"})
    collected_at TEXT NOT NULL,          -- ISO 8601 timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_system_metrics_name_collected
    ON system_metrics(metric_name, collected_at DESC);

CREATE INDEX IF NOT EXISTS idx_system_metrics_collected
    ON system_metrics(collected_at DESC);

-- Configurable alert rules with thresholds and notification targets.
CREATE TABLE IF NOT EXISTS alert_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    metric_name TEXT NOT NULL,           -- which metric to monitor
    condition TEXT NOT NULL,             -- 'above', 'below', 'equal', 'not_equal'
    threshold REAL NOT NULL,             -- threshold value for the condition
    window_minutes INTEGER NOT NULL DEFAULT 60,  -- time window for aggregation
    severity TEXT NOT NULL DEFAULT 'warning',  -- 'info', 'warning', 'critical'
    enabled BOOLEAN NOT NULL DEFAULT true,  -- 1 = enabled, 0 = disabled
    notification_webhook TEXT,           -- optional webhook URL for notifications
    cooldown_minutes INTEGER NOT NULL DEFAULT 30,  -- minimum interval between alerts for this rule
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled
    ON alert_rules(enabled);

CREATE INDEX IF NOT EXISTS idx_alert_rules_metric
    ON alert_rules(metric_name);

-- Fired alert events with acknowledgment tracking.
CREATE TABLE IF NOT EXISTS alert_events (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    alert_name TEXT NOT NULL,
    severity TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    threshold REAL NOT NULL,
    condition TEXT NOT NULL,
    message TEXT NOT NULL,              -- human-readable alert description
    acknowledged BOOLEAN NOT NULL DEFAULT false,  -- 0 = pending, 1 = acknowledged
    acknowledged_by TEXT,               -- agent name or 'system'
    acknowledged_at TEXT,
    resolved BOOLEAN NOT NULL DEFAULT false,  -- 0 = active, 1 = resolved
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_alert_events_created
    ON alert_events(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_alert_events_acknowledged
    ON alert_events(acknowledged);

CREATE INDEX IF NOT EXISTS idx_alert_events_resolved
    ON alert_events(resolved);

CREATE INDEX IF NOT EXISTS idx_alert_events_severity
    ON alert_events(severity);

-- Seed default alert rules for common monitoring scenarios.
INSERT INTO alert_rules (id, name, description, metric_name, condition, threshold, window_minutes, severity, cooldown_minutes, created_at, updated_at)
VALUES
    ('rule-validation-score-low', 'Low Validation Score',
     'Average validation score dropped below threshold — AI replies may be hallucinating.',
     'validation_score', 'below', 0.7, 60, 'warning', 30,
     datetime('now'), datetime('now')),

    ('rule-error-rate-high', 'High Error Rate',
     'Error rate exceeded 5% — system may be experiencing issues.',
     'error_rate', 'above', 0.05, 30, 'critical', 15,
     datetime('now'), datetime('now')),

    ('rule-response-time-slow', 'Slow Response Time',
     'Average response time exceeded 5 seconds — users may experience delays.',
     'response_time_ms', 'above', 5000.0, 30, 'warning', 30,
     datetime('now'), datetime('now')),

    ('rule-negative-feedback-spike', 'Negative Feedback Spike',
     'Negative feedback rate exceeded 20% — AI quality may be degrading.',
     'negative_feedback_rate', 'above', 0.2, 60, 'warning', 30,
     datetime('now'), datetime('now')),

    ('rule-draft-latency-high', 'High Draft Generation Latency',
     'Draft generation taking longer than 10 seconds — Claude API may be slow.',
     'draft_generation_latency', 'above', 10000.0, 30, 'warning', 30,
     datetime('now'), datetime('now'));
