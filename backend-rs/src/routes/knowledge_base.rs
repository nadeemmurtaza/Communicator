//! Knowledge Base routes — thin gRPC client forwarding to svc-knowledge-base.

use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use newax_proto::knowledge_base::v1::knowledge_base_service_client::KnowledgeBaseServiceClient;
use newax_proto::knowledge_base::v1::{EmptyRequest, IdRequest, QueryRequest, CreateRequest, ApplyRequest};

async fn client(
    state: &AppState,
) -> Result<KnowledgeBaseServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.knowledge_base_grpc_url.is_empty() {
        "http://127.0.0.1:50117".to_string()
    } else {
        state.config.knowledge_base_grpc_url.clone()
    };
    KnowledgeBaseServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("knowledge_base gRPC connect failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "error": "knowledge base service unavailable" })))
        })
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub topic: Option<String>,
    pub status: Option<String>,
}

pub async fn list_knowledge_base(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Query(query): Query<ListQuery>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let mut qj = serde_json::json!({});
    if let Some(t) = &query.topic { qj["topic"] = serde_json::json!(t); }
    if let Some(s) = &query.status { qj["status"] = serde_json::json!(s); }
    let resp = cli.list_knowledge_base(QueryRequest { json: qj.to_string() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_knowledge_base_entry(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.create_knowledge_base_entry(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_knowledge_base_entry(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.delete_knowledge_base_entry(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct UploadQuery {
    pub topic: Option<String>,
}

pub async fn upload_knowledge_document(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    _query: Query<UploadQuery>,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.upload_knowledge_document(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn ingest_url(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.ingest_url(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct HubQuery {
    pub q: Option<String>,
    pub topic: Option<String>,
    pub touchpoint: Option<String>,
}

pub async fn get_knowledge_hub_overview(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Query(query): Query<HubQuery>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let mut qj = serde_json::json!({});
    if let Some(q) = &query.q { qj["q"] = serde_json::json!(q); }
    if let Some(t) = &query.topic { qj["topic"] = serde_json::json!(t); }
    if let Some(tp) = &query.touchpoint { qj["touchpoint"] = serde_json::json!(tp); }
    let resp = cli.get_knowledge_hub_overview(QueryRequest { json: qj.to_string() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct RecommendationsQuery {
    pub status: Option<String>,
    pub gap_type: Option<String>,
}

pub async fn list_knowledge_recommendations(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Query(query): Query<RecommendationsQuery>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let mut qj = serde_json::json!({});
    if let Some(s) = &query.status { qj["status"] = serde_json::json!(s); }
    if let Some(g) = &query.gap_type { qj["gapType"] = serde_json::json!(g); }
    let resp = cli.list_knowledge_recommendations(QueryRequest { json: qj.to_string() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ApplyRecommendationPayload {
    pub custom_content: Option<String>,
}

pub async fn apply_knowledge_recommendation(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<ApplyRecommendationPayload>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let body = serde_json::json!({ "customContent": payload.custom_content });
    let resp = cli.apply_knowledge_recommendation(ApplyRequest { id, json: body.to_string() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn dismiss_knowledge_recommendation(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.dismiss_knowledge_recommendation(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct GapsQuery {
    pub status: Option<String>,
}

pub async fn list_knowledge_gaps(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Query(query): Query<GapsQuery>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let mut qj = serde_json::json!({});
    if let Some(s) = &query.status { qj["status"] = serde_json::json!(s); }
    let resp = cli.list_knowledge_gaps(QueryRequest { json: qj.to_string() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn scan_knowledge_gaps(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.scan_knowledge_gaps(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ApproveGapPayload {
    pub custom_answer: Option<String>,
}

pub async fn approve_knowledge_gap(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<ApproveGapPayload>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let body = serde_json::json!({ "customAnswer": payload.custom_answer });
    let resp = cli.approve_knowledge_gap(ApplyRequest { id, json: body.to_string() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn dismiss_knowledge_gap(
    State(state): State<AppState>,
    Tenant(_tenant): Tenant,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.dismiss_knowledge_gap(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
