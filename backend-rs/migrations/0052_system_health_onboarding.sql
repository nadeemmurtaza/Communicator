-- Phase 44: System Health Check & Onboarding Wizard
-- Tracks onboarding progress for new organizations and stores
-- system health check results for all components.

-- Onboarding steps (defines the setup checklist)
CREATE TABLE IF NOT EXISTS onboarding_steps (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL, -- 'channels', 'ai', 'analytics', 'compliance', 'integrations'
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    route TEXT NOT NULL, -- link to the relevant dashboard
    check_endpoint TEXT, -- optional API endpoint to auto-check completion
    display_order INTEGER NOT NULL DEFAULT 0,
    is_required BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_onboarding_steps_category ON onboarding_steps(category);
CREATE INDEX IF NOT EXISTS idx_onboarding_steps_order ON onboarding_steps(display_order);

-- Onboarding progress (per-organization tracking)
CREATE TABLE IF NOT EXISTS onboarding_progress (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL UNIQUE,
    step_id TEXT NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    completed_at TIMESTAMPTZ,
    completed_by TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (step_id) REFERENCES onboarding_steps(id)
);

CREATE INDEX IF NOT EXISTS idx_onboarding_progress_org ON onboarding_progress(organization_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_progress_step ON onboarding_progress(step_id);

-- Health check results (stores the latest check for each component)
CREATE TABLE IF NOT EXISTS health_check_results (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    component TEXT NOT NULL, -- 'database', 'claude', 'whatsapp', 'email', 'sms', etc.
    status TEXT NOT NULL, -- 'healthy', 'degraded', 'unhealthy', 'unknown'
    message TEXT,
    details TEXT, -- JSON details
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    check_duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_health_check_org ON health_check_results(organization_id);
CREATE INDEX IF NOT EXISTS idx_health_check_component ON health_check_results(component);

-- Onboarding settings (per-organization config)
CREATE TABLE IF NOT EXISTS onboarding_settings (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL UNIQUE,
    show_onboarding BOOLEAN NOT NULL DEFAULT true,
    auto_check BOOLEAN NOT NULL DEFAULT true,
    health_check_interval_minutes INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default onboarding steps
INSERT INTO onboarding_steps (id, category, title, description, route, display_order, is_required)
VALUES
('onboard-welcome', 'general', 'Welcome to Communicator', 'Complete your organization profile and set up your first brand', '/workspace/organization', 0, true),
('onboard-whatsapp', 'channels', 'Connect WhatsApp', 'Set up WhatsApp Business API for customer messaging', '/channels/whatsapp', 1, false),
('onboard-email', 'channels', 'Connect Email', 'Configure your email domain and SMTP settings', '/channels/email', 2, false),
('onboard-sms', 'channels', 'Connect SMS', 'Set up SMS gateway for text messaging', '/channels/sms', 3, false),
('onboard-knowledge', 'ai', 'Build Knowledge Base', 'Add service catalog entries and upload documents for AI grounding', '/knowledge-base', 4, true),
('onboard-services', 'ai', 'Configure Services', 'Define your service catalog with FAQs and details', '/services', 5, true),
('onboard-tone', 'ai', 'Set AI Tone & Voice', 'Configure how your AI agent communicates with customers', '/tone-of-voice', 6, false),
('onboard-help-center', 'ai', 'Create Help Center', 'Build a public-facing help center for self-service', '/help-center', 7, false),
('onboard-feedback', 'analytics', 'Enable Feedback Loop', 'Verify thumbs up/down feedback collection is working', '/suggestions', 8, false),
('onboard-cx-qa', 'analytics', 'Set Up CX Scoring', 'Configure AI-powered conversation quality scoring', '/cx-qa', 9, false),
('onboard-csat', 'analytics', 'Configure CSAT Surveys', 'Set up customer satisfaction surveys', '/csat', 10, false),
('onboard-monitoring', 'analytics', 'Configure Monitoring', 'Set up system monitoring and alert rules', '/monitoring', 11, false),
('onboard-compliance', 'compliance', 'Review Compliance', 'Configure suppression lists, GDPR settings, and audit logging', '/compliance', 12, true),
('onboard-team', 'integrations', 'Add Team Members', 'Invite your team and configure routing', '/settings/team', 13, false),
('onboard-crm', 'integrations', 'Connect CRM', 'Sync with Salesforce, HubSpot, or Pipedrive', '/crm-sync', 14, false),
('onboard-autonomous', 'ai', 'Enable Autonomous AI', 'Configure AI to resolve conversations without human approval', '/autonomous-resolution', 15, false)
ON CONFLICT (id) DO NOTHING;

-- Default settings
INSERT INTO onboarding_settings (id, organization_id)
VALUES ('onboard-settings-default', 'org-default')
ON CONFLICT (id) DO NOTHING;
