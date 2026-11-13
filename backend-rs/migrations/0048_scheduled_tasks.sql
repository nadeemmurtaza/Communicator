-- Phase 40: Scheduled Tasks & Cron Jobs
-- Automates recurring operations: metric collection, alert evaluation,
-- CSAT survey sending, KB URL re-crawling, analytics snapshots, etc.

-- Scheduled Task Definitions
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    task_type TEXT NOT NULL, -- metric_collection, alert_evaluation, csat_send, kb_crawl, analytics_snapshot, operator_scan, custom_webhook
    cron_schedule TEXT NOT NULL, -- e.g. "0 */6 * * *" (every 6 hours)
    next_run_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB NOT NULL DEFAULT '{}', -- task-specific config
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_st_active ON scheduled_tasks(is_active);
CREATE INDEX IF NOT EXISTS idx_st_type ON scheduled_tasks(task_type);

-- Task Execution Logs
CREATE TABLE IF NOT EXISTS scheduled_task_runs (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    task_id TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'running', -- running, success, failed, skipped
    duration_ms INTEGER,
    result JSONB, -- output data
    error_message TEXT,
    records_processed INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (task_id) REFERENCES scheduled_tasks(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_str_task ON scheduled_task_runs(task_id);
CREATE INDEX IF NOT EXISTS idx_str_status ON scheduled_task_runs(status);
CREATE INDEX IF NOT EXISTS idx_str_started ON scheduled_task_runs(started_at);

-- Task Execution Queue (for tasks that need manual trigger or are queued)
CREATE TABLE IF NOT EXISTS scheduled_task_queue (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    task_id TEXT NOT NULL,
    triggered_by TEXT NOT NULL DEFAULT 'scheduler', -- scheduler, manual, api
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'pending', -- pending, running, completed, failed
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    FOREIGN KEY (task_id) REFERENCES scheduled_tasks(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_stq_status ON scheduled_task_queue(status);

-- Scheduler Settings
CREATE TABLE IF NOT EXISTS scheduler_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_concurrent_tasks INTEGER NOT NULL DEFAULT 3,
    task_timeout_seconds INTEGER NOT NULL DEFAULT 300,
    retry_failed BOOLEAN NOT NULL DEFAULT true,
    retry_count INTEGER NOT NULL DEFAULT 3,
    retry_delay_seconds INTEGER NOT NULL DEFAULT 60,
    log_retention_days INTEGER NOT NULL DEFAULT 30,
    webhook_url TEXT, -- notification webhook for task failures
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default scheduled tasks
INSERT INTO scheduled_tasks (name, description, task_type, cron_schedule, is_active, config) VALUES
    ('Metric Collection', 'Collect system metrics from feedback, validation, conversations', 'metric_collection', '0 */6 * * *', true, '{"window_hours": 6}'),
    ('Alert Evaluation', 'Evaluate alert rules and fire notifications', 'alert_evaluation', '0 */1 * * *', true, '{}'),
    ('CSAT Survey Sending', 'Send CSAT surveys to resolved conversations', 'csat_send', '0 9 * * *', true, '{"hours_since_resolution": 24}'),
    ('KB URL Re-crawling', 'Re-crawl configured URL sources', 'kb_crawl', '0 0 * * 0', true, '{"max_urls": 50}'),
    ('Analytics Snapshot', 'Generate analytics snapshot for unified dashboard', 'analytics_snapshot', '0 0 * * *', true, '{"window_days": 1}'),
    ('Fin Operator Scan', 'Run Fin Operator scan for anomalies and proposals', 'operator_scan', '0 */12 * * *', true, '{}'),
    ('Old Task Log Cleanup', 'Clean up old task execution logs', 'cleanup', '0 3 * * 0', true, '{"retention_days": 30}')
ON CONFLICT (name) DO NOTHING;

-- Seed default scheduler settings
INSERT INTO scheduler_settings (id, org_id) VALUES
    ('default', 'default')
ON CONFLICT (id) DO NOTHING;
