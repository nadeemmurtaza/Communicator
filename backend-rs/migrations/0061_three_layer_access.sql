-- Migration 0061: Three-Layer Access Control System
-- Layer 1: Entity-level access (tenant/org/brand hierarchy)
-- Layer 2: Department-level access (inherited from tenant → org → brand)
-- Layer 3: Feature-level custom permissions (roles with granular permissions)

-- 1. Access Grants: Entity-level access linking team members to tenants/orgs/brands
CREATE TABLE IF NOT EXISTS access_grants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_member_id TEXT NOT NULL REFERENCES team_members(id) ON DELETE CASCADE,
    tenant_id TEXT NOT NULL REFERENCES nid_tenants(id) ON DELETE CASCADE,
    org_id UUID REFERENCES nid_organizations(id) ON DELETE CASCADE,
    brand_id UUID REFERENCES nid_brands(id) ON DELETE CASCADE,
    access_type VARCHAR(20) NOT NULL CHECK (access_type IN ('owner', 'partner', 'admin', 'member')),
    partnership_percentage NUMERIC(5,2),
    granted_by TEXT NOT NULL REFERENCES team_members(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked', 'pending')),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_member_id, tenant_id, org_id, brand_id)
);

CREATE INDEX IF NOT EXISTS idx_access_grants_member ON access_grants(team_member_id);
CREATE INDEX IF NOT EXISTS idx_access_grants_tenant ON access_grants(tenant_id);
CREATE INDEX IF NOT EXISTS idx_access_grants_org ON access_grants(org_id);
CREATE INDEX IF NOT EXISTS idx_access_grants_brand ON access_grants(brand_id);
CREATE INDEX IF NOT EXISTS idx_access_grants_status ON access_grants(status);

-- 2. Departments: Department definitions at tenant level
CREATE TABLE IF NOT EXISTS departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id TEXT NOT NULL REFERENCES nid_tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    default_permissions JSONB,
    icon VARCHAR(100),
    color VARCHAR(7),
    inherited_from_tenant BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_departments_tenant ON departments(tenant_id);

-- 3. Department Assignments: Links members to departments at specific scopes
CREATE TABLE IF NOT EXISTS department_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_member_id TEXT NOT NULL REFERENCES team_members(id) ON DELETE CASCADE,
    department_id UUID NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    tenant_id TEXT NOT NULL REFERENCES nid_tenants(id) ON DELETE CASCADE,
    org_id UUID REFERENCES nid_organizations(id) ON DELETE CASCADE,
    brand_id UUID REFERENCES nid_brands(id) ON DELETE CASCADE,
    scope VARCHAR(20) NOT NULL DEFAULT 'tenant' CHECK (scope IN ('tenant', 'organization', 'brand')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_member_id, department_id, scope, org_id, brand_id)
);

CREATE INDEX IF NOT EXISTS idx_dept_assignments_member ON department_assignments(team_member_id);
CREATE INDEX IF NOT EXISTS idx_dept_assignments_dept ON department_assignments(department_id);
CREATE INDEX IF NOT EXISTS idx_dept_assignments_tenant ON department_assignments(tenant_id);

-- 4. Role Assignments: Links members to roles at specific entity levels
CREATE TABLE IF NOT EXISTS role_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_member_id TEXT NOT NULL REFERENCES team_members(id) ON DELETE CASCADE,
    role_definition_id TEXT NOT NULL REFERENCES role_definitions(id) ON DELETE CASCADE,
    tenant_id TEXT NOT NULL REFERENCES nid_tenants(id) ON DELETE CASCADE,
    org_id UUID REFERENCES nid_organizations(id) ON DELETE CASCADE,
    brand_id UUID REFERENCES nid_brands(id) ON DELETE CASCADE,
    scope VARCHAR(20) NOT NULL DEFAULT 'tenant' CHECK (scope IN ('tenant', 'organization', 'brand')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_member_id, role_definition_id, scope, org_id, brand_id)
);

CREATE INDEX IF NOT EXISTS idx_role_assignments_member ON role_assignments(team_member_id);
CREATE INDEX IF NOT EXISTS idx_role_assignments_role ON role_assignments(role_definition_id);
CREATE INDEX IF NOT EXISTS idx_role_assignments_tenant ON role_assignments(tenant_id);

-- 5. Permission Catalog: Service groups, services, and permissions hierarchy
CREATE TABLE IF NOT EXISTS permission_catalog (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_name VARCHAR(255) NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    permission_key VARCHAR(500) NOT NULL,
    label VARCHAR(500) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (group_name, service_name, permission_key)
);

