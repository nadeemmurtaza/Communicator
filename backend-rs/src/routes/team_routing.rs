#![allow(dead_code)]

use crate::db::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn db_err(e: sqlx::Error) -> Response {
    err(StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}"))
}

/// A failed statement is reported with the driver's own message and the id it
/// worked on, instead of being folded into "not found" or into the success
/// shape the handler would otherwise build from the values it intended to write.
fn refuse(status: StatusCode, message: String, extra: serde_json::Value) -> Response {
    let mut body = json!({ "error": message, "detail": message });
    if let (Some(target), Some(source)) = (body.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    (status, Json(body)).into_response()
}

fn db_error(context: &str, err: sqlx::Error) -> Response {
    refuse(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("{context}: {err}"),
        json!({ "databaseError": true }),
    )
}

/// 0 rows on an id-scoped statement means the row is not there, so the answer
/// names the id and says nothing was changed instead of claiming the write.
fn write_not_found(table: &str, id: &str, rows: u64) -> Response {
    refuse(
        StatusCode::NOT_FOUND,
        format!(
            "no {table} row with id '{id}' exists (the statement affected {rows} rows), so \
             nothing was changed."
        ),
        json!({ "id": id, "rowsAffected": rows }),
    )
}

/// `teams` / `team_members` / `team_memberships` are created by init_db DDL.
/// When a deployment predates that DDL we must not report an empty team list
/// as if it were real data -- it is a missing schema, so say exactly that. A
/// failed `information_schema` read is NOT evidence the table is missing, so `Err`
/// travels up instead of collapsing into `false` ("missing schema").
async fn table_exists(db: &sqlx::PgPool, table: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema='public' AND table_name=$1",
    )
    .bind(table)
    .fetch_one(db)
    .await?;
    Ok(count > 0)
}

async fn require_table(db: &sqlx::PgPool, table: &str) -> Result<(), Response> {
    match table_exists(db, table).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "Schema table '{table}' is not present in this database. Apply the team DDL (see WIRING NEEDED) and restart -- refusing to report invented team data."
            ),
        )),
        Err(e) => Err(db_error(
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

/// Real live load per agent: open conversations currently assigned to them.
/// A failed read is an error, not "every agent has zero conversations" -- the
/// capacity math downstream would act on that invention.
async fn active_load(
    db: &sqlx::PgPool,
) -> Result<std::collections::HashMap<String, i64>, sqlx::Error> {
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

    let mut map = std::collections::HashMap::new();
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

fn member_json(m: &MemberRow, active: i64) -> serde_json::Value {
    // `active` comes from SQLite COUNT(*) (i64); widen the column value to the
    // same type instead of narrowing the count.
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

pub async fn list_team(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }

    let members = match sqlx::query_as::<_, MemberRow>(&format!(
        "SELECT {MEMBER_COLUMNS} FROM team_members ORDER BY name ASC"
    ))
    .fetch_all(&state.db)
    .await
    {
        Ok(m) => m,
        Err(e) => return db_err(e),
    };

    let load = match active_load(&state.db).await {
        Ok(l) => l,
        Err(e) => return db_error("the per-agent active conversation load could not be read", e),
    };
    let out: Vec<serde_json::Value> = members
        .iter()
        .map(|m| member_json(m, *load.get(&m.id).unwrap_or(&0)))
        .collect();

    Json(out).into_response()
}

#[derive(Deserialize)]
pub struct CreateTeamMemberPayload {
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub max_capacity: Option<i32>,
}

pub async fn create_team_member(
    State(state): State<AppState>,
    Json(payload): Json<CreateTeamMemberPayload>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }
    let name = payload.name.trim();
    if name.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Member name cannot be empty");
    }
    // `max_concurrent_conversations` is NOT NULL DEFAULT 5. The old fallback of
    // 10 was a capacity the schema never declared, so an unspecified request now
    // takes the column's own default and the answer says the caller never chose.
    let capacity = payload.max_capacity.unwrap_or(5);
    let capacity_supplied = payload.max_capacity.is_some();
    if capacity <= 0 || capacity > 200 {
        return err(
            StatusCode::BAD_REQUEST,
            "max_capacity must be between 1 and 200",
        );
    }

    // Do not invent an address the customer can never be reached at.
    let email = match payload
        .email
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
    {
        Some(e) => {
            if !e.contains('@') {
                return err(StatusCode::BAD_REQUEST, format!("Invalid email: {e}"));
            }
            e.to_string()
        }
        None => String::new(),
    };

    // A failed duplicate check is not "no duplicate": inserting anyway could
    // create a second member with the same address.
    let dup: Option<String> = match sqlx::query_scalar(
        "SELECT id FROM team_members WHERE email = $1 AND email <> '' LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await
    {
        Ok(d) => d,
        Err(e) => {
            return db_error(
                &format!("the duplicate-email check for '{email}' could not be run"),
                e,
            )
        }
    };
    if dup.is_some() {
        return err(
            StatusCode::CONFLICT,
            format!("A team member with email {email} already exists"),
        );
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
                (StatusCode::CREATED, Json(body)).into_response()
            }
            // The write said 1 row and the read cannot see it: report which of
            // the two answers is in doubt instead of one vague sentence.
            Ok(None) => refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "INSERT INTO team_members affected 1 row, but member '{id}' could not be read \
                     back from team_members, so the stored state is unknown."
                ),
                json!({ "id": id, "memberStored": true, "readBackOk": false }),
            ),
            Err(e) => db_error(
                &format!("the newly inserted team member '{id}' could not be read back"),
                e,
            ),
        },
        Ok(report) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "INSERT INTO team_members affected {} rows instead of 1, so member '{id}' was not \
                 stored.",
                report.rows_affected()
            ),
            json!({ "id": id, "memberStored": false, "rowsAffected": report.rows_affected() }),
        ),
        Err(e) => db_error(&format!("the team member '{id}' could not be inserted"), e),
    }
}

