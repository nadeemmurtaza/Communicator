use newax_monitoring as m;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.monitoring.v1");
}

use proto::{
    monitoring_service_server::{MonitoringService, MonitoringServiceServer},
    AcknowledgeRequest, AcknowledgeResponse, AlertEvent as ProtoAlertEvent, AlertEventsResponse, AlertIdRequest,
    AlertRule as ProtoAlertRule, AlertRulesResponse, AlertsRequest, CollectResponse, ComponentHealth as ProtoComponent,
    CreateAlertRuleRequest, CreateAlertRuleResponse, DeleteResponse, Empty, EvaluateResponse,
    HealthCheckResult as ProtoHealth, JsonResponse, MetricSummary as ProtoSummary, MetricsRequest, MetricsResponse,
    ResolveResponse, RuleIdRequest, SummariesResponse, SystemMetric as ProtoMetric, UpdateAlertRuleRequest,
    UpdateAlertRuleResponse,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn metric_to_proto(x: m::SystemMetric) -> ProtoMetric {
    ProtoMetric {
        id: x.id,
        metric_name: x.metric_name,
        metric_value: x.metric_value,
        metric_type: x.metric_type,
        labels: x.labels,
        collected_at: x.collected_at,
        created_at: x.created_at.to_rfc3339(),
    }
}

fn summary_to_proto(x: m::MetricSummary) -> ProtoSummary {
    ProtoSummary {
        metric_name: x.metric_name,
        avg_value: x.avg_value,
        min_value: x.min_value,
        max_value: x.max_value,
        latest_value: x.latest_value,
        sample_count: x.sample_count,
    }
}

fn health_to_proto(x: m::HealthCheckResult) -> ProtoHealth {
    ProtoHealth {
        overall_status: x.overall_status,
        components: x
            .components
            .into_iter()
            .map(|c| ProtoComponent { name: c.name, status: c.status, detail: c.detail })
            .collect(),
        checked_at: x.checked_at,
    }
}

fn event_to_proto(x: m::AlertEvent) -> ProtoAlertEvent {
    ProtoAlertEvent {
        id: x.id,
        rule_id: x.rule_id,
        alert_name: x.alert_name,
        severity: x.severity,
        metric_name: x.metric_name,
        metric_value: x.metric_value,
        threshold: x.threshold,
        condition: x.condition,
        message: x.message,
        acknowledged: x.acknowledged,
        acknowledged_by: x.acknowledged_by,
        acknowledged_at: x.acknowledged_at,
        resolved: x.resolved,
        resolved_at: x.resolved_at.map(|d| d.to_rfc3339()),
        created_at: x.created_at.to_rfc3339(),
    }
}

fn rule_to_proto(x: m::AlertRule) -> ProtoAlertRule {
    ProtoAlertRule {
        id: x.id,
        name: x.name,
        description: x.description,
        metric_name: x.metric_name,
        condition: x.condition,
        threshold: x.threshold,
        window_minutes: x.window_minutes,
        severity: x.severity,
        enabled: x.enabled,
        notification_webhook: x.notification_webhook,
        cooldown_minutes: x.cooldown_minutes,
        created_at: x.created_at.to_rfc3339(),
        updated_at: x.updated_at.to_rfc3339(),
    }
}

#[tonic::async_trait]
impl MonitoringService for Svc {
    async fn health_check_extended(&self, _req: Request<Empty>) -> Result<Response<ProtoHealth>, Status> {
        Ok(Response::new(health_to_proto(m::run_health_checks(&self.pool).await)))
    }

    async fn get_metrics(&self, req: Request<MetricsRequest>) -> Result<Response<MetricsResponse>, Status> {
        let r = req.into_inner();
        let metrics = m::get_metrics(&self.pool, r.name.as_deref(), r.window_hours, r.limit).await;
        let count = metrics.len() as i64;
        Ok(Response::new(MetricsResponse {
            metrics: metrics.into_iter().map(metric_to_proto).collect(),
            window_hours: r.window_hours,
            count,
        }))
    }

    async fn get_metrics_summary(&self, req: Request<MetricsRequest>) -> Result<Response<SummariesResponse>, Status> {
        let r = req.into_inner();
        let summaries = m::get_metric_summaries(&self.pool, r.window_hours).await.map_err(to_status)?;
        let count = summaries.len() as i64;
        Ok(Response::new(SummariesResponse {
            summaries: summaries.into_iter().map(summary_to_proto).collect(),
            window_hours: r.window_hours,
            count,
        }))
    }

    async fn collect_metrics(&self, _req: Request<Empty>) -> Result<Response<CollectResponse>, Status> {
        let collected = m::collect_all_metrics(&self.pool).await.map_err(to_status)?;
        let alerts_fired = m::evaluate_alert_rules(&self.pool).await.unwrap_or(0);
        Ok(Response::new(CollectResponse {
            collected,
            alerts_fired,
            collected_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    async fn get_alert_events(&self, req: Request<AlertsRequest>) -> Result<Response<AlertEventsResponse>, Status> {
        let r = req.into_inner();
        let events = m::get_alert_events(&self.pool, r.severity.as_deref(), r.acknowledged, r.resolved, r.limit).await;
        let count = events.len() as i64;
        Ok(Response::new(AlertEventsResponse { alerts: events.into_iter().map(event_to_proto).collect(), count }))
    }

    async fn acknowledge_alert(&self, req: Request<AcknowledgeRequest>) -> Result<Response<AcknowledgeResponse>, Status> {
        let r = req.into_inner();
        let result = m::acknowledge_alert(&self.pool, &r.alert_id, r.acknowledged_by.as_deref()).await.map_err(to_status)?;
        match result {
            Some((acknowledged_by, acknowledged_at)) => Ok(Response::new(AcknowledgeResponse {
                acknowledged: true,
                alert_id: r.alert_id,
                acknowledged_by,
                acknowledged_at,
            })),
            None => Err(Status::not_found(format!("Alert {} not found or already acknowledged", r.alert_id))),
        }
    }

    async fn resolve_alert(&self, req: Request<AlertIdRequest>) -> Result<Response<ResolveResponse>, Status> {
        let alert_id = req.into_inner().alert_id;
        let result = m::resolve_alert(&self.pool, &alert_id).await.map_err(to_status)?;
        match result {
            Some(resolved_at) => Ok(Response::new(ResolveResponse { resolved: true, alert_id, resolved_at })),
            None => Err(Status::not_found(format!("Alert {} not found or already resolved", alert_id))),
        }
    }

    async fn get_alert_rules(&self, _req: Request<Empty>) -> Result<Response<AlertRulesResponse>, Status> {
        let rules = m::get_alert_rules(&self.pool).await;
        let count = rules.len() as i64;
        Ok(Response::new(AlertRulesResponse { rules: rules.into_iter().map(rule_to_proto).collect(), count }))
    }

    async fn get_alert_rule(&self, req: Request<RuleIdRequest>) -> Result<Response<ProtoAlertRule>, Status> {
        let rule_id = req.into_inner().rule_id;
        match m::get_alert_rule(&self.pool, &rule_id).await {
            Some(rule) => Ok(Response::new(rule_to_proto(rule))),
            None => Err(Status::not_found(format!("Alert rule {} not found", rule_id))),
        }
    }

    async fn create_alert_rule(
        &self,
        req: Request<CreateAlertRuleRequest>,
    ) -> Result<Response<CreateAlertRuleResponse>, Status> {
        let r = req.into_inner();
        let logic_req = m::CreateAlertRuleRequest {
            name: r.name,
            description: r.description,
            metric_name: r.metric_name,
            condition: r.condition,
            threshold: r.threshold,
            window_minutes: r.window_minutes,
            severity: r.severity,
            notification_webhook: r.notification_webhook,
            cooldown_minutes: r.cooldown_minutes,
        };
        let (id, name, created_at) = m::create_alert_rule(&self.pool, &logic_req).await.map_err(to_status)?;
        Ok(Response::new(CreateAlertRuleResponse { id, name, created_at }))
    }

    async fn update_alert_rule(
        &self,
        req: Request<UpdateAlertRuleRequest>,
    ) -> Result<Response<UpdateAlertRuleResponse>, Status> {
        let r = req.into_inner();
        let rule_id = r.rule_id.clone();
        let logic_req = m::UpdateAlertRuleRequest {
            name: r.name,
            description: r.description,
            metric_name: r.metric_name,
            condition: r.condition,
            threshold: r.threshold,
            window_minutes: r.window_minutes,
            severity: r.severity,
            enabled: r.enabled,
            notification_webhook: r.notification_webhook,
            cooldown_minutes: r.cooldown_minutes,
        };
        let result = m::update_alert_rule(&self.pool, &rule_id, &logic_req).await.map_err(to_status)?;
        match result {
            Some(updated_at) => Ok(Response::new(UpdateAlertRuleResponse { updated: true, rule_id, updated_at })),
            None => Err(Status::not_found(format!("Alert rule {} not found", rule_id))),
        }
    }

    async fn delete_alert_rule(&self, req: Request<RuleIdRequest>) -> Result<Response<DeleteResponse>, Status> {
        let rule_id = req.into_inner().rule_id;
        let deleted = m::delete_alert_rule(&self.pool, &rule_id).await.map_err(to_status)?;
        if deleted {
            Ok(Response::new(DeleteResponse { deleted: true, rule_id }))
        } else {
            Err(Status::not_found(format!("Alert rule {} not found", rule_id)))
        }
    }

    async fn get_dashboard_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = m::get_dashboard_summary(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(JsonResponse { json: v.to_string() }))
    }

    async fn evaluate_alerts(&self, _req: Request<Empty>) -> Result<Response<EvaluateResponse>, Status> {
        let alerts_fired = m::evaluate_alert_rules(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(EvaluateResponse {
            evaluated: true,
            alerts_fired,
            evaluated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_monitoring=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("MONITORING_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50066);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-monitoring starting (gRPC)");
    Server::builder()
        .add_service(MonitoringServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
