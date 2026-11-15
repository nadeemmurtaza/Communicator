#![allow(dead_code)]

//! CX QA Scoring API routes — Phase 16. Thin gRPC client for svc-cx-qa-scoring.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use crate::db::AppState;

use newax_proto::cx_qa_scoring::v1::{
    cx_qa_scoring_service_client::CxQaScoringServiceClient, ConversationIdRequest, Empty,
    UpdateQaRuleRequest as GrpcUpdateQaRuleRequest,
};

async fn client() -> Result<CxQaScoringServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("CX_QA_SCORING_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50069".to_string());
    CxQaScoringServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("cx-qa-scoring service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    let status = if e.code() == tonic::Code::NotFound { StatusCode::NOT_FOUND } else { StatusCode::INTERNAL_SERVER_ERROR };
    (status, e.message().to_string())
}

#[derive(Debug, Serialize)]
pub struct CXSummary {
    pub avg_score: f64,
    pub total_conversations: i64,
    pub scored_conversations: i64,
    pub trend: String,
}

#[derive(Debug, Serialize)]
pub struct QAScore {
    pub conversation_id: String,
    pub score: f64,
    pub feedback: String,
    pub rules_triggered: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchScoringResult {
    pub total: i64,
    pub scored: i64,
    pub failed: i64,
    pub avg_score: f64,
}

#[derive(Debug, Serialize)]
pub struct QAScoringRule {
    pub id: String,
    pub name: String,
    pub is_enabled: bool,
    pub score_impact: f64,
}

/// GET /cx-qa/summary — Get CX score summary with trend analysis.
pub async fn get_cx_summary(State(_state): State<AppState>) -> Result<Json<CXSummary>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_cx_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(CXSummary {
        avg_score: resp.avg_score,
        total_conversations: resp.total_conversations,
        scored_conversations: resp.scored_conversations,
        trend: resp.trend,
    }))
}

/// GET /cx-qa/{conversation_id} — Get QA score for a conversation.
pub async fn get_qa_score(
    State(_state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<QAScore>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c
        .get_qa_score(ConversationIdRequest { conversation_id })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(QAScore {
        conversation_id: resp.conversation_id,
        score: resp.score,
        feedback: resp.feedback,
        rules_triggered: resp.rules_triggered,
    }))
}

/// POST /cx-qa/batch-score — Score all unresolved conversations.
pub async fn batch_score(State(_state): State<AppState>) -> Result<Json<BatchScoringResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.batch_score(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(BatchScoringResult {
        total: resp.total,
        scored: resp.scored,
        failed: resp.failed,
        avg_score: resp.avg_score,
    }))
}

/// GET /cx-qa/rules — List all QA scoring rules.
pub async fn list_qa_rules(State(_state): State<AppState>) -> Result<Json<Vec<QAScoringRule>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.list_qa_rules(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(
        resp.rules
            .into_iter()
            .map(|r| QAScoringRule { id: r.id, name: r.name, is_enabled: r.is_enabled, score_impact: r.score_impact })
            .collect(),
    ))
}

/// PUT /cx-qa/rules/{id} — Update a QA scoring rule.
pub async fn update_qa_rule(
    State(_state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<QAScoringRule>, (StatusCode, String)> {
    let is_enabled = payload.get("is_enabled").and_then(|v| v.as_bool());
    let score_impact = payload.get("score_impact").and_then(|v| v.as_f64());

    let mut c = client().await?;
    let resp = c
        .update_qa_rule(GrpcUpdateQaRuleRequest { rule_id, is_enabled, score_impact })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(QAScoringRule {
        id: resp.id,
        name: resp.name,
        is_enabled: resp.is_enabled,
        score_impact: resp.score_impact,
    }))
}
