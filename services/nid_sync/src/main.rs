use newax_nid_sync as nid_sync;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.nid_sync.v1");
}

use proto::nid_sync_service_server::{NidSyncService as NidSyncServiceTrait, NidSyncServiceServer};
use proto::{Empty, IdRequest, JsonResponse, SetOrderPlatformInfoRequest};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

#[tonic::async_trait]
impl NidSyncServiceTrait for Svc {
    async fn get_launchpad_overview(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let overview = svc.get_launchpad_overview(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(overview)?))
    }

    async fn get_personal_profile(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let overview = svc.get_personal_profile_overview(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(overview)?))
    }

    async fn get_work_profile(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let overview = svc.get_work_profile_overview(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(overview)?))
    }

    async fn get_user_organizations(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let orgs = svc.get_user_organizations(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(orgs)?))
    }

    async fn get_user_memberships(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let memberships = svc.get_user_memberships(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(memberships)?))
    }

    async fn get_invoices(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let invoices = svc.get_invoices_by_tenant(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(invoices)?))
    }

    async fn get_payments(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let payments = svc.get_payments_by_tenant(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payments)?))
    }

    async fn get_subscriptions(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let subs = svc.get_subscriptions_by_tenant(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(subs)?))
    }

    async fn set_order_platform_info(&self, req: Request<SetOrderPlatformInfoRequest>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        svc.set_order_platform_info(
            &r.order_id,
            &r.platform_name,
            &r.platform_type,
            &r.brand_name,
            r.shop_name.as_deref(),
        ).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_tenants(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let svc = nid_sync::NidSyncService::new(self.pool.clone());
        let tenants = svc.get_tenants().await.map_err(to_status)?;
        Ok(Response::new(serialize_response(tenants)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "svc-nid-sync=debug,tonic=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let port: u16 = std::env::var("NID_SYNC_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50097);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!("NID Sync service listening on {}", addr);

    Server::builder()
        .add_service(NidSyncServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;

    Ok(())
}
