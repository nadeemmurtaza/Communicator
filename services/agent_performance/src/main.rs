use newax_agent_performance as agent_performance;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.agent_performance.v1");
}

use proto::{
    agent_performance_service_server::{AgentPerformanceService, AgentPerformanceServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: agent_performance::AgentPerformanceState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl AgentPerformanceService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = agent_performance::get_performance_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_snapshots(
        &self,
        req: Request<proto::ListSnapshotsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let agent_id = if r.agent_id.is_empty() { None } else { Some(r.agent_id) };
        let v = agent_performance::list_snapshots(&self.state, agent_id.as_deref(), r.days).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn create_snapshot(
        &self,
        req: Request<proto::CreateSnapshotRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let data = parse_json(&r.data)?;
        let v = agent_performance::create_snapshot(&self.state, &r.agent_id, &r.agent_name, &data).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_insights(
        &self,
        req: Request<proto::ListInsightsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let agent_id = if r.agent_id.is_empty() { None } else { Some(r.agent_id) };
        let addressed = if r.has_addressed { Some(r.addressed) } else { None };
        let v = agent_performance::list_insights(&self.state, agent_id.as_deref(), addressed).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn create_insight(
        &self,
        req: Request<proto::CreateInsightRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let data = parse_json(&r.data)?;
        let v = agent_performance::create_insight(&self.state, &r.agent_id, &r.agent_name, &data).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn address_insight(
        &self,
        req: Request<proto::AddressInsightRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = agent_performance::address_insight(&self.state, &r.insight_id, &r.addressed_by).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_comparison(
        &self,
        req: Request<proto::GetComparisonRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = agent_performance::get_comparison_metrics(&self.state, r.days).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn generate_comparison(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = agent_performance::generate_comparison_snapshot(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = agent_performance::get_settings(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn update_settings(
        &self,
        req: Request<proto::UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let data = parse_json(&r.data)?;
        let v = agent_performance::update_settings(&self.state, &data).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Cannot connect to database: {e}"))?;

    let state = agent_performance::AgentPerformanceState::new(pool);

    let addr = "[::1]:50087".parse().unwrap();
    info!("agent-performance listening on {}", addr);

    Server::builder()
        .add_service(AgentPerformanceServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