pub async fn delete_team_member(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }
    match sqlx::query("DELETE FROM team_members WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => err(
            StatusCode::NOT_FOUND,
            format!("Team member {id} not found"),
        ),
        Ok(deleted) => {
            // Keep conversation history referentially honest: a deleted agent
            // must not stay as the assignee of live work. If this statement
            // fails the member row is already gone while conversations still
            // name them, so the answer cannot claim a clean deletion.
            let now = Utc::now().to_rfc3339();
            let unassigned = match sqlx::query(
                "UPDATE conversations SET assigned_to_id = NULL, updated_at = $1 WHERE assigned_to_id = $2",
            )
            .bind(&now)
            .bind(&id)
            .execute(&state.db)
            .await
            {
                Ok(report) => report.rows_affected(),
                Err(e) => return refuse(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "The team member '{id}' was deleted, but clearing them as the assignee of \
                         their conversations failed (UPDATE conversations SET assigned_to_id = \
                         NULL WHERE assigned_to_id = '{id}'): {e} -- live work may still name the \
                         deleted agent."
                    ),
                    json!({ "id": id, "databaseError": true }),
                ),
            };
            // 0 rows here is legitimate: the agent may never have joined a
            // team, so the actual count is reported instead of claiming a
            // membership deletion that did not happen.
            let memberships_removed = match sqlx::query("DELETE FROM team_memberships WHERE team_member_id = $1")
                .bind(&id)
                .execute(&state.db)
                .await
            {
                Ok(report) => report.rows_affected(),
                Err(e) => return refuse(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "The team member '{id}' was deleted, but DELETE FROM team_memberships \
                         WHERE team_member_id = '{id}' failed: {e} -- the deleted agent may still \
                         appear in team rosters."
                    ),
                    json!({ "id": id, "databaseError": true }),
                ),
            };
            Json(json!({
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
            .into_response()
        }
        Err(e) => db_err(e),
    }
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
    // A failed existence read is not "the conversation is missing".
    let exists = sqlx::query_scalar::<_, String>("SELECT id FROM conversations WHERE id = $1 LIMIT 1")
        .bind(&conversation_id)
        .fetch_optional(&state.db)
        .await;
    let exists = match exists {
        Ok(id) => id,
        Err(e) => {
            return db_error(
                &format!("the existence of conversation '{conversation_id}' could not be checked"),
                e,
            )
        }
    };
    if exists.is_none() {
        return err(
            StatusCode::NOT_FOUND,
            format!("Conversation {conversation_id} not found"),
        );
    }

    if let Some(member_id) = query.member_id.as_deref().filter(|m| !m.is_empty()) {
        if let Err(r) = require_table(&state.db, "team_members").await {
            return r;
        }
        let member = match load_member(&state.db, member_id).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                return err(
                    StatusCode::NOT_FOUND,
                    format!("Team member {member_id} not found"),
                )
            }
            Err(e) => {
                return db_error(&format!("the team member '{member_id}' could not be read"), e)
            }
        };
        let now = Utc::now().to_rfc3339();
        // The JSON below claims this conversation is assigned, so the UPDATE
        // has to be proven: an error is a 500, 0 rows is a 404 naming the id.
        let assigned = sqlx::query(
            "UPDATE conversations SET assigned_to_id = $1, assigned_to_name = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&member.id)
        .bind(&member.name)
        .bind(&now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await;
        match assigned {
            Ok(report) if report.rows_affected() == 1 => {}
            Ok(report) => {
                return write_not_found("conversations", &conversation_id, report.rows_affected())
            }
            Err(e) => {
                return db_error(
                    &format!(
                        "conversation '{conversation_id}' could not be assigned to member '{member_id}'"
                    ),
                    e,
                )
            }
        }
        // Auxiliary bookkeeping: the response claims the assignment (proven
        // above), not the roster timestamp, so a failure is logged, not lost.
        crate::db::report_write(
            sqlx::query("UPDATE team_members SET last_assigned_at = $1 WHERE id = $2")
                .bind(&now)
                .bind(&member.id)
                .execute(&state.db)
                .await
                .map(|report| report.rows_affected()),
            "team_members.last_assigned_at after assigning a conversation",
        );
        let load = match active_load(&state.db).await {
            Ok(l) => l,
            Err(e) => {
                return db_error("the per-agent active conversation load could not be read", e)
            }
        };
        return Json(json!({
            "id": conversation_id,
            "assignedToId": member.id,
            "assignedToName": member.name,
            "activeConversations": *load.get(&member.id).unwrap_or(&0),
        }))
        .into_response();
    }

    // Explicit unassignment (mirrors Python member_id=None).
    let now = Utc::now().to_rfc3339();
    let unassigned = sqlx::query(
        "UPDATE conversations SET assigned_to_id = NULL, assigned_to_name = NULL, updated_at = $1 WHERE id = $2",
    )
    .bind(&now)
    .bind(&conversation_id)
    .execute(&state.db)
    .await;
    match unassigned {
        Ok(report) if report.rows_affected() == 1 => Json(json!({
            "id": conversation_id,
            "assignedToId": null,
            "assignedToName": null,
        }))
        .into_response(),
        Ok(report) => {
            write_not_found("conversations", &conversation_id, report.rows_affected())
        }
        Err(e) => db_error(
            &format!("conversation '{conversation_id}' could not be unassigned"),
            e,
        ),
    }
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
    match sqlx::query("UPDATE conversations SET human_takeover = $1, updated_at = $2 WHERE id = $3")
        .bind(query.enabled)
        .bind(&now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => err(
            StatusCode::NOT_FOUND,
            format!("Conversation {conversation_id} not found"),
        ),
        Ok(_) => Json(json!({
            "id": conversation_id,
            "humanTakeover": query.enabled,
            "updatedAt": now,
        }))
        .into_response(),
        Err(e) => db_err(e),
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
            // A decode failure is a schema problem, not an empty name: surface
            // it instead of fabricating "" for a member id or name.
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
) -> Result<serde_json::Value, sqlx::Error> {
    // membersCount and the member roster below are what the response claims,
    // so a failed member read travels up instead of becoming "0 members".
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

pub async fn list_teams(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "teams").await {
        return r;
    }
    if let Err(r) = require_table(&state.db, "team_memberships").await {
        return r;
    }

    let teams = match sqlx::query_as::<_, TeamRow>(&format!(
        "SELECT {TEAM_COLUMNS} FROM teams ORDER BY name ASC"
    ))
    .fetch_all(&state.db)
    .await
    {
        Ok(t) => t,
        Err(e) => return db_err(e),
    };

    let mut out = Vec::with_capacity(teams.len());
    for t in &teams {
        match serialize_team(&state.db, t).await {
            Ok(v) => out.push(v),
            Err(e) => {
                return db_error(
                    &format!("the members of team '{}' could not be read", t.name),
                    e,
                )
            }
        }
    }
    Json(out).into_response()
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
    if let Err(r) = require_table(&state.db, "teams").await {
        return r;
    }
    let name = payload.name.trim();
    if name.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Team name cannot be empty");
    }
    // A failed duplicate read is not "no duplicate": reporting 400/201 off it
    // would guess either way.
    let dup: Option<String> = match sqlx::query_scalar("SELECT id FROM teams WHERE name = $1 LIMIT 1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return db_error(
                &format!("the duplicate-name check for team '{name}' could not be run"),
                e,
            )
        }
    };
    if dup.is_some() {
        return err(
            StatusCode::BAD_REQUEST,
            format!("Team '{name}' already exists"),
        );
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    // `teams.auto_assign BOOLEAN NOT NULL` carries no schema default on purpose:
    // whether a team takes part in auto-routing is the caller's policy. The old
    // `unwrap_or(true)` opted every team created without the flag in, then
    // echoed that invented policy back as the team's own setting.
    let auto_assign = match payload.auto_assign {
        Some(value) => value,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                format!(
                    "auto_assign is required for team '{name}': teams.auto_assign is NOT NULL and \
                     has no default, so enabling or disabling auto-routing is not this endpoint's \
                     decision to make. Nothing was stored."
                ),
            )
        }
    };
    let description = payload.description.clone().unwrap_or_default();

    // The team is echoed back below, so the INSERT itself has to be proven.
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
        Ok(report) => {
            return refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "INSERT INTO teams affected {} rows (expected 1): team '{name}' (id '{id}') \
                     was not stored.",
                    report.rows_affected()
                ),
                json!({ "id": id, "rowsAffected": report.rows_affected(), "databaseError": true }),
            )
        }
        Err(e) => {
            return db_error(&format!("team '{name}' (id '{id}') could not be stored"), e)
        }
    }

    let row = sqlx::query_as::<_, TeamRow>(&format!(
        "SELECT {TEAM_COLUMNS} FROM teams WHERE id = $1"
    ))
    .bind(&id)
    .fetch_optional(&state.db)
    .await;
    // A failed read-back says so; only a genuinely absent row is "not stored".
    let row = match row {
        Ok(r) => r,
        Err(e) => {
            return db_error(
                &format!("the new team '{id}' could not be read back after the INSERT"),
                e,
            )
        }
    };

    match row {
        Some(t) => match serialize_team(&state.db, &t).await {
            Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
            Err(e) => db_error(
                &format!("the members of the new team '{id}' could not be read"),
                e,
            ),
        },
        None => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Team was inserted but no teams row with id '{id}' exists; the team is not stored"),
        ),
    }
}

