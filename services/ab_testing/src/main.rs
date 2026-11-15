use newax_ab_testing as ab;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.ab_testing.v1");
}

use proto::{
    ab_testing_service_server::{AbTestingService, AbTestingServiceServer},
    AbExperiment as ProtoExperiment, AbExperimentSummary as ProtoExperimentSummary, AbResult as ProtoResult,
    AbSettings as ProtoSettings, AbTestingSummary as ProtoSummary, AbVariant as ProtoVariant, AssignRequest,
    AssignmentResult as ProtoAssignmentResult, CreateExperimentRequest, CreateExperimentResult as ProtoCreateResult,
    Empty, ExperimentIdRequest, ExperimentTypeCount as ProtoTypeCount, ExperimentsResponse, ListExperimentsRequest,
    RecordResultRequest, ResultsResponse, SummarySnapshotResult as ProtoSnapshotResult, UpdateStatusRequest,
    VariantStats as ProtoVariantStats, VariantsResponse,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn experiment_to_proto(e: ab::ABExperiment) -> ProtoExperiment {
    ProtoExperiment {
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

fn variant_to_proto(v: ab::ABVariant) -> ProtoVariant {
    ProtoVariant {
        id: v.id,
        experiment_id: v.experiment_id,
        variant_name: v.variant_name,
        config_json: v.config.to_string(),
        is_control: v.is_control,
        created_at: v.created_at,
    }
}

fn result_to_proto(r: ab::ABResult) -> ProtoResult {
    ProtoResult {
        id: r.id,
        experiment_id: r.experiment_id,
        variant_id: r.variant_id,
        assignment_id: r.assignment_id,
        metric_name: r.metric_name,
        metric_value: r.metric_value,
        metadata_json: r.metadata.to_string(),
        created_at: r.created_at,
    }
}

fn settings_to_proto(s: ab::ABSettings) -> ProtoSettings {
    ProtoSettings {
        id: s.id,
        org_id: s.org_id,
        enabled: s.enabled,
        auto_detect_winner: s.auto_detect_winner,
        min_sample_size: s.min_sample_size,
        significance_threshold: s.significance_threshold,
        max_concurrent_experiments: s.max_concurrent_experiments,
    }
}

fn settings_from_proto(s: ProtoSettings) -> ab::ABSettings {
    ab::ABSettings {
        id: s.id,
        org_id: s.org_id,
        enabled: s.enabled,
        auto_detect_winner: s.auto_detect_winner,
        min_sample_size: s.min_sample_size,
        significance_threshold: s.significance_threshold,
        max_concurrent_experiments: s.max_concurrent_experiments,
    }
}

fn variant_stats_to_proto(v: ab::VariantStats) -> ProtoVariantStats {
    ProtoVariantStats {
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

fn summary_snapshot_to_proto(s: ab::ABExperimentSummary) -> ProtoExperimentSummary {
    ProtoExperimentSummary {
        id: s.id,
        experiment_id: s.experiment_id,
        total_assignments: s.total_assignments,
        total_results: s.total_results,
        variant_stats: s.variant_stats.into_iter().map(variant_stats_to_proto).collect(),
        statistical_significance: s.statistical_significance,
        recommended_winner: s.recommended_winner,
        snapshot_at: s.snapshot_at,
    }
}

#[tonic::async_trait]
impl AbTestingService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<ProtoSummary>, Status> {
        let s = ab::ABTestingService::new(&self.pool).get_summary().await.map_err(to_status)?;
        Ok(Response::new(ProtoSummary {
            total_experiments: s.total_experiments,
            running_experiments: s.running_experiments,
            completed_experiments: s.completed_experiments,
            total_assignments: s.total_assignments,
            total_results: s.total_results,
            experiments_by_type: s
                .experiments_by_type
                .into_iter()
                .map(|t| ProtoTypeCount { experiment_type: t.experiment_type, count: t.count })
                .collect(),
            recent_experiments: s.recent_experiments.into_iter().map(experiment_to_proto).collect(),
            settings: Some(settings_to_proto(s.settings)),
        }))
    }

    async fn list_experiments(&self, req: Request<ListExperimentsRequest>) -> Result<Response<ExperimentsResponse>, Status> {
        let r = req.into_inner();
        let experiments = ab::ABTestingService::new(&self.pool)
            .list_experiments(r.status.as_deref(), r.experiment_type.as_deref())
            .await
            .map_err(to_status)?;
        Ok(Response::new(ExperimentsResponse { experiments: experiments.into_iter().map(experiment_to_proto).collect() }))
    }

    async fn create_experiment(&self, req: Request<CreateExperimentRequest>) -> Result<Response<ProtoCreateResult>, Status> {
        let r = req.into_inner();
        let variants: Vec<ab::ABVariantInput> = r
            .variants
            .into_iter()
            .map(|v| ab::ABVariantInput {
                variant_name: v.variant_name,
                config: serde_json::from_str(&v.config_json).unwrap_or(serde_json::Value::Object(Default::default())),
            })
            .collect();
        let result = ab::ABTestingService::new(&self.pool)
            .create_experiment(&r.name, r.description.as_deref(), &r.experiment_type, r.traffic_split, &variants)
            .await
            .map_err(to_status)?;
        Ok(Response::new(ProtoCreateResult {
            experiment: Some(experiment_to_proto(result.experiment)),
            variants: result.variants.into_iter().map(variant_to_proto).collect(),
        }))
    }

    async fn get_experiment(&self, req: Request<ExperimentIdRequest>) -> Result<Response<ProtoExperiment>, Status> {
        let experiment_id = req.into_inner().experiment_id;
        let exp = ab::ABTestingService::new(&self.pool)
            .get_experiment(&experiment_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        Ok(Response::new(experiment_to_proto(exp)))
    }

    async fn update_experiment_status(&self, req: Request<UpdateStatusRequest>) -> Result<Response<ProtoExperiment>, Status> {
        let r = req.into_inner();
        let exp = ab::ABTestingService::new(&self.pool)
            .update_experiment_status(&r.experiment_id, &r.status)
            .await
            .map_err(to_status)?;
        Ok(Response::new(experiment_to_proto(exp)))
    }

    async fn delete_experiment(&self, req: Request<ExperimentIdRequest>) -> Result<Response<Empty>, Status> {
        let experiment_id = req.into_inner().experiment_id;
        ab::ABTestingService::new(&self.pool).delete_experiment(&experiment_id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_variants(&self, req: Request<ExperimentIdRequest>) -> Result<Response<VariantsResponse>, Status> {
        let experiment_id = req.into_inner().experiment_id;
        let variants = ab::ABTestingService::new(&self.pool).get_variants(&experiment_id).await.map_err(to_status)?;
        Ok(Response::new(VariantsResponse { variants: variants.into_iter().map(variant_to_proto).collect() }))
    }

    async fn assign_conversation(&self, req: Request<AssignRequest>) -> Result<Response<ProtoAssignmentResult>, Status> {
        let r = req.into_inner();
        let result = ab::ABTestingService::new(&self.pool)
            .assign_conversation(&r.experiment_id, &r.conversation_id, r.contact_id.as_deref())
            .await
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(ProtoAssignmentResult {
            assignment: Some(proto::AbAssignment {
                id: result.assignment.id,
                experiment_id: result.assignment.experiment_id,
                variant_id: result.assignment.variant_id,
                conversation_id: result.assignment.conversation_id,
                contact_id: result.assignment.contact_id,
                assigned_at: result.assignment.assigned_at,
            }),
            variant: Some(variant_to_proto(result.variant)),
        }))
    }

    async fn record_result(&self, req: Request<RecordResultRequest>) -> Result<Response<ProtoResult>, Status> {
        let r = req.into_inner();
        let result = ab::ABTestingService::new(&self.pool)
            .record_result(&r.experiment_id, &r.variant_id, &r.assignment_id, &r.metric_name, r.metric_value)
            .await
            .map_err(to_status)?;
        Ok(Response::new(result_to_proto(result)))
    }

    async fn get_results(&self, req: Request<ExperimentIdRequest>) -> Result<Response<ResultsResponse>, Status> {
        let experiment_id = req.into_inner().experiment_id;
        let results = ab::ABTestingService::new(&self.pool).get_results(&experiment_id).await.map_err(to_status)?;
        Ok(Response::new(ResultsResponse { results: results.into_iter().map(result_to_proto).collect() }))
    }

    async fn generate_summary(&self, req: Request<ExperimentIdRequest>) -> Result<Response<ProtoSnapshotResult>, Status> {
        let experiment_id = req.into_inner().experiment_id;
        let result = ab::ABTestingService::new(&self.pool).generate_summary(&experiment_id).await.map_err(to_status)?;
        Ok(Response::new(ProtoSnapshotResult { summary: Some(summary_snapshot_to_proto(result.summary)) }))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<ProtoSettings>, Status> {
        let settings = ab::ABTestingService::new(&self.pool).get_settings().await.map_err(to_status)?;
        Ok(Response::new(settings_to_proto(settings)))
    }

    async fn update_settings(&self, req: Request<ProtoSettings>) -> Result<Response<ProtoSettings>, Status> {
        let settings = settings_from_proto(req.into_inner());
        let updated = ab::ABTestingService::new(&self.pool).update_settings(&settings).await.map_err(to_status)?;
        Ok(Response::new(settings_to_proto(updated)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_ab_testing=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("AB_TESTING_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50074);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-ab-testing starting (gRPC)");
    Server::builder()
        .add_service(AbTestingServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
