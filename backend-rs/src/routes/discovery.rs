//! Discovery-flow routes — thin gRPC client forwarding to svc-discovery.

use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use newax_proto::discovery::v1::discovery_service_client::DiscoveryServiceClient;
use newax_proto::discovery::v1::{EmptyRequest, IdRequest, QueryRequest};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct InquiryQuery {
    #[serde(alias = "tenantKey", alias = "tenant_key")]
    pub tenant_key: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CampaignQuery {
    #[serde(alias = "tenantKey")]
    pub tenant_key: Option<String>,
    pub channel: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SyncQuery {
    #[serde(alias = "tenantKey")]
    pub tenant_key: Option<String>,
}

async fn client(
    state: &AppState,
) -> Result<DiscoveryServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.discovery_grpc_url.is_empty() {
        "http://127.0.0.1:50113".to_string()
    } else {
        state.config.discovery_grpc_url.clone()
    };
    DiscoveryServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("discovery gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "discovery service unavailable" })),
            )
        })
}

pub async fn list_discovery_templates(
    State(state): State<AppState>,
    Tenant(_tenant_key): Tenant,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_discovery_templates(QueryRequest { json: String::new() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_inquiries(
    State(state): State<AppState>,
    Tenant(_tenant_key): Tenant,
    Query(q): Query<InquiryQuery>,
) -> Response {
    let json = serde_json::to_string(&q).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json_resp)) => return (status, json_resp).into_response(),
    };
    let resp = cli.list_inquiries(QueryRequest { json }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_inquiry(
    State(state): State<AppState>,
    Tenant(_tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.get_inquiry(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_campaign_templates(
    State(state): State<AppState>,
    Tenant(_tenant_key): Tenant,
    Query(q): Query<CampaignQuery>,
) -> Response {
    let json = serde_json::to_string(&q).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json_resp)) => return (status, json_resp).into_response(),
    };
    let resp = cli.list_campaign_templates(QueryRequest { json }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn sync_campaign_template_status(
    State(state): State<AppState>,
    Tenant(_tenant_key): Tenant,
    Query(q): Query<SyncQuery>,
) -> Response {
    let json = serde_json::to_string(&q).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json_resp)) => return (status, json_resp).into_response(),
    };
    let resp = cli.sync_campaign_template_status(QueryRequest { json }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
