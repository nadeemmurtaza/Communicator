//! Omni-Channel Broadcast gRPC service — standalone binary.

mod proto {
    include!(concat!(env!("OUT_DIR"), "/newax.omni_broadcast.v1.rs"));
}

use proto::omni_broadcast_service_server::{OmniBroadcastService, OmniBroadcastServiceServer};
use proto::*;
use newax_omni_broadcast::*;
use serde_json::json;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct OmniBroadcastServer {
    state: newax_omni_broadcast::OmniBroadcastState,
}

impl OmniBroadcastServer {
    pub fn new(pool: PgPool) -> Self {
        Self {
            state: newax_omni_broadcast::OmniBroadcastState { db: pool },
        }
    }
}

#[tonic::async_trait]
impl OmniBroadcastService for OmniBroadcastServer {
    async fn get_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        match get_broadcast_summary(&self.state).await {
            Ok(summary) => {
                let val = json!({
                    "total_broadcasts": summary.total_broadcasts,
                    "draft_broadcasts": summary.draft_broadcasts,
                    "sent_broadcasts": summary.sent_broadcasts,
                    "failed_broadcasts": summary.failed_broadcasts,
                    "scheduled_broadcasts": summary.scheduled_broadcasts,
                    "total_recipients_sent": summary.total_recipients_sent,
                    "total_delivered": summary.total_delivered,
                    "total_failed": summary.total_failed,
                    "total_read": summary.total_read,
                    "total_clicked": summary.total_clicked,
                    "total_replied": summary.total_replied,
                    "by_channel": summary.by_channel,
                    "recent_broadcasts": summary.recent_broadcasts,
                });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_broadcasts(
        &self,
        request: Request<ListBroadcastsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status: Option<&str> = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        match Broadcast::list(&self.state.db, status, req.limit).await {
            Ok(broadcasts) => {
                let val = json!({ "broadcasts": broadcasts });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_broadcast(
        &self,
        request: Request<GetBroadcastRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match Broadcast::get(&self.state.db, &req.id).await {
            Ok(broadcast) => {
                let val = json!(broadcast);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_broadcast(
        &self,
        request: Request<CreateBroadcastRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        if req.title.is_empty() {
            return Err(Status::invalid_argument("Title is required"));
        }
        if req.body_text.is_empty() {
            return Err(Status::invalid_argument("Body text is required"));
        }
        let brand_id: Option<&str> = if req.brand_id.is_empty() { None } else { Some(req.brand_id.as_str()) };
        let body_html: Option<&str> = if req.body_html.is_empty() { None } else { Some(req.body_html.as_str()) };
        let subject_line: Option<&str> = if req.subject_line.is_empty() { None } else { Some(req.subject_line.as_str()) };
        let preview_text: Option<&str> = if req.preview_text.is_empty() { None } else { Some(req.preview_text.as_str()) };
        let primary_media_url: Option<&str> = if req.primary_media_url.is_empty() { None } else { Some(req.primary_media_url.as_str()) };
        let media_caption: Option<&str> = if req.media_caption.is_empty() { None } else { Some(req.media_caption.as_str()) };
        let link_url: Option<&str> = if req.link_url.is_empty() { None } else { Some(req.link_url.as_str()) };
        let link_title: Option<&str> = if req.link_title.is_empty() { None } else { Some(req.link_title.as_str()) };
        let link_description: Option<&str> = if req.link_description.is_empty() { None } else { Some(req.link_description.as_str()) };
        let link_image_url: Option<&str> = if req.link_image_url.is_empty() { None } else { Some(req.link_image_url.as_str()) };
        let scheduled_at: Option<&str> = if req.scheduled_at.is_empty() { None } else { Some(req.scheduled_at.as_str()) };

        match Broadcast::create(
            &self.state.db, &req.organization_id, brand_id, &req.title, &req.created_by,
            &req.body_text, body_html, subject_line, preview_text,
            &req.media_type, primary_media_url, media_caption,
            link_url, link_title, link_description,
            link_image_url, &req.target_type, &req.target_ids, &req.channels,
            scheduled_at,
        ).await {
            Ok(broadcast) => {
                let val = json!(broadcast);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn update_broadcast(
        &self,
        request: Request<UpdateBroadcastRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let updates: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&req.updates_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match Broadcast::update(&self.state.db, &req.id, updates).await {
            Ok(broadcast) => {
                let val = json!(broadcast);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn delete_broadcast(
        &self,
        request: Request<DeleteBroadcastRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match Broadcast::delete(&self.state.db, &req.id).await {
            Ok(()) => {
                let val = json!({ "success": true });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn update_broadcast_status(
        &self,
        request: Request<UpdateBroadcastStatusRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match Broadcast::update_status(&self.state.db, &req.id, &req.status).await {
            Ok(broadcast) => {
                let val = json!(broadcast);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn send_broadcast(
        &self,
        request: Request<SendBroadcastRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match send_broadcast(&self.state.db, &req.id).await {
            Ok(result) => {
                let val = json!(result);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_format_preview(
        &self,
        request: Request<GetFormatPreviewRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match get_format_preview(&self.state.db, &req.id).await {
            Ok(preview) => {
                let val = json!(preview);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_available_channels(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let channels = get_available_channels();
        let val = json!({ "channels": channels });
        Ok(Response::new(JsonResponse { json: val.to_string() }))
    }

    async fn upload_media(
        &self,
        request: Request<UploadMediaRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        if req.broadcast_id.is_empty() || req.file_url.is_empty() {
            return Err(Status::invalid_argument("broadcast_id and file_url are required"));
        }
        let thumbnail_url: Option<&str> = if req.thumbnail_url.is_empty() { None } else { Some(req.thumbnail_url.as_str()) };
        match BroadcastMedia::upload(
            &self.state.db, &req.broadcast_id, &req.media_type, &req.file_name, req.file_size_bytes,
            &req.mime_type, &req.file_url, thumbnail_url,
        ).await {
            Ok(media) => {
                let val = json!(media);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_media(
        &self,
        request: Request<ListMediaRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match BroadcastMedia::list_by_broadcast(&self.state.db, &req.broadcast_id).await {
            Ok(media) => {
                let val = json!({ "media": media });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn delete_media(
        &self,
        request: Request<DeleteMediaRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match BroadcastMedia::delete(&self.state.db, &req.id).await {
            Ok(()) => {
                let val = json!({ "success": true });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_broadcast_channels(
        &self,
        request: Request<ListBroadcastChannelsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match BroadcastChannel::list_by_broadcast(&self.state.db, &req.broadcast_id).await {
            Ok(channels) => {
                let val = json!({ "channels": channels });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_broadcast_recipients(
        &self,
        request: Request<ListBroadcastRecipientsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status: Option<&str> = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        match BroadcastRecipient::list_by_broadcast(&self.state.db, &req.broadcast_id, status, req.limit).await {
            Ok(recipients) => {
                let val = json!({ "recipients": recipients });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_broadcast_logs(
        &self,
        request: Request<ListBroadcastLogsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match BroadcastLog::list_by_broadcast(&self.state.db, &req.broadcast_id, req.limit).await {
            Ok(logs) => {
                let val = json!({ "logs": logs });
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_settings(
        &self,
        request: Request<GetSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let org_id = if req.organization_id.is_empty() { "default" } else { &req.organization_id };
        match BroadcastSettings::get(&self.state.db, org_id).await {
            Ok(settings) => {
                let val = json!(settings);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn update_settings(
        &self,
        request: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let org_id = if req.organization_id.is_empty() { "default" } else { &req.organization_id };
        let updates: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&req.updates_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match BroadcastSettings::update(&self.state.db, org_id, updates).await {
            Ok(settings) => {
                let val = json!(settings);
                Ok(Response::new(JsonResponse { json: val.to_string() }))
            }
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn preview_html_for_channels(
        &self,
        request: Request<PreviewHtmlForChannelsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        if req.html.is_empty() && req.body_text.is_empty() {
            return Err(Status::invalid_argument("Provide html or bodyText"));
        }
        let formats = preview_html_for_all_channels(&req.html, &req.body_text, &req.channels);
        let val = json!({ "formats": formats });
        Ok(Response::new(JsonResponse { json: val.to_string() }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_omni_broadcast=info".parse()?)
        )
        .init();

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await
        .expect("Failed to connect to database");

    let addr = "0.0.0.0:50096".parse()
        .expect("Invalid address");

    let server = OmniBroadcastServer::new(pool);

    tracing::info!("Starting omni_broadcast gRPC service on {}", addr);

    tonic::transport::Server::builder()
        .add_service(OmniBroadcastServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
