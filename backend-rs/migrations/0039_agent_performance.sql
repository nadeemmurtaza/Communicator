-- Migration 0039: Agent Performance Analytics (PostgreSQL)
-- Per-agent metrics, coaching insights, and performance comparison.
-- Complements AI-powered CX scoring (Phase 16) with human agent analytics.

-- ============================================================================
-- 1. AGENT PERFORMANCE SNAPSHOTS
-- ============================================================================
-- Aggregated per-agent metrics over time windows.
CREATE TABLE IF NOT EXISTS agent_performance_snapshots (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    agent_name TEXT NOT NULL DEFAULT '',
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    conversations_handled INTEGER NOT NULL DEFAULT 0,
    conversations_resolved INTEGER NOT NULL DEFAULT 0,
    avg_response_time_seconds FLOAT NOT NULL DEFAULT 0,
    avg_first_response_time_seconds FLOAT NOT NULL DEFAULT 0,
    avg_resolution_time_seconds FLOAT NOT NULL DEFAULT 0,
    resolution_rate FLOAT NOT NULL DEFAULT 0,
    customer_satisfaction_avg FLOAT,
    customer_satisfaction_count INTEGER NOT NULL DEFAULT 0,
    positive_feedback_count INTEGER NOT NULL DEFAULT 0,
    negative_feedback_count INTEGER NOT NULL DEFAULT 0,
    feedback_rate FLOAT NOT NULL DEFAULT 0,
    messages_sent INTEGER NOT NULL DEFAULT 0,
    messages_received INTEGER NOT NULL DEFAULT 0,
    drafts_approved INTEGER NOT NULL DEFAULT 0,
    drafts_rejected INTEGER NOT NULL DEFAULT 0,
    draft_approval_rate FLOAT NOT NULL DEFAULT 0,
    escalations_received INTEGER NOT NULL DEFAULT 0,
    escalations_sent INTEGER NOT NULL DEFAULT 0,
    sla_breaches INTEGER NOT NULL DEFAULT 0,
    sla_compliance_rate FLOAT NOT NULL DEFAULT 1,
    cx_score_avg FLOAT,
    qa_score_avg FLOAT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_agent_perf_org ON agent_performance_snapshots(organization_id);
CREATE INDEX IF NOT EXISTS idx_agent_perf_agent ON agent_performance_snapshots(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_perf_window ON agent_performance_snapshots(window_start, window_end);

-- ============================================================================
-- 2. AGENT COACHING INSIGHTS
-- ============================================================================
-- AI-generated coaching suggestions for each agent based on performance data.
CREATE TABLE IF NOT EXISTS agent_coaching_insights (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    agent_name TEXT NOT NULL DEFAULT '',
    insight_type TEXT NOT NULL DEFAULT 'improvement',
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'response_quality',
    severity TEXT NOT NULL DEFAULT 'low',
    recommendation TEXT NOT NULL DEFAULT '',
    related_conversation_ids TEXT NOT NULL DEFAULT '[]',
    related_metrics TEXT NOT NULL DEFAULT '{}',
    is_addressed BOOLEAN NOT NULL DEFAULT false,
    addressed_at TIMESTAMPTZ,
    addressed_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_coaching_org ON agent_coaching_insights(organization_id);
CREATE INDEX IF NOT EXISTS idx_coaching_agent ON agent_coaching_insights(agent_id);
CREATE INDEX IF NOT EXISTS idx_coaching_addressed ON agent_coaching_insights(is_addressed);
CREATE INDEX IF NOT EXISTS idx_coaching_severity ON agent_coaching_insights(severity);

-- ============================================================================
-- 3. AGENT COMPARISON METRICS
-- ============================================================================
-- Team-wide comparison metrics for leaderboards and benchmarks.
CREATE TABLE IF NOT EXISTS agent_comparison_metrics (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    team_avg_response_time FLOAT NOT NULL DEFAULT 0,
    team_avg_resolution_rate FLOAT NOT NULL DEFAULT 0,
    team_avg_satisfaction FLOAT,
    team_avg_cx_score FLOAT,
    team_avg_qa_score FLOAT,
    top_performer_id TEXT,
    top_performer_score FLOAT,
    needs_coaching_count INTEGER NOT NULL DEFAULT 0,
    exceeds_benchmark_count INTEGER NOT NULL DEFAULT 0,
    total_agents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_comparison_org ON agent_comparison_metrics(organization_id);
CREATE INDEX IF NOT EXISTS idx_comparison_window ON agent_comparison_metrics(window_start, window_end);

-- ============================================================================
-- 4. AGENT PERFORMANCE SETTINGS
-- ============================================================================
-- Per-organization configuration for agent performance tracking.
CREATE TABLE IF NOT EXISTS agent_performance_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    auto_generate_insights BOOLEAN NOT NULL DEFAULT true,
    insight_frequency TEXT NOT NULL DEFAULT 'weekly',
    benchmark_response_time_seconds INTEGER NOT NULL DEFAULT 300,
    benchmark_resolution_rate FLOAT NOT NULL DEFAULT 0.8,
    benchmark_satisfaction FLOAT NOT NULL DEFAULT 4.0,
    benchmark_cx_score FLOAT NOT NULL DEFAULT 0.8,
    coaching_severity_threshold TEXT NOT NULL DEFAULT 'medium',
    show_leaderboard BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_agent_perf_settings_org ON agent_performance_settings(organization_id);

-- Seed default settings
INSERT INTO agent_performance_settings (id, organization_id, enabled, auto_generate_insights)
VALUES ('agent-perf-settings-default', 'org-default', true, true)
ON CONFLICT (id) DO NOTHING;
