use newax_whatsapp_template_sync::WhatsAppTemplateSyncState;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.whatsapp_template_sync.v1");
}

use proto::whats_app_template_sync_service_server::{
    WhatsAppTemplateSyncService, WhatsAppTemplateSyncServiceServer,
};
use proto::{
    JsonResponse, SyncTemplatesFromMetaRequest, FetchMessageLimitsFromMetaRequest,
    UpdateMessageLimitsInDbRequest, CheckMessageLimitRequest, GetRemainingLimitsRequest,
    ListTemplatesRequest, SyncCustomerPhoneNumbersRequest,
};

struct Svc {
    state: WhatsAppTemplateSyncState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl WhatsAppTemplateSyncService for Svc {
    async fn sync_templates_from_meta(
        &self,
        request: Request<SyncTemplatesFromMetaRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.sync_templates_from_meta(&req.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn fetch_message_limits_from_meta(
        &self,
        request: Request<FetchMessageLimitsFromMetaRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.fetch_message_limits_from_meta(&req.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_message_limits_in_db(
        &self,
        request: Request<UpdateMessageLimitsInDbRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.update_message_limits_in_db(&req.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn check_message_limit(
        &self,
        request: Request<CheckMessageLimitRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.check_message_limit(&req.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!(result))))
    }

    async fn get_remaining_limits(
        &self,
        request: Request<GetRemainingLimitsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.get_remaining_limits(&req.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }

    async fn list_templates(
        &self,
        request: Request<ListTemplatesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let category = if req.category.is_empty() { None } else { Some(req.category.as_str()) };
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let result = self.state.list_templates(&req.brand_id, category, status).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!(result))))
    }

    async fn sync_customer_phone_numbers(
        &self,
        request: Request<SyncCustomerPhoneNumbersRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.sync_customer_phone_numbers(&req.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!(result))))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_whatsapp_template_sync=info".parse()?),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let addr = "[::1]:50104".parse()?;
    let svc = Svc {
        state: WhatsAppTemplateSyncState::new(pool),
    };

    info!("WhatsApp Template Sync service listening on {}", addr);

    Server::builder()
        .add_service(WhatsAppTemplateSyncServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
