-- Phase 18: AI Procedures & Actions â€” Multi-step AI workflows and external API actions

-- Procedures: natural-language multi-step workflows that teach the AI how to handle complex processes
CREATE TABLE IF NOT EXISTS procedures (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL DEFAULT 'general' CHECK (category IN ('refund', 'troubleshooting', 'order_change', 'account_update', 'escalation', 'billing', 'shipping', 'general')),
    trigger_keywords TEXT NOT NULL DEFAULT '[]', -- JSON array of keywords that trigger this procedure
    steps TEXT NOT NULL DEFAULT '[]', -- JSON array of step objects: {order, name, description, action_type, action_config, expected_response, timeout_seconds}
    max_iterations INTEGER NOT NULL DEFAULT 10,
    timeout_seconds INTEGER NOT NULL DEFAULT 300,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'paused', 'archived')),
    brand_id TEXT, -- NULL means global procedure
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Procedure executions: tracks each time a procedure runs for a conversation
CREATE TABLE IF NOT EXISTS procedure_executions (
    id TEXT PRIMARY KEY,
    procedure_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    current_step INTEGER NOT NULL DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'running' CHECK (state IN ('running', 'completed', 'failed', 'timed_out', 'abandoned')),
    context_data TEXT, -- JSON: accumulated context across steps
    result_data TEXT, -- JSON: final result when completed
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (NOW()),
    completed_at TEXT,
    FOREIGN KEY (procedure_id) REFERENCES procedures(id)
);

-- Procedure step logs: detailed log of each step execution
CREATE TABLE IF NOT EXISTS procedure_step_logs (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL,
    step_order INTEGER NOT NULL,
    step_name TEXT NOT NULL,
    action_type TEXT NOT NULL,
    action_input TEXT, -- JSON: input to the action
    action_output TEXT, -- JSON: output from the action
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'skipped')),
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (execution_id) REFERENCES procedure_executions(id)
);

-- Actions: reusable API connectors that procedures can call
CREATE TABLE IF NOT EXISTS actions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    action_type TEXT NOT NULL CHECK (action_type IN ('http_request', 'database_query', 'webhook', 'stripe', 'internal_api', 'custom')),
    endpoint TEXT, -- URL for HTTP/webhook actions
    method TEXT DEFAULT 'POST', -- HTTP method
    headers TEXT DEFAULT '{}', -- JSON: default headers
    body_template TEXT, -- JSON template with placeholders
    auth_config TEXT, -- JSON: authentication configuration
    timeout_seconds INTEGER NOT NULL DEFAULT 30,
    retry_count INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    brand_id TEXT, -- NULL means global action
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Action executions: tracks each action call
CREATE TABLE IF NOT EXISTS action_executions (
    id TEXT PRIMARY KEY,
    action_id TEXT NOT NULL,
    procedure_execution_id TEXT, -- NULL if called directly
    input_params TEXT, -- JSON: input parameters
    output_data TEXT, -- JSON: response from the action
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'timeout')),
    http_status_code INTEGER,
    error_message TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (action_id) REFERENCES actions(id),
    FOREIGN KEY (procedure_execution_id) REFERENCES procedure_executions(id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_procedures_status ON procedures(status);
CREATE INDEX IF NOT EXISTS idx_procedures_category ON procedures(category);
CREATE INDEX IF NOT EXISTS idx_procedures_brand ON procedures(brand_id);
CREATE INDEX IF NOT EXISTS idx_procedure_executions_conversation ON procedure_executions(conversation_id);
CREATE INDEX IF NOT EXISTS idx_procedure_executions_state ON procedure_executions(state);
CREATE INDEX IF NOT EXISTS idx_procedure_executions_procedure ON procedure_executions(procedure_id);
CREATE INDEX IF NOT EXISTS idx_procedure_step_logs_execution ON procedure_step_logs(execution_id);
CREATE INDEX IF NOT EXISTS idx_actions_type ON actions(action_type);
CREATE INDEX IF NOT EXISTS idx_actions_enabled ON actions(enabled);
CREATE INDEX IF NOT EXISTS idx_action_executions_action ON action_executions(action_id);
CREATE INDEX IF NOT EXISTS idx_action_executions_procedure ON action_executions(procedure_execution_id);

-- Default actions for common integrations
INSERT INTO actions (id, name, description, action_type, endpoint, method, timeout_seconds, enabled)
VALUES 
('action-stripe-refund', 'Stripe Refund', 'Process a refund via Stripe API', 'stripe', 'https://api.stripe.com/v1/refunds', 'POST', 30, 1),
('action-stripe-lookup', 'Stripe Customer Lookup', 'Look up customer billing info via Stripe', 'stripe', 'https://api.stripe.com/v1/customers', 'GET', 30, 1),
('action-internal-order', 'Internal Order Lookup', 'Look up order details from internal database', 'internal_api', '/api/v1/commerce/orders/{order_id}', 'GET', 15, 1),
('action-internal-update', 'Internal Order Update', 'Update order status from internal database', 'internal_api', '/api/v1/commerce/orders/{order_id}/update-status', 'POST', 15, 1);

-- Example procedure: Refund workflow
INSERT INTO procedures (id, name, description, category, trigger_keywords, steps, max_iterations, timeout_seconds, status)
VALUES (
    'proc-refund',
    'Process Refund',
    'Guide the customer through the refund process: verify order, check eligibility, process refund',
    'refund',
    '["refund", "money back", "return", "reimburse"]',
    '[
        {
            "order": 1,
            "name": "Verify Order",
            "description": "Ask the customer for their order ID and look up the order details",
            "action_type": "ask_customer",
            "action_config": {"question": "What is your order ID?", "required": true},
            "expected_response": "order_id",
            "timeout_seconds": 120
        },
        {
            "order": 2,
            "name": "Lookup Order",
            "description": "Look up the order details using the order ID",
            "action_type": "call_action",
            "action_config": {"action_id": "action-internal-order", "params": {"order_id": "{{order_id}}"}},
            "expected_response": "order_details",
            "timeout_seconds": 30
        },
        {
            "order": 3,
            "name": "Check Eligibility",
            "description": "Check if the order is eligible for refund based on policy",
            "action_type": "evaluate",
            "action_config": {"condition": "order_details.status IN (\"delivered\", \"shipped\") AND order_details.days_since_delivery < 30"},
            "expected_response": "eligible",
            "timeout_seconds": 5
        },
        {
            "order": 4,
            "name": "Confirm Refund",
            "description": "Confirm the refund details with the customer",
            "action_type": "ask_customer",
            "action_config": {"question": "I found your order {{order_details.order_number}} for {{order_details.total}}. Would you like to proceed with the refund?", "required": true},
            "expected_response": "confirmed",
            "timeout_seconds": 120
        },
        {
            "order": 5,
            "name": "Process Refund",
            "description": "Process the refund via Stripe",
            "action_type": "call_action",
            "action_config": {"action_id": "action-stripe-refund", "params": {"payment_intent": "{{order_details.payment_intent}}", "amount": "{{order_details.total}}"}},
            "expected_response": "refund_result",
            "timeout_seconds": 30
        }
    ]',
    10,
    600,
    'active'
);
