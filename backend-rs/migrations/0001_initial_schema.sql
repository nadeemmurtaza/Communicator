-- Consolidated schema migration for Communicator Rust Backend
-- Translated from 30 Alembic Python migrations
-- Target: SQLite (default for desktop/web apps), Postgres-compatible where possible
-- This single migration creates the entire schema from scratch

-- Tenants
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    owner_account_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tenants_tenant_key ON tenants(tenant_key);

-- Leads
CREATE TABLE IF NOT EXISTS leads (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    harvester_institution_id TEXT NOT NULL,
    name TEXT NOT NULL,
    institution_type TEXT NOT NULL DEFAULT '',
    city TEXT NOT NULL DEFAULT '',
    qualification_rationale TEXT NOT NULL DEFAULT '',
    synced_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_leads_harvester_id ON leads(harvester_institution_id);
CREATE INDEX IF NOT EXISTS idx_leads_tenant_key ON leads(tenant_key);

-- Customers (cross-channel person identity)
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    lead_id TEXT,
    display_name TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    FOREIGN KEY (lead_id) REFERENCES leads(id)
);
CREATE INDEX IF NOT EXISTS idx_customers_tenant_key ON customers(tenant_key);

-- Contacts
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    customer_id TEXT,
    lead_id TEXT NOT NULL,
    harvester_contact_point_id TEXT NOT NULL DEFAULT '',
    channel TEXT NOT NULL,
    value TEXT NOT NULL,
    person_name TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT '',
    language TEXT NOT NULL DEFAULT 'en',
    custom_fields TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    FOREIGN KEY (lead_id) REFERENCES leads(id),
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);
CREATE INDEX IF NOT EXISTS idx_contacts_tenant_key ON contacts(tenant_key);
CREATE INDEX IF NOT EXISTS idx_contacts_lead_id ON contacts(lead_id);
CREATE INDEX IF NOT EXISTS idx_contacts_customer_id ON contacts(customer_id);
CREATE INDEX IF NOT EXISTS idx_contacts_harvester_cp_id ON contacts(harvester_contact_point_id);
CREATE INDEX IF NOT EXISTS idx_contacts_value ON contacts(value);
CREATE INDEX IF NOT EXISTS idx_contacts_channel ON contacts(channel);

-- Consent Records
CREATE TABLE IF NOT EXISTS consent_records (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    recorded_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_consent_contact_id ON consent_records(contact_id);

-- Contact Memories
CREATE TABLE IF NOT EXISTS contact_memories (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    fact TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'preference',
    confidence REAL NOT NULL DEFAULT 1.0,
    source_message_id TEXT,
    noted_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id),
    FOREIGN KEY (source_message_id) REFERENCES messages(id)
);
CREATE INDEX IF NOT EXISTS idx_contact_memories_contact_id ON contact_memories(contact_id);

