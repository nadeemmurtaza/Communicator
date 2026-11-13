//! Fin Operator — API endpoints (Phase 17).
//!
//! Endpoints:
//! - GET  /operator/summary              — Get operator dashboard summary
//! - GET  /operator/config               — Get operator configuration
//! - PUT  /operator/config               — Update operator configuration
//! - POST /operator/scan                 — Run a full operator scan
//! - GET  /operator/proposals            — List proposals with filters
//! - GET  /operator/proposals/{id}       — Get a single proposal
//! - POST /operator/proposals/{id}/approve  — Approve a proposal
//! - POST /operator/proposals/{id}/dismiss  — Dismiss a proposal
//! - GET  /operator/logs                 — Get performance logs
//! - GET  /operator/snapshots            — Get metrics snapshots

use crate::db::AppState;
use crate::services::fin_operator;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Default)]
pub struct ProposalQueryParams {
    pub status: Option<String>,
    pub proposal_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize, Default)]
pub struct LogQueryParams {
    pub action_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize, Default)]
pub struct SnapshotQueryParams {
    pub limit: Option<i32>,
}

/// GET /operator/summary
pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match fin_operator::get_summary(&state).await {
        Ok(summary) => {
            let val = serde_json::to_value(&summary).unwrap_or_else(|_| json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

/// GET /operator/config
pub async fn get_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match fin_operator::get_config(&state).await {
        Ok(config) => {
            let val = serde_json::to_value(&config).unwrap_or_else(|_| json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

/// PUT /operator/config
pub async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match fin_operator::update_config(&state, &body).await {
        Ok(config) => {
            let val = serde_json::to_value(&config).unwrap_or_else(|_| json!({}));
            Ok(Json(val))
        }
        Err((status, msg)) => Err((status, Json(json!({"error": msg})))),
    }
}

/// POST /operator/scan
pub async fn run_scan(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match fin_operator::run_scan(&state).await {
        Ok(result) => {
            let val = serde_json::to_value(&result).unwrap_or_else(|_| json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

/// GET /operator/proposals
pub async fn list_proposals(
    State(state): State<AppState>,
    Query(params): Query<ProposalQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let status = params.status.as_deref();
    let proposal_type = params.proposal_type.as_deref();
    let limit = params.limit.unwrap_or(50);

    match fin_operator::list_proposals(&state, status, proposal_type, limit).await {
        Ok(proposals) => {
            let val = serde_json::to_value(&proposals).unwrap_or_else(|_| json!([]));
            Ok(Json(json!({
                "proposals": val,
                "total": proposals.len(),
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

/// GET /operator/proposals/{id}
pub async fn get_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match fin_operator::get_proposal(&state, &id).await {
        Ok(proposal) => {
            let val = serde_json::to_value(&proposal).unwrap_or_else(|_| json!({}));
            Ok(Json(val))
        }
        Err((status, msg)) => Err((status, Json(json!({"error": msg})))),
    }
}

/// POST /operator/proposals/{id}/approve
pub async fn approve_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let reviewer = body.get("reviewer").and_then(|v| v.as_str()).unwrap_or("admin").to_string();

    match fin_operator::approve_proposal(&state, &id, &reviewer).await {
        Ok(proposal) => {
            let val = serde_json::to_value(&proposal).unwrap_or_else(|_| json!({}));
            Ok(Json(val))
        }
        Err((status, msg)) => Err((status, Json(json!({"error": msg})))),
    }
}

/// POST /operator/proposals/{id}/dismiss
pub async fn dismiss_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let reviewer = body.get("reviewer").and_then(|v| v.as_str()).unwrap_or("admin").to_string();

    match fin_operator::dismiss_proposal(&state, &id, &reviewer).await {
        Ok(()) => Ok(Json(json!({
            "message": "Proposal dismissed",
            "id": id,
        }))),
        Err((status, msg)) => Err((status, Json(json!({"error": msg})))),
    }
}

/// GET /operator/logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let action_type = params.action_type.as_deref();
    let limit = params.limit.unwrap_or(50);

    match fin_operator::list_logs(&state, action_type, limit).await {
        Ok(logs) => {
            let val = serde_json::to_value(&logs).unwrap_or_else(|_| json!([]));
            Ok(Json(json!({
                "logs": val,
                "total": logs.len(),
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

/// GET /operator/snapshots
pub async fn list_snapshots(
    State(state): State<AppState>,
    Query(params): Query<SnapshotQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let limit = params.limit.unwrap_or(20);

    match fin_operator::list_snapshots(&state, limit).await {
        Ok(snapshots) => {
            let val = serde_json::to_value(&snapshots).unwrap_or_else(|_| json!([]));
            Ok(Json(json!({
                "snapshots": val,
                "total": snapshots.len(),
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}
