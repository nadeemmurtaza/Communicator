//! Team Routing Service
//!
//! Manages team members, teams, conversation assignment, agent status/capacity,
//! and batch auto-assignment with capacity-based or round-robin strategies.

use sqlx::postgres::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct TeamRoutingState {
    pub db: PgPool,
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn refuse(status: http::StatusCode, message: String, extra: Value) -> (http::StatusCode, Value) {
    let mut body = json!({ "error": message, "detail": message });
    if let (Some(target), Some(source)) = (body.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    (status, body)
}

fn db_error_tuple(context: &str, err: sqlx::Error) -> (http::StatusCode, Value) {
    refuse(
        http::StatusCode::INTERNAL_SERVER_ERROR,
        format!("{context}: {err}"),
        json!({ "databaseError": true }),
    )
}

fn write_not_found(table: &str, id: &str, rows: u64) -> (http::StatusCode, Value) {
    refuse(
        http::StatusCode::NOT_FOUND,
        format!(
            "no {table} row with id '{id}' exists (the statement affected {rows} rows), so \
             nothing was changed."
        ),
        json!({ "id": id, "rowsAffected": rows }),
    )
}

async fn table_exists(db: &sqlx::PgPool, table: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema='public' AND table_name=$1",
    )
    .bind(table)
    .fetch_one(db)
    .await?;
    Ok(count > 0)
}

async fn require_table(db: &sqlx::PgPool, table: &str) -> Result<(), (http::StatusCode, Value)> {
    match table_exists(db, table).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            http::StatusCode::SERVICE_UNAVAILABLE,
            json!({
                "detail": format!(
                    "Schema table '{table}' is not present in this database. Apply the team DDL and restart -- refusing to report invented team data."
                )
            }),
        )),
        Err(e) => Err(db_error_tuple(
            &format!("the schema check for table '{table}' could not be read"),
            e,
        )),
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct MemberRow {
    id: String,
    name: String,
    email: Option<String>,
    role: Option<String>,
    availability_status: Option<String>,
    max_concurrent_conversations: Option<i32>,
    last_assigned_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct LoadRow {
    agent_id: Option<String>,
    active: i64,
}

async fn active_load(
    db: &sqlx::PgPool,
) -> Result<HashMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query_as::<_, LoadRow>(
        r#"
        SELECT assigned_to_id AS agent_id, COUNT(*) AS active
        FROM conversations
        WHERE assigned_to_id IS NOT NULL
          AND LOWER(state) NOT IN ('resolved', 'closed', 'closed_won', 'closed_lost')
        GROUP BY assigned_to_id
        "#,
    )
    .fetch_all(db)
    .await?;

    let mut map = HashMap::new();
    for r in rows {
        if let Some(id) = r.agent_id {
            map.insert(id, r.active);
        }
    }
    Ok(map)
}

const MEMBER_COLUMNS: &str = "id, name, email, role, availability_status, max_concurrent_conversations, last_assigned_at";

