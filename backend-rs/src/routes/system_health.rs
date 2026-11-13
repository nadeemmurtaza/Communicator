use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::AppState;
use crate::services::system_health::SystemHealthService;

fn err_response(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message })))
}

pub async fn get_summary(State(state): State<AppState>) -> Json<serde_json::Value> {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    match service.get_summary(org_id).await {
        Ok(summary) => Json(serde_json::to_value(summary).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

pub async fn get_steps(State(state): State<AppState>) -> Json<serde_json::Value> {
    let service = SystemHealthService::new(&state.db);
    match service.get_steps().await {
        Ok(steps) => Json(serde_json::to_value(steps).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

pub async fn get_progress(State(state): State<AppState>) -> Json<serde_json::Value> {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    match service.get_progress(org_id).await {
        Ok(progress) => Json(serde_json::to_value(progress).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

pub async fn complete_step(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    let step_id = payload.get("stepId").and_then(|v| v.as_str()).unwrap_or("");
    let completed_by = payload.get("completedBy").and_then(|v| v.as_str());

    if step_id.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "stepId is required.")
            .into_response();
    }

    match service
        .complete_step(org_id, step_id, completed_by)
        .await
    {
        Ok(result) => Json(serde_json::to_value(result).unwrap_or_default()).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e).into_response(),
    }
}

pub async fn run_health_check(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    match service.run_health_check(org_id).await {
        Ok(result) => Json(serde_json::to_value(result).unwrap_or_default()).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e).into_response(),
    }
}

pub async fn get_health_summary(State(state): State<AppState>) -> Json<serde_json::Value> {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    match service.get_health_summary(org_id).await {
        Ok(summary) => Json(serde_json::to_value(summary).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

pub async fn get_settings(State(state): State<AppState>) -> Json<serde_json::Value> {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    match service.get_settings(org_id).await {
        Ok(settings) => Json(serde_json::to_value(settings).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let service = SystemHealthService::new(&state.db);
    let org_id = "org-default";
    let show_onboarding = payload.get("showOnboarding").and_then(|v| v.as_bool());
    let auto_check = payload.get("autoCheck").and_then(|v| v.as_bool());
    let health_check_interval = payload
        .get("healthCheckIntervalMinutes")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    match service
        .update_settings(org_id, show_onboarding, auto_check, health_check_interval)
        .await
    {
        Ok(settings) => Json(serde_json::to_value(settings).unwrap_or_default()).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e).into_response(),
    }
}
