use newax_system_health::SystemHealthService as Logic;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.system_health.v1");
}

use proto::{
    system_health_service_server::{SystemHealthService as SystemHealthRpc, SystemHealthServiceServer},
    CompleteStepRequest, Empty, JsonResponse, OrgRequest, UpdateSettingsRequest,
};

struct SystemHealthGrpc {
    logic: Logic,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn json_response<T: serde::Serialize>(v: &T) -> Result<JsonResponse, Status> {
    Ok(JsonResponse {
        json: serde_json::to_string(v).map_err(to_status)?,
    })
}

#[tonic::async_trait]
impl SystemHealthRpc for SystemHealthGrpc {
    async fn get_summary(&self, req: Request<OrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = self.logic.get_summary(&req.into_inner().organization_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn get_steps(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let r = self.logic.get_steps().await.map_err(to_status)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn get_progress(&self, req: Request<OrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = self.logic.get_progress(&req.into_inner().organization_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn complete_step(
        &self,
        req: Request<CompleteStepRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        if r.step_id.is_empty() {
            return Err(Status::invalid_argument("step_id is required"));
        }
        let result = self
            .logic
            .complete_step(&r.organization_id, &r.step_id, r.completed_by.as_deref())
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(&result)?))
    }

    async fn run_health_check(&self, req: Request<OrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = self
            .logic
            .run_health_check(&req.into_inner().organization_id)
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn get_health_summary(&self, req: Request<OrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = self
            .logic
            .get_health_summary(&req.into_inner().organization_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn get_settings(&self, req: Request<OrgRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = self.logic.get_settings(&req.into_inner().organization_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(&r)?))
    }

    async fn update_settings(
        &self,
        req: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = self
            .logic
            .update_settings(
                &r.organization_id,
                r.show_onboarding,
                r.auto_check,
                r.health_check_interval_minutes,
            )
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(&result)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_system_health=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("SYSTEM_HEALTH_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50062);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = SystemHealthGrpc { logic: Logic::new(&pool) };

    info!(port, "svc-system-health starting (gRPC)");
    Server::builder()
        .add_service(SystemHealthServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