async fn load_member(db: &sqlx::PgPool, id: &str) -> Result<Option<MemberRow>, sqlx::Error> {
    sqlx::query_as::<_, MemberRow>(&format!(
        "SELECT {MEMBER_COLUMNS} FROM team_members WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

fn member_json(m: &MemberRow, active: i64) -> Value {
    let capacity = m.max_concurrent_conversations.unwrap_or(0) as i64;
    json!({
        "id": m.id,
        "name": m.name,
        "email": m.email,
        "role": m.role,
        "availabilityStatus": m.availability_status.clone().unwrap_or_else(|| "offline".to_string()),
        "maxCapacity": capacity,
        "activeConversations": active,
        "availableSlots": (capacity - active).max(0),
        "utilization": if capacity > 0 {
            ((active as f64 / capacity as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        },
        "lastAssignedAt": m.last_assigned_at,
    })
}

// ---------------------------------------------------------------------------
// /team -- individual team members
// ---------------------------------------------------------------------------

pub async fn list_team(state: &TeamRoutingState) -> Result<Value, String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let members = sqlx::query_as::<_, MemberRow>(&format!(
        "SELECT {MEMBER_COLUMNS} FROM team_members ORDER BY name ASC"
    ))
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Database error: {e}"))?;

    let load = active_load(&state.db).await
        .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;
    let out: Vec<Value> = members
        .iter()
        .map(|m| member_json(m, *load.get(&m.id).unwrap_or(&0)))
        .collect();

    Ok(json!(out))
}

#[derive(Deserialize)]
struct CreateTeamMemberPayload {
    name: String,
    email: Option<String>,
    role: Option<String>,
    max_capacity: Option<i32>,
}

pub async fn create_team_member(
    state: &TeamRoutingState,
    body: &Value,
) -> Result<(u16, Value), String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let payload: CreateTeamMemberPayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err("Member name cannot be empty".to_string());
    }
    let capacity = payload.max_capacity.unwrap_or(5);
    let capacity_supplied = payload.max_capacity.is_some();
    if capacity <= 0 || capacity > 200 {
        return Err("max_capacity must be between 1 and 200".to_string());
    }

    let email = match payload
        .email
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
    {
        Some(e) => {
            if !e.contains('@') {
                return Err(format!("Invalid email: {e}"));
            }
            e.to_string()
        }
        None => String::new(),
    };

    let dup: Option<String> = sqlx::query_scalar(
        "SELECT id FROM team_members WHERE email = $1 AND email <> '' LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("the duplicate-email check for '{email}' could not be run: {e}"))?;
    if dup.is_some() {
        return Err(format!("A team member with email {email} already exists"));
    }

    let id = Uuid::new_v4().to_string();
    let role = payload
        .role
        .filter(|r| !r.trim().is_empty())
        .unwrap_or_else(|| "Agent".to_string());

    let res = sqlx::query(
        "INSERT INTO team_members (id, name, email, role, availability_status, max_concurrent_conversations) VALUES ($1, $2, $3, $4, 'offline', $5)",
    )
    .bind(&id)
    .bind(name)
    .bind(&email)
    .bind(&role)
    .bind(capacity)
    .execute(&state.db)
    .await;

    match res {
        Ok(report) if report.rows_affected() == 1 => match load_member(&state.db, &id).await {
            Ok(Some(m)) => {
                let mut body = member_json(&m, 0);
                if !capacity_supplied {
                    body["warnings"] = json!([format!(
                        "max_capacity was not supplied, so the team_members column default of {} \
                         was stored instead of a capacity this endpoint invented",
                        capacity
                    )]);
                }
                Ok((201, body))
            }
            Ok(None) => Err(format!(
                "INSERT INTO team_members affected 1 row, but member '{}' could not be read back",
                id
            )),
            Err(e) => Err(format!("the newly inserted team member '{}' could not be read back: {e}", id)),
        },
        Ok(report) => Err(format!(
            "INSERT INTO team_members affected {} rows instead of 1",
            report.rows_affected()
        )),
        Err(e) => Err(format!("the team member '{}' could not be inserted: {e}", id)),
    }
}

pub async fn delete_team_member(
    state: &TeamRoutingState,
    id: &str,
) -> Result<Value, String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    match sqlx::query("DELETE FROM team_members WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => Err(format!("Team member {id} not found")),
        Ok(deleted) => {
            let now = Utc::now().to_rfc3339();
            let unassigned = sqlx::query(
                "UPDATE conversations SET assigned_to_id = NULL, updated_at = $1 WHERE assigned_to_id = $2",
            )
            .bind(&now)
            .bind(id)
            .execute(&state.db)
            .await
            .map(|report| report.rows_affected())
            .map_err(|e| format!("clearing assignee failed: {e}"))?;

            let memberships_removed = sqlx::query("DELETE FROM team_memberships WHERE team_member_id = $1")
                .bind(id)
                .execute(&state.db)
                .await
                .map(|report| report.rows_affected())
                .map_err(|e| format!("deleting memberships failed: {e}"))?;

            Ok(json!({
                "status": "deleted",
                "id": id,
                "memberRowsDeleted": deleted.rows_affected(),
                "conversationsUnassigned": unassigned,
                "membershipsRemoved": memberships_removed,
                "membershipsNote": if memberships_removed == 0 {
                    "the agent was not a member of any team, so no membership row existed to delete"
                } else {
                    ""
                },
            }))
        }
        Err(e) => Err(format!("Database error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// Conversation assignment
// ---------------------------------------------------------------------------

pub async fn assign_conversation(
    state: &TeamRoutingState,
    conversation_id: &str,
    member_id: Option<&str>,
) -> Result<Value, String> {
    let exists = sqlx::query_scalar::<_, String>("SELECT id FROM conversations WHERE id = $1 LIMIT 1")
        .bind(conversation_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("the existence of conversation '{}' could not be checked: {e}", conversation_id))?;
    if exists.is_none() {
        return Err(format!("Conversation {conversation_id} not found"));
    }

    if let Some(member_id) = member_id.filter(|m| !m.is_empty()) {
        require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;
        let member = load_member(&state.db, member_id).await
            .map_err(|e| format!("the team member '{}' could not be read: {e}", member_id))?
            .ok_or_else(|| format!("Team member {member_id} not found"))?;

        let now = Utc::now().to_rfc3339();
        let assigned = sqlx::query(
            "UPDATE conversations SET assigned_to_id = $1, assigned_to_name = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&member.id)
        .bind(&member.name)
        .bind(&now)
        .bind(conversation_id)
        .execute(&state.db)
        .await;

        match assigned {
            Ok(report) if report.rows_affected() == 1 => {}
            Ok(report) => return Err(format!(
                "no conversations row with id '{}' exists (affected {} rows)",
                conversation_id, report.rows_affected()
            )),
            Err(e) => return Err(format!("conversation '{}' could not be assigned: {e}", conversation_id)),
        }

        // Auxiliary bookkeeping
        let _ = sqlx::query("UPDATE team_members SET last_assigned_at = $1 WHERE id = $2")
            .bind(&now)
            .bind(&member.id)
            .execute(&state.db)
            .await;

        let load = active_load(&state.db).await
            .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;
        Ok(json!({
            "id": conversation_id,
            "assignedToId": member.id,
            "assignedToName": member.name,
            "activeConversations": *load.get(&member.id).unwrap_or(&0),
        }))
    } else {
        let now = Utc::now().to_rfc3339();
        let unassigned = sqlx::query(
            "UPDATE conversations SET assigned_to_id = NULL, assigned_to_name = NULL, updated_at = $1 WHERE id = $2",
        )
        .bind(&now)
        .bind(conversation_id)
        .execute(&state.db)
        .await;

        match unassigned {
            Ok(report) if report.rows_affected() == 1 => Ok(json!({
                "id": conversation_id,
                "assignedToId": null,
                "assignedToName": null,
            })),
            Ok(report) => Err(format!(
                "no conversations row with id '{}' exists (affected {} rows)",
                conversation_id, report.rows_affected()
            )),
            Err(e) => Err(format!("conversation '{}' could not be unassigned: {e}", conversation_id)),
        }
    }
}

pub async fn set_human_takeover(
    state: &TeamRoutingState,
    conversation_id: &str,
    enabled: bool,
) -> Result<Value, String> {
    let now = Utc::now().to_rfc3339();
    match sqlx::query("UPDATE conversations SET human_takeover = $1, updated_at = $2 WHERE id = $3")
        .bind(enabled)
        .bind(&now)
        .bind(conversation_id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => Err(format!("Conversation {conversation_id} not found")),
        Ok(_) => Ok(json!({
            "id": conversation_id,
            "humanTakeover": enabled,
            "updatedAt": now,
        })),
        Err(e) => Err(format!("Database error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// /teams -- queues with real membership
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
struct TeamRow {
    id: String,
    name: String,
    description: Option<String>,
    auto_assign: Option<bool>,
    created_at: Option<String>,
}

const TEAM_COLUMNS: &str = "id, name, description, auto_assign, created_at";

async fn team_members_of(
    db: &sqlx::PgPool,
    team_id: &str,
) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT tm.id, tm.name, COALESCE(tm.email, '') AS email
        FROM team_members tm
        JOIN team_memberships tms ON tms.team_member_id = tm.id
        WHERE tms.team_id = $1
        ORDER BY tm.name ASC
        "#,
    )
    .bind(team_id)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            Ok((
                r.try_get::<String, _>("id")?,
                r.try_get::<String, _>("name")?,
                r.try_get::<String, _>("email")?,
            ))
        })
        .collect::<Result<Vec<(String, String, String)>, sqlx::Error>>()?)
}

async fn serialize_team(
    db: &sqlx::PgPool,
    t: &TeamRow,
) -> Result<Value, sqlx::Error> {
    let members = team_members_of(db, &t.id).await?;
    Ok(json!({
        "id": t.id,
        "name": t.name,
        "description": t.description.clone().unwrap_or_default(),
        "autoAssign": t.auto_assign.unwrap_or(false),
        "membersCount": members.len(),
        "members": members
            .iter()
            .map(|(id, name, email)| json!({ "id": id, "name": name, "email": email }))
            .collect::<Vec<_>>(),
        "createdAt": t.created_at,
    }))
}

pub async fn list_teams(state: &TeamRoutingState) -> Result<Value, String> {
    require_table(&state.db, "teams").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;
    require_table(&state.db, "team_memberships").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let teams = sqlx::query_as::<_, TeamRow>(&format!(
        "SELECT {TEAM_COLUMNS} FROM teams ORDER BY name ASC"
    ))
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Database error: {e}"))?;

    let mut out = Vec::with_capacity(teams.len());
    for t in &teams {
        match serialize_team(&state.db, t).await {
            Ok(v) => out.push(v),
            Err(e) => return Err(format!("the members of team '{}' could not be read: {e}", t.name)),
        }
    }
    Ok(json!(out))
}

#[derive(Deserialize)]
struct CreateTeamPayload {
    name: String,
    description: Option<String>,
    auto_assign: Option<bool>,
}

pub async fn create_team(
    state: &TeamRoutingState,
    body: &Value,
) -> Result<(u16, Value), String> {
    require_table(&state.db, "teams").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let payload: CreateTeamPayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err("Team name cannot be empty".to_string());
    }

    let dup: Option<String> = sqlx::query_scalar("SELECT id FROM teams WHERE name = $1 LIMIT 1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("the duplicate-name check for team '{}' could not be run: {e}", name))?;
    if dup.is_some() {
        return Err(format!("Team '{}' already exists", name));
    }

    let auto_assign = match payload.auto_assign {
        Some(value) => value,
        None => return Err(format!(
            "auto_assign is required for team '{}': teams.auto_assign is NOT NULL and has no default",
            name
        )),
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let description = payload.description.clone().unwrap_or_default();

    let stored = sqlx::query(
        "INSERT INTO teams (id, name, description, auto_assign, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(name)
    .bind(&description)
    .bind(auto_assign)
    .bind(&now)
    .execute(&state.db)
    .await;

    match stored {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => return Err(format!(
            "INSERT INTO teams affected {} rows (expected 1)",
            report.rows_affected()
        )),
        Err(e) => return Err(format!("team '{}' could not be stored: {e}", name)),
    }

    let row = sqlx::query_as::<_, TeamRow>(&format!(
        "SELECT {TEAM_COLUMNS} FROM teams WHERE id = $1"
    ))
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("the new team '{}' could not be read back: {e}", id))?;

    match row {
        Some(t) => match serialize_team(&state.db, &t).await {
            Ok(v) => Ok((201, v)),
            Err(e) => Err(format!("the members of the new team '{}' could not be read: {e}", id)),
        },
        None => Err(format!("Team was inserted but no teams row with id '{}' exists", id)),
    }
}

pub async fn delete_team(
    state: &TeamRoutingState,
    id: &str,
) -> Result<Value, String> {
    require_table(&state.db, "teams").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    match sqlx::query("DELETE FROM teams WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => Err(format!("Team {id} not found")),
        Ok(deleted) => {
            let memberships_removed = sqlx::query("DELETE FROM team_memberships WHERE team_id = $1")
                .bind(id)
                .execute(&state.db)
                .await
                .map(|report| report.rows_affected())
                .map_err(|e| format!("deleting memberships failed: {e}"))?;

            Ok(json!({
                "status": "deleted",
                "id": id,
                "teamRowsDeleted": deleted.rows_affected(),
                "membershipsRemoved": memberships_removed,
                "membershipsNote": if memberships_removed == 0 {
                    "the team had no membership rows, so none were deleted"
                } else {
                    ""
                },
            }))
        }
        Err(e) => Err(format!("Database error: {e}")),
    }
}

#[derive(Deserialize)]
struct AddTeamMemberPayload {
    member_id: String,
}

pub async fn add_member_to_team(
    state: &TeamRoutingState,
    team_id: &str,
    body: &Value,
) -> Result<Value, String> {
    for table in ["teams", "team_members", "team_memberships"] {
        require_table(&state.db, table).await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;
    }

    let team = sqlx::query_as::<_, TeamRow>(&format!("SELECT {TEAM_COLUMNS} FROM teams WHERE id = $1"))
        .bind(team_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?
        .ok_or_else(|| format!("Team {team_id} not found"))?;

    let payload: AddTeamMemberPayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    let member = load_member(&state.db, &payload.member_id).await
        .map_err(|e| format!("Database error: {e}"))?
        .ok_or_else(|| format!("Team member {} not found", payload.member_id))?;

    let already: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM team_memberships WHERE team_id = $1 AND team_member_id = $2 LIMIT 1",
    )
    .bind(team_id)
    .bind(&member.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("the existing membership could not be checked: {e}"))?;

    if already.is_none() {
        let inserted = sqlx::query(
            "INSERT INTO team_memberships (team_id, team_member_id) VALUES ($1, $2)",
        )
        .bind(team_id)
        .bind(&member.id)
        .execute(&state.db)
        .await;

        match inserted {
            Ok(report) if report.rows_affected() == 1 => {}
            Ok(report) => return Err(format!(
                "INSERT INTO team_memberships affected {} rows (expected 1)",
                report.rows_affected()
            )),
            Err(e) => return Err(format!("member '{}' could not be added: {e}", member.id)),
        }
    }

    serialize_team(&state.db, &team).await
        .map_err(|e| format!("team '{}' could not be read back: {e}", team_id))
}

pub async fn remove_member_from_team(
    state: &TeamRoutingState,
    team_id: &str,
    member_id: &str,
) -> Result<Value, String> {
    for table in ["teams", "team_memberships"] {
        require_table(&state.db, table).await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;
    }

    let team = sqlx::query_as::<_, TeamRow>(&format!("SELECT {TEAM_COLUMNS} FROM teams WHERE id = $1"))
        .bind(team_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?
        .ok_or_else(|| format!("Team {team_id} not found"))?;

    let report = sqlx::query("DELETE FROM team_memberships WHERE team_id = $1 AND team_member_id = $2")
        .bind(team_id)
        .bind(member_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("the membership could not be removed: {e}"))?;

    let mut team_json = serialize_team(&state.db, &team).await
        .map_err(|e| format!("team '{}' could not be read back: {e}", team_id))?;
    team_json["membershipRowsRemoved"] = json!(report.rows_affected());
    if report.rows_affected() == 0 {
        team_json["warning"] = json!(format!(
            "no team_memberships row linked member '{}' to team '{}', so there was nothing to remove",
            member_id, team_id
        ));
    }
    Ok(team_json)
}

// ---------------------------------------------------------------------------
// Routing & capacity
// ---------------------------------------------------------------------------

pub async fn list_routing_agents(state: &TeamRoutingState) -> Result<Value, String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let members = sqlx::query_as::<_, MemberRow>(&format!(
        "SELECT {MEMBER_COLUMNS} FROM team_members ORDER BY name ASC"
    ))
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("the routing agents could not be listed: {e}"))?;

    let load = active_load(&state.db).await
        .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;
    let out: Vec<Value> = members
        .iter()
        .map(|m| member_json(m, *load.get(&m.id).unwrap_or(&0)))
        .collect();
    Ok(json!(out))
}

#[derive(Deserialize)]
struct UpdateStatusPayload {
    status: String,
}

pub async fn update_agent_status(
    state: &TeamRoutingState,
    id: &str,
    body: &Value,
) -> Result<Value, String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let payload: UpdateStatusPayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    let status = payload.status.trim().to_lowercase();
    if !matches!(status.as_str(), "online" | "busy" | "offline") {
        return Err(format!("Invalid agent status '{}'. Allowed: online, busy, offline", status));
    }

    match sqlx::query("UPDATE team_members SET availability_status = $1 WHERE id = $2")
        .bind(&status)
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => Err(format!("Agent {id} not found")),
        Ok(_) => match load_member(&state.db, id).await {
            Ok(Some(m)) => {
                let load = active_load(&state.db).await
                    .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;
                Ok(member_json(&m, *load.get(id).unwrap_or(&0)))
            }
            Ok(None) => Err(format!("Agent {id} not found")),
            Err(e) => Err(format!("the team member '{}' could not be read back: {e}", id)),
        },
        Err(e) => Err(format!("the availability of team member '{}' could not be updated: {e}", id)),
    }
}

#[derive(Deserialize)]
struct UpdateCapacityPayload {
    max_capacity: i32,
}

pub async fn update_agent_capacity(
    state: &TeamRoutingState,
    id: &str,
    body: &Value,
) -> Result<Value, String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let payload: UpdateCapacityPayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    if payload.max_capacity < 1 || payload.max_capacity > 200 {
        return Err(format!(
            "max_capacity {} is out of range (1..200)",
            payload.max_capacity
        ));
    }

    match sqlx::query(
        "UPDATE team_members SET max_concurrent_conversations = $1 WHERE id = $2",
    )
    .bind(payload.max_capacity)
    .bind(id)
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => Err(format!("Agent {id} not found")),
        Ok(_) => match load_member(&state.db, id).await {
            Ok(Some(m)) => {
                let load = active_load(&state.db).await
                    .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;
                let active = *load.get(id).unwrap_or(&0);
                let mut body = member_json(&m, active);
                if active > payload.max_capacity as i64 {
                    body["warning"] = json!(format!(
                        "Agent already holds {active} open conversations, above the new cap of {}. Auto-assignment will skip them until load drops.",
                        payload.max_capacity
                    ));
                }
                Ok(body)
            }
            Ok(None) => Err(format!("Agent {id} not found")),
            Err(e) => Err(format!("the team member '{}' could not be read back: {e}", id)),
        },
        Err(e) => Err(format!("the capacity of team member '{}' could not be updated: {e}", id)),
    }
}

pub async fn get_capacity_overview(
    state: &TeamRoutingState,
    team_id: Option<&str>,
) -> Result<Value, String> {
    require_table(&state.db, "team_members").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;

    let scoped = team_id.filter(|t| !t.is_empty());
    let sql = if scoped.is_some() {
        format!(
            "SELECT {MEMBER_COLUMNS} FROM team_members WHERE id IN (SELECT team_member_id FROM team_memberships WHERE team_id = $1)"
        )
    } else {
        format!("SELECT {MEMBER_COLUMNS} FROM team_members")
    };

    let mut q = sqlx::query_as::<_, MemberRow>(&sql);
    if let Some(t) = scoped {
        q = q.bind(t);
    }
    let members = q.fetch_all(&state.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    let load = active_load(&state.db).await
        .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;

    let total_capacity: i64 = members
        .iter()
        .map(|m| m.max_concurrent_conversations.unwrap_or(0) as i64)
        .sum();
    let online_capacity: i64 = members
        .iter()
        .filter(|m| m.availability_status.as_deref() == Some("online"))
        .map(|m| m.max_concurrent_conversations.unwrap_or(0) as i64)
        .sum();
    let assigned: i64 = members
        .iter()
        .map(|m| *load.get(&m.id).unwrap_or(&0))
        .sum();

    let open_unassigned: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversations WHERE assigned_to_id IS NULL AND LOWER(state) NOT IN ('resolved','closed','closed_won','closed_lost')",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("the open unassigned conversation count could not be read: {e}"))?;

    let free_slots = (total_capacity - assigned).max(0);
    Ok(json!({
        "teamId": scoped,
        "totalAgents": members.len(),
        "onlineAgents": members.iter().filter(|m| m.availability_status.as_deref() == Some("online")).count(),
        "busyAgents": members.iter().filter(|m| m.availability_status.as_deref() == Some("busy")).count(),
        "offlineAgents": members.iter().filter(|m| m.availability_status.as_deref() != Some("online") && m.availability_status.as_deref() != Some("busy")).count(),
        "totalCapacity": total_capacity,
        "onlineCapacity": online_capacity,
        "assignedConversations": assigned,
        "freeSlots": free_slots,
        "unassignedOpenConversations": open_unassigned,
        "overallUtilization": if total_capacity > 0 {
            ((assigned as f64 / total_capacity as f64) * 1000.0).round() / 10.0
        } else { 0.0 },
    }))
}

#[derive(Deserialize)]
struct BatchAutoAssignPayload {
    strategy: Option<String>,
    team_id: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct UnassignedConversation {
    id: String,
    channel: Option<String>,
}

async fn pick_agent(
    candidates: &[MemberRow],
    load: &mut HashMap<String, i64>,
    strategy: &str,
    cursor: &mut usize,
) -> Option<MemberRow> {
    if candidates.is_empty() {
        return None;
    }
    match strategy {
        "round_robin" => {
            let len = candidates.len();
            for _ in 0..len {
                let c = candidates[*cursor % len].clone();
                *cursor = (*cursor + 1) % len;
                if c.availability_status.as_deref() == Some("online") {
                    let active = *load.get(&c.id).unwrap_or(&0);
                    if active < c.max_concurrent_conversations.unwrap_or(0) as i64 {
                        return Some(c);
                    }
                }
            }
            None
        }
        _ => {
            let mut best: Option<(i64, MemberRow)> = None;
            for c in candidates {
                if c.availability_status.as_deref() != Some("online") {
                    continue;
                }
                let cap = c.max_concurrent_conversations.unwrap_or(0) as i64;
                let active = *load.get(&c.id).unwrap_or(&0);
                if active >= cap {
                    continue;
                }
                let free = cap - active;
                if best.as_ref().map(|(b, _)| free > *b).unwrap_or(true) {
                    best = Some((free, c.clone()));
                }
            }
            best.map(|(_, m)| m)
        }
    }
}

pub async fn batch_auto_assign(
    state: &TeamRoutingState,
    body: &Value,
) -> Result<Value, String> {
    for table in ["team_members", "conversations"] {
        require_table(&state.db, table).await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;
    }

    let payload: BatchAutoAssignPayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    let strategy = payload
        .strategy
        .clone()
        .unwrap_or_else(|| "capacity_based".to_string());
    if !matches!(strategy.as_str(), "capacity_based" | "round_robin") {
        return Err(format!("Unknown strategy '{}'. Use capacity_based or round_robin", strategy));
    }

    let scoped = payload.team_id.as_deref().filter(|t| !t.is_empty());
    if scoped.is_some() {
        require_table(&state.db, "team_memberships").await.map_err(|(s, v)| format!("{}: {}", s.as_u16(), v["detail"]))?;
    }

    let members_sql = if scoped.is_some() {
        format!("SELECT {MEMBER_COLUMNS} FROM team_members WHERE id IN (SELECT team_member_id FROM team_memberships WHERE team_id = $1) ORDER BY name ASC")
    } else {
        format!("SELECT {MEMBER_COLUMNS} FROM team_members ORDER BY name ASC")
    };
    let mut mq = sqlx::query_as::<_, MemberRow>(&members_sql);
    if let Some(t) = scoped {
        mq = mq.bind(t);
    }
    let candidates = mq.fetch_all(&state.db).await
        .map_err(|e| format!("Database error: {e}"))?;

    if candidates.is_empty() {
        return Err(format!(
            "No {} agents are available to receive work. Set an agent online with capacity before auto-assigning.",
            if scoped.is_some() { "team" } else { "configured" }
        ));
    }

    let unassigned = sqlx::query_as::<_, UnassignedConversation>(
        "SELECT id, channel FROM conversations WHERE assigned_to_id IS NULL AND LOWER(state) NOT IN ('resolved','closed','closed_won','closed_lost') ORDER BY updated_at ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("the open unassigned conversations could not be read: {e}"))?;

    let mut load = active_load(&state.db).await
        .map_err(|e| format!("the per-agent active conversation load could not be read: {e}"))?;
    let mut cursor = 0usize;
    let mut results = Vec::new();
    let mut assigned_count = 0usize;
    let mut overflow_count = 0usize;

    for conv in &unassigned {
        match pick_agent(&candidates, &mut load, &strategy, &mut cursor).await {
            Some(agent) => {
                let now = Utc::now().to_rfc3339();
                let res = sqlx::query(
                    "UPDATE conversations SET assigned_to_id = $1, assigned_to_name = $2, updated_at = $3 WHERE id = $4 AND assigned_to_id IS NULL",
                )
                .bind(&agent.id)
                .bind(&agent.name)
                .bind(&now)
                .bind(&conv.id)
                .execute(&state.db)
                .await;

                match res {
                    Ok(r) if r.rows_affected() == 1 => {
                        *load.entry(agent.id.clone()).or_insert(0) += 1;
                        let _ = sqlx::query("UPDATE team_members SET last_assigned_at = $1 WHERE id = $2")
                            .bind(&now)
                            .bind(&agent.id)
                            .execute(&state.db)
                            .await;
                        assigned_count += 1;
                        results.push(json!({
                            "conversationId": conv.id,
                            "agentId": agent.id,
                            "agentName": agent.name,
                            "strategy": strategy,
                            "activeCount": *load.get(&agent.id).unwrap_or(&0),
                            "maxCapacity": agent.max_concurrent_conversations.unwrap_or(0),
                            "status": "assigned",
                        }));
                    }
                    Ok(_) => {
                        overflow_count += 1;
                        results.push(json!({
                            "conversationId": conv.id,
                            "status": "skipped",
                            "reason": "Conversation was assigned concurrently",
                        }));
                    }
                    Err(e) => {
                        results.push(json!({
                            "conversationId": conv.id,
                            "status": "failed",
                            "error": e.to_string(),
                        }));
                    }
                }
            }
            None => {
                overflow_count += 1;
                results.push(json!({
                    "conversationId": conv.id,
                    "status": "overflow",
                    "reason": "All online agents are at capacity",
                }));
            }
        }
    }

    Ok(json!({
        "strategy": strategy,
        "teamId": scoped,
        "totalProcessed": unassigned.len(),
        "assignedCount": assigned_count,
        "overflowCount": overflow_count,
        "results": results,
    }))
}
