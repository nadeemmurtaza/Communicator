//! CSAT Surveys API routes — thin gRPC client forwarding to svc-csat-surveys.

use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use newax_proto::csat_surveys::v1::csat_surveys_service_client::CsatSurveysServiceClient;
use newax_proto::csat_surveys::v1::{
    EmptyRequest, IdRequest, QueryRequest, CreateRequest, UpdateRequest, RespondRequest,
};

#[derive(Deserialize)]
pub struct CSATQueryParams {
    pub status: Option<String>,
    pub channel: Option<String>,
    pub template_id: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

async fn client(
    state: &AppState,
) -> Result<CsatSurveysServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.csat_surveys_grpc_url.is_empty() {
        "http://127.0.0.1:50112".to_string()
    } else {
        state.config.csat_surveys_grpc_url.clone()
    };
    CsatSurveysServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("csat_surveys gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "csat_surveys service unavailable" })),
            )
        })
}

pub async fn list_templates(
    State(state): State<AppState>,
    Query(_params): Query<CSATQueryParams>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_templates(QueryRequest { json: String::new() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.get_template(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_template(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.create_template(CreateRequest { json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.update_template(UpdateRequest { id, json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.delete_template(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_surveys(
    State(state): State<AppState>,
    Query(_params): Query<CSATQueryParams>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_surveys(QueryRequest { json: String::new() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_survey(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.create_survey(CreateRequest { json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn respond_to_survey(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.respond_to_survey(RespondRequest { id, json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn send_survey(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.send_survey(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_csat_summary(
    State(state): State<AppState>,
    Query(_params): Query<CSATQueryParams>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.get_csat_summary(QueryRequest { json: String::new() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn generate_snapshot(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.generate_snapshot(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_snapshots(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_snapshots(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}