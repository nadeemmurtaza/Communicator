//! Automation routes — thin gRPC client forwarding to svc-automation.

use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use newax_proto::automation::v1::automation_service_client::AutomationServiceClient;
use newax_proto::automation::v1::{EmptyRequest, IdRequest, QueryRequest, CreateRequest, UpdateRequest};

async fn client(
    state: &AppState,
) -> Result<AutomationServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.automation_grpc_url.is_empty() {
        "http://127.0.0.1:50114".to_string()
    } else {
        state.config.automation_grpc_url.clone()
    };
    AutomationServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("automation gRPC connect failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "error": "automation service unavailable" })))
        })
}

pub async fn list_automation_rules(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_automation_rules(QueryRequest { json: String::new() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_automation_rule(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.create_automation_rule(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_automation_rule(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.update_automation_rule(UpdateRequest { id, json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_automation_rule(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.delete_automation_rule(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn test_automation_rule(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.test_automation_rule(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn apply_automation_rules(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.apply_automation_rules(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_automation_stats(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.get_automation_stats(QueryRequest { json: String::new() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn run_inactivity_nudges(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.run_inactivity_nudges(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
