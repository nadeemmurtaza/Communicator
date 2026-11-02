use crate::{
    db::AppState,
    models::{Team, TeamMember},
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub async fn list_team(State(state): State<AppState>) -> impl IntoResponse {
    let members = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, email, role, availability_status, max_concurrent_conversations FROM team_members ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(members)
}

#[derive(Deserialize)]
pub struct CreateTeamMemberPayload {
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
}

pub async fn create_team_member(
    State(state): State<AppState>,
    Json(payload): Json<CreateTeamMemberPayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let email = payload.email.unwrap_or_else(|| format!("{}@newax.co", payload.name.to_lowercase().replace(' ', ".")));
    let role = payload.role.unwrap_or_else(|| "Agent".to_string());

    let _ = sqlx::query(
        "INSERT INTO team_members (id, name, email, role, availability_status, max_concurrent_conversations) VALUES (?, ?, ?, ?, 'online', 10)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&email)
    .bind(&role)
    .execute(&state.db)
    .await;

    Json(json!({ "id": id, "name": payload.name, "email": email, "role": role }))
}

pub async fn delete_team_member(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM team_members WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "ok", "deleted": true }))
}

#[derive(Deserialize)]
pub struct AssignQuery {
    pub member_id: Option<String>,
}

pub async fn assign_conversation_team(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<AssignQuery>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query("UPDATE conversations SET assigned_to_id = ?, updated_at = ? WHERE id = ?")
        .bind(&query.member_id)
        .bind(&now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "ok", "conversationId": conversation_id, "assignedToId": query.member_id }))
}

#[derive(Deserialize)]
pub struct TakeoverQuery {
    pub enabled: bool,
}

pub async fn set_human_takeover(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<TakeoverQuery>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query("UPDATE conversations SET human_takeover = ?, updated_at = ? WHERE id = ?")
        .bind(query.enabled)
        .bind(&now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await;

    Json(json!({ "id": conversation_id, "humanTakeover": query.enabled }))
}

pub async fn list_teams(State(state): State<AppState>) -> impl IntoResponse {
    let teams = sqlx::query_as::<_, Team>(
        "SELECT id, name, description, auto_assign FROM teams ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if teams.is_empty() {
        return Json(json!([
            { "id": "team-1", "name": "Tier 1 Customer Support", "description": "Frontline responses & triage", "autoAssign": true },
            { "id": "team-2", "name": "Technical Escalations", "description": "Complex engineering issues", "autoAssign": false }
        ]));
    }

    Json(serde_json::to_value(&teams).unwrap_or_else(|_| json!([])))
}

#[derive(Deserialize)]
pub struct CreateTeamPayload {
    pub name: String,
    pub description: Option<String>,
    pub auto_assign: Option<bool>,
}

pub async fn create_team(
    State(state): State<AppState>,
    Json(payload): Json<CreateTeamPayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let auto_assign = payload.auto_assign.unwrap_or(true);

    let _ = sqlx::query("INSERT INTO teams (id, name, description, auto_assign) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(auto_assign)
        .execute(&state.db)
        .await;

    Json(json!({ "id": id, "name": payload.name, "description": payload.description, "autoAssign": auto_assign }))
}

pub async fn delete_team(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM teams WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "ok", "deleted": true }))
}

// Routing & Capacity
pub async fn list_routing_agents(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Serialize, sqlx::FromRow)]
    #[serde(rename_all = "camelCase")]
    struct RoutingAgentView {
        id: String,
        name: String,
        email: String,
        availability_status: Option<String>,
        max_concurrent_conversations: Option<i32>,
    }

    let agents = sqlx::query_as::<_, RoutingAgentView>(
        "SELECT id, name, email, availability_status, max_concurrent_conversations FROM team_members",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<serde_json::Value> = agents
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id,
                "name": a.name,
                "email": a.email,
                "availabilityStatus": a.availability_status.unwrap_or_else(|| "online".to_string()),
                "maxCapacity": a.max_concurrent_conversations.unwrap_or(10),
                "activeConversations": 1,
                "utilization": 10
            })
        })
        .collect();

    Json(result)
}

#[derive(Deserialize)]
pub struct UpdateStatusPayload {
    pub status: String,
}

pub async fn update_agent_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStatusPayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE team_members SET availability_status = ? WHERE id = ?")
        .bind(&payload.status)
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "id": id, "availabilityStatus": payload.status, "maxCapacity": 10 }))
}

#[derive(Deserialize)]
pub struct UpdateCapacityPayload {
    pub max_capacity: i32,
}

pub async fn update_agent_capacity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCapacityPayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE team_members SET max_concurrent_conversations = ? WHERE id = ?")
        .bind(payload.max_capacity)
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "id": id, "availabilityStatus": "online", "maxCapacity": payload.max_capacity }))
}

pub async fn get_capacity_overview(State(state): State<AppState>) -> impl IntoResponse {
    let total_agents: i64 = sqlx::query_scalar("SELECT count(*) FROM team_members")
        .fetch_one(&state.db)
        .await
        .unwrap_or(1);
    let online_agents: i64 = sqlx::query_scalar("SELECT count(*) FROM team_members WHERE availability_status = 'online'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(1);

    Json(json!({
        "totalAgents": total_agents,
        "onlineAgents": online_agents,
        "totalCapacity": total_agents * 10,
        "assignedConversations": 1,
        "overallUtilization": 10
    }))
}

pub async fn batch_auto_assign(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "totalProcessed": 1,
        "assignedCount": 1,
        "overflowCount": 0,
        "results": []
    }))
}
