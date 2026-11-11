#![allow(dead_code)]

use crate::db::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn parse_dt(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
        return Some(DateTime::from_naive_utc_and_offset(ndt, Utc));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(ndt, Utc));
    }
    None
}

/// House style (see `routes/broadcasts.rs`): a write that did not land answers
/// with `saved: false` and names the id and the statement, so a client can never
/// render a canned response, macro or SLA policy as changed when no row was
/// touched.
fn refuse(status: StatusCode, message: &str, extra: serde_json::Value) -> Response {
    let mut body = json!({ "error": message, "saved": false });
    if let (Some(target), Some(source)) = (body.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    (status, Json(body)).into_response()
}

/// A failed statement is reported with the driver's own message instead of being
/// folded into "not found" or into the success shape the handler would otherwise
/// build from the values it intended to write.
fn db_error(context: &str, err: sqlx::Error) -> Response {
    refuse(
        StatusCode::INTERNAL_SERVER_ERROR,
        &format!("{context}: {err}"),
        json!({ "databaseError": true }),
    )
}

/// 0 rows on an id-scoped UPDATE means the row is not there for this caller, so
/// it is reported as 404 naming the id rather than as a success.
fn write_not_found(table: &str, assignment: &str, id: &str, rows: u64) -> Response {
    refuse(
        StatusCode::NOT_FOUND,
        &format!(
            "UPDATE {table} SET {assignment} affected {rows} rows: no {table} row with id '{id}' \
             exists, so nothing was changed."
        ),
        json!({ "id": id, "rowsAffected": rows }),
    )
}

/// An INSERT that reports anything but one row created nothing, even though the
/// handler holds a fully built object to serialize.
fn write_not_stored(statement: &str, id: &str, rows: u64) -> Response {
    refuse(
        StatusCode::INTERNAL_SERVER_ERROR,
        &format!(
            "{statement} affected {rows} rows (expected 1): id '{id}' was not stored."
        ),
        json!({ "id": id, "rowsAffected": rows, "databaseError": true }),
    )
}

// ============================================================================
// CANNED RESPONSES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CannedResponseRow {
    pub id: String,
    pub short_code: String,
    pub content: String,
    pub category: Option<String>,
    pub created_at: Option<String>,
}

fn serialize_canned(c: &CannedResponseRow) -> serde_json::Value {
    json!({
        "id": c.id,
        "shortCode": c.short_code,
        "short_code": c.short_code,
        "content": c.content,
        "category": c.category.as_deref().unwrap_or("general"),
        "createdAt": c.created_at,
        "created_at": c.created_at,
    })
}

#[derive(Deserialize)]
pub struct CannedQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

pub async fn list_canned_responses(
    State(state): State<AppState>,
    Query(query): Query<CannedQuery>,
) -> impl IntoResponse {

    let canned = sqlx::query_as::<_, CannedResponseRow>(
        "SELECT id, short_code, content, category, created_at FROM canned_responses ORDER BY short_code ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut filtered = canned;
    if let Some(ref cat) = query.category {
        let cat_trimmed = cat.trim().to_lowercase();
        if !cat_trimmed.is_empty() && cat_trimmed != "all" {
            filtered.retain(|c| c.category.as_deref().unwrap_or("general").to_lowercase() == cat_trimmed);
        }
    }
    if let Some(ref q) = query.q {
        let q_trimmed = q.trim().trim_start_matches('/').to_lowercase();
        if !q_trimmed.is_empty() {
            filtered.retain(|c| {
                c.short_code.to_lowercase().contains(&q_trimmed)
                    || c.content.to_lowercase().contains(&q_trimmed)
            });
        }
    }

    let items: Vec<serde_json::Value> = filtered.iter().map(serialize_canned).collect();
    Json(items)
}

#[derive(Deserialize)]
pub struct CreateCannedPayload {
    pub short_code: String,
    pub content: String,
    pub category: Option<String>,
}

pub async fn create_canned_response(
    State(state): State<AppState>,
    Json(payload): Json<CreateCannedPayload>,
) -> impl IntoResponse {
    let code = payload.short_code.trim().trim_start_matches('/').to_string();
    if code.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Short code cannot be empty." })),
        )
            .into_response();
    }

    // Check duplicate short_code
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT id FROM canned_responses WHERE LOWER(short_code) = LOWER(?) LIMIT 1",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if existing.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Short code /{} already exists", code) })),
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();
    let category = payload.category.unwrap_or_else(|| "general".to_string()).trim().to_lowercase();
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        "INSERT INTO canned_responses (id, short_code, content, category, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&code)
    .bind(payload.content.trim())
    .bind(&category)
    .bind(&now)
    .execute(&state.db)
    .await;

    if let Err(e) = res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
            .into_response();
    }

    let created = CannedResponseRow {
        id,
        short_code: code,
        content: payload.content.trim().to_string(),
        category: Some(category),
        created_at: Some(now),
    };

    (StatusCode::OK, Json(serialize_canned(&created))).into_response()
}

#[derive(Deserialize)]
pub struct UpdateCannedPayload {
    pub short_code: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
}

pub async fn update_canned_response(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCannedPayload>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, CannedResponseRow>(
        "SELECT id, short_code, content, category, created_at FROM canned_responses WHERE id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    let mut current = match existing {
        Ok(Some(item)) => item,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Canned response not found", "id": id })),
            )
                .into_response();
        }
        Err(err) => return db_error(&format!("the canned response '{id}' could not be read"), err),
    };

    if let Some(ref sc) = payload.short_code {
        let code = sc.trim().trim_start_matches('/').to_string();
        if code.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Short code cannot be empty." })),
            )
                .into_response();
        }
        if code.to_lowercase() != current.short_code.to_lowercase() {
            let dup: Option<String> = sqlx::query_scalar(
                "SELECT id FROM canned_responses WHERE LOWER(short_code) = LOWER(?) AND id != ? LIMIT 1",
            )
            .bind(&code)
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

            if dup.is_some() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Short code /{} already exists", code) })),
                )
                    .into_response();
            }
        }
        current.short_code = code;
    }

    if let Some(ref c) = payload.content {
        current.content = c.trim().to_string();
    }

    if let Some(ref cat) = payload.category {
        current.category = Some(cat.trim().to_lowercase());
    }

    // The response below serializes the intended row, so the UPDATE itself has to
    // be proven: an error or 0 rows means nothing changed.
    let updated = sqlx::query(
        "UPDATE canned_responses SET short_code = ?, content = ?, category = ? WHERE id = ?",
    )
    .bind(&current.short_code)
    .bind(&current.content)
    .bind(&current.category)
    .bind(&id)
    .execute(&state.db)
    .await;

    match updated {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => {
            return write_not_found(
                "canned_responses",
                "short_code, content, category",
                &id,
                report.rows_affected(),
            );
        }
        Err(err) => {
            return db_error(&format!("the canned response '{id}' could not be updated"), err)
        }
    }

    (StatusCode::OK, Json(serialize_canned(&current))).into_response()
}

pub async fn delete_canned_response(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM canned_responses WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    let rows = res.map(|r| r.rows_affected()).unwrap_or(0);
    if rows == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Canned response not found", "deleted": false })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({ "status": "deleted", "deleted": true, "id": id })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct ExpandCannedPayload {
    pub short_code: String,
    pub conversation_id: Option<String>,
    pub custom_vars: Option<HashMap<String, String>>,
}