pub async fn delete_team(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "teams").await {
        return r;
    }
    match sqlx::query("DELETE FROM teams WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, format!("Team {id} not found"))
        }
        Ok(deleted) => {
            // 0 rows here is legitimate: the team may never have had members,
            // so the actual count is reported instead of claiming memberships
            // were deleted. A failure leaves stale rows pointing at a dead team,
            // which the answer has to name.
            let memberships_removed = match sqlx::query("DELETE FROM team_memberships WHERE team_id = $1")
                .bind(&id)
                .execute(&state.db)
                .await
            {
                Ok(report) => report.rows_affected(),
                Err(e) => return refuse(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "The team '{id}' was deleted, but DELETE FROM team_memberships WHERE \
                         team_id = '{id}' failed: {e} -- stale membership rows remain."
                    ),
                    json!({ "id": id, "databaseError": true }),
                ),
            };
            Json(json!({
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
            .into_response()
        }
        Err(e) => db_err(e),
    }
}

#[derive(Deserialize)]
pub struct AddTeamMemberPayload {
    pub member_id: String,
}

pub async fn add_member_to_team(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AddTeamMemberPayload>,
) -> impl IntoResponse {
    for table in ["teams", "team_members", "team_memberships"] {
        if let Err(r) = require_table(&state.db, table).await {
            return r;
        }
    }

    let team = sqlx::query_as::<_, TeamRow>(&format!("SELECT {TEAM_COLUMNS} FROM teams WHERE id = $1"))
        .bind(&id)
        .fetch_optional(&state.db)
        .await;
    let team = match team {
        Ok(Some(t)) => t,
        Ok(None) => return err(StatusCode::NOT_FOUND, format!("Team {id} not found")),
        Err(e) => return db_err(e),
    };

    let member = match load_member(&state.db, &payload.member_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                format!("Team member {} not found", payload.member_id),
            )
        }
        Err(e) => return db_err(e),
    };

    // A failed existence read is not "not a member yet".
    let already: Option<i64> = match sqlx::query_scalar(
        "SELECT 1 FROM team_memberships WHERE team_id = $1 AND team_member_id = $2 LIMIT 1",
    )
    .bind(&id)
    .bind(&member.id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(a) => a,
        Err(e) => {
            return db_error(
                &format!(
                    "the existing membership of member '{}' in team '{id}' could not be checked",
                    member.id
                ),
                e,
            )
        }
    };

    if already.is_none() {
        let inserted = sqlx::query(
            "INSERT INTO team_memberships (team_id, team_member_id) VALUES ($1, $2)",
        )
        .bind(&id)
        .bind(&member.id)
        .execute(&state.db)
        .await;
        match inserted {
            Ok(report) if report.rows_affected() == 1 => {}
            Ok(report) => {
                return refuse(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "INSERT INTO team_memberships affected {} rows (expected 1): member '{}' \
                         was not added to team '{id}'.",
                        report.rows_affected(),
                        member.id
                    ),
                    json!({ "id": id, "memberId": member.id, "rowsAffected": report.rows_affected(), "databaseError": true }),
                )
            }
            Err(e) => {
                return db_error(
                    &format!("member '{}' could not be added to team '{id}'", member.id),
                    e,
                )
            }
        }
    }

    // The roster returned below is the team's real state; a failed read of it
    // is reported as such rather than as an empty team.
    match serialize_team(&state.db, &team).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => db_error(
            &format!(
                "team '{id}' could not be read back after adding member '{}'",
                member.id
            ),
            e,
        ),
    }
}

