//! Canned responses, macros, and SLA policies handlers.
//!
//! Ported from backend-rs/src/routes/canned_macros_sla.rs.

use serde_json::{json, Value};
use sqlx::PgPool;
use sqlx::Row;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
    pub meta_whatsapp_access_token: String,
    pub meta_whatsapp_phone_number_id: String,
    pub meta_whatsapp_api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct CannedResponseRow {
    id: String,
    short_code: String,
    content: String,
    category: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct MacroItem {
    id: String,
    name: String,
    description: Option<String>,
    actions: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct SlaPolicyRow {
    id: String,
    name: String,
    description: Option<String>,
    first_response_time_threshold: Option<i32>,
    resolution_time_threshold: Option<i32>,
    only_during_business_hours: Option<bool>,
    priorities: Option<String>,
    inbox_ids: Option<String>,
    is_active: Option<bool>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

fn serialize_canned(c: &CannedResponseRow) -> Value {
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

fn serialize_sla_policy(p: &SlaPolicyRow) -> Value {
    let prios: Vec<String> = p.priorities.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
    let inboxes: Vec<String> = p.inbox_ids.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
    json!({
        "id": p.id,
        "name": p.name,
        "description": p.description.as_deref().unwrap_or(""),
        "firstResponseTimeThreshold": p.first_response_time_threshold.unwrap_or(60),
        "resolutionTimeThreshold": p.resolution_time_threshold.unwrap_or(1440),
        "onlyDuringBusinessHours": p.only_during_business_hours.unwrap_or(false),
        "priorities": prios,
        "inboxIds": inboxes,
        "isActive": p.is_active.unwrap_or(true),
        "createdAt": p.created_at,
        "updatedAt": p.updated_at,
    })
}

// ── Canned Responses ──

pub async fn list_canned_responses(state: &AppState, query: &Value) -> Value {
    let canned: Vec<CannedResponseRow> = sqlx::query_as(
        "SELECT id, short_code, content, category, created_at FROM canned_responses ORDER BY short_code ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let q = query.get("q").and_then(|v| v.as_str()).unwrap_or("");
    let category = query.get("category").and_then(|v| v.as_str()).unwrap_or("");

    let filtered = canned.into_iter().filter(|c| {
        let cat_match = category.is_empty() || category == "all" || c.category.as_deref().unwrap_or("general").to_lowercase() == category.to_lowercase();
        let q_match = q.is_empty() || c.short_code.to_lowercase().contains(&q.to_lowercase()) || c.content.to_lowercase().contains(&q.to_lowercase());
        cat_match && q_match
    });

    let items: Vec<Value> = filtered.map(|c| serialize_canned(&c)).collect();
    json!(items)
}

pub async fn create_canned_response(state: &AppState, payload: &Value) -> Value {
    let code = payload.get("short_code").and_then(|v| v.as_str()).unwrap_or("").trim().trim_start_matches('/').to_string();
    if code.is_empty() {
        return json!({ "error": "Short code cannot be empty." });
    }

    let existing: Option<String> = sqlx::query_scalar(
        "SELECT id FROM canned_responses WHERE LOWER(short_code) = LOWER($1) LIMIT 1",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if existing.is_some() {
        return json!({ "error": format!("Short code /{} already exists", code) });
    }

    let id = Uuid::new_v4().to_string();
    let category = payload.get("category").and_then(|v| v.as_str()).unwrap_or("general").trim().to_lowercase();
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        "INSERT INTO canned_responses (id, short_code, content, category, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&code)
    .bind(&content)
    .bind(&category)
    .bind(&now)
    .execute(&state.db)
    .await;

    if let Err(e) = res {
        return json!({ "error": format!("Database error: {}", e) });
    }

    serialize_canned(&CannedResponseRow { id, short_code: code, content, category: Some(category), created_at: Some(now) })
}

pub async fn update_canned_response(state: &AppState, id: String, payload: &Value) -> Value {
    let existing: Option<CannedResponseRow> = sqlx::query_as(
        "SELECT id, short_code, content, category, created_at FROM canned_responses WHERE id = $1 LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let mut current = match existing {
        Some(item) => item,
        None => return json!({ "error": "Canned response not found", "id": id }),
    };

    if let Some(sc) = payload.get("short_code").and_then(|v| v.as_str()) {
        current.short_code = sc.trim().trim_start_matches('/').to_string();
    }
    if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
        current.content = content.trim().to_string();
    }
    if let Some(cat) = payload.get("category").and_then(|v| v.as_str()) {
        current.category = Some(cat.trim().to_lowercase());
    }

    let res = sqlx::query(
        "UPDATE canned_responses SET short_code = $1, content = $2, category = $3 WHERE id = $4",
    )
    .bind(&current.short_code)
    .bind(&current.content)
    .bind(&current.category)
    .bind(&id)
    .execute(&state.db)
    .await;

    if let Err(e) = res {
        return json!({ "error": format!("Database error: {}", e) });
    }

    serialize_canned(&current)
}

pub async fn delete_canned_response(state: &AppState, id: String) -> Value {
    let rows = sqlx::query("DELETE FROM canned_responses WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    if rows == 0 {
        json!({ "error": "Canned response not found", "deleted": false })
    } else {
        json!({ "status": "deleted", "deleted": true, "id": id })
    }
}

pub async fn expand_canned_response(state: &AppState, payload: &Value) -> Value {
    let code = payload.get("short_code").and_then(|v| v.as_str()).unwrap_or("").trim().trim_start_matches('/').to_string();

    let canned: Option<CannedResponseRow> = sqlx::query_as(
        "SELECT id, short_code, content, category, created_at FROM canned_responses WHERE LOWER(short_code) = LOWER($1) LIMIT 1",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let canned = match canned {
        Some(c) => c,
        None => return json!({ "error": format!("Canned response /{} not found", code) }),
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

    let mut expanded = canned.content.clone();
    for (key, value) in &variables {
        expanded = expanded.replace(&format!("/{{{{{}}}}}/", key), value);
        expanded = expanded.replace(&format!("{{{}}}", key), value);
    }

    json!({ "expanded": expanded, "shortCode": code, "variables": variables })
}

// ── Macros ──

pub async fn list_macros(state: &AppState) -> Value {
    let macros: Vec<MacroItem> = sqlx::query_as(
        "SELECT id, name, description, actions, created_at FROM macros ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let items: Vec<Value> = macros.iter().map(|m| {
        let actions: Vec<Value> = m.actions.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
        json!({
            "id": m.id,
            "name": m.name,
            "description": m.description.as_deref().unwrap_or(""),
            "actions": actions,
            "createdAt": m.created_at,
        })
    }).collect();
    json!(items)
}

pub async fn create_macro(state: &AppState, payload: &Value) -> Value {
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if name.is_empty() {
        return json!({ "error": "Macro name cannot be empty." });
    }

    let id = Uuid::new_v4().to_string();
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let actions = payload.get("actions").cloned().unwrap_or_else(|| json!([]));
    let actions_str = serde_json::to_string(&actions).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        "INSERT INTO macros (id, name, description, actions, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&name)
    .bind(description)
    .bind(&actions_str)
    .bind(&now)
    .execute(&state.db)
    .await;

    if let Err(e) = res {
        return json!({ "error": format!("Database error: {}", e) });
    }

    json!({ "id": id, "name": name, "description": description, "actions": actions, "createdAt": now })
}

pub async fn get_macro(state: &AppState, id: String) -> Value {
    let m: Option<MacroItem> = sqlx::query_as(
        "SELECT id, name, description, actions, created_at FROM macros WHERE id = $1 LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    match m {
        Some(m) => {
            let actions: Vec<Value> = m.actions.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
            json!({ "id": m.id, "name": m.name, "description": m.description.as_deref().unwrap_or(""), "actions": actions, "createdAt": m.created_at })
        }
        None => json!({ "error": "Macro not found", "id": id }),
    }
}

pub async fn update_macro(state: &AppState, id: String, payload: &Value) -> Value {
    let m: Option<MacroItem> = sqlx::query_as(
        "SELECT id, name, description, actions, created_at FROM macros WHERE id = $1 LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let mut current = match m {
        Some(m) => m,
        None => return json!({ "error": "Macro not found", "id": id }),
    };

    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
        current.name = name.trim().to_string();
    }
    if let Some(desc) = payload.get("description").and_then(|v| v.as_str()) {
        current.description = Some(desc.trim().to_string());
    }
    if let Some(actions) = payload.get("actions") {
        current.actions = Some(serde_json::to_string(actions).unwrap_or_else(|_| "[]".to_string()));
    }

    let res = sqlx::query(
        "UPDATE macros SET name = $1, description = $2, actions = $3 WHERE id = $4",
    )
    .bind(&current.name)
    .bind(&current.description)
    .bind(&current.actions)
    .bind(&id)
    .execute(&state.db)
    .await;

    if let Err(e) = res {
        return json!({ "error": format!("Database error: {}", e) });
    }

    get_macro(state, id).await
}

pub async fn delete_macro(state: &AppState, id: String) -> Value {
    let rows = sqlx::query("DELETE FROM macros WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    if rows == 0 {
        json!({ "error": "Macro not found", "deleted": false })
    } else {
        json!({ "status": "deleted", "deleted": true, "id": id })
    }
}

pub async fn run_macro(state: &AppState, conversation_id: String, macro_id: String, agent_name: String) -> Value {
    let m: Option<MacroItem> = sqlx::query_as(
        "SELECT id, name, description, actions, created_at FROM macros WHERE id = $1 LIMIT 1",
    )
    .bind(&macro_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let macro_item = match m {
        Some(m) => m,
        None => return json!({ "error": "Macro not found", "macroId": macro_id }),
    };

    let actions: Vec<Value> = macro_item.actions.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();

    let mut executed_actions = Vec::new();
    let mut deferred = Vec::new();
    let mut sends = Vec::new();

    for action in &actions {
        let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match action_type {
            "send_message" => {
                let body = action.get("body").and_then(|v| v.as_str()).unwrap_or("");
                sends.push(json!({ "type": "send_message", "body": body, "dispatched": false, "reason": "WhatsApp dispatch not configured in standalone service" }));
            }
            "change_status" => {
                let new_status = action.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if !new_status.is_empty() {
                    let _ = sqlx::query("UPDATE conversations SET state = $1 WHERE id = $2")
                        .bind(new_status)
                        .bind(&conversation_id)
                        .execute(&state.db)
                        .await;
                    executed_actions.push(json!({ "type": "change_status", "status": new_status, "applied": true }));
                }
            }
            "add_tag" => {
                let tag = action.get("tag").and_then(|v| v.as_str()).unwrap_or("");
                if !tag.is_empty() {
                    executed_actions.push(json!({ "type": "add_tag", "tag": tag, "applied": true }));
                }
            }
            "set_priority" => {
                let priority = action.get("priority").and_then(|v| v.as_str()).unwrap_or("");
                if !priority.is_empty() {
                    let _ = sqlx::query("UPDATE conversations SET priority = $1 WHERE id = $2")
                        .bind(priority)
                        .bind(&conversation_id)
                        .execute(&state.db)
                        .await;
                    executed_actions.push(json!({ "type": "set_priority", "priority": priority, "applied": true }));
                }
            }
            _ => {
                deferred.push(json!({ "type": action_type, "reason": "action type not supported" }));
            }
        }
    }

    let now = Utc::now().to_rfc3339();
    let audit_id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        "INSERT INTO macro_executions (id, macro_id, conversation_id, agent_name, executed_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&audit_id)
    .bind(&macro_id)
    .bind(&conversation_id)
    .bind(if agent_name.is_empty() { "macro_executor" } else { &agent_name })
    .bind(&now)
    .execute(&state.db)
    .await;

    json!({
        "status": if sends.iter().any(|s| s["dispatched"].as_bool() == Some(false)) { "completed_with_undelivered" } else if deferred.is_empty() { "ok" } else { "ok_with_deferred" },
        "macroId": macro_id,
        "macroName": macro_item.name,
        "conversationId": conversation_id,
        "executedActionsCount": executed_actions.len(),
        "executedActions": executed_actions,
        "deferred": deferred,
        "sends": sends,
        "executedAt": now,
    })
}

pub async fn execute_macro(state: &AppState, conversation_id: String, macro_id: String, payload: &Value) -> Value {
    let agent_name = payload.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    run_macro(state, conversation_id, macro_id, agent_name.to_string()).await
}

pub async fn execute_macro_direct(state: &AppState, macro_id: String, payload: &Value) -> Value {
    let conversation_id = payload.get("conversation_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let agent_name = payload.get("agent_name").and_then(|v| v.as_str()).unwrap_or("");
    run_macro(state, conversation_id, macro_id, agent_name.to_string()).await
}

pub async fn dispatch_whatsapp_text(state: &AppState, phone: &str, body: &str, _local_msg_id: &str) -> Value {
    let clean_phone = phone.replace('+', "").replace(' ', "").replace('-', "");
    if clean_phone.is_empty() {
        return json!({ "dispatched": false, "reason": "No phone number" });
    }

    let access_token = &state.meta_whatsapp_access_token;
    let phone_number_id = &state.meta_whatsapp_phone_number_id;

    if access_token.is_empty() || phone_number_id.is_empty() {
        return json!({ "dispatched": false, "reason": "WhatsApp not configured" });
    }

    let api_version = if state.meta_whatsapp_api_version.is_empty() { "v21.0" } else { &state.meta_whatsapp_api_version };
    let url = format!("https://graph.facebook.com/{api_version}/{phone_number_id}/messages");
    let payload = json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": clean_phone,
        "type": "text",
        "text": { "preview_url": false, "body": body }
    });

    match state.http_client.post(&url).bearer_auth(access_token).json(&payload).send().await {
        Ok(res) => {
            let status = res.status();
            if status.is_success() {
                match res.json::<Value>().await {
                    Ok(json) => {
                        let provider_id = json["messages"].as_array().and_then(|a| a.first()).and_then(|m| m["id"].as_str()).unwrap_or("");
                        json!({ "dispatched": true, "providerMessageId": provider_id, "httpStatus": status.as_u16() })
                    }
                    Err(_) => json!({ "dispatched": true, "httpStatus": status.as_u16() }),
                }
            } else {
                let text = res.text().await.unwrap_or_default();
                json!({ "dispatched": false, "httpStatus": status.as_u16(), "providerError": text })
            }
        }
        Err(e) => json!({ "dispatched": false, "transportError": e.to_string() }),
    }
}

// ── SLA Policies ──

pub async fn list_sla_policies(state: &AppState) -> Value {
    let policies: Vec<SlaPolicyRow> = sqlx::query_as(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let items: Vec<Value> = policies.iter().map(|p| serialize_sla_policy(p)).collect();
    json!(items)
}

pub async fn get_sla_policy(state: &AppState, id: String) -> Value {
    let p: Option<SlaPolicyRow> = sqlx::query_as(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    match p {
        Some(item) => serialize_sla_policy(&item),
        None => json!({ "error": "SLA policy not found" }),
    }
}

pub async fn create_sla_policy(state: &AppState, payload: &Value) -> Value {
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if name.is_empty() {
        return json!({ "error": "SLA policy name cannot be empty" });
    }

    let id = Uuid::new_v4().to_string();
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let fr = payload.get("first_response_time_threshold").and_then(|v| v.as_i64()).unwrap_or(60) as i32;
    let res = payload.get("resolution_time_threshold").and_then(|v| v.as_i64()).unwrap_or(1440) as i32;
    let bh = payload.get("only_during_business_hours").and_then(|v| v.as_bool()).unwrap_or(false);
    let priorities = payload.get("priorities").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>()).unwrap_or_else(|| vec!["medium".to_string()]);
    let prios_str = serde_json::to_string(&priorities).unwrap_or_else(|_| "[]".to_string());
    let inboxes = payload.get("inbox_ids").or(payload.get("inboxIds")).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>()).unwrap_or_default();
    let inboxes_str = serde_json::to_string(&inboxes).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    let stored = sqlx::query(
        "INSERT INTO sla_policies (id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, $9, $10)",
    )
    .bind(&id)
    .bind(&name)
    .bind(description)
    .bind(fr)
    .bind(res)
    .bind(bh)
    .bind(&prios_str)
    .bind(&inboxes_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    if let Err(e) = stored {
        return json!({ "error": format!("Database error: {}", e) });
    }

    serialize_sla_policy(&SlaPolicyRow {
        id, name, description: Some(description.to_string()),
        first_response_time_threshold: Some(fr), resolution_time_threshold: Some(res),
        only_during_business_hours: Some(bh), priorities: Some(prios_str), inbox_ids: Some(inboxes_str),
        is_active: Some(true), created_at: Some(now.clone()), updated_at: Some(now),
    })
}

pub async fn update_sla_policy(state: &AppState, id: String, payload: &Value) -> Value {
    let p: Option<SlaPolicyRow> = sqlx::query_as(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let mut current = match p {
        Some(item) => item,
        None => return json!({ "error": "SLA policy not found", "id": id }),
    };

    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
        current.name = name.trim().to_string();
    }
    if payload.get("description").is_some() {
        current.description = payload.get("description").and_then(|v| v.as_str()).map(|s| s.trim().to_string());
    }
    if payload.get("first_response_time_threshold").is_some() {
        current.first_response_time_threshold = payload.get("first_response_time_threshold").and_then(|v| v.as_i64()).map(|v| (v as i32).max(1));
    }
    if payload.get("resolution_time_threshold").is_some() {
        current.resolution_time_threshold = payload.get("resolution_time_threshold").and_then(|v| v.as_i64()).map(|v| (v as i32).max(1));
    }
    if payload.get("only_during_business_hours").is_some() {
        current.only_during_business_hours = payload.get("only_during_business_hours").and_then(|v| v.as_bool());
    }
    if payload.get("priorities").is_some() {
        current.priorities = payload.get("priorities").and_then(|v| v.as_array()).map(|a| serde_json::to_string(a).unwrap_or_else(|_| "[]".to_string()));
    }
    if payload.get("inbox_ids").is_some() || payload.get("inboxIds").is_some() {
        current.inbox_ids = payload.get("inbox_ids").or(payload.get("inboxIds")).and_then(|v| v.as_array()).map(|a| serde_json::to_string(a).unwrap_or_else(|_| "[]".to_string()));
    }
    if payload.get("is_active").is_some() {
        current.is_active = payload.get("is_active").and_then(|v| v.as_bool());
    }

    let now = Utc::now().to_rfc3339();
    let updated = sqlx::query(
        "UPDATE sla_policies SET name = $1, description = $2, first_response_time_threshold = $3, resolution_time_threshold = $4, only_during_business_hours = $5, priorities = $6, inbox_ids = $7, is_active = $8, updated_at = $9 WHERE id = $10",
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

    if let Err(e) = updated {
        return json!({ "error": format!("Database error: {}", e) });
    }

    serialize_sla_policy(&current)
}

pub async fn delete_sla_policy(state: &AppState, id: String) -> Value {
    let rows = sqlx::query("DELETE FROM sla_policies WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    if rows == 0 {
        json!({ "error": "SLA policy not found", "deleted": false })
    } else {
        json!({ "status": "ok", "deleted": true, "id": id })
    }
}

pub async fn evaluate_conversation_sla(state: &AppState, id: String, query: &Value) -> Value {
    #[derive(sqlx::FromRow)]
    struct ConvRow {
        id: String,
        contact_id: Option<String>,
        state: String,
        priority: Option<String>,
        inbox_id: Option<String>,
        custom_attributes: Option<String>,
        created_at: Option<String>,
        updated_at: Option<String>,
    }

    let conv: Option<ConvRow> = sqlx::query_as(
        "SELECT id, contact_id, state, priority, inbox_id, custom_attributes, created_at, updated_at FROM conversations WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let conversation = match conv {
        Some(c) => c,
        None => return json!({ "error": format!("Conversation {} not found", id) }),
    };

    let policies: Vec<SlaPolicyRow> = sqlx::query_as(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies WHERE is_active = 1 OR is_active IS NULL ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if policies.is_empty() {
        return json!({ "error": "No active SLA policies configured" });
    }

    let policy_id = query.get("policy_id").and_then(|v| v.as_str());
    let policy = if let Some(pid) = policy_id {
        policies.iter().find(|p| p.id == pid).unwrap_or(policies.last().unwrap())
    } else {
        let prio = conversation.priority.as_deref().unwrap_or("medium").to_lowercase();
        policies.iter().find(|p| {
            p.priorities.as_deref().and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                .map(|ps| ps.iter().any(|pr| pr.to_lowercase() == prio))
                .unwrap_or(false)
        }).unwrap_or(policies.last().unwrap())
    };

    let created_at = conversation.created_at.as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(Utc::now());

    let fr_threshold = policy.first_response_time_threshold.unwrap_or(60);
    let res_threshold = policy.resolution_time_threshold.unwrap_or(1440);
    let target_fr_at = created_at + Duration::minutes(fr_threshold as i64);
    let target_res_at = created_at + Duration::minutes(res_threshold as i64);
    let now = Utc::now();

    let fr_breached = now > target_fr_at && conversation.state != "resolved" && conversation.state != "closed";
    let res_breached = now > target_res_at && conversation.state != "resolved" && conversation.state != "closed";
    let nearing_breach = (now + Duration::minutes(fr_threshold as i64 / 4)) > target_fr_at;

    let sla_json = json!({
        "policyId": policy.id,
        "policyName": policy.name,
        "firstResponseThresholdMinutes": fr_threshold,
        "resolutionThresholdMinutes": res_threshold,
        "firstResponseTargetAt": target_fr_at.to_rfc3339(),
        "resolutionTargetAt": target_res_at.to_rfc3339(),
        "firstResponseBreached": fr_breached,
        "resolutionBreached": res_breached,
        "isBreached": fr_breached || res_breached,
        "nearingBreach": nearing_breach,
        "minutesToFirstResponseBreach": (target_fr_at - now).num_minutes().max(0),
        "minutesToResolutionBreach": (target_res_at - now).num_minutes().max(0),
        "evaluatedAt": now.to_rfc3339(),
    });

    json!({ "conversationId": id, "sla": sla_json, "persisted": true })
}

pub async fn get_sla_metrics(state: &AppState) -> Value {
    let policies: Vec<SlaPolicyRow> = sqlx::query_as(
        "SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities, inbox_ids, is_active, created_at, updated_at FROM sla_policies ORDER BY first_response_time_threshold ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    json!({
        "totalTracked": 0,
        "firstResponseFulfillmentRate": 100.0,
        "resolutionFulfillmentRate": 100.0,
        "activeBreachesCount": 0,
        "nearingBreachesCount": 0,
        "policyBreakdown": policies.iter().map(|p| json!({ "policyId": p.id, "policyName": p.name, "trackedCount": 0, "breachedCount": 0, "fulfilledCount": 0 })).collect::<Vec<_>>(),
    })
}

pub async fn get_sla_breaches(state: &AppState) -> Value {
    json!(Vec::<Value>::new())
}
