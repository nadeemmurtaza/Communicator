use newax_whatsapp_flows as whatsapp_flows;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.whatsapp_flows.v1");
}

use proto::{
    whats_app_flows_service_server::{WhatsAppFlowsService as WhatsAppFlowsServiceServerTrait, WhatsAppFlowsServiceServer},
    JsonResponse,
};

struct Svc {
    db: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl WhatsAppFlowsServiceServerTrait for Svc {
    async fn list_industries(
        &self,
        _req: Request<proto::ListIndustriesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.list_industries().await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_industry(
        &self,
        req: Request<proto::CreateIndustryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.create_industry(
            &r.name,
            &r.description,
            if r.has_icon { Some(&r.icon) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_industry(
        &self,
        req: Request<proto::UpdateIndustryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.update_industry(
            &r.industry_id,
            if r.has_name { Some(&r.name) } else { None },
            if r.has_description { Some(&r.description) } else { None },
            if r.has_icon { Some(&r.icon) } else { None },
            if r.has_is_active { Some(r.is_active.parse().unwrap_or(false)) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_industry(
        &self,
        req: Request<proto::DeleteIndustryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.delete_industry(&r.industry_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_template_groups(
        &self,
        req: Request<proto::ListTemplateGroupsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.list_template_groups(
            &r.brand_id,
            if r.has_industry_id { Some(&r.industry_id) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_template_group(
        &self,
        req: Request<proto::CreateTemplateGroupRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.create_template_group(
            &r.brand_id,
            if r.has_industry_id { Some(&r.industry_id) } else { None },
            &r.name,
            &r.description,
            r.sort_order,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_template_group(
        &self,
        req: Request<proto::UpdateTemplateGroupRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.update_template_group(
            &r.group_id,
            if r.has_name { Some(&r.name) } else { None },
            if r.has_description { Some(&r.description) } else { None },
            if r.has_sort_order { Some(r.sort_order.parse().unwrap_or(0)) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_template_group(
        &self,
        req: Request<proto::DeleteTemplateGroupRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.delete_template_group(&r.group_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_flows(
        &self,
        req: Request<proto::ListFlowsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.list_template_flows(
            &r.brand_id,
            if r.has_industry_id { Some(&r.industry_id) } else { None },
            if r.has_group_id { Some(&r.group_id) } else { None },
            if r.has_category { Some(&r.category) } else { None },
            if r.has_status { Some(&r.status) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_flow(
        &self,
        req: Request<proto::CreateFlowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.create_template_flow(
            &r.brand_id,
            if r.has_group_id { Some(&r.group_id) } else { None },
            if r.has_industry_id { Some(&r.industry_id) } else { None },
            &r.name,
            &r.flow_type,
            &r.category,
            &r.body,
            if r.has_header { Some(&r.header) } else { None },
            if r.has_footer { Some(&r.footer) } else { None },
            if r.has_buttons { Some(&r.buttons) } else { None },
            if r.has_sections { Some(&r.sections) } else { None },
            &r.language,
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_flow(
        &self,
        req: Request<proto::GetFlowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.get_template_flow(&r.flow_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_flow(
        &self,
        req: Request<proto::UpdateFlowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.update_template_flow(
            &r.flow_id,
            if r.has_name { Some(&r.name) } else { None },
            if r.has_body { Some(&r.body) } else { None },
            if r.has_header { Some(&r.header) } else { None },
            if r.has_footer { Some(&r.footer) } else { None },
            if r.has_buttons { Some(&r.buttons) } else { None },
            if r.has_sections { Some(&r.sections) } else { None },
            if r.has_status { Some(&r.status) } else { None },
            if r.has_is_active { Some(r.is_active.parse().unwrap_or(false)) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_flow(
        &self,
        req: Request<proto::DeleteFlowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.delete_template_flow(&r.flow_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn duplicate_flow(
        &self,
        req: Request<proto::DuplicateFlowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.duplicate_flow(&r.flow_id, &r.name).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn validate_flow(
        &self,
        req: Request<proto::ValidateFlowRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.validate_flow(&r.flow_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_flow_steps(
        &self,
        req: Request<proto::ListFlowStepsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.list_flow_steps(&r.flow_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_flow_step(
        &self,
        req: Request<proto::CreateFlowStepRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.create_flow_step(
            &r.flow_id,
            r.step_number,
            if r.has_condition { Some(&r.condition) } else { None },
            &r.action_type,
            if r.has_action_config { Some(&r.action_config) } else { None },
            if r.has_next_step_id { Some(&r.next_step_id) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_flow_step(
        &self,
        req: Request<proto::UpdateFlowStepRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.update_flow_step(
            &r.step_id,
            if r.has_step_number { Some(r.step_number.parse().unwrap_or(0)) } else { None },
            if r.has_condition { Some(&r.condition) } else { None },
            if r.has_action_type { Some(&r.action_type) } else { None },
            if r.has_action_config { Some(&r.action_config) } else { None },
            if r.has_next_step_id { Some(&r.next_step_id) } else { None },
        ).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_flow_step(
        &self,
        req: Request<proto::DeleteFlowStepRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = whatsapp_flows::WhatsAppFlowsService::new(self.db.clone());
        let result = svc.delete_flow_step(&r.step_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("failed to connect to database");

    let addr = "0.0.0.0:50100".parse()?;
    let svc = Svc { db: pool };

    info!("WhatsApp Flows gRPC server listening on {}", addr);
    Server::builder()
        .add_service(WhatsAppFlowsServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
