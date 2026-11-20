use std::env;
use std::net::SocketAddr;

use anyhow::Context;
use sqlx::PgPool;
use tonic::transport::Server;
use tracing::info;

use newax_meta_sync::{MetaSyncState, fetch_meta_business_profile, resolve_meta_credentials, refresh_channel_info};

mod meta_sync_svc {
    include!(concat!(env!("OUT_DIR"), "/newax.meta_sync.v1.rs"));
}

use meta_sync_svc::meta_sync_service_server::{MetaSyncServiceServer, MetaSyncService};
use meta_sync_svc::JsonResponse;
use meta_sync_svc::FetchBusinessProfileRequest;
use meta_sync_svc::ResolveMetaCredentialsRequest;
use meta_sync_svc::RefreshChannelInfoRequest;

#[derive(Clone)]
struct MetaSyncServer {
    state: MetaSyncState,
}

#[tonic::async_trait]
impl MetaSyncService for MetaSyncServer {
    async fn fetch_business_profile(
        &self,
        request: tonic::Request<FetchBusinessProfileRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let profile = fetch_meta_business_profile(
            &self.state,
            &req.access_token,
            &req.phone_number_id,
            &req.api_version,
        )
        .await;
        let json = serde_json::to_string(&profile).unwrap_or_default();
        Ok(tonic::Response::new(JsonResponse { json }))
    }

    async fn resolve_meta_credentials(
        &self,
        _request: tonic::Request<ResolveMetaCredentialsRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let (token, phone_id) = resolve_meta_credentials(&self.state).await;
        let json = serde_json::to_string(&serde_json::json!({
            "access_token": token,
            "phone_number_id": phone_id,
        })).unwrap_or_default();
        Ok(tonic::Response::new(JsonResponse { json }))
    }

    async fn refresh_channel_info(
        &self,
        _request: tonic::Request<RefreshChannelInfoRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let value = refresh_channel_info(&self.state).await;
        let json = serde_json::to_string(&value).unwrap_or_default();
        Ok(tonic::Response::new(JsonResponse { json }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_url = env::var("DATABASE_URL")
        .context("DATABASE_URL is required")?;
    let pool = PgPool::connect(&db_url)
        .await
        .context("Failed to connect to database")?;

    let http_client = reqwest::Client::new();
    let meta_whatsapp_access_token = env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default();
    let meta_whatsapp_phone_number_id = env::var("META_WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default();
    let meta_whatsapp_api_version = env::var("META_WHATSAPP_API_VERSION").unwrap_or_else(|_| "v18.0".to_string());

    let state = MetaSyncState {
        db: pool,
        http_client,
        meta_whatsapp_access_token,
        meta_whatsapp_phone_number_id,
        meta_whatsapp_api_version,
    };

    let addr: SocketAddr = "127.0.0.1:50113"
        .parse()
        .context("Invalid socket address")?;

    let server = MetaSyncServer { state };

    info!("Meta sync service listening on {}", addr);
    Server::builder()
        .add_service(MetaSyncServiceServer::new(server))
        .serve(addr)
        .await
        .context("Server failed")?;

    Ok(())
}
