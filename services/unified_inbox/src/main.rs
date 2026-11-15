use newax_unified_inbox as unified_inbox;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.unified_inbox.v1");
}

use proto::{
    unified_inbox_service_server::{UnifiedInboxService, UnifiedInboxServiceServer},
    IdRequest, JsonResponse, ListInboxRequest, ResolveCustomerRequest,
    CreateMappingRequest, GetSettingsRequest, UpdateSettingsRequest,
};

struct Svc {
    state: unified_inbox::UnifiedInboxState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl UnifiedInboxService for Svc {
    async fn list_inbox(
        &self,
        req: Request<ListInboxRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = unified_inbox::get_unified_inbox_summary(
            &self.state,
            r.limit,
            r.offset,
            r.channel.as_deref(),
            r.state.as_deref(),
            r.search.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_customer(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = unified_inbox::get_unified_customer(&self.state, &r.id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn resolve_customer(
        &self,
        req: Request<ResolveCustomerRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = unified_inbox::resolve_customer(&self.state, &r.channel, &r.identifier)
            .await
            .map_err(to_status)?;
        let val = match v {
            Some(customer) => serde_json::to_value(customer).map_err(|e| Status::internal(e.to_string()))?,
            None => serde_json::json!({ "found": false, "message": "No customer found for this channel + identifier" }),
        };
        Ok(Response::new(JsonResponse { json: val.to_string() }))
    }

    async fn create_mapping(
        &self,
        req: Request<CreateMappingRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = unified_inbox::create_customer_mapping(
            &self.state,
            &r.customer_id,
            &r.contact_id,
            &r.channel,
            &r.channel_identifier,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_settings(
        &self,
        req: Request<GetSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = unified_inbox::get_unified_inbox_settings(&self.state, &r.organization_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn update_settings(
        &self,
        req: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload = parse_json(&r.payload_json)?;
        let v = unified_inbox::update_unified_inbox_settings(&self.state, &r.organization_id, &payload)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("info,svc_unified_inbox=debug").unwrap(),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("UNIFIED_INBOX_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50081);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = Svc {
        state: unified_inbox::UnifiedInboxState { db },
    };

    info!(port, "svc-unified-inbox starting (gRPC)");
    Server::builder()
        .add_service(UnifiedInboxServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
