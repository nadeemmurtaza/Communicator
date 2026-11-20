//! MCP Protocol routes — thin gRPC client forwarding to svc-mcp-protocol.

use serde_json::{json, Value};
use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
};
use crate::db::AppState;
use newax_proto::mcp_protocol::v1::mcp_protocol_service_client::McpProtocolServiceClient;

async fn client(state: &AppState) -> Result<McpProtocolServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.mcp_protocol_grpc_url.is_empty() {
        "http://127.0.0.1:50097".to_string()
    } else {
        state.config.mcp_protocol_grpc_url.clone()
    };
    McpProtocolServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("mcp_protocol gRPC connect failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": "mcp_protocol service unavailable" })))
        })
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.get_summary(newax_proto::mcp_protocol::v1::Empty {}).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Servers
// ---------------------------------------------------------------------------

pub async fn get_servers(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let organization_id = params.get("organization_id").map(|s| s.as_str().to_string()).unwrap_or_default();
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i32 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut cli = client(&state).await?;
    let resp = cli.list_servers(newax_proto::mcp_protocol::v1::ListServersRequest {
        organization_id,
        is_active: is_active.unwrap_or(false),
        has_is_active: is_active.is_some(),
        limit,
        offset,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn create_server(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let organization_id = body.get("organization_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let server_url = body.get("server_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let transport_type = body.get("transport_type").and_then(|v| v.as_str()).unwrap_or("http").to_string();

    if name.is_empty() || server_url.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "name and server_url are required" }))));
    }

    let mut cli = client(&state).await?;
    let resp = cli.create_server(newax_proto::mcp_protocol::v1::CreateServerRequest {
        organization_id, name, server_url, transport_type,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let name = body.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let is_active = body.get("is_active").and_then(|v| v.as_bool());
    let server_url = body.get("server_url").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut cli = client(&state).await?;
    let resp = cli.update_server(newax_proto::mcp_protocol::v1::UpdateServerRequest {
        id,
        name: name.clone().unwrap_or_default(),
        has_name: name.is_some(),
        is_active: is_active.unwrap_or(false),
        has_is_active: is_active.is_some(),
        server_url: server_url.clone().unwrap_or_default(),
        has_server_url: server_url.is_some(),
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.delete_server(newax_proto::mcp_protocol::v1::DeleteServerRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn discover_tools(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.discover_tools(newax_proto::mcp_protocol::v1::DiscoverToolsRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

pub async fn get_tools(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let server_id = params.get("server_id").map(|s| s.as_str().to_string()).unwrap_or_default();
    let is_enabled = params.get("is_enabled").and_then(|s| s.parse::<bool>().ok());
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(100);
    let offset: i32 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut cli = client(&state).await?;
    let resp = cli.list_tools(newax_proto::mcp_protocol::v1::ListToolsRequest {
        server_id,
        is_enabled: is_enabled.unwrap_or(false),
        has_is_enabled: is_enabled.is_some(),
        limit,
        offset,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn call_tool(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tool_id = body.get("tool_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tool_name = body.get("tool_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let input_data = body.get("input_data").and_then(|v| v.as_str()).unwrap_or("{}").to_string();
    let conversation_id = body.get("conversation_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let contact_id = body.get("contact_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut cli = client(&state).await?;
    let resp = cli.call_tool(newax_proto::mcp_protocol::v1::CallToolRequest {
        server_id, tool_id, tool_name, input_data, conversation_id, contact_id,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

pub async fn get_templates(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_templates(newax_proto::mcp_protocol::v1::Empty {}).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.get_settings(newax_proto::mcp_protocol::v1::Empty {}).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let enabled = body.get("enabled").and_then(|v| v.as_bool());
    let auto_discover_tools = body.get("auto_discover_tools").and_then(|v| v.as_bool());
    let log_all_calls = body.get("log_all_calls").and_then(|v| v.as_bool());
    let max_concurrent_calls = body.get("max_concurrent_calls").and_then(|v| v.as_i64()).map(|v| v as i32);
    let timeout_seconds = body.get("timeout_seconds").and_then(|v| v.as_i64()).map(|v| v as i32);

    let mut cli = client(&state).await?;
    let resp = cli.update_settings(newax_proto::mcp_protocol::v1::UpdateSettingsRequest {
        enabled: enabled.unwrap_or(false),
        has_enabled: enabled.is_some(),
        auto_discover_tools: auto_discover_tools.unwrap_or(false),
        has_auto_discover_tools: auto_discover_tools.is_some(),
        log_all_calls: log_all_calls.unwrap_or(false),
        has_log_all_calls: log_all_calls.is_some(),
        max_concurrent_calls: max_concurrent_calls.unwrap_or(0),
        has_max_concurrent_calls: max_concurrent_calls.is_some(),
        timeout_seconds: timeout_seconds.unwrap_or(0),
        has_timeout_seconds: timeout_seconds.is_some(),
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Call logs
// ---------------------------------------------------------------------------

pub async fn get_call_logs(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let server_id = params.get("server_id").map(|s| s.as_str().to_string()).unwrap_or_default();
    let status = params.get("status").map(|s| s.as_str().to_string()).unwrap_or_default();
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i32 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut cli = client(&state).await?;
    let resp = cli.list_call_logs(newax_proto::mcp_protocol::v1::ListCallLogsRequest {
        server_id, status, limit, offset,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}