pub async fn expand_canned_response(
    State(state): State<AppState>,
    Json(payload): Json<ExpandCannedPayload>,
) -> impl IntoResponse {
    let code = payload.short_code.trim().trim_start_matches('/').to_string();

    let canned = sqlx::query_as::<_, CannedResponseRow>(
        "SELECT id, short_code, content, category, created_at FROM canned_responses WHERE LOWER(short_code) = LOWER(?) LIMIT 1",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let canned = match canned {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Canned response /{} not found", code) })),
            )
                .into_response();
        }
    };

    let mut variables: HashMap<String, String> = HashMap::new();
    variables.insert("customer.name".to_string(), "Customer".to_string());
    variables.insert("customer_name".to_string(), "Customer".to_string());
    variables.insert("customer.first_name".to_string(), "there".to_string());
    variables.insert("first_name".to_string(), "there".to_string());
    variables.insert("agent.name".to_string(), "Support Agent".to_string());
    variables.insert("agent_name".to_string(), "Support Agent".to_string());
    variables.insert("channel".to_string(), "Support".to_string());
    variables.insert("ticket.id".to_string(), "—".to_string());
    variables.insert("ticket_id".to_string(), "—".to_string());
    variables.insert("conversation_id".to_string(), "—".to_string());

    if let Some(ref conv_id) = payload.conversation_id {
        #[derive(sqlx::FromRow)]
        struct ConvContext {
            id: String,
            contact_id: Option<String>,
            assigned_to_id: Option<String>,
        }

        let conv = sqlx::query_as::<_, ConvContext>(
            "SELECT id, contact_id, assigned_to_id FROM conversations WHERE id = ? LIMIT 1",
        )
        .bind(conv_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        if let Some(c) = conv {
            let t_id = if c.id.len() >= 8 { &c.id[..8] } else { &c.id };
            variables.insert("ticket.id".to_string(), t_id.to_string());
            variables.insert("ticket_id".to_string(), t_id.to_string());
            variables.insert("conversation_id".to_string(), c.id.clone());

            if let Some(ref agent_id) = c.assigned_to_id {
                let agent_name: Option<String> = sqlx::query_scalar(
                    "SELECT name FROM team_members WHERE id = ? LIMIT 1",
                )
                .bind(agent_id)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);

                if let Some(name) = agent_name {
                    variables.insert("agent.name".to_string(), name.clone());
                    variables.insert("agent_name".to_string(), name);
                }
            }

            if let Some(ref contact_id) = c.contact_id {
                #[derive(sqlx::FromRow)]
                struct ContactRow {
                    person_name: Option<String>,
                    channel: Option<String>,
                }

                let contact = sqlx::query_as::<_, ContactRow>(
                    "SELECT person_name, channel FROM contacts WHERE id = ? LIMIT 1",
                )
                .bind(contact_id)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);

                if let Some(ct) = contact {
                    if let Some(p_name) = ct.person_name {
                        let trimmed = p_name.trim();
                        let first = trimmed.split_whitespace().next().unwrap_or(trimmed);
                        variables.insert("customer.name".to_string(), trimmed.to_string());
                        variables.insert("customer_name".to_string(), trimmed.to_string());
                        variables.insert("customer.first_name".to_string(), first.to_string());
                        variables.insert("first_name".to_string(), first.to_string());
                    }
                    if let Some(ch) = ct.channel {
                        let mut chars = ch.chars();
                        let title = match chars.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        };
                        variables.insert("channel".to_string(), title);
                    }
                }
            }
        }
    }

    if let Some(ref custom) = payload.custom_vars {
        for (k, v) in custom {
            variables.insert(k.clone(), v.clone());
        }
    }

    let mut rendered = canned.content.clone();
    for (k, v) in &variables {
        let pat1 = format!("{{{{{}}}}}", k);
        let pat2 = format!("{{{{ {} }}}}", k);
        rendered = rendered.replace(&pat1, v).replace(&pat2, v);
    }

    (
        StatusCode::OK,
        Json(json!({
            "shortCode": canned.short_code,
            "rawContent": canned.content,
            "renderedContent": rendered,
            "variables": variables,
        })),
    )
        .into_response()
}

// ============================================================================
// MACROS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MacroItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub actions: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn serialize_macro(m: &MacroItem) -> serde_json::Value {
    let acts: serde_json::Value = m
        .actions
        .as_deref()
        .and_then(|a| serde_json::from_str::<serde_json::Value>(a).ok())
        .unwrap_or_else(|| json!([]));

    json!({
        "id": m.id,
        "name": m.name,
        "description": m.description.as_deref().unwrap_or(""),
        "actions": acts,
        "createdAt": m.created_at,
        "updatedAt": m.updated_at,
    })
}

