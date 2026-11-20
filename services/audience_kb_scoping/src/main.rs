use newax_audience_kb_scoping as audience_kb_scoping;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.audience_kb_scoping.v1");
}

use proto::audience_kb_scoping_service_server::{
    AudienceKbScopingService as AudienceKbScopingServiceTrait,
    AudienceKbScopingServiceServer,
};
use proto::{
    AddMembershipRequest, Empty, IdRequest, JsonResponse, LimitRequest, LimitedIdRequest,
    LinkKbRequest, LogUsageRequest, PairRequest, UpdateSettingsRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

#[tonic::async_trait]
impl AudienceKbScopingServiceTrait for Svc {
    async fn get_audience_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let summary = svc.get_audience_summary().await.map_err(to_status)?;
        Ok(Response::new(serialize_response(summary)?))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let settings = svc.get_audience_kb_settings().await;
        match settings {
            Some(s) => Ok(Response::new(serialize_response(s)?)),
            None => {
                let default_settings = json!({
                    "enabled": true,
                    "fallback_to_global_kb": true,
                    "merge_segments": true,
                    "default_scope": "all"
                });
                Ok(Response::new(serialize_response(default_settings)?))
            }
        }
    }

    async fn update_settings(
        &self,
        req: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let settings = svc
            .update_audience_kb_settings(
                r.enabled,
                r.fallback_to_global_kb,
                r.merge_segments,
                r.default_scope.as_deref(),
            )
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(settings)?))
    }

    async fn get_segment_kb_links(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let links = svc.get_segment_kb_links(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(json!({
            "links": links,
            "segment_id": r.id,
        }))?))
    }

    async fn get_all_links(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let links = svc.get_all_links().await.map_err(to_status)?;
        Ok(Response::new(serialize_response(json!({
            "links": links,
        }))?))
    }

    async fn link_kb_to_segment(
        &self,
        req: Request<LinkKbRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let result = svc
            .link_kb_to_segment(&r.segment_id, &r.kb_entry_id, r.priority)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn unlink_kb_from_segment(&self, req: Request<PairRequest>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        svc.unlink_kb_from_segment(&r.first, &r.second)
            .await
            .map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_scoped_kb_for_contact(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let result = svc.get_scoped_kb_for_contact(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn get_segment_memberships(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let memberships = svc.get_segment_memberships(&r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(json!({
            "memberships": memberships,
            "segment_id": r.id,
        }))?))
    }

    async fn get_contact_segment_memberships(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let memberships = svc
            .get_contact_segment_memberships(&r.id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(json!({
            "memberships": memberships,
            "contact_id": r.id,
        }))?))
    }

    async fn get_all_memberships(
        &self,
        req: Request<LimitRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let limit = if r.limit == 0 { 50 } else { r.limit };
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let memberships = svc.get_all_memberships(limit).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(json!({
            "memberships": memberships,
        }))?))
    }

    async fn add_contact_to_segment(
        &self,
        req: Request<AddMembershipRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let membership_type = if r.membership_type.is_empty() {
            "static".to_string()
        } else {
            r.membership_type
        };
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let result = svc
            .add_contact_to_segment(&r.segment_id, &r.contact_id, &membership_type)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn remove_contact_from_segment(
        &self,
        req: Request<PairRequest>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        svc.remove_contact_from_segment(&r.first, &r.second)
            .await
            .map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn log_kb_usage(&self, req: Request<LogUsageRequest>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        svc.log_kb_usage(
            &r.conversation_id,
            &r.contact_id,
            &r.segment_id,
            &r.kb_entry_id,
            &r.query_text,
            r.relevance_score,
            r.used_in_response,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_segment_usage_logs(
        &self,
        req: Request<LimitedIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let limit = if r.limit == 0 { 50 } else { r.limit };
        let svc = audience_kb_scoping::AudienceKbScopingService::new(self.pool.clone());
        let logs = svc
            .get_segment_usage_logs(&r.id, limit)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(json!({
            "logs": logs,
            "segment_id": r.id,
        }))?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "svc-audience-kb-scoping=debug,tonic=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let addr = "127.0.0.1:50098".parse().expect("Invalid address");
    let svc = Svc { pool };

    info!("Audience KB Scoping service listening on {}", addr);

    Server::builder()
        .add_service(AudienceKbScopingServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
