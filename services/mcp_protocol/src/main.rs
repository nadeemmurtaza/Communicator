use newax_mcp_protocol as mcp_protocol;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.mcp_protocol.v1");
}

use proto::{
    mcp_protocol_service_server::{McpProtocolService, McpProtocolServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: mcp_protocol::MCPProtocolState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl McpProtocolService for Svc {
    async fn get_summary(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let summary = mcp_protocol::get_mcp_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(summary).map_err(to_status)?)))
    }

    async fn list_servers(
        &self,
        req: Request<proto::ListServersRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let servers = mcp_protocol::list_mcp_servers(
            &self.state,
            if r.organization_id.is_empty() { None } else { Some(&r.organization_id) },
            if r.has_is_active { Some(r.is_active) } else { None },
            r.limit,
            r.offset,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(servers).map_err(to_status)?)))
    }

    async fn create_server(
        &self,
        req: Request<proto::CreateServerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = mcp_protocol::create_mcp_server(
            &self.state,
            &r.organization_id,
            &r.name,
            &r.server_url,
            &r.transport_type,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_server(
        &self,
        req: Request<proto::UpdateServerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        mcp_protocol::update_mcp_server(
            &self.state,
            &r.id,
            if r.has_name { Some(&r.name) } else { None },
            if r.has_is_active { Some(r.is_active) } else { None },
            if r.has_server_url { Some(&r.server_url) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "ok": true }))))
    }

    async fn delete_server(
        &self,
        req: Request<proto::DeleteServerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        mcp_protocol::delete_mcp_server(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "ok": true }))))
    }

    async fn discover_tools(
        &self,
        req: Request<proto::DiscoverToolsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = mcp_protocol::discover_tools(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_tools(
        &self,
        req: Request<proto::ListToolsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let tools = mcp_protocol::list_mcp_tools(
            &self.state,
            if r.server_id.is_empty() { None } else { Some(&r.server_id) },
            if r.has_is_enabled { Some(r.is_enabled) } else { None },
            r.limit,
            r.offset,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(tools).map_err(to_status)?)))
    }

    async fn call_tool(
        &self,
        req: Request<proto::CallToolRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = mcp_protocol::call_mcp_tool(
            &self.state,
            &r.server_id,
            &r.tool_id,
            &r.tool_name,
            &r.input_data,
            if r.conversation_id.is_empty() { None } else { Some(&r.conversation_id) },
            if r.contact_id.is_empty() { None } else { Some(&r.contact_id) },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_templates(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let templates = mcp_protocol::list_mcp_templates(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(templates).map_err(to_status)?)))
    }

    async fn get_settings(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let settings = mcp_protocol::get_mcp_settings(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(settings).map_err(to_status)?)))
    }

    async fn update_settings(
        &self,
        req: Request<proto::UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        mcp_protocol::update_mcp_settings(
            &self.state,
            if r.has_enabled { Some(r.enabled) } else { None },
            if r.has_auto_discover_tools { Some(r.auto_discover_tools) } else { None },
            if r.has_log_all_calls { Some(r.log_all_calls) } else { None },
            if r.has_max_concurrent_calls { Some(r.max_concurrent_calls) } else { None },
            if r.has_timeout_seconds { Some(r.timeout_seconds) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "ok": true }))))
    }

    async fn list_call_logs(
        &self,
        req: Request<proto::ListCallLogsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let logs = mcp_protocol::list_mcp_call_logs(
            &self.state,
            if r.server_id.is_empty() { None } else { Some(&r.server_id) },
            if r.status.is_empty() { None } else { Some(&r.status) },
            r.limit,
            r.offset,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(logs).map_err(to_status)?)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Cannot connect to database: {e}"))?;

    let state = mcp_protocol::MCPProtocolState { db: pool };

    let addr = "[::1]:50097".parse().unwrap();
    info!("mcp-protocol listening on {}", addr);

    Server::builder()
        .add_service(McpProtocolServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
