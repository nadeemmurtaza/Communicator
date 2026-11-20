mod gen;

use gen::sso::v1::sso_service_server::{SsoService, SsoServiceServer};
use gen::sso::v1::{
    JsonResponse, JsonBody, EmptyRequest, UpdateConfigRequest, SpMetadataRequest,
    InitiateLoginRequest, AcsRequest, TestConnectionRequest, CheckEmailRequest,
};
use newax_sso::SsoState;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use tracing_subscriber::EnvFilter;

#[tonic::async_trait]
impl SsoService for SsoState {
    async fn get_config(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match newax_sso::get_config(self).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse {
                json: val.to_string(),
            })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn update_config(
        &self,
        request: tonic::Request<UpdateConfigRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let payload: serde_json::Value =
            serde_json::from_str(&req.body.json).map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match newax_sso::update_config(self, &payload).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse {
                json: val.to_string(),
            })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn sp_metadata(
        &self,
        request: tonic::Request<SpMetadataRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let headers: HashMap<String, String> =
            serde_json::from_str(&req.headers_json).unwrap_or_default();
        match newax_sso::sp_metadata(self, &headers).await {
            Ok(xml) => Ok(tonic::Response::new(JsonResponse {
                json: serde_json::json!({ "xml": xml }).to_string(),
            })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn initiate_login(
        &self,
        request: tonic::Request<InitiateLoginRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let headers: HashMap<String, String> =
            serde_json::from_str(&req.headers_json).unwrap_or_default();
        match newax_sso::initiate_login(self, &headers, Some(req.email), Some(req.relay_state)).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse {
                json: val.to_string(),
            })),
            Err(e) => Err(tonic::Status::invalid_argument(e)),
        }
    }

    async fn assertion_consumer_service(
        &self,
        request: tonic::Request<AcsRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let headers: HashMap<String, String> =
            serde_json::from_str(&req.headers_json).unwrap_or_default();
        let content_type = headers.get("content-type").cloned().unwrap_or_default();
        match newax_sso::assertion_consumer_service(self, &headers, &content_type, &req.body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse {
                json: val.to_string(),
            })),
            Err(e) => Err(tonic::Status::invalid_argument(e)),
        }
    }

    async fn test_connection(
        &self,
        request: tonic::Request<TestConnectionRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let headers: HashMap<String, String> =
            serde_json::from_str(&req.headers_json).unwrap_or_default();
        let payload: serde_json::Value =
            serde_json::from_str(&req.body.json).unwrap_or_default();
        match newax_sso::test_connection(self, &headers, &payload).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse {
                json: val.to_string(),
            })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn check_email(
        &self,
        request: tonic::Request<CheckEmailRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let headers: HashMap<String, String> =
            serde_json::from_str(&req.headers_json).unwrap_or_default();
        match newax_sso::check_email(self, &headers, &req.email).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse {
                json: val.to_string(),
            })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let state = SsoState::new(pool);

    let addr = "[::1]:50109".parse()?;
    tracing::info!("SSO service listening on {}", addr);

    SsoServiceServer::bind()
        .add_service(SsoServiceServer::new(state))
        .serve(addr)
        .await?;

    Ok(())
}
