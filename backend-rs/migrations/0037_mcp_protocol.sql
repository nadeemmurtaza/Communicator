-- Migration 0037: MCP Protocol Support (PostgreSQL)
-- Enables connection to MCP-compatible servers for extensible AI tool connections.
-- Mirrors Intercom's "MCP connections to Jira, Sentry, Glean, Guru, Slite, Pendo, Fullstory" capability.
-- MCP (Model Context Protocol) is an open protocol for connecting AI models to external tools.

-- ============================================================================
-- 1. MCP SERVERS
-- ============================================================================
-- Defines external MCP-compatible servers that can be connected.
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    server_url TEXT NOT NULL,
    transport_type TEXT NOT NULL DEFAULT 'stdio',
    headers TEXT NOT NULL DEFAULT '{}',
    environment_vars TEXT NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    connection_status TEXT NOT NULL DEFAULT 'disconnected',
    last_connected_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_org ON mcp_servers(organization_id);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_status ON mcp_servers(connection_status);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_active ON mcp_servers(is_active);

-- ============================================================================
-- 2. MCP TOOLS
-- ============================================================================
-- Discovered tools from connected MCP servers.
CREATE TABLE IF NOT EXISTS mcp_tools (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    server_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    description TEXT DEFAULT '',
    input_schema TEXT NOT NULL DEFAULT '{}',
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    ai_accessible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_mcp_tools_server ON mcp_tools(server_id);
CREATE INDEX IF NOT EXISTS idx_mcp_tools_name ON mcp_tools(tool_name);
CREATE INDEX IF NOT EXISTS idx_mcp_tools_enabled ON mcp_tools(is_enabled);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_tools_unique ON mcp_tools(server_id, tool_name);

-- ============================================================================
-- 3. MCP SESSIONS
-- ============================================================================
-- Tracks active MCP sessions for stateful connections.
CREATE TABLE IF NOT EXISTS mcp_sessions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    server_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    metadata TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_mcp_sessions_server ON mcp_sessions(server_id);
CREATE INDEX IF NOT EXISTS idx_mcp_sessions_status ON mcp_sessions(status);
CREATE INDEX IF NOT EXISTS idx_mcp_sessions_started ON mcp_sessions(started_at);

-- ============================================================================
-- 4. MCP CALL LOGS
-- ============================================================================
-- Tracks every tool call for auditing and analytics.
CREATE TABLE IF NOT EXISTS mcp_call_logs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    server_id TEXT NOT NULL,
    tool_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    conversation_id UUID,
    contact_id UUID,
    input_data TEXT NOT NULL DEFAULT '{}',
    output_data TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    duration_ms INTEGER,
    tokens_used INTEGER DEFAULT 0,
    called_by TEXT DEFAULT 'ai',
    called_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    FOREIGN KEY (server_id) REFERENCES mcp_servers(id) ON DELETE SET NULL,
    FOREIGN KEY (tool_id) REFERENCES mcp_tools(id) ON DELETE SET NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_server ON mcp_call_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_tool ON mcp_call_logs(tool_id);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_conversation ON mcp_call_logs(conversation_id);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_status ON mcp_call_logs(status);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_called_at ON mcp_call_logs(called_at);

-- ============================================================================
-- 5. MCP SETTINGS
-- ============================================================================
-- Per-organization settings for MCP behavior.
CREATE TABLE IF NOT EXISTS mcp_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    auto_discover_tools BOOLEAN NOT NULL DEFAULT true,
    log_all_calls BOOLEAN NOT NULL DEFAULT true,
    max_concurrent_calls INTEGER NOT NULL DEFAULT 10,
    timeout_seconds INTEGER NOT NULL DEFAULT 30,
    allowed_tools TEXT NOT NULL DEFAULT '[]',
    blocked_tools TEXT NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_mcp_settings_org ON mcp_settings(organization_id);

-- Seed default settings
INSERT INTO mcp_settings (id, organization_id, enabled, auto_discover_tools, log_all_calls)
VALUES ('mcp-settings-default', 'org-default', true, true, true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 6. PREBUILT MCP SERVER TEMPLATES
-- ============================================================================
-- Predefined server templates for common integrations (Sentry, Guru, Pendo, etc.).
CREATE TABLE IF NOT EXISTS mcp_server_templates (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    provider TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    default_url TEXT NOT NULL,
    transport_type TEXT NOT NULL DEFAULT 'stdio',
    required_env_vars TEXT NOT NULL DEFAULT '[]',
    default_tools TEXT NOT NULL DEFAULT '[]',
    icon_url TEXT DEFAULT '',
    documentation_url TEXT DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_mcp_templates_provider ON mcp_server_templates(provider);

-- Seed prebuilt templates for common integrations
INSERT INTO mcp_server_templates (id, provider, name, description, default_url, transport_type, required_env_vars, default_tools)
VALUES
    ('mcp-template-sentry', 'sentry', 'Sentry', 'Error tracking and monitoring', 'https://sentry.io/api/mcp/', 'http', '["SENTRY_AUTH_TOKEN"]', '["search_issues", "get_issue_details", "create_issue", "get_releases"]'),
    ('mcp-template-guru', 'guru', 'Guru', 'Knowledge base and sales intelligence', 'https://api.getguru.com/mcp', 'http', '["GURU_API_KEY"]', '["search_knowledge", "get_card", "create_card", "get_recommendations"]'),
    ('mcp-template-pendo', 'pendo', 'Pendo', 'Product analytics and user behavior', 'https://api.pendo.io/mcp', 'http', '["PENDO_API_KEY"]', '["get_guides", "get_reports", "get_user_activity", "get_guided_walkthroughs"]'),
    ('mcp-template-fullstory', 'fullstory', 'Fullstory', 'Digital experience analytics', 'https://api.fullstory.com/mcp', 'http', '["FULLSTORY_API_KEY"]', '["search_sessions", "get_session_details", "get_analytics", "create_tag"]'),
    ('mcp-template-jira', 'jira', 'Jira', 'Issue and project tracking', 'https://api.atlassian.com/mcp/jira', 'http', '["JIRA_API_TOKEN"]', '["search_issues", "create_issue", "update_issue", "get_transitions"]'),
    ('mcp-template-github', 'github', 'GitHub', 'Code repository management', 'https://api.github.com/mcp', 'http', '["GITHUB_TOKEN"]', '["search_repos", "get_issues", "create_issue", "get_pull_requests"]'),
    ('mcp-template-slack', 'slack', 'Slack', 'Team communication and collaboration', 'https://slack.com/api/mcp', 'http', '["SLACK_BOT_TOKEN"]', '["search_messages", "send_message", "get_channels", "get_threads"]')
ON CONFLICT (id) DO NOTHING;
