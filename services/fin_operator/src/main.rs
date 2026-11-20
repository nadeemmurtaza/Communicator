mod proto {
    include!(concat!(env!("OUT_DIR"), "/newax/fin_operator/v1/mod.rs"));
}

use fin_operator::FinOperatorService;
use proto::fin_operator_service_server::{FinOperatorServiceServer, FinOperatorService as Trait};
use proto::{
    Empty, JsonResponse, UpdateConfigRequest, ListProposalsRequest, IdRequest,
    ApproveProposalRequest, DismissProposalRequest, ListLogsRequest, ListSnapshotsRequest, JsonBody,
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
struct FinOperatorServer {
    service: FinOperatorService,
}

impl FinOperatorServer {
    fn new(pool: PgPool) -> Self {
        Self {
            service: FinOperatorService::new(&pool),
        }
    }
}

#[tonic::async_trait]
impl Trait for FinOperatorServer {
    async fn get_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let result = self.service.get_summary().await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_config(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let result = self.service.get_config().await
            .map_err(|e| Status::internal(e.to_string()))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let body = req.body.ok_or_else(|| Status::invalid_argument("Missing body"))?;
        let updates: serde_json::Value = serde_json::from_str(&body.json)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON: {}", e)))?;
        let result = self.service.update_config(&updates).await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn run_scan(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let result = self.service.run_scan().await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_proposals(
        &self,
        request: Request<ListProposalsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let proposal_type = if req.proposal_type.is_empty() { None } else { Some(req.proposal_type.as_str()) };
        let limit = req.limit;
        let result = self.service.list_proposals(status, proposal_type, limit).await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_proposal(
        &self,
        request: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.service.get_proposal(&req.id).await
            .map_err(|e| Status::not_found(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn approve_proposal(
        &self,
        request: Request<ApproveProposalRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let result = self.service.approve_proposal(&req.id, &req.reviewer).await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn dismiss_proposal(
        &self,
        request: Request<DismissProposalRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        self.service.dismiss_proposal(&req.id, &req.reviewer).await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::json!({"message": "Proposal dismissed", "id": req.id}).to_string();
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_logs(
        &self,
        request: Request<ListLogsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let action_type = if req.action_type.is_empty() { None } else { Some(req.action_type.as_str()) };
        let limit = req.limit;
        let result = self.service.list_logs(action_type, limit).await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_snapshots(
        &self,
        request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let limit = req.limit;
        let result = self.service.list_snapshots(limit).await
            .map_err(|e| Status::internal(e))?;
        let json = serde_json::to_string(&result).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JsonResponse { json }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("svc_fin_operator=info".parse()?),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/communicator".to_string());

    let pool = PgPool::connect(&database_url).await
        .expect("Failed to connect to database");

    let addr = "[::1]:50104".parse().expect("Invalid address");
    let server = FinOperatorServer::new(pool);

    tracing::info!("Fin Operator service listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(FinOperatorServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
