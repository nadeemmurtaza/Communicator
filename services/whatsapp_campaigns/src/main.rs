use newax_whatsapp_campaigns as whatsapp_campaigns;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.whatsapp_campaigns.v1");
}

use proto::{
    whats_app_campaigns_service_server::{WhatsAppCampaignsService, WhatsAppCampaignsServiceServer},
    JsonResponse,
    ListCampaignsRequest, CreateCampaignRequest, GetCampaignRequest, DeleteCampaignRequest,
    StartCampaignRequest, PauseCampaignRequest, ResumeCampaignRequest, CancelCampaignRequest,
    GetCampaignContactsRequest, AddCampaignContactsRequest, RetryCampaignRequest, GetCampaignAnalyticsRequest,
};

struct Svc {
    state: whatsapp_campaigns::WhatsAppCampaignsState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

#[tonic::async_trait]
impl WhatsAppCampaignsService for Svc {
    async fn list_campaigns(
        &self,
        req: Request<ListCampaignsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let status = if r.status.is_empty() { None } else { Some(r.status.as_str()) };
        let v = whatsapp_campaigns::WhatsAppCampaignsState::list_campaigns(
            &self.state, &r.brand_id, status,
        ).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn create_campaign(
        &self,
        req: Request<CreateCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let template_flow_id = if r.template_flow_id.is_empty() { None } else { Some(r.template_flow_id.as_str()) };
        let scheduled_at = if r.scheduled_at.is_empty() { None } else { Some(r.scheduled_at.as_str()) };
        let v = whatsapp_campaigns::WhatsAppCampaignsState::create_campaign(
            &self.state, &r.brand_id, &r.name, &r.campaign_type, template_flow_id, scheduled_at,
        ).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_campaign(
        &self,
        req: Request<GetCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = whatsapp_campaigns::WhatsAppCampaignsState::get_campaign(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn delete_campaign(
        &self,
        req: Request<DeleteCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        whatsapp_campaigns::WhatsAppCampaignsState::delete_campaign(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"deleted": true}))?))
    }

    async fn start_campaign(
        &self,
        req: Request<StartCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        whatsapp_campaigns::WhatsAppCampaignsState::start_campaign(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"status": "running"}))?))
    }

    async fn pause_campaign(
        &self,
        req: Request<PauseCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        whatsapp_campaigns::WhatsAppCampaignsState::pause_campaign(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"status": "paused"}))?))
    }

    async fn resume_campaign(
        &self,
        req: Request<ResumeCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        whatsapp_campaigns::WhatsAppCampaignsState::resume_campaign(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"status": "running"}))?))
    }

    async fn cancel_campaign(
        &self,
        req: Request<CancelCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        whatsapp_campaigns::WhatsAppCampaignsState::cancel_campaign(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"status": "cancelled"}))?))
    }

    async fn get_campaign_contacts(
        &self,
        req: Request<GetCampaignContactsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let contacts = whatsapp_campaigns::WhatsAppCampaignsState::get_campaign_contacts(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"contacts": contacts}))?))
    }

    async fn add_campaign_contacts(
        &self,
        req: Request<AddCampaignContactsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let count = whatsapp_campaigns::WhatsAppCampaignsState::add_contacts_to_campaign(
            &self.state, &r.campaign_id, &r.contact_ids,
        ).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"added": count}))?))
    }

    async fn retry_campaign(
        &self,
        req: Request<RetryCampaignRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let count = whatsapp_campaigns::WhatsAppCampaignsState::retry_failed_messages(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({"retried": count}))?))
    }

    async fn get_campaign_analytics(
        &self,
        req: Request<GetCampaignAnalyticsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = whatsapp_campaigns::WhatsAppCampaignsState::get_campaign_analytics(&self.state, &r.campaign_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_whatsapp_campaigns=info".parse()?),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let state = whatsapp_campaigns::WhatsAppCampaignsState::new(pool);
    let svc = Svc { state };

    let addr = "[::1]:50107".parse()?;
    info!("WhatsApp Campaigns gRPC server listening on {}", addr);
    Server::builder()
        .add_service(WhatsAppCampaignsServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
