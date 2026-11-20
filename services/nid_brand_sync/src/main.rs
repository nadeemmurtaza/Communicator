mod proto {
    include!(concat!(env!("OUT_DIR"), "/communicator/nid_brand_sync/v1/mod.rs"));
}

use nid_brand_sync::{
    NidBrandSyncService, BrandWebhookPayload, UpdateConnectionRequestDto, CreateRedirectRequestDto,
    NidBrand, BrandConnectionSettings, SyncLogEntry,
};
use proto::nid_brand_sync_service_server::{NidBrandSyncServiceServer, NidBrandSyncService as Trait};
use proto::{
    Empty, JsonResponse, WebhookRequest, ListBrandsRequest, BrandsResponse, IdRequest,
    BrandConnectionSettings as ProtoBrandConnectionSettings, UpdateConnectionRequest,
    GetSyncLogRequest, SyncLogResponse, SyncLogEntry as ProtoSyncLogEntry,
    CreateRedirectRequest, RedirectCallbackRequest, NidBrand as ProtoNidBrand,
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
struct NidBrandSyncServer {
    service: NidBrandSyncService,
}

impl NidBrandSyncServer {
    fn new(pool: PgPool) -> Self {
        Self {
            service: NidBrandSyncService::new(&pool),
        }
    }
}

#[tonic::async_trait]
impl Trait for NidBrandSyncServer {
    async fn webhook(
        &self,
        request: Request<WebhookRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let payload: BrandWebhookPayload = serde_json::from_str(&req.json)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON: {}", e)))?;

        self.service.process_webhook(&payload, "").await
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(Empty {}))
    }

    async fn list_brands(
        &self,
        request: Request<ListBrandsRequest>,
    ) -> Result<Response<BrandsResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = req.tenant_id.as_deref();

        let brands = self.service.list_brands(tenant_id).await
            .map_err(|e| Status::internal(e))?;

        let brands: Vec<ProtoNidBrand> = brands.into_iter().map(|b| ProtoNidBrand {
            id: b.id,
            nid_brand_id: b.nid_brand_id,
            nid_org_id: b.nid_org_id,
            nid_tenant_id: b.nid_tenant_id,
            name: b.name,
            slug: b.slug,
            description: b.description,
            bio: b.bio,
            website: b.website,
            email: b.email,
            phone: b.phone,
            primary_color: b.primary_color,
            secondary_color: b.secondary_color,
            active: b.active,
            synced_at: b.synced_at,
            created_at: b.created_at,
        }).collect();

        Ok(Response::new(BrandsResponse { brands }))
    }

    async fn get_brand(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<ProtoNidBrand>, Status> {
        let req = request.into_inner();
        let brand = self.service.get_brand_by_nid_id(&req.id).await
            .map_err(|e| Status::not_found(e))?;

        let pb = ProtoNidBrand {
            id: brand.id,
            nid_brand_id: brand.nid_brand_id,
            nid_org_id: brand.nid_org_id,
            nid_tenant_id: brand.nid_tenant_id,
            name: brand.name,
            slug: brand.slug,
            description: brand.description,
            bio: brand.bio,
            website: brand.website,
            email: brand.email,
            phone: brand.phone,
            primary_color: brand.primary_color,
            secondary_color: brand.secondary_color,
            active: brand.active,
            synced_at: brand.synced_at,
            created_at: brand.created_at,
        };

        Ok(Response::new(pb))
    }

    async fn get_connection(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ProtoBrandConnectionSettings>, Status> {
        let settings = self.service.get_connection_settings_public().await;

        let pb = ProtoBrandConnectionSettings {
            webhook_url: settings.webhook_url,
            webhook_secret: settings.webhook_secret,
            polling_interval_sec: settings.polling_interval_sec,
            enabled: settings.enabled,
            health_status: settings.health_status,
        };

        Ok(Response::new(pb))
    }

    async fn update_connection(
        &self,
        request: Request<UpdateConnectionRequest>,
    ) -> Result<Response<ProtoBrandConnectionSettings>, Status> {
        let req = request.into_inner();
        let dto: UpdateConnectionRequestDto = serde_json::from_str(&req.json)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON: {}", e)))?;

        let settings = self.service.update_connection_settings(&dto).await
            .map_err(|e| Status::internal(e))?;

        let pb = ProtoBrandConnectionSettings {
            webhook_url: settings.webhook_url,
            webhook_secret: settings.webhook_secret,
            polling_interval_sec: settings.polling_interval_sec,
            enabled: settings.enabled,
            health_status: settings.health_status,
        };

        Ok(Response::new(pb))
    }

    async fn health_check(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let result = self.service.run_health_check().await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&result).unwrap_or_default();

        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_sync_log(
        &self,
        request: Request<GetSyncLogRequest>,
    ) -> Result<Response<SyncLogResponse>, Status> {
        let req = request.into_inner();
        let entries = self.service.get_sync_log(req.limit).await
            .map_err(|e| Status::internal(e))?;

        let proto_entries: Vec<ProtoSyncLogEntry> = entries.into_iter().map(|e| ProtoSyncLogEntry {
            id: e.id,
            nid_brand_id: e.nid_brand_id,
            event_type: e.event_type,
            status: e.status,
            error_msg: e.error_msg,
            created_at: e.created_at,
        }).collect();

        Ok(Response::new(SyncLogResponse { entries: proto_entries }))
    }

    async fn create_redirect(
        &self,
        request: Request<CreateRedirectRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let dto: CreateRedirectRequestDto = serde_json::from_str(&req.json)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON: {}", e)))?;

        let result = self.service.create_redirect_session(&dto).await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&result).unwrap_or_default();

        Ok(Response::new(JsonResponse { json }))
    }

    async fn redirect_callback(
        &self,
        request: Request<RedirectCallbackRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();

        let result = self.service.redirect_callback(&req.state, &req.brand_id).await
            .map_err(|e| Status::internal(e))?;

        let json = serde_json::to_string(&result).unwrap_or_default();

        Ok(Response::new(JsonResponse { json }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_nid_brand_sync=info".parse()?),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/communicator".to_string());

    let pool = PgPool::connect(&database_url).await
        .expect("Failed to connect to database");

    let addr = "[::1]:50103".parse().expect("Invalid address");
    let server = NidBrandSyncServer::new(pool);

    tracing::info!("nID Brand Sync service listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(NidBrandSyncServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
