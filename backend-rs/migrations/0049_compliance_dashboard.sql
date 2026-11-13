-- Phase 41: Compliance & Data Privacy Dashboard
-- Comprehensive compliance management: suppression lists, GDPR data requests,
-- regulatory policies, audit logs, and compliance reporting.

-- Suppression List (contacts who should not receive messages)
CREATE TABLE IF NOT EXISTS suppression_list (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    contact_id TEXT NOT NULL,
    channel TEXT NOT NULL, -- email, sms, whatsapp, all
    reason TEXT NOT NULL, -- opt_out, bounce, complaint, gdpr_deletion, manual, blocked
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by TEXT NOT NULL DEFAULT 'system', -- system, manual, api, gdpr
    note TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true
);
CREATE INDEX IF NOT EXISTS idx_suppression_contact ON suppression_list(contact_id);
CREATE INDEX IF NOT EXISTS idx_suppression_channel ON suppression_list(channel);
CREATE INDEX IF NOT EXISTS idx_suppression_active ON suppression_list(is_active);
CREATE INDEX IF NOT EXISTS idx_suppression_reason ON suppression_list(reason);

-- GDPR Data Requests (access, deletion, portability)
CREATE TABLE IF NOT EXISTS gdpr_requests (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    contact_id TEXT,
    email TEXT,
    request_type TEXT NOT NULL, -- access, deletion, portability, rectification, objection
    status TEXT NOT NULL DEFAULT 'pending', -- pending, in_progress, completed, denied, expired
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    handled_by TEXT,
    notes TEXT,
    data_export_url TEXT, -- for portability requests
    expires_at TIMESTAMPTZ -- auto-expire unhandled requests after 30 days
);
CREATE INDEX IF NOT EXISTS idx_gdpr_status ON gdpr_requests(status);
CREATE INDEX IF NOT EXISTS idx_gdpr_type ON gdpr_requests(request_type);
CREATE INDEX IF NOT EXISTS idx_gdpr_requested ON gdpr_requests(requested_at);

-- Compliance Policies (regulatory rules per channel)
CREATE TABLE IF NOT EXISTS compliance_policies (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    policy_name TEXT NOT NULL UNIQUE,
    regulation TEXT NOT NULL, -- can_spam, tcpa, gdpr, whatsapp_business, ccpa, hipaa
    channel TEXT NOT NULL, -- email, sms, whatsapp, all
    description TEXT,
    rules JSONB NOT NULL DEFAULT '[]', -- specific rules: e.g. {"unsubscribe_link": true, "physical_address": true}
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_policy_regulation ON compliance_policies(regulation);
CREATE INDEX IF NOT EXISTS idx_policy_channel ON compliance_policies(channel);

-- Compliance Audit Log (all compliance-related actions)
CREATE TABLE IF NOT EXISTS compliance_audit_log (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    action TEXT NOT NULL, -- consent_recorded, message_sent, opt_out_processed, gdpr_request_created,
                          -- suppression_added, policy_updated, compliance_check_passed, compliance_violation
    entity_type TEXT NOT NULL, -- contact, message, policy, gdpr_request, suppression
    entity_id TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    performed_by TEXT NOT NULL DEFAULT 'system',
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    severity TEXT NOT NULL DEFAULT 'info' -- info, warning, critical
);
CREATE INDEX IF NOT EXISTS idx_audit_action ON compliance_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON compliance_audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_performed ON compliance_audit_log(performed_at);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON compliance_audit_log(severity);

-- Compliance Settings
CREATE TABLE IF NOT EXISTS compliance_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'default',
    gdpr_response_days INTEGER NOT NULL DEFAULT 30,
    auto_expire_gdpr_requests BOOLEAN NOT NULL DEFAULT true,
    require_consent_before_send BOOLEAN NOT NULL DEFAULT true,
    check_suppression_before_send BOOLEAN NOT NULL DEFAULT true,
    log_all_compliance_actions BOOLEAN NOT NULL DEFAULT true,
    can_spam_unsubscribe_link BOOLEAN NOT NULL DEFAULT true,
    can_spam_physical_address TEXT,
    tcpa_consent_required BOOLEAN NOT NULL DEFAULT true,
    whatsapp_opt_in_required BOOLEAN NOT NULL DEFAULT true,
    data_retention_days INTEGER NOT NULL DEFAULT 365,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default compliance policies
INSERT INTO compliance_policies (policy_name, regulation, channel, description, rules, is_active) VALUES
    ('CAN-SPAM Email Requirements', 'can_spam', 'email', 'All commercial emails must include unsubscribe link, physical address, and clear subject line', '["unsubscribe_link", "physical_address", "clear_subject", "no_deceptive_headers"]', true),
    ('TCPA SMS Consent', 'tcpa', 'sms', 'Prior express consent required before sending SMS; STOP keyword handling mandatory', '["prior_consent", "stop_keyword", "help_keyword", "opt_in_record"]', true),
    ('WhatsApp Business Policy', 'whatsapp_business', 'whatsapp', '24-hour session window, template approval for outbound, quality rating monitoring', '["session_window", "template_approval", "quality_rating", "opt_in_required"]', true),
    ('GDPR Data Protection', 'gdpr', 'all', 'Right to access, deletion, portability; data minimization and purpose limitation', '["right_to_access", "right_to_delete", "right_to_portability", "data_minimization"]', true),
    ('CCPA California Privacy', 'ccpa', 'all', 'Right to know, delete, opt-out of sale; no discrimination for exercising rights', '["right_to_know", "right_to_delete", "opt_out_of_sale"]', true)
ON CONFLICT (policy_name) DO NOTHING;

-- Seed default compliance settings
INSERT INTO compliance_settings (id, org_id) VALUES
    ('default', 'default')
ON CONFLICT (id) DO NOTHING;