pub async fn list_macros(State(state): State<AppState>) -> impl IntoResponse {

    let macros = sqlx::query_as::<_, MacroItem>(
        "SELECT id, name, description, actions, created_at, updated_at FROM macros ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<serde_json::Value> = macros.iter().map(serialize_macro).collect();
    Json(result)
}

#[derive(Deserialize)]
pub struct CreateMacroPayload {
    pub name: String,
    pub description: Option<String>,
    pub actions: serde_json::Value,
}

pub async fn create_macro(
    State(state): State<AppState>,
    Json(payload): Json<CreateMacroPayload>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Macro name cannot be empty" })),
        )
            .into_response();
    }
    if !payload.actions.is_array() || payload.actions.as_array().map(|a| a.is_empty()).unwrap_or(true) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Macro must contain at least one action" })),
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();
    let actions_str = payload.actions.to_string();
    let now = Utc::now().to_rfc3339();

    // The macro object built below is only true once the INSERT lands.
    let stored = sqlx::query(
        "INSERT INTO macros (id, name, description, actions, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.description.as_deref().unwrap_or(""))
    .bind(&actions_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    match stored {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => return write_not_stored("INSERT INTO macros", &id, report.rows_affected()),
        Err(err) => return db_error("the macro could not be stored", err),
    }

    let m = MacroItem {
        id,
        name: payload.name.trim().to_string(),
        description: payload.description,
        actions: Some(actions_str),
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    (StatusCode::OK, Json(serialize_macro(&m))).into_response()
}

pub async fn get_macro(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let m = sqlx::query_as::<_, MacroItem>(
        "SELECT id, name, description, actions, created_at, updated_at FROM macros WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(item) = m {
        (StatusCode::OK, Json(serialize_macro(&item))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Macro not found" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct UpdateMacroPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub actions: Option<serde_json::Value>,
}

pub async fn update_macro(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateMacroPayload>,
) -> impl IntoResponse {
    let m = sqlx::query_as::<_, MacroItem>(
        "SELECT id, name, description, actions, created_at, updated_at FROM macros WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    let mut current = match m {
        Ok(Some(item)) => item,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Macro not found", "id": id })),
            )
                .into_response();
        }
        Err(err) => return db_error(&format!("the macro '{id}' could not be read"), err),
    };

    if let Some(ref name) = payload.name {
        current.name = name.trim().to_string();
    }
    if let Some(ref desc) = payload.description {
        current.description = Some(desc.trim().to_string());
    }
    if let Some(ref acts) = payload.actions {
        current.actions = Some(acts.to_string());
    }
    let now = Utc::now().to_rfc3339();
    current.updated_at = Some(now.clone());

    // The serialized macro returned below carries these exact values, so the
    // UPDATE has to be proven before they can be reported as stored.
    let updated = sqlx::query("UPDATE macros SET name = ?, description = ?, actions = ?, updated_at = ? WHERE id = ?")
        .bind(&current.name)
        .bind(&current.description)
        .bind(&current.actions)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match updated {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => {
            return write_not_found("macros", "name, description, actions, updated_at", &id, report.rows_affected());
        }
        Err(err) => return db_error(&format!("the macro '{id}' could not be updated"), err),
    }

    (StatusCode::OK, Json(serialize_macro(&current))).into_response()
}

pub async fn delete_macro(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM macros WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    let rows = res.map(|r| r.rows_affected()).unwrap_or(0);
    if rows == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Macro not found", "deleted": false })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "deleted": true, "id": id })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct ExecuteMacroPayload {
    pub agent_name: Option<String>,
}

/// Shared macro engine. Every action either performs a real, persisted change
/// or reports exactly what it could not do -- no "executed" for a no-op.
async fn run_macro(state: &AppState, conversation_id: String, macro_id: String, agent_name: String) -> Response {
    let macro_row = sqlx::query_as::<_, MacroItem>(
        "SELECT id, name, description, actions, created_at, updated_at FROM macros WHERE id = ?",
    )
    .bind(&macro_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let m = match macro_row {
        Some(item) => item,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Macro {} not found", macro_id) })),
            )
                .into_response();
        }
    };

    #[derive(sqlx::FromRow)]
    struct ConvItem {
        id: String,
        contact_id: Option<String>,
        state: String,
        priority: Option<String>,
        resolution_type: Option<String>,
        custom_attributes: Option<String>,
        team_id: Option<String>,
        assigned_to_id: Option<String>,
    }

    let conv = sqlx::query_as::<_, ConvItem>(
        "SELECT id, contact_id, state, priority, resolution_type, custom_attributes, team_id, assigned_to_id FROM conversations WHERE id = ?",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let mut conversation = match conv {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Conversation {} not found", conversation_id) })),
            )
                .into_response();
        }
    };

    let agent_name = if agent_name.trim().is_empty() {
        "Agent".to_string()
    } else {
        agent_name
    };
    let now = Utc::now().to_rfc3339();

    let mut attrs: serde_json::Map<String, serde_json::Value> = conversation
        .custom_attributes
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v: serde_json::Value| v.as_object().cloned())
        .unwrap_or_default();

    let mut current_tags: Vec<String> = attrs
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let actions: Vec<serde_json::Value> = m
        .actions
        .as_deref()
        .and_then(|a| serde_json::from_str(a).ok())
        .unwrap_or_default();

    let mut executed_actions = Vec::new();
    let mut dispatched = 0usize;
    let mut deferred = 0usize;

    // Outbound channel resolution for send_message actions (real dispatch).
    #[derive(sqlx::FromRow)]
    struct SendTarget {
        channel: Option<String>,
        phone: Option<String>,
        email: Option<String>,
    }
    let send_target = if let Some(ref contact_id) = conversation.contact_id {
        sqlx::query_as::<_, SendTarget>(
            "SELECT channel, value AS phone, email FROM contacts WHERE id = ? LIMIT 1",
        )
        .bind(contact_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
    } else {
        None
    };

    for action in &actions {
        let a_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match a_type {
            "resolve_conversation" => {
                conversation.state = "closed_won".to_string();
                let res_type = action
                    .get("resolution_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("human_assisted");
                conversation.resolution_type = Some(res_type.to_string());
                executed_actions.push(json!({ "action": a_type, "status": "applied" }));
            }
            "change_priority" => {
                if let Some(p) = action.get("priority").and_then(|v| v.as_str()) {
                    conversation.priority = Some(p.to_lowercase());
                    executed_actions.push(json!({ "action": a_type, "priority": p.to_lowercase(), "status": "applied" }));
                } else {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": "action is missing a 'priority' value",
                    }));
                    deferred += 1;
                }
            }
            "change_status" => {
                if let Some(st) = action.get("status").and_then(|v| v.as_str()) {
                    conversation.state = st.to_lowercase();
                    executed_actions.push(json!({ "action": a_type, "state": st.to_lowercase(), "status": "applied" }));
                } else {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": "action is missing a 'status' value",
                    }));
                    deferred += 1;
                }
            }
            "assign_team" => {
                let team_id = action.get("team_id").and_then(|v| v.as_str());
                let team_name = action.get("team_name").and_then(|v| v.as_str());
                if let Some(tid) = team_id {
                    conversation.team_id = Some(tid.to_string());
                    executed_actions.push(json!({ "action": a_type, "teamId": tid, "status": "applied" }));
                } else if let Some(tname) = team_name {
                    let matched: Option<String> = sqlx::query_scalar(
                        "SELECT id FROM teams WHERE LOWER(name) LIKE LOWER(?) LIMIT 1",
                    )
                    .bind(format!("%{tname}%"))
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None);
                    match matched {
                        Some(tid) => {
                            conversation.team_id = Some(tid.clone());
                            executed_actions.push(json!({ "action": a_type, "teamId": tid, "status": "applied" }));
                        }
                        None => {
                            executed_actions.push(json!({
                                "action": a_type,
                                "requestedTeam": tname,
                                "status": "failed",
                                "error": format!("No row in the `teams` table matches '{tname}' (the table may be absent from this database -- see WIRING NEEDED)"),
                            }));
                            deferred += 1;
                        }
                    }
                } else {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": "action needs either team_id or team_name",
                    }));
                    deferred += 1;
                }
            }
            "assign_agent" => {
                let agent_id = action.get("agent_id").and_then(|v| v.as_str());
                let agent_lookup = action.get("agent_name").and_then(|v| v.as_str());
                let resolved: Option<(String, String)> = if let Some(aid) = agent_id {
                    sqlx::query_as::<_, (String, String)>(
                        "SELECT id, name FROM team_members WHERE id = ? LIMIT 1",
                    )
                    .bind(aid)
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None)
                } else if let Some(aname) = agent_lookup {
                    sqlx::query_as::<_, (String, String)>(
                        "SELECT id, name FROM team_members WHERE LOWER(name) LIKE LOWER(?) LIMIT 1",
                    )
                    .bind(format!("%{aname}%"))
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None)
                } else {
                    None
                };

                match resolved {
                    Some((aid, aname)) => {
                        conversation.assigned_to_id = Some(aid.clone());
                        executed_actions.push(json!({
                            "action": a_type, "agentId": aid, "agentName": aname, "status": "applied",
                        }));
                    }
                    None => {
                        executed_actions.push(json!({
                            "action": a_type,
                            "status": "failed",
                            "error": "No row in `team_members` matched this agent_id/agent_name (the table may be absent from this database); conversation left unassigned",
                        }));
                        deferred += 1;
                    }
                }
            }
            "add_tags" => {
                if let Some(tags_arr) = action.get("tags").and_then(|v| v.as_array()) {
                    let mut added = Vec::new();
                    for t in tags_arr {
                        if let Some(tag_str) = t.as_str() {
                            let tag_lower = tag_str.trim().to_lowercase();
                            if !current_tags.contains(&tag_lower) {
                                current_tags.push(tag_lower.clone());
                                added.push(tag_lower);
                            }
                        }
                    }
                    executed_actions.push(json!({ "action": a_type, "added": added, "status": "applied" }));
                } else {
                    executed_actions.push(json!({
                        "action": a_type, "status": "failed", "error": "action has no 'tags' array",
                    }));
                    deferred += 1;
                }
            }
            "remove_tags" => {
                if let Some(tags_arr) = action.get("tags").and_then(|v| v.as_array()) {
                    let to_remove: Vec<String> = tags_arr
                        .iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_lowercase()))
                        .collect();
                    current_tags.retain(|t| !to_remove.contains(t));
                    executed_actions.push(json!({ "action": a_type, "removed": to_remove, "status": "applied" }));
                } else {
                    executed_actions.push(json!({
                        "action": a_type, "status": "failed", "error": "action has no 'tags' array",
                    }));
                    deferred += 1;
                }
            }
            "mutate_custom_attributes" => {
                if let Some(extra) = action.get("attributes").and_then(|v| v.as_object()) {
                    let mut keys = Vec::new();
                    for (k, v) in extra {
                        attrs.insert(k.clone(), v.clone());
                        keys.push(k.clone());
                    }
                    executed_actions.push(json!({ "action": a_type, "mutated": keys, "status": "applied" }));
                } else {
                    executed_actions.push(json!({
                        "action": a_type, "status": "failed", "error": "action has no 'attributes' object",
                    }));
                    deferred += 1;
                }
            }
            "send_message" => {
                let msg_text = action
                    .get("message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .unwrap_or("");
                if msg_text.is_empty() {
                    executed_actions.push(json!({
                        "action": a_type, "status": "failed", "error": "action has no 'message' text",
                    }));
                    deferred += 1;
                    continue;
                }

                let msg_id = Uuid::new_v4().to_string();
                let Some(target) = send_target.as_ref() else {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": "the conversation has no contact record, so there is no address to send to; nothing was stored or dispatched",
                    }));
                    deferred += 1;
                    continue;
                };
                let Some(channel) = target
                    .channel
                    .clone()
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                else {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": "the contact row has no channel value, so the message cannot be attributed to a real channel",
                    }));
                    deferred += 1;
                    continue;
                };
                let phone = target.phone.clone().unwrap_or_default();
                if channel.eq_ignore_ascii_case("whatsapp") && phone.trim().is_empty() {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": "the whatsapp contact has no address on the conversation or contact row; nothing was dispatched",
                    }));
                    deferred += 1;
                    continue;
                }

                // Persist the message first, with the delivery state we can prove.
                if let Err(err) = sqlx::query(
                    "INSERT INTO messages (id, conversation_id, sender, sender_name, direction, channel, body, status, delivery_status, sent_at, created_at, provider) VALUES (?, ?, 'agent', ?, 'outbound', ?, ?, 'pending', 'pending', ?, ?, 'macro_executor')",
                )
                .bind(&msg_id)
                .bind(&conversation_id)
                .bind(&agent_name)
                .bind(&channel)
                .bind(msg_text)
                .bind(&now)
                .bind(&now)
                .execute(&state.db)
                .await
                {
                    executed_actions.push(json!({
                        "action": a_type,
                        "status": "failed",
                        "error": format!("the message row could not be written, so nothing was dispatched: {err}"),
                    }));
                    deferred += 1;
                    continue;
                }

                let outcome = if channel.eq_ignore_ascii_case("whatsapp") {
                    dispatch_whatsapp_text(state, &phone, msg_text, &msg_id).await
                } else {
                    json!({
                        "dispatched": false,
                        "reason": format!("No outbound dispatcher is ported for channel '{channel}'; message stored locally only"),
                    })
                };

                let dispatched_ok = outcome["dispatched"].as_bool().unwrap_or(false);
                let mut state_errors: Vec<String> = Vec::new();
                if dispatched_ok {
                    dispatched += 1;
                    if let Err(err) = sqlx::query(
                        "UPDATE messages SET status = 'sent', delivery_status = 'sent', provider_message_id = ?, sent_at = ? WHERE id = ?",
                    )
                    .bind(outcome["providerMessageId"].as_str().unwrap_or(""))
                    .bind(&now)
                    .bind(&msg_id)
                    .execute(&state.db)
                    .await
                    {
                        state_errors.push(format!("delivered but the message row still reads 'pending': {err}"));
                    }
                    if let Err(err) = sqlx::query("UPDATE conversations SET last_outbound_at = ? WHERE id = ?")
                        .bind(&now)
                        .bind(&conversation_id)
                        .execute(&state.db)
                        .await
                    {
                        state_errors.push(format!("last_outbound_at could not be recorded: {err}"));
                    }
                } else {
                    deferred += 1;
                    if let Err(err) = sqlx::query(
                        "UPDATE messages SET status = 'failed', delivery_status = 'failed' WHERE id = ?",
                    )
                    .bind(&msg_id)
                    .execute(&state.db)
                    .await
                    {
                        state_errors.push(format!("the message row could not be marked failed: {err}"));
                    }
                }

                executed_actions.push(json!({
                    "action": a_type,
                    "messageId": msg_id,
                    "message": msg_text,
                    "status": if dispatched_ok { "dispatched" } else { "stored_not_dispatched" },
                    "dispatch": outcome,
                    "stateWriteErrors": state_errors,
                }));
            }
            "add_private_note" => {
                if let Some(note) = action.get("note").and_then(|v| v.as_str()) {
                    let msg_id = Uuid::new_v4().to_string();
                    let res = sqlx::query(
                        "INSERT INTO messages (id, conversation_id, direction, channel, body, status, sent_at, created_at, is_private, sender, sender_name, provider) VALUES (?, ?, 'outbound', 'whatsapp', ?, 'internal', ?, ?, 1, ?, ?, 'macro_executor')",
                    )
                    .bind(&msg_id)
                    .bind(&conversation_id)
                    .bind(note)
                    .bind(&now)
                    .bind(&now)
                    .bind(&agent_name)
                    .bind(&agent_name)
                    .execute(&state.db)
                    .await;
                    match res {
                        Ok(_) => executed_actions.push(json!({
                            "action": a_type, "messageId": msg_id, "note": note, "status": "applied",
                        })),
                        Err(e) => {
                            deferred += 1;
                            executed_actions.push(json!({
                                "action": a_type, "status": "failed", "error": e.to_string(),
                            }));
                        }
                    }
                } else {
                    executed_actions.push(json!({
                        "action": a_type, "status": "failed", "error": "action has no 'note' text",
                    }));
                    deferred += 1;
                }
            }
            "send_csat_survey" => {
                // Only report a dispatch we can prove: the CSAT link/builder is
                // not ported in this build, so we record intent, not a send.
                attrs.insert("csat_requested_at".to_string(), json!(now.clone()));
                deferred += 1;
                executed_actions.push(json!({
                    "action": a_type,
                    "status": "deferred",
                    "reason": "CSAT survey dispatch requires the outbound survey builder (not ported in backend-rs); no survey was sent.",
                }));
            }
            "snooze_conversation" => {
                let hours = action.get("duration_hours").and_then(|v| v.as_i64()).unwrap_or(24);
                let snooze_until = Utc::now() + Duration::hours(hours);
                let snooze_str = snooze_until.to_rfc3339();
                let res = sqlx::query("UPDATE conversations SET snoozed_until = ? WHERE id = ?")
                    .bind(&snooze_str)
                    .bind(&conversation_id)
                    .execute(&state.db)
                    .await;
                match res {
                    Ok(_) => executed_actions.push(json!({
                        "action": a_type, "snoozedUntil": snooze_str, "status": "applied",
                    })),
                    Err(e) => {
                        deferred += 1;
                        executed_actions.push(json!({
                            "action": a_type, "status": "failed", "error": e.to_string(),
                        }));
                    }
                }
            }
            other => {
                deferred += 1;
                executed_actions.push(json!({
                    "action": other,
                    "status": "unsupported",
                    "reason": format!("Macro action type '{other}' has no executor in backend-rs and was not performed."),
                }));
            }
        }
    }

    attrs.insert("tags".to_string(), json!(current_tags));
    let execution_record = json!({
        "macro_id": m.id,
        "macro_name": m.name,
        "executed_at": now.clone(),
        "executed_by": agent_name,
        "action_count": actions.len(),
        "dispatched_count": dispatched,
        "deferred_count": deferred,
        "results": executed_actions,
    });
    attrs.insert(
        "last_macro_executed".to_string(),
        json!({
            "macro_id": m.id,
            "macro_name": m.name,
            "executed_at": now.clone(),
            "executed_by": agent_name,
        }),
    );

    let attrs_str = serde_json::to_string(&attrs).unwrap_or_else(|_| "{}".to_string());

    let update_res = sqlx::query(
        "UPDATE conversations SET state = ?, priority = ?, resolution_type = ?, custom_attributes = ?, team_id = ?, assigned_to_id = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&conversation.state)
    .bind(&conversation.priority)
    .bind(&conversation.resolution_type)
    .bind(&attrs_str)
    .bind(&conversation.team_id)
    .bind(&conversation.assigned_to_id)
    .bind(&now)
    .bind(&conversation_id)
    .execute(&state.db)
    .await;

    if let Err(e) = update_res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Macro actions ran but the conversation could not be saved: {e}") })),
        )
            .into_response();
    }

    // Persist a real, queryable execution record in the existing audit trail.
    let audit_id = Uuid::new_v4().to_string();
    let audit_details = serde_json::to_string(&execution_record).unwrap_or_else(|_| "{}".to_string());
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, actor_name, severity, compliance_category, description, details, created_at) VALUES (?, 'macro', ?, 'macro_executed', ?, ?, 'info', 'audit_trail', ?, ?, ?)",
    )
    .bind(&audit_id)
    .bind(&conversation_id)
    .bind(&agent_name)
    .bind(&agent_name)
    .bind(format!("Macro '{}' executed on conversation {}", m.name, conversation_id))
    .bind(&audit_details)
    .bind(&now)
    .execute(&state.db)
    .await;

    let all_applied = deferred == 0;
    (
        StatusCode::OK,
        Json(json!({
            "status": if all_applied { "ok" } else { "partial" },
            "macroId": m.id,
            "macroName": m.name,
            "conversationId": conversation_id,
            "executedActionsCount": executed_actions.len(),
            "dispatchedMessageCount": dispatched,
            "deferredActionCount": deferred,
            "executedActions": executed_actions,
            "conversationState": conversation.state.to_lowercase(),
            "conversationPriority": conversation.priority.as_deref(),
            "conversationTags": current_tags,
            "executionRecordId": audit_id,
            "executionRecordStored": audit_res.is_ok(),
            "executedAt": now,
            "note": if all_applied {
                "Every action performed a persisted change.".to_string()
            } else {
                format!("{deferred} action(s) were NOT performed for real; see the status/reason on each entry.")
            },
        })),
    )
        .into_response()
}

