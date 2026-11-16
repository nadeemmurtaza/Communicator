use newax_crm_sync as crm_sync;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.crm_sync.v1");
}

use proto::{
    crm_sync_service_server::{CrmSyncService, CrmSyncServiceServer},
    Empty, IdRequest, JsonResponse, ListConnectionsRequest,
    CreateConnectionRequest, UpdateConnectionRequest, TriggerSyncRequest,
    ListSyncLogsRequest, ListFieldMappingsRequest, GetExternalRefsRequest,
    CreateExternalRefRequest,
};

struct Svc {
    state: crm_sync::CRMSyncState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

#[tonic::async_trait]
impl CrmSyncService for Svc {
    async fn get_summary(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = crm_sync::get_crm_sync_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_providers(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = crm_sync::get_crm_providers();
        Ok(Response::new(serialize_response(v)?))
    }

    async fn list_connections(
        &self,
        req: Request<ListConnectionsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::list_crm_connections(
            &self.state,
            r.provider.as_deref(),
            r.status.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn create_connection(
        &self,
        req: Request<CreateConnectionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::create_crm_connection(
            &self.state,
            &r.organization_id,
            &r.provider,
            &r.auth_type,
            r.credentials.as_deref(),
            r.config.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn update_connection(
        &self,
        req: Request<UpdateConnectionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::update_crm_connection_status(
            &self.state,
            &r.id,
            &r.status,
            r.credentials.as_deref(),
            r.last_error.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn delete_connection(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        crm_sync::delete_crm_connection(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(JsonResponse {
            json: serde_json::json!({"deleted": true}).to_string(),
        }))
    }

    async fn trigger_sync(
        &self,
        req: Request<TriggerSyncRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::trigger_sync(&self.state, &r.connection_id, &r.direction, &r.object_type)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn list_sync_logs(
        &self,
        req: Request<ListSyncLogsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::list_sync_logs(&self.state, r.connection_id.as_deref(), r.limit)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn list_field_mappings(
        &self,
        req: Request<ListFieldMappingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::list_field_mappings(
            &self.state,
            r.connection_id.as_deref(),
            r.object_type.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_external_refs(
        &self,
        req: Request<GetExternalRefsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::get_external_refs(&self.state, &r.local_type, &r.local_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn create_external_ref(
        &self,
        req: Request<CreateExternalRefRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = crm_sync::create_external_ref(
            &self.state,
            &r.connection_id,
            &r.local_type,
            &r.local_id,
            &r.crm_object_type,
            &r.crm_record_id,
            r.crm_record_url.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_crm_sync=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;
    let http_client = reqwest::Client::new();
    let default_tenant_key = std::env::var("DEFAULT_TENANT_KEY").unwrap_or_else(|_| "newax".to_string());

    let port: u16 = std::env::var("CRM_SYNC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50084);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = Svc {
        state: crm_sync::CRMSyncState { db, http_client, default_tenant_key },
    };

    info!(port, "svc-crm-sync starting (gRPC)");
    Server::builder()
        .add_service(CrmSyncServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