CREATE INDEX IF NOT EXISTS idx_permission_catalog_group ON permission_catalog(group_name);
CREATE INDEX IF NOT EXISTS idx_permission_catalog_service ON permission_catalog(service_name);
CREATE INDEX IF NOT EXISTS idx_permission_catalog_key ON permission_catalog(permission_key);

-- 6. Invitations: Invite records for email and username invites
CREATE TABLE IF NOT EXISTS invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id TEXT NOT NULL REFERENCES nid_tenants(id) ON DELETE CASCADE,
    email VARCHAR(500),
    username VARCHAR(500),
    invited_by TEXT NOT NULL REFERENCES team_members(id),
    org_id UUID REFERENCES nid_organizations(id) ON DELETE CASCADE,
    brand_id UUID REFERENCES nid_brands(id) ON DELETE CASCADE,
    scope VARCHAR(20) NOT NULL CHECK (scope IN ('tenant', 'organization', 'brand')),
    role_definition_id TEXT REFERENCES role_definitions(id) ON DELETE SET NULL,
    department_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'declined', 'expired', 'revoked')),
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    accepted_by TEXT REFERENCES team_members(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_invitations_email ON invitations(email);
CREATE INDEX IF NOT EXISTS idx_invitations_username ON invitations(username);
CREATE INDEX IF NOT EXISTS idx_invitations_invited_by ON invitations(invited_by);
CREATE INDEX IF NOT EXISTS idx_invitations_status ON invitations(status, expires_at);
CREATE INDEX IF NOT EXISTS idx_invitations_tenant ON invitations(tenant_id);

-- Seed Permission Catalog with ~260 permissions across 21 groups covering all 85+ services
INSERT INTO permission_catalog (group_name, service_name, permission_key, label, description) VALUES
-- 1. Inbox Group
('Inbox', 'omni-inbox', 'inbox:omni-inbox:view', 'View Omni Inbox', 'View all messages across channels'),
('Inbox', 'omni-inbox', 'inbox:omni-inbox:reply', 'Reply in Omni Inbox', 'Send replies from the unified inbox'),
('Inbox', 'omni-inbox', 'inbox:omni-inbox:assign', 'Assign in Omni Inbox', 'Assign conversations to team members'),
('Inbox', 'omni-inbox', 'inbox:omni-inbox:notes', 'Add Notes in Omni Inbox', 'Add internal notes to conversations'),
('Inbox', 'whatsapp-inbox', 'inbox:whatsapp-inbox:view', 'View WhatsApp Inbox', 'View WhatsApp messages'),
('Inbox', 'whatsapp-inbox', 'inbox:whatsapp-inbox:reply', 'Reply via WhatsApp', 'Send WhatsApp replies'),
('Inbox', 'sms-inbox', 'inbox:sms-inbox:view', 'View SMS Inbox', 'View SMS messages'),
('Inbox', 'sms-inbox', 'inbox:sms-inbox:reply', 'Reply via SMS', 'Send SMS replies'),
('Inbox', 'email-inbox', 'inbox:email-inbox:view', 'View Email Inbox', 'View email messages'),
('Inbox', 'email-inbox', 'inbox:email-inbox:reply', 'Reply via Email', 'Send email replies'),
('Inbox', 'live-chat-inbox', 'inbox:live-chat-inbox:view', 'View Live Chat', 'View live chat messages'),
('Inbox', 'live-chat-inbox', 'inbox:live-chat-inbox:reply', 'Reply via Live Chat', 'Send live chat replies'),
-- 2. Campaigns & Broadcasts Group
('Campaigns & Broadcasts', 'omni-broadcast', 'campaigns:omni-broadcast:view', 'View Broadcasts', 'View all broadcast campaigns'),
('Campaigns & Broadcasts', 'omni-broadcast', 'campaigns:omni-broadcast:create', 'Create Broadcasts', 'Create new broadcast campaigns'),
('Campaigns & Broadcasts', 'omni-broadcast', 'campaigns:omni-broadcast:send', 'Send Broadcasts', 'Send broadcast campaigns'),
('Campaigns & Broadcasts', 'omni-broadcast', 'campaigns:omni-broadcast:edit', 'Edit Broadcasts', 'Edit existing broadcasts'),
('Campaigns & Broadcasts', 'omni-broadcast', 'campaigns:omni-broadcast:delete', 'Delete Broadcasts', 'Delete broadcast campaigns'),
('Campaigns & Broadcasts', 'omni-broadcast', 'campaigns:omni-broadcast:analytics', 'Broadcast Analytics', 'View broadcast analytics'),
('Campaigns & Broadcasts', 'whatsapp-campaigns', 'campaigns:whatsapp-campaigns:view', 'View WhatsApp Campaigns', 'View WhatsApp campaigns'),
('Campaigns & Broadcasts', 'whatsapp-campaigns', 'campaigns:whatsapp-campaigns:create', 'Create WhatsApp Campaigns', 'Create WhatsApp campaigns'),
('Campaigns & Broadcasts', 'whatsapp-campaigns', 'campaigns:whatsapp-campaigns:send', 'Send WhatsApp Campaigns', 'Send WhatsApp campaigns'),
('Campaigns & Broadcasts', 'sms-campaigns', 'campaigns:sms-campaigns:view', 'View SMS Campaigns', 'View SMS campaigns'),
('Campaigns & Broadcasts', 'sms-campaigns', 'campaigns:sms-campaigns:create', 'Create SMS Campaigns', 'Create SMS campaigns'),
('Campaigns & Broadcasts', 'sms-campaigns', 'campaigns:sms-campaigns:send', 'Send SMS Campaigns', 'Send SMS campaigns'),
('Campaigns & Broadcasts', 'email-campaigns', 'campaigns:email-campaigns:view', 'View Email Campaigns', 'View email campaigns'),
('Campaigns & Broadcasts', 'email-campaigns', 'campaigns:email-campaigns:create', 'Create Email Campaigns', 'Create email campaigns'),
('Campaigns & Broadcasts', 'email-campaigns', 'campaigns:email-campaigns:send', 'Send Email Campaigns', 'Send email campaigns'),
-- 3. Commerce Group
('Commerce', 'catalog', 'commerce:catalog:view', 'View Catalog', 'View product catalog'),
('Commerce', 'catalog', 'commerce:catalog:edit', 'Edit Catalog', 'Edit products and categories'),
('Commerce', 'catalog', 'commerce:catalog:delete', 'Delete Catalog Items', 'Delete products from catalog'),
('Commerce', 'catalog', 'commerce:catalog:manage', 'Manage Catalog', 'Full catalog management'),
('Commerce', 'orders', 'commerce:orders:view', 'View Orders', 'View order details'),
('Commerce', 'orders', 'commerce:orders:edit', 'Edit Orders', 'Edit order details'),
('Commerce', 'orders', 'commerce:orders:process', 'Process Orders', 'Process and fulfill orders'),
('Commerce', 'orders', 'commerce:orders:refund', 'Refund Orders', 'Process order refunds'),
('Commerce', 'payments', 'commerce:payments:view', 'View Payments', 'View payment details'),
('Commerce', 'payments', 'commerce:payments:process', 'Process Payments', 'Process payments'),
('Commerce', 'payments', 'commerce:payments:refund', 'Refund Payments', 'Process payment refunds'),
('Commerce', 'inventory', 'commerce:inventory:view', 'View Inventory', 'View inventory levels'),
('Commerce', 'inventory', 'commerce:inventory:edit', 'Edit Inventory', 'Edit inventory levels'),
('Commerce', 'inventory', 'commerce:inventory:manage', 'Manage Inventory', 'Full inventory management'),
('Commerce', 'storefront', 'commerce:storefront:view', 'View Storefront', 'View storefront settings'),
('Commerce', 'storefront', 'commerce:storefront:configure', 'Configure Storefront', 'Configure storefront appearance'),
-- 4. CRM Group
('CRM', 'contacts', 'crm:contacts:view', 'View Contacts', 'View contact details'),
('CRM', 'contacts', 'crm:contacts:edit', 'Edit Contacts', 'Edit contact information'),
('CRM', 'contacts', 'crm:contacts:delete', 'Delete Contacts', 'Delete contacts'),
('CRM', 'contacts', 'crm:contacts:import', 'Import Contacts', 'Import contacts from files'),
('CRM', 'contacts', 'crm:contacts:export', 'Export Contacts', 'Export contacts to files'),
('CRM', 'client-profiles', 'crm:client-profiles:view', 'View Client Profiles', 'View client profiles'),
('CRM', 'client-profiles', 'crm:client-profiles:edit', 'Edit Client Profiles', 'Edit client profiles'),
('CRM', 'client-profiles', 'crm:client-profiles:manage', 'Manage Client Profiles', 'Full client profile management'),
-- 5. Analytics Group
('Analytics', 'reports', 'analytics:reports:view', 'View Reports', 'View analytics reports'),
('Analytics', 'reports', 'analytics:reports:export', 'Export Reports', 'Export reports to files'),
('Analytics', 'reports', 'analytics:reports:schedule', 'Schedule Reports', 'Schedule automated reports'),
('Analytics', 'unified-analytics', 'analytics:unified-analytics:view', 'View Unified Analytics', 'View unified analytics dashboard'),
('Analytics', 'unified-analytics', 'analytics:unified-analytics:configure', 'Configure Analytics', 'Configure analytics settings'),
('Analytics', 'agent-performance', 'analytics:agent-performance:view', 'View Agent Performance', 'View agent performance metrics'),
('Analytics', 'agent-performance', 'analytics:agent-performance:manage', 'Manage Agent Performance', 'Manage agent performance settings'),
-- 6. Channels Group
('Channels', 'whatsapp', 'channels:whatsapp:view', 'View WhatsApp', 'View WhatsApp channel settings'),
('Channels', 'whatsapp', 'channels:whatsapp:configure', 'Configure WhatsApp', 'Configure WhatsApp channel'),
('Channels', 'whatsapp', 'channels:whatsapp:send', 'Send via WhatsApp', 'Send messages via WhatsApp'),
('Channels', 'sms', 'channels:sms:view', 'View SMS', 'View SMS channel settings'),
('Channels', 'sms', 'channels:sms:configure', 'Configure SMS', 'Configure SMS channel'),
('Channels', 'sms', 'channels:sms:send', 'Send via SMS', 'Send messages via SMS'),
('Channels', 'email', 'channels:email:view', 'View Email', 'View email channel settings'),
('Channels', 'email', 'channels:email:configure', 'Configure Email', 'Configure email channel'),
('Channels', 'email', 'channels:email:send', 'Send via Email', 'Send messages via email'),
('Channels', 'live-chat', 'channels:live-chat:view', 'View Live Chat', 'View live chat settings'),
('Channels', 'live-chat', 'channels:live-chat:configure', 'Configure Live Chat', 'Configure live chat'),
('Channels', 'telegram', 'channels:telegram:view', 'View Telegram', 'View Telegram settings'),
('Channels', 'telegram', 'channels:telegram:configure', 'Configure Telegram', 'Configure Telegram'),
('Channels', 'facebook', 'channels:facebook:view', 'View Facebook', 'View Facebook settings'),
('Channels', 'facebook', 'channels:facebook:configure', 'Configure Facebook', 'Configure Facebook'),
('Channels', 'instagram', 'channels:instagram:view', 'View Instagram', 'View Instagram settings'),
('Channels', 'instagram', 'channels:instagram:configure', 'Configure Instagram', 'Configure Instagram'),
('Channels', 'tiktok', 'channels:tiktok:view', 'View TikTok', 'View TikTok settings'),
('Channels', 'tiktok', 'channels:tiktok:configure', 'Configure TikTok', 'Configure TikTok'),
('Channels', 'discord', 'channels:discord:view', 'View Discord', 'View Discord settings'),
('Channels', 'discord', 'channels:discord:configure', 'Configure Discord', 'Configure Discord'),
('Channels', 'slack', 'channels:slack:view', 'View Slack', 'View Slack settings'),
('Channels', 'slack', 'channels:slack:configure', 'Configure Slack', 'Configure Slack'),
('Channels', 'voice', 'channels:voice:view', 'View Voice', 'View voice channel settings'),
('Channels', 'voice', 'channels:voice:configure', 'Configure Voice', 'Configure voice channel'),
-- 7. AI & Automation Group
('AI & Automation', 'ai-copilot', 'ai:ai-copilot:view', 'View AI Copilot', 'View AI copilot settings'),
('AI & Automation', 'ai-copilot', 'ai:ai-copilot:configure', 'Configure AI Copilot', 'Configure AI copilot'),
('AI & Automation', 'ai-copilot', 'ai:ai-copilot:train', 'Train AI Copilot', 'Train AI copilot models'),
('AI & Automation', 'automations', 'ai:automations:view', 'View Automations', 'View automation rules'),
('AI & Automation', 'automations', 'ai:automations:create', 'Create Automations', 'Create automation rules'),
('AI & Automation', 'automations', 'ai:automations:edit', 'Edit Automations', 'Edit automation rules'),
('AI & Automation', 'automations', 'ai:automations:delete', 'Delete Automations', 'Delete automation rules'),
('AI & Automation', 'workflows', 'ai:workflows:view', 'View Workflows', 'View workflow definitions'),
('AI & Automation', 'workflows', 'ai:workflows:create', 'Create Workflows', 'Create workflows'),
('AI & Automation', 'workflows', 'ai:workflows:edit', 'Edit Workflows', 'Edit workflows'),
('AI & Automation', 'workflows', 'ai:workflows:delete', 'Delete Workflows', 'Delete workflows'),
('AI & Automation', 'tone-of-voice', 'ai:tone-of-voice:view', 'View Tone of Voice', 'View tone of voice settings'),
('AI & Automation', 'tone-of-voice', 'ai:tone-of-voice:configure', 'Configure Tone of Voice', 'Configure brand tone'),
-- 8. Settings & Admin Group
('Settings & Admin', 'team', 'settings:team:view', 'View Team', 'View team members'),
('Settings & Admin', 'team', 'settings:team:manage', 'Manage Team', 'Manage team members'),
('Settings & Admin', 'team', 'settings:team:invite', 'Invite Team Members', 'Send invitations'),
('Settings & Admin', 'roles', 'settings:roles:view', 'View Roles', 'View role definitions'),
('Settings & Admin', 'roles', 'settings:roles:create', 'Create Roles', 'Create custom roles'),
('Settings & Admin', 'roles', 'settings:roles:edit', 'Edit Roles', 'Edit role definitions'),
('Settings & Admin', 'roles', 'settings:roles:delete', 'Delete Roles', 'Delete custom roles'),
('Settings & Admin', 'audit-logs', 'settings:audit-logs:view', 'View Audit Logs', 'View audit log entries'),
('Settings & Admin', 'audit-logs', 'settings:audit-logs:export', 'Export Audit Logs', 'Export audit logs'),
('Settings & Admin', 'compliance', 'settings:compliance:view', 'View Compliance', 'View compliance settings'),
('Settings & Admin', 'compliance', 'settings:compliance:configure', 'Configure Compliance', 'Configure compliance rules'),
('Settings & Admin', 'webhooks', 'settings:webhooks:view', 'View Webhooks', 'View webhook configurations'),
('Settings & Admin', 'webhooks', 'settings:webhooks:configure', 'Configure Webhooks', 'Configure webhooks'),
('Settings & Admin', 'custom-attributes', 'settings:custom-attributes:view', 'View Custom Attributes', 'View custom attributes'),
('Settings & Admin', 'custom-attributes', 'settings:custom-attributes:manage', 'Manage Custom Attributes', 'Manage custom attributes'),
('Settings & Admin', 'macros', 'settings:macros:view', 'View Macros', 'View macro definitions'),
('Settings & Admin', 'macros', 'settings:macros:create', 'Create Macros', 'Create macros'),
('Settings & Admin', 'macros', 'settings:macros:edit', 'Edit Macros', 'Edit macros'),
('Settings & Admin', 'macros', 'settings:macros:delete', 'Delete Macros', 'Delete macros'),
('Settings & Admin', 'canned-responses', 'settings:canned-responses:view', 'View Canned Responses', 'View canned responses'),
('Settings & Admin', 'canned-responses', 'settings:canned-responses:create', 'Create Canned Responses', 'Create canned responses'),
('Settings & Admin', 'canned-responses', 'settings:canned-responses:edit', 'Edit Canned Responses', 'Edit canned responses'),
('Settings & Admin', 'canned-responses', 'settings:canned-responses:delete', 'Delete Canned Responses', 'Delete canned responses'),
-- 9. ERP & Integrations Group
('ERP & Integrations', 'erp-catalog', 'erp:catalog:view', 'View ERP Catalog', 'View ERP product catalog'),
('ERP & Integrations', 'erp-catalog', 'erp:catalog:sync', 'Sync ERP Catalog', 'Sync products from ERP'),
('ERP & Integrations', 'erp-orders', 'erp:orders:view', 'View ERP Orders', 'View orders from ERP'),
('ERP & Integrations', 'erp-orders', 'erp:orders:sync', 'Sync ERP Orders', 'Sync orders with ERP'),
('ERP & Integrations', 'erp-payments', 'erp:payments:view', 'View ERP Payments', 'View payments from ERP'),
('ERP & Integrations', 'erp-payments', 'erp:payments:sync', 'Sync ERP Payments', 'Sync payments with ERP'),
('ERP & Integrations', 'erp-inventory', 'erp:inventory:view', 'View ERP Inventory', 'View inventory from ERP'),
('ERP & Integrations', 'erp-inventory', 'erp:inventory:sync', 'Sync ERP Inventory', 'Sync inventory with ERP'),
('ERP & Integrations', 'erp-fulfilment', 'erp:fulfilment:view', 'View ERP Fulfilment', 'View fulfilment status from ERP'),
('ERP & Integrations', 'erp-fulfilment', 'erp:fulfilment:manage', 'Manage ERP Fulfilment', 'Manage fulfilment in ERP'),
('ERP & Integrations', 'sync-shopify', 'integrations:shopify:view', 'View Shopify Sync', 'View Shopify sync status'),
('ERP & Integrations', 'sync-shopify', 'integrations:shopify:configure', 'Configure Shopify Sync', 'Configure Shopify integration'),
('ERP & Integrations', 'sync-woocommerce', 'integrations:woocommerce:view', 'View WooCommerce Sync', 'View WooCommerce sync status'),
('ERP & Integrations', 'sync-woocommerce', 'integrations:woocommerce:configure', 'Configure WooCommerce Sync', 'Configure WooCommerce integration'),
('ERP & Integrations', 'sync-wix', 'integrations:wix:view', 'View Wix Sync', 'View Wix sync status'),
('ERP & Integrations', 'sync-wix', 'integrations:wix:configure', 'Configure Wix Sync', 'Configure Wix integration'),
('ERP & Integrations', 'sync-square', 'integrations:square:view', 'View Square Sync', 'View Square sync status'),
('ERP & Integrations', 'sync-square', 'integrations:square:configure', 'Configure Square Sync', 'Configure Square integration'),
('ERP & Integrations', 'crm-sync', 'integrations:crm:view', 'View CRM Sync', 'View CRM sync status'),
('ERP & Integrations', 'crm-sync', 'integrations:crm:configure', 'Configure CRM Sync', 'Configure CRM integration'),
-- 10. Help Center & Knowledge Group
('Help Center & Knowledge', 'help-center', 'help:help-center:view', 'View Help Center', 'View help center articles'),
('Help Center & Knowledge', 'help-center', 'help:help-center:edit', 'Edit Help Center', 'Edit help center articles'),
('Help Center & Knowledge', 'help-center', 'help:help-center:publish', 'Publish Help Center', 'Publish help center articles'),
('Help Center & Knowledge', 'kb-documents', 'help:kb-documents:view', 'View KB Documents', 'View knowledge base documents'),
('Help Center & Knowledge', 'kb-documents', 'help:kb-documents:edit', 'Edit KB Documents', 'Edit knowledge base documents'),
('Help Center & Knowledge', 'kb-documents', 'help:kb-documents:delete', 'Delete KB Documents', 'Delete knowledge base documents'),
('Help Center & Knowledge', 'procedures', 'help:procedures:view', 'View Procedures', 'View standard procedures'),
('Help Center & Knowledge', 'procedures', 'help:procedures:edit', 'Edit Procedures', 'Edit standard procedures'),
('Help Center & Knowledge', 'procedures', 'help:procedures:delete', 'Delete Procedures', 'Delete standard procedures'),
-- 11. Quality & Scoring Group
('Quality & Scoring', 'cx-qa-scoring', 'quality:cx-qa:view', 'View CX QA Scoring', 'View customer experience QA scores'),
('Quality & Scoring', 'cx-qa-scoring', 'quality:cx-qa:configure', 'Configure CX QA', 'Configure QA scoring rules'),
('Quality & Scoring', 'cx-qa-scoring', 'quality:cx-qa:review', 'Review CX QA', 'Review and override QA scores'),
('Quality & Scoring', 'copilot-reviewer', 'quality:copilot-reviewer:view', 'View Copilot Reviews', 'View AI copilot review results'),
('Quality & Scoring', 'copilot-reviewer', 'quality:copilot-reviewer:configure', 'Configure Copilot Reviewer', 'Configure copilot review settings'),
('Quality & Scoring', 'ab-testing', 'quality:ab-testing:view', 'View A/B Tests', 'View A/B test results'),
('Quality & Scoring', 'ab-testing', 'quality:ab-testing:create', 'Create A/B Tests', 'Create new A/B tests'),
('Quality & Scoring', 'ab-testing', 'quality:ab-testing:edit', 'Edit A/B Tests', 'Edit A/B test configurations'),
('Quality & Scoring', 'ab-testing', 'quality:ab-testing:delete', 'Delete A/B Tests', 'Delete A/B tests'),
-- 12. Security & Vault Group
('Security & Vault', 'vault', 'security:vault:view', 'View Vault', 'View vault contents'),
('Security & Vault', 'vault', 'security:vault:manage', 'Manage Vault', 'Manage vault secrets'),
('Security & Vault', 'compliance-dashboard', 'security:compliance-dashboard:view', 'View Compliance Dashboard', 'View compliance dashboard'),
('Security & Vault', 'compliance-dashboard', 'security:compliance-dashboard:configure', 'Configure Compliance', 'Configure compliance dashboard'),
('Security & Vault', 'crypto-mesh', 'security:crypto-mesh:view', 'View Crypto Mesh', 'View crypto mesh status'),
('Security & Vault', 'crypto-mesh', 'security:crypto-mesh:configure', 'Configure Crypto Mesh', 'Configure crypto mesh'),
('Security & Vault', 'monitoring', 'security:monitoring:view', 'View Monitoring', 'View system monitoring'),
('Security & Vault', 'monitoring', 'security:monitoring:configure', 'Configure Monitoring', 'Configure monitoring alerts'),
('Security & Vault', 'alerts', 'security:alerts:view', 'View Alerts', 'View system alerts'),
('Security & Vault', 'alerts', 'security:alerts:configure', 'Configure Alerts', 'Configure alert rules'),
-- 13. Messaging & Notifications Group
('Messaging & Notifications', 'in-app-messaging', 'messaging:in-app:view', 'View In-App Messages', 'View in-app messages'),
('Messaging & Notifications', 'in-app-messaging', 'messaging:in-app:send', 'Send In-App Messages', 'Send in-app messages'),
('Messaging & Notifications', 'notifications', 'messaging:notifications:view', 'View Notifications', 'View notification settings'),
('Messaging & Notifications', 'notifications', 'messaging:notifications:configure', 'Configure Notifications', 'Configure notification preferences'),
('Messaging & Notifications', 'messaging', 'messaging:messaging:view', 'View Messaging', 'View messaging logs'),
('Messaging & Notifications', 'messaging', 'messaging:messaging:configure', 'Configure Messaging', 'Configure messaging settings'),
-- 14. WhatsApp Extended Group
('WhatsApp Extended', 'whatsapp-automations', 'whatsapp:automations:view', 'View WhatsApp Automations', 'View WhatsApp automation rules'),
('WhatsApp Extended', 'whatsapp-automations', 'whatsapp:automations:edit', 'Edit WhatsApp Automations', 'Edit WhatsApp automation rules'),
('WhatsApp Extended', 'whatsapp-campaigns', 'whatsapp:campaigns:view', 'View WhatsApp Campaigns', 'View WhatsApp campaign history'),
('WhatsApp Extended', 'whatsapp-campaigns', 'whatsapp:campaigns:manage', 'Manage WhatsApp Campaigns', 'Manage WhatsApp campaigns'),
('WhatsApp Extended', 'whatsapp-flows', 'whatsapp:flows:view', 'View WhatsApp Flows', 'View WhatsApp flow definitions'),
('WhatsApp Extended', 'whatsapp-flows', 'whatsapp:flows:edit', 'Edit WhatsApp Flows', 'Edit WhatsApp flows'),
('WhatsApp Extended', 'whatsapp-template-sync', 'whatsapp:template-sync:view', 'View Template Sync', 'View WhatsApp template sync status'),
('WhatsApp Extended', 'whatsapp-template-sync', 'whatsapp:template-sync:manage', 'Manage Template Sync', 'Manage WhatsApp template sync'),
-- 15. Commerce Extended Group
('Commerce Extended', 'commerce-cart', 'commerce:cart:view', 'View Cart', 'View shopping cart data'),
('Commerce Extended', 'commerce-cart', 'commerce:cart:manage', 'Manage Cart', 'Manage shopping cart'),
('Commerce Extended', 'commerce-delivery', 'commerce:delivery:view', 'View Delivery', 'View delivery status'),
('Commerce Extended', 'commerce-delivery', 'commerce:delivery:manage', 'Manage Delivery', 'Manage delivery options'),
('Commerce Extended', 'commerce-discounts', 'commerce:discounts:view', 'View Discounts', 'View discount codes'),
('Commerce Extended', 'commerce-discounts', 'commerce:discounts:manage', 'Manage Discounts', 'Manage discount codes'),
('Commerce Extended', 'commerce-options', 'commerce:options:view', 'View Product Options', 'View product options'),
('Commerce Extended', 'commerce-options', 'commerce:options:manage', 'Manage Product Options', 'Manage product options'),
('Commerce Extended', 'channel-commerce-templates', 'commerce:templates:view', 'View Commerce Templates', 'View channel commerce templates'),
('Commerce Extended', 'channel-commerce-templates', 'commerce:templates:edit', 'Edit Commerce Templates', 'Edit channel commerce templates'),
-- 16. AI Extended Group
('AI Extended', 'autonomous-resolution', 'ai:autonomous:view', 'View Autonomous Resolution', 'View autonomous resolution results'),
('AI Extended', 'autonomous-resolution', 'ai:autonomous:configure', 'Configure Autonomous Resolution', 'Configure autonomous resolution'),
('AI Extended', 'suggestions', 'ai:suggestions:view', 'View Suggestions', 'View AI suggestions'),
('AI Extended', 'suggestions', 'ai:suggestions:configure', 'Configure Suggestions', 'Configure AI suggestions'),
('AI Extended', 'topic-categorization', 'ai:topic-categorization:view', 'View Topic Categories', 'View topic categorization'),
('AI Extended', 'topic-categorization', 'ai:topic-categorization:configure', 'Configure Topics', 'Configure topic categories'),
('AI Extended', 'audience-kb-scoping', 'ai:audience-kb:view', 'View Audience KB', 'View audience knowledge base scoping'),
('AI Extended', 'audience-kb-scoping', 'ai:audience-kb:configure', 'Configure Audience KB', 'Configure audience KB scoping'),
-- 17. Finance Group
('Finance', 'fin-operator', 'finance:operator:view', 'View Finance Operator', 'View finance operations'),
('Finance', 'fin-operator', 'finance:operator:manage', 'Manage Finance Operator', 'Manage finance operations'),
('Finance', 'fin-vision', 'finance:vision:view', 'View Finance Vision', 'View financial projections'),
('Finance', 'fin-vision', 'finance:vision:configure', 'Configure Finance Vision', 'Configure financial projections'),
-- 18. Widget & Embed Group
('Widget & Embed', 'embed-widget', 'widget:embed:view', 'View Embed Widget', 'View embed widget settings'),
('Widget & Embed', 'embed-widget', 'widget:embed:configure', 'Configure Embed Widget', 'Configure embed widget'),
('Widget & Embed', 'widget-support', 'widget:support:view', 'View Widget Support', 'View widget support data'),
('Widget & Embed', 'widget-support', 'widget:support:configure', 'Configure Widget Support', 'Configure widget support'),
-- 19. System & Infrastructure Group
('System & Infrastructure', 'system-health', 'system:health:view', 'View System Health', 'View system health status'),
('System & Infrastructure', 'system-health', 'system:health:configure', 'Configure System Health', 'Configure health checks'),
('System & Infrastructure', 'scheduled-tasks', 'system:scheduled-tasks:view', 'View Scheduled Tasks', 'View scheduled tasks'),
('System & Infrastructure', 'scheduled-tasks', 'system:scheduled-tasks:manage', 'Manage Scheduled Tasks', 'Manage scheduled tasks'),
('System & Infrastructure', 'business-hours', 'system:business-hours:view', 'View Business Hours', 'View business hours settings'),
('System & Infrastructure', 'business-hours', 'system:business-hours:configure', 'Configure Business Hours', 'Configure business hours'),
('System & Infrastructure', 'conversation-search', 'system:search:view', 'View Conversation Search', 'Search conversations'),
('System & Infrastructure', 'draft-actions', 'system:drafts:view', 'View Draft Actions', 'View draft messages'),
('System & Infrastructure', 'draft-actions', 'system:drafts:manage', 'Manage Draft Actions', 'Manage draft messages'),
('System & Infrastructure', 'brand-overrides', 'system:brand-overrides:view', 'View Brand Overrides', 'View brand-specific overrides'),
('System & Infrastructure', 'brand-overrides', 'system:brand-overrides:configure', 'Configure Brand Overrides', 'Configure brand overrides'),
('System & Infrastructure', 'integration-settings', 'system:integration:view', 'View Integration Settings', 'View integration configurations'),
('System & Infrastructure', 'integration-settings', 'system:integration:configure', 'Configure Integrations', 'Configure integration settings'),
-- 20. OTP & Security Group
('OTP & Security', 'otp-engine', 'otp:engine:view', 'View OTP Engine', 'View OTP engine status'),
('OTP & Security', 'otp-engine', 'otp:engine:configure', 'Configure OTP Engine', 'Configure OTP settings'),
('OTP & Security', 'smtp-email', 'otp:smtp:view', 'View SMTP Settings', 'View SMTP configuration'),
('OTP & Security', 'smtp-email', 'otp:smtp:configure', 'Configure SMTP', 'Configure SMTP settings'),
('OTP & Security', 'external-otp', 'otp:external:view', 'View External OTP', 'View external OTP integrations'),
('OTP & Security', 'external-otp', 'otp:external:configure', 'Configure External OTP', 'Configure external OTP'),
('OTP & Security', 'store-connection', 'otp:store:view', 'View Store Connections', 'View store connection status'),
('OTP & Security', 'store-connection', 'otp:store:configure', 'Configure Store Connections', 'Configure store connections'),
-- 21. Access Control Group
('Access Control', 'access-control', 'access:control:view', 'View Access Control', 'View access control settings'),
('Access Control', 'access-control', 'access:control:manage', 'Manage Access Control', 'Manage access grants and roles'),
('Access Control', 'access-control', 'access:control:transfer', 'Transfer Ownership', 'Transfer ownership of entities'),
('Access Control', 'departments', 'access:departments:view', 'View Departments', 'View department structure'),
('Access Control', 'departments', 'access:departments:manage', 'Manage Departments', 'Manage departments and assignments')
ON CONFLICT (group_name, service_name, permission_key) DO NOTHING;