pub async fn remove_member_from_team(
    State(state): State<AppState>,
    Path((id, member_id)): Path<(String, String)>,
) -> impl IntoResponse {
    for table in ["teams", "team_memberships"] {
        if let Err(r) = require_table(&state.db, table).await {
            return r;
        }
    }

    let team = sqlx::query_as::<_, TeamRow>(&format!("SELECT {TEAM_COLUMNS} FROM teams WHERE id = $1"))
        .bind(&id)
        .fetch_optional(&state.db)
        .await;
    let team = match team {
        Ok(Some(t)) => t,
        Ok(None) => return err(StatusCode::NOT_FOUND, format!("Team {id} not found")),
        Err(e) => return db_err(e),
    };

    match sqlx::query("DELETE FROM team_memberships WHERE team_id = $1 AND team_member_id = $2")
        .bind(&id)
        .bind(&member_id)
        .execute(&state.db)
        .await
    {
        Ok(report) => {
            // 0 rows is legitimate: the membership may already have been gone.
            // The roster below is re-read from the database, and the counts say
            // plainly whether this DELETE removed anything.
            let mut team_json = match serialize_team(&state.db, &team).await {
                Ok(v) => v,
                Err(e) => {
                    return db_error(
                        &format!(
                            "team '{id}' could not be read back after removing member '{member_id}'"
                        ),
                        e,
                    )
                }
            };
            team_json["membershipRowsRemoved"] = json!(report.rows_affected());
            if report.rows_affected() == 0 {
                team_json["warning"] = json!(format!(
                    "no team_memberships row linked member '{member_id}' to team '{id}', so \
                     there was nothing to remove"
                ));
            }
            Json(team_json).into_response()
        }
        Err(e) => db_error(
            &format!(
                "the membership of member '{member_id}' could not be removed from team '{id}'"
            ),
            e,
        ),
    }
}

