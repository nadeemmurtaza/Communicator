//! Team Routing API routes — thin gRPC client forwarding to svc-team-routing.

use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use newax_proto::team_routing::v1::team_routing_service_client::TeamRoutingServiceClient;
use newax_proto::team_routing::v1::{
    EmptyRequest, IdRequest, CreateRequest, AssignRequest, TakeoverRequest,
    AddMemberRequest, RemoveMemberRequest, StatusRequest, CapacityRequest, QueryRequest,
};

#[derive(Deserialize)]
pub struct AssignQuery {
    pub member_id: Option<String>,
}

#[derive(Deserialize)]
pub struct TakeoverQuery {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct CapacityQuery {
    pub team_id: Option<String>,
}

async fn client(
    state: &AppState,
) -> Result<TeamRoutingServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.team_routing_grpc_url.is_empty() {
        "http://127.0.0.1:50116".to_string()
    } else {
        state.config.team_routing_grpc_url.clone()
    };
    TeamRoutingServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("team_routing gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "team_routing service unavailable" })),
            )
        })
}

// ---------------------------------------------------------------------------
// /team -- individual team members
// ---------------------------------------------------------------------------

pub async fn list_team(State(state): State<AppState>) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_team(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_team_member(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.create_team_member(CreateRequest { json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_team_member(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.delete_team_member(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Conversation assignment
// ---------------------------------------------------------------------------

pub async fn assign_conversation_team(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<AssignQuery>,
) -> Response {
    let member_id = query.member_id.clone().unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.assign_conversation(AssignRequest { conversation_id, member_id }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn set_human_takeover(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<TakeoverQuery>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.set_human_takeover(TakeoverRequest { conversation_id, enabled: query.enabled }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// /teams -- queues with real membership
// ---------------------------------------------------------------------------

pub async fn list_teams(State(state): State<AppState>) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_teams(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_team(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.create_team(CreateRequest { json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::CREATED, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_team(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.delete_team(IdRequest { id }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn add_member_to_team(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.add_member_to_team(AddMemberRequest { team_id, json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn remove_member_from_team(
    State(state): State<AppState>,
    Path((team_id, member_id)): Path<(String, String)>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.remove_member_from_team(RemoveMemberRequest { team_id, member_id }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Routing & capacity
// ---------------------------------------------------------------------------

pub async fn list_routing_agents(State(state): State<AppState>) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.list_routing_agents(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_agent_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.update_agent_status(StatusRequest { id, json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_agent_capacity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.update_agent_capacity(CapacityRequest { id, json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_capacity_overview(
    State(state): State<AppState>,
    Query(query): Query<CapacityQuery>,
) -> Response {
    let team_id = query.team_id.clone().unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.get_capacity_overview(QueryRequest { team_id }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn batch_auto_assign(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.batch_auto_assign(CreateRequest { json: json_str }).await;
    match resp {
        Ok(r) => {
            let val: Value = serde_json::from_str(&r.into_inner().json).unwrap_or(Value::Null);
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
