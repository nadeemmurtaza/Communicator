-- Phase 20: Visual Workflow Builder â€” No-code automation engine
-- Enables creating multi-step workflows with conditional branching,
-- assignment rules, and trigger-based execution, mirroring Intercom's
-- visual workflow builder capability.

-- Workflow definitions (top-level automation flows)
CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    workflow_type TEXT NOT NULL DEFAULT 'automation', -- 'automation' | 'assignment' | 'notification' | 'escalation' | 'custom'
    status TEXT NOT NULL DEFAULT 'draft', -- 'draft' | 'active' | 'paused' | 'archived'
    trigger_type TEXT NOT NULL, -- 'event' | 'schedule' | 'manual' | 'webhook'
    trigger_config TEXT, -- JSON: event type, schedule cron, webhook path, etc.
    steps_json TEXT, -- JSON: ordered list of step definitions (visual editor state)
    execution_count BOOLEAN NOT NULL DEFAULT false,
    last_executed_at TEXT,
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflows_org ON workflows(organization_id);
CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
CREATE INDEX IF NOT EXISTS idx_workflows_type ON workflows(workflow_type);

-- Workflow steps (individual actions within a workflow)
CREATE TABLE IF NOT EXISTS workflow_steps (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    step_order INTEGER NOT NULL,
    step_type TEXT NOT NULL, -- 'wait' | 'send_message' | 'assign' | 'tag' | 'condition' | 'action' | 'notify' | 'update_field' | 'create_ticket' | 'custom'
    name TEXT NOT NULL,
    config TEXT, -- JSON: step-specific configuration
    condition_branch TEXT, -- 'true' | 'false' | NULL (for conditional steps)
    parent_step_id TEXT, -- NULL for top-level steps; references parent condition step ID
    on_success_next TEXT, -- step_id to go to on success (for branching)
    on_failure_next TEXT, -- step_id to go to on failure (for error handling)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow ON workflow_steps(workflow_id, step_order);

-- Workflow conditions (conditional branching logic)
CREATE TABLE IF NOT EXISTS workflow_conditions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    step_id TEXT NOT NULL,
    condition_type TEXT NOT NULL, -- 'attribute' | 'conversation' | 'time' | 'custom'
    field TEXT NOT NULL,
    operator TEXT NOT NULL, -- 'equals' | 'not_equals' | 'contains' | 'greater_than' | 'less_than' | 'is_empty' | 'is_not_empty'
    value TEXT,
    logic_operator TEXT NOT NULL DEFAULT 'and', -- 'and' | 'or' (relative to previous condition)
    branch_on_match TEXT NOT NULL DEFAULT 'true', -- which branch to take when condition matches
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_conditions_workflow ON workflow_conditions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_conditions_step ON workflow_conditions(step_id);

-- Workflow execution tracking
CREATE TABLE IF NOT EXISTS workflow_executions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    trigger_source TEXT, -- 'event' | 'schedule' | 'manual' | 'webhook'
    trigger_data TEXT, -- JSON: data that triggered the workflow
    target_contact_id TEXT,
    target_conversation_id TEXT,
    current_step_id TEXT,
    status TEXT NOT NULL DEFAULT 'running', -- 'running' | 'completed' | 'failed' | 'paused' | 'cancelled'
    step_results TEXT, -- JSON: results of each step execution
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (NOW()),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow ON workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_contact ON workflow_executions(target_contact_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_started ON workflow_executions(started_at);

-- Workflow triggers (event-based triggers that start workflows)
CREATE TABLE IF NOT EXISTS workflow_triggers (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    trigger_name TEXT NOT NULL,
    trigger_event TEXT NOT NULL, -- 'conversation_created' | 'message_received' | 'message_sent' | 'contact_updated' | 'tag_added' | 'state_changed' | 'feedback_received' | 'scheduled' | 'webhook'
    filter_conditions TEXT, -- JSON: additional filter conditions for the trigger
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_triggers_workflow ON workflow_triggers(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_triggers_event ON workflow_triggers(trigger_event);

-- Default workflows for common use cases
INSERT INTO workflows (id, organization_id, name, description, workflow_type, status, trigger_type, trigger_config)
SELECT 'wf-new-conversation', 'org-default', 'New Conversation Notification', 'Notify team when a new conversation is created', 'notification', 'draft', 'event', '{"event": "conversation_created"}'
WHERE NOT EXISTS (SELECT 1 FROM workflows WHERE id = 'wf-new-conversation');

INSERT INTO workflows (id, organization_id, name, description, workflow_type, status, trigger_type, trigger_config)
SELECT 'wf-auto-assign', 'org-default', 'Auto-Assign by Topic', 'Automatically assign conversations based on topic', 'assignment', 'draft', 'event', '{"event": "conversation_created"}'
WHERE NOT EXISTS (SELECT 1 FROM workflows WHERE id = 'wf-auto-assign');

INSERT INTO workflows (id, organization_id, name, description, workflow_type, status, trigger_type, trigger_config)
SELECT 'wf-escalation', 'org-default', 'Escalate Unresolved', 'Escalate conversations unresolved after 24 hours', 'escalation', 'draft', 'schedule', '{"cron": "0 * * * *", "condition": "unresolved_24h"}'
WHERE NOT EXISTS (SELECT 1 FROM workflows WHERE id = 'wf-escalation');
