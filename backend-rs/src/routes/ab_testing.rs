//! Phase 38: A/B Testing & Phased Deployments Routes

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use crate::AppState;
use crate::services::ab_testing::{
    ABTestingService, ABVariantInput, CreateExperimentResult, AssignmentResult, ABResult,
    SummarySnapshotResult, ABSettings, ABExperiment, ABVariant, ABTestingSummary,
};

/// GET /ab-testing/summary
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<ABTestingSummary>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.get_summary().await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /ab-testing/experiments
pub async fn list_experiments(
    State(state): State<AppState>,
    Query(params): Query<ExperimentListParams>,
) -> Result<Json<Vec<ABExperiment>>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.list_experiments(params.status.as_deref(), params.experiment_type.as_deref()).await {
        Ok(experiments) => Ok(Json(experiments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// POST /ab-testing/experiments
pub async fn create_experiment(
    State(state): State<AppState>,
    Json(body): Json<CreateExperimentRequest>,
) -> Result<Json<CreateExperimentResult>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    let variants: Vec<ABVariantInput> = body.variants.into_iter().map(|v| ABVariantInput {
        variant_name: v.variant_name,
        config: v.config,
    }).collect();

    match service.create_experiment(
        &body.name,
        body.description.as_deref(),
        &body.experiment_type,
        body.traffic_split,
        &variants,
    ).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /ab-testing/experiments/{id}
pub async fn get_experiment(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ABExperiment>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.get_experiment(&experiment_id).await {
        Ok(exp) => Ok(Json(exp)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

/// PUT /ab-testing/experiments/{id}/status
pub async fn update_experiment_status(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<ABExperiment>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.update_experiment_status(&experiment_id, &body.status).await {
        Ok(exp) => Ok(Json(exp)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// DELETE /ab-testing/experiments/{id}
pub async fn delete_experiment(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.delete_experiment(&experiment_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /ab-testing/experiments/{id}/variants
pub async fn get_variants(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<Vec<ABVariant>>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.get_variants(&experiment_id).await {
        Ok(variants) => Ok(Json(variants)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// POST /ab-testing/experiments/{id}/assign
pub async fn assign_conversation(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
    Json(body): Json<AssignRequest>,
) -> Result<Json<AssignmentResult>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.assign_conversation(&experiment_id, &body.conversation_id, body.contact_id.as_deref()).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

/// POST /ab-testing/experiments/{id}/results
pub async fn record_result(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
    Json(body): Json<RecordResultRequest>,
) -> Result<Json<ABResult>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.record_result(
        &experiment_id,
        &body.variant_id,
        &body.assignment_id,
        &body.metric_name,
        body.metric_value,
    ).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /ab-testing/experiments/{id}/results
pub async fn get_results(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<Vec<ABResult>>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.get_results(&experiment_id).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// POST /ab-testing/experiments/{id}/summary
pub async fn generate_summary(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<SummarySnapshotResult>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.generate_summary(&experiment_id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /ab-testing/settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<ABSettings>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.get_settings().await {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// PUT /ab-testing/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<ABSettings>,
) -> Result<Json<ABSettings>, (StatusCode, String)> {
    let service = ABTestingService::new(&state.db);
    match service.update_settings(&body).await {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Request/Response types

#[derive(serde::Deserialize)]
pub struct ExperimentListParams {
    #[allow(dead_code)]
    pub status: Option<String>,
    #[allow(dead_code)]
    pub experiment_type: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateExperimentRequest {
    pub name: String,
    pub description: Option<String>,
    pub experiment_type: String,
    pub traffic_split: f64,
    pub variants: Vec<CreateVariantRequest>,
}

#[derive(serde::Deserialize)]
pub struct CreateVariantRequest {
    pub variant_name: String,
    pub config: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(serde::Deserialize)]
pub struct AssignRequest {
    pub conversation_id: String,
    pub contact_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RecordResultRequest {
    pub variant_id: String,
    pub assignment_id: String,
    pub metric_name: String,
    pub metric_value: f64,
}
