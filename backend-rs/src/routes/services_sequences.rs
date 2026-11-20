//! Services & Sequences — thin gRPC client forwarding to svc-services-sequences (port 50107).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::db::AppState;

use newax_proto::services_sequences::v1::services_sequences_service_client::ServicesSequencesServiceClient;
use newax_proto::services_sequences::v1::{
    Empty, IdRequest, ListServicesRequest, CreateServiceRequest, CreateSequenceRequest, JsonBody,
};

async fn client(state: &AppState) -> Result<ServicesSequencesServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = if state.config.services_sequences_grpc_url.is_empty() {
        "http://127.0.0.1:50107".to_string()
    } else {
        state.config.services_sequences_grpc_url.clone()
    };
    ServicesSequencesServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("gRPC connection failed: {}", e)))
}

fn parse_json_resp(resp: tonic::Response<newax_proto::services_sequences::v1::JsonResponse>) -> Value {
    let json_str = resp.into_inner().json;
    serde_json::from_str(&json_str).unwrap_or_else(|_| serde_json::json!({"raw": json_str}))
}

use super::auth::Tenant;

pub async fn list_services(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
) -> Response {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let req = ListServicesRequest { tenant_key: tenant };
    match c.list_services(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::OK, Json(parse_json_resp(resp))).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let req = IdRequest { id };
    match c.get_service(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::OK, Json(parse_json_resp(resp))).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn create_service(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Json(payload): Json<Value>,
) -> Response {
    let json_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    };
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let req = CreateServiceRequest {
        body: Some(JsonBody { json: json_str }),
        tenant_key: tenant,
    };
    match c.create_service(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::CREATED, Json(parse_json_resp(resp))).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn list_sequences(State(state): State<AppState>) -> Response {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let req = Empty {};
    match c.list_sequences(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::OK, Json(parse_json_resp(resp))).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn create_sequence(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let json_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    };
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let req = CreateSequenceRequest {
        body: Some(JsonBody { json: json_str }),
    };
    match c.create_sequence(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::CREATED, Json(parse_json_resp(resp))).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn sequence_variant_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let req = IdRequest { id };
    match c.sequence_variant_stats(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::OK, Json(parse_json_resp(resp))).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
