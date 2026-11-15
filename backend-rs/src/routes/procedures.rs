//! AI Procedures & Actions routes (Phase 18).
//!
//! Thin gRPC client forwarding to svc-procedures; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use newax_proto::procedures::v1::{
    procedures_service_client::ProceduresServiceClient,
    Empty, IdRequest, JsonResponse, ListActionsRequest, ListProceduresRequest,
    MatchRequest, StartExecutionRequest, ExecuteStepRequest, CompleteRequest,
    FailRequest, UpdateRequest, TestActionRequest,
};
use serde::Deserialize;
use serde_json::json;
use tonic::transport::Channel;

use crate::db::AppState;

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn grpc_err(status: tonic::Status) -> Response {
    let code = match status.code() {
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    err(code, status.message().to_string())
}

async fn client() -> Result<ProceduresServiceClient<Channel>, Response> {
    let url = std::env::var("PROCEDURES_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50080".to_string());
    ProceduresServiceClient::connect(url)
        .await
        .map_err(|e| err(StatusCode::SERVICE_UNAVAILABLE, format!("procedures service unreachable: {e}")))
}

fn json_value(json_str: String) -> serde_json::Value {
    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
}

#[derive(Deserialize)]
pub struct ListProceduresQuery {
    status: Option<String>,
    category: Option<String>,
}

#[derive(Deserialize)]
pub struct ListActionsQuery {
    action_type: Option<String>,
}

// ——— Handlers ———

/// GET /procedures/summary — Get procedure and actions summary
pub async fn get_summary(State(_state): State<AppState>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.get_summary(Empty {}).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /procedures — List procedures with optional filters
pub async fn list_procedures(
    State(_state): State<AppState>,
    Query(params): Query<ListProceduresQuery>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ListProceduresRequest {
        status: params.status,
        category: params.category,
    };
    match c.list_procedures(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /procedures — Create a new procedure
pub async fn create_procedure(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = JsonResponse { json: body.to_string() };
    match c.create_procedure(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /procedures/{id} — Get a procedure
pub async fn get_procedure(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id };
    match c.get_procedure(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// PUT /procedures/{id} — Update a procedure
pub async fn update_procedure(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = UpdateRequest {
        id,
        updates_json: body.to_string(),
    };
    match c.update_procedure(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// DELETE /procedures/{id} — Delete a procedure
pub async fn delete_procedure(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id };
    match c.delete_procedure(req).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /procedures/match — Match a message against procedures
pub async fn match_procedures(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let message = match body.get("message").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "message is required"),
    };
    let conversation_id = match body.get("conversation_id").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "conversation_id is required"),
    };
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = MatchRequest { message, conversation_id };
    match c.match_procedures(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /procedures/{id}/execute — Start a procedure execution
pub async fn start_execution(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let conversation_id = match body.get("conversation_id").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return err(StatusCode::BAD_REQUEST, "conversation_id is required"),
    };
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = StartExecutionRequest {
        procedure_id: id,
        conversation_id,
    };
    match c.start_execution(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /procedures/executions/{id} — Get execution details
pub async fn get_execution(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id };
    match c.get_execution(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /procedures/executions/conversation/{id} — Get running execution for conversation
pub async fn get_running_execution(
    State(_state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id: conversation_id };
    match c.get_running_execution(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /procedures/executions/{id}/step — Execute current step
pub async fn execute_step(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let customer_input = body.get("customer_input").and_then(|v| v.as_str()).map(|s| s.to_string());
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ExecuteStepRequest {
        execution_id: id,
        customer_input,
    };
    match c.execute_step(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /procedures/executions/{id}/complete — Complete an execution
pub async fn complete_execution(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let result = body.get("result").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = CompleteRequest {
        execution_id: id,
        result,
    };
    match c.complete_execution(req).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /procedures/executions/{id}/fail — Fail an execution
pub async fn fail_execution(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let error = body.get("error").and_then(|v| v.as_str()).unwrap_or("failed").to_string();
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = FailRequest {
        execution_id: id,
        error,
    };
    match c.fail_execution(req).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => grpc_err(e),
    }
}

// ——— Actions ———

/// GET /actions — List actions
pub async fn list_actions(
    State(_state): State<AppState>,
    Query(params): Query<ListActionsQuery>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ListActionsRequest {
        action_type: params.action_type,
    };
    match c.list_actions(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /actions — Create an action
pub async fn create_action(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = JsonResponse { json: body.to_string() };
    match c.create_action(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /actions/{id} — Get an action
pub async fn get_action(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id };
    match c.get_action(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// PUT /actions/{id} — Update an action
pub async fn update_action(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = UpdateRequest {
        id,
        updates_json: body.to_string(),
    };
    match c.update_action(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// DELETE /actions/{id} — Delete an action
pub async fn delete_action(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = IdRequest { id };
    match c.delete_action(req).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /actions/{id}/test — Test an action
pub async fn test_action(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = TestActionRequest {
        action_id: id,
        params_json: body.to_string(),
    };
    match c.test_action(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}
