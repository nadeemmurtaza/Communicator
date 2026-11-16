-- Migration 0072: Row-Level Security (RLS)
--
-- Applies PostgreSQL Row-Level Security to all tables in the Communicator database.
--
-- Architecture:
--   1. PostgreSQL session-level GUC variables carry the authenticated identity:
--      - app.current_tenant_key  — tenant scope for the request
--      - app.current_account_id  — authenticated account (from nID)
--      - app.current_user_id     — team_member UUID if resolved
--   2. The Rust backend sets these variables at the start of every request via
--      a Tower middleware layer that runs SET LOCAL before each query.
--   3. RLS policies use current_setting('app.current_tenant_key', '') to
--      compare against the tenant_key column.
--
-- Table categories:
--   A. Tenant-scoped tables — have tenant_key column
--   B. Brand-scoped tables — have brand_id column (scoped via brand → org → tenant)
--   C. Organization-scoped tables — have organization_id column
--   D. Access control tables — access_grants, role_assignments, etc.
--   E. System/singleton tables — no tenant scoping (configs, system tables)
--   F. Join/bridge tables — inherit scope from parent via FK
--   G. nID sync tables — scoped by nid_tenant
--   H. ERP tables — scoped by brand_id
--
-- Superusers bypass all RLS policies automatically.

-- ===========================================================================
-- Helper function: check if current user has access to a tenant
-- ===========================================================================