// ---------------------------------------------------------------------------
// Routing & capacity
// ---------------------------------------------------------------------------

pub async fn list_routing_agents(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }
    let members = sqlx::query_as::<_, MemberRow>(&format!(
        "SELECT {MEMBER_COLUMNS} FROM team_members ORDER BY name ASC"
    ))
    .fetch_all(&state.db)
    .await;

    let members = match members {
        // A failed read is not "there are no routing agents".
        Ok(m) => m,
        Err(e) => return db_error("the routing agents could not be listed from team_members", e),
    };

    let load = match active_load(&state.db).await {
        Ok(l) => l,
        Err(e) => return db_error("the per-agent active conversation load could not be read", e),
    };
    let out: Vec<serde_json::Value> = members
        .iter()
        .map(|m| member_json(m, *load.get(&m.id).unwrap_or(&0)))
        .collect();
    Json(out).into_response()
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
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }
    let status = payload.status.trim().to_lowercase();
    if !matches!(status.as_str(), "online" | "busy" | "offline") {
        return err(
            StatusCode::BAD_REQUEST,
            format!("Invalid agent status '{status}'. Allowed: online, busy, offline"),
        );
    }

    match sqlx::query("UPDATE team_members SET availability_status = $1 WHERE id = $2")
        .bind(&status)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, format!("Agent {id} not found"))
        }
        Ok(_) => match load_member(&state.db, &id).await {
            Ok(Some(m)) => {
                let load = match active_load(&state.db).await {
                    Ok(l) => l,
                    Err(e) => {
                        return db_error(
                            "the per-agent active conversation load could not be read",
                            e,
                        )
                    }
                };
                Json(member_json(&m, *load.get(&id).unwrap_or(&0))).into_response()
            }
            Ok(None) => err(StatusCode::NOT_FOUND, format!("Agent {id} not found")),
            Err(e) => db_error(
                &format!(
                    "the team member '{id}' could not be read back after the availability UPDATE"
                ),
                e,
            ),
        },
        Err(e) => db_error(
            &format!("the availability of team member '{id}' could not be updated"),
            e,
        ),
    }
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
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }
    if payload.max_capacity < 1 || payload.max_capacity > 200 {
        return err(
            StatusCode::BAD_REQUEST,
            format!(
                "max_capacity {} is out of range (1..200)",
                payload.max_capacity
            ),
        );
    }

    match sqlx::query(
        "UPDATE team_members SET max_concurrent_conversations = $1 WHERE id = $2",
    )
    .bind(payload.max_capacity)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, format!("Agent {id} not found"))
        }
        Ok(_) => match load_member(&state.db, &id).await {
            Ok(Some(m)) => {
                let load = match active_load(&state.db).await {
                    Ok(l) => l,
                    Err(e) => {
                        return db_error(
                            "the per-agent active conversation load could not be read",
                            e,
                        )
                    }
                };
                let active = *load.get(&id).unwrap_or(&0);
                let mut body = member_json(&m, active);
                if active > payload.max_capacity as i64 {
                    // Real warning, computed from real state.
                    body["warning"] = json!(format!(
                        "Agent already holds {active} open conversations, above the new cap of {}. Auto-assignment will skip them until load drops.",
                        payload.max_capacity
                    ));
                }
                Json(body).into_response()
            }
            Ok(None) => err(StatusCode::NOT_FOUND, format!("Agent {id} not found")),
            Err(e) => db_error(
                &format!(
                    "the team member '{id}' could not be read back after the capacity UPDATE"
                ),
                e,
            ),
        },
        Err(e) => db_error(
            &format!("the capacity of team member '{id}' could not be updated"),
            e,
        ),
    }
}

