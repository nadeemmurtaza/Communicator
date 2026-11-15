//! Phase 38: A/B Testing & Phased Deployments Routes
//!
//! Thin gRPC client forwarding to svc-ab-testing; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use crate::AppState;

use newax_proto::ab_testing::v1::{
    ab_testing_service_client::AbTestingServiceClient, AssignRequest as GrpcAssignRequest,
    CreateExperimentRequest as GrpcCreateExperimentRequest, CreateExperimentVariantInput, Empty, ExperimentIdRequest,
    ListExperimentsRequest, RecordResultRequest as GrpcRecordResultRequest, UpdateStatusRequest as GrpcUpdateStatusRequest,
};

async fn client() -> Result<AbTestingServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("AB_TESTING_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50074".to_string());
    AbTestingServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("ab-testing service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    let status = if e.code() == tonic::Code::InvalidArgument { StatusCode::BAD_REQUEST } else { StatusCode::INTERNAL_SERVER_ERROR };
    (status, e.message().to_string())
}

fn not_found_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, e.message().to_string())
}

// ——— Wire types (mirror the old service types) ———

#[derive(Debug, Serialize, Clone)]
pub struct ABExperiment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub experiment_type: String,
    pub status: String,
    pub traffic_split: f64,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub winner_variant_id: Option<String>,
    pub created_by: Option<String>,
    pub org_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ABVariant {
    pub id: String,
    pub experiment_id: String,
    pub variant_name: String,
    pub config: serde_json::Value,
    pub is_control: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ABAssignment {
    pub id: String,
    pub experiment_id: String,
    pub variant_id: String,
    pub conversation_id: String,
    pub contact_id: Option<String>,
    pub assigned_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ABResult {
    pub id: String,
    pub experiment_id: String,
    pub variant_id: String,
    pub assignment_id: Option<String>,
    pub metric_name: String,
    pub metric_value: f64,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct VariantStats {
    pub variant_id: String,
    pub variant_name: String,
    pub is_control: bool,
    pub assignments: i64,
    pub results: i64,
    pub avg_metric_values: std::collections::HashMap<String, f64>,
    pub min_metric_values: std::collections::HashMap<String, f64>,
    pub max_metric_values: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ABExperimentSummary {
    pub id: String,
    pub experiment_id: String,
    pub total_assignments: i64,
    pub total_results: i64,
    pub variant_stats: Vec<VariantStats>,
    pub statistical_significance: f64,
    pub recommended_winner: Option<String>,
    pub snapshot_at: String,
}

#[derive(Debug, Serialize, Clone, serde::Deserialize)]
pub struct ABSettings {
    pub id: String,
    pub org_id: String,
    pub enabled: bool,
    pub auto_detect_winner: bool,
    pub min_sample_size: i32,
    pub significance_threshold: f64,
    pub max_concurrent_experiments: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExperimentTypeCount {
    pub experiment_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ABTestingSummary {
    pub total_experiments: i64,
    pub running_experiments: i64,
    pub completed_experiments: i64,
    pub total_assignments: i64,
    pub total_results: i64,
    pub experiments_by_type: Vec<ExperimentTypeCount>,
    pub recent_experiments: Vec<ABExperiment>,
    pub settings: ABSettings,
}

#[derive(Debug, Serialize, Clone)]
pub struct AssignmentResult {
    pub assignment: ABAssignment,
    pub variant: ABVariant,
}

#[derive(Debug, Serialize, Clone)]
pub struct CreateExperimentResult {
    pub experiment: ABExperiment,
    pub variants: Vec<ABVariant>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SummarySnapshotResult {
    pub summary: ABExperimentSummary,
}

// ——— proto <-> wire conversions ———

fn experiment_from_grpc(e: newax_proto::ab_testing::v1::AbExperiment) -> ABExperiment {
    ABExperiment {
        id: e.id,
        name: e.name,
        description: e.description,
        experiment_type: e.experiment_type,
        status: e.status,
        traffic_split: e.traffic_split,
        start_at: e.start_at,
        end_at: e.end_at,
        winner_variant_id: e.winner_variant_id,
        created_by: e.created_by,
        org_id: e.org_id,
        created_at: e.created_at,
        updated_at: e.updated_at,
    }
}

fn variant_from_grpc(v: newax_proto::ab_testing::v1::AbVariant) -> ABVariant {
    ABVariant {
        id: v.id,
        experiment_id: v.experiment_id,
        variant_name: v.variant_name,
        config: serde_json::from_str(&v.config_json).unwrap_or(serde_json::Value::Object(Default::default())),
        is_control: v.is_control,
        created_at: v.created_at,
    }
}

fn result_from_grpc(r: newax_proto::ab_testing::v1::AbResult) -> ABResult {
    ABResult {
        id: r.id,
        experiment_id: r.experiment_id,
        variant_id: r.variant_id,
        assignment_id: r.assignment_id,
        metric_name: r.metric_name,
        metric_value: r.metric_value,
        metadata: serde_json::from_str(&r.metadata_json).unwrap_or(serde_json::Value::Object(Default::default())),
        created_at: r.created_at,
    }
}

fn settings_from_grpc(s: newax_proto::ab_testing::v1::AbSettings) -> ABSettings {
    ABSettings {
        id: s.id,
        org_id: s.org_id,
        enabled: s.enabled,
        auto_detect_winner: s.auto_detect_winner,
        min_sample_size: s.min_sample_size,
        significance_threshold: s.significance_threshold,
        max_concurrent_experiments: s.max_concurrent_experiments,
    }
}

fn settings_to_grpc(s: ABSettings) -> newax_proto::ab_testing::v1::AbSettings {
    newax_proto::ab_testing::v1::AbSettings {
        id: s.id,
        org_id: s.org_id,
        enabled: s.enabled,
        auto_detect_winner: s.auto_detect_winner,
        min_sample_size: s.min_sample_size,
        significance_threshold: s.significance_threshold,
        max_concurrent_experiments: s.max_concurrent_experiments,
    }
}

fn variant_stats_from_grpc(v: newax_proto::ab_testing::v1::VariantStats) -> VariantStats {
    VariantStats {
        variant_id: v.variant_id,
        variant_name: v.variant_name,
        is_control: v.is_control,
        assignments: v.assignments,
        results: v.results,
        avg_metric_values: v.avg_metric_values,
        min_metric_values: v.min_metric_values,
        max_metric_values: v.max_metric_values,
    }
}

fn summary_from_grpc(s: newax_proto::ab_testing::v1::AbExperimentSummary) -> ABExperimentSummary {
    ABExperimentSummary {
        id: s.id,
        experiment_id: s.experiment_id,
        total_assignments: s.total_assignments,
        total_results: s.total_results,
        variant_stats: s.variant_stats.into_iter().map(variant_stats_from_grpc).collect(),
        statistical_significance: s.statistical_significance,
        recommended_winner: s.recommended_winner,
        snapshot_at: s.snapshot_at,
    }
}

// ——— Handlers ———

/// GET /ab-testing/summary
pub async fn get_summary(State(_state): State<AppState>) -> Result<Json<ABTestingSummary>, (StatusCode, String)> {
    let mut c = client().await?;
    let s = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(ABTestingSummary {
        total_experiments: s.total_experiments,
        running_experiments: s.running_experiments,
        completed_experiments: s.completed_experiments,
        total_assignments: s.total_assignments,
        total_results: s.total_results,
        experiments_by_type: s
            .experiments_by_type
            .into_iter()
            .map(|t| ExperimentTypeCount { experiment_type: t.experiment_type, count: t.count })
            .collect(),
        recent_experiments: s.recent_experiments.into_iter().map(experiment_from_grpc).collect(),
        settings: settings_from_grpc(s.settings.unwrap_or_default()),
    }))
}

/// GET /ab-testing/experiments
pub async fn list_experiments(
    State(_state): State<AppState>,
    Query(params): Query<ExperimentListParams>,
) -> Result<Json<Vec<ABExperiment>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .list_experiments(ListExperimentsRequest { status: params.status, experiment_type: params.experiment_type })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(resp.experiments.into_iter().map(experiment_from_grpc).collect()))
}

/// POST /ab-testing/experiments
pub async fn create_experiment(
    State(_state): State<AppState>,
    Json(body): Json<CreateExperimentRequest>,
) -> Result<Json<CreateExperimentResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let variants: Vec<CreateExperimentVariantInput> = body
        .variants
        .into_iter()
        .map(|v| CreateExperimentVariantInput { variant_name: v.variant_name, config_json: v.config.to_string() })
        .collect();
    let resp = c
        .create_experiment(GrpcCreateExperimentRequest {
            name: body.name,
            description: body.description,
            experiment_type: body.experiment_type,
            traffic_split: body.traffic_split,
            variants,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(CreateExperimentResult {
        experiment: experiment_from_grpc(resp.experiment.unwrap_or_default()),
        variants: resp.variants.into_iter().map(variant_from_grpc).collect(),
    }))
}

/// GET /ab-testing/experiments/{id}
pub async fn get_experiment(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ABExperiment>, (StatusCode, String)> {
    let mut c = client().await?;
    let exp = c.get_experiment(ExperimentIdRequest { experiment_id }).await.map_err(not_found_err)?.into_inner();
    Ok(Json(experiment_from_grpc(exp)))
}

/// PUT /ab-testing/experiments/{id}/status
pub async fn update_experiment_status(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<ABExperiment>, (StatusCode, String)> {
    let mut c = client().await?;
    let exp = c
        .update_experiment_status(GrpcUpdateStatusRequest { experiment_id, status: body.status })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(experiment_from_grpc(exp)))
}

/// DELETE /ab-testing/experiments/{id}
pub async fn delete_experiment(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client().await?;
    c.delete_experiment(ExperimentIdRequest { experiment_id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /ab-testing/experiments/{id}/variants
pub async fn get_variants(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<Vec<ABVariant>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_variants(ExperimentIdRequest { experiment_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(resp.variants.into_iter().map(variant_from_grpc).collect()))
}

/// POST /ab-testing/experiments/{id}/assign
pub async fn assign_conversation(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
    Json(body): Json<AssignRequest>,
) -> Result<Json<AssignmentResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .assign_conversation(GrpcAssignRequest { experiment_id, conversation_id: body.conversation_id, contact_id: body.contact_id })
        .await
        .map_err(grpc_err)?
        .into_inner();
    let assignment = resp.assignment.unwrap_or_default();
    Ok(Json(AssignmentResult {
        assignment: ABAssignment {
            id: assignment.id,
            experiment_id: assignment.experiment_id,
            variant_id: assignment.variant_id,
            conversation_id: assignment.conversation_id,
            contact_id: assignment.contact_id,
            assigned_at: assignment.assigned_at,
        },
        variant: variant_from_grpc(resp.variant.unwrap_or_default()),
    }))
}

/// POST /ab-testing/experiments/{id}/results
pub async fn record_result(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
    Json(body): Json<RecordResultRequest>,
) -> Result<Json<ABResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .record_result(GrpcRecordResultRequest {
            experiment_id,
            variant_id: body.variant_id,
            assignment_id: body.assignment_id,
            metric_name: body.metric_name,
            metric_value: body.metric_value,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(result_from_grpc(result)))
}

/// GET /ab-testing/experiments/{id}/results
pub async fn get_results(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<Vec<ABResult>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_results(ExperimentIdRequest { experiment_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(resp.results.into_iter().map(result_from_grpc).collect()))
}

/// POST /ab-testing/experiments/{id}/summary
pub async fn generate_summary(
    State(_state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<SummarySnapshotResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.generate_summary(ExperimentIdRequest { experiment_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(SummarySnapshotResult { summary: summary_from_grpc(resp.summary.unwrap_or_default()) }))
}

/// GET /ab-testing/settings
pub async fn get_settings(State(_state): State<AppState>) -> Result<Json<ABSettings>, (StatusCode, String)> {
    let mut c = client().await?;
    let settings = c.get_settings(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(settings_from_grpc(settings)))
}

/// PUT /ab-testing/settings
pub async fn update_settings(
    State(_state): State<AppState>,
    Json(body): Json<ABSettings>,
) -> Result<Json<ABSettings>, (StatusCode, String)> {
    let mut c = client().await?;
    let settings = c.update_settings(settings_to_grpc(body)).await.map_err(grpc_err)?.into_inner();
    Ok(Json(settings_from_grpc(settings)))
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
