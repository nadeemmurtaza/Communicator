//! Audience-Based Knowledge Scoping Routes
//!
//! Thin gRPC client forwarding to svc-audience-kb-scoping; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;

use newax_proto::audience_kb_scoping::v1::{
    audience_kb_scoping_service_client::AudienceKbScopingServiceClient,
    AddMembershipRequest, Empty, IdRequest, LimitRequest, LimitedIdRequest,
    LinkKbRequest, LogUsageRequest, PairRequest, UpdateSettingsRequest,
};

async fn client() -> Result<AudienceKbScopingServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("AUDIENCE_KB_SCOPING_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50098".to_string());
    AudienceKbScopingServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("audience-kb-scoping service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

// ——— Query Params ———

#[derive(Deserialize)]
pub struct ListLinksQuery {
    segment_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ListMembershipsQuery {
    segment_id: Option<String>,
    contact_id: Option<String>,
    limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct ScopedKBQuery {
    contact_id: String,
}

// ——— Request Bodies ———

#[derive(Deserialize)]
pub struct LinkKBBody {
    segment_id: String,
    kb_entry_id: String,
    priority: Option<i32>,
}

#[derive(Deserialize)]
pub struct AddMembershipBody {
    segment_id: String,
    contact_id: String,
    membership_type: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSettingsBody {
    enabled: Option<bool>,
    fallback_to_global_kb: Option<bool>,
    merge_segments: Option<bool>,
    default_scope: Option<String>,
}

#[derive(Deserialize)]
pub struct LogUsageBody {
    conversation_id: String,
    contact_id: String,
    segment_id: String,
    kb_entry_id: String,
    query_text: Option<String>,
    relevance_score: Option<f64>,
    used_in_response: Option<bool>,
}

// ——— Handlers ———

pub async fn get_summary(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_audience_summary(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_settings(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_settings(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.update_settings(UpdateSettingsRequest {
        enabled: body.enabled,
        fallback_to_global_kb: body.fallback_to_global_kb,
        merge_segments: body.merge_segments,
        default_scope: body.default_scope,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn list_links(
    State(_state): State<AppState>,
    Query(params): Query<ListLinksQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if let Some(segment_id) = params.segment_id {
        let mut cli = client().await.map_err(|e| (e.0, e.1))?;
        let resp = cli.get_segment_kb_links(IdRequest { id: segment_id }).await.map_err(grpc_err)?;
        let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(val))
    } else {
        let mut cli = client().await.map_err(|e| (e.0, e.1))?;
        let resp = cli.get_all_links(Empty {}).await.map_err(grpc_err)?;
        let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(val))
    }
}

pub async fn link_kb(
    State(_state): State<AppState>,
    Json(body): Json<LinkKBBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.link_kb_to_segment(LinkKbRequest {
        segment_id: body.segment_id,
        kb_entry_id: body.kb_entry_id,
        priority: body.priority.unwrap_or(0),
    }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.message().to_string()))?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn unlink_kb(
    State(_state): State<AppState>,
    Path((segment_id, kb_entry_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    cli.unlink_kb_from_segment(PairRequest {
        first: segment_id,
        second: kb_entry_id,
    }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.message().to_string()))?;
    Ok(Json(serde_json::json!({"message": "KB entry unlinked from segment"})))
}

pub async fn get_scoped_kb(
    State(_state): State<AppState>,
    Query(params): Query<ScopedKBQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_scoped_kb_for_contact(IdRequest { id: params.contact_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn list_memberships(
    State(_state): State<AppState>,
    Query(params): Query<ListMembershipsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50);
    if let Some(segment_id) = params.segment_id {
        let mut cli = client().await.map_err(|e| (e.0, e.1))?;
        let resp = cli.get_segment_memberships(IdRequest { id: segment_id }).await.map_err(grpc_err)?;
        let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(val))
    } else if let Some(contact_id) = params.contact_id {
        let mut cli = client().await.map_err(|e| (e.0, e.1))?;
        let resp = cli.get_contact_segment_memberships(IdRequest { id: contact_id }).await.map_err(grpc_err)?;
        let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(val))
    } else {
        let mut cli = client().await.map_err(|e| (e.0, e.1))?;
        let resp = cli.get_all_memberships(LimitRequest { limit }).await.map_err(grpc_err)?;
        let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(val))
    }
}

pub async fn add_membership(
    State(_state): State<AppState>,
    Json(body): Json<AddMembershipBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.add_contact_to_segment(AddMembershipRequest {
        segment_id: body.segment_id,
        contact_id: body.contact_id,
        membership_type: body.membership_type.unwrap_or_else(|| "static".to_string()),
    }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.message().to_string()))?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn remove_membership(
    State(_state): State<AppState>,
    Path((segment_id, contact_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    cli.remove_contact_from_segment(PairRequest {
        first: segment_id,
        second: contact_id,
    }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.message().to_string()))?;
    Ok(Json(serde_json::json!({"message": "Contact removed from segment"})))
}

pub async fn log_usage(
    State(_state): State<AppState>,
    Json(body): Json<LogUsageBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    cli.log_kb_usage(LogUsageRequest {
        conversation_id: body.conversation_id,
        contact_id: body.contact_id,
        segment_id: body.segment_id,
        kb_entry_id: body.kb_entry_id,
        query_text: body.query_text.unwrap_or_default(),
        relevance_score: body.relevance_score.unwrap_or(0.5),
        used_in_response: body.used_in_response.unwrap_or(false),
    }).await.map_err(grpc_err)?;
    Ok(Json(serde_json::json!({"message": "Usage logged"})))
}

pub async fn get_usage_logs(
    State(_state): State<AppState>,
    Path(segment_id): Path<String>,
    Query(params): Query<ListMembershipsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50);
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_segment_usage_logs(LimitedIdRequest {
        id: segment_id,
        limit,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}
