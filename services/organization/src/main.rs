//! Organization standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use gen::organization::v1::organization_service_server::{
    OrganizationService, OrganizationServiceServer,
};
use gen::organization::v1::*;
use newax_organization::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct OrganizationServerImpl {
    state: OrganizationState,
}

#[tonic::async_trait]
impl OrganizationService for OrganizationServerImpl {
    async fn get_profile(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let (status, body) = get_profile(&self.state).await;
        if (200..300).contains(&status) {
            Ok(tonic::Response::new(JsonResponse { json: body }))
        } else {
            Err(tonic::Status::internal(body))
        }
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body = request.into_inner().json;
        let (status, resp_body) = update_profile(&self.state, &body).await;
        if (200..300).contains(&status) {
            Ok(tonic::Response::new(JsonResponse { json: resp_body }))
        } else {
            Err(tonic::Status::invalid_argument(resp_body))
        }
    }

    async fn get_settings(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let (status, body) = get_settings(&self.state).await;
        if (200..300).contains(&status) {
            Ok(tonic::Response::new(JsonResponse { json: body }))
        } else {
            Err(tonic::Status::internal(body))
        }
    }

    async fn update_settings(
        &self,
        request: Request<UpdateSettingsRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body = request.into_inner().json;
        let (status, resp_body) = update_settings_handler(&self.state, &body).await;
        if (200..300).contains(&status) {
            Ok(tonic::Response::new(JsonResponse { json: resp_body }))
        } else {
            Err(tonic::Status::invalid_argument(resp_body))
        }
    }

    async fn get_stats(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let (status, body) = get_stats(&self.state).await;
        if (200..300).contains(&status) {
            Ok(tonic::Response::new(JsonResponse { json: body }))
        } else {
            Err(tonic::Status::internal(body))
        }
    }

    async fn verify_channel(
        &self,
        request: Request<VerifyChannelRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let channel = request.into_inner().channel;
        let (status, body) = verify_channel(&self.state, &channel).await;
        if (200..300).contains(&status) {
            Ok(tonic::Response::new(JsonResponse { json: body }))
        } else {
            Err(tonic::Status::invalid_argument(body))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let state = OrganizationState {
        db: pool,
        default_tenant_key: std::env::var("DEFAULT_TENANT_KEY").unwrap_or_default(),
        meta_whatsapp_access_token: std::env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default(),
        meta_whatsapp_phone_number_id: std::env::var("META_WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default(),
        meta_app_secret: std::env::var("META_APP_SECRET").unwrap_or_default(),
        meta_webhook_verify_token: std::env::var("META_WEBHOOK_VERIFY_TOKEN").unwrap_or_default(),
        twilio_account_sid: std::env::var("TWILIO_ACCOUNT_SID").unwrap_or_default(),
        twilio_auth_token: std::env::var("TWILIO_AUTH_TOKEN").unwrap_or_default(),
    };

    let server = OrganizationServerImpl { state };
    let addr = "[::1]:50121".parse()?;
    info!("organization service listening on {}", addr);

    OrganizationServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
