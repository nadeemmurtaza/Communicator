//! Live Proof routes — thin gRPC client forwarding to svc-live-proof.

use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use newax_proto::live_proof::v1::live_proof_service_client::LiveProofServiceClient;
use newax_proto::live_proof::v1::{
    CreateSessionRequest, GetSessionDetailsRequest, RecordDecisionRequest,
    UploadEvidenceRequest, GetEvidenceMediaRequest, GetCapturePageRequest,
};

// Re-exports for backward compatibility with reply_engine
pub use newax_live_proof::{NewLiveProofSession, CreatedLiveProofSession, DEFAULT_TTL_MINUTES, DEFAULT_MAX_VIDEO_SECONDS};

/// Adapter for reply_engine: builds a LiveProofState from AppState and calls the crate function.
pub async fn create_pending_session(
    state: &AppState,
    spec: &NewLiveProofSession,
) -> Result<CreatedLiveProofSession, String> {
    let lp_state = newax_live_proof::LiveProofState {
        db: state.db.clone(),
        public_url: state.config.public_url.clone(),
        live_proof_storage_dir: state.config.live_proof_storage_dir.clone(),
    };
    newax_live_proof::create_pending_session(&lp_state, spec).await
}

fn grpc_url(state: &AppState) -> String {
    if state.config.live_proof_grpc_url.is_empty() {
        "http://127.0.0.1:50118".to_string()
    } else {
        state.config.live_proof_grpc_url.clone()
    }
}

async fn client(
    state: &AppState,
) -> Result<LiveProofServiceClient<tonic::transport::Channel>, Response> {
    LiveProofServiceClient::connect(grpc_url(state))
        .await
        .map_err(|e| {
            tracing::error!("live_proof gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "detail": "live_proof service unavailable" })),
            )
                .into_response()
        })
}

fn parse_json_response(resp: String) -> Response {
    match serde_json::from_str::<Value>(&resp) {
        Ok(v) => Json(v).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "invalid response" }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct PageQuery {
    token: Option<String>,
}

// ---------------------------------------------------------------------------
// POST /live-proof/sessions
// ---------------------------------------------------------------------------

pub async fn create_session(
    State(state): State<AppState>,
    Tenant(header_tenant): Tenant,
    headers: HeaderMap,
    payload: Option<Json<Value>>,
) -> Response {
    let Some(body) = payload.map(|Json(v)| v) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "detail": "A JSON body is required to open a verification session." })),
        )
            .into_response();
    };

    // Add header_tenant for the gRPC service to use
    let mut body_map = body.as_object().cloned().unwrap_or_default();
    body_map.insert("header_tenant".to_string(), json!(header_tenant));
    let body_with_tenant = Value::Object(body_map);

    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let resp = match cli
        .create_session(CreateSessionRequest {
            json: body_with_tenant.to_string(),
        })
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": e.to_string() })),
            )
                .into_response();
        }
    };
    parse_json_response(resp.json)
}

// ---------------------------------------------------------------------------
// GET /live-proof/sessions/:id
// ---------------------------------------------------------------------------

pub async fn get_session_details(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<TokenQuery>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let resp = match cli
        .get_session_details(GetSessionDetailsRequest {
            session_id,
            token,
        })
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": e.to_string() })),
            )
                .into_response();
        }
    };
    parse_json_response(resp.json)
}

// ---------------------------------------------------------------------------
// POST /live-proof/sessions/:id/decision
// ---------------------------------------------------------------------------

pub async fn record_decision(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<TokenQuery>,
    Json(body): Json<Value>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let resp = match cli
        .record_decision(RecordDecisionRequest {
            session_id,
            token,
            json: body.to_string(),
        })
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": e.to_string() })),
            )
                .into_response();
        }
    };
    parse_json_response(resp.json)
}

// ---------------------------------------------------------------------------
// POST /live-proof/sessions/:id/evidence
// ---------------------------------------------------------------------------

pub async fn upload_evidence(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<TokenQuery>,
    Json(body): Json<Value>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let resp = match cli
        .upload_evidence(UploadEvidenceRequest {
            session_id,
            token,
            json: body.to_string(),
        })
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": e.to_string() })),
            )
                .into_response();
        }
    };
    parse_json_response(resp.json)
}

// ---------------------------------------------------------------------------
// GET /live-proof/sessions/:session_id/evidence/:evidence_id/media
// ---------------------------------------------------------------------------

pub async fn evidence_media(
    State(state): State<AppState>,
    Path((session_id, evidence_id)): Path<(String, String)>,
    Query(params): Query<TokenQuery>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let resp = match cli
        .get_evidence_media(GetEvidenceMediaRequest {
            session_id,
            evidence_id,
            token,
        })
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": e.to_string() })),
            )
                .into_response();
        }
    };
    parse_json_response(resp.json)
}

// ---------------------------------------------------------------------------
// GET /verify/proof/:id (capture page)
// ---------------------------------------------------------------------------

pub async fn capture_page(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<PageQuery>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let resp = match cli
        .get_capture_page(GetCapturePageRequest {
            session_id,
            token,
        })
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": e.to_string() })),
            )
                .into_response();
        }
    };
    parse_json_response(resp.json)
}