#[derive(Deserialize)]
pub struct CapacityQuery {
    pub team_id: Option<String>,
}

pub async fn get_capacity_overview(
    State(state): State<AppState>,
    Query(query): Query<CapacityQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "team_members").await {
        return r;
    }

    let scoped = query.team_id.as_deref().filter(|t| !t.is_empty());
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
    let members = match q.fetch_all(&state.db).await {
        Ok(m) => m,
        Err(e) => return db_err(e),
    };

    let load = match active_load(&state.db).await {
        Ok(l) => l,
        Err(e) => return db_error("the per-agent active conversation load could not be read", e),
    };
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
    // A failed count is not "zero unassigned conversations".
    let open_unassigned: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversations WHERE assigned_to_id IS NULL AND LOWER(state) NOT IN ('resolved','closed','closed_won','closed_lost')",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(n) => n,
        Err(e) => {
            return db_error("the open unassigned conversation count could not be read", e)
        }
    };

    let free_slots = (total_capacity - assigned).max(0);
    Json(json!({
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
    .into_response()
}

#[derive(Deserialize)]
pub struct BatchAutoAssignPayload {
    pub strategy: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct UnassignedConversation {
    id: String,
    channel: Option<String>,
}

/// Shared capacity/round-robin assignment used by the batch endpoint.
async fn pick_agent(
    candidates: &[MemberRow],
    load: &mut std::collections::HashMap<String, i64>,
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
    State(state): State<AppState>,
    Json(payload): Json<BatchAutoAssignPayload>,
) -> impl IntoResponse {
    for table in ["team_members", "conversations"] {
        if let Err(r) = require_table(&state.db, table).await {
            return r;
        }
    }

    let strategy = payload
        .strategy
        .clone()
        .unwrap_or_else(|| "capacity_based".to_string());
    if !matches!(strategy.as_str(), "capacity_based" | "round_robin") {
        return err(
            StatusCode::BAD_REQUEST,
            format!("Unknown strategy '{strategy}'. Use capacity_based or round_robin"),
        );
    }

    let scoped = payload.team_id.as_deref().filter(|t| !t.is_empty());
    if scoped.is_some() {
        // The old `if let Some(t) = scoped` bound the team id here, discarded it
        // with `let _ = t;`, and re-bound it below where the member query
        // actually uses it. `t` was never a transaction or timer: the table
        // check needs no value, so the unused binding is simply gone.
        if let Err(r) = require_table(&state.db, "team_memberships").await {
            return r;
        }
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
    let candidates = match mq.fetch_all(&state.db).await {
        Ok(m) => m,
        Err(e) => return db_err(e),
    };

    if candidates.is_empty() {
        return err(
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "No {} agents are available to receive work. Set an agent online with capacity before auto-assigning.",
                if scoped.is_some() { "team" } else { "configured" }
            ),
        );
    }

    // A failed read is reported as one, not as totalProcessed 0 (which would
    // claim there was no open work).
    let unassigned = match sqlx::query_as::<_, UnassignedConversation>(
        "SELECT id, channel FROM conversations WHERE assigned_to_id IS NULL AND LOWER(state) NOT IN ('resolved','closed','closed_won','closed_lost') ORDER BY updated_at ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return db_error("the open unassigned conversations could not be read", e)
        }
    };

    let mut load = match active_load(&state.db).await {
        Ok(l) => l,
        Err(e) => return db_error("the per-agent active conversation load could not be read", e),
    };
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
                        // Side bookkeeping: the result row claims the
                        // assignment (proven by the UPDATE above), not the
                        // roster timestamp, so a failure is logged, not lost.
                        crate::db::report_write(
                            sqlx::query("UPDATE team_members SET last_assigned_at = $1 WHERE id = $2")
                                .bind(&now)
                                .bind(&agent.id)
                                .execute(&state.db)
                                .await
                                .map(|report| report.rows_affected()),
                            "team_members.last_assigned_at after a batch auto-assignment",
                        );
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

    Json(json!({
        "strategy": strategy,
        "teamId": scoped,
        "totalProcessed": unassigned.len(),
        "assignedCount": assigned_count,
        "overflowCount": overflow_count,
        "results": results,
    }))
    .into_response()
}
