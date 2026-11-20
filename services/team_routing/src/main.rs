//! Team Routing standalone gRPC service.

mod gen {
    include!("gen/mod.rs");
}

use newax_team_routing::*;
use gen::team_routing_service_server::{TeamRoutingService, TeamRoutingServiceServer};
use gen::*;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tonic::Request;
use tracing::info;

#[derive(Clone)]
struct TeamRoutingServer {
    state: TeamRoutingState,
}

#[tonic::async_trait]
impl TeamRoutingService for TeamRoutingServer {
    async fn list_team(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match list_team(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn create_team_member(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match create_team_member(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn delete_team_member(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match delete_team_member(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn assign_conversation(
        &self,
        request: Request<AssignRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let member_id = if inner.member_id.is_empty() {
            None
        } else {
            Some(inner.member_id.as_str())
        };
        match assign_conversation(&self.state, &inner.conversation_id, member_id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn set_human_takeover(
        &self,
        request: Request<TakeoverRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match set_human_takeover(&self.state, &inner.conversation_id, inner.enabled).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn list_teams(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match list_teams(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn create_team(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match create_team(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn delete_team(
        &self,
        request: Request<IdRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let id = request.into_inner().id;
        match delete_team(&self.state, &id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn add_member_to_team(
        &self,
        request: Request<AddMemberRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match add_member_to_team(&self.state, &inner.team_id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn remove_member_from_team(
        &self,
        request: Request<RemoveMemberRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        match remove_member_from_team(&self.state, &inner.team_id, &inner.member_id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn list_routing_agents(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        match list_routing_agents(&self.state).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn update_agent_status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match update_agent_status(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn update_agent_capacity(
        &self,
        request: Request<CapacityRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let body: serde_json::Value = serde_json::from_str(&inner.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match update_agent_capacity(&self.state, &inner.id, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn get_capacity_overview(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let inner = request.into_inner();
        let team_id = if inner.team_id.is_empty() {
            None
        } else {
            Some(inner.team_id.as_str())
        };
        match get_capacity_overview(&self.state, team_id).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }

    async fn batch_auto_assign(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let body_str = request.into_inner().json;
        let body: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        match batch_auto_assign(&self.state, &body).await {
            Ok(val) => Ok(tonic::Response::new(JsonResponse { json: val.to_string() })),
            Err(e) => Err(tonic::Status::internal(e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let state = TeamRoutingState { db: pool };
    let server = TeamRoutingServer { state };

    let addr = "[::1]:50116".parse()?;
    info!("team_routing service listening on {}", addr);

    TeamRoutingServiceServer::new(server)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("Shutdown signal received");
}
