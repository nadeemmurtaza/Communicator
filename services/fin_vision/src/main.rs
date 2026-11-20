//! Fin Vision standalone gRPC service

mod proto {
    include!(concat!(env!("OUT_DIR"), "/newax/fin_vision/v1/mod.rs"));
}

use fin_vision::FinVisionService;
use proto::fin_vision_service_server::{FinVisionServiceServer, FinVisionService as Trait};
use proto::{
    Empty, JsonResponse, UploadImageRequest, AnalyzeImageRequest,
    ListImagesRequest, IdRequest, GetSettingsRequest, UpdateSettingsRequest, JsonBody,
};
use serde_json::json;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
struct FinVisionServer {
    service: FinVisionService,
}

impl FinVisionServer {
    fn new(pool: PgPool) -> Self {
        Self {
            service: FinVisionService::new(&pool),
        }
    }

    fn ok_json(value: serde_json::Value) -> Result<Response<JsonResponse>, Status> {
        Ok(Response::new(JsonResponse {
            json: serde_json::to_string(&value).unwrap_or_default(),
        }))
    }

    fn err(msg: impl Into<String>) -> Status {
        Status::internal(msg)
    }
}

#[tonic::async_trait]
impl Trait for FinVisionServer {
    async fn get_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        match self.service.get_vision_summary().await {
            Ok(summary) => Self::ok_json(serde_json::to_value(summary).unwrap_or(json!({}))),
            Err(e) => Err(Self::err(e)),
        }
    }

    async fn upload_image(
        &self,
        request: Request<UploadImageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let conversation_id = if msg.conversation_id.is_empty() { None } else { Some(msg.conversation_id.as_str()) };
        let contact_id = if msg.contact_id.is_empty() { None } else { Some(msg.contact_id.as_str()) };
        let channel = if msg.channel.is_empty() { None } else { Some(msg.channel.as_str()) };

        match self.service.upload_image(
            &msg.organization_id,
            conversation_id,
            contact_id,
            channel,
            &msg.image_data,
            &msg.mime_type,
            msg.file_size,
        ).await {
            Ok(result) => Self::ok_json(serde_json::to_value(result).unwrap_or(json!({}))),
            Err(e) => Err(Self::err(e)),
        }
    }

    async fn analyze_image(
        &self,
        request: Request<AnalyzeImageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let prompt = if msg.prompt.is_empty() { None } else { Some(msg.prompt) };
        let claude_model = if msg.claude_model.is_empty() { None } else { Some(msg.claude_model) };

        match self.service.analyze_image(&msg.image_id, &msg.analysis_type, prompt, claude_model).await {
            Ok(result) => Self::ok_json(serde_json::to_value(result).unwrap_or(json!({}))),
            Err(e) => Err(Self::err(e)),
        }
    }

    async fn list_images(
        &self,
        request: Request<ListImagesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let status_filter = if msg.status.is_empty() { None } else { Some(msg.status.as_str()) };

        match self.service.list_images(&msg.organization_id, status_filter, msg.limit).await {
            Ok(images) => Self::ok_json(serde_json::to_value(images).unwrap_or(json!({}))),
            Err(e) => Err(Self::err(e)),
        }
    }

    async fn get_image(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        match self.service.get_image_with_analysis(&msg.id).await {
            Ok((image, analyses)) => Self::ok_json(json!({
                "image": image,
                "analyses": analyses,
            })),
            Err(e) => Err(Self::err(e)),
        }
    }

    async fn delete_image(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        match self.service.delete_image(&msg.id).await {
            Ok(()) => Self::ok_json(json!({"success": true, "message": "Image deleted"})),
            Err(e) => Err(Self::err(e)),
        }
    }

    async fn get_settings(
        &self,
        request: Request<GetSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        match self.service.get_vision_settings(&msg.organization_id).await {
            Some(settings) => Self::ok_json(serde_json::to_value(settings).unwrap_or(json!({}))),
            None => Err(Self::err("Settings not found")),
        }
    }

    async fn update_settings(
        &self,
        request: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let msg = request.into_inner();
        let body = msg.body.unwrap_or(JsonBody { json: String::new() });
        let payload: serde_json::Value = serde_json::from_str(&body.json).unwrap_or(json!({}));

        match self.service.update_vision_settings(&msg.organization_id, &payload).await {
            Ok(settings) => Self::ok_json(serde_json::to_value(settings).unwrap_or(json!({}))),
            Err(e) => Err(Self::err(e)),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_fin_vision=info".parse()?)
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let addr = "127.0.0.1:50105".parse()?;
    let server = FinVisionServer::new(pool);

    tracing::info!("Fin Vision service listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(FinVisionServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}