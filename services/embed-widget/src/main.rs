use newax_embed_widget::EmbedWidgetService as Logic;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.embed_widget.v1");
}

use proto::{
    embed_widget_service_server::{EmbedWidgetService as EmbedWidgetRpc, EmbedWidgetServiceServer},
    CreateConfigRequest, Empty, GetEventsRequest, IdRequest, JsonResponse, ListConfigsRequest, OrgRequest,
    RecordEventRequest, UpdateConfigRequest, UpdateSettingsRequest,
};

struct EmbedWidgetGrpc {
    logic: Logic,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn json_response<T: serde::Serialize>(v: &T) -> Result<JsonResponse, Status> {
    Ok(JsonResponse { json: serde_json::to_string(v).map_err(to_status)? })
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl EmbedWidgetRpc for EmbedWidgetGrpc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let r = self.logic.get_summary().await.map_err(to_status)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn list_configs(&self, req: Request<ListConfigsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let configs = self.logic.list_configs(&r.org_id, r.status.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(&configs)?))
    }

    async fn get_config(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let config = self.logic.get_config(&req.into_inner().id).await.map_err(to_status)?;
        match config {
            Some(c) => Ok(Response::new(json_response(&c)?)),
            None => Err(Status::not_found("Config not found")),
        }
    }

    async fn create_config(&self, req: Request<CreateConfigRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let settings = parse_json(&r.settings_json)?;
        let styles = parse_json(&r.styles_json)?;
        let trigger_config = parse_json(&r.trigger_config_json)?;
        let result = self
            .logic
            .create_config(&r.org_id, &r.name, &r.widget_type, &settings, &styles, &trigger_config)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(&result)?))
    }

    async fn update_config(&self, req: Request<UpdateConfigRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let updates = parse_json(&r.updates_json)?;
        let config = self.logic.update_config(&r.id, &updates).await.map_err(to_status)?;
        Ok(Response::new(json_response(&config)?))
    }

    async fn delete_config(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        self.logic.delete_config(&req.into_inner().id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_settings(&self, req: Request<OrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let settings = self.logic.get_settings(&req.into_inner().org_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(&settings)?))
    }

    async fn update_settings(&self, req: Request<UpdateSettingsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let updates = parse_json(&r.updates_json)?;
        let settings = self.logic.update_settings(&r.org_id, &updates).await.map_err(to_status)?;
        Ok(Response::new(json_response(&settings)?))
    }

    async fn record_event(&self, req: Request<RecordEventRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = parse_json(&r.payload_json)?;
        let event_id = self.logic.record_event(&r.instance_id, &r.event_type, &payload).await.map_err(to_status)?;
        Ok(Response::new(json_response(&serde_json::json!({ "event_id": event_id }))?))
    }

    async fn get_events(&self, req: Request<GetEventsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let events = self
            .logic
            .get_events(r.instance_id.as_deref(), r.event_type.as_deref(), r.limit)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(&events)?))
    }

    async fn get_public_widget_data(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let data = self.logic.get_public_widget_data(&req.into_inner().id).await.map_err(to_status)?;
        Ok(Response::new(json_response(&data)?))
    }

    async fn get_embed_script(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = req.into_inner().id;
        let script = self.logic.generate_embed_script(&id);
        Ok(Response::new(json_response(&serde_json::json!({ "script": script, "config_id": id }))?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_embed_widget=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("EMBED_WIDGET_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50063);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = EmbedWidgetGrpc { logic: Logic::new(&pool) };

    info!(port, "svc-embed-widget starting (gRPC)");
    Server::builder()
        .add_service(EmbedWidgetServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