/// Real Meta Cloud API text dispatch. Reports the provider's own error text.
pub async fn dispatch_whatsapp_text(
    state: &AppState,
    phone: &str,
    body: &str,
    _local_msg_id: &str,
) -> serde_json::Value {
    let clean_phone = phone.replace('+', "").replace(' ', "").replace('-', "");
    if clean_phone.is_empty() {
        return json!({ "dispatched": false, "reason": "Conversation contact has no phone number to send to" });
    }

    let access_token = if !state.config.meta_whatsapp_access_token.is_empty() {
        state.config.meta_whatsapp_access_token.clone()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT meta_access_token FROM integration_settings WHERE meta_access_token IS NOT NULL AND meta_access_token != '' LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .unwrap_or_default()
    };
    let phone_number_id = if !state.config.meta_whatsapp_phone_number_id.is_empty() {
        state.config.meta_whatsapp_phone_number_id.clone()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT meta_phone_number_id FROM integration_settings WHERE meta_phone_number_id IS NOT NULL AND meta_phone_number_id != '' LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .unwrap_or_default()
    };

    let mut missing = Vec::new();
    if access_token.is_empty() {
        missing.push("META_WHATSAPP_ACCESS_TOKEN");
    }
    if phone_number_id.is_empty() {
        missing.push("META_WHATSAPP_PHONE_NUMBER_ID");
    }
    if !missing.is_empty() {
        return json!({
            "dispatched": false,
            "reason": format!("Outbound WhatsApp is not configured: {} missing", missing.join(", ")),
        });
    }

    let api_version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0"
    } else {
        &state.config.meta_whatsapp_api_version
    };
    let url = format!("https://graph.facebook.com/{api_version}/{phone_number_id}/messages");
    let payload = json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": clean_phone,
        "type": "text",
        "text": { "preview_url": false, "body": body }
    });

    match state
        .http_client
        .post(&url)
        .bearer_auth(&access_token)
        .json(&payload)
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status();
            if status.is_success() {
                match res.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let provider_id = json["messages"]
                            .as_array()
                            .and_then(|a| a.first())
                            .and_then(|m| m["id"].as_str())
                            .unwrap_or("")
                            .to_string();
                        json!({
                            "dispatched": true,
                            "providerMessageId": provider_id,
                            "httpStatus": status.as_u16(),
                        })
                    }
                    Err(e) => json!({
                        "dispatched": true,
                        "httpStatus": status.as_u16(),
                        "warning": format!("Provider accepted the message but the response could not be parsed: {e}"),
                    }),
                }
            } else {
                let text = res.text().await.unwrap_or_default();
                json!({
                    "dispatched": false,
                    "httpStatus": status.as_u16(),
                    "providerError": text,
                })
            }
        }
        Err(e) => json!({ "dispatched": false, "transportError": e.to_string() }),
    }
}

