mod gen {
    include!("gen/nid_organization_sync.rs");
}

use gen::nid_organization_sync::v1::nid_organization_sync_server::{NidOrganizationSync, NidOrganizationSyncServer};
use gen::nid_organization_sync::v1::{
    Empty, JsonResponse, IdRequest, WebhookRequest, UpdateConnectionRequest,
    GetSyncLogRequest, CreateRedirectRequest, RedirectCallbackRequest,
};
use sqlx::PgPool;
use std::env;
use tonic::{Request, Response, Status};

pub struct NidOrganizationSyncService {
    pool: PgPool,
}

impl NidOrganizationSyncService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl NidOrganizationSync for NidOrganizationSyncService {
    async fn webhook(
        &self,
        request: Request<WebhookRequest>,
    ) -> Result<Response<Empty>, Status> {
        let msg = request.into_inner();
        let payload: serde_json::Value = serde_json::from_str(&msg.json)
            .map_err(|e| Status::invalid_argument(format!("invalid json: {}", e)))?;

        newax_nid_organization_sync::handle_webhook(&self.pool, &payload)
            .await
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(Empty {}))
    }

    async fn list_organizations(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let orgs = newax_nid_organization_sync::list_organizations(&self.pool)
            .await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&orgs)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_organization(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;

        let org = newax_nid_organization_sync::fetch_organization_by_nid_id(&self.pool, &id)
            .await
            .map_err(|e| Status::not_found(e))?;

        let json = serde_json::to_string(&org)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_connection(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        use newax_nid_organization_sync::NidOrganizationConnection;

        let conn = sqlx::query_as!(
            NidOrganizationConnection,
            r#"SELECT id::TEXT as "id!", nid_base_url as "nID_base_url", webhook_secret, auto_sync_enabled,
                      auto_link_new_organizations, sync_interval_seconds, connection_status,
                      last_error, last_health_check_at::TEXT as last_health_check_at
               FROM nid_organization_connection LIMIT 1"#
        )
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        let json = serde_json::to_string(&conn)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_connection(
        &self,
        request: Request<UpdateConnectionRequest>,
    ) -> Result<Response<Empty>, Status> {
        let msg = request.into_inner();
        let req: newax_nid_organization_sync::UpdateConnectionRequestDto =
            serde_json::from_str(&msg.json)
                .map_err(|e| Status::invalid_argument(format!("invalid json: {}", e)))?;

        newax_nid_organization_sync::handle_update_connection(&self.pool, req)
            .await
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(Empty {}))
    }

    async fn get_health(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let health = newax_nid_organization_sync::get_health(&self.pool)
            .await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&health)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_sync_log(
        &self,
        request: Request<GetSyncLogRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let limit = request.into_inner().limit;

        let logs = newax_nid_organization_sync::get_sync_log(&self.pool, limit)
            .await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&logs)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_redirect(
        &self,
        request: Request<CreateRedirectRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_nid_organization_sync::CreateRedirectRequestDto =
            serde_json::from_str(&msg.json)
                .map_err(|e| Status::invalid_argument(format!("invalid json: {}", e)))?;

        let session = newax_nid_organization_sync::create_redirect_session(&self.pool, req)
            .await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&session)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }

    async fn redirect_callback(
        &self,
        request: Request<RedirectCallbackRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();

        let org_id = if msg.organization_id.is_empty() {
            None
        } else {
            Some(msg.organization_id.as_str())
        };
        let error = if msg.error.is_empty() {
            None
        } else {
            Some(msg.error.as_str())
        };

        let result = newax_nid_organization_sync::handle_redirect_callback(
            &self.pool,
            &msg.state,
            org_id,
            error,
        )
        .await
        .map_err(|e| Status::invalid_argument(e))?;

        let json = serde_json::to_string(&result)
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(JsonResponse { json }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://newax:newax@localhost/communicator".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let addr = "127.0.0.1:50102".parse()?;
    let service = NidOrganizationSyncService::new(pool);

    tonic::transport::Server::builder()
        .add_service(NidOrganizationSyncServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
