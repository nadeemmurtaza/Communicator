//! SSO routes — thin gRPC client forwarding to svc-sso.

use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;
use newax_proto::sso::v1::sso_service_client::SsoServiceClient;

async fn client(
    state: &AppState,
) -> Result<SsoServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.sso_grpc_url.is_empty() {
        "http://127.0.0.1:50109".to_string()
    } else {
        state.config.sso_grpc_url.clone()
    };
    SsoServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("sso gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "sso service unavailable" })),
            )
        })
}

fn headers_to_json(headers: &HeaderMap) -> String {
    let map: HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| {
            let k_str = k.to_string();
            v.to_str().ok().map(|v| (k_str, v.to_string()))
        })
        .collect();
    serde_json::to_string(&map).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LoginQuery {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    relay_state: Option<String>,
}

#[derive(Deserialize)]
pub struct CheckEmailPayload {
    #[serde(default)]
    email: Option<String>,
}

// ---------------------------------------------------------------------------
// Get Config
// ---------------------------------------------------------------------------

pub async fn get_config(State(state): State<AppState>) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let resp = cli.get_config(newax_proto::sso::v1::EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            Json(val).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Update Config
// ---------------------------------------------------------------------------

pub async fn update_config(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let resp = cli.update_config(newax_proto::sso::v1::UpdateConfigRequest {
        body: Some(newax_proto::sso::v1::JsonBody {
            json: payload.to_string(),
        }),
        headers_json: String::new(),
    }).await;

    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            Json(val).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// SP Metadata
// ---------------------------------------------------------------------------

pub async fn sp_metadata(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let resp = cli.sp_metadata(newax_proto::sso::v1::SpMetadataRequest {
        headers_json: headers_to_json(&headers),
    }).await;

    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            let xml = val.get("xml").and_then(|v| v.as_str()).unwrap_or("");
            (
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                xml.to_string(),
            ).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Initiate Login
// ---------------------------------------------------------------------------

pub async fn initiate_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<LoginQuery>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let resp = cli.initiate_login(newax_proto::sso::v1::InitiateLoginRequest {
        headers_json: headers_to_json(&headers),
        email: params.email.unwrap_or_default(),
        relay_state: params.relay_state.unwrap_or_default(),
    }).await;

    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            Json(val).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Assertion Consumer Service
// ---------------------------------------------------------------------------

pub async fn assertion_consumer_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let content_type = headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or_default();
    let _ = content_type; // used in service

    let resp = cli.assertion_consumer_service(newax_proto::sso::v1::AcsRequest {
        headers_json: headers_to_json(&headers),
        body: body.to_vec(),
    }).await;

    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            let status = if val.get("status").and_then(|v| v.as_str()) == Some("unsupported") {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            };
            (status, Json(val)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Test Connection
// ---------------------------------------------------------------------------

pub async fn test_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let resp = cli.test_connection(newax_proto::sso::v1::TestConnectionRequest {
        headers_json: headers_to_json(&headers),
        body: Some(newax_proto::sso::v1::JsonBody {
            json: payload.to_string(),
        }),
    }).await;

    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            Json(val).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Check Email
// ---------------------------------------------------------------------------

pub async fn check_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CheckEmailPayload>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };

    let email = payload.email.clone().unwrap_or_default();
    let resp = cli.check_email(newax_proto::sso::v1::CheckEmailRequest {
        headers_json: headers_to_json(&headers),
        email,
    }).await;

    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or_default();
            Json(val).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