pub async fn execute_macro(
    State(state): State<AppState>,
    Path((conversation_id, macro_id)): Path<(String, String)>,
    Json(payload): Json<ExecuteMacroPayload>,
) -> impl IntoResponse {
    run_macro(
        &state,
        conversation_id,
        macro_id,
        payload.agent_name.unwrap_or_default(),
    )
    .await
}

#[derive(Deserialize)]
pub struct ExecuteMacroDirectPayload {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    pub agent_name: Option<String>,
}

/// POST /macros/{macro_id}/execute -- conversation_id supplied in the body.
pub async fn execute_macro_direct(
    State(state): State<AppState>,
    Path(macro_id): Path<String>,
    Json(payload): Json<ExecuteMacroDirectPayload>,
) -> impl IntoResponse {
    run_macro(
        &state,
        payload.conversation_id,
        macro_id,
        payload.agent_name.unwrap_or_default(),
    )
    .await
}

// ============================================================================
// SLA POLICIES & MONITORING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SlaPolicyRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i32>,
    pub resolution_time_threshold: Option<i32>,
    pub only_during_business_hours: Option<bool>,
    pub priorities: Option<String>,
    pub inbox_ids: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn serialize_sla_policy(p: &SlaPolicyRow) -> serde_json::Value {
    let prios: Vec<String> = p
        .priorities
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default();
    let inboxes: Vec<String> = p
        .inbox_ids
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default();

    json!({
        "id": p.id,
        "name": p.name,
        "description": p.description.as_deref().unwrap_or(""),
        "firstResponseTimeThreshold": p.first_response_time_threshold.unwrap_or(60),
        "first_response_time_threshold": p.first_response_time_threshold.unwrap_or(60),
        "resolutionTimeThreshold": p.resolution_time_threshold.unwrap_or(1440),
        "resolution_time_threshold": p.resolution_time_threshold.unwrap_or(1440),
        "onlyDuringBusinessHours": p.only_during_business_hours.unwrap_or(false),
        "only_during_business_hours": p.only_during_business_hours.unwrap_or(false),
        "priorities": prios,
        "inboxIds": inboxes,
        "inbox_ids": inboxes,
        "isActive": p.is_active.unwrap_or(true),
        "is_active": p.is_active.unwrap_or(true),
        "createdAt": p.created_at,
        "created_at": p.created_at,
        "updatedAt": p.updated_at,
        "updated_at": p.updated_at,
    })
}

