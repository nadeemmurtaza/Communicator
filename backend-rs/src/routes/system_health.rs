//! System health / onboarding progress routes.
//!
//! Split out of the monolith the same way as kb_documents -- see
//! services/system-health and crates/system-health at the repo root.
//! REST paths and payload shapes are unchanged.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use newax_proto::system_health::v1::{
    system_health_service_client::SystemHealthServiceClient, CompleteStepRequest, Empty, OrgRequest,
    UpdateSettingsRequest,
};
use tonic::transport::Channel;

use crate::AppState;

fn err_response(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message })))
}

async fn client() -> Result<SystemHealthServiceClient<Channel>, (StatusCode, Json<serde_json::Value>)> {
    let url = std::env::var("SYSTEM_HEALTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50062".to_string());
    SystemHealthServiceClient::connect(url)
        .await
        .map_err(|e| err_response(StatusCode::SERVICE_UNAVAILABLE, &format!("system-health service unreachable: {e}")))
}

fn json_value(json_str: String) -> serde_json::Value {
    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
}

fn grpc_err(status: tonic::Status) -> serde_json::Value {
    serde_json::json!({ "error": status.message() })
}

pub async fn get_summary(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let mut c = match client().await {
        Ok(c) => c,
        Err((_, j)) => return j,
    };
    let org_id = "org-default".to_string();
    match c.get_summary(OrgRequest { organization_id: org_id }).await {
        Ok(r) => Json(json_value(r.into_inner().json)),
        Err(e) => Json(grpc_err(e)),
    }
}

pub async fn get_steps(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let mut c = match client().await {
        Ok(c) => c,
        Err((_, j)) => return j,
    };
    match c.get_steps(Empty {}).await {
        Ok(r) => Json(json_value(r.into_inner().json)),
        Err(e) => Json(grpc_err(e)),
    }
}

pub async fn get_progress(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let mut c = match client().await {
        Ok(c) => c,
        Err((_, j)) => return j,
    };
    let org_id = "org-default".to_string();
    match c.get_progress(OrgRequest { organization_id: org_id }).await {
        Ok(r) => Json(json_value(r.into_inner().json)),
        Err(e) => Json(grpc_err(e)),
    }
}

pub async fn complete_step(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let step_id = payload.get("stepId").and_then(|v| v.as_str()).unwrap_or("");
    let completed_by = payload.get("completedBy").and_then(|v| v.as_str()).map(String::from);

    if step_id.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "stepId is required.").into_response();
    }

    let mut c = match client().await {
        Ok(c) => c,
        Err((status, j)) => return (status, j).into_response(),
    };
    let req = CompleteStepRequest {
        organization_id: "org-default".to_string(),
        step_id: step_id.to_string(),
        completed_by,
    };
    match c.complete_step(req).await {
        Ok(r) => Json(json_value(r.into_inner().json)).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, e.message()).into_response(),
    }
}

pub async fn run_health_check(State(_state): State<AppState>) -> impl axum::response::IntoResponse {
    let mut c = match client().await {
        Ok(c) => c,
        Err((status, j)) => return (status, j).into_response(),
    };
    let org_id = "org-default".to_string();
    match c.run_health_check(OrgRequest { organization_id: org_id }).await {
        Ok(r) => Json(json_value(r.into_inner().json)).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, e.message()).into_response(),
    }
}

pub async fn get_health_summary(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let mut c = match client().await {
        Ok(c) => c,
        Err((_, j)) => return j,
    };
    let org_id = "org-default".to_string();
    match c.get_health_summary(OrgRequest { organization_id: org_id }).await {
        Ok(r) => Json(json_value(r.into_inner().json)),
        Err(e) => Json(grpc_err(e)),
    }
}

pub async fn get_settings(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let mut c = match client().await {
        Ok(c) => c,
        Err((_, j)) => return j,
    };
    let org_id = "org-default".to_string();
    match c.get_settings(OrgRequest { organization_id: org_id }).await {
        Ok(r) => Json(json_value(r.into_inner().json)),
        Err(e) => Json(grpc_err(e)),
    }
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let show_onboarding = payload.get("showOnboarding").and_then(|v| v.as_bool());
    let auto_check = payload.get("autoCheck").and_then(|v| v.as_bool());
    let health_check_interval_minutes = payload
        .get("healthCheckIntervalMinutes")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    let mut c = match client().await {
        Ok(c) => c,
        Err((status, j)) => return (status, j).into_response(),
    };
    let req = UpdateSettingsRequest {
        organization_id: "org-default".to_string(),
        show_onboarding,
        auto_check,
        health_check_interval_minutes,
    };
    match c.update_settings(req).await {
        Ok(r) => Json(json_value(r.into_inner().json)).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, e.message()).into_response(),
    }
}
