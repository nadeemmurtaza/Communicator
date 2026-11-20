//! svc-external-otp — standalone gRPC server for external OTP integration (port 50108).

use std::env;
use tonic::{Request, Response, Status};
use serde_json::json;

mod external_otp {
    include!(concat!(env!("OUT_DIR"), "/newax.external_otp.v1.rs"));
}

use external_otp::external_otp_service_server::{ExternalOtpService, ExternalOtpServiceServer};
use external_otp::{
    JsonResponse, JsonBody, Empty, IdRequest, PublicKeyRequest, SecretKeyRequest,
    CreateIntegrationRequest as ProtoCreateIntegrationRequest,
    ExternalGenerateRequest as ProtoExternalGenerateRequest,
    ExternalVerifyRequest as ProtoExternalVerifyRequest,
};

use newax_external_otp::{
    ExternalOtpState, ExternalIntegration, CreateIntegrationRequest, ExternalGenerateRequest,
    ExternalVerifyRequest,
};

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ExternalOtpServer {
    state: ExternalOtpState,
}

impl ExternalOtpServer {
    fn new(state: ExternalOtpState) -> Self {
        Self { state }
    }
}

fn json_response(value: serde_json::Value) -> JsonResponse {
    JsonResponse {
        json: value.to_string(),
    }
}

fn to_status(e: String) -> Status {
    Status::invalid_argument(e)
}

#[tonic::async_trait]
impl ExternalOtpService for ExternalOtpServer {
    async fn create_integration(
        &self,
        req: Request<ProtoCreateIntegrationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: CreateIntegrationRequest = serde_json::from_str(&r.body.json).map_err(to_status)?;
        let resp = newax_external_otp::create_integration(&self.state, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(json!({
            "success": true,
            "id": resp.id,
            "public_key": resp.public_key,
            "secret_key": resp.secret_key,
            "webhook_secret": resp.webhook_secret,
            "message": resp.message,
        }))))
    }

    async fn list_integrations(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let integrations = newax_external_otp::list_by_brand(&self.state, &r.id).await.map_err(to_status)?;
        let items: Vec<serde_json::Value> = integrations.iter().map(|i| json!({
            "id": i.id,
            "platform_name": i.platform_name,
            "platform_url": i.platform_url,
            "display_name": i.display_name,
            "public_key": i.public_key,
            "webhook_url": i.webhook_url,
            "allowed_purposes": i.allowed_purposes,
            "allowed_channels": i.allowed_channels,
            "is_active": i.is_active,
            "rate_limit_per_minute": i.rate_limit_per_minute,
            "rate_limit_per_hour": i.rate_limit_per_hour,
            "last_used_at": i.last_used_at,
            "created_at": i.created_at,
        })).collect();
        Ok(Response::new(json_response(json!({
            "success": true,
            "integrations": items,
        }))))
    }

    async fn deactivate_integration(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        newax_external_otp::deactivate(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(json!({
            "success": true,
            "message": "Integration deactivated",
        }))))
    }

    async fn external_generate(
        &self,
        req: Request<ProtoExternalGenerateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: ExternalGenerateRequest = serde_json::from_str(&r.body.json).map_err(to_status)?;

        let integration = newax_external_otp::verify_secret_key(&self.state, &r.secret_key).await.map_err(to_status)?;
        let resp = newax_external_otp::external_generate(
            &self.state, &integration, &body, &r.ip_address, Some(r.user_agent),
        ).await.map_err(to_status)?;

        Ok(Response::new(json_response(json!({
            "success": resp.success,
            "otp_id": resp.otp_id,
            "channel": resp.channel,
            "message": resp.message,
            "expires_in_seconds": resp.expires_in_seconds,
            "error": resp.error,
        }))))
    }

    async fn external_verify(
        &self,
        req: Request<ProtoExternalVerifyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: ExternalVerifyRequest = serde_json::from_str(&r.body.json).map_err(to_status)?;

        let integration = newax_external_otp::verify_secret_key(&self.state, &r.secret_key).await.map_err(to_status)?;
        let resp = newax_external_otp::external_verify(
            &self.state, &integration, &body, &r.ip_address, Some(r.user_agent),
        ).await.map_err(to_status)?;

        Ok(Response::new(json_response(json!({
            "success": resp.success,
            "verified": resp.verified,
            "message": resp.message,
            "attempts_remaining": resp.attempts_remaining,
            "session_token": resp.session_token,
            "error": resp.error,
        }))))
    }

    async fn get_integration_info(
        &self,
        req: Request<PublicKeyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let integration = newax_external_otp::get_by_public_key(&self.state, &r.public_key).await.map_err(to_status)?;
        Ok(Response::new(json_response(json!({
            "success": true,
            "id": integration.id,
            "platform_name": integration.platform_name,
            "display_name": integration.display_name,
            "public_key": integration.public_key,
            "allowed_purposes": integration.allowed_purposes,
            "allowed_channels": integration.allowed_channels,
            "is_active": integration.is_active,
        }))))
    }

    async fn verify_secret_key(
        &self,
        req: Request<SecretKeyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let integration = newax_external_otp::verify_secret_key(&self.state, &r.secret_key).await.map_err(to_status)?;
        Ok(Response::new(json_response(json!({
            "success": true,
            "id": integration.id,
            "brand_id": integration.brand_id,
            "platform_name": integration.platform_name,
        }))))
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let otp_engine_url = env::var("OTP_ENGINE_URL").unwrap_or_else(|_| "http://localhost:50090".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "50108".to_string());

    let pool = sqlx::PgPool::connect(&database_url).await.expect("Failed to connect to database");

    let state = ExternalOtpState::new(pool, otp_engine_url);
    let server = ExternalOtpServer::new(state);

    let addr = format!("0.0.0.0:{}", port).parse()?;
    tracing::info!("svc-external-otp listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(ExternalOtpServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