pub async fn list_sla_policies(State(state): State<AppState>) -> impl IntoResponse {

    let policies = sqlx::query_as::<_, SlaPolicyRow>(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = policies.iter().map(serialize_sla_policy).collect();
    Json(items)
}

pub async fn get_sla_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let p = sqlx::query_as::<_, SlaPolicyRow>(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(item) = p {
        (StatusCode::OK, Json(serialize_sla_policy(&item))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "SLA policy not found" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct CreateSlaPayload {
    pub name: String,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i32>,
    #[serde(rename = "firstResponseTimeThreshold")]
    pub first_response_time_threshold_camel: Option<i32>,
    pub resolution_time_threshold: Option<i32>,
    #[serde(rename = "resolutionTimeThreshold")]
    pub resolution_time_threshold_camel: Option<i32>,
    pub only_during_business_hours: Option<bool>,
    #[serde(rename = "onlyDuringBusinessHours")]
    pub only_during_business_hours_camel: Option<bool>,
    pub priorities: Option<Vec<String>>,
    pub inbox_ids: Option<Vec<String>>,
    #[serde(rename = "inboxIds")]
    pub inbox_ids_camel: Option<Vec<String>>,
}

pub async fn create_sla_policy(
    State(state): State<AppState>,
    Json(payload): Json<CreateSlaPayload>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "SLA policy name cannot be empty" })),
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();
    let fr = payload
        .first_response_time_threshold
        .or(payload.first_response_time_threshold_camel)
        .unwrap_or(60);
    let res = payload
        .resolution_time_threshold
        .or(payload.resolution_time_threshold_camel)
        .unwrap_or(1440);
    let bh = payload
        .only_during_business_hours
        .or(payload.only_during_business_hours_camel)
        .unwrap_or(false);
    let priorities = payload.priorities.unwrap_or_else(|| vec!["medium".to_string()]);
    let prios_str = serde_json::to_string(&priorities).unwrap_or_else(|_| "[]".to_string());
    let inboxes = payload.inbox_ids.or(payload.inbox_ids_camel).unwrap_or_default();
    let inboxes_str = serde_json::to_string(&inboxes).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    // The policy object returned below is only real once the INSERT lands.
    let stored = sqlx::query(
        "INSERT INTO sla_policies (id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.description.as_deref().unwrap_or(""))
    .bind(fr)
    .bind(res)
    .bind(bh)
    .bind(&prios_str)
    .bind(&inboxes_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    match stored {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => return write_not_stored("INSERT INTO sla_policies", &id, report.rows_affected()),
        Err(err) => return db_error("the SLA policy could not be stored", err),
    }

    let p = SlaPolicyRow {
        id,
        name: payload.name.trim().to_string(),
        description: payload.description,
        first_response_time_threshold: Some(fr),
        resolution_time_threshold: Some(res),
        only_during_business_hours: Some(bh),
        priorities: Some(prios_str),
        inbox_ids: Some(inboxes_str),
        is_active: Some(true),
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    (StatusCode::OK, Json(serialize_sla_policy(&p))).into_response()
}

#[derive(Deserialize)]
pub struct UpdateSlaPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i32>,
    #[serde(rename = "firstResponseTimeThreshold")]
    pub first_response_time_threshold_camel: Option<i32>,
    pub resolution_time_threshold: Option<i32>,
    #[serde(rename = "resolutionTimeThreshold")]
    pub resolution_time_threshold_camel: Option<i32>,
    pub only_during_business_hours: Option<bool>,
    #[serde(rename = "onlyDuringBusinessHours")]
    pub only_during_business_hours_camel: Option<bool>,
    pub priorities: Option<Vec<String>>,
    pub inbox_ids: Option<Vec<String>>,
    #[serde(rename = "inboxIds")]
    pub inbox_ids_camel: Option<Vec<String>>,
    pub is_active: Option<bool>,
    #[serde(rename = "isActive")]
    pub is_active_camel: Option<bool>,
}

pub async fn update_sla_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSlaPayload>,
) -> impl IntoResponse {
    let p = sqlx::query_as::<_, SlaPolicyRow>(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    let mut current = match p {
        Ok(Some(item)) => item,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "SLA policy not found", "id": id })),
            )
                .into_response();
        }
        Err(err) => return db_error(&format!("the SLA policy '{id}' could not be read"), err),
    };

    if let Some(ref name) = payload.name {
        current.name = name.trim().to_string();
    }
    if let Some(ref desc) = payload.description {
        current.description = Some(desc.trim().to_string());
    }
    if let Some(fr) = payload.first_response_time_threshold.or(payload.first_response_time_threshold_camel) {
        current.first_response_time_threshold = Some(fr.max(1));
    }
    if let Some(res) = payload.resolution_time_threshold.or(payload.resolution_time_threshold_camel) {
        current.resolution_time_threshold = Some(res.max(1));
    }
    if let Some(bh) = payload.only_during_business_hours.or(payload.only_during_business_hours_camel) {
        current.only_during_business_hours = Some(bh);
    }
    if let Some(ref prios) = payload.priorities {
        current.priorities = Some(serde_json::to_string(prios).unwrap_or_else(|_| "[]".to_string()));
    }
    if let Some(ref inboxes) = payload.inbox_ids.or(payload.inbox_ids_camel) {
        current.inbox_ids = Some(serde_json::to_string(inboxes).unwrap_or_else(|_| "[]".to_string()));
    }
    if let Some(active) = payload.is_active.or(payload.is_active_camel) {
        current.is_active = Some(active);
    }
    let now = Utc::now().to_rfc3339();
    current.updated_at = Some(now.clone());

    // The policy returned below is the row as this handler believes it now is, so
    // the UPDATE has to be proven rather than assumed.
    let updated = sqlx::query(
        "UPDATE sla_policies SET name = ?, description = ?, first_response_time_threshold = ?, resolution_time_threshold = ?, only_during_business_hours = ?, priorities = ?, inbox_ids = ?, is_active = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&current.name)
    .bind(&current.description)
    .bind(current.first_response_time_threshold)
    .bind(current.resolution_time_threshold)
    .bind(current.only_during_business_hours)
    .bind(&current.priorities)
    .bind(&current.inbox_ids)
    .bind(current.is_active)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    match updated {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => {
            return write_not_found(
                "sla_policies",
                "name, description, thresholds, priorities, inbox_ids, is_active, updated_at",
                &id,
                report.rows_affected(),
            );
        }
        Err(err) => return db_error(&format!("the SLA policy '{id}' could not be updated"), err),
    }

    (StatusCode::OK, Json(serialize_sla_policy(&current))).into_response()
}

pub async fn delete_sla_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM sla_policies WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    let rows = res.map(|r| r.rows_affected()).unwrap_or(0);
    if rows == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "SLA policy not found", "deleted": false })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "deleted": true, "id": id })),
    )
        .into_response()
}

