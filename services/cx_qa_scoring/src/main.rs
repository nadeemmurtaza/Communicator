use newax_cx_qa_scoring as qa;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.cx_qa_scoring.v1");
}

use proto::{
    cx_qa_scoring_service_server::{CxQaScoringService, CxQaScoringServiceServer},
    BatchScoringResult as ProtoBatchResult, ConversationIdRequest, CxSummary as ProtoSummary, Empty,
    QaRulesResponse, QaScore as ProtoScore, QaScoringRule as ProtoRule, UpdateQaRuleRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn summary_to_proto(x: qa::CXSummary) -> ProtoSummary {
    ProtoSummary {
        avg_score: x.avg_score,
        total_conversations: x.total_conversations,
        scored_conversations: x.scored_conversations,
        trend: x.trend,
    }
}

fn score_to_proto(x: qa::QAScore) -> ProtoScore {
    ProtoScore {
        conversation_id: x.conversation_id,
        score: x.score,
        feedback: x.feedback,
        rules_triggered: x.rules_triggered,
    }
}

fn batch_to_proto(x: qa::BatchScoringResult) -> ProtoBatchResult {
    ProtoBatchResult {
        total: x.total,
        scored: x.scored,
        failed: x.failed,
        avg_score: x.avg_score,
    }
}

fn rule_to_proto(x: qa::QAScoringRule) -> ProtoRule {
    ProtoRule {
        id: x.id,
        name: x.name,
        is_enabled: x.is_enabled,
        score_impact: x.score_impact,
    }
}

#[tonic::async_trait]
impl CxQaScoringService for Svc {
    async fn get_cx_summary(&self, _req: Request<Empty>) -> Result<Response<ProtoSummary>, Status> {
        let summary = qa::get_cx_summary(&self.pool).await.map_err(Status::internal)?;
        Ok(Response::new(summary_to_proto(summary)))
    }

    async fn get_qa_score(&self, req: Request<ConversationIdRequest>) -> Result<Response<ProtoScore>, Status> {
        let conversation_id = req.into_inner().conversation_id;
        let score = qa::score_conversation(&self.pool, &conversation_id).await.map_err(|e| {
            if e.contains("No messages") {
                Status::not_found(e)
            } else {
                Status::internal(e)
            }
        })?;
        Ok(Response::new(score_to_proto(score)))
    }

    async fn batch_score(&self, _req: Request<Empty>) -> Result<Response<ProtoBatchResult>, Status> {
        let result = qa::batch_score_conversations(&self.pool).await.map_err(Status::internal)?;
        Ok(Response::new(batch_to_proto(result)))
    }

    async fn list_qa_rules(&self, _req: Request<Empty>) -> Result<Response<QaRulesResponse>, Status> {
        let rules = qa::list_qa_rules(&self.pool).await.map_err(Status::internal)?;
        Ok(Response::new(QaRulesResponse { rules: rules.into_iter().map(rule_to_proto).collect() }))
    }

    async fn update_qa_rule(&self, req: Request<UpdateQaRuleRequest>) -> Result<Response<ProtoRule>, Status> {
        let r = req.into_inner();
        let rule = qa::update_qa_rule(&self.pool, &r.rule_id, r.is_enabled, r.score_impact)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(rule_to_proto(rule)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_cx_qa_scoring=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("CX_QA_SCORING_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50069);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-cx-qa-scoring starting (gRPC)");
    Server::builder()
        .add_service(CxQaScoringServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
