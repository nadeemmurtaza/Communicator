-- Migration 0038: Autonomous Resolution (PostgreSQL)
-- Enables the AI agent to resolve conversations autonomously without human approval.
-- Mirrors Intercom's "Fin AI Agent" that "resolves conversations autonomously across all channels."
-- Includes confidence-based auto-send, escalation rules, and resolution analytics.

-- ============================================================================
-- 1. RESOLUTION CONFIGURATION
-- ============================================================================
-- Per-organization settings for autonomous resolution behavior.
CREATE TABLE IF NOT EXISTS resolution_config (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    auto_send_threshold FLOAT NOT NULL DEFAULT 0.8,
    require_validation BOOLEAN NOT NULL DEFAULT true,
    validation_threshold FLOAT NOT NULL DEFAULT 0.7,
    max_autonomous_attempts INTEGER NOT NULL DEFAULT 3,
    escalate_on_low_confidence BOOLEAN NOT NULL DEFAULT true,
    low_confidence_threshold FLOAT NOT NULL DEFAULT 0.4,
    escalate_on_sensitive_topic BOOLEAN NOT NULL DEFAULT true,
    sensitive_topics TEXT NOT NULL DEFAULT '["complaint", "refund", "legal", "billing"]',
    allowed_channels TEXT NOT NULL DEFAULT '["whatsapp", "email", "sms", "discord", "voice", "live_chat"]',
    cooldown_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_resolution_config_org ON resolution_config(organization_id);

-- Seed default config
INSERT INTO resolution_config (id, organization_id, enabled, auto_send_threshold, require_validation)
VALUES ('resolution-config-default', 'org-default', false, 0.8, true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 2. ESCALATION RULES
-- ============================================================================
-- Defines when and how to escalate from AI to human.
CREATE TABLE IF NOT EXISTS escalation_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    condition_type TEXT NOT NULL DEFAULT 'confidence_below',
    condition_value FLOAT NOT NULL DEFAULT 0.4,
    condition_topic TEXT,
    condition_keyword TEXT,
    action_type TEXT NOT NULL DEFAULT 'assign_to_agent',
    assign_to_agent_id TEXT,
    assign_to_team_id TEXT,
    notify_webhook TEXT,
    notification_message TEXT DEFAULT 'Conversation escalated: {{conversation_id}}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_escalation_rules_org ON escalation_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_escalation_rules_active ON escalation_rules(is_active);
CREATE INDEX IF NOT EXISTS idx_escalation_rules_priority ON escalation_rules(priority DESC);

-- Seed default escalation rules
INSERT INTO escalation_rules (id, organization_id, name, description, condition_type, condition_value, action_type, priority)
VALUES
    ('escalation-low-confidence', 'org-default', 'Low Confidence Escalation', 'Escalate when AI confidence is below threshold', 'confidence_below', 0.4, 'assign_to_agent', 1),
    ('escalation-complaint', 'org-default', 'Complaint Escalation', 'Escalate customer complaints to human agent', 'topic_match', 0.0, 'assign_to_agent', 2),
    ('escalation-refund', 'org-default', 'Refund Request Escalation', 'Escalate refund requests to billing team', 'topic_match', 0.0, 'assign_to_team', 3)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 3. RESOLUTION LOGS
-- ============================================================================
-- Tracks every autonomous resolution attempt and outcome.
CREATE TABLE IF NOT EXISTS resolution_logs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    channel TEXT NOT NULL,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    ai_confidence FLOAT NOT NULL,
    validation_score FLOAT,
    validation_passed BOOLEAN,
    resolution_type TEXT NOT NULL DEFAULT 'auto_send',
    escalated BOOLEAN NOT NULL DEFAULT false,
    escalation_rule_id TEXT,
    escalation_reason TEXT,
    message_sent BOOLEAN NOT NULL DEFAULT false,
    message_id TEXT,
    customer_resolved BOOLEAN,
    customer_feedback_rating INTEGER,
    outcome TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (escalation_rule_id) REFERENCES escalation_rules(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_resolution_logs_org ON resolution_logs(organization_id);
CREATE INDEX IF NOT EXISTS idx_resolution_logs_conversation ON resolution_logs(conversation_id);
CREATE INDEX IF NOT EXISTS idx_resolution_logs_outcome ON resolution_logs(outcome);
CREATE INDEX IF NOT EXISTS idx_resolution_logs_escalated ON resolution_logs(escalated);
CREATE INDEX IF NOT EXISTS idx_resolution_logs_created ON resolution_logs(created_at);

-- ============================================================================
-- 4. RESOLUTION ANALYTICS
-- ============================================================================
-- Aggregated resolution metrics over time windows.
CREATE TABLE IF NOT EXISTS resolution_analytics (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    total_attempts INTEGER NOT NULL DEFAULT 0,
    successful_resolutions INTEGER NOT NULL DEFAULT 0,
    escalated_count INTEGER NOT NULL DEFAULT 0,
    auto_sent_count INTEGER NOT NULL DEFAULT 0,
    avg_confidence FLOAT NOT NULL DEFAULT 0,
    avg_validation_score FLOAT,
    resolution_rate FLOAT NOT NULL DEFAULT 0,
    escalation_rate FLOAT NOT NULL DEFAULT 0,
    customer_satisfaction_rate FLOAT,
    top_escalation_reasons TEXT NOT NULL DEFAULT '[]',
    resolution_by_channel TEXT NOT NULL DEFAULT '{}',
    resolution_by_topic TEXT NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_resolution_analytics_org ON resolution_analytics(organization_id);
CREATE INDEX IF NOT EXISTS idx_resolution_analytics_window ON resolution_analytics(window_start, window_end);

-- ============================================================================
-- 5. AUTONOMOUS AGENT POLICIES
-- ============================================================================
-- Fine-grained policies controlling AI behavior during autonomous resolution.
CREATE TABLE IF NOT EXISTS resolution_agent_policies (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    policy_type TEXT NOT NULL DEFAULT 'response_guardrail',
    condition TEXT NOT NULL,
    action TEXT NOT NULL DEFAULT 'block_and_escalate',
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_resolution_agent_policies_org ON resolution_agent_policies(organization_id);
CREATE INDEX IF NOT EXISTS idx_resolution_agent_policies_active ON resolution_agent_policies(is_active);
CREATE INDEX IF NOT EXISTS idx_resolution_agent_policies_priority ON resolution_agent_policies(priority DESC);

-- Seed default agent policies
INSERT INTO resolution_agent_policies (id, organization_id, name, description, policy_type, condition, action, priority)
VALUES
    ('policy-no-hallucination', 'org-default', 'No Hallucination', 'Block responses with low grounding scores', 'response_guardrail', 'grounding_score < 0.5', 'block_and_escalate', 1),
    ('policy-sensitive-topics', 'org-default', 'Sensitive Topics', 'Escalate sensitive topics to human', 'topic_guardrail', 'topic IN (complaint, refund, legal, billing)', 'escalate', 2),
    ('policy-max-attempts', 'org-default', 'Max Attempts', 'Escalate after max autonomous attempts', 'attempt_limit', 'attempt_count >= 3', 'escalate', 3)
ON CONFLICT (id) DO NOTHING;