// ----------------------------------------------------------------------------
// SLA Evaluation Engine
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ComputedSlaResult {
    pub policy_id: String,
    pub policy_name: String,
    pub fr_threshold_minutes: i32,
    pub res_threshold_minutes: i32,
    pub target_fr_at: DateTime<Utc>,
    pub target_res_at: DateTime<Utc>,
    pub first_resp_at: Option<DateTime<Utc>>,
    pub first_resp_status: String,
    pub first_resp_breached: bool,
    pub resolution_at: Option<DateTime<Utc>>,
    pub resolution_status: String,
    pub resolution_breached: bool,
    pub is_breached: bool,
    pub nearing_breach: bool,
    pub minutes_to_fr_breach: i64,
    pub minutes_to_res_breach: i64,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ConversationRowForSla {
    id: String,
    contact_id: Option<String>,
    state: String,
    priority: Option<String>,
    inbox_id: Option<String>,
    custom_attributes: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

async fn evaluate_single_conversation_sla(
    db: &sqlx::PgPool,
    conv: &ConversationRowForSla,
    policies: &[SlaPolicyRow],
    target_policy_id: Option<&str>,
) -> (ComputedSlaResult, Vec<String>) {
    // 1. Select matching policy
    let matched_policy = if let Some(pid) = target_policy_id {
        policies.iter().find(|p| p.id == pid)
    } else {
        None
    };

    let policy = match matched_policy {
        Some(p) => p,
        None => {
            let prio = conv.priority.as_deref().unwrap_or("medium").to_lowercase();
            let mut by_prio = None;
            for p in policies {
                let prios: Vec<String> = p
                    .priorities
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                    .unwrap_or_default();
                if prios.iter().any(|pr| pr.to_lowercase() == prio) {
                    by_prio = Some(p);
                    break;
                }
            }

            if let Some(p) = by_prio {
                p
            } else if let Some(ref ibox) = conv.inbox_id {
                let mut by_inbox = None;
                for p in policies {
                    let inboxes: Vec<String> = p
                        .inbox_ids
                        .as_deref()
                        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                        .unwrap_or_default();
                    if inboxes.contains(ibox) {
                        by_inbox = Some(p);
                        break;
                    }
                }
                by_inbox.unwrap_or_else(|| policies.last().unwrap())
            } else {
                policies.last().unwrap()
            }
        }
    };

    let now = Utc::now();
    let conv_created = conv
        .created_at
        .as_deref()
        .and_then(parse_dt)
        .unwrap_or(now);

    let fr_threshold = policy.first_response_time_threshold.unwrap_or(60).max(1);
    let res_threshold = policy.resolution_time_threshold.unwrap_or(1440).max(1);

    let target_fr_at = conv_created + Duration::minutes(fr_threshold as i64);
    let target_res_at = conv_created + Duration::minutes(res_threshold as i64);

    // 2. First response check from messages table
    #[derive(sqlx::FromRow)]
    struct MsgRow {
        sent_at: Option<String>,
        created_at: Option<String>,
    }

    let first_msg = sqlx::query_as::<_, MsgRow>(
        "SELECT sent_at, created_at FROM messages WHERE conversation_id = ? AND LOWER(direction) = 'outbound' AND (is_private IS NULL OR is_private = 0) ORDER BY COALESCE(sent_at, created_at) ASC LIMIT 1",
    )
    .bind(&conv.id)
    .fetch_optional(db)
    .await
    .unwrap_or(None);

    let (first_resp_at, first_resp_breached, first_resp_status, minutes_to_fr_breach) =
        if let Some(m) = first_msg {
            let msg_dt = m
                .sent_at
                .as_deref()
                .or(m.created_at.as_deref())
                .and_then(parse_dt)
                .unwrap_or(conv_created);
            let breached = msg_dt > target_fr_at;
            (
                Some(msg_dt),
                breached,
                if breached { "breached" } else { "fulfilled" }.to_string(),
                0i64,
            )
        } else {
            let breached = now > target_fr_at;
            let mins = (target_fr_at - now).num_minutes();
            (
                None,
                breached,
                if breached { "breached" } else { "active" }.to_string(),
                mins,
            )
        };

    // 3. Resolution check
    let state_lower = conv.state.to_lowercase();
    let is_resolved = matches!(
        state_lower.as_str(),
        "closed_won" | "closed_lost" | "resolved" | "closed"
    );

    let (resolution_at, resolution_breached, resolution_status, minutes_to_res_breach) =
        if is_resolved {
            let res_dt = conv
                .updated_at
                .as_deref()
                .and_then(parse_dt)
                .unwrap_or(now);
            let breached = res_dt > target_res_at;
            (
                Some(res_dt),
                breached,
                if breached { "breached" } else { "fulfilled" }.to_string(),
                0i64,
            )
        } else {
            let breached = now > target_res_at;
            let mins = (target_res_at - now).num_minutes();
            (
                None,
                breached,
                if breached { "breached" } else { "active" }.to_string(),
                mins,
            )
        };

    let is_breached = first_resp_breached || resolution_breached;
    let nearing_breach = !is_breached
        && ((0 <= minutes_to_fr_breach && minutes_to_fr_breach <= 15 && first_resp_status == "active")
            || (0 <= minutes_to_res_breach && minutes_to_res_breach <= 60 && resolution_status == "active"));

    // 4. Update tags in custom_attributes
    let attrs: serde_json::Map<String, serde_json::Value> = conv
        .custom_attributes
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v: serde_json::Value| v.as_object().cloned())
        .unwrap_or_default();

    let mut current_tags: Vec<String> = attrs
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    if is_breached {
        if !current_tags.contains(&"sla-breached".to_string()) {
            current_tags.push("sla-breached".to_string());
        }
        current_tags.retain(|t| t != "sla-warning");
    } else if nearing_breach {
        if !current_tags.contains(&"sla-warning".to_string()) {
            current_tags.push("sla-warning".to_string());
        }
        current_tags.retain(|t| t != "sla-breached");
    } else {
        current_tags.retain(|t| t != "sla-breached" && t != "sla-warning");
    }

    let result = ComputedSlaResult {
        policy_id: policy.id.clone(),
        policy_name: policy.name.clone(),
        fr_threshold_minutes: fr_threshold,
        res_threshold_minutes: res_threshold,
        target_fr_at,
        target_res_at,
        first_resp_at,
        first_resp_status,
        first_resp_breached,
        resolution_at,
        resolution_status,
        resolution_breached,
        is_breached,
        nearing_breach,
        minutes_to_fr_breach,
        minutes_to_res_breach,
        evaluated_at: now,
    };

    (result, current_tags)
}

#[derive(Deserialize)]
pub struct EvaluateSlaQuery {
    pub policy_id: Option<String>,
}

pub async fn evaluate_conversation_sla(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EvaluateSlaQuery>,
) -> impl IntoResponse {

    let conv = sqlx::query_as::<_, ConversationRowForSla>(
        "SELECT id, contact_id, state, priority, inbox_id, custom_attributes, created_at, updated_at FROM conversations WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    let conversation = match conv {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Conversation {} not found", id) })),
            )
                .into_response();
        }
        Err(err) => return db_error(&format!("conversation '{id}' could not be read"), err),
    };

    let policies = sqlx::query_as::<_, SlaPolicyRow>(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies WHERE is_active = 1 OR is_active IS NULL ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if policies.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "No active SLA policies configured" })),
        )
            .into_response();
    }

    let (sla, tags) = evaluate_single_conversation_sla(
        &state.db,
        &conversation,
        &policies,
        query.policy_id.as_deref(),
    )
    .await;

    let sla_json = json!({
        "policyId": sla.policy_id,
        "policyName": sla.policy_name,
        "firstResponseThresholdMinutes": sla.fr_threshold_minutes,
        "resolutionThresholdMinutes": sla.res_threshold_minutes,
        "firstResponseTargetAt": sla.target_fr_at.to_rfc3339(),
        "resolutionTargetAt": sla.target_res_at.to_rfc3339(),
        "firstResponseAt": sla.first_resp_at.map(|t| t.to_rfc3339()),
        "firstResponseStatus": sla.first_resp_status,
        "firstResponseBreached": sla.first_resp_breached,
        "resolutionAt": sla.resolution_at.map(|t| t.to_rfc3339()),
        "resolutionStatus": sla.resolution_status,
        "resolutionBreached": sla.resolution_breached,
        "isBreached": sla.is_breached,
        "nearingBreach": sla.nearing_breach,
        "minutesToFirstResponseBreach": sla.minutes_to_fr_breach,
        "minutesToResolutionBreach": sla.minutes_to_res_breach,
        "evaluatedAt": sla.evaluated_at.to_rfc3339(),
    });

    // Persist into conversation custom_attributes
    let mut attrs: serde_json::Map<String, serde_json::Value> = conversation
        .custom_attributes
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v: serde_json::Value| v.as_object().cloned())
        .unwrap_or_default();

    attrs.insert("sla".to_string(), sla_json.clone());
    attrs.insert("tags".to_string(), json!(tags));

    let attrs_str = serde_json::to_string(&attrs).unwrap_or_else(|_| "{}".to_string());
    let stored = sqlx::query("UPDATE conversations SET custom_attributes = ? WHERE id = ?")
        .bind(&attrs_str)
        .bind(&id)
        .execute(&state.db)
        .await;

    match stored {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => {
            return write_not_found(
                "conversations",
                "custom_attributes (the SLA snapshot)",
                &id,
                report.rows_affected(),
            );
        }
        Err(err) => {
            // The computed SLA is still returned, but the answer says plainly that
            // nothing was stored, so a client cannot show it as a recorded state.
            return refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("the SLA was evaluated but could not be stored on conversation '{id}': {err}"),
                json!({
                    "conversationId": id,
                    "databaseError": true,
                    "sla": sla_json,
                    "tags": tags,
                }),
            );
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "conversationId": conversation.id,
            "sla": sla_json,
            "tags": tags,
            "persisted": true,
        })),
    )
        .into_response()
}

