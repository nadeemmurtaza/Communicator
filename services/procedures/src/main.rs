use newax_procedures as procedures;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.procedures.v1");
}

use proto::{
    procedures_service_server::{ProceduresService, ProceduresServiceServer},
    Empty, IdRequest, JsonResponse, ListActionsRequest, ListProceduresRequest,
    MatchRequest, StartExecutionRequest, ExecuteStepRequest, CompleteRequest,
    FailRequest, UpdateRequest, TestActionRequest,
};

struct Svc {
    state: procedures::ProceduresState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(json_response(v))
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl ProceduresService for Svc {
    async fn get_summary(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = procedures::get_procedure_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn list_procedures(
        &self,
        req: Request<ListProceduresRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::list_procedures(
            &self.state,
            r.status.as_deref(),
            r.category.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn create_procedure(
        &self,
        req: Request<JsonResponse>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body = parse_json(&r.json)?;
        let name = body.get("name").and_then(|v| v.as_str()).ok_or_else(|| Status::invalid_argument("name is required"))?;
        let description = body.get("description").and_then(|v| v.as_str());
        let category = body.get("category").and_then(|v| v.as_str()).unwrap_or("general");
        let trigger_keywords = body.get("trigger_keywords").cloned().unwrap_or_else(|| serde_json::json!([]));
        let steps = body.get("steps").cloned().unwrap_or_else(|| serde_json::json!([]));
        let max_iterations = body.get("max_iterations").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
        let timeout_seconds = body.get("timeout_seconds").and_then(|v| v.as_i64()).unwrap_or(300) as i32;
        let brand_id = body.get("brand_id").and_then(|v| v.as_str());
        let created_by = body.get("created_by").and_then(|v| v.as_str());
        let v = procedures::create_procedure(
            &self.state, name, description, category, &trigger_keywords, &steps,
            max_iterations, timeout_seconds, brand_id, created_by,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_procedure(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::get_procedure(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn update_procedure(
        &self,
        req: Request<UpdateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let updates = parse_json(&r.updates_json)?;
        let v = procedures::update_procedure(&self.state, &r.id, &updates).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn delete_procedure(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        procedures::delete_procedure(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn match_procedures(
        &self,
        req: Request<MatchRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::match_procedures(&self.state, &r.message, &r.conversation_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn start_execution(
        &self,
        req: Request<StartExecutionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::start_procedure_execution(&self.state, &r.procedure_id, &r.conversation_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_execution(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::get_procedure_execution(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_running_execution(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::get_running_execution(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn execute_step(
        &self,
        req: Request<ExecuteStepRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::execute_current_step(&self.state, &r.execution_id, r.customer_input.as_deref())
            .await
            .map_err(|(code, msg)| to_status(format!("{}: {}", code, msg)))?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn complete_execution(
        &self,
        req: Request<CompleteRequest>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        procedures::complete_procedure_execution(&self.state, &r.execution_id, &r.result)
            .await
            .map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn fail_execution(
        &self,
        req: Request<FailRequest>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        procedures::fail_procedure_execution(&self.state, &r.execution_id, &r.error)
            .await
            .map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn list_actions(
        &self,
        req: Request<ListActionsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::list_actions(&self.state, r.action_type.as_deref())
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn create_action(
        &self,
        req: Request<JsonResponse>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body = parse_json(&r.json)?;
        let name = body.get("name").and_then(|v| v.as_str()).ok_or_else(|| Status::invalid_argument("name is required"))?;
        let description = body.get("description").and_then(|v| v.as_str());
        let action_type = body.get("action_type").and_then(|v| v.as_str()).ok_or_else(|| Status::invalid_argument("action_type is required"))?;
        let endpoint = body.get("endpoint").and_then(|v| v.as_str());
        let method = body.get("method").and_then(|v| v.as_str()).unwrap_or("POST");
        let headers = body.get("headers").cloned().unwrap_or_else(|| serde_json::json!({}));
        let body_template = body.get("body_template").and_then(|v| v.as_str());
        let auth_config = body.get("auth_config").cloned();
        let timeout_seconds = body.get("timeout_seconds").and_then(|v| v.as_i64()).unwrap_or(30) as i32;
        let retry_count = body.get("retry_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let brand_id = body.get("brand_id").and_then(|v| v.as_str());
        let created_by = body.get("created_by").and_then(|v| v.as_str());
        let v = procedures::create_action(
            &self.state, name, description, action_type, endpoint, method, &headers,
            body_template, auth_config.as_ref(), timeout_seconds, retry_count, brand_id, created_by,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn get_action(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::get_action(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn update_action(
        &self,
        req: Request<UpdateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let updates = parse_json(&r.updates_json)?;
        let v = procedures::update_action(&self.state, &r.id, &updates).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }

    async fn delete_action(
        &self,
        req: Request<IdRequest>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        procedures::delete_action(&self.state, &r.id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn test_action(
        &self,
        req: Request<TestActionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = procedures::get_action(&self.state, &r.action_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(v)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_procedures=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;
    let http_client = reqwest::Client::new();

    let port: u16 = std::env::var("PROCEDURES_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50080);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = Svc {
        state: procedures::ProceduresState { db, http_client },
    };

    info!(port, "svc-procedures starting (gRPC)");
    Server::builder()
        .add_service(ProceduresServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