-- Contact Notes
CREATE TABLE IF NOT EXISTS contact_notes (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    author_name TEXT NOT NULL DEFAULT 'Agent',
    body TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_contact_notes_contact_id ON contact_notes(contact_id);

-- Tags & Contact Tags (many-to-many)
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_tenant_name ON tags(tenant_key, name);

CREATE TABLE IF NOT EXISTS contact_tags (
    contact_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (contact_id, tag_id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Inboxes
CREATE TABLE IF NOT EXISTS inboxes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    phone_number TEXT NOT NULL DEFAULT '',
    avatar_url TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    credentials TEXT NOT NULL DEFAULT '{}',
    greeting_enabled INTEGER NOT NULL DEFAULT 1,
    greeting_message TEXT NOT NULL DEFAULT '',
    out_of_office_enabled INTEGER NOT NULL DEFAULT 0,
    out_of_office_message TEXT NOT NULL DEFAULT '',
    csat_enabled INTEGER NOT NULL DEFAULT 1,
    auto_read_receipts INTEGER NOT NULL DEFAULT 1,
    working_hours_enabled INTEGER NOT NULL DEFAULT 0,
    working_hours_start INTEGER NOT NULL DEFAULT 9,
    working_hours_end INTEGER NOT NULL DEFAULT 18,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_inboxes_channel_type ON inboxes(channel_type);

-- Conversations
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    contact_id TEXT NOT NULL,
    contact_name TEXT,
    customer_name TEXT,
    lead_name TEXT,
    contact_phone TEXT,
    phone TEXT,
    channel TEXT,
    state TEXT NOT NULL DEFAULT 'open',
    priority TEXT,
    resolution_type TEXT,
    broadcast_id TEXT,
    broadcast_name TEXT,
    inbox_id TEXT,
    team_id TEXT,
    team_name TEXT,
    csat_score INTEGER,
    human_takeover INTEGER NOT NULL DEFAULT 0,
    assigned_to_id TEXT,
    assigned_to_name TEXT,
    snoozed_until TEXT,
    language TEXT,
    sequence_id TEXT,
    sequence_step INTEGER NOT NULL DEFAULT 0,
    sequence_variant TEXT DEFAULT 'control',
    next_followup_at TEXT,
    custom_attributes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id),
    FOREIGN KEY (inbox_id) REFERENCES inboxes(id),
    FOREIGN KEY (sequence_id) REFERENCES follow_up_sequences(id)
);
CREATE INDEX IF NOT EXISTS idx_conversations_tenant_key ON conversations(tenant_key);
CREATE INDEX IF NOT EXISTS idx_conversations_contact_id ON conversations(contact_id);
CREATE INDEX IF NOT EXISTS idx_conversations_state ON conversations(state);
CREATE INDEX IF NOT EXISTS idx_conversations_team_id ON conversations(team_id);
CREATE INDEX IF NOT EXISTS idx_conversations_assigned_to ON conversations(assigned_to_id);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    channel TEXT NOT NULL,
    body TEXT NOT NULL,
    intent TEXT,
    status TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT '',
    provider_message_id TEXT NOT NULL DEFAULT '',
    provider_metadata TEXT NOT NULL DEFAULT '{}',
    sender_name TEXT,
    is_private INTEGER,
    sent_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_provider_msg_id ON messages(provider_message_id);

-- Drafts
CREATE TABLE IF NOT EXISTS drafts (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    body TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.0,
    citations TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    reviewer TEXT NOT NULL DEFAULT '',
    reviewer_feedback TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
CREATE INDEX IF NOT EXISTS idx_drafts_conversation_id ON drafts(conversation_id);

-- Audit Logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    actor TEXT NOT NULL DEFAULT 'system',
    details TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity_id ON audit_logs(entity_id);

-- Services
CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    name TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    details TEXT NOT NULL DEFAULT '',
    target_institution_types TEXT NOT NULL DEFAULT '[]',
    faq TEXT NOT NULL DEFAULT '[]',
    pricing_notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_services_tenant_key ON services(tenant_key);

-- Follow-up Sequences & Steps
CREATE TABLE IF NOT EXISTS follow_up_sequences (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sequence_steps (
    id TEXT PRIMARY KEY,
    sequence_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    delay_days INTEGER NOT NULL,
    channel TEXT NOT NULL,
    template TEXT NOT NULL DEFAULT '',
    variant TEXT NOT NULL DEFAULT 'control',
    FOREIGN KEY (sequence_id) REFERENCES follow_up_sequences(id)
);
CREATE INDEX IF NOT EXISTS idx_sequence_steps_seq_id ON sequence_steps(sequence_id);

-- Automation Rules
CREATE TABLE IF NOT EXISTS automation_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    trigger_keyword TEXT NOT NULL,
    match_type TEXT NOT NULL,
    action_type TEXT NOT NULL,
    action_value TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    execution_count INTEGER,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL
);

-- Canned Responses
CREATE TABLE IF NOT EXISTS canned_responses (
    id TEXT PRIMARY KEY,
    short_code TEXT NOT NULL,
    content TEXT NOT NULL,
    category TEXT,
    created_at TEXT NOT NULL
);

-- Macros
CREATE TABLE IF NOT EXISTS macros (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    actions TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Team Members & Teams
CREATE TABLE IF NOT EXISTS team_members (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    role TEXT,
    availability_status TEXT,
    max_concurrent_conversations INTEGER
);

CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    auto_assign INTEGER
);

CREATE TABLE IF NOT EXISTS team_memberships (
    team_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    PRIMARY KEY (team_id, member_id),
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (member_id) REFERENCES team_members(id) ON DELETE CASCADE
);

-- RBAC Roles
CREATE TABLE IF NOT EXISTS role_definitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_system_role INTEGER NOT NULL DEFAULT 0,
    permissions TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- SLA Policies
CREATE TABLE IF NOT EXISTS sla_policies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    first_response_time_threshold INTEGER,
    resolution_time_threshold INTEGER,
    only_during_business_hours INTEGER,
    priorities TEXT,
    inbox_ids TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Tickets & Activities
CREATE TABLE IF NOT EXISTS tickets (
    id TEXT PRIMARY KEY,
    ticket_number TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    ticket_type TEXT NOT NULL,
    department TEXT NOT NULL,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    conversation_id TEXT,
    contact_id TEXT,
    assigned_to_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_tickets_ticket_number ON tickets(ticket_number);
CREATE INDEX IF NOT EXISTS idx_tickets_status ON tickets(status);

CREATE TABLE IF NOT EXISTS ticket_activities (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id)
);
CREATE INDEX IF NOT EXISTS idx_ticket_activities_ticket_id ON ticket_activities(ticket_id);

-- Knowledge Base
CREATE TABLE IF NOT EXISTS knowledge_base_entries (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    source_type TEXT NOT NULL DEFAULT 'note',
    source_url TEXT,
    file_name TEXT,
    topic TEXT NOT NULL DEFAULT 'General',
    target_touchpoints TEXT NOT NULL DEFAULT '["agent","copilot","help_center"]',
    status TEXT NOT NULL DEFAULT 'published',
    resolution_rate REAL NOT NULL DEFAULT 85.0,
    usage_count INTEGER NOT NULL DEFAULT 0,
    helpful_count INTEGER NOT NULL DEFAULT 0,
    unhelpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_kb_entries_tenant_key ON knowledge_base_entries(tenant_key);
CREATE INDEX IF NOT EXISTS idx_kb_entries_topic ON knowledge_base_entries(topic);

-- Knowledge Gaps
CREATE TABLE IF NOT EXISTS knowledge_gaps (
    id TEXT PRIMARY KEY,
    question TEXT NOT NULL,
    suggested_answer TEXT NOT NULL DEFAULT '',
    source_query TEXT NOT NULL DEFAULT '',
    frequency INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'suggested',
    created_at TEXT NOT NULL
);

-- Knowledge Recommendations
CREATE TABLE IF NOT EXISTS knowledge_recommendations (
    id TEXT PRIMARY KEY,
    gap_type TEXT NOT NULL DEFAULT 'content_gap',
    topic TEXT NOT NULL DEFAULT 'General',
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    impact_on_resolution REAL NOT NULL DEFAULT 3.5,
    suggested_action TEXT NOT NULL DEFAULT '',
    suggested_content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL,
    resolved_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_kb_recommendations_status ON knowledge_recommendations(status);

-- Broadcasts & Recipients
CREATE TABLE IF NOT EXISTS broadcasts (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    name TEXT NOT NULL,
    campaign_type TEXT NOT NULL DEFAULT '',
    objective TEXT NOT NULL DEFAULT '',
    campaign_template_id TEXT,
    parent_campaign_id TEXT,
    step_index INTEGER NOT NULL DEFAULT 0,
    step_delay_days INTEGER NOT NULL DEFAULT 0,
    stop_on_reply INTEGER NOT NULL DEFAULT 1,
    template_name TEXT NOT NULL,
    template_language TEXT NOT NULL DEFAULT 'en_US',
    body_preview TEXT NOT NULL DEFAULT '',
    template_params TEXT NOT NULL DEFAULT '{}',
    tag_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    scheduled_at TEXT,
    audience_count INTEGER NOT NULL DEFAULT 0,
    sent_count INTEGER NOT NULL DEFAULT 0,
    delivered_count INTEGER NOT NULL DEFAULT 0,
    read_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (campaign_template_id) REFERENCES campaign_templates(id),
    FOREIGN KEY (parent_campaign_id) REFERENCES broadcasts(id)
);
CREATE INDEX IF NOT EXISTS idx_broadcasts_tenant_key ON broadcasts(tenant_key);

CREATE TABLE IF NOT EXISTS broadcast_recipients (
    id TEXT PRIMARY KEY,
    broadcast_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    provider_message_id TEXT NOT NULL DEFAULT '',
    error TEXT NOT NULL DEFAULT '',
    sent_at TEXT,
    delivered_at TEXT,
    read_at TEXT,
    FOREIGN KEY (broadcast_id) REFERENCES broadcasts(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_broadcast_recipients_broadcast_id ON broadcast_recipients(broadcast_id);
CREATE INDEX IF NOT EXISTS idx_broadcast_recipients_provider_msg_id ON broadcast_recipients(provider_message_id);

-- Campaign Templates
CREATE TABLE IF NOT EXISTS campaign_templates (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL,
    key TEXT NOT NULL,
    channel TEXT NOT NULL DEFAULT '',
    name TEXT NOT NULL,
    purpose TEXT NOT NULL DEFAULT '',
    meta_template_name TEXT NOT NULL,
    meta_language TEXT NOT NULL DEFAULT 'en_US',
    meta_category TEXT NOT NULL DEFAULT 'MARKETING',
    meta_status TEXT NOT NULL DEFAULT 'PENDING',
    body_preview TEXT NOT NULL DEFAULT '',
    variables TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_campaign_templates_tenant_key ON campaign_templates(tenant_key);

-- Integration Settings (singleton)
CREATE TABLE IF NOT EXISTS integration_settings (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax' UNIQUE,
    whatsapp_provider TEXT NOT NULL DEFAULT 'meta',
    meta_access_token TEXT NOT NULL DEFAULT '',
    meta_phone_number_id TEXT NOT NULL DEFAULT '',
    meta_waba_id TEXT NOT NULL DEFAULT '',
    meta_app_secret TEXT NOT NULL DEFAULT '',
    meta_webhook_verify_token TEXT NOT NULL DEFAULT '',
    whatsapp_verified_at TEXT,
    whatsapp_verified_name TEXT NOT NULL DEFAULT '',
    whatsapp_display_number TEXT NOT NULL DEFAULT '',
    business_name TEXT NOT NULL DEFAULT '',
    business_description TEXT NOT NULL DEFAULT '',
    agent_guidelines TEXT NOT NULL DEFAULT '',
    auto_reply_enabled INTEGER NOT NULL DEFAULT 1,
    business_hours_enabled INTEGER NOT NULL DEFAULT 0,
    business_hours_start INTEGER NOT NULL DEFAULT 9,
    business_hours_end INTEGER NOT NULL DEFAULT 18,
    business_timezone TEXT NOT NULL DEFAULT 'UTC',
    away_message TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_integration_settings_tenant ON integration_settings(tenant_key);

-- Channel Gateway Connections
CREATE TABLE IF NOT EXISTS channel_gateway_connections (
    id TEXT PRIMARY KEY,
    gateway_type TEXT NOT NULL,
    provider TEXT NOT NULL,
    name TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    credentials TEXT NOT NULL DEFAULT '{}',
    config TEXT NOT NULL DEFAULT '{}',
    test_status TEXT NOT NULL DEFAULT 'untested',
    latency_ms INTEGER NOT NULL DEFAULT 0,
    last_tested_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- AI Connections
CREATE TABLE IF NOT EXISTS ai_connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'anthropic',
    model_id TEXT NOT NULL,
    api_key TEXT NOT NULL DEFAULT '',
    base_url TEXT,
    temperature REAL NOT NULL DEFAULT 0.7,
    max_tokens INTEGER NOT NULL DEFAULT 1024,
    is_active INTEGER NOT NULL DEFAULT 1,
    assigned_jobs TEXT NOT NULL DEFAULT '[]',
    last_tested_at TEXT,
    test_status TEXT NOT NULL DEFAULT 'untested',
    latency_ms INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ai_connections_provider ON ai_connections(provider);

-- AI Custom Policies
CREATE TABLE IF NOT EXISTS ai_custom_policies (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL DEFAULT 'self_improving' UNIQUE,
    tone TEXT NOT NULL DEFAULT 'empathetic',
    guidelines TEXT NOT NULL DEFAULT '',
    rules TEXT NOT NULL DEFAULT '[]',
    guardrails TEXT NOT NULL DEFAULT '[]',
    auto_approval_mode TEXT NOT NULL DEFAULT 'require_manager_review',
    min_csat_learning_threshold REAL NOT NULL DEFAULT 4.0,
    max_autonomous_refund REAL NOT NULL DEFAULT 50.0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ai_custom_policies_job_type ON ai_custom_policies(job_type);

-- Copilot Approval Requests
CREATE TABLE IF NOT EXISTS copilot_approvals (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    contact_id TEXT,
    request_type TEXT NOT NULL DEFAULT 'refund',
    status TEXT NOT NULL DEFAULT 'pending',
    title TEXT NOT NULL,
    proposed_reply TEXT NOT NULL DEFAULT '',
    amended_reply TEXT,
    action_data TEXT NOT NULL DEFAULT '{}',
    internal_note TEXT NOT NULL DEFAULT '',
    question_to_teammate TEXT,
    teammate_answer TEXT,
    assigned_to_id TEXT,
    reviewer_name TEXT,
    reviewer_feedback TEXT,
    dispatched_message_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    resolved_at TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (assigned_to_id) REFERENCES team_members(id) ON DELETE SET NULL,
    FOREIGN KEY (dispatched_message_id) REFERENCES messages(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_copilot_approvals_conversation_id ON copilot_approvals(conversation_id);
CREATE INDEX IF NOT EXISTS idx_copilot_approvals_status ON copilot_approvals(status);

-- Discovery Flow Templates
CREATE TABLE IF NOT EXISTS discovery_flow_templates (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    steps TEXT NOT NULL DEFAULT '[]',
    problem_category_keywords TEXT NOT NULL DEFAULT '{}',
    capability_keywords TEXT NOT NULL DEFAULT '{}',
    capability_labels TEXT NOT NULL DEFAULT '{}',
    scoring_weights TEXT NOT NULL DEFAULT '{}',
    intro_message TEXT NOT NULL DEFAULT '',
    closing_message TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Inquiries
CREATE TABLE IF NOT EXISTS inquiries (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    conversation_id TEXT NOT NULL UNIQUE,
    contact_id TEXT NOT NULL,
    lead_id TEXT,
    organization_name TEXT NOT NULL DEFAULT '',
    organization_status TEXT NOT NULL DEFAULT '',
    industry TEXT NOT NULL DEFAULT '',
    contact_role TEXT NOT NULL DEFAULT '',
    org_size_range TEXT NOT NULL DEFAULT '',
    locations_count TEXT NOT NULL DEFAULT '',
    problem_description TEXT NOT NULL DEFAULT '',
    problem_categories TEXT NOT NULL DEFAULT '[]',
    reported_impact TEXT NOT NULL DEFAULT '[]',
    frequency TEXT NOT NULL DEFAULT '',
    impact_level TEXT NOT NULL DEFAULT '',
    current_workaround TEXT NOT NULL DEFAULT '',
    limitations TEXT NOT NULL DEFAULT '[]',
    desired_outcome TEXT NOT NULL DEFAULT '',
    existing_systems TEXT NOT NULL DEFAULT '[]',
    integration_required TEXT NOT NULL DEFAULT '',
    capability_mapping TEXT NOT NULL DEFAULT '[]',
    timeline TEXT NOT NULL DEFAULT '',
    decision_process TEXT NOT NULL DEFAULT '',
    budget_range TEXT NOT NULL DEFAULT '',
    qualification_score INTEGER,
    qualification_factors TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'discovery_in_progress',
    assigned_owner TEXT NOT NULL DEFAULT '',
    next_action TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id),
    FOREIGN KEY (lead_id) REFERENCES leads(id)
);
CREATE INDEX IF NOT EXISTS idx_inquiries_tenant_key ON inquiries(tenant_key);

-- Live Proof Sessions & Evidence
CREATE TABLE IF NOT EXISTS live_proof_sessions (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    conversation_id TEXT,
    contact_id TEXT,
    order_id TEXT,
    customer_identifier TEXT NOT NULL DEFAULT '',
    claim_type TEXT NOT NULL DEFAULT 'refund',
    session_token TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending',
    anti_spoof_nonce TEXT NOT NULL,
    max_video_seconds INTEGER NOT NULL DEFAULT 45,
    require_video INTEGER NOT NULL DEFAULT 1,
    require_photos INTEGER NOT NULL DEFAULT 1,
    expires_at TEXT NOT NULL,
    completed_at TEXT,
    rejection_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_live_proof_sessions_tenant ON live_proof_sessions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_live_proof_sessions_token ON live_proof_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_live_proof_sessions_status ON live_proof_sessions(status);

CREATE TABLE IF NOT EXISTS live_proof_evidences (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    media_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    mime_type TEXT NOT NULL,
    sha256_hash TEXT NOT NULL,
    duration_seconds REAL,
    resolution TEXT,
    device_fingerprint TEXT,
    watermark_verified INTEGER NOT NULL DEFAULT 1,
    verification_score REAL NOT NULL DEFAULT 1.0,
    captured_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES live_proof_sessions(id)
);
CREATE INDEX IF NOT EXISTS idx_live_proof_evidence_session ON live_proof_evidences(session_id);
CREATE INDEX IF NOT EXISTS idx_live_proof_evidence_hash ON live_proof_evidences(sha256_hash);

-- Contact Segments
CREATE TABLE IF NOT EXISTS contact_segments (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL DEFAULT 'newax',
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    filters TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_segments_tenant_name ON contact_segments(tenant_key, name);

-- Custom Attribute Definitions
CREATE TABLE IF NOT EXISTS custom_attribute_definitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    attribute_model TEXT NOT NULL DEFAULT 'contact_attribute',
    attribute_type TEXT NOT NULL DEFAULT 'text',
    default_value TEXT,
    options TEXT NOT NULL DEFAULT '[]',
    is_required INTEGER NOT NULL DEFAULT 0,
    regex_pattern TEXT,
    regex_cue TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_custom_attr_name_model ON custom_attribute_definitions(name, attribute_model);

-- Agent Policies & Simulations
CREATE TABLE IF NOT EXISTS agent_policies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'general',
    description TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '1.0',
    status TEXT NOT NULL DEFAULT 'active',
    process_steps TEXT NOT NULL DEFAULT '[]',
    rules TEXT NOT NULL DEFAULT '{}',
    guardrails TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_agent_policies_category ON agent_policies(category);
CREATE INDEX IF NOT EXISTS idx_agent_policies_status ON agent_policies(status);

CREATE TABLE IF NOT EXISTS simulation_scenarios (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general',
    policy_id TEXT,
    customer_persona TEXT NOT NULL DEFAULT '{}',
    expected_outcome TEXT NOT NULL DEFAULT 'resolve_autonomous',
    customer_messages TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    FOREIGN KEY (policy_id) REFERENCES agent_policies(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_sim_scenarios_category ON simulation_scenarios(category);

CREATE TABLE IF NOT EXISTS simulation_runs (
    id TEXT PRIMARY KEY,
    scenario_id TEXT NOT NULL,
    policy_id TEXT,
    status TEXT NOT NULL DEFAULT 'completed',
    verdict TEXT NOT NULL DEFAULT 'passed',
    policy_adherence_score REAL NOT NULL DEFAULT 95.0,
    empathy_score REAL NOT NULL DEFAULT 90.0,
    accuracy_score REAL NOT NULL DEFAULT 92.0,
    overall_score REAL NOT NULL DEFAULT 92.5,
    turn_count INTEGER NOT NULL DEFAULT 2,
    transcript TEXT NOT NULL DEFAULT '[]',
    evaluation_summary TEXT NOT NULL DEFAULT '',
    recommendations TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    FOREIGN KEY (scenario_id) REFERENCES simulation_scenarios(id) ON DELETE CASCADE,
    FOREIGN KEY (policy_id) REFERENCES agent_policies(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_sim_runs_verdict ON simulation_runs(verdict);

-- Self-Improving: Insights & Retraining
CREATE TABLE IF NOT EXISTS agent_insights (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'trend',
    description TEXT NOT NULL DEFAULT '',
    human_rep_id TEXT,
    human_rep_name TEXT,
    exemplar_content TEXT,
    impact_score REAL NOT NULL DEFAULT 8.0,
    estimated_resolution_lift REAL NOT NULL DEFAULT 3.5,
    suggested_action TEXT NOT NULL DEFAULT 'retrain_agent',
    action_payload TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'open',
    applied_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (human_rep_id) REFERENCES team_members(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_agent_insights_category ON agent_insights(category);
CREATE INDEX IF NOT EXISTS idx_agent_insights_status ON agent_insights(status);

CREATE TABLE IF NOT EXISTS agent_retraining_events (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    trigger_type TEXT NOT NULL DEFAULT 'human_exemplar_sync',
    version_from TEXT NOT NULL DEFAULT '2.1',
    version_to TEXT NOT NULL DEFAULT '2.2',
    insight_id TEXT,
    changes_applied TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'applied',
    baseline_resolution_rate REAL NOT NULL DEFAULT 86.4,
    post_training_resolution_rate REAL NOT NULL DEFAULT 91.2,
    resolution_lift REAL NOT NULL DEFAULT 4.8,
    created_at TEXT NOT NULL,
    FOREIGN KEY (insight_id) REFERENCES agent_insights(id) ON DELETE SET NULL
);

-- SSO Config (singleton)
CREATE TABLE IF NOT EXISTS sso_configs (
    id TEXT PRIMARY KEY,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    provider_type TEXT NOT NULL DEFAULT 'saml',
    idp_entity_id TEXT NOT NULL DEFAULT '',
    idp_sso_url TEXT NOT NULL DEFAULT '',
    idp_x509_cert TEXT NOT NULL DEFAULT '',
    sp_entity_id TEXT NOT NULL DEFAULT 'urn:communicator:sp',
    sp_acs_url TEXT NOT NULL DEFAULT '/api/v1/sso/saml/acs',
    enforced_domains TEXT NOT NULL DEFAULT '[]',
    allow_unlinked_passwords INTEGER NOT NULL DEFAULT 1,
    jit_provisioning_enabled INTEGER NOT NULL DEFAULT 1,
    default_role TEXT NOT NULL DEFAULT 'agent',
    attribute_mapping TEXT NOT NULL DEFAULT '{}',
    last_tested_at TEXT,
    last_test_status TEXT NOT NULL DEFAULT 'untested',
    last_test_message TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Stripe Config & Billing Logs
CREATE TABLE IF NOT EXISTS stripe_configs (
    id TEXT PRIMARY KEY,
    account_name TEXT NOT NULL DEFAULT 'Primary Stripe Account',
    api_key TEXT NOT NULL DEFAULT '',
    publishable_key TEXT NOT NULL DEFAULT '',
    webhook_secret TEXT NOT NULL DEFAULT '',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    test_mode INTEGER NOT NULL DEFAULT 1,
    default_currency TEXT NOT NULL DEFAULT 'usd',
    last_tested_at TEXT,
    last_test_status TEXT NOT NULL DEFAULT 'untested',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stripe_billing_action_logs (
    id TEXT PRIMARY KEY,
    contact_id TEXT,
    customer_identifier TEXT NOT NULL DEFAULT '',
    action_type TEXT NOT NULL,
    amount_cents INTEGER NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'usd',
    status TEXT NOT NULL DEFAULT 'succeeded',
    stripe_id TEXT NOT NULL DEFAULT '',
    actor_email TEXT NOT NULL DEFAULT 'agent@newax.com',
    reason TEXT NOT NULL DEFAULT '',
    details TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_stripe_logs_customer ON stripe_billing_action_logs(customer_identifier);

-- Translation Settings (singleton)
CREATE TABLE IF NOT EXISTS translation_settings (
    id TEXT PRIMARY KEY,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    auto_translate_inbound INTEGER NOT NULL DEFAULT 1,
    auto_translate_outbound INTEGER NOT NULL DEFAULT 1,
    agent_primary_language TEXT NOT NULL DEFAULT 'en',
    preferred_engine TEXT NOT NULL DEFAULT 'ai_gateway',
    confidence_threshold REAL NOT NULL DEFAULT 0.6,
    enabled_languages TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Webhook Endpoints & Delivery Logs
CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events TEXT NOT NULL DEFAULT '[]',
    headers TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    retry_count INTEGER NOT NULL DEFAULT 3,
    timeout_seconds INTEGER NOT NULL DEFAULT 10,
    total_delivered INTEGER NOT NULL DEFAULT 0,
    total_failed INTEGER NOT NULL DEFAULT 0,
    last_status_code INTEGER,
    last_delivered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS webhook_delivery_logs (
    id TEXT PRIMARY KEY,
    endpoint_id TEXT NOT NULL,
    event TEXT NOT NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    status_code INTEGER NOT NULL DEFAULT 0,
    response_headers TEXT NOT NULL DEFAULT '{}',
    response_body TEXT NOT NULL DEFAULT '',
    duration_ms INTEGER NOT NULL DEFAULT 0,
    attempt INTEGER NOT NULL DEFAULT 1,
    success INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (endpoint_id) REFERENCES webhook_endpoints(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_webhook_logs_endpoint ON webhook_delivery_logs(endpoint_id);
CREATE INDEX IF NOT EXISTS idx_webhook_logs_event ON webhook_delivery_logs(event);

-- Issue Tracker Configs
CREATE TABLE IF NOT EXISTS issue_tracker_configs (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL UNIQUE,
    account_name TEXT NOT NULL DEFAULT '',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    test_mode INTEGER NOT NULL DEFAULT 1,
    credentials TEXT NOT NULL DEFAULT '{}',
    config TEXT NOT NULL DEFAULT '{}',
    last_tested_at TEXT,
    last_test_status TEXT NOT NULL DEFAULT 'untested',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Linked Issues
CREATE TABLE IF NOT EXISTS linked_issues (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    ticket_id TEXT,
    provider TEXT NOT NULL,
    external_issue_id TEXT NOT NULL DEFAULT '',
    issue_key TEXT NOT NULL,
    issue_url TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Open',
    priority TEXT NOT NULL DEFAULT 'medium',
    assigned_to TEXT NOT NULL DEFAULT '',
    created_by_email TEXT NOT NULL DEFAULT 'agent@newax.com',
    sync_status TEXT NOT NULL DEFAULT 'synced',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_linked_issues_key ON linked_issues(issue_key);

-- Slack Conversation Syncs
CREATE TABLE IF NOT EXISTS slack_conversation_syncs (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL UNIQUE,
    slack_channel_id TEXT NOT NULL,
    slack_channel_name TEXT NOT NULL DEFAULT '#customer-support',
    slack_thread_ts TEXT NOT NULL,
    is_synced INTEGER NOT NULL DEFAULT 1,
    last_synced_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- Crypto Mesh Outbox (for encrypted domain events)
CREATE TABLE IF NOT EXISTS crypto_mesh_outbox (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    encrypted_payload TEXT NOT NULL,
    recipient TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    sent_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_crypto_mesh_outbox_status ON crypto_mesh_outbox(status);

-- Scheduler Jobs
CREATE TABLE IF NOT EXISTS scheduler_jobs (
    key TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cron_expression TEXT NOT NULL DEFAULT '0 * * * *',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- WhatsApp Pricing Rates (operator-entered Meta conversation rates)
CREATE TABLE IF NOT EXISTS whatsapp_pricing_rates (
    region TEXT NOT NULL,
    category TEXT NOT NULL,
    currency TEXT NOT NULL,
    rate_per_conversation REAL NOT NULL,
    source TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL,
    PRIMARY KEY (region, category)
);

-- Widget Sessions (visitor cursor, presence, close state)
CREATE TABLE IF NOT EXISTS widget_sessions (
    id TEXT PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    conversation_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    inbox_id TEXT NOT NULL,
    website_token TEXT,
    origin TEXT,
    client_ip TEXT,
    state TEXT NOT NULL DEFAULT 'open',
    last_cursor INTEGER NOT NULL DEFAULT 0,
    visitor_presence TEXT NOT NULL DEFAULT 'unknown',
    last_presence_at TEXT,
    last_typing_at TEXT,
    closed_at TEXT,
    close_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_widget_sessions_conversation ON widget_sessions(conversation_id, state);

-- Agent Presence (typing/presence signals)
CREATE TABLE IF NOT EXISTS agent_presence (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    agent_id TEXT,
    agent_name TEXT NOT NULL,
    agent_avatar TEXT,
    action TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_presence_key ON agent_presence(conversation_id, agent_name);

-- Ecosystem Events (cross-repo durable events)
CREATE TABLE IF NOT EXISTS ecosystem_events (
    id TEXT PRIMARY KEY,
    event_name TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT '',
    tenant_id TEXT NOT NULL DEFAULT '',
    payload TEXT NOT NULL DEFAULT '{}',
    outcome TEXT NOT NULL DEFAULT '',
    received_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ecosystem_events_event_name ON ecosystem_events(event_name);

-- Ecosystem Egress Outbox (outbound mesh events pending relay acceptance)
CREATE TABLE IF NOT EXISTS ecosystem_egress_outbox (
    id TEXT PRIMARY KEY,
    event_name TEXT NOT NULL,
    recipient_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL DEFAULT '',
    organization_id TEXT,
    correlation_id TEXT NOT NULL DEFAULT '',
    envelope TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    sent_at TEXT
);

-- Additional indexes from db/mod.rs
CREATE INDEX IF NOT EXISTS ix_consent_records_contact_id ON consent_records (contact_id);
CREATE INDEX IF NOT EXISTS ix_messages_provider_message_id ON messages (provider_message_id);
CREATE INDEX IF NOT EXISTS ix_inboxes_channel_type ON inboxes (channel_type);
CREATE INDEX IF NOT EXISTS ix_contacts_channel_value ON contacts (channel, value);
CREATE INDEX IF NOT EXISTS ix_live_proof_sessions_tenant_id ON live_proof_sessions (tenant_id);
CREATE INDEX IF NOT EXISTS ix_live_proof_evidences_session_id ON live_proof_evidences (session_id);
CREATE INDEX IF NOT EXISTS ix_live_proof_evidences_sha256 ON live_proof_evidences (sha256_hash);
CREATE INDEX IF NOT EXISTS ix_team_members_email ON team_members (email);
CREATE INDEX IF NOT EXISTS ix_team_members_name ON team_members (name);
CREATE INDEX IF NOT EXISTS ix_team_members_role ON team_members (role);
CREATE INDEX IF NOT EXISTS ix_team_memberships_team_member_id ON team_memberships (member_id);
CREATE INDEX IF NOT EXISTS ix_contact_notes_contact_id ON contact_notes (contact_id);
CREATE INDEX IF NOT EXISTS ix_contact_memories_contact_id ON contact_memories (contact_id);
CREATE INDEX IF NOT EXISTS ix_drafts_conversation_id ON drafts (conversation_id);
CREATE INDEX IF NOT EXISTS ix_drafts_status ON drafts (status);
CREATE INDEX IF NOT EXISTS ix_inquiries_tenant_key ON inquiries (tenant_key);
CREATE INDEX IF NOT EXISTS ix_inquiries_contact_id ON inquiries (contact_id);
CREATE INDEX IF NOT EXISTS ix_inquiries_status ON inquiries (status);
CREATE INDEX IF NOT EXISTS ix_knowledge_base_entries_topic ON knowledge_base_entries (topic);
CREATE INDEX IF NOT EXISTS ix_knowledge_base_entries_status ON knowledge_base_entries (status);
CREATE INDEX IF NOT EXISTS ix_knowledge_gaps_status ON knowledge_gaps (status);
CREATE INDEX IF NOT EXISTS ix_knowledge_recommendations_status ON knowledge_recommendations (status, gap_type);
CREATE INDEX IF NOT EXISTS ix_contacts_customer_id ON contacts (customer_id);