pub async fn get_sla_metrics(State(state): State<AppState>) -> impl IntoResponse {

    let policies = sqlx::query_as::<_, SlaPolicyRow>(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let convs = sqlx::query_as::<_, ConversationRowForSla>(
        "SELECT id, contact_id, state, priority, inbox_id, custom_attributes, created_at, updated_at FROM conversations",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut total_tracked = 0;
    let mut fr_fulfilled = 0;
    let mut fr_breached = 0;
    let mut res_fulfilled = 0;
    let mut res_breached = 0;
    let mut active_breaches = 0;
    let mut nearing_breaches = 0;

    let mut policy_stats: HashMap<String, (String, i32, i32, i32)> = HashMap::new();
    for p in &policies {
        policy_stats.insert(p.id.clone(), (p.name.clone(), 0, 0, 0));
    }

    for c in &convs {
        let (sla, _) = evaluate_single_conversation_sla(&state.db, c, &policies, None).await;
        total_tracked += 1;

        if sla.first_resp_status == "fulfilled" {
            fr_fulfilled += 1;
        } else if sla.first_resp_status == "breached" {
            fr_breached += 1;
        }

        if sla.resolution_status == "fulfilled" {
            res_fulfilled += 1;
        } else if sla.resolution_status == "breached" {
            res_breached += 1;
        }

        if sla.is_breached {
            active_breaches += 1;
            if let Some(entry) = policy_stats.get_mut(&sla.policy_id) {
                entry.1 += 1; // tracked
                entry.2 += 1; // breached
            }
        } else {
            if let Some(entry) = policy_stats.get_mut(&sla.policy_id) {
                entry.1 += 1; // tracked
                entry.3 += 1; // fulfilled
            }
        }

        if sla.nearing_breach {
            nearing_breaches += 1;
        }
    }

    let fr_evaluated = fr_fulfilled + fr_breached;
    let res_evaluated = res_fulfilled + res_breached;

    let fr_rate = if fr_evaluated > 0 {
        ((fr_fulfilled as f64 / fr_evaluated as f64) * 1000.0).round() / 10.0
    } else {
        100.0
    };

    let res_rate = if res_evaluated > 0 {
        ((res_fulfilled as f64 / res_evaluated as f64) * 1000.0).round() / 10.0
    } else {
        100.0
    };

    let breakdown: Vec<serde_json::Value> = policy_stats
        .into_iter()
        .map(|(p_id, (p_name, tracked, breached, fulfilled))| {
            json!({
                "policyId": p_id,
                "policyName": p_name,
                "trackedCount": tracked,
                "breachedCount": breached,
                "fulfilledCount": fulfilled,
            })
        })
        .collect();

    Json(json!({
        "totalTracked": total_tracked,
        "totalMonitored": total_tracked,
        "firstResponseFulfillmentRate": fr_rate,
        "firstResponseMetRate": fr_rate,
        "firstResponseFulfilledCount": fr_fulfilled,
        "firstResponseBreachedCount": fr_breached,
        "resolutionFulfillmentRate": res_rate,
        "resolutionMetRate": res_rate,
        "resolutionFulfilledCount": res_fulfilled,
        "resolutionBreachedCount": res_breached,
        "activeBreachesCount": active_breaches,
        "breachedCount": active_breaches,
        "nearingBreachesCount": nearing_breaches,
        "policyBreakdown": breakdown,
    }))
}

pub async fn get_sla_breaches(State(state): State<AppState>) -> impl IntoResponse {

    let policies = sqlx::query_as::<_, SlaPolicyRow>(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let convs = sqlx::query_as::<_, ConversationRowForSla>(
        "SELECT id, contact_id, state, priority, inbox_id, custom_attributes, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut results = Vec::new();

    for c in &convs {
        let (sla, _) = evaluate_single_conversation_sla(&state.db, c, &policies, None).await;

        if sla.is_breached || sla.nearing_breach {
            #[derive(sqlx::FromRow)]
            struct ContactInfo {
                person_name: Option<String>,
                value: String,
                channel: String,
            }

            let contact = if let Some(ref cid) = c.contact_id {
                sqlx::query_as::<_, ContactInfo>(
                    "SELECT person_name, value, channel FROM contacts WHERE id = ? LIMIT 1",
                )
                .bind(cid)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None)
            } else {
                None
            };

            let contact_name = contact
                .as_ref()
                .and_then(|ct| ct.person_name.clone())
                .unwrap_or_else(|| "Anonymous Visitor".to_string());
            let contact_id_val = contact
                .as_ref()
                .map(|ct| ct.value.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let channel_val = contact
                .as_ref()
                .map(|ct| ct.channel.clone())
                .unwrap_or_else(|| "whatsapp".to_string());

            results.push(json!({
                "conversationId": c.id,
                "contactName": contact_name,
                "contactIdentifier": contact_id_val,
                "priority": c.priority.as_deref().unwrap_or("medium"),
                "channel": channel_val,
                "state": c.state,
                "policyName": sla.policy_name,
                "firstResponseStatus": sla.first_resp_status,
                "firstResponseTargetAt": sla.target_fr_at.to_rfc3339(),
                "resolutionStatus": sla.resolution_status,
                "resolutionTargetAt": sla.target_res_at.to_rfc3339(),
                "isBreached": sla.is_breached,
                "nearingBreach": sla.nearing_breach,
                "minutesToFirstResponseBreach": sla.minutes_to_fr_breach,
                "minutesToResolutionBreach": sla.minutes_to_res_breach,
                "updatedAt": c.updated_at,
            }));
        }
    }

    Json(results)
}