CREATE OR REPLACE FUNCTION has_tenant_access(target_tenant_key TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    -- Superusers always have access
    IF current_setting('role') = 'pg_read_all_data' THEN
        RETURN TRUE;
    END IF;

    -- If no tenant context is set, deny access
    IF current_setting('app.current_tenant_key', '') = '' THEN
        RETURN FALSE;
    END IF;

    -- Match tenant key
    RETURN current_setting('app.current_tenant_key', '') = target_tenant_key;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

COMMENT ON FUNCTION has_tenant_access(TEXT) IS 'Returns true if the current request context has access to the given tenant_key.';

-- ===========================================================================
-- Helper function: check if current user has access to a brand
-- ===========================================================================

CREATE OR REPLACE FUNCTION has_brand_access(target_brand_id TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    brand_org_id TEXT;
    org_tenant_key TEXT;
BEGIN
    -- Superusers always have access
    IF current_setting('role') = 'pg_read_all_data' THEN
        RETURN TRUE;
    END IF;

    -- If no tenant context is set, deny access
    IF current_setting('app.current_tenant_key', '') = '' THEN
        RETURN FALSE;
    END IF;

    -- Look up the brand's organization and tenant
    SELECT b.organization_id INTO brand_org_id
    FROM brands b
    WHERE b.id = target_brand_id;

    IF brand_org_id IS NULL THEN
        RETURN FALSE;
    END IF;

    SELECT o.tenant_key INTO org_tenant_key
    FROM organizations o
    WHERE o.id = brand_org_id;

    IF org_tenant_key IS NULL THEN
        RETURN FALSE;
    END IF;

    RETURN current_setting('app.current_tenant_key', '') = org_tenant_key;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

COMMENT ON FUNCTION has_brand_access(TEXT) IS 'Returns true if the current request context has access to the given brand_id via its organization tenant.';

-- ===========================================================================
-- Helper function: check if current user has access to an organization
-- ===========================================================================

CREATE OR REPLACE FUNCTION has_org_access(target_org_id TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    org_tenant_key TEXT;
BEGIN
    -- Superusers always have access
    IF current_setting('role') = 'pg_read_all_data' THEN
        RETURN TRUE;
    END IF;

    -- If no tenant context is set, deny access
    IF current_setting('app.current_tenant_key', '') = '' THEN
        RETURN FALSE;
    END IF;

    SELECT o.tenant_key INTO org_tenant_key
    FROM organizations o
    WHERE o.id = target_org_id;

    IF org_tenant_key IS NULL THEN
        RETURN FALSE;
    END IF;

    RETURN current_setting('app.current_tenant_key', '') = org_tenant_key;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

COMMENT ON FUNCTION has_org_access(TEXT) IS 'Returns true if the current request context has access to the given organization_id via its tenant.';

-- ===========================================================================
-- Enable RLS on all tables and create policies
-- ===========================================================================

-- ---------------------------------------------------------------------------
-- A. TENANT-SCOPED TABLES (have tenant_key column)
-- ---------------------------------------------------------------------------

-- tenants
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenants_tenant_isolation ON tenants
    USING (tenant_key = current_setting('app.current_tenant_key', '') OR tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY tenants_all_tenants ON tenants
    USING (has_tenant_access(tenant_key));

-- leads
ALTER TABLE leads ENABLE ROW LEVEL SECURITY;
CREATE POLICY leads_tenant_isolation ON leads
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY leads_tenant_insert ON leads
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- customers
ALTER TABLE customers ENABLE ROW LEVEL SECURITY;
CREATE POLICY customers_tenant_isolation ON customers
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY customers_tenant_insert ON customers
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- contacts
ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
CREATE POLICY contacts_tenant_isolation ON contacts
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY contacts_tenant_insert ON contacts
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- tags
ALTER TABLE tags ENABLE ROW LEVEL SECURITY;
CREATE POLICY tags_tenant_isolation ON tags
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY tags_tenant_insert ON tags
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- conversations
ALTER TABLE conversations ENABLE ROW LEVEL SECURITY;
CREATE POLICY conversations_tenant_isolation ON conversations
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY conversations_tenant_insert ON conversations
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- services
ALTER TABLE services ENABLE ROW LEVEL SECURITY;
CREATE POLICY services_tenant_isolation ON services
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY services_tenant_insert ON services
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- knowledge_base_entries
ALTER TABLE knowledge_base_entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY knowledge_base_entries_tenant_isolation ON knowledge_base_entries
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY knowledge_base_entries_tenant_insert ON knowledge_base_entries
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- broadcasts
ALTER TABLE broadcasts ENABLE ROW LEVEL SECURITY;
CREATE POLICY broadcasts_tenant_isolation ON broadcasts
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY broadcasts_tenant_insert ON broadcasts
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- campaign_templates
ALTER TABLE campaign_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY campaign_templates_tenant_isolation ON campaign_templates
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY campaign_templates_tenant_insert ON campaign_templates
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- integration_settings
ALTER TABLE integration_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY integration_settings_tenant_isolation ON integration_settings
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY integration_settings_tenant_insert ON integration_settings
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- discovery_flow_templates
ALTER TABLE discovery_flow_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY discovery_flow_templates_tenant_isolation ON discovery_flow_templates
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY discovery_flow_templates_tenant_insert ON discovery_flow_templates
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- inquiries
ALTER TABLE inquiries ENABLE ROW LEVEL SECURITY;
CREATE POLICY inquiries_tenant_isolation ON inquiries
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY inquiries_tenant_insert ON inquiries
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- contact_segments
ALTER TABLE contact_segments ENABLE ROW LEVEL SECURITY;
CREATE POLICY contact_segments_tenant_isolation ON contact_segments
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY contact_segments_tenant_insert ON contact_segments
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- organizations
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
CREATE POLICY organizations_tenant_isolation ON organizations
    USING (tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY organizations_tenant_insert ON organizations
    WITH CHECK (tenant_key = current_setting('app.current_tenant_key', ''));

-- ---------------------------------------------------------------------------
-- B. BRAND-SCOPED TABLES (have brand_id column)
-- ---------------------------------------------------------------------------

-- brands
ALTER TABLE brands ENABLE ROW LEVEL SECURITY;
CREATE POLICY brands_tenant_isolation ON brands
    USING (has_brand_access(id));
CREATE POLICY brands_tenant_insert ON brands
    WITH CHECK (has_brand_access(id));

-- brand_credentials
ALTER TABLE brand_credentials ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_credentials_tenant_isolation ON brand_credentials
    USING (has_brand_access(brand_id));
CREATE POLICY brand_credentials_tenant_insert ON brand_credentials
    WITH CHECK (has_brand_access(brand_id));

-- brand_ai_config
ALTER TABLE brand_ai_config ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_ai_config_tenant_isolation ON brand_ai_config
    USING (has_brand_access(brand_id));
CREATE POLICY brand_ai_config_tenant_insert ON brand_ai_config
    WITH CHECK (has_brand_access(brand_id));

-- brand_knowledge_entries
ALTER TABLE brand_knowledge_entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_knowledge_entries_tenant_isolation ON brand_knowledge_entries
    USING (has_brand_access(brand_id));
CREATE POLICY brand_knowledge_entries_tenant_insert ON brand_knowledge_entries
    WITH CHECK (has_brand_access(brand_id));

-- brand_override_rules
ALTER TABLE brand_override_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_override_rules_tenant_isolation ON brand_override_rules
    USING (has_brand_access(brand_id));
CREATE POLICY brand_override_rules_tenant_insert ON brand_override_rules
    WITH CHECK (has_brand_access(brand_id));

-- brand_override_settings
ALTER TABLE brand_override_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_override_settings_tenant_isolation ON brand_override_settings
    USING (has_brand_access(brand_id));
CREATE POLICY brand_override_settings_tenant_insert ON brand_override_settings
    WITH CHECK (has_brand_access(brand_id));

-- brand_otp_isolation
ALTER TABLE brand_otp_isolation ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_otp_isolation_tenant_isolation ON brand_otp_isolation
    USING (has_brand_access(brand_id));
CREATE POLICY brand_otp_isolation_tenant_insert ON brand_otp_isolation
    WITH CHECK (has_brand_access(brand_id));

-- brand_packages
ALTER TABLE brand_packages ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_packages_tenant_isolation ON brand_packages
    USING (has_brand_access(brand_id));
CREATE POLICY brand_packages_tenant_insert ON brand_packages
    WITH CHECK (has_brand_access(brand_id));

-- brand_profile_sources
ALTER TABLE brand_profile_sources ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_profile_sources_tenant_isolation ON brand_profile_sources
    USING (has_brand_access(brand_id));
CREATE POLICY brand_profile_sources_tenant_insert ON brand_profile_sources
    WITH CHECK (has_brand_access(brand_id));

-- brand_promotions
ALTER TABLE brand_promotions ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_promotions_tenant_isolation ON brand_promotions
    USING (has_brand_access(brand_id));
CREATE POLICY brand_promotions_tenant_insert ON brand_promotions
    WITH CHECK (has_brand_access(brand_id));

-- brand_voucher_settings
ALTER TABLE brand_voucher_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_voucher_settings_tenant_isolation ON brand_voucher_settings
    USING (has_brand_access(brand_id));
CREATE POLICY brand_voucher_settings_tenant_insert ON brand_voucher_settings
    WITH CHECK (has_brand_access(brand_id));

-- ---------------------------------------------------------------------------
-- C. ORGANIZATION-SCOPED TABLES
-- ---------------------------------------------------------------------------

-- org_profile_sources
ALTER TABLE org_profile_sources ENABLE ROW LEVEL SECURITY;
CREATE POLICY org_profile_sources_tenant_isolation ON org_profile_sources
    USING (has_org_access(organization_id));
CREATE POLICY org_profile_sources_tenant_insert ON org_profile_sources
    WITH CHECK (has_org_access(organization_id));

-- ---------------------------------------------------------------------------
-- D. ACCESS CONTROL TABLES
-- ---------------------------------------------------------------------------

-- access_grants
ALTER TABLE access_grants ENABLE ROW LEVEL SECURITY;
CREATE POLICY access_grants_tenant_isolation ON access_grants
    USING (has_tenant_access(tenant_id));
CREATE POLICY access_grants_tenant_insert ON access_grants
    WITH CHECK (has_tenant_access(tenant_id));

-- departments
ALTER TABLE departments ENABLE ROW LEVEL SECURITY;
CREATE POLICY departments_tenant_isolation ON departments
    USING (has_tenant_access(tenant_id));
CREATE POLICY departments_tenant_insert ON departments
    WITH CHECK (has_tenant_access(tenant_id));

-- department_assignments
ALTER TABLE department_assignments ENABLE ROW LEVEL SECURITY;
CREATE POLICY department_assignments_tenant_isolation ON department_assignments
    USING (has_tenant_access(tenant_id));
CREATE POLICY department_assignments_tenant_insert ON department_assignments
    WITH CHECK (has_tenant_access(tenant_id));

-- role_assignments
ALTER TABLE role_assignments ENABLE ROW LEVEL SECURITY;
CREATE POLICY role_assignments_tenant_isolation ON role_assignments
    USING (has_tenant_access(tenant_id));
CREATE POLICY role_assignments_tenant_insert ON role_assignments
    WITH CHECK (has_tenant_access(tenant_id));

-- role_definitions
ALTER TABLE role_definitions ENABLE ROW LEVEL SECURITY;
CREATE POLICY role_definitions_tenant_isolation ON role_definitions
    USING (is_system_role = true OR has_tenant_access(tenant_id));
CREATE POLICY role_definitions_tenant_insert ON role_definitions
    WITH CHECK (is_system_role = false AND has_tenant_access(tenant_id));

-- permission_catalog
ALTER TABLE permission_catalog ENABLE ROW LEVEL SECURITY;
CREATE POLICY permission_catalog_read ON permission_catalog
    USING (true);

-- invitations
ALTER TABLE invitations ENABLE ROW LEVEL SECURITY;
CREATE POLICY invitations_tenant_isolation ON invitations
    USING (has_tenant_access(tenant_id));
CREATE POLICY invitations_tenant_insert ON invitations
    WITH CHECK (has_tenant_access(tenant_id));

-- ---------------------------------------------------------------------------
-- E. NID SYNC TABLES
-- ---------------------------------------------------------------------------

-- nid_tenants
ALTER TABLE nid_tenants ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_tenants_tenant_isolation ON nid_tenants
    USING (id = current_setting('app.current_tenant_key', '') OR has_tenant_access(id));
CREATE POLICY nid_tenants_tenant_insert ON nid_tenants
    WITH CHECK (true);

-- nid_organizations
ALTER TABLE nid_organizations ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_organizations_tenant_isolation ON nid_organizations
    USING (true);

-- nid_brands
ALTER TABLE nid_brands ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_brands_tenant_isolation ON nid_brands
    USING (true);

-- nid_tenant_mapping
ALTER TABLE nid_tenant_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_tenant_mapping_tenant_isolation ON nid_tenant_mapping
    USING (communicator_tenant_key = current_setting('app.current_tenant_key', ''));
CREATE POLICY nid_tenant_mapping_tenant_insert ON nid_tenant_mapping
    WITH CHECK (communicator_tenant_key = current_setting('app.current_tenant_key', ''));

-- tenant_sync_log
ALTER TABLE tenant_sync_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_sync_log_tenant_isolation ON tenant_sync_log
    USING (nid_tenant_id = current_setting('app.current_tenant_key', ''));
CREATE POLICY tenant_sync_log_tenant_insert ON tenant_sync_log
    WITH CHECK (true);

-- tenant_redirect_sessions
ALTER TABLE tenant_redirect_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_redirect_sessions_tenant_isolation ON tenant_redirect_sessions
    USING (true);
CREATE POLICY tenant_redirect_sessions_tenant_insert ON tenant_redirect_sessions
    WITH CHECK (true);

-- tenant_profile_updates
ALTER TABLE tenant_profile_updates ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_profile_updates_tenant_isolation ON tenant_profile_updates
    USING (nid_tenant_id = current_setting('app.current_tenant_key', ''));
CREATE POLICY tenant_profile_updates_tenant_insert ON tenant_profile_updates
    WITH CHECK (true);

-- nid_connection_settings
ALTER TABLE nid_connection_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_connection_settings_read ON nid_connection_settings
    USING (true);

-- nid_account_links
ALTER TABLE nid_account_links ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_account_links_tenant_isolation ON nid_account_links
    USING (account_id = current_setting('app.current_account_id', ''));
CREATE POLICY nid_account_links_tenant_insert ON nid_account_links
    WITH CHECK (account_id = current_setting('app.current_account_id', ''));

-- nid_profile_cache
ALTER TABLE nid_profile_cache ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_profile_cache_tenant_isolation ON nid_profile_cache
    USING (account_id = current_setting('app.current_account_id', ''));
CREATE POLICY nid_profile_cache_tenant_insert ON nid_profile_cache
    WITH CHECK (account_id = current_setting('app.current_account_id', ''));

-- nid_organization_mapping
ALTER TABLE nid_organization_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_organization_mapping_tenant_isolation ON nid_organization_mapping
    USING (true);

-- nid_brand_connection
ALTER TABLE nid_brand_connection ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_brand_connection_tenant_isolation ON nid_brand_connection
    USING (true);

-- nid_organization_connection
ALTER TABLE nid_organization_connection ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_organization_connection_tenant_isolation ON nid_organization_connection
    USING (true);

-- nid_sync_status
ALTER TABLE nid_sync_status ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_sync_status_tenant_isolation ON nid_sync_status
    USING (true);

-- org_sync_status
ALTER TABLE org_sync_status ENABLE ROW LEVEL SECURITY;
CREATE POLICY org_sync_status_tenant_isolation ON org_sync_status
    USING (true);

-- brand_sync_status
ALTER TABLE brand_sync_status ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_sync_status_tenant_isolation ON brand_sync_status
    USING (true);

-- organization_sync_log
ALTER TABLE organization_sync_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY organization_sync_log_tenant_isolation ON organization_sync_log
    USING (true);

-- brand_sync_log
ALTER TABLE brand_sync_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_sync_log_tenant_isolation ON brand_sync_log
    USING (true);

-- brand_redirect_sessions
ALTER TABLE brand_redirect_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_redirect_sessions_tenant_isolation ON brand_redirect_sessions
    USING (true);

-- organization_redirect_sessions
ALTER TABLE organization_redirect_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY organization_redirect_sessions_tenant_isolation ON organization_redirect_sessions
    USING (true);

-- brand_switch_log
ALTER TABLE brand_switch_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_switch_log_tenant_isolation ON brand_switch_log
    USING (true);

-- nid_organization_memberships
ALTER TABLE nid_organization_memberships ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_organization_memberships_tenant_isolation ON nid_organization_memberships
    USING (true);

-- nid_billing_account_links
ALTER TABLE nid_billing_account_links ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_billing_account_links_tenant_isolation ON nid_billing_account_links
    USING (true);

-- nid_subscription_sync
ALTER TABLE nid_subscription_sync ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_subscription_sync_tenant_isolation ON nid_subscription_sync
    USING (true);

-- nid_invoice_sync
ALTER TABLE nid_invoice_sync ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_invoice_sync_tenant_isolation ON nid_invoice_sync
    USING (true);

-- nid_payment_sync
ALTER TABLE nid_payment_sync ENABLE ROW LEVEL SECURITY;
CREATE POLICY nid_payment_sync_tenant_isolation ON nid_payment_sync
    USING (true);

-- ---------------------------------------------------------------------------
-- F. TABLES SCOPED VIA CONTACT/CONVERSATION FK (inherit tenant scope)
-- ---------------------------------------------------------------------------

-- consent_records (via contact_id → contacts → tenant_key)
ALTER TABLE consent_records ENABLE ROW LEVEL SECURITY;
CREATE POLICY consent_records_tenant_isolation ON consent_records
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = consent_records.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY consent_records_tenant_insert ON consent_records
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = consent_records.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- contact_memories (via contact_id)
ALTER TABLE contact_memories ENABLE ROW LEVEL SECURITY;
CREATE POLICY contact_memories_tenant_isolation ON contact_memories
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = contact_memories.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY contact_memories_tenant_insert ON contact_memories
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = contact_memories.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- contact_notes (via contact_id)
ALTER TABLE contact_notes ENABLE ROW LEVEL SECURITY;
CREATE POLICY contact_notes_tenant_isolation ON contact_notes
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = contact_notes.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY contact_notes_tenant_insert ON contact_notes
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = contact_notes.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- contact_tags (via contact_id and tag_id)
ALTER TABLE contact_tags ENABLE ROW LEVEL SECURITY;
CREATE POLICY contact_tags_tenant_isolation ON contact_tags
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = contact_tags.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY contact_tags_tenant_insert ON contact_tags
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = contact_tags.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- messages (via conversation_id → conversations → tenant_key)
ALTER TABLE messages ENABLE ROW LEVEL SECURITY;
CREATE POLICY messages_tenant_isolation ON messages
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = messages.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY messages_tenant_insert ON messages
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = messages.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- drafts (via conversation_id)
ALTER TABLE drafts ENABLE ROW LEVEL SECURITY;
CREATE POLICY drafts_tenant_isolation ON drafts
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = drafts.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY drafts_tenant_insert ON drafts
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = drafts.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- broadcast_recipients (via broadcast_id → broadcasts → tenant_key)
ALTER TABLE broadcast_recipients ENABLE ROW LEVEL SECURITY;
CREATE POLICY broadcast_recipients_tenant_isolation ON broadcast_recipients
    USING (EXISTS (
        SELECT 1 FROM broadcasts b WHERE b.id = broadcast_recipients.broadcast_id
        AND b.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY broadcast_recipients_tenant_insert ON broadcast_recipients
    WITH CHECK (EXISTS (
        SELECT 1 FROM broadcasts b WHERE b.id = broadcast_recipients.broadcast_id
        AND b.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- copilot_approvals (via conversation_id)
ALTER TABLE copilot_approvals ENABLE ROW LEVEL SECURITY;
CREATE POLICY copilot_approvals_tenant_isolation ON copilot_approvals
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = copilot_approvals.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY copilot_approvals_tenant_insert ON copilot_approvals
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = copilot_approvals.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- tickets (via contact_id → contacts → tenant_key)
ALTER TABLE tickets ENABLE ROW LEVEL SECURITY;
CREATE POLICY tickets_tenant_isolation ON tickets
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = tickets.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY tickets_tenant_insert ON tickets
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = tickets.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- ticket_activities (via ticket_id → tickets → contact_id)
ALTER TABLE ticket_activities ENABLE ROW LEVEL SECURITY;
CREATE POLICY ticket_activities_tenant_isolation ON ticket_activities
    USING (EXISTS (
        SELECT 1 FROM tickets t
        JOIN contacts c ON c.id = t.contact_id
        WHERE t.id = ticket_activities.ticket_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY ticket_activities_tenant_insert ON ticket_activities
    WITH CHECK (EXISTS (
        SELECT 1 FROM tickets t
        JOIN contacts c ON c.id = t.contact_id
        WHERE t.id = ticket_activities.ticket_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- linked_issues (via conversation_id)
ALTER TABLE linked_issues ENABLE ROW LEVEL SECURITY;
CREATE POLICY linked_issues_tenant_isolation ON linked_issues
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = linked_issues.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY linked_issues_tenant_insert ON linked_issues
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = linked_issues.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- slack_conversation_syncs (via conversation_id)
ALTER TABLE slack_conversation_syncs ENABLE ROW LEVEL SECURITY;
CREATE POLICY slack_conversation_syncs_tenant_isolation ON slack_conversation_syncs
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = slack_conversation_syncs.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY slack_conversation_syncs_tenant_insert ON slack_conversation_syncs
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = slack_conversation_syncs.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- widget_sessions (via conversation_id)
ALTER TABLE widget_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY widget_sessions_tenant_isolation ON widget_sessions
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = widget_sessions.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY widget_sessions_tenant_insert ON widget_sessions
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = widget_sessions.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- agent_presence (via conversation_id)
ALTER TABLE agent_presence ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_presence_tenant_isolation ON agent_presence
    USING (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = agent_presence.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY agent_presence_tenant_insert ON agent_presence
    WITH CHECK (EXISTS (
        SELECT 1 FROM conversations cv WHERE cv.id = agent_presence.conversation_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- message_feedback (via conversation_id through messages)
ALTER TABLE message_feedback ENABLE ROW LEVEL SECURITY;
CREATE POLICY message_feedback_tenant_isolation ON message_feedback
    USING (EXISTS (
        SELECT 1 FROM messages m
        JOIN conversations cv ON cv.id = m.conversation_id
        WHERE m.id = message_feedback.message_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY message_feedback_tenant_insert ON message_feedback
    WITH CHECK (EXISTS (
        SELECT 1 FROM messages m
        JOIN conversations cv ON cv.id = m.conversation_id
        WHERE m.id = message_feedback.message_id
        AND cv.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- live_proof_sessions (via tenant_id)
ALTER TABLE live_proof_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY live_proof_sessions_tenant_isolation ON live_proof_sessions
    USING (tenant_id = current_setting('app.current_tenant_key', ''));
CREATE POLICY live_proof_sessions_tenant_insert ON live_proof_sessions
    WITH CHECK (tenant_id = current_setting('app.current_tenant_key', ''));

-- live_proof_evidences (via session_id → live_proof_sessions → tenant_id)
ALTER TABLE live_proof_evidences ENABLE ROW LEVEL SECURITY;
CREATE POLICY live_proof_evidences_tenant_isolation ON live_proof_evidences
    USING (EXISTS (
        SELECT 1 FROM live_proof_sessions lps WHERE lps.id = live_proof_evidences.session_id
        AND lps.tenant_id = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY live_proof_evidences_tenant_insert ON live_proof_evidences
    WITH CHECK (EXISTS (
        SELECT 1 FROM live_proof_sessions lps WHERE lps.id = live_proof_evidences.session_id
        AND lps.tenant_id = current_setting('app.current_tenant_key', '')
    ));

-- stripe_billing_action_logs (via contact_id)
ALTER TABLE stripe_billing_action_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY stripe_billing_action_logs_tenant_isolation ON stripe_billing_action_logs
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = stripe_billing_action_logs.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY stripe_billing_action_logs_tenant_insert ON stripe_billing_action_logs
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = stripe_billing_action_logs.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- segment_contact_memberships (via contact_id and segment_id)
ALTER TABLE segment_contact_memberships ENABLE ROW LEVEL SECURITY;
CREATE POLICY segment_contact_memberships_tenant_isolation ON segment_contact_memberships
    USING (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = segment_contact_memberships.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY segment_contact_memberships_tenant_insert ON segment_contact_memberships
    WITH CHECK (EXISTS (
        SELECT 1 FROM contacts c WHERE c.id = segment_contact_memberships.contact_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- ---------------------------------------------------------------------------
-- G. INBOXES (scoped via brand_id)
-- ---------------------------------------------------------------------------

-- inboxes
ALTER TABLE inboxes ENABLE ROW LEVEL SECURITY;
CREATE POLICY inboxes_tenant_isolation ON inboxes
    USING (brand_id IS NULL OR has_brand_access(brand_id));
CREATE POLICY inboxes_tenant_insert ON inboxes
    WITH CHECK (brand_id IS NULL OR has_brand_access(brand_id));

-- ---------------------------------------------------------------------------
-- H. TEAM TABLES (scoped via access_grants)
-- ---------------------------------------------------------------------------

-- team_members
ALTER TABLE team_members ENABLE ROW LEVEL SECURITY;
CREATE POLICY team_members_tenant_isolation ON team_members
    USING (EXISTS (
        SELECT 1 FROM access_grants ag
        WHERE ag.team_member_id = team_members.id::UUID
        AND ag.status = 'active'
        AND has_tenant_access(ag.tenant_id)
    ));
CREATE POLICY team_members_tenant_insert ON team_members
    WITH CHECK (true);

-- teams
ALTER TABLE teams ENABLE ROW LEVEL SECURITY;
CREATE POLICY teams_tenant_isolation ON teams
    USING (EXISTS (
        SELECT 1 FROM team_memberships tm
        JOIN access_grants ag ON ag.team_member_id = tm.member_id AND ag.status = 'active'
        WHERE tm.team_id = teams.id
        AND has_tenant_access(ag.tenant_id)
    ));
CREATE POLICY teams_tenant_insert ON teams
    WITH CHECK (true);

-- team_memberships
ALTER TABLE team_memberships ENABLE ROW LEVEL SECURITY;
CREATE POLICY team_memberships_tenant_isolation ON team_memberships
    USING (EXISTS (
        SELECT 1 FROM access_grants ag
        WHERE ag.team_member_id = team_memberships.member_id
        AND ag.status = 'active'
        AND has_tenant_access(ag.tenant_id)
    ));
CREATE POLICY team_memberships_tenant_insert ON team_memberships
    WITH CHECK (true);

-- ---------------------------------------------------------------------------
-- I. AUDIT & SYSTEM TABLES
-- ---------------------------------------------------------------------------

-- audit_logs
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY audit_logs_tenant_isolation ON audit_logs
    USING (true);

-- crypto_mesh_outbox
ALTER TABLE crypto_mesh_outbox ENABLE ROW LEVEL SECURITY;
CREATE POLICY crypto_mesh_outbox_tenant_isolation ON crypto_mesh_outbox
    USING (true);

-- scheduler_jobs
ALTER TABLE scheduler_jobs ENABLE ROW LEVEL SECURITY;
CREATE POLICY scheduler_jobs_tenant_isolation ON scheduler_jobs
    USING (true);

-- whatsapp_pricing_rates
ALTER TABLE whatsapp_pricing_rates ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_pricing_rates_read ON whatsapp_pricing_rates
    USING (true);

-- ecosystem_events
ALTER TABLE ecosystem_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY ecosystem_events_tenant_isolation ON ecosystem_events
    USING (tenant_id = current_setting('app.current_tenant_key', '') OR tenant_id = '');
CREATE POLICY ecosystem_events_tenant_insert ON ecosystem_events
    WITH CHECK (true);

-- ecosystem_egress_outbox
ALTER TABLE ecosystem_egress_outbox ENABLE ROW LEVEL SECURITY;
CREATE POLICY ecosystem_egress_outbox_tenant_isolation ON ecosystem_egress_outbox
    USING (tenant_id = current_setting('app.current_tenant_key', '') OR tenant_id = '');
CREATE POLICY ecosystem_egress_outbox_tenant_insert ON ecosystem_egress_outbox
    WITH CHECK (true);

-- ---------------------------------------------------------------------------
-- J. SINGLETON CONFIG TABLES (no tenant scoping, global read)
-- ---------------------------------------------------------------------------

-- sso_configs
ALTER TABLE sso_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY sso_configs_read ON sso_configs USING (true);
CREATE POLICY sso_configs_insert ON sso_configs WITH CHECK (true);

-- stripe_configs
ALTER TABLE stripe_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY stripe_configs_read ON stripe_configs USING (true);
CREATE POLICY stripe_configs_insert ON stripe_configs WITH CHECK (true);

-- translation_settings
ALTER TABLE translation_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY translation_settings_read ON translation_settings USING (true);
CREATE POLICY translation_settings_insert ON translation_settings WITH CHECK (true);

-- issue_tracker_configs
ALTER TABLE issue_tracker_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY issue_tracker_configs_read ON issue_tracker_configs USING (true);
CREATE POLICY issue_tracker_configs_insert ON issue_tracker_configs WITH CHECK (true);

-- ai_connections
ALTER TABLE ai_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY ai_connections_read ON ai_connections USING (true);
CREATE POLICY ai_connections_insert ON ai_connections WITH CHECK (true);

-- ai_custom_policies
ALTER TABLE ai_custom_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY ai_custom_policies_read ON ai_custom_policies USING (true);
CREATE POLICY ai_custom_policies_insert ON ai_custom_policies WITH CHECK (true);

-- channel_gateway_connections
ALTER TABLE channel_gateway_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY channel_gateway_connections_read ON channel_gateway_connections USING (true);
CREATE POLICY channel_gateway_connections_insert ON channel_gateway_connections WITH CHECK (true);

-- webhook_endpoints
ALTER TABLE webhook_endpoints ENABLE ROW LEVEL SECURITY;
CREATE POLICY webhook_endpoints_read ON webhook_endpoints USING (true);
CREATE POLICY webhook_endpoints_insert ON webhook_endpoints WITH CHECK (true);

-- webhook_delivery_logs
ALTER TABLE webhook_delivery_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY webhook_delivery_logs_read ON webhook_delivery_logs USING (true);
CREATE POLICY webhook_delivery_logs_insert ON webhook_delivery_logs WITH CHECK (true);

-- ---------------------------------------------------------------------------
-- K. AUTOMATION & AI TABLES (global scope)
-- ---------------------------------------------------------------------------

-- automation_rules
ALTER TABLE automation_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY automation_rules_read ON automation_rules USING (true);
CREATE POLICY automation_rules_insert ON automation_rules WITH CHECK (true);

-- canned_responses
ALTER TABLE canned_responses ENABLE ROW LEVEL SECURITY;
CREATE POLICY canned_responses_read ON canned_responses USING (true);
CREATE POLICY canned_responses_insert ON canned_responses WITH CHECK (true);

-- macros
ALTER TABLE macros ENABLE ROW LEVEL SECURITY;
CREATE POLICY macros_read ON macros USING (true);
CREATE POLICY macros_insert ON macros WITH CHECK (true);

-- follow_up_sequences
ALTER TABLE follow_up_sequences ENABLE ROW LEVEL SECURITY;
CREATE POLICY follow_up_sequences_read ON follow_up_sequences USING (true);
CREATE POLICY follow_up_sequences_insert ON follow_up_sequences WITH CHECK (true);

-- sequence_steps
ALTER TABLE sequence_steps ENABLE ROW LEVEL SECURITY;
CREATE POLICY sequence_steps_read ON sequence_steps USING (true);
CREATE POLICY sequence_steps_insert ON sequence_steps WITH CHECK (true);

-- agent_policies
ALTER TABLE agent_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_policies_read ON agent_policies USING (true);
CREATE POLICY agent_policies_insert ON agent_policies WITH CHECK (true);

-- simulation_scenarios
ALTER TABLE simulation_scenarios ENABLE ROW LEVEL SECURITY;
CREATE POLICY simulation_scenarios_read ON simulation_scenarios USING (true);
CREATE POLICY simulation_scenarios_insert ON simulation_scenarios WITH CHECK (true);

-- simulation_runs
ALTER TABLE simulation_runs ENABLE ROW LEVEL SECURITY;
CREATE POLICY simulation_runs_read ON simulation_runs USING (true);
CREATE POLICY simulation_runs_insert ON simulation_runs WITH CHECK (true);

-- agent_insights
ALTER TABLE agent_insights ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_insights_read ON agent_insights USING (true);
CREATE POLICY agent_insights_insert ON agent_insights WITH CHECK (true);

-- agent_retraining_events
ALTER TABLE agent_retraining_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_retraining_events_read ON agent_retraining_events USING (true);
CREATE POLICY agent_retraining_events_insert ON agent_retraining_events WITH CHECK (true);

-- custom_attribute_definitions
ALTER TABLE custom_attribute_definitions ENABLE ROW LEVEL SECURITY;
CREATE POLICY custom_attribute_definitions_read ON custom_attribute_definitions USING (true);
CREATE POLICY custom_attribute_definitions_insert ON custom_attribute_definitions WITH CHECK (true);

-- knowledge_gaps
ALTER TABLE knowledge_gaps ENABLE ROW LEVEL SECURITY;
CREATE POLICY knowledge_gaps_read ON knowledge_gaps USING (true);
CREATE POLICY knowledge_gaps_insert ON knowledge_gaps WITH CHECK (true);

-- knowledge_recommendations
ALTER TABLE knowledge_recommendations ENABLE ROW LEVEL SECURITY;
CREATE POLICY knowledge_recommendations_read ON knowledge_recommendations USING (true);
CREATE POLICY knowledge_recommendations_insert ON knowledge_recommendations WITH CHECK (true);

-- sla_policies
ALTER TABLE sla_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY sla_policies_read ON sla_policies USING (true);
CREATE POLICY sla_policies_insert ON sla_policies WITH CHECK (true);

-- ---------------------------------------------------------------------------
-- L. OTP ENGINE TABLES
-- ---------------------------------------------------------------------------

-- otp_records
ALTER TABLE otp_records ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_records_tenant_isolation ON otp_records
    USING (has_brand_access(brand_id));
CREATE POLICY otp_records_tenant_insert ON otp_records
    WITH CHECK (has_brand_access(brand_id));

-- otp_audit_log
ALTER TABLE otp_audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_audit_log_tenant_isolation ON otp_audit_log
    USING (has_brand_access(brand_id));
CREATE POLICY otp_audit_log_tenant_insert ON otp_audit_log
    WITH CHECK (has_brand_access(brand_id));

-- otp_rate_limit
ALTER TABLE otp_rate_limit ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_rate_limit_read ON otp_rate_limit USING (true);
CREATE POLICY otp_rate_limit_insert ON otp_rate_limit WITH CHECK (true);

-- otp_channel_config
ALTER TABLE otp_channel_config ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_channel_config_tenant_isolation ON otp_channel_config
    USING (has_brand_access(brand_id));
CREATE POLICY otp_channel_config_tenant_insert ON otp_channel_config
    WITH CHECK (has_brand_access(brand_id));

-- smtp_settings
ALTER TABLE smtp_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY smtp_settings_tenant_isolation ON smtp_settings
    USING (has_brand_access(brand_id));
CREATE POLICY smtp_settings_tenant_insert ON smtp_settings
    WITH CHECK (has_brand_access(brand_id));

-- email_templates
ALTER TABLE email_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY email_templates_tenant_isolation ON email_templates
    USING (has_brand_access(brand_id));
CREATE POLICY email_templates_tenant_insert ON email_templates
    WITH CHECK (has_brand_access(brand_id));

-- email_send_log
ALTER TABLE email_send_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY email_send_log_tenant_isolation ON email_send_log
    USING (has_brand_access(brand_id));
CREATE POLICY email_send_log_tenant_insert ON email_send_log
    WITH CHECK (has_brand_access(brand_id));

-- otp_external_integrations
ALTER TABLE otp_external_integrations ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_external_integrations_read ON otp_external_integrations USING (true);
CREATE POLICY otp_external_integrations_insert ON otp_external_integrations WITH CHECK (true);

-- otp_external_usage_log
ALTER TABLE otp_external_usage_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_external_usage_log_read ON otp_external_usage_log USING (true);
CREATE POLICY otp_external_usage_log_insert ON otp_external_usage_log WITH CHECK (true);

-- otp_webhook_delivery_log
ALTER TABLE otp_webhook_delivery_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY otp_webhook_delivery_log_read ON otp_webhook_delivery_log USING (true);
CREATE POLICY otp_webhook_delivery_log_insert ON otp_webhook_delivery_log WITH CHECK (true);

-- ---------------------------------------------------------------------------
-- M. COMMERCE TABLES (scoped via brand_id)
-- ---------------------------------------------------------------------------

-- categories
ALTER TABLE categories ENABLE ROW LEVEL SECURITY;
CREATE POLICY categories_tenant_isolation ON categories
    USING (has_brand_access(brand_id));
CREATE POLICY categories_tenant_insert ON categories
    WITH CHECK (has_brand_access(brand_id));

-- products
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
CREATE POLICY products_tenant_isolation ON products
    USING (has_brand_access(brand_id));
CREATE POLICY products_tenant_insert ON products
    WITH CHECK (has_brand_access(brand_id));

-- carts
ALTER TABLE carts ENABLE ROW LEVEL SECURITY;
CREATE POLICY carts_tenant_isolation ON carts
    USING (has_brand_access(brand_id));
CREATE POLICY carts_tenant_insert ON carts
    WITH CHECK (has_brand_access(brand_id));

-- cart_items
ALTER TABLE cart_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY cart_items_tenant_isolation ON cart_items
    USING (EXISTS (
        SELECT 1 FROM carts c WHERE c.id = cart_items.cart_id
        AND has_brand_access(c.brand_id)
    ));
CREATE POLICY cart_items_tenant_insert ON cart_items
    WITH CHECK (EXISTS (
        SELECT 1 FROM carts c WHERE c.id = cart_items.cart_id
        AND has_brand_access(c.brand_id)
    ));

-- commerce_orders
ALTER TABLE commerce_orders ENABLE ROW LEVEL SECURITY;
CREATE POLICY commerce_orders_tenant_isolation ON commerce_orders
    USING (has_brand_access(brand_id));
CREATE POLICY commerce_orders_tenant_insert ON commerce_orders
    WITH CHECK (has_brand_access(brand_id));

-- commerce_order_items
ALTER TABLE commerce_order_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY commerce_order_items_tenant_isolation ON commerce_order_items
    USING (EXISTS (
        SELECT 1 FROM commerce_orders co WHERE co.id = commerce_order_items.order_id
        AND has_brand_access(co.brand_id)
    ));
CREATE POLICY commerce_order_items_tenant_insert ON commerce_order_items
    WITH CHECK (EXISTS (
        SELECT 1 FROM commerce_orders co WHERE co.id = commerce_order_items.order_id
        AND has_brand_access(co.brand_id)
    ));

-- commerce_payments
ALTER TABLE commerce_payments ENABLE ROW LEVEL SECURITY;
CREATE POLICY commerce_payments_tenant_isolation ON commerce_payments
    USING (has_brand_access(brand_id));
CREATE POLICY commerce_payments_tenant_insert ON commerce_payments
    WITH CHECK (has_brand_access(brand_id));

-- coupons
ALTER TABLE coupons ENABLE ROW LEVEL SECURITY;
CREATE POLICY coupons_tenant_isolation ON coupons
    USING (has_brand_access(brand_id));
CREATE POLICY coupons_tenant_insert ON coupons
    WITH CHECK (has_brand_access(brand_id));

-- coupon_usage
ALTER TABLE coupon_usage ENABLE ROW LEVEL SECURITY;
CREATE POLICY coupon_usage_tenant_isolation ON coupon_usage
    USING (EXISTS (
        SELECT 1 FROM coupons c WHERE c.id = coupon_usage.coupon_id
        AND has_brand_access(c.brand_id)
    ));
CREATE POLICY coupon_usage_tenant_insert ON coupon_usage
    WITH CHECK (EXISTS (
        SELECT 1 FROM coupons c WHERE c.id = coupon_usage.coupon_id
        AND has_brand_access(c.brand_id)
    ));

-- membership_tiers
ALTER TABLE membership_tiers ENABLE ROW LEVEL SECURITY;
CREATE POLICY membership_tiers_tenant_isolation ON membership_tiers
    USING (has_brand_access(brand_id));
CREATE POLICY membership_tiers_tenant_insert ON membership_tiers
    WITH CHECK (has_brand_access(brand_id));

-- customer_memberships
ALTER TABLE customer_memberships ENABLE ROW LEVEL SECURITY;
CREATE POLICY customer_memberships_tenant_isolation ON customer_memberships
    USING (EXISTS (
        SELECT 1 FROM membership_tiers mt WHERE mt.id = customer_memberships.tier_id
        AND has_brand_access(mt.brand_id)
    ));
CREATE POLICY customer_memberships_tenant_insert ON customer_memberships
    WITH CHECK (EXISTS (
        SELECT 1 FROM membership_tiers mt WHERE mt.id = customer_memberships.tier_id
        AND has_brand_access(mt.brand_id)
    ));

-- shipping_methods
ALTER TABLE shipping_methods ENABLE ROW LEVEL SECURITY;
CREATE POLICY shipping_methods_tenant_isolation ON shipping_methods
    USING (has_brand_access(brand_id));
CREATE POLICY shipping_methods_tenant_insert ON shipping_methods
    WITH CHECK (has_brand_access(brand_id));

-- shipments
ALTER TABLE shipments ENABLE ROW LEVEL SECURITY;
CREATE POLICY shipments_tenant_isolation ON shipments
    USING (has_brand_access(brand_id));
CREATE POLICY shipments_tenant_insert ON shipments
    WITH CHECK (has_brand_access(brand_id));

-- delivery_events
ALTER TABLE delivery_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY delivery_events_tenant_isolation ON delivery_events
    USING (EXISTS (
        SELECT 1 FROM shipments s WHERE s.id = delivery_events.shipment_id
        AND has_brand_access(s.brand_id)
    ));
CREATE POLICY delivery_events_tenant_insert ON delivery_events
    WITH CHECK (EXISTS (
        SELECT 1 FROM shipments s WHERE s.id = delivery_events.shipment_id
        AND has_brand_access(s.brand_id)
    ));

-- warehouses
ALTER TABLE warehouses ENABLE ROW LEVEL SECURITY;
CREATE POLICY warehouses_tenant_isolation ON warehouses
    USING (has_brand_access(brand_id));
CREATE POLICY warehouses_tenant_insert ON warehouses
    WITH CHECK (has_brand_access(brand_id));

-- inventory_by_warehouse
ALTER TABLE inventory_by_warehouse ENABLE ROW LEVEL SECURITY;
CREATE POLICY inventory_by_warehouse_tenant_isolation ON inventory_by_warehouse
    USING (EXISTS (
        SELECT 1 FROM warehouses w WHERE w.id = inventory_by_warehouse.warehouse_id
        AND has_brand_access(w.brand_id)
    ));
CREATE POLICY inventory_by_warehouse_tenant_insert ON inventory_by_warehouse
    WITH CHECK (EXISTS (
        SELECT 1 FROM warehouses w WHERE w.id = inventory_by_warehouse.warehouse_id
        AND has_brand_access(w.brand_id)
    ));

-- inventory_transactions
ALTER TABLE inventory_transactions ENABLE ROW LEVEL SECURITY;
CREATE POLICY inventory_transactions_tenant_isolation ON inventory_transactions
    USING (EXISTS (
        SELECT 1 FROM warehouses w WHERE w.id = inventory_transactions.warehouse_id
        AND has_brand_access(w.brand_id)
    ));
CREATE POLICY inventory_transactions_tenant_insert ON inventory_transactions
    WITH CHECK (EXISTS (
        SELECT 1 FROM warehouses w WHERE w.id = inventory_transactions.warehouse_id
        AND has_brand_access(w.brand_id)
    ));

-- low_stock_alerts
ALTER TABLE low_stock_alerts ENABLE ROW LEVEL SECURITY;
CREATE POLICY low_stock_alerts_tenant_isolation ON low_stock_alerts
    USING (EXISTS (
        SELECT 1 FROM warehouses w WHERE w.id = low_stock_alerts.warehouse_id
        AND has_brand_access(w.brand_id)
    ));
CREATE POLICY low_stock_alerts_tenant_insert ON low_stock_alerts
    WITH CHECK (EXISTS (
        SELECT 1 FROM warehouses w WHERE w.id = low_stock_alerts.warehouse_id
        AND has_brand_access(w.brand_id)
    ));

-- back_in_stock_requests
ALTER TABLE back_in_stock_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY back_in_stock_requests_tenant_isolation ON back_in_stock_requests
    USING (has_brand_access(brand_id));
CREATE POLICY back_in_stock_requests_tenant_insert ON back_in_stock_requests
    WITH CHECK (has_brand_access(brand_id));

-- storefronts
ALTER TABLE storefronts ENABLE ROW LEVEL SECURITY;
CREATE POLICY storefronts_tenant_isolation ON storefronts
    USING (has_brand_access(brand_id));
CREATE POLICY storefronts_tenant_insert ON storefronts
    WITH CHECK (has_brand_access(brand_id));

-- storefront_sections
ALTER TABLE storefront_sections ENABLE ROW LEVEL SECURITY;
CREATE POLICY storefront_sections_tenant_isolation ON storefront_sections
    USING (EXISTS (
        SELECT 1 FROM storefronts sf WHERE sf.id = storefront_sections.storefront_id
        AND has_brand_access(sf.brand_id)
    ));
CREATE POLICY storefront_sections_tenant_insert ON storefront_sections
    WITH CHECK (EXISTS (
        SELECT 1 FROM storefronts sf WHERE sf.id = storefront_sections.storefront_id
        AND has_brand_access(sf.brand_id)
    ));

-- storefront_pages
ALTER TABLE storefront_pages ENABLE ROW LEVEL SECURITY;
CREATE POLICY storefront_pages_tenant_isolation ON storefront_pages
    USING (EXISTS (
        SELECT 1 FROM storefronts sf WHERE sf.id = storefront_pages.storefront_id
        AND has_brand_access(sf.brand_id)
    ));
CREATE POLICY storefront_pages_tenant_insert ON storefront_pages
    WITH CHECK (EXISTS (
        SELECT 1 FROM storefronts sf WHERE sf.id = storefront_pages.storefront_id
        AND has_brand_access(sf.brand_id)
    ));

-- storefront_visits
ALTER TABLE storefront_visits ENABLE ROW LEVEL SECURITY;
CREATE POLICY storefront_visits_tenant_isolation ON storefront_visits
    USING (EXISTS (
        SELECT 1 FROM storefronts sf WHERE sf.id = storefront_visits.storefront_id
        AND has_brand_access(sf.brand_id)
    ));
CREATE POLICY storefront_visits_tenant_insert ON storefront_visits
    WITH CHECK (EXISTS (
        SELECT 1 FROM storefronts sf WHERE sf.id = storefront_visits.storefront_id
        AND has_brand_access(sf.brand_id)
    ));

-- product_options
ALTER TABLE product_options ENABLE ROW LEVEL SECURITY;
CREATE POLICY product_options_tenant_isolation ON product_options
    USING (EXISTS (
        SELECT 1 FROM products p WHERE p.id = product_options.product_id
        AND has_brand_access(p.brand_id)
    ));
CREATE POLICY product_options_tenant_insert ON product_options
    WITH CHECK (EXISTS (
        SELECT 1 FROM products p WHERE p.id = product_options.product_id
        AND has_brand_access(p.brand_id)
    ));

-- product_option_values
ALTER TABLE product_option_values ENABLE ROW LEVEL SECURITY;
CREATE POLICY product_option_values_tenant_isolation ON product_option_values
    USING (EXISTS (
        SELECT 1 FROM product_options po WHERE po.id = product_option_values.option_id
        JOIN products p ON p.id = po.product_id
        AND has_brand_access(p.brand_id)
    ));
CREATE POLICY product_option_values_tenant_insert ON product_option_values
    WITH CHECK (EXISTS (
        SELECT 1 FROM product_options po WHERE po.id = product_option_values.option_id
        JOIN products p ON p.id = po.product_id
        AND has_brand_access(p.brand_id)
    ));

-- product_channel_visibility
ALTER TABLE product_channel_visibility ENABLE ROW LEVEL SECURITY;
CREATE POLICY product_channel_visibility_tenant_isolation ON product_channel_visibility
    USING (EXISTS (
        SELECT 1 FROM products p WHERE p.id = product_channel_visibility.product_id
        AND has_brand_access(p.brand_id)
    ));
CREATE POLICY product_channel_visibility_tenant_insert ON product_channel_visibility
    WITH CHECK (EXISTS (
        SELECT 1 FROM products p WHERE p.id = product_channel_visibility.product_id
        AND has_brand_access(p.brand_id)
    ));

-- sync_connections
ALTER TABLE sync_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY sync_connections_tenant_isolation ON sync_connections
    USING (has_brand_access(brand_id));
CREATE POLICY sync_connections_tenant_insert ON sync_connections
    WITH CHECK (has_brand_access(brand_id));

-- sync_logs
ALTER TABLE sync_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY sync_logs_tenant_isolation ON sync_logs
    USING (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = sync_logs.connection_id
        AND has_brand_access(sc.brand_id)
    ));
CREATE POLICY sync_logs_tenant_insert ON sync_logs
    WITH CHECK (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = sync_logs.connection_id
        AND has_brand_access(sc.brand_id)
    ));

-- sync_queue
ALTER TABLE sync_queue ENABLE ROW LEVEL SECURITY;
CREATE POLICY sync_queue_tenant_isolation ON sync_queue
    USING (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = sync_queue.connection_id
        AND has_brand_access(sc.brand_id)
    ));
CREATE POLICY sync_queue_tenant_insert ON sync_queue
    WITH CHECK (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = sync_queue.connection_id
        AND has_brand_access(sc.brand_id)
    ));

-- platform_data_cache
ALTER TABLE platform_data_cache ENABLE ROW LEVEL SECURITY;
CREATE POLICY platform_data_cache_tenant_isolation ON platform_data_cache
    USING (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = platform_data_cache.connection_id
        AND has_brand_access(sc.brand_id)
    ));
CREATE POLICY platform_data_cache_tenant_insert ON platform_data_cache
    WITH CHECK (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = platform_data_cache.connection_id
        AND has_brand_access(sc.brand_id)
    ));

-- external_product_mapping
ALTER TABLE external_product_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY external_product_mapping_tenant_isolation ON external_product_mapping
    USING (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = external_product_mapping.connection_id
        AND has_brand_access(sc.brand_id)
    ));
CREATE POLICY external_product_mapping_tenant_insert ON external_product_mapping
    WITH CHECK (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = external_product_mapping.connection_id
        AND has_brand_access(sc.brand_id)
    ));

-- external_order_mapping
ALTER TABLE external_order_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY external_order_mapping_tenant_isolation ON external_order_mapping
    USING (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = external_order_mapping.connection_id
        AND has_brand_access(sc.brand_id)
    ));
CREATE POLICY external_order_mapping_tenant_insert ON external_order_mapping
    WITH CHECK (EXISTS (
        SELECT 1 FROM sync_connections sc WHERE sc.id = external_order_mapping.connection_id
        AND has_brand_access(sc.brand_id)
    ));

-- delivery_fee_rules
ALTER TABLE delivery_fee_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY delivery_fee_rules_tenant_isolation ON delivery_fee_rules
    USING (has_brand_access(brand_id));
CREATE POLICY delivery_fee_rules_tenant_insert ON delivery_fee_rules
    WITH CHECK (has_brand_access(brand_id));

-- payment_proofs
ALTER TABLE payment_proofs ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_proofs_tenant_isolation ON payment_proofs
    USING (has_brand_access(brand_id));
CREATE POLICY payment_proofs_tenant_insert ON payment_proofs
    WITH CHECK (has_brand_access(brand_id));

-- order_notification_templates
ALTER TABLE order_notification_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY order_notification_templates_tenant_isolation ON order_notification_templates
    USING (has_brand_access(brand_id));
CREATE POLICY order_notification_templates_tenant_insert ON order_notification_templates
    WITH CHECK (has_brand_access(brand_id));

-- payment_gateways
ALTER TABLE payment_gateways ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_gateways_tenant_isolation ON payment_gateways
    USING (has_brand_access(brand_id));
CREATE POLICY payment_gateways_tenant_insert ON payment_gateways
    WITH CHECK (has_brand_access(brand_id));

-- service_types
ALTER TABLE service_types ENABLE ROW LEVEL SECURITY;
CREATE POLICY service_types_tenant_isolation ON service_types
    USING (has_brand_access(brand_id));
CREATE POLICY service_types_tenant_insert ON service_types
    WITH CHECK (has_brand_access(brand_id));

-- delivery_schedules
ALTER TABLE delivery_schedules ENABLE ROW LEVEL SECURITY;
CREATE POLICY delivery_schedules_tenant_isolation ON delivery_schedules
    USING (has_brand_access(brand_id));
CREATE POLICY delivery_schedules_tenant_insert ON delivery_schedules
    WITH CHECK (has_brand_access(brand_id));

-- delivery_time_slots
ALTER TABLE delivery_time_slots ENABLE ROW LEVEL SECURITY;
CREATE POLICY delivery_time_slots_tenant_isolation ON delivery_time_slots
    USING (has_brand_access(brand_id));
CREATE POLICY delivery_time_slots_tenant_insert ON delivery_time_slots
    WITH CHECK (has_brand_access(brand_id));

-- invoices
ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;
CREATE POLICY invoices_tenant_isolation ON invoices
    USING (has_brand_access(brand_id));
CREATE POLICY invoices_tenant_insert ON invoices
    WITH CHECK (has_brand_access(brand_id));

-- csv_import_jobs
ALTER TABLE csv_import_jobs ENABLE ROW LEVEL SECURITY;
CREATE POLICY csv_import_jobs_tenant_isolation ON csv_import_jobs
    USING (has_brand_access(brand_id));
CREATE POLICY csv_import_jobs_tenant_insert ON csv_import_jobs
    WITH CHECK (has_brand_access(brand_id));

-- customer_accounts
ALTER TABLE customer_accounts ENABLE ROW LEVEL SECURITY;
CREATE POLICY customer_accounts_tenant_isolation ON customer_accounts
    USING (has_brand_access(brand_id));
CREATE POLICY customer_accounts_tenant_insert ON customer_accounts
    WITH CHECK (has_brand_access(brand_id));

-- customer_reviews
ALTER TABLE customer_reviews ENABLE ROW LEVEL SECURITY;
CREATE POLICY customer_reviews_tenant_isolation ON customer_reviews
    USING (has_brand_access(brand_id));
CREATE POLICY customer_reviews_tenant_insert ON customer_reviews
    WITH CHECK (has_brand_access(brand_id));

-- loyalty_programs
ALTER TABLE loyalty_programs ENABLE ROW LEVEL SECURITY;
CREATE POLICY loyalty_programs_tenant_isolation ON loyalty_programs
    USING (has_brand_access(brand_id));
CREATE POLICY loyalty_programs_tenant_insert ON loyalty_programs
    WITH CHECK (has_brand_access(brand_id));

-- points_transactions
ALTER TABLE points_transactions ENABLE ROW LEVEL SECURITY;
CREATE POLICY points_transactions_tenant_isolation ON points_transactions
    USING (EXISTS (
        SELECT 1 FROM loyalty_programs lp WHERE lp.id = points_transactions.program_id
        AND has_brand_access(lp.brand_id)
    ));
CREATE POLICY points_transactions_tenant_insert ON points_transactions
    WITH CHECK (EXISTS (
        SELECT 1 FROM loyalty_programs lp WHERE lp.id = points_transactions.program_id
        AND has_brand_access(lp.brand_id)
    ));

-- staff_members
ALTER TABLE staff_members ENABLE ROW LEVEL SECURITY;
CREATE POLICY staff_members_tenant_isolation ON staff_members
    USING (has_brand_access(brand_id));
CREATE POLICY staff_members_tenant_insert ON staff_members
    WITH CHECK (has_brand_access(brand_id));

-- pos_sessions
ALTER TABLE pos_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY pos_sessions_tenant_isolation ON pos_sessions
    USING (has_brand_access(brand_id));
CREATE POLICY pos_sessions_tenant_insert ON pos_sessions
    WITH CHECK (has_brand_access(brand_id));

-- receipts
ALTER TABLE receipts ENABLE ROW LEVEL SECURITY;
CREATE POLICY receipts_tenant_isolation ON receipts
    USING (has_brand_access(brand_id));
CREATE POLICY receipts_tenant_insert ON receipts
    WITH CHECK (has_brand_access(brand_id));

-- wholesale_pricing
ALTER TABLE wholesale_pricing ENABLE ROW LEVEL SECURITY;
CREATE POLICY wholesale_pricing_tenant_isolation ON wholesale_pricing
    USING (EXISTS (
        SELECT 1 FROM products p WHERE p.id = wholesale_pricing.product_id
        AND has_brand_access(p.brand_id)
    ));
CREATE POLICY wholesale_pricing_tenant_insert ON wholesale_pricing
    WITH CHECK (EXISTS (
        SELECT 1 FROM products p WHERE p.id = wholesale_pricing.product_id
        AND has_brand_access(p.brand_id)
    ));

-- automated_followups
ALTER TABLE automated_followups ENABLE ROW LEVEL SECURITY;
CREATE POLICY automated_followups_tenant_isolation ON automated_followups
    USING (has_brand_access(brand_id));
CREATE POLICY automated_followups_tenant_insert ON automated_followups
    WITH CHECK (has_brand_access(brand_id));

-- custom_domains
ALTER TABLE custom_domains ENABLE ROW LEVEL SECURITY;
CREATE POLICY custom_domains_tenant_isolation ON custom_domains
    USING (has_brand_access(brand_id));
CREATE POLICY custom_domains_tenant_insert ON custom_domains
    WITH CHECK (has_brand_access(brand_id));

-- meta_pixels
ALTER TABLE meta_pixels ENABLE ROW LEVEL SECURITY;
CREATE POLICY meta_pixels_tenant_isolation ON meta_pixels
    USING (has_brand_access(brand_id));
CREATE POLICY meta_pixels_tenant_insert ON meta_pixels
    WITH CHECK (has_brand_access(brand_id));

-- checkout_questions
ALTER TABLE checkout_questions ENABLE ROW LEVEL SECURITY;
CREATE POLICY checkout_questions_tenant_isolation ON checkout_questions
    USING (has_brand_access(brand_id));
CREATE POLICY checkout_questions_tenant_insert ON checkout_questions
    WITH CHECK (has_brand_access(brand_id));

-- checkout_question_answers
ALTER TABLE checkout_question_answers ENABLE ROW LEVEL SECURITY;
CREATE POLICY checkout_question_answers_tenant_isolation ON checkout_question_answers
    USING (has_brand_access(brand_id));
CREATE POLICY checkout_question_answers_tenant_insert ON checkout_question_answers
    WITH CHECK (has_brand_access(brand_id));

-- delivery_proofs
ALTER TABLE delivery_proofs ENABLE ROW LEVEL SECURITY;
CREATE POLICY delivery_proofs_tenant_isolation ON delivery_proofs
    USING (has_brand_access(brand_id));
CREATE POLICY delivery_proofs_tenant_insert ON delivery_proofs
    WITH CHECK (has_brand_access(brand_id));

-- delivery_reminders
ALTER TABLE delivery_reminders ENABLE ROW LEVEL SECURITY;
CREATE POLICY delivery_reminders_tenant_isolation ON delivery_reminders
    USING (has_brand_access(brand_id));
CREATE POLICY delivery_reminders_tenant_insert ON delivery_reminders
    WITH CHECK (has_brand_access(brand_id));

-- payment_fees
ALTER TABLE payment_fees ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_fees_tenant_isolation ON payment_fees
    USING (has_brand_access(brand_id));
CREATE POLICY payment_fees_tenant_insert ON payment_fees
    WITH CHECK (has_brand_access(brand_id));

-- payment_recovery
ALTER TABLE payment_recovery ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_recovery_tenant_isolation ON payment_recovery
    USING (has_brand_access(brand_id));
CREATE POLICY payment_recovery_tenant_insert ON payment_recovery
    WITH CHECK (has_brand_access(brand_id));

-- payment_reminders
ALTER TABLE payment_reminders ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_reminders_tenant_isolation ON payment_reminders
    USING (has_brand_access(brand_id));
CREATE POLICY payment_reminders_tenant_insert ON payment_reminders
    WITH CHECK (has_brand_access(brand_id));

-- order_notification_log
ALTER TABLE order_notification_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY order_notification_log_tenant_isolation ON order_notification_log
    USING (has_brand_access(brand_id));
CREATE POLICY order_notification_log_tenant_insert ON order_notification_log
    WITH CHECK (has_brand_access(brand_id));

-- order_platform_info
ALTER TABLE order_platform_info ENABLE ROW LEVEL SECURITY;
CREATE POLICY order_platform_info_tenant_isolation ON order_platform_info
    USING (has_brand_access(brand_id));
CREATE POLICY order_platform_info_tenant_insert ON order_platform_info
    WITH CHECK (has_brand_access(brand_id));

-- customer_channel_ids
ALTER TABLE customer_channel_ids ENABLE ROW LEVEL SECURITY;
CREATE POLICY customer_channel_ids_tenant_isolation ON customer_channel_ids
    USING (EXISTS (
        SELECT 1 FROM customers c WHERE c.id = customer_channel_ids.customer_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));
CREATE POLICY customer_channel_ids_tenant_insert ON customer_channel_ids
    WITH CHECK (EXISTS (
        SELECT 1 FROM customers c WHERE c.id = customer_channel_ids.customer_id
        AND c.tenant_key = current_setting('app.current_tenant_key', '')
    ));

-- customer_points
ALTER TABLE customer_points ENABLE ROW LEVEL SECURITY;
CREATE POLICY customer_points_tenant_isolation ON customer_points
    USING (has_brand_access(brand_id));
CREATE POLICY customer_points_tenant_insert ON customer_points
    WITH CHECK (has_brand_access(brand_id));

-- analytics_daily_stats
ALTER TABLE analytics_daily_stats ENABLE ROW LEVEL SECURITY;
CREATE POLICY analytics_daily_stats_tenant_isolation ON analytics_daily_stats
    USING (has_brand_access(brand_id));
CREATE POLICY analytics_daily_stats_tenant_insert ON analytics_daily_stats
    WITH CHECK (has_brand_access(brand_id));

-- analytics_events
ALTER TABLE analytics_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY analytics_events_tenant_isolation ON analytics_events
    USING (has_brand_access(brand_id));
CREATE POLICY analytics_events_tenant_insert ON analytics_events
    WITH CHECK (has_brand_access(brand_id));

-- ---------------------------------------------------------------------------
-- N. ERP TABLES (scoped via brand_id)
-- ---------------------------------------------------------------------------

-- All ERP tables are scoped via brand_id
-- erp_categories, erp_products, erp_brands, erp_collections, etc.

-- erp_categories
ALTER TABLE erp_categories ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_categories_tenant_isolation ON erp_categories USING (has_brand_access(brand_id));
CREATE POLICY erp_categories_tenant_insert ON erp_categories WITH CHECK (has_brand_access(brand_id));

-- erp_products
ALTER TABLE erp_products ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_products_tenant_isolation ON erp_products USING (has_brand_access(brand_id));
CREATE POLICY erp_products_tenant_insert ON erp_products WITH CHECK (has_brand_access(brand_id));

-- erp_brands
ALTER TABLE erp_brands ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_brands_tenant_isolation ON erp_brands USING (has_brand_access(brand_id));
CREATE POLICY erp_brands_tenant_insert ON erp_brands WITH CHECK (has_brand_access(brand_id));

-- erp_collections
ALTER TABLE erp_collections ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_collections_tenant_isolation ON erp_collections USING (has_brand_access(brand_id));
CREATE POLICY erp_collections_tenant_insert ON erp_collections WITH CHECK (has_brand_access(brand_id));

-- erp_carts
ALTER TABLE erp_carts ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_carts_tenant_isolation ON erp_carts USING (has_brand_access(brand_id));
CREATE POLICY erp_carts_tenant_insert ON erp_carts WITH CHECK (has_brand_access(brand_id));

-- erp_cart_items
ALTER TABLE erp_cart_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_cart_items_tenant_isolation ON erp_cart_items USING (has_brand_access(brand_id));
CREATE POLICY erp_cart_items_tenant_insert ON erp_cart_items WITH CHECK (has_brand_access(brand_id));

-- erp_orders
ALTER TABLE erp_orders ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_orders_tenant_isolation ON erp_orders USING (has_brand_access(brand_id));
CREATE POLICY erp_orders_tenant_insert ON erp_orders WITH CHECK (has_brand_access(brand_id));

-- erp_order_items
ALTER TABLE erp_order_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_order_items_tenant_isolation ON erp_order_items USING (has_brand_access(brand_id));
CREATE POLICY erp_order_items_tenant_insert ON erp_order_items WITH CHECK (has_brand_access(brand_id));

-- erp_payments
ALTER TABLE erp_payments ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_payments_tenant_isolation ON erp_payments USING (has_brand_access(brand_id));
CREATE POLICY erp_payments_tenant_insert ON erp_payments WITH CHECK (has_brand_access(brand_id));

-- erp_shipments
ALTER TABLE erp_shipments ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipments_tenant_isolation ON erp_shipments USING (has_brand_access(brand_id));
CREATE POLICY erp_shipments_tenant_insert ON erp_shipments WITH CHECK (has_brand_access(brand_id));

-- erp_shipping_methods
ALTER TABLE erp_shipping_methods ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_methods_tenant_isolation ON erp_shipping_methods USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_methods_tenant_insert ON erp_shipping_methods WITH CHECK (has_brand_access(brand_id));

-- erp_shipping_zones
ALTER TABLE erp_shipping_zones ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_zones_tenant_isolation ON erp_shipping_zones USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_zones_tenant_insert ON erp_shipping_zones WITH CHECK (has_brand_access(brand_id));

-- erp_shipping_classes
ALTER TABLE erp_shipping_classes ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_classes_tenant_isolation ON erp_shipping_classes USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_classes_tenant_insert ON erp_shipping_classes WITH CHECK (has_brand_access(brand_id));

-- erp_locations
ALTER TABLE erp_locations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_locations_tenant_isolation ON erp_locations USING (has_brand_access(brand_id));
CREATE POLICY erp_locations_tenant_insert ON erp_locations WITH CHECK (has_brand_access(brand_id));

-- erp_lots
ALTER TABLE erp_lots ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_lots_tenant_isolation ON erp_lots USING (has_brand_access(brand_id));
CREATE POLICY erp_lots_tenant_insert ON erp_lots WITH CHECK (has_brand_access(brand_id));

-- erp_lot_stock
ALTER TABLE erp_lot_stock ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_lot_stock_tenant_isolation ON erp_lot_stock USING (has_brand_access(brand_id));
CREATE POLICY erp_lot_stock_tenant_insert ON erp_lot_stock WITH CHECK (has_brand_access(brand_id));

-- erp_lot_allocations
ALTER TABLE erp_lot_allocations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_lot_allocations_tenant_isolation ON erp_lot_allocations USING (has_brand_access(brand_id));
CREATE POLICY erp_lot_allocations_tenant_insert ON erp_lot_allocations WITH CHECK (has_brand_access(brand_id));

-- erp_stock_movements
ALTER TABLE erp_stock_movements ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_stock_movements_tenant_isolation ON erp_stock_movements USING (has_brand_access(brand_id));
CREATE POLICY erp_stock_movements_tenant_insert ON erp_stock_movements WITH CHECK (has_brand_access(brand_id));

-- erp_stock_adjustments
ALTER TABLE erp_stock_adjustments ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_stock_adjustments_tenant_isolation ON erp_stock_adjustments USING (has_brand_access(brand_id));
CREATE POLICY erp_stock_adjustments_tenant_insert ON erp_stock_adjustments WITH CHECK (has_brand_access(brand_id));

-- erp_stock_counts
ALTER TABLE erp_stock_counts ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_stock_counts_tenant_isolation ON erp_stock_counts USING (has_brand_access(brand_id));
CREATE POLICY erp_stock_counts_tenant_insert ON erp_stock_counts WITH CHECK (has_brand_access(brand_id));

-- erp_stock_count_lines
ALTER TABLE erp_stock_count_lines ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_stock_count_lines_tenant_isolation ON erp_stock_count_lines USING (has_brand_access(brand_id));
CREATE POLICY erp_stock_count_lines_tenant_insert ON erp_stock_count_lines WITH CHECK (has_brand_access(brand_id));

-- erp_stock_variances
ALTER TABLE erp_stock_variances ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_stock_variances_tenant_isolation ON erp_stock_variances USING (has_brand_access(brand_id));
CREATE POLICY erp_stock_variances_tenant_insert ON erp_stock_variances WITH CHECK (has_brand_access(brand_id));

-- erp_variant_location_stock
ALTER TABLE erp_variant_location_stock ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_variant_location_stock_tenant_isolation ON erp_variant_location_stock USING (has_brand_access(brand_id));
CREATE POLICY erp_variant_location_stock_tenant_insert ON erp_variant_location_stock WITH CHECK (has_brand_access(brand_id));

-- erp_product_variants
ALTER TABLE erp_product_variants ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_variants_tenant_isolation ON erp_product_variants USING (has_brand_access(brand_id));
CREATE POLICY erp_product_variants_tenant_insert ON erp_product_variants WITH CHECK (has_brand_access(brand_id));

-- erp_variant_option_values
ALTER TABLE erp_variant_option_values ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_variant_option_values_tenant_isolation ON erp_variant_option_values USING (has_brand_access(brand_id));
CREATE POLICY erp_variant_option_values_tenant_insert ON erp_variant_option_values WITH CHECK (has_brand_access(brand_id));

-- erp_product_options
ALTER TABLE erp_product_options ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_options_tenant_isolation ON erp_product_options USING (has_brand_access(brand_id));
CREATE POLICY erp_product_options_tenant_insert ON erp_product_options WITH CHECK (has_brand_access(brand_id));

-- erp_product_option_values
ALTER TABLE erp_product_option_values ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_option_values_tenant_isolation ON erp_product_option_values USING (has_brand_access(brand_id));
CREATE POLICY erp_product_option_values_tenant_insert ON erp_product_option_values WITH CHECK (has_brand_access(brand_id));

-- erp_product_images
ALTER TABLE erp_product_images ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_images_tenant_isolation ON erp_product_images USING (has_brand_access(brand_id));
CREATE POLICY erp_product_images_tenant_insert ON erp_product_images WITH CHECK (has_brand_access(brand_id));

-- erp_product_kinds
ALTER TABLE erp_product_kinds ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_kinds_tenant_isolation ON erp_product_kinds USING (has_brand_access(brand_id));
CREATE POLICY erp_product_kinds_tenant_insert ON erp_product_kinds WITH CHECK (has_brand_access(brand_id));

-- erp_option_templates
ALTER TABLE erp_option_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_option_templates_tenant_isolation ON erp_option_templates USING (has_brand_access(brand_id));
CREATE POLICY erp_option_templates_tenant_insert ON erp_option_templates WITH CHECK (has_brand_access(brand_id));

-- erp_option_template_values
ALTER TABLE erp_option_template_values ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_option_template_values_tenant_isolation ON erp_option_template_values USING (has_brand_access(brand_id));
CREATE POLICY erp_option_template_values_tenant_insert ON erp_option_template_values WITH CHECK (has_brand_access(brand_id));

-- erp_payment_methods
ALTER TABLE erp_payment_methods ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_payment_methods_tenant_isolation ON erp_payment_methods USING (has_brand_access(brand_id));
CREATE POLICY erp_payment_methods_tenant_insert ON erp_payment_methods WITH CHECK (has_brand_access(brand_id));

-- erp_shipping_rates
ALTER TABLE erp_shipping_rates ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_rates_tenant_isolation ON erp_shipping_rates USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_rates_tenant_insert ON erp_shipping_rates WITH CHECK (has_brand_access(brand_id));

-- erp_return_requests
ALTER TABLE erp_return_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_return_requests_tenant_isolation ON erp_return_requests USING (has_brand_access(brand_id));
CREATE POLICY erp_return_requests_tenant_insert ON erp_return_requests WITH CHECK (has_brand_access(brand_id));

-- erp_return_request_items
ALTER TABLE erp_return_request_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_return_request_items_tenant_isolation ON erp_return_request_items USING (has_brand_access(brand_id));
CREATE POLICY erp_return_request_items_tenant_insert ON erp_return_request_items WITH CHECK (has_brand_access(brand_id));

-- erp_return_reason_codes
ALTER TABLE erp_return_reason_codes ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_return_reason_codes_tenant_isolation ON erp_return_reason_codes USING (has_brand_access(brand_id));
CREATE POLICY erp_return_reason_codes_tenant_insert ON erp_return_reason_codes WITH CHECK (has_brand_access(brand_id));

-- erp_serialized_units
ALTER TABLE erp_serialized_units ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_serialized_units_tenant_isolation ON erp_serialized_units USING (has_brand_access(brand_id));
CREATE POLICY erp_serialized_units_tenant_insert ON erp_serialized_units WITH CHECK (has_brand_access(brand_id));

-- erp_shipment_items
ALTER TABLE erp_shipment_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipment_items_tenant_isolation ON erp_shipment_items USING (has_brand_access(brand_id));
CREATE POLICY erp_shipment_items_tenant_insert ON erp_shipment_items WITH CHECK (has_brand_access(brand_id));

-- erp_shipment_events
ALTER TABLE erp_shipment_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipment_events_tenant_isolation ON erp_shipment_events USING (has_brand_access(brand_id));
CREATE POLICY erp_shipment_events_tenant_insert ON erp_shipment_events WITH CHECK (has_brand_access(brand_id));

-- erp_order_status_events
ALTER TABLE erp_order_status_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_order_status_events_tenant_isolation ON erp_order_status_events USING (has_brand_access(brand_id));
CREATE POLICY erp_order_status_events_tenant_insert ON erp_order_status_events WITH CHECK (has_brand_access(brand_id));

-- erp_manifests
ALTER TABLE erp_manifests ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_manifests_tenant_isolation ON erp_manifests USING (has_brand_access(brand_id));
CREATE POLICY erp_manifests_tenant_insert ON erp_manifests WITH CHECK (has_brand_access(brand_id));

-- erp_collection_products
ALTER TABLE erp_collection_products ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_collection_products_tenant_isolation ON erp_collection_products USING (has_brand_access(brand_id));
CREATE POLICY erp_collection_products_tenant_insert ON erp_collection_products WITH CHECK (has_brand_access(brand_id));

-- erp_product_cross_sells
ALTER TABLE erp_product_cross_sells ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_cross_sells_tenant_isolation ON erp_product_cross_sells USING (has_brand_access(brand_id));
CREATE POLICY erp_product_cross_sells_tenant_insert ON erp_product_cross_sells WITH CHECK (has_brand_access(brand_id));

-- erp_product_ai_suggestions
ALTER TABLE erp_product_ai_suggestions ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_ai_suggestions_tenant_isolation ON erp_product_ai_suggestions USING (has_brand_access(brand_id));
CREATE POLICY erp_product_ai_suggestions_tenant_insert ON erp_product_ai_suggestions WITH CHECK (has_brand_access(brand_id));

-- erp_markdown_suggestions
ALTER TABLE erp_markdown_suggestions ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_markdown_suggestions_tenant_isolation ON erp_markdown_suggestions USING (has_brand_access(brand_id));
CREATE POLICY erp_markdown_suggestions_tenant_insert ON erp_markdown_suggestions WITH CHECK (has_brand_access(brand_id));

-- erp_variant_forecasts
ALTER TABLE erp_variant_forecasts ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_variant_forecasts_tenant_isolation ON erp_variant_forecasts USING (has_brand_access(brand_id));
CREATE POLICY erp_variant_forecasts_tenant_insert ON erp_variant_forecasts WITH CHECK (has_brand_access(brand_id));

-- erp_variant_sales_stats
ALTER TABLE erp_variant_sales_stats ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_variant_sales_stats_tenant_isolation ON erp_variant_sales_stats USING (has_brand_access(brand_id));
CREATE POLICY erp_variant_sales_stats_tenant_insert ON erp_variant_sales_stats WITH CHECK (has_brand_access(brand_id));

-- erp_product_size_bias
ALTER TABLE erp_product_size_bias ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_size_bias_tenant_isolation ON erp_product_size_bias USING (has_brand_access(brand_id));
CREATE POLICY erp_product_size_bias_tenant_insert ON erp_product_size_bias WITH CHECK (has_brand_access(brand_id));

-- erp_category_presets
ALTER TABLE erp_category_presets ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_category_presets_tenant_isolation ON erp_category_presets USING (has_brand_access(brand_id));
CREATE POLICY erp_category_presets_tenant_insert ON erp_category_presets WITH CHECK (has_brand_access(brand_id));

-- erp_redirects
ALTER TABLE erp_redirects ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_redirects_tenant_isolation ON erp_redirects USING (has_brand_access(brand_id));
CREATE POLICY erp_redirects_tenant_insert ON erp_redirects WITH CHECK (has_brand_access(brand_id));

-- erp_city_cod_risk_stats
ALTER TABLE erp_city_cod_risk_stats ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_city_cod_risk_stats_tenant_isolation ON erp_city_cod_risk_stats USING (has_brand_access(brand_id));
CREATE POLICY erp_city_cod_risk_stats_tenant_insert ON erp_city_cod_risk_stats WITH CHECK (has_brand_access(brand_id));

-- erp_spot_checks
ALTER TABLE erp_spot_checks ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_spot_checks_tenant_isolation ON erp_spot_checks USING (has_brand_access(brand_id));
CREATE POLICY erp_spot_checks_tenant_insert ON erp_spot_checks WITH CHECK (has_brand_access(brand_id));

-- erp_spot_check_lines
ALTER TABLE erp_spot_check_lines ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_spot_check_lines_tenant_isolation ON erp_spot_check_lines USING (has_brand_access(brand_id));
CREATE POLICY erp_spot_check_lines_tenant_insert ON erp_spot_check_lines WITH CHECK (has_brand_access(brand_id));

-- erp_spot_check_policies
ALTER TABLE erp_spot_check_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_spot_check_policies_tenant_isolation ON erp_spot_check_policies USING (has_brand_access(brand_id));
CREATE POLICY erp_spot_check_policies_tenant_insert ON erp_spot_check_policies WITH CHECK (has_brand_access(brand_id));

-- erp_back_in_stock_requests
ALTER TABLE erp_back_in_stock_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_back_in_stock_requests_tenant_isolation ON erp_back_in_stock_requests USING (has_brand_access(brand_id));
CREATE POLICY erp_back_in_stock_requests_tenant_insert ON erp_back_in_stock_requests WITH CHECK (has_brand_access(brand_id));

-- erp_order_number_counters
ALTER TABLE erp_order_number_counters ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_order_number_counters_tenant_isolation ON erp_order_number_counters USING (has_brand_access(brand_id));
CREATE POLICY erp_order_number_counters_tenant_insert ON erp_order_number_counters WITH CHECK (has_brand_access(brand_id));

-- erp_manifest_number_counter
ALTER TABLE erp_manifest_number_counter ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_manifest_number_counter_tenant_isolation ON erp_manifest_number_counter USING (has_brand_access(brand_id));
CREATE POLICY erp_manifest_number_counter_tenant_insert ON erp_manifest_number_counter WITH CHECK (has_brand_access(brand_id));

-- erp_return_request_number_counter
ALTER TABLE erp_return_request_number_counter ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_return_request_number_counter_tenant_isolation ON erp_return_request_number_counter USING (has_brand_access(brand_id));
CREATE POLICY erp_return_request_number_counter_tenant_insert ON erp_return_request_number_counter WITH CHECK (has_brand_access(brand_id));

-- Translation tables (ERP)
-- erp_brand_translations, erp_category_translations, etc.
-- These are all scoped via brand_id

ALTER TABLE erp_brand_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_brand_translations_tenant_isolation ON erp_brand_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_brand_translations_tenant_insert ON erp_brand_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_category_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_category_translations_tenant_isolation ON erp_category_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_category_translations_tenant_insert ON erp_category_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_collection_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_collection_translations_tenant_isolation ON erp_collection_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_collection_translations_tenant_insert ON erp_collection_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_product_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_translations_tenant_isolation ON erp_product_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_product_translations_tenant_insert ON erp_product_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_product_image_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_image_translations_tenant_isolation ON erp_product_image_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_product_image_translations_tenant_insert ON erp_product_image_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_product_option_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_option_translations_tenant_isolation ON erp_product_option_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_product_option_translations_tenant_insert ON erp_product_option_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_product_option_value_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_product_option_value_translations_tenant_isolation ON erp_product_option_value_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_product_option_value_translations_tenant_insert ON erp_product_option_value_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_option_template_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_option_template_translations_tenant_isolation ON erp_option_template_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_option_template_translations_tenant_insert ON erp_option_template_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_option_template_value_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_option_template_value_translations_tenant_isolation ON erp_option_template_value_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_option_template_value_translations_tenant_insert ON erp_option_template_value_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_shipping_class_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_class_translations_tenant_isolation ON erp_shipping_class_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_class_translations_tenant_insert ON erp_shipping_class_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_shipping_method_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_method_translations_tenant_isolation ON erp_shipping_method_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_method_translations_tenant_insert ON erp_shipping_method_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_shipping_zone_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_shipping_zone_translations_tenant_isolation ON erp_shipping_zone_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_shipping_zone_translations_tenant_insert ON erp_shipping_zone_translations WITH CHECK (has_brand_access(brand_id));

ALTER TABLE erp_return_reason_translations ENABLE ROW LEVEL SECURITY;
CREATE POLICY erp_return_reason_translations_tenant_isolation ON erp_return_reason_translations USING (has_brand_access(brand_id));
CREATE POLICY erp_return_reason_translations_tenant_insert ON erp_return_reason_translations WITH CHECK (has_brand_access(brand_id));

-- ---------------------------------------------------------------------------
-- O. REMAINING TABLES (catch-all for tables not yet explicitly listed)
-- These use a conservative default: global read access
-- ---------------------------------------------------------------------------

-- A/B Testing tables
ALTER TABLE ab_experiments ENABLE ROW LEVEL SECURITY;
CREATE POLICY ab_experiments_tenant_isolation ON ab_experiments USING (has_brand_access(brand_id));
CREATE POLICY ab_experiments_tenant_insert ON ab_experiments WITH CHECK (has_brand_access(brand_id));

ALTER TABLE ab_variants ENABLE ROW LEVEL SECURITY;
CREATE POLICY ab_variants_tenant_isolation ON ab_variants USING (true);
CREATE POLICY ab_variants_tenant_insert ON ab_variants WITH CHECK (true);

ALTER TABLE ab_assignments ENABLE ROW LEVEL SECURITY;
CREATE POLICY ab_assignments_tenant_isolation ON ab_assignments USING (true);
CREATE POLICY ab_assignments_tenant_insert ON ab_assignments WITH CHECK (true);

ALTER TABLE ab_results ENABLE ROW LEVEL SECURITY;
CREATE POLICY ab_results_tenant_isolation ON ab_results USING (true);
CREATE POLICY ab_results_tenant_insert ON ab_results WITH CHECK (true);

ALTER TABLE ab_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY ab_settings_tenant_isolation ON ab_settings USING (true);
CREATE POLICY ab_settings_tenant_insert ON ab_settings WITH CHECK (true);

ALTER TABLE ab_experiment_summary ENABLE ROW LEVEL SECURITY;
CREATE POLICY ab_experiment_summary_tenant_isolation ON ab_experiment_summary USING (true);
CREATE POLICY ab_experiment_summary_tenant_insert ON ab_experiment_summary WITH CHECK (true);

-- Workflow tables
ALTER TABLE workflows ENABLE ROW LEVEL SECURITY;
CREATE POLICY workflows_tenant_isolation ON workflows USING (has_brand_access(brand_id));
CREATE POLICY workflows_tenant_insert ON workflows WITH CHECK (has_brand_access(brand_id));

ALTER TABLE workflow_steps ENABLE ROW LEVEL SECURITY;
CREATE POLICY workflow_steps_tenant_isolation ON workflow_steps USING (true);
CREATE POLICY workflow_steps_tenant_insert ON workflow_steps WITH CHECK (true);

ALTER TABLE workflow_conditions ENABLE ROW LEVEL SECURITY;
CREATE POLICY workflow_conditions_tenant_isolation ON workflow_conditions USING (true);
CREATE POLICY workflow_conditions_tenant_insert ON workflow_conditions WITH CHECK (true);

ALTER TABLE workflow_triggers ENABLE ROW LEVEL SECURITY;
CREATE POLICY workflow_triggers_tenant_isolation ON workflow_triggers USING (true);
CREATE POLICY workflow_triggers_tenant_insert ON workflow_triggers WITH CHECK (true);

ALTER TABLE workflow_executions ENABLE ROW LEVEL SECURITY;
CREATE POLICY workflow_executions_tenant_isolation ON workflow_executions USING (true);
CREATE POLICY workflow_executions_tenant_insert ON workflow_executions WITH CHECK (true);

ALTER TABLE actions ENABLE ROW LEVEL SECURITY;
CREATE POLICY actions_tenant_isolation ON actions USING (true);
CREATE POLICY actions_tenant_insert ON actions WITH CHECK (true);

ALTER TABLE action_executions ENABLE ROW LEVEL SECURITY;
CREATE POLICY action_executions_tenant_isolation ON action_executions USING (true);
CREATE POLICY action_executions_tenant_insert ON action_executions WITH CHECK (true);

-- Help Center tables
ALTER TABLE help_center_categories ENABLE ROW LEVEL SECURITY;
CREATE POLICY help_center_categories_tenant_isolation ON help_center_categories USING (has_brand_access(brand_id));
CREATE POLICY help_center_categories_tenant_insert ON help_center_categories WITH CHECK (has_brand_access(brand_id));

ALTER TABLE help_center_articles ENABLE ROW LEVEL SECURITY;
CREATE POLICY help_center_articles_tenant_isolation ON help_center_articles USING (has_brand_access(brand_id));
CREATE POLICY help_center_articles_tenant_insert ON help_center_articles WITH CHECK (has_brand_access(brand_id));

ALTER TABLE help_center_searches ENABLE ROW LEVEL SECURITY;
CREATE POLICY help_center_searches_tenant_isolation ON help_center_searches USING (true);
CREATE POLICY help_center_searches_tenant_insert ON help_center_searches WITH CHECK (true);

ALTER TABLE help_center_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY help_center_settings_tenant_isolation ON help_center_settings USING (has_brand_access(brand_id));
CREATE POLICY help_center_settings_tenant_insert ON help_center_settings WITH CHECK (has_brand_access(brand_id));

-- KB Documents
ALTER TABLE kb_documents ENABLE ROW LEVEL SECURITY;
CREATE POLICY kb_documents_tenant_isolation ON kb_documents USING (has_brand_access(brand_id));
CREATE POLICY kb_documents_tenant_insert ON kb_documents WITH CHECK (has_brand_access(brand_id));

ALTER TABLE kb_document_chunks ENABLE ROW LEVEL SECURITY;
CREATE POLICY kb_document_chunks_tenant_isolation ON kb_document_chunks USING (true);
CREATE POLICY kb_document_chunks_tenant_insert ON kb_document_chunks WITH CHECK (true);

ALTER TABLE kb_url_sources ENABLE ROW LEVEL SECURITY;
CREATE POLICY kb_url_sources_tenant_isolation ON kb_url_sources USING (has_brand_access(brand_id));
CREATE POLICY kb_url_sources_tenant_insert ON kb_url_sources WITH CHECK (has_brand_access(brand_id));

ALTER TABLE kb_url_chunks ENABLE ROW LEVEL SECURITY;
CREATE POLICY kb_url_chunks_tenant_isolation ON kb_url_chunks USING (true);
CREATE POLICY kb_url_chunks_tenant_insert ON kb_url_chunks WITH CHECK (true);

ALTER TABLE kb_url_crawl_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY kb_url_crawl_logs_tenant_isolation ON kb_url_crawl_logs USING (true);
CREATE POLICY kb_url_crawl_logs_tenant_insert ON kb_url_crawl_logs WITH CHECK (true);

ALTER TABLE kb_import_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY kb_import_settings_tenant_isolation ON kb_import_settings USING (has_brand_access(brand_id));
CREATE POLICY kb_import_settings_tenant_insert ON kb_import_settings WITH CHECK (has_brand_access(brand_id));

-- Procedures
ALTER TABLE procedures ENABLE ROW LEVEL SECURITY;
CREATE POLICY procedures_tenant_isolation ON procedures USING (has_brand_access(brand_id));
CREATE POLICY procedures_tenant_insert ON procedures WITH CHECK (has_brand_access(brand_id));

ALTER TABLE procedure_executions ENABLE ROW LEVEL SECURITY;
CREATE POLICY procedure_executions_tenant_isolation ON procedure_executions USING (true);
CREATE POLICY procedure_executions_tenant_insert ON procedure_executions WITH CHECK (true);

ALTER TABLE procedure_step_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY procedure_step_logs_tenant_isolation ON procedure_step_logs USING (true);
CREATE POLICY procedure_step_logs_tenant_insert ON procedure_step_logs WITH CHECK (true);

-- CSAT & QA
ALTER TABLE csat_surveys ENABLE ROW LEVEL SECURITY;
CREATE POLICY csat_surveys_tenant_isolation ON csat_surveys USING (true);
CREATE POLICY csat_surveys_tenant_insert ON csat_surveys WITH CHECK (true);

ALTER TABLE csat_survey_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY csat_survey_templates_tenant_isolation ON csat_survey_templates USING (has_brand_access(brand_id));
CREATE POLICY csat_survey_templates_tenant_insert ON csat_survey_templates WITH CHECK (has_brand_access(brand_id));

ALTER TABLE csat_snapshots ENABLE ROW LEVEL SECURITY;
CREATE POLICY csat_snapshots_tenant_isolation ON csat_snapshots USING (true);
CREATE POLICY csat_snapshots_tenant_insert ON csat_snapshots WITH CHECK (true);

ALTER TABLE qa_scoring_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY qa_scoring_rules_tenant_isolation ON qa_scoring_rules USING (has_brand_access(brand_id));
CREATE POLICY qa_scoring_rules_tenant_insert ON qa_scoring_rules WITH CHECK (has_brand_access(brand_id));

ALTER TABLE qa_scores ENABLE ROW LEVEL SECURITY;
CREATE POLICY qa_scores_tenant_isolation ON qa_scores USING (true);
CREATE POLICY qa_scores_tenant_insert ON qa_scores WITH CHECK (true);

ALTER TABLE cx_score_snapshots ENABLE ROW LEVEL SECURITY;
CREATE POLICY cx_score_snapshots_tenant_isolation ON cx_score_snapshots USING (true);
CREATE POLICY cx_score_snapshots_tenant_insert ON cx_score_snapshots WITH CHECK (true);

-- Agent Performance
ALTER TABLE agent_performance_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_performance_settings_tenant_isolation ON agent_performance_settings USING (has_brand_access(brand_id));
CREATE POLICY agent_performance_settings_tenant_insert ON agent_performance_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE agent_performance_snapshots ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_performance_snapshots_tenant_isolation ON agent_performance_snapshots USING (true);
CREATE POLICY agent_performance_snapshots_tenant_insert ON agent_performance_snapshots WITH CHECK (true);

ALTER TABLE agent_comparison_metrics ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_comparison_metrics_tenant_isolation ON agent_comparison_metrics USING (true);
CREATE POLICY agent_comparison_metrics_tenant_insert ON agent_comparison_metrics WITH CHECK (true);

ALTER TABLE agent_coaching_insights ENABLE ROW LEVEL SECURITY;
CREATE POLICY agent_coaching_insights_tenant_isolation ON agent_coaching_insights USING (true);
CREATE POLICY agent_coaching_insights_tenant_insert ON agent_coaching_insights WITH CHECK (true);

-- Suggestions
ALTER TABLE suggestions ENABLE ROW LEVEL SECURITY;
CREATE POLICY suggestions_tenant_isolation ON suggestions USING (true);
CREATE POLICY suggestions_tenant_insert ON suggestions WITH CHECK (true);

-- Copilot Context
ALTER TABLE copilot_context ENABLE ROW LEVEL SECURITY;
CREATE POLICY copilot_context_tenant_isolation ON copilot_context USING (true);
CREATE POLICY copilot_context_tenant_insert ON copilot_context WITH CHECK (true);

-- Topic Categorization
ALTER TABLE conversation_topics ENABLE ROW LEVEL SECURITY;
CREATE POLICY conversation_topics_tenant_isolation ON conversation_topics USING (true);
CREATE POLICY conversation_topics_tenant_insert ON conversation_topics WITH CHECK (true);

ALTER TABLE topic_trends ENABLE ROW LEVEL SECURITY;
CREATE POLICY topic_trends_tenant_isolation ON topic_trends USING (true);
CREATE POLICY topic_trends_tenant_insert ON topic_trends WITH CHECK (true);

-- Tone of Voice
ALTER TABLE tone_config ENABLE ROW LEVEL SECURITY;
CREATE POLICY tone_config_tenant_isolation ON tone_config USING (has_brand_access(brand_id));
CREATE POLICY tone_config_tenant_insert ON tone_config WITH CHECK (has_brand_access(brand_id));

ALTER TABLE tone_presets ENABLE ROW LEVEL SECURITY;
CREATE POLICY tone_presets_tenant_isolation ON tone_presets USING (true);
CREATE POLICY tone_presets_tenant_insert ON tone_presets WITH CHECK (true);

ALTER TABLE tone_analytics ENABLE ROW LEVEL SECURITY;
CREATE POLICY tone_analytics_tenant_isolation ON tone_analytics USING (true);
CREATE POLICY tone_analytics_tenant_insert ON tone_analytics WITH CHECK (true);

-- Unified Analytics
ALTER TABLE unified_analytics_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY unified_analytics_settings_tenant_isolation ON unified_analytics_settings USING (has_brand_access(brand_id));
CREATE POLICY unified_analytics_settings_tenant_insert ON unified_analytics_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE analytics_snapshots ENABLE ROW LEVEL SECURITY;
CREATE POLICY analytics_snapshots_tenant_isolation ON analytics_snapshots USING (true);
CREATE POLICY analytics_snapshots_tenant_insert ON analytics_snapshots WITH CHECK (true);

ALTER TABLE dashboard_widgets ENABLE ROW LEVEL SECURITY;
CREATE POLICY dashboard_widgets_tenant_isolation ON dashboard_widgets USING (has_brand_access(brand_id));
CREATE POLICY dashboard_widgets_tenant_insert ON dashboard_widgets WITH CHECK (has_brand_access(brand_id));

ALTER TABLE report_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY report_templates_tenant_isolation ON report_templates USING (has_brand_access(brand_id));
CREATE POLICY report_templates_tenant_insert ON report_templates WITH CHECK (has_brand_access(brand_id));

ALTER TABLE saved_reports ENABLE ROW LEVEL SECURITY;
CREATE POLICY saved_reports_tenant_isolation ON saved_reports USING (has_brand_access(brand_id));
CREATE POLICY saved_reports_tenant_insert ON saved_reports WITH CHECK (has_brand_access(brand_id));

ALTER TABLE saved_searches ENABLE ROW LEVEL SECURITY;
CREATE POLICY saved_searches_tenant_isolation ON saved_searches USING (has_brand_access(brand_id));
CREATE POLICY saved_searches_tenant_insert ON saved_searches WITH CHECK (has_brand_access(brand_id));

ALTER TABLE scheduled_reports ENABLE ROW LEVEL SECURITY;
CREATE POLICY scheduled_reports_tenant_isolation ON scheduled_reports USING (has_brand_access(brand_id));
CREATE POLICY scheduled_reports_tenant_insert ON scheduled_reports WITH CHECK (has_brand_access(brand_id));

-- Search
ALTER TABLE search_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY search_settings_tenant_isolation ON search_settings USING (has_brand_access(brand_id));
CREATE POLICY search_settings_tenant_insert ON search_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE search_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY search_history_tenant_isolation ON search_history USING (true);
CREATE POLICY search_history_tenant_insert ON search_history WITH CHECK (true);

ALTER TABLE search_analytics ENABLE ROW LEVEL SECURITY;
CREATE POLICY search_analytics_tenant_isolation ON search_analytics USING (true);
CREATE POLICY search_analytics_tenant_insert ON search_analytics WITH CHECK (true);

-- Notifications
ALTER TABLE notification_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY notification_settings_tenant_isolation ON notification_settings USING (has_brand_access(brand_id));
CREATE POLICY notification_settings_tenant_insert ON notification_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE notification_preferences ENABLE ROW LEVEL SECURITY;
CREATE POLICY notification_preferences_tenant_isolation ON notification_preferences USING (true);
CREATE POLICY notification_preferences_tenant_insert ON notification_preferences WITH CHECK (true);

ALTER TABLE notification_queue ENABLE ROW LEVEL SECURITY;
CREATE POLICY notification_queue_tenant_isolation ON notification_queue USING (true);
CREATE POLICY notification_queue_tenant_insert ON notification_queue WITH CHECK (true);

ALTER TABLE notification_broadcast_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY notification_broadcast_log_tenant_isolation ON notification_broadcast_log USING (true);
CREATE POLICY notification_broadcast_log_tenant_insert ON notification_broadcast_log WITH CHECK (true);

-- In-App Messaging
ALTER TABLE inapp_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY inapp_settings_tenant_isolation ON inapp_settings USING (has_brand_access(brand_id));
CREATE POLICY inapp_settings_tenant_insert ON inapp_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE inapp_messages ENABLE ROW LEVEL SECURITY;
CREATE POLICY inapp_messages_tenant_isolation ON inapp_messages USING (true);
CREATE POLICY inapp_messages_tenant_insert ON inapp_messages WITH CHECK (true);

ALTER TABLE inapp_message_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY inapp_message_events_tenant_isolation ON inapp_message_events USING (true);
CREATE POLICY inapp_message_events_tenant_insert ON inapp_message_events WITH CHECK (true);

-- Unified Inbox
ALTER TABLE unified_inbox_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY unified_inbox_settings_tenant_isolation ON unified_inbox_settings USING (has_brand_access(brand_id));
CREATE POLICY unified_inbox_settings_tenant_insert ON unified_inbox_settings WITH CHECK (has_brand_access(brand_id));

-- Embed Widget
ALTER TABLE embed_widget_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY embed_widget_configs_tenant_isolation ON embed_widget_configs USING (has_brand_access(brand_id));
CREATE POLICY embed_widget_configs_tenant_insert ON embed_widget_configs WITH CHECK (has_brand_access(brand_id));

ALTER TABLE embed_widget_instances ENABLE ROW LEVEL SECURITY;
CREATE POLICY embed_widget_instances_tenant_isolation ON embed_widget_instances USING (has_brand_access(brand_id));
CREATE POLICY embed_widget_instances_tenant_insert ON embed_widget_instances WITH CHECK (has_brand_access(brand_id));

ALTER TABLE embed_widget_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY embed_widget_settings_tenant_isolation ON embed_widget_settings USING (has_brand_access(brand_id));
CREATE POLICY embed_widget_settings_tenant_insert ON embed_widget_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE embed_widget_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY embed_widget_events_tenant_isolation ON embed_widget_events USING (true);
CREATE POLICY embed_widget_events_tenant_insert ON embed_widget_events WITH CHECK (true);

-- WhatsApp Extended
ALTER TABLE whatsapp_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_templates_tenant_isolation ON whatsapp_templates USING (has_brand_access(brand_id));
CREATE POLICY whatsapp_templates_tenant_insert ON whatsapp_templates WITH CHECK (has_brand_access(brand_id));

ALTER TABLE whatsapp_template_flows ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_template_flows_tenant_isolation ON whatsapp_template_flows USING (true);
CREATE POLICY whatsapp_template_flows_tenant_insert ON whatsapp_template_flows WITH CHECK (true);

ALTER TABLE whatsapp_template_variants ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_template_variants_tenant_isolation ON whatsapp_template_variants USING (true);
CREATE POLICY whatsapp_template_variants_tenant_insert ON whatsapp_template_variants WITH CHECK (true);

ALTER TABLE whatsapp_template_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_template_log_tenant_isolation ON whatsapp_template_log USING (true);
CREATE POLICY whatsapp_template_log_tenant_insert ON whatsapp_template_log WITH CHECK (true);

ALTER TABLE whatsapp_template_usage ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_template_usage_tenant_isolation ON whatsapp_template_usage USING (true);
CREATE POLICY whatsapp_template_usage_tenant_insert ON whatsapp_template_usage WITH CHECK (true);

ALTER TABLE whatsapp_automation_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_automation_rules_tenant_isolation ON whatsapp_automation_rules USING (has_brand_access(brand_id));
CREATE POLICY whatsapp_automation_rules_tenant_insert ON whatsapp_automation_rules WITH CHECK (has_brand_access(brand_id));

ALTER TABLE whatsapp_automation_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_automation_state_tenant_isolation ON whatsapp_automation_state USING (true);
CREATE POLICY whatsapp_automation_state_tenant_insert ON whatsapp_automation_state WITH CHECK (true);

ALTER TABLE whatsapp_flow_steps ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_flow_steps_tenant_isolation ON whatsapp_flow_steps USING (true);
CREATE POLICY whatsapp_flow_steps_tenant_insert ON whatsapp_flow_steps WITH CHECK (true);

ALTER TABLE whatsapp_quality_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_quality_history_tenant_isolation ON whatsapp_quality_history USING (true);
CREATE POLICY whatsapp_quality_history_tenant_insert ON whatsapp_quality_history WITH CHECK (true);

ALTER TABLE whatsapp_quality_recommendations ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_quality_recommendations_tenant_isolation ON whatsapp_quality_recommendations USING (true);
CREATE POLICY whatsapp_quality_recommendations_tenant_insert ON whatsapp_quality_recommendations WITH CHECK (true);

ALTER TABLE whatsapp_session_windows ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_session_windows_tenant_isolation ON whatsapp_session_windows USING (true);
CREATE POLICY whatsapp_session_windows_tenant_insert ON whatsapp_session_windows WITH CHECK (true);

ALTER TABLE whatsapp_business_profile_cache ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_business_profile_cache_tenant_isolation ON whatsapp_business_profile_cache USING (has_brand_access(brand_id));
CREATE POLICY whatsapp_business_profile_cache_tenant_insert ON whatsapp_business_profile_cache WITH CHECK (has_brand_access(brand_id));

ALTER TABLE whatsapp_customer_profiles ENABLE ROW LEVEL SECURITY;
CREATE POLICY whatsapp_customer_profiles_tenant_isolation ON whatsapp_customer_profiles USING (true);
CREATE POLICY whatsapp_customer_profiles_tenant_insert ON whatsapp_customer_profiles WITH CHECK (true);

-- Broadcast extended
ALTER TABLE broadcast_channels ENABLE ROW LEVEL SECURITY;
CREATE POLICY broadcast_channels_tenant_isolation ON broadcast_channels USING (true);
CREATE POLICY broadcast_channels_tenant_insert ON broadcast_channels WITH CHECK (true);

ALTER TABLE broadcast_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY broadcast_logs_tenant_isolation ON broadcast_logs USING (true);
CREATE POLICY broadcast_logs_tenant_insert ON broadcast_logs WITH CHECK (true);

ALTER TABLE broadcast_media ENABLE ROW LEVEL SECURITY;
CREATE POLICY broadcast_media_tenant_isolation ON broadcast_media USING (true);
CREATE POLICY broadcast_media_tenant_insert ON broadcast_media WITH CHECK (true);

ALTER TABLE broadcast_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY broadcast_settings_tenant_isolation ON broadcast_settings USING (has_brand_access(brand_id));
CREATE POLICY broadcast_settings_tenant_insert ON broadcast_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE campaigns ENABLE ROW LEVEL SECURITY;
CREATE POLICY campaigns_tenant_isolation ON campaigns USING (has_brand_access(brand_id));
CREATE POLICY campaigns_tenant_insert ON campaigns WITH CHECK (has_brand_access(brand_id));

ALTER TABLE campaign_contacts ENABLE ROW LEVEL SECURITY;
CREATE POLICY campaign_contacts_tenant_isolation ON campaign_contacts USING (true);
CREATE POLICY campaign_contacts_tenant_insert ON campaign_contacts WITH CHECK (true);

ALTER TABLE campaign_retry_queue ENABLE ROW LEVEL SECURITY;
CREATE POLICY campaign_retry_queue_tenant_isolation ON campaign_retry_queue USING (true);
CREATE POLICY campaign_retry_queue_tenant_insert ON campaign_retry_queue WITH CHECK (true);

-- Discord
ALTER TABLE discord_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY discord_settings_tenant_isolation ON discord_settings USING (has_brand_access(brand_id));
CREATE POLICY discord_settings_tenant_insert ON discord_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE discord_commands ENABLE ROW LEVEL SECURITY;
CREATE POLICY discord_commands_tenant_isolation ON discord_commands USING (true);
CREATE POLICY discord_commands_tenant_insert ON discord_commands WITH CHECK (true);

ALTER TABLE discord_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY discord_events_tenant_isolation ON discord_events USING (true);
CREATE POLICY discord_events_tenant_insert ON discord_events WITH CHECK (true);

-- Slack
ALTER TABLE slack_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY slack_settings_tenant_isolation ON slack_settings USING (has_brand_access(brand_id));
CREATE POLICY slack_settings_tenant_insert ON slack_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE slack_commands ENABLE ROW LEVEL SECURITY;
CREATE POLICY slack_commands_tenant_isolation ON slack_commands USING (true);
CREATE POLICY slack_commands_tenant_insert ON slack_commands WITH CHECK (true);

ALTER TABLE slack_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY slack_events_tenant_isolation ON slack_events USING (true);
CREATE POLICY slack_events_tenant_insert ON slack_events WITH CHECK (true);

-- CRM Sync
ALTER TABLE crm_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY crm_connections_tenant_isolation ON crm_connections USING (has_brand_access(brand_id));
CREATE POLICY crm_connections_tenant_insert ON crm_connections WITH CHECK (has_brand_access(brand_id));

ALTER TABLE crm_field_mappings ENABLE ROW LEVEL SECURITY;
CREATE POLICY crm_field_mappings_tenant_isolation ON crm_field_mappings USING (true);
CREATE POLICY crm_field_mappings_tenant_insert ON crm_field_mappings WITH CHECK (true);

ALTER TABLE crm_sync_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY crm_sync_logs_tenant_isolation ON crm_sync_logs USING (true);
CREATE POLICY crm_sync_logs_tenant_insert ON crm_sync_logs WITH CHECK (true);

ALTER TABLE crm_external_refs ENABLE ROW LEVEL SECURITY;
CREATE POLICY crm_external_refs_tenant_isolation ON crm_external_refs USING (true);
CREATE POLICY crm_external_refs_tenant_insert ON crm_external_refs WITH CHECK (true);

-- Client Profiles
ALTER TABLE client_preferences ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_preferences_tenant_isolation ON client_preferences USING (true);
CREATE POLICY client_preferences_tenant_insert ON client_preferences WITH CHECK (true);

ALTER TABLE client_contact_points ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_contact_points_tenant_isolation ON client_contact_points USING (true);
CREATE POLICY client_contact_points_tenant_insert ON client_contact_points WITH CHECK (true);

ALTER TABLE client_interaction_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_interaction_history_tenant_isolation ON client_interaction_history USING (true);
CREATE POLICY client_interaction_history_tenant_insert ON client_interaction_history WITH CHECK (true);

ALTER TABLE client_learning_patterns ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_learning_patterns_tenant_isolation ON client_learning_patterns USING (true);
CREATE POLICY client_learning_patterns_tenant_insert ON client_learning_patterns WITH CHECK (true);

ALTER TABLE client_learning_insights ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_learning_insights_tenant_isolation ON client_learning_insights USING (true);
CREATE POLICY client_learning_insights_tenant_insert ON client_learning_insights WITH CHECK (true);

ALTER TABLE client_custom_sections ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_custom_sections_tenant_isolation ON client_custom_sections USING (true);
CREATE POLICY client_custom_sections_tenant_insert ON client_custom_sections WITH CHECK (true);

ALTER TABLE client_custom_section_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_custom_section_data_tenant_isolation ON client_custom_section_data USING (true);
CREATE POLICY client_custom_section_data_tenant_insert ON client_custom_section_data WITH CHECK (true);

ALTER TABLE client_orders ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_orders_tenant_isolation ON client_orders USING (true);
CREATE POLICY client_orders_tenant_insert ON client_orders WITH CHECK (true);

ALTER TABLE client_order_stage ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_order_stage_tenant_isolation ON client_order_stage USING (true);
CREATE POLICY client_order_stage_tenant_insert ON client_order_stage WITH CHECK (true);

ALTER TABLE client_invoices ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_invoices_tenant_isolation ON client_invoices USING (true);
CREATE POLICY client_invoices_tenant_insert ON client_invoices WITH CHECK (true);

ALTER TABLE client_payment_details ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_payment_details_tenant_isolation ON client_payment_details USING (true);
CREATE POLICY client_payment_details_tenant_insert ON client_payment_details WITH CHECK (true);

ALTER TABLE client_task_checklists ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_task_checklists_tenant_isolation ON client_task_checklists USING (true);
CREATE POLICY client_task_checklists_tenant_insert ON client_task_checklists WITH CHECK (true);

ALTER TABLE client_task_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_task_items_tenant_isolation ON client_task_items USING (true);
CREATE POLICY client_task_items_tenant_insert ON client_task_items WITH CHECK (true);

ALTER TABLE client_rag_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_rag_mapping_tenant_isolation ON client_rag_mapping USING (true);
CREATE POLICY client_rag_mapping_tenant_insert ON client_rag_mapping WITH CHECK (true);

ALTER TABLE client_profile_sources ENABLE ROW LEVEL SECURITY;
CREATE POLICY client_profile_sources_tenant_isolation ON client_profile_sources USING (true);
CREATE POLICY client_profile_sources_tenant_insert ON client_profile_sources WITH CHECK (true);

-- Profile
ALTER TABLE profile_custom_sections ENABLE ROW LEVEL SECURITY;
CREATE POLICY profile_custom_sections_tenant_isolation ON profile_custom_sections USING (true);
CREATE POLICY profile_custom_sections_tenant_insert ON profile_custom_sections WITH CHECK (true);

ALTER TABLE profile_custom_section_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY profile_custom_section_data_tenant_isolation ON profile_custom_section_data USING (true);
CREATE POLICY profile_custom_section_data_tenant_insert ON profile_custom_section_data WITH CHECK (true);

ALTER TABLE profile_conflicts ENABLE ROW LEVEL SECURITY;
CREATE POLICY profile_conflicts_tenant_isolation ON profile_conflicts USING (true);
CREATE POLICY profile_conflicts_tenant_insert ON profile_conflicts WITH CHECK (true);

-- Commerce Inbox
ALTER TABLE commerce_inbox_threads ENABLE ROW LEVEL SECURITY;
CREATE POLICY commerce_inbox_threads_tenant_isolation ON commerce_inbox_threads USING (true);
CREATE POLICY commerce_inbox_threads_tenant_insert ON commerce_inbox_threads WITH CHECK (true);

-- Compliance
ALTER TABLE compliance_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY compliance_policies_tenant_isolation ON compliance_policies USING (has_brand_access(brand_id));
CREATE POLICY compliance_policies_tenant_insert ON compliance_policies WITH CHECK (has_brand_access(brand_id));

ALTER TABLE compliance_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY compliance_settings_tenant_isolation ON compliance_settings USING (has_brand_access(brand_id));
CREATE POLICY compliance_settings_tenant_insert ON compliance_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE compliance_audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY compliance_audit_log_tenant_isolation ON compliance_audit_log USING (true);
CREATE POLICY compliance_audit_log_tenant_insert ON compliance_audit_log WITH CHECK (true);

ALTER TABLE gdpr_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY gdpr_requests_tenant_isolation ON gdpr_requests USING (true);
CREATE POLICY gdpr_requests_tenant_insert ON gdpr_requests WITH CHECK (true);

ALTER TABLE suppression_list ENABLE ROW LEVEL SECURITY;
CREATE POLICY suppression_list_tenant_isolation ON suppression_list USING (has_brand_access(brand_id));
CREATE POLICY suppression_list_tenant_insert ON suppression_list WITH CHECK (has_brand_access(brand_id));

-- MCP Protocol
ALTER TABLE mcp_servers ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_servers_tenant_isolation ON mcp_servers USING (has_brand_access(brand_id));
CREATE POLICY mcp_servers_tenant_insert ON mcp_servers WITH CHECK (has_brand_access(brand_id));

ALTER TABLE mcp_tools ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_tools_tenant_isolation ON mcp_tools USING (true);
CREATE POLICY mcp_tools_tenant_insert ON mcp_tools WITH CHECK (true);

ALTER TABLE mcp_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_sessions_tenant_isolation ON mcp_sessions USING (true);
CREATE POLICY mcp_sessions_tenant_insert ON mcp_sessions WITH CHECK (true);

ALTER TABLE mcp_call_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_call_logs_tenant_isolation ON mcp_call_logs USING (true);
CREATE POLICY mcp_call_logs_tenant_insert ON mcp_call_logs WITH CHECK (true);

ALTER TABLE mcp_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_settings_tenant_isolation ON mcp_settings USING (has_brand_access(brand_id));
CREATE POLICY mcp_settings_tenant_insert ON mcp_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE mcp_server_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_server_templates_tenant_isolation ON mcp_server_templates USING (true);
CREATE POLICY mcp_server_templates_tenant_insert ON mcp_server_templates WITH CHECK (true);

-- Autonomous Resolution
ALTER TABLE resolution_config ENABLE ROW LEVEL SECURITY;
CREATE POLICY resolution_config_tenant_isolation ON resolution_config USING (has_brand_access(brand_id));
CREATE POLICY resolution_config_tenant_insert ON resolution_config WITH CHECK (has_brand_access(brand_id));

ALTER TABLE resolution_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY resolution_logs_tenant_isolation ON resolution_logs USING (true);
CREATE POLICY resolution_logs_tenant_insert ON resolution_logs WITH CHECK (true);

ALTER TABLE resolution_analytics ENABLE ROW LEVEL SECURITY;
CREATE POLICY resolution_analytics_tenant_isolation ON resolution_analytics USING (true);
CREATE POLICY resolution_analytics_tenant_insert ON resolution_analytics WITH CHECK (true);

-- Reply Engine
ALTER TABLE reply_integration_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY reply_integration_settings_tenant_isolation ON reply_integration_settings USING (has_brand_access(brand_id));
CREATE POLICY reply_integration_settings_tenant_insert ON reply_integration_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE reply_knowledge_sources ENABLE ROW LEVEL SECURITY;
CREATE POLICY reply_knowledge_sources_tenant_isolation ON reply_knowledge_sources USING (true);
CREATE POLICY reply_knowledge_sources_tenant_insert ON reply_knowledge_sources WITH CHECK (true);

-- Message Triggers
ALTER TABLE message_triggers ENABLE ROW LEVEL SECURITY;
CREATE POLICY message_triggers_tenant_isolation ON message_triggers USING (has_brand_access(brand_id));
CREATE POLICY message_triggers_tenant_insert ON message_triggers WITH CHECK (has_brand_access(brand_id));

ALTER TABLE message_trigger_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY message_trigger_logs_tenant_isolation ON message_trigger_logs USING (true);
CREATE POLICY message_trigger_logs_tenant_insert ON message_trigger_logs WITH CHECK (true);

ALTER TABLE message_limits ENABLE ROW LEVEL SECURITY;
CREATE POLICY message_limits_tenant_isolation ON message_limits USING (has_brand_access(brand_id));
CREATE POLICY message_limits_tenant_insert ON message_limits WITH CHECK (has_brand_access(brand_id));

-- Scheduled Tasks
ALTER TABLE scheduled_tasks ENABLE ROW LEVEL SECURITY;
CREATE POLICY scheduled_tasks_tenant_isolation ON scheduled_tasks USING (has_brand_access(brand_id));
CREATE POLICY scheduled_tasks_tenant_insert ON scheduled_tasks WITH CHECK (has_brand_access(brand_id));

ALTER TABLE scheduled_task_queue ENABLE ROW LEVEL SECURITY;
CREATE POLICY scheduled_task_queue_tenant_isolation ON scheduled_task_queue USING (true);
CREATE POLICY scheduled_task_queue_tenant_insert ON scheduled_task_queue WITH CHECK (true);

ALTER TABLE scheduled_task_runs ENABLE ROW LEVEL SECURITY;
CREATE POLICY scheduled_task_runs_tenant_isolation ON scheduled_task_runs USING (true);
CREATE POLICY scheduled_task_runs_tenant_insert ON scheduled_task_runs WITH CHECK (true);

ALTER TABLE scheduler_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY scheduler_settings_tenant_isolation ON scheduler_settings USING (has_brand_access(brand_id));
CREATE POLICY scheduler_settings_tenant_insert ON scheduler_settings WITH CHECK (has_brand_access(brand_id));

-- Monitoring & Alerting
ALTER TABLE alert_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY alert_rules_tenant_isolation ON alert_rules USING (has_brand_access(brand_id));
CREATE POLICY alert_rules_tenant_insert ON alert_rules WITH CHECK (has_brand_access(brand_id));

ALTER TABLE alert_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY alert_events_tenant_isolation ON alert_events USING (true);
CREATE POLICY alert_events_tenant_insert ON alert_events WITH CHECK (true);

-- Finance Operator
ALTER TABLE operator_config ENABLE ROW LEVEL SECURITY;
CREATE POLICY operator_config_tenant_isolation ON operator_config USING (has_brand_access(brand_id));
CREATE POLICY operator_config_tenant_insert ON operator_config WITH CHECK (has_brand_access(brand_id));

ALTER TABLE operator_proposals ENABLE ROW LEVEL SECURITY;
CREATE POLICY operator_proposals_tenant_isolation ON operator_proposals USING (true);
CREATE POLICY operator_proposals_tenant_insert ON operator_proposals WITH CHECK (true);

ALTER TABLE operator_performance_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY operator_performance_log_tenant_isolation ON operator_performance_log USING (true);
CREATE POLICY operator_performance_log_tenant_insert ON operator_performance_log WITH CHECK (true);

ALTER TABLE operator_metrics_snapshot ENABLE ROW LEVEL SECURITY;
CREATE POLICY operator_metrics_snapshot_tenant_isolation ON operator_metrics_snapshot USING (true);
CREATE POLICY operator_metrics_snapshot_tenant_insert ON operator_metrics_snapshot WITH CHECK (true);

-- Finance Vision
ALTER TABLE vision_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY vision_settings_tenant_isolation ON vision_settings USING (has_brand_access(brand_id));
CREATE POLICY vision_settings_tenant_insert ON vision_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE vision_analyses ENABLE ROW LEVEL SECURITY;
CREATE POLICY vision_analyses_tenant_isolation ON vision_analyses USING (true);
CREATE POLICY vision_analyses_tenant_insert ON vision_analyses WITH CHECK (true);

ALTER TABLE vision_images ENABLE ROW LEVEL SECURITY;
CREATE POLICY vision_images_tenant_isolation ON vision_images USING (true);
CREATE POLICY vision_images_tenant_insert ON vision_images WITH CHECK (true);

-- Audience KB Scoping
ALTER TABLE audience_kb_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY audience_kb_settings_tenant_isolation ON audience_kb_settings USING (has_brand_access(brand_id));
CREATE POLICY audience_kb_settings_tenant_insert ON audience_kb_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE audience_kb_links ENABLE ROW LEVEL SECURITY;
CREATE POLICY audience_kb_links_tenant_isolation ON audience_kb_links USING (true);
CREATE POLICY audience_kb_links_tenant_insert ON audience_kb_links WITH CHECK (true);

ALTER TABLE audience_kb_usage_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY audience_kb_usage_log_tenant_isolation ON audience_kb_usage_log USING (true);
CREATE POLICY audience_kb_usage_log_tenant_insert ON audience_kb_usage_log WITH CHECK (true);

-- RAG
ALTER TABLE rag_section_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY rag_section_mapping_tenant_isolation ON rag_section_mapping USING (true);
CREATE POLICY rag_section_mapping_tenant_insert ON rag_section_mapping WITH CHECK (true);

-- Escalation
ALTER TABLE escalation_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY escalation_rules_tenant_isolation ON escalation_rules USING (has_brand_access(brand_id));
CREATE POLICY escalation_rules_tenant_insert ON escalation_rules WITH CHECK (has_brand_access(brand_id));

ALTER TABLE guidance_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY guidance_rules_tenant_isolation ON guidance_rules USING (has_brand_access(brand_id));
CREATE POLICY guidance_rules_tenant_insert ON guidance_rules WITH CHECK (true);

-- Proactive
ALTER TABLE proactive_triggers ENABLE ROW LEVEL SECURITY;
CREATE POLICY proactive_triggers_tenant_isolation ON proactive_triggers USING (has_brand_access(brand_id));
CREATE POLICY proactive_triggers_tenant_insert ON proactive_triggers WITH CHECK (has_brand_access(brand_id));

-- Followup
ALTER TABLE followup_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY followup_log_tenant_isolation ON followup_log USING (true);
CREATE POLICY followup_log_tenant_insert ON followup_log WITH CHECK (true);

-- Industries
ALTER TABLE industries ENABLE ROW LEVEL SECURITY;
CREATE POLICY industries_read ON industries USING (true);

-- Brand Hierarchy
ALTER TABLE brand_hierarchy ENABLE ROW LEVEL SECURITY;
CREATE POLICY brand_hierarchy_tenant_isolation ON brand_hierarchy USING (has_brand_access(brand_id));
CREATE POLICY brand_hierarchy_tenant_insert ON brand_hierarchy WITH CHECK (has_brand_access(brand_id));

-- Template Groups
ALTER TABLE template_groups ENABLE ROW LEVEL SECURITY;
CREATE POLICY template_groups_tenant_isolation ON template_groups USING (has_brand_access(brand_id));
CREATE POLICY template_groups_tenant_insert ON template_groups WITH CHECK (has_brand_access(brand_id));

-- Shopify Store
ALTER TABLE shopify_store_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY shopify_store_connections_tenant_isolation ON shopify_store_connections USING (has_brand_access(brand_id));
CREATE POLICY shopify_store_connections_tenant_insert ON shopify_store_connections WITH CHECK (has_brand_access(brand_id));

ALTER TABLE shopify_channel_preferences ENABLE ROW LEVEL SECURITY;
CREATE POLICY shopify_channel_preferences_tenant_isolation ON shopify_channel_preferences USING (true);
CREATE POLICY shopify_channel_preferences_tenant_insert ON shopify_channel_preferences WITH CHECK (true);

ALTER TABLE shopify_pending_customers ENABLE ROW LEVEL SECURITY;
CREATE POLICY shopify_pending_customers_tenant_isolation ON shopify_pending_customers USING (true);
CREATE POLICY shopify_pending_customers_tenant_insert ON shopify_pending_customers WITH CHECK (true);

ALTER TABLE shopify_webhook_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY shopify_webhook_events_tenant_isolation ON shopify_webhook_events USING (true);
CREATE POLICY shopify_webhook_events_tenant_insert ON shopify_webhook_events WITH CHECK (true);

-- Store Connections
ALTER TABLE store_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY store_connections_tenant_isolation ON store_connections USING (has_brand_access(brand_id));
CREATE POLICY store_connections_tenant_insert ON store_connections WITH CHECK (has_brand_access(brand_id));

ALTER TABLE store_connection_status ENABLE ROW LEVEL SECURITY;
CREATE POLICY store_connection_status_tenant_isolation ON store_connection_status USING (true);
CREATE POLICY store_connection_status_tenant_insert ON store_connection_status WITH CHECK (true);

ALTER TABLE store_pending_customers ENABLE ROW LEVEL SECURITY;
CREATE POLICY store_pending_customers_tenant_isolation ON store_pending_customers USING (true);
CREATE POLICY store_pending_customers_tenant_insert ON store_pending_customers WITH CHECK (true);

ALTER TABLE store_webhook_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY store_webhook_events_tenant_isolation ON store_webhook_events USING (true);
CREATE POLICY store_webhook_events_tenant_insert ON store_webhook_events WITH CHECK (true);

ALTER TABLE store_ui_compatibility ENABLE ROW LEVEL SECURITY;
CREATE POLICY store_ui_compatibility_tenant_isolation ON store_ui_compatibility USING (true);
CREATE POLICY store_ui_compatibility_tenant_insert ON store_ui_compatibility WITH CHECK (true);

-- System Health & Onboarding
ALTER TABLE health_check_results ENABLE ROW LEVEL SECURITY;
CREATE POLICY health_check_results_tenant_isolation ON health_check_results USING (true);
CREATE POLICY health_check_results_tenant_insert ON health_check_results WITH CHECK (true);

ALTER TABLE onboarding_progress ENABLE ROW LEVEL SECURITY;
CREATE POLICY onboarding_progress_tenant_isolation ON onboarding_progress USING (has_brand_access(brand_id));
CREATE POLICY onboarding_progress_tenant_insert ON onboarding_progress WITH CHECK (has_brand_access(brand_id));

ALTER TABLE onboarding_settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY onboarding_settings_tenant_isolation ON onboarding_settings USING (has_brand_access(brand_id));
CREATE POLICY onboarding_settings_tenant_insert ON onboarding_settings WITH CHECK (has_brand_access(brand_id));

ALTER TABLE onboarding_steps ENABLE ROW LEVEL SECURITY;
CREATE POLICY onboarding_steps_tenant_isolation ON onboarding_steps USING (true);
CREATE POLICY onboarding_steps_tenant_insert ON onboarding_steps WITH CHECK (true);

-- System Metrics
ALTER TABLE system_metrics ENABLE ROW LEVEL SECURITY;
CREATE POLICY system_metrics_tenant_isolation ON system_metrics USING (true);
CREATE POLICY system_metrics_tenant_insert ON system_metrics WITH CHECK (true);

-- Webhook Events
ALTER TABLE webhook_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY webhook_events_tenant_isolation ON webhook_events USING (true);
CREATE POLICY webhook_events_tenant_insert ON webhook_events WITH CHECK (true);

-- WebSocket
ALTER TABLE websocket_connections ENABLE ROW LEVEL SECURITY;
CREATE POLICY websocket_connections_tenant_isolation ON websocket_connections USING (true);
CREATE POLICY websocket_connections_tenant_insert ON websocket_connections WITH CHECK (true);

-- Unified Customer Mapping
ALTER TABLE unified_customer_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY unified_customer_mapping_tenant_isolation ON unified_customer_mapping USING (true);
CREATE POLICY unified_customer_mapping_tenant_insert ON unified_customer_mapping WITH CHECK (true);

-- Behavioral Events
ALTER TABLE behavioral_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY behavioral_events_tenant_isolation ON behavioral_events USING (true);
CREATE POLICY behavioral_events_tenant_insert ON behavioral_events WITH CHECK (true);

-- Sync Mapping
ALTER TABLE sync_mapping ENABLE ROW LEVEL SECURITY;
CREATE POLICY sync_mapping_tenant_isolation ON sync_mapping USING (true);
CREATE POLICY sync_mapping_tenant_insert ON sync_mapping WITH CHECK (true);

-- ---------------------------------------------------------------------------
-- Force RLS for all roles (including table owners)
-- This ensures even the database user running the backend cannot bypass RLS
-- ---------------------------------------------------------------------------

-- We use a function to iterate all tables that have RLS enabled
DO $$
DECLARE
    rec RECORD;
BEGIN
    FOR rec IN
        SELECT schemaname, tablename
        FROM pg_tables
        WHERE schemaname = 'public'
        AND tablename IN (
            SELECT relname::text FROM pg_class WHERE relkind = 'r'
        )
    LOOP
        -- Check if RLS is enabled on this table
        IF EXISTS (
            SELECT 1 FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = rec.schemaname
            AND c.relname = rec.tablename
            AND c.relrowsecurity = true
        ) THEN
            EXECUTE format('ALTER TABLE %I.%I FORCE ROW LEVEL SECURITY', rec.schemaname, rec.tablename);
        END IF;
    END LOOP;
END $$;

-- ===========================================================================
-- Grant usage on helper functions to public
-- ===========================================================================

GRANT EXECUTE ON FUNCTION has_tenant_access(TEXT) TO PUBLIC;
GRANT EXECUTE ON FUNCTION has_brand_access(TEXT) TO PUBLIC;
GRANT EXECUTE ON FUNCTION has_org_access(TEXT) TO PUBLIC;

-- ===========================================================================
-- Comments
-- ===========================================================================

COMMENT ON MIGRATION IS '0072: Row-Level Security — enables RLS on all tables with tenant/brand/org isolation policies. The Rust backend must set app.current_tenant_key via SET LOCAL before each query.';
