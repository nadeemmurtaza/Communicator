use newax_nid_tenant_sync::NidTenantSyncService as BusinessService;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.nid_tenant_sync.v1");
}

use proto::nid_tenant_sync_service_server::{NidTenantSyncService, NidTenantSyncServiceServer};
use proto::{
    JsonResponse,
    ProcessWebhookRequest, ListTenantsRequest, GetTenantRequest, GetConnectionHealthRequest,
    UpdateConnectionSettingsRequest, CreateRedirectSessionRequest, HandleRedirectCallbackRequest,
    GetRedirectSessionRequest, GetSyncLogRequest,
};

struct Svc {
    service: BusinessService,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl NidTenantSyncService for Svc {
    async fn process_webhook(&self, request: Request<ProcessWebhookRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.process_webhook(&req.payload, &req.signature)
            .await
            .map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(json_response(json)))
    }

    async fn list_tenants(&self, _request: Request<ListTenantsRequest>) -> Result<Response<JsonResponse>, Status> {
        let tenants = self.service.list_tenants().await.map_err(to_status)?;
        let json = serde_json::json!({
            "tenants": tenants,
            "count": tenants.len()
        });
        Ok(Response::new(json_response(json)))
    }

    async fn get_tenant(&self, request: Request<GetTenantRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let tenant = self.service.get_tenant(&req.tenant_id).await.map_err(|e| Status::not_found(e))?;
        let json = match tenant {
            Some(t) => serde_json::to_value(t).map_err(to_status)?,
            None => serde_json::json!({"error": "Tenant not found"}),
        };
        Ok(Response::new(json_response(json)))
    }

    async fn get_connection_health(&self, _request: Request<GetConnectionHealthRequest>) -> Result<Response<JsonResponse>, Status> {
        let health = self.service.check_connection_health().await;
        Ok(Response::new(json_response(health)))
    }

    async fn update_connection_settings(&self, request: Request<UpdateConnectionSettingsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let body: newax_nid_tenant_sync::UpdateConnectionSettingsRequest =
            serde_json::from_str(&req.body.unwrap().json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let settings = self.service.update_connection_settings(&body).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(settings).map_err(to_status)?)))
    }

    async fn create_redirect_session(&self, request: Request<CreateRedirectSessionRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let body: newax_nid_tenant_sync::CreateRedirectSessionRequest =
            serde_json::from_str(&req.body.unwrap().json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let session = self.service.create_redirect_session(&body).await.map_err(to_status)?;
        let settings = self.service.get_connection_settings().await;
        let json = serde_json::json!({
            "session": session,
            "redirect_url": format!(
                "{}/tenant-create?state={}&return_url={}",
                settings.nID_base_url,
                session.state_token,
                urlencoding::encode(&session.return_url)
            )
        });
        Ok(Response::new(json_response(json)))
    }

    async fn handle_redirect_callback(&self, request: Request<HandleRedirectCallbackRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = if req.tenant_id.is_empty() { None } else { Some(req.tenant_id.as_str()) };
        let error = if req.error.is_empty() { None } else { Some(req.error.as_str()) };
        let result = self.service.handle_redirect_callback(&req.state, tenant_id, error)
            .await
            .map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(json_response(result)))
    }

    async fn get_redirect_session(&self, request: Request<GetRedirectSessionRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match self.service.get_redirect_session(&req.state).await {
            Some(session) => Ok(Response::new(json_response(serde_json::to_value(session).map_err(to_status)?))),
            None => Err(Status::not_found("Redirect session not found")),
        }
    }

    async fn get_sync_log(&self, request: Request<GetSyncLogRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let event_type = if req.event_type.is_empty() { None } else { Some(req.event_type.as_str()) };
        let logs = self.service.get_sync_log(req.limit, event_type).await.map_err(to_status)?;
        let json = serde_json::json!({
            "logs": logs,
            "count": logs.len()
        });
        Ok(Response::new(json_response(json)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_nid_tenant_sync=info".parse()?),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let addr = "[::1]:50103".parse()?;
    let svc = Svc {
        service: BusinessService::new(pool),
    };

    info!("nID Tenant Sync service listening on {}", addr);

    Server::builder()
        .add_service(NidTenantSyncServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
