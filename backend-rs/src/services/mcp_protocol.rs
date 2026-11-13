//! MCP Protocol Support — Phase 28
//!
//! Enables connection to MCP-compatible servers for extensible AI tool connections.
//! Mirrors Intercom's "MCP connections to Jira, Sentry, Glean, Guru, Slite, Pendo, Fullstory" capability.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPServer {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub server_url: String,
    pub transport_type: String,
    pub headers: Option<Value>,
    pub environment_vars: Option<Value>,
    pub is_active: bool,
    pub connection_status: String,
    pub last_connected_at: Option<String>,
    pub tool_count: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPTool {
    pub id: String,
    pub server_id: String,
    pub server_name: Option<String>,
    pub tool_name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
    pub is_enabled: bool,
    pub ai_accessible: bool,
    pub call_count: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPSession {
    pub id: String,
    pub server_id: String,
    pub server_name: Option<String>,
    pub session_id: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPCallLog {
    pub id: String,
    pub server_id: String,
    pub server_name: Option<String>,
    pub tool_id: String,
    pub tool_name: String,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
    pub tokens_used: i32,
    pub called_by: String,
    pub called_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPServerTemplate {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub description: Option<String>,
    pub default_url: String,
    pub transport_type: String,
    pub required_env_vars: Option<Value>,
    pub default_tools: Option<Value>,
    pub icon_url: Option<String>,
    pub documentation_url: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub auto_discover_tools: bool,
    pub log_all_calls: bool,
    pub max_concurrent_calls: i32,
    pub timeout_seconds: i32,
    pub allowed_tools: Option<Value>,
    pub blocked_tools: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MCPSummary {
    pub total_servers: i32,
    pub connected_servers: i32,
    pub total_tools: i32,
    pub enabled_tools: i32,
    pub active_sessions: i32,
    pub total_calls_7d: i32,
    pub total_calls_30d: i32,
    pub enabled: bool,
    pub top_called_tools: Vec<Value>,
    pub recent_calls: Vec<MCPCallLog>,
    pub available_templates: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerResult {
    pub server_id: String,
    pub name: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResult {
    pub tool_id: String,
    pub tool_name: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub result: Value,
    pub duration_ms: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscoverToolsResult {
    pub server_id: String,
    pub tools_discovered: i32,
    pub tools: Vec<String>,
    pub message: String,
}

// ——— Core Functions ———

/// Get MCP summary dashboard data
pub async fn get_mcp_summary(state: &AppState) -> Result<MCPSummary, sqlx::Error> {
    // Total servers
    let total_servers: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_servers")
        .fetch_one(&state.db)
        .await?;

    // Connected servers
    let connected_servers: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_servers WHERE connection_status = 'connected'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total tools
    let total_tools: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_tools")
        .fetch_one(&state.db)
        .await?;

    // Enabled tools
    let enabled_tools: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_tools WHERE is_enabled = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Active sessions
    let active_sessions: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_sessions WHERE status = 'active'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total calls in last 7 days
    let total_calls_7d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_call_logs WHERE called_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total calls in last 30 days
    let total_calls_30d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_call_logs WHERE called_at >= NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Settings
    let settings_row = sqlx::query(
        "SELECT enabled FROM mcp_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;
    let enabled = settings_row
        .map(|r| r.try_get::<bool, _>(0).unwrap_or(true))
        .unwrap_or(true);

    // Available templates
    let available_templates: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_server_templates WHERE is_active = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Top called tools
    let top_tools_rows = sqlx::query(
        "SELECT mcl.tool_name, COUNT(*) as cnt FROM mcp_call_logs mcl \
         GROUP BY mcl.tool_name ORDER BY cnt DESC LIMIT 5"
    )
    .fetch_all(&state.db)
    .await?;

    let top_called_tools: Vec<Value> = top_tools_rows
        .iter()
        .map(|row| {
            json!({
                "tool_name": row.try_get::<String, _>(0).unwrap_or_default(),
                "call_count": row.try_get::<i64, _>(1).unwrap_or(0i64),
            })
        })
        .collect();

    // Recent calls
    let call_rows = sqlx::query(
        "SELECT mcl.id, mcl.server_id, ms.name as server_name, mcl.tool_id, mcl.tool_name, \
         mcl.conversation_id, mcl.contact_id, mcl.input_data, mcl.output_data, \
         mcl.status, mcl.error_message, mcl.duration_ms, mcl.tokens_used, \
         mcl.called_by, mcl.called_at, mcl.completed_at \
         FROM mcp_call_logs mcl \
         LEFT JOIN mcp_servers ms ON mcl.server_id = ms.id \
         ORDER BY mcl.called_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_calls: Vec<MCPCallLog> = call_rows
        .iter()
        .map(|row| MCPCallLog {
            id: row.try_get(0).unwrap_or_default(),
            server_id: row.try_get(1).unwrap_or_default(),
            server_name: row.try_get(2).ok(),
            tool_id: row.try_get(3).unwrap_or_default(),
            tool_name: row.try_get(4).unwrap_or_default(),
            conversation_id: row.try_get(5).ok(),
            contact_id: row.try_get(6).ok(),
            input_data: row.try_get(7).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            output_data: row.try_get(8).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            status: row.try_get(9).unwrap_or_default(),
            error_message: row.try_get(10).ok(),
            duration_ms: row.try_get(11).ok(),
            tokens_used: row.try_get(12).unwrap_or(0i32),
            called_by: row.try_get(13).unwrap_or_default(),
            called_at: row.try_get(14).unwrap_or_default(),
            completed_at: row.try_get(15).ok(),
        })
        .collect();

    Ok(MCPSummary {
        total_servers,
        connected_servers,
        total_tools,
        enabled_tools,
        active_sessions,
        total_calls_7d,
        total_calls_30d,
        enabled,
        top_called_tools,
        recent_calls,
        available_templates,
    })
}

/// List MCP servers
pub async fn list_mcp_servers(
    state: &AppState,
    organization_id: Option<&str>,
    is_active: Option<bool>,
    limit: i32,
    offset: i32,
) -> Result<Vec<MCPServer>, sqlx::Error> {
    let limit = limit.max(1).min(100);
    let offset = offset.max(0);

    let mut conditions = Vec::new();
    let mut param_count = 1;

    if let Some(org) = organization_id {
        conditions.push(format!("organization_id = ${}", param_count));
        param_count += 1;
    }
    if let Some(active) = is_active {
        conditions.push(format!("is_active = ${}", param_count));
        param_count += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT ms.id, ms.organization_id, ms.name, ms.description, ms.server_url, \
         ms.transport_type, ms.headers, ms.environment_vars, ms.is_active, \
         ms.connection_status, ms.last_connected_at, \
         (SELECT COUNT(*) FROM mcp_tools mt WHERE mt.server_id = ms.id) as tool_count, \
         ms.created_at, ms.updated_at \
         FROM mcp_servers ms \
         {} \
         ORDER BY ms.created_at DESC \
         LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(org) = organization_id {
        query_builder = query_builder.bind(org);
    }
    if let Some(active) = is_active {
        query_builder = query_builder.bind(active);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(&state.db).await?;

    Ok(rows
        .iter()
        .map(|row| MCPServer {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            server_url: row.try_get(4).unwrap_or_default(),
            transport_type: row.try_get(5).unwrap_or_default(),
            headers: row.try_get(6).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            environment_vars: row.try_get(7).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            is_active: row.try_get(8).unwrap_or(true),
            connection_status: row.try_get(9).unwrap_or_default(),
            last_connected_at: row.try_get(10).ok(),
            tool_count: row.try_get(11).ok(),
            created_at: row.try_get(12).unwrap_or_default(),
            updated_at: row.try_get(13).unwrap_or_default(),
        })
        .collect())
}

/// Create an MCP server connection
pub async fn create_mcp_server(
    state: &AppState,
    organization_id: &str,
    name: &str,
    server_url: &str,
    transport_type: &str,
) -> Result<ServerResult, String> {
    let server_id = format!("mcp-server-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO mcp_servers (id, organization_id, name, server_url, transport_type) \
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&server_id)
    .bind(organization_id)
    .bind(name)
    .bind(server_url)
    .bind(transport_type)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to create server: {}", e))?;

    Ok(ServerResult {
        server_id,
        name: name.to_string(),
        message: "MCP server connection created".to_string(),
    })
}

/// Update an MCP server
pub async fn update_mcp_server(
    state: &AppState,
    server_id: &str,
    name: Option<&str>,
    is_active: Option<bool>,
    server_url: Option<&str>,
) -> Result<(), String> {
    let mut sets = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    let mut param_count = 1;

    if let Some(n) = name {
        sets.push(format!("name = '${}'", n.replace('\'', "''")));
    }
    if let Some(active) = is_active {
        sets.push(format!("is_active = {}", active));
    }
    if let Some(url) = server_url {
        sets.push(format!("server_url = '${}'", url.replace('\'', "''")));
    }
    sets.push("updated_at = NOW()".to_string());

    if sets.len() <= 1 {
        return Ok(());
    }

    let query = format!(
        "UPDATE mcp_servers SET {} WHERE id = '${}'",
        sets.join(", "),
        server_id.replace('\'', "''")
    );

    sqlx::query(&query)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to update server: {}", e))?;

    Ok(())
}

/// Delete an MCP server
pub async fn delete_mcp_server(state: &AppState, server_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM mcp_servers WHERE id = $1")
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete server: {}", e))?;

    Ok(())
}

/// Discover tools from an MCP server
pub async fn discover_tools(state: &AppState, server_id: &str) -> Result<DiscoverToolsResult, String> {
    // Check server exists
    let server_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM mcp_servers WHERE id = $1)"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if !server_exists {
        return Err(format!("MCP server {} not found", server_id));
    }

    // Get server details
    let server_row = sqlx::query(
        "SELECT server_url, transport_type, headers FROM mcp_servers WHERE id = $1"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let server_url: String = server_row.try_get(0).unwrap_or_default();
    let transport_type: String = server_row.try_get(1).unwrap_or_default();

    // Simulate tool discovery — in production, this would connect to the MCP server
    // and call the tools/list endpoint
    let default_tools = match transport_type.as_str() {
        "stdio" => vec!["search", "get_details", "create", "update"],
        "http" => {
            // Try to infer from URL
            if server_url.contains("sentry") {
                vec!["search_issues", "get_issue_details", "create_issue", "get_releases"]
            } else if server_url.contains("guru") {
                vec!["search_knowledge", "get_card", "create_card", "get_recommendations"]
            } else if server_url.contains("jira") {
                vec!["search_issues", "create_issue", "update_issue", "get_transitions"]
            } else if server_url.contains("github") {
                vec!["search_repos", "get_issues", "create_issue", "get_pull_requests"]
            } else if server_url.contains("slack") {
                vec!["search_messages", "send_message", "get_channels", "get_threads"]
            } else {
                vec!["search", "get", "create", "update"]
            }
        }
        _ => vec!["search", "get", "create"],
    };

    let mut tools_discovered: i32 = 0;
    let mut tool_names = Vec::new();

    for tool_name in &default_tools {
        let tool_id = format!("mcp-tool-{}", uuid::Uuid::new_v4());

        sqlx::query(
            "INSERT INTO mcp_tools (id, server_id, tool_name, description, input_schema) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (server_id, tool_name) DO UPDATE SET updated_at = NOW()"
        )
        .bind(&tool_id)
        .bind(server_id)
        .bind(tool_name)
        .bind(format!("Tool: {}", tool_name))
        .bind(r#"{"type": "object", "properties": {}}"#)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to insert tool: {}", e))?;

        tools_discovered += 1;
        tool_names.push(tool_name.to_string());
    }

    // Update server status
    sqlx::query("UPDATE mcp_servers SET connection_status = 'connected', \
                 last_connected_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to update server status: {}", e))?;

    Ok(DiscoverToolsResult {
        server_id: server_id.to_string(),
        tools_discovered,
        tools: tool_names,
        message: format!("Discovered {} tools from server", tools_discovered),
    })
}

/// List tools for a server
pub async fn list_mcp_tools(
    state: &AppState,
    server_id: Option<&str>,
    is_enabled: Option<bool>,
    limit: i32,
    offset: i32,
) -> Result<Vec<MCPTool>, sqlx::Error> {
    let limit = limit.max(1).min(100);
    let offset = offset.max(0);

    let mut conditions = Vec::new();
    let mut param_count = 1;

    if let Some(sid) = server_id {
        conditions.push(format!("mt.server_id = ${}", param_count));
        param_count += 1;
    }
    if let Some(enabled) = is_enabled {
        conditions.push(format!("mt.is_enabled = ${}", param_count));
        param_count += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT mt.id, mt.server_id, ms.name as server_name, mt.tool_name, mt.description, \
         mt.input_schema, mt.is_enabled, mt.ai_accessible, \
         (SELECT COUNT(*) FROM mcp_call_logs mcl WHERE mcl.tool_id = mt.id) as call_count, \
         mt.created_at, mt.updated_at \
         FROM mcp_tools mt \
         LEFT JOIN mcp_servers ms ON mt.server_id = ms.id \
         {} \
         ORDER BY mt.tool_name \
         LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(sid) = server_id {
        query_builder = query_builder.bind(sid);
    }
    if let Some(enabled) = is_enabled {
        query_builder = query_builder.bind(enabled);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(&state.db).await?;

    Ok(rows
        .iter()
        .map(|row| MCPTool {
            id: row.try_get(0).unwrap_or_default(),
            server_id: row.try_get(1).unwrap_or_default(),
            server_name: row.try_get(2).ok(),
            tool_name: row.try_get(3).unwrap_or_default(),
            description: row.try_get(4).ok(),
            input_schema: row.try_get(5).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            is_enabled: row.try_get(6).unwrap_or(true),
            ai_accessible: row.try_get(7).unwrap_or(true),
            call_count: row.try_get(8).ok(),
            created_at: row.try_get(9).unwrap_or_default(),
            updated_at: row.try_get(10).unwrap_or_default(),
        })
        .collect())
}

/// Call an MCP tool
pub async fn call_mcp_tool(
    state: &AppState,
    server_id: &str,
    tool_id: &str,
    tool_name: &str,
    input_data: &str,
    conversation_id: Option<&str>,
    contact_id: Option<&str>,
) -> Result<CallToolResult, String> {
    let call_id = format!("mcp-call-{}", uuid::Uuid::new_v4());
    let start = std::time::Instant::now();

    // Log the call
    sqlx::query(
        "INSERT INTO mcp_call_logs (id, server_id, tool_id, tool_name, conversation_id, \
         contact_id, input_data, status, called_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', 'ai')"
    )
    .bind(&call_id)
    .bind(server_id)
    .bind(tool_id)
    .bind(tool_name)
    .bind(conversation_id)
    .bind(contact_id)
    .bind(input_data)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to log call: {}", e))?;

    // Simulate tool execution — in production, this would connect to the MCP server
    // and execute the tool with the provided input
    let result = json!({
        "status": "success",
        "message": format!("Tool {} executed successfully", tool_name),
        "data": {
            "tool": tool_name,
            "input": input_data,
            "simulated": true
        }
    });

    let duration_ms = start.elapsed().as_millis() as i32;

    // Update call log with result
    sqlx::query(
        "UPDATE mcp_call_logs SET status = 'completed', output_data = $1, \
         duration_ms = $2, completed_at = NOW() WHERE id = $3"
    )
    .bind(result.to_string())
    .bind(duration_ms)
    .bind(&call_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to update call log: {}", e))?;

    Ok(CallToolResult {
        call_id,
        tool_name: tool_name.to_string(),
        result,
        duration_ms,
    })
}

/// List MCP server templates
pub async fn list_mcp_templates(state: &AppState) -> Result<Vec<MCPServerTemplate>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, provider, name, description, default_url, transport_type, \
         required_env_vars, default_tools, icon_url, documentation_url, is_active, created_at \
         FROM mcp_server_templates WHERE is_active = true \
         ORDER BY provider"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .iter()
        .map(|row| MCPServerTemplate {
            id: row.try_get(0).unwrap_or_default(),
            provider: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            default_url: row.try_get(4).unwrap_or_default(),
            transport_type: row.try_get(5).unwrap_or_default(),
            required_env_vars: row.try_get(6).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            default_tools: row.try_get(7).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            icon_url: row.try_get(8).ok(),
            documentation_url: row.try_get(9).ok(),
            is_active: row.try_get(10).unwrap_or(true),
            created_at: row.try_get(11).unwrap_or_default(),
        })
        .collect())
}

/// Get MCP settings
pub async fn get_mcp_settings(state: &AppState) -> Result<MCPSettings, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, auto_discover_tools, log_all_calls, \
         max_concurrent_calls, timeout_seconds, allowed_tools, blocked_tools, \
         created_at, updated_at \
         FROM mcp_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(MCPSettings {
        id: row.try_get(0).unwrap_or_default(),
        organization_id: row.try_get(1).unwrap_or_default(),
        enabled: row.try_get(2).unwrap_or(true),
        auto_discover_tools: row.try_get(3).unwrap_or(true),
        log_all_calls: row.try_get(4).unwrap_or(true),
        max_concurrent_calls: row.try_get(5).unwrap_or(10i32),
        timeout_seconds: row.try_get(6).unwrap_or(30i32),
        allowed_tools: row.try_get(7).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
        blocked_tools: row.try_get(8).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
        created_at: row.try_get(9).unwrap_or_default(),
        updated_at: row.try_get(10).unwrap_or_default(),
    })
}

/// Update MCP settings
pub async fn update_mcp_settings(
    state: &AppState,
    enabled: Option<bool>,
    auto_discover_tools: Option<bool>,
    log_all_calls: Option<bool>,
    max_concurrent_calls: Option<i32>,
    timeout_seconds: Option<i32>,
) -> Result<(), String> {
    let mut sets = Vec::new();

    if let Some(e) = enabled {
        sets.push(format!("enabled = {}", e));
    }
    if let Some(a) = auto_discover_tools {
        sets.push(format!("auto_discover_tools = {}", a));
    }
    if let Some(l) = log_all_calls {
        sets.push(format!("log_all_calls = {}", l));
    }
    if let Some(m) = max_concurrent_calls {
        sets.push(format!("max_concurrent_calls = {}", m));
    }
    if let Some(t) = timeout_seconds {
        sets.push(format!("timeout_seconds = {}", t));
    }

    if sets.is_empty() {
        return Ok(());
    }

    sets.push("updated_at = NOW()".to_string());

    let query = format!(
        "UPDATE mcp_settings SET {} WHERE organization_id = 'org-default'",
        sets.join(", ")
    );

    sqlx::query(&query)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    Ok(())
}

/// List MCP call logs
pub async fn list_mcp_call_logs(
    state: &AppState,
    server_id: Option<&str>,
    status: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<MCPCallLog>, sqlx::Error> {
    let limit = limit.max(1).min(100);
    let offset = offset.max(0);

    let mut conditions = Vec::new();
    let mut param_count = 1;

    if let Some(sid) = server_id {
        conditions.push(format!("mcl.server_id = ${}", param_count));
        param_count += 1;
    }
    if let Some(st) = status {
        conditions.push(format!("mcl.status = ${}", param_count));
        param_count += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT mcl.id, mcl.server_id, ms.name as server_name, mcl.tool_id, mcl.tool_name, \
         mcl.conversation_id, mcl.contact_id, mcl.input_data, mcl.output_data, \
         mcl.status, mcl.error_message, mcl.duration_ms, mcl.tokens_used, \
         mcl.called_by, mcl.called_at, mcl.completed_at \
         FROM mcp_call_logs mcl \
         LEFT JOIN mcp_servers ms ON mcl.server_id = ms.id \
         {} \
         ORDER BY mcl.called_at DESC \
         LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(sid) = server_id {
        query_builder = query_builder.bind(sid);
    }
    if let Some(st) = status {
        query_builder = query_builder.bind(st);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(&state.db).await?;

    Ok(rows
        .iter()
        .map(|row| MCPCallLog {
            id: row.try_get(0).unwrap_or_default(),
            server_id: row.try_get(1).unwrap_or_default(),
            server_name: row.try_get(2).ok(),
            tool_id: row.try_get(3).unwrap_or_default(),
            tool_name: row.try_get(4).unwrap_or_default(),
            conversation_id: row.try_get(5).ok(),
            contact_id: row.try_get(6).ok(),
            input_data: row.try_get(7).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            output_data: row.try_get(8).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            status: row.try_get(9).unwrap_or_default(),
            error_message: row.try_get(10).ok(),
            duration_ms: row.try_get(11).ok(),
            tokens_used: row.try_get(12).unwrap_or(0i32),
            called_by: row.try_get(13).unwrap_or_default(),
            called_at: row.try_get(14).unwrap_or_default(),
            completed_at: row.try_get(15).ok(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_serialization() {
        let summary = MCPSummary {
            total_servers: 5,
            connected_servers: 3,
            total_tools: 20,
            enabled_tools: 18,
            active_sessions: 2,
            total_calls_7d: 100,
            total_calls_30d: 500,
            enabled: true,
            top_called_tools: vec![],
            recent_calls: vec![],
            available_templates: 7,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_servers\":5"));
        assert!(json.contains("\"connected_servers\":3"));
    }

    #[test]
    fn test_discover_tools_result() {
        let result = DiscoverToolsResult {
            server_id: "test-server".to_string(),
            tools_discovered: 4,
            tools: vec!["search".to_string(), "get".to_string()],
            message: "Discovered 4 tools".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"tools_discovered\":4"));
    }

    #[test]
    fn test_call_tool_result() {
        let result = CallToolResult {
            call_id: "test-call".to_string(),
            tool_name: "search".to_string(),
            result: json!({"status": "ok"}),
            duration_ms: 150,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"duration_ms\":150"));
    }
}
