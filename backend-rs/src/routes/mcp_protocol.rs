//! MCP Protocol Routes — Phase 28
//!
//! API endpoints for managing MCP server connections, tools, and calls.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db::AppState;
use crate::services::mcp_protocol;

// ——— Query Params ———

#[derive(Deserialize)]
pub struct MCPListQuery {
    pub organization_id: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct MCPToolsQuery {
    pub server_id: Option<String>,
    pub is_enabled: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct MCPCallsQuery {
    pub server_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ——— Request/Response Bodies ———

#[derive(Deserialize)]
pub struct CreateMCPServerRequest {
    pub organization_id: String,
    pub name: String,
    pub server_url: String,
    pub transport_type: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMCPServerRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub server_url: Option<String>,
}

#[derive(Deserialize)]
pub struct CallToolRequest {
    pub tool_id: String,
    pub tool_name: String,
    pub input_data: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMCPSettingsRequest {
    pub enabled: Option<bool>,
    pub auto_discover_tools: Option<bool>,
    pub log_all_calls: Option<bool>,
    pub max_concurrent_calls: Option<i32>,
    pub timeout_seconds: Option<i32>,
}

// ——— Handlers ———

/// GET /mcp/summary
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let summary = mcp_protocol::get_mcp_summary(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

/// GET /mcp/servers
pub async fn get_servers(
    State(state): State<AppState>,
    Query(params): Query<MCPListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let servers = mcp_protocol::list_mcp_servers(
        &state,
        params.organization_id.as_deref(),
        params.is_active,
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(servers).unwrap_or_default()))
}

/// POST /mcp/servers
pub async fn create_server(
    State(state): State<AppState>,
    Json(body): Json<CreateMCPServerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = mcp_protocol::create_mcp_server(
        &state,
        &body.organization_id,
        &body.name,
        &body.server_url,
        body.transport_type.as_deref().unwrap_or("http"),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// PUT /mcp/servers/{id}
pub async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateMCPServerRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    mcp_protocol::update_mcp_server(
        &state,
        &id,
        body.name.as_deref(),
        body.is_active,
        body.server_url.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(StatusCode::OK)
}

/// DELETE /mcp/servers/{id}
pub async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    mcp_protocol::delete_mcp_server(&state, &id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(StatusCode::OK)
}

/// POST /mcp/servers/{id}/discover
pub async fn discover_tools(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = mcp_protocol::discover_tools(&state, &id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /mcp/tools
pub async fn get_tools(
    State(state): State<AppState>,
    Query(params): Query<MCPToolsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let tools = mcp_protocol::list_mcp_tools(
        &state,
        params.server_id.as_deref(),
        params.is_enabled,
        params.limit.unwrap_or(100),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(tools).unwrap_or_default()))
}

/// POST /mcp/servers/{id}/call
pub async fn call_tool(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CallToolRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = mcp_protocol::call_mcp_tool(
        &state,
        &id,
        &body.tool_id,
        &body.tool_name,
        body.input_data.as_deref().unwrap_or("{}"),
        body.conversation_id.as_deref(),
        body.contact_id.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /mcp/templates
pub async fn get_templates(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let templates = mcp_protocol::list_mcp_templates(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(templates).unwrap_or_default()))
}

/// GET /mcp/settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = mcp_protocol::get_mcp_settings(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap_or_default()))
}

/// PUT /mcp/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateMCPSettingsRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    mcp_protocol::update_mcp_settings(
        &state,
        body.enabled,
        body.auto_discover_tools,
        body.log_all_calls,
        body.max_concurrent_calls,
        body.timeout_seconds,
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(StatusCode::OK)
}

/// GET /mcp/calls
pub async fn get_call_logs(
    State(state): State<AppState>,
    Query(params): Query<MCPCallsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let logs = mcp_protocol::list_mcp_call_logs(
        &state,
        params.server_id.as_deref(),
        params.status.as_deref(),
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(logs).unwrap_or_default()))
}
