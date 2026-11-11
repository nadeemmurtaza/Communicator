-- Phase 17: Fin Operator â€” Meta-agent for autonomous self-improvement

-- Operator proposals: AI-generated improvement suggestions with review workflow
CREATE TABLE IF NOT EXISTS operator_proposals (
    id TEXT PRIMARY KEY,
    proposal_type TEXT NOT NULL CHECK (proposal_type IN ('knowledge_update', 'procedure_change', 'policy_update', 'prompt_tuning', 'threshold_adjustment', 'content_creation', 'escalation_rule', 'general')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,
    expected_impact TEXT, -- JSON: {"metric": "validation_score", "direction": "increase", "estimated_delta": 0.05}
    implementation_plan TEXT, -- JSON array of steps
    status TEXT NOT NULL DEFAULT 'proposed' CHECK (status IN ('proposed', 'reviewing', 'approved', 'testing', 'deployed', 'dismissed', 'failed')),
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    confidence REAL NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT 'auto' CHECK (source IN ('auto', 'manual', 'feedback_trigger', 'threshold_breach')),
    source_data TEXT, -- JSON: reference to the data that triggered this proposal
    related_feedback_ids TEXT, -- JSON array of feedback IDs
    related_conversation_ids TEXT, -- JSON array of conversation IDs
    created_by TEXT NOT NULL DEFAULT 'operator',
    reviewed_by TEXT,
    reviewed_at TEXT,
    deployed_at TEXT,
    deployment_result TEXT, -- JSON: {"success": true, "metrics_before": {}, "metrics_after": {}}
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Operator performance log: tracks what the operator did and the outcome
CREATE TABLE IF NOT EXISTS operator_performance_log (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL CHECK (action_type IN ('scan', 'analyze', 'propose', 'test', 'deploy', 'rollback', 'monitor')),
    action_detail TEXT NOT NULL,
    target TEXT, -- what was acted upon (e.g., proposal_id, metric_name)
    result TEXT NOT NULL CHECK (result IN ('success', 'failure', 'partial', 'skipped')),
    output_data TEXT, -- JSON: result of the action
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Operator configuration: controls how the operator behaves
CREATE TABLE IF NOT EXISTS operator_config (
    id TEXT PRIMARY KEY DEFAULT 'default',
    scan_interval_minutes INTEGER NOT NULL DEFAULT 60,
    auto_deploy_enabled BOOLEAN NOT NULL DEFAULT false,
    auto_deploy_max_risk TEXT NOT NULL DEFAULT 'low' CHECK (auto_deploy_max_risk IN ('low', 'medium', 'high')),
    max_proposals_per_scan INTEGER NOT NULL DEFAULT 5,
    min_confidence_threshold REAL NOT NULL DEFAULT 0.7,
    metrics_to_monitor TEXT NOT NULL DEFAULT '["validation_score", "feedback_rate", "error_rate", "response_time", "resolution_rate"]',
    alert_on_failure BOOLEAN NOT NULL DEFAULT true,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_scan_at TEXT,
    last_scan_result TEXT, -- JSON summary of last scan
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Operator metrics snapshot: periodic snapshot of all monitored metrics
CREATE TABLE IF NOT EXISTS operator_metrics_snapshot (
    id TEXT PRIMARY KEY,
    snapshot_time TEXT NOT NULL,
    metrics_data TEXT NOT NULL, -- JSON: {"validation_score": 0.85, "feedback_rate": 0.92, ...}
    anomalies TEXT, -- JSON array of detected anomalies
    trend_data TEXT, -- JSON: {"validation_score": {"trend": "up", "change": 0.02}, ...}
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default operator config
INSERT INTO operator_config (id, scan_interval_minutes, auto_deploy_enabled, auto_deploy_max_risk, max_proposals_per_scan, min_confidence_threshold, enabled)
VALUES ('default', 60, 0, 'low', 5, 0.7, 1);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_operator_proposals_status ON operator_proposals(status);
CREATE INDEX IF NOT EXISTS idx_operator_proposals_type ON operator_proposals(proposal_type);
CREATE INDEX IF NOT EXISTS idx_operator_proposals_created ON operator_proposals(created_at);
CREATE INDEX IF NOT EXISTS idx_operator_proposals_priority ON operator_proposals(priority);
CREATE INDEX IF NOT EXISTS idx_operator_performance_action ON operator_performance_log(action_type);
CREATE INDEX IF NOT EXISTS idx_operator_performance_created ON operator_performance_log(created_at);
CREATE INDEX IF NOT EXISTS idx_operator_metrics_snapshot_time ON operator_metrics_snapshot(snapshot_time);
