use crate::routes::auth::Tenant;
use crate::routes::canned_macros_sla::dispatch_whatsapp_text;
use crate::{db::AppState, models::AutomationRule};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Normalised inbound message used by the automation matcher.
struct InboundProbe {
    text_norm: String,
    payload_norm: String,
    intent_norm: String,
    raw_text: String,
}

impl InboundProbe {
    fn new(text: &str, payload_id: Option<&str>, intent: Option<&str>) -> Self {
        Self {
            text_norm: text.trim().to_lowercase(),
            payload_norm: payload_id.unwrap_or("").trim().to_lowercase(),
            intent_norm: intent.unwrap_or("").trim().to_lowercase(),
            raw_text: text.to_string(),
        }
    }
}

fn rule_matches(r: &AutomationRule, p: &InboundProbe) -> bool {
    let kw = r.trigger_keyword.trim().to_lowercase();
    if kw.is_empty() {
        return false;
    }
    let text_norm = p.text_norm.as_str();
    let payload_norm = p.payload_norm.as_str();
    let intent_norm = p.intent_norm.as_str();

    match r.match_type.as_str() {
        "exact" => text_norm == kw,
        "contains" => !text_norm.is_empty() && text_norm.contains(&kw),
        "starts_with" => !text_norm.is_empty() && text_norm.starts_with(&kw),
        "regex" => {
            if let Ok(re) = regex::RegexBuilder::new(&r.trigger_keyword).case_insensitive(true).build() {
                re.is_match(&p.raw_text)
            } else {
                false
            }
        }
        "button_click" | "list_select" => {
            payload_norm == kw || (!payload_norm.is_empty() && payload_norm.contains(&kw)) || text_norm == kw
        }
        "intent" => !intent_norm.is_empty() && intent_norm == kw,
        _ => !text_norm.is_empty() && text_norm.contains(&kw),
    }
}

async fn active_rules(state: &AppState, tenant: &str) -> Result<Vec<AutomationRule>, sqlx::Error> {
    sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE is_active = 1 AND COALESCE(tenant_key, '') = ? ORDER BY priority DESC, created_at ASC",
    )
    .bind(tenant)
    .fetch_all(&state.db)
    .await
}

/// Coalesced outbound address + channel for a conversation, from real rows only.
async fn conversation_target(state: &AppState, conversation_id: &str) -> Option<(Option<String>, String)> {
    sqlx::query_as::<_, (Option<String>, String)>(
        "SELECT COALESCE(NULLIF(c.contact_phone, ''), NULLIF(c.phone, ''), NULLIF(ct.value, '')) AS phone, c.channel AS channel \
         FROM conversations c LEFT JOIN contacts ct ON ct.id = c.contact_id WHERE c.id = ? LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
}

fn rule_json(r: &AutomationRule) -> Value {
    json!({
        "id": r.id,
        "name": r.name,
        "description": r.description,
        "triggerKeyword": r.trigger_keyword,
        "matchType": r.match_type,
        "actionType": r.action_type,
        "actionValue": r.action_value,
        "priority": r.priority,
        "isActive": r.is_active,
        "executionCount": r.execution_count,
        "lastTriggeredAt": r.last_triggered_at,
        "createdAt": r.created_at
    })
}

pub async fn list_automation_rules(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
) -> Response {
    let rules = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE COALESCE(tenant_key, '') = ? ORDER BY priority DESC, created_at ASC",
    )
    .bind(&tenant)
    .fetch_all(&state.db)
    .await;

    match rules {
        Ok(rules) => (StatusCode::OK, Json(rules)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("could not read the automation_rules table: {err}") })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CreateRulePayload {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(alias = "triggerKeyword")]
    pub trigger_keyword: String,
    #[serde(alias = "matchType", default = "default_match_type")]
    pub match_type: String,
    #[serde(alias = "actionType")]
    pub action_type: String,
    #[serde(alias = "actionValue", default)]
    pub action_value: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(alias = "isActive", default = "default_true")]
    pub is_active: Option<bool>,
}

fn default_match_type() -> String {
    "contains".to_string()
}

fn default_true() -> Option<bool> {
    Some(true)
}

pub async fn create_automation_rule(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Json(payload): Json<CreateRulePayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let priority = payload.priority.unwrap_or(0);
    let is_active = payload.is_active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        "INSERT INTO automation_rules (id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, created_at, tenant_key, channel) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, '')",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.trigger_keyword)
    .bind(&payload.match_type)
    .bind(&payload.action_type)
    .bind(&payload.action_value)
    .bind(priority)
    .bind(is_active)
    .bind(&now)
    .bind(&tenant)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => (
            axum::http::StatusCode::CREATED,
            Json(json!({
                "id": id,
                "name": payload.name,
                "description": payload.description,
                "triggerKeyword": payload.trigger_keyword,
                "matchType": payload.match_type,
                "actionType": payload.action_type,
                "actionValue": payload.action_value,
                "priority": priority,
                "isActive": is_active,
                "executionCount": 0,
                "lastTriggeredAt": null,
                "createdAt": now
            })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create automation rule: {}", e) })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateRulePayload {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "triggerKeyword")]
    pub trigger_keyword: Option<String>,
    #[serde(alias = "matchType")]
    pub match_type: Option<String>,
    #[serde(alias = "actionType")]
    pub action_type: Option<String>,
    #[serde(alias = "actionValue")]
    pub action_value: Option<String>,
    pub priority: Option<i32>,
    #[serde(alias = "isActive")]
    pub is_active: Option<bool>,
}

pub async fn update_automation_rule(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRulePayload>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE id = ? AND COALESCE(tenant_key, '') = ?",
    )
    .bind(&id)
    .bind(&tenant)
    .fetch_optional(&state.db)
    .await;

    let existing = match existing {
        Ok(Some(rule)) => rule,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({ "error": "Automation rule not found" })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let name = payload.name.unwrap_or(existing.name);
    let description = payload.description.or(existing.description);
    let trigger_keyword = payload.trigger_keyword.unwrap_or(existing.trigger_keyword);
    let match_type = payload.match_type.unwrap_or(existing.match_type);
    let action_type = payload.action_type.unwrap_or(existing.action_type);
    let action_value = payload.action_value.unwrap_or(existing.action_value);
    let priority = payload.priority.unwrap_or(existing.priority);
    let is_active = payload.is_active.unwrap_or(existing.is_active);

    let res = sqlx::query(
        "UPDATE automation_rules SET name = ?, description = ?, trigger_keyword = ?, match_type = ?, action_type = ?, action_value = ?, priority = ?, is_active = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&description)
    .bind(&trigger_keyword)
    .bind(&match_type)
    .bind(&action_type)
    .bind(&action_value)
    .bind(priority)
    .bind(is_active)
    .bind(&id)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(json!({
                "id": id,
                "name": name,
                "description": description,
                "triggerKeyword": trigger_keyword,
                "matchType": match_type,
                "actionType": action_type,
                "actionValue": action_value,
                "priority": priority,
                "isActive": is_active,
                "executionCount": existing.execution_count,
                "lastTriggeredAt": existing.last_triggered_at,
                "createdAt": existing.created_at
            })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_automation_rule(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM automation_rules WHERE id = ? AND COALESCE(tenant_key, '') = ?")
        .bind(&id)
        .bind(&tenant)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) => {
            if result.rows_affected() == 0 {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Rule not found" })),
                )
                    .into_response()
            } else {
                (
                    axum::http::StatusCode::OK,
                    Json(json!({ "status": "ok", "deleted": true })),
                )
                    .into_response()
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct TestRulePayload {
    #[serde(default)]
    pub text: String,
    #[serde(alias = "payloadId", alias = "payload_id")]
    pub payload_id: Option<String>,
    pub intent: Option<String>,
}

pub async fn test_automation_rule(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Json(payload): Json<TestRulePayload>,
) -> Response {
    let probe = InboundProbe::new(&payload.text, payload.payload_id.as_deref(), payload.intent.as_deref());

    let rules = match active_rules(&state, &tenant).await {
        Ok(rules) => rules,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("could not read the active automation rules: {err}") })),
            )
                .into_response()
        }
    };

    for r in rules {
        if rule_matches(&r, &probe) {
            let preview = match serde_json::from_str::<Value>(&r.action_value) {
                Ok(parsed) => (parsed, Value::Null),
                Err(err) => (
                    Value::Null,
                    json!({ "error": format!("this rule's actionValue is not valid JSON: {err}") }),
                ),
            };

            return (
                StatusCode::OK,
                Json(json!({
                    "matched": true,
                    "applied": false,
                    "rule": rule_json(&r),
                    "actionType": r.action_type,
                    "actionValue": r.action_value,
                    "preview": preview.0,
                    "previewError": preview.1,
                    "note": "Dry-run match only. No message was sent, no conversation was mutated and execution_count was not incremented. Use POST /automation-rules/apply to execute for real.",
                })),
            )
                .into_response();
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "matched": false,
            "applied": false,
            "message": "No active automation rule matched this input.",
            "note": "The AI reply engine owns unmatched inbound messages; no rule action was performed."
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct ApplyRulePayload {
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    #[serde(default)]
    pub text: String,
    #[serde(alias = "payloadId", alias = "payload_id")]
    pub payload_id: Option<String>,
    pub intent: Option<String>,
}

/// POST /automation-rules/apply -- real execution of the first matching rule:
/// the action is performed, the counter is bumped and an audit record is written.
pub async fn apply_automation_rules(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Json(payload): Json<ApplyRulePayload>,
) -> Response {
    let conv: Option<(String, String, Option<String>)> = match sqlx::query_as(
        "SELECT id, channel, custom_attributes FROM conversations WHERE id = ? LIMIT 1",
    )
    .bind(&payload.conversation_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("could not read conversation '{}': {err}", payload.conversation_id) })),
            )
                .into_response()
        }
    };

    let Some((conv_id, conv_channel, attrs_raw)) = conv else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Conversation '{}' does not exist; nothing was executed.", payload.conversation_id) })),
        )
            .into_response();
    };

    let probe = InboundProbe::new(&payload.text, payload.payload_id.as_deref(), payload.intent.as_deref());
    let matched = match active_rules(&state, &tenant).await {
        Ok(rules) => rules.into_iter().find(|r| rule_matches(r, &probe)),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("could not read the active automation rules: {err}") })),
            )
                .into_response()
        }
    };

    let Some(rule) = matched else {
        return (
            StatusCode::OK,
            Json(json!({
                "matched": false,
                "status": "no_match",
                "conversationId": conv_id,
                "note": "No active automation rule matched; no changes written.",
            })),
        )
            .into_response();
    };

    let mut results: Vec<Value> = Vec::new();
    let mut performed = 0usize;
    let mut not_performed = 0usize;
    let now = Utc::now().to_rfc3339();

    let action_value_text = rule.action_value.clone();
    let parsed_action: Value = serde_json::from_str(&action_value_text).unwrap_or(Value::Null);

    match rule.action_type.as_str() {
        "auto_reply" => {
            let body = if parsed_action.is_string() {
                parsed_action.as_str().unwrap_or("").to_string()
            } else if let Some(b) = parsed_action.get("body").and_then(|v| v.as_str()) {
                b.to_string()
            } else {
                action_value_text.clone()
            };

            let (phone, _c) = conversation_target(&state, &conv_id).await.unwrap_or((None, conv_channel.clone()));
            let msg_id = Uuid::new_v4().to_string();
            let outcome = if conv_channel.eq_ignore_ascii_case("whatsapp") {
                dispatch_whatsapp_text(&state, phone.as_deref().unwrap_or(""), &body, &msg_id).await
            } else {
                json!({
                    "dispatched": false,
                    "reason": format!("No outbound dispatcher is ported for channel '{conv_channel}'; reply stored locally only"),
                })
            };

            let dispatched = outcome["dispatched"].as_bool().unwrap_or(false);
            let delivery_state = if dispatched { "sent" } else { "failed" };

            let insert = sqlx::query(
                "INSERT INTO messages (id, conversation_id, sender, sender_name, direction, channel, body, status, delivery_status, sent_at, created_at, provider, intent) VALUES (?, ?, 'bot', ?, 'outbound', ?, ?, ?, ?, ?, ?, 'automation_rule', ?)",
            )
            .bind(&msg_id)
            .bind(&conv_id)
            .bind(format!("automation:{}", rule.name))
            .bind(&conv_channel)
            .bind(&body)
            .bind(delivery_state)
            .bind(delivery_state)
            .bind(&now)
            .bind(&now)
            .bind(&rule.id)
            .execute(&state.db)
            .await;

            if let Err(e) = insert {
                results.push(json!({ "actionType": rule.action_type, "status": "failed", "error": format!("Reply could not be stored: {e}") }));
                not_performed += 1;
            } else if dispatched {
                // The send and the two follow-up writes are reported separately: a
                // reply that left the process but whose stored state did not land is
                // not a clean success, so the failure rides alongside the dispatch.
                let mut state_errors: Vec<String> = Vec::new();

                let stamp_res = sqlx::query("UPDATE conversations SET last_outbound_at = ?, updated_at = ? WHERE id = ?")
                    .bind(&now)
                    .bind(&now)
                    .bind(&conv_id)
                    .execute(&state.db)
                    .await;
                match stamp_res {
                    Ok(report) if report.rows_affected() == 1 => {}
                    Ok(report) => state_errors.push(format!(
                        "the reply was sent, but last_outbound_at/updated_at were not recorded on conversation '{conv_id}': UPDATE conversations affected {} rows (expected 1)",
                        report.rows_affected()
                    )),
                    Err(err) => state_errors.push(format!(
                        "the reply was sent, but last_outbound_at/updated_at could not be recorded on conversation '{conv_id}': {err}"
                    )),
                }

                let provider_message_id = outcome["providerMessageId"].as_str().unwrap_or("");
                let provider_res = sqlx::query("UPDATE messages SET provider_message_id = ? WHERE id = ?")
                    .bind(provider_message_id)
                    .bind(&msg_id)
                    .execute(&state.db)
                    .await;
                match provider_res {
                    Ok(report) if report.rows_affected() == 1 => {}
                    Ok(report) => state_errors.push(format!(
                        "the reply was sent, but provider_message_id was not stored on message '{msg_id}': UPDATE messages affected {} rows (expected 1)",
                        report.rows_affected()
                    )),
                    Err(err) => state_errors.push(format!(
                        "the reply was sent, but provider_message_id could not be stored on message '{msg_id}': {err}"
                    )),
                }

                if !state_errors.is_empty() {
                    not_performed += 1;
                    results.push(json!({
                        "actionType": "outbound_state_write",
                        "status": "failed",
                        "messageId": msg_id,
                        "conversationId": conv_id,
                        "error": state_errors.join("; "),
                        "note": "The message really was accepted by the provider; only the follow-up stored state is missing.",
                    }));
                }

                results.push(json!({
                    "actionType": rule.action_type, "messageId": msg_id, "status": "dispatched",
                    "text": body, "dispatch": outcome,
                    "stateWriteErrors": state_errors,
                }));
                performed += 1;
            } else {
                results.push(json!({
                    "actionType": rule.action_type, "messageId": msg_id, "status": "stored_not_dispatched",
                    "text": body, "dispatch": outcome,
                }));
                not_performed += 1;
            }
        }
        "handoff" => {
            let res = sqlx::query("UPDATE conversations SET human_takeover = 1, updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(&conv_id)
                .execute(&state.db)
                .await;
            match res {
                Ok(_) => {
                    results.push(json!({
                        "actionType": rule.action_type, "status": "applied",
                        "note": "human_takeover set to 1 on the conversation; the bot reply engine reads this flag.",
                    }));
                    performed += 1;
                }
                Err(e) => {
                    results.push(json!({ "actionType": rule.action_type, "status": "failed", "error": e.to_string() }));
                    not_performed += 1;
                }
            }
        }
        "add_tag" => {
            let mut attrs: Value = serde_json::from_str(attrs_raw.as_deref().unwrap_or("{}"))
                .unwrap_or_else(|_| json!({}));
            if !attrs.is_object() {
                attrs = json!({});
            }
            let tag = if parsed_action.is_string() {
                parsed_action.as_str().unwrap_or("").to_string()
            } else {
                action_value_text.clone()
            };
            let tag = tag.trim().to_lowercase();
            let obj = attrs.as_object_mut().unwrap();
            if !obj.contains_key("tags") {
                obj.insert("tags".to_string(), json!([]));
            }
            let list = obj.get_mut("tags").unwrap();
            let mut added = false;
            if let Some(arr) = list.as_array_mut() {
                if !arr.iter().any(|v| v.as_str().map(|s| s.to_lowercase()).as_deref() == Some(tag.as_str())) {
                    arr.push(json!(tag.clone()));
                    added = true;
                }
            }
            let serialized = serde_json::to_string(obj).unwrap_or_else(|_| "{}".to_string());
            let res = sqlx::query("UPDATE conversations SET custom_attributes = ?, updated_at = ? WHERE id = ?")
                .bind(&serialized)
                .bind(&now)
                .bind(&conv_id)
                .execute(&state.db)
                .await;
            match res {
                Ok(_) => {
                    results.push(json!({
                        "actionType": rule.action_type, "tag": tag, "addedNow": added, "status": "applied",
                    }));
                    performed += 1;
                }
                Err(e) => {
                    results.push(json!({ "actionType": rule.action_type, "status": "failed", "error": e.to_string() }));
                    not_performed += 1;
                }
            }
        }
        "interactive_buttons" | "list_reply" | "send_cta_url" | "send_csat" => {
            not_performed += 1;
            results.push(json!({
                "actionType": rule.action_type,
                "status": "deferred",
                "reason": format!(
                    "The outbound {} builder is not ported in backend-rs, so NO message was sent. Only plain-text dispatch exists.",
                    rule.action_type
                ),
                "wouldHaveSent": parsed_action,
            }));
        }
        other => {
            not_performed += 1;
            results.push(json!({
                "actionType": other,
                "status": "unsupported",
                "reason": format!("Automation action type '{other}' has no executor in backend-rs and was not performed."),
            }));
        }
    }

    // Only a rule that actually changed something bumps the counters.
    let counter_bumped = performed > 0;
    let mut execution_count_after: Option<i64> = None;
    if counter_bumped {
        let bump = sqlx::query(
            "UPDATE automation_rules SET execution_count = COALESCE(execution_count, 0) + 1, last_triggered_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&rule.id)
        .execute(&state.db)
        .await;
        if bump.is_ok() {
            execution_count_after = sqlx::query_scalar::<_, i64>("SELECT execution_count FROM automation_rules WHERE id = ?")
                .bind(&rule.id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();
        } else {
            results.push(json!({
                "actionType": "counter", "status": "failed",
                "error": "execution_count/last_triggered_at could not be written for this rule",
            }));
            not_performed += 1;
        }
    }

    let record = json!({
        "rule_id": rule.id,
        "rule_name": rule.name,
        "action_type": rule.action_type,
        "conversation_id": conv_id,
        "matched_text": payload.text,
        "payload_id": payload.payload_id,
        "intent": payload.intent,
        "performed": performed,
        "not_performed": not_performed,
        "results": results,
        "executed_at": now,
    });
    let audit_id = Uuid::new_v4().to_string();
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, severity, compliance_category, description, details, created_at) VALUES (?, 'automation_rule', ?, 'automation_rule_applied', 'automation_engine', 'info', 'audit_trail', ?, ?, ?)",
    )
    .bind(&audit_id)
    .bind(&rule.id)
    .bind(format!("Automation '{}' executed on conversation {}", rule.name, conv_id))
    .bind(serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string()))
    .bind(&now)
    .execute(&state.db)
    .await;

    let status = if performed > 0 && not_performed == 0 {
        "ok"
    } else if performed > 0 {
        "partial"
    } else {
        "no_effect"
    };

    (
        StatusCode::OK,
        Json(json!({
            "matched": true,
            "status": status,
            "rule": rule_json(&rule),
            "conversationId": conv_id,
            "performedActionCount": performed,
            "notPerformedActionCount": not_performed,
            "results": results,
            "executionCountBefore": rule.execution_count,
            "executionCountAfter": execution_count_after,
            "counterBumped": counter_bumped,
            "executionRecordId": audit_id,
            "executionRecordStored": audit_res.is_ok(),
            "executedAt": now,
            "note": if status == "no_effect" {
                "The rule matched but nothing was actually performed; see results[] for the honest reason."
            } else if status == "partial" {
                "Some actions were performed for real; the rest were deferred or failed and are labelled per entry."
            } else {
                "All matched actions produced real persisted effects."
            },
        })),
    )
        .into_response()
}

/// A scalar aggregate that must never silently degrade to `0`: the `Err` arm
/// keeps the failure visible so the caller can report why the number is absent.
async fn scalar_i64(state: &AppState, sql: &str) -> Result<Option<i64>, sqlx::Error> {
    match sqlx::query_scalar::<_, i64>(sql).fetch_optional(&state.db).await {
        Ok(value) => Ok(value),
        Err(err) => {
            tracing::warn!("automation stats query failed ({sql}): {err}");
            Err(err)
        }
    }
}

async fn scalar_i64_scoped(
    state: &AppState,
    sql: &str,
    tenant: &str,
) -> Result<Option<i64>, sqlx::Error> {
    match sqlx::query_scalar::<_, i64>(sql)
        .bind(tenant)
        .fetch_optional(&state.db)
        .await
    {
        Ok(value) => Ok(value),
        Err(err) => {
            tracing::warn!("automation stats query failed ({sql}): {err}");
            Err(err)
        }
    }
}

pub async fn get_automation_stats(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
) -> impl IntoResponse {
    let mut warnings: Vec<String> = Vec::new();

    let total_rules = scalar_i64_scoped(
        &state,
        "SELECT COUNT(*) FROM automation_rules WHERE COALESCE(tenant_key, '') = ?",
        &tenant,
    )
    .await;
    let active_rules = scalar_i64_scoped(
        &state,
        "SELECT COUNT(*) FROM automation_rules WHERE is_active = 1 AND COALESCE(tenant_key, '') = ?",
        &tenant,
    )
    .await;
    let total_executions = scalar_i64_scoped(
        &state,
        "SELECT COALESCE(SUM(execution_count), 0) FROM automation_rules WHERE COALESCE(tenant_key, '') = ?",
        &tenant,
    )
    .await;

    let csat_row: Option<(i64, Option<f64>)> = match sqlx::query_as::<_, (i64, Option<f64>)>(
        "SELECT COUNT(*), AVG(csat_score) FROM conversations WHERE csat_score IS NOT NULL",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => {
            warnings.push(format!("CSAT could not be read: {err}"));
            None
        }
    };

    let csat_responses_count = csat_row.map(|row| row.0);
    let csat_average = csat_row.and_then(|row| row.1).map(|avg| (avg * 100.0).round() / 100.0);

    // Heuristic, explicitly labelled: 5 minutes saved per recorded execution.
    // It is an estimate, not a measurement, and the response says so.
    let total_executions_value = total_executions.as_ref().ok().copied().flatten();
    let time_saved_hours =
        total_executions_value.map(|execs| ((execs as f64 * 5.0) / 60.0 * 10.0).round() / 10.0);

    let nudges_sent =
        scalar_i64(&state, "SELECT COUNT(*) FROM messages WHERE provider = 'inactivity_nudge' AND delivery_status = 'sent'").await;
    let nudges_failed =
        scalar_i64(&state, "SELECT COUNT(*) FROM messages WHERE provider = 'inactivity_nudge' AND delivery_status = 'failed'").await;

    // Rows written by the pre-dispatch implementation: marked 'sent' with no
    // provider id, so they cannot be proven. Reported separately, not counted.
    let legacy_unverified = scalar_i64(
        &state,
        "SELECT COUNT(*) FROM messages WHERE provider_message_id = 'inactivity_nudge' AND (provider IS NULL OR provider != 'inactivity_nudge')",
    )
    .await;

    let execution_records =
        scalar_i64(&state, "SELECT COUNT(*) FROM audit_logs WHERE action = 'automation_rule_applied'").await;
    let macro_records = scalar_i64(&state, "SELECT COUNT(*) FROM audit_logs WHERE action = 'macro_executed'").await;

    let label = |name: &str, value: &Result<Option<i64>, sqlx::Error>| match value {
        Ok(_) => None,
        Err(err) => Some(format!("{name} could not be read: {err}")),
    };
    for maybe in [
        label("automation rule count", &total_rules),
        label("active automation rule count", &active_rules),
        label("automation execution total", &total_executions),
        label("nudge sent count", &nudges_sent),
        label("nudge failed count", &nudges_failed),
        label("unverified legacy nudge count", &legacy_unverified),
        label("automation audit record count", &execution_records),
        label("macro audit record count", &macro_records),
    ] {
        if let Some(warning) = maybe {
            warnings.push(warning);
        }
    }

    Json(json!({
        "tenantKey": tenant,
        "totalRules": total_rules.ok().flatten(),
        "activeRules": active_rules.ok().flatten(),
        "totalExecutions": total_executions.ok().flatten(),
        "csatAverage": csat_average,
        "csatResponsesCount": csat_responses_count,
        "timeSavedHours": time_saved_hours,
        "timeSavedHoursBasis": "estimate only: execution_count x 5 minutes. Not a measured value.",
        "nudgesSent": nudges_sent.ok().flatten(),
        "nudgesFailedToDispatch": nudges_failed.ok().flatten(),
        "legacyUnverifiedNudgeRows": legacy_unverified.ok().flatten(),
        "ruleExecutionAuditRecords": execution_records.ok().flatten(),
        "macroExecutionAuditRecords": macro_records.ok().flatten(),
        "warnings": warnings,
        "nullMeans": "a null metric means the database could not answer that count; see warnings",
    }))
}

#[derive(sqlx::FromRow)]
struct NudgeCandidate {
    id: String,
    last_inbound_at: Option<String>,
    channel: String,
    phone: Option<String>,
}

pub async fn run_inactivity_nudges(State(state): State<AppState>) -> Response {
    let now = Utc::now();
    let window_start = now - chrono::Duration::hours(24);
    let window_end = now - chrono::Duration::hours(18);

    let candidates = match sqlx::query_as::<_, NudgeCandidate>(
        "SELECT c.id AS id, c.last_inbound_at AS last_inbound_at, c.channel AS channel, \
         COALESCE(NULLIF(c.contact_phone, ''), NULLIF(c.phone, ''), NULLIF(ct.value, '')) AS phone \
         FROM conversations c LEFT JOIN contacts ct ON ct.id = c.contact_id \
         WHERE (c.state = 'ENGAGED' OR c.state = 'AWAITING_REPLY' OR c.state = 'engaged' OR c.state = 'awaiting_reply') \
         AND (c.human_takeover = 0 OR c.human_takeover IS NULL) AND c.last_inbound_at IS NOT NULL",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        // A failed read is not "no candidates": answering here would tell the
        // scheduler that the window was empty when nothing was actually checked.
        Err(err) => {
            tracing::warn!("[AUTOMATION] inactivity nudge candidates could not be read: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("the nudge candidates could not be read, so nothing was sent: {err}"),
                    "status": "read_failed",
                    "nudgedCount": 0,
                    "notDispatchedCount": 0,
                })),
            )
                .into_response();
        }
    };

    let mut dispatched_ids = Vec::new();
    let mut results: Vec<Value> = Vec::new();
    let mut failures = 0usize;
    let mut followup_failures = 0usize;
    let now_str = now.to_rfc3339();

    for cand in candidates {
        let should_nudge = if let Some(ref inbound_str) = cand.last_inbound_at {
            parse_candidate_ts(inbound_str).map(|inbound_utc| {
                inbound_utc >= window_start && inbound_utc <= window_end
            }).unwrap_or(false)
        } else {
            false
        };

        if !should_nudge {
            continue;
        }

        // Do not nudge twice within the window, counting only provable sends.
        // created_at is stored in two historical formats, so both bounds are compared.
        let window_start_naive = window_start.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let recent_nudge: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = ? AND provider = 'inactivity_nudge' AND delivery_status = 'sent' AND (created_at >= ? OR created_at >= ?)",
        )
        .bind(&cand.id)
        .bind(window_start.to_rfc3339())
        .bind(&window_start_naive)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

        if recent_nudge > 0 {
            continue;
        }

        let body = "Hi there, just checking in to see if you needed any more assistance or had further questions before our active chat session wraps up!";
        let msg_id = Uuid::new_v4().to_string();

        let outcome = if cand.channel.eq_ignore_ascii_case("whatsapp") {
            dispatch_whatsapp_text(&state, cand.phone.as_deref().unwrap_or(""), body, &msg_id).await
        } else {
            json!({
                "dispatched": false,
                "reason": format!("Channel '{}' has no ported outbound dispatcher; no nudge was sent.", cand.channel),
            })
        };

        let dispatched = outcome["dispatched"].as_bool().unwrap_or(false);
        let delivery_state = if dispatched { "sent" } else { "failed" };

        let insert = sqlx::query(
            "INSERT INTO messages (id, conversation_id, sender, sender_name, direction, channel, body, status, delivery_status, sent_at, created_at, provider, intent) VALUES (?, ?, 'bot', 'inactivity_nudge', 'outbound', ?, ?, ?, ?, ?, ?, 'inactivity_nudge', 'inactivity_nudge')",
        )
        .bind(&msg_id)
        .bind(&cand.id)
        .bind(&cand.channel)
        .bind(body)
        .bind(delivery_state)
        .bind(delivery_state)
        .bind(&now_str)
        .bind(&now_str)
        .execute(&state.db)
        .await;

        if insert.is_err() {
            failures += 1;
            results.push(json!({
                "conversationId": cand.id,
                "status": "failed",
                "error": "The nudge row could not be written to messages.",
            }));
            continue;
        }

        let mut state_errors: Vec<String> = Vec::new();
        if dispatched {
            // The dispatch result and the follow-up writes are reported apart: a
            // nudge the provider accepted but whose stored state is missing is not
            // a clean success.
            let provider_res = sqlx::query("UPDATE messages SET provider_message_id = ? WHERE id = ?")
                .bind(outcome["providerMessageId"].as_str().unwrap_or(""))
                .bind(&msg_id)
                .execute(&state.db)
                .await;
            match provider_res {
                Ok(report) if report.rows_affected() == 1 => {}
                Ok(report) => state_errors.push(format!(
                    "the nudge was sent, but provider_message_id was not stored on message '{msg_id}': UPDATE messages affected {} rows (expected 1)",
                    report.rows_affected()
                )),
                Err(err) => state_errors.push(format!(
                    "the nudge was sent, but provider_message_id could not be stored on message '{msg_id}': {err}"
                )),
            }

            let stamp_res = sqlx::query("UPDATE conversations SET last_outbound_at = ?, updated_at = ? WHERE id = ?")
                .bind(&now_str)
                .bind(&now_str)
                .bind(&cand.id)
                .execute(&state.db)
                .await;
            match stamp_res {
                Ok(report) if report.rows_affected() == 1 => {}
                Ok(report) => state_errors.push(format!(
                    "the nudge was sent, but last_outbound_at/updated_at were not recorded on conversation '{}': UPDATE conversations affected {} rows (expected 1)",
                    cand.id, report.rows_affected()
                )),
                Err(err) => state_errors.push(format!(
                    "the nudge was sent, but last_outbound_at/updated_at could not be recorded on conversation '{}': {err}",
                    cand.id
                )),
            }

            // The send really happened, so the conversation stays listed as nudged;
            // the missing follow-up state is counted separately so the overall
            // status and note cannot read as a clean run.
            dispatched_ids.push(cand.id.clone());
            if !state_errors.is_empty() {
                followup_failures += 1;
            }
        } else {
            failures += 1;
        }

        results.push(json!({
            "conversationId": cand.id,
            "messageId": msg_id,
            "status": if dispatched { "dispatched" } else { "stored_not_dispatched" },
            "dispatch": outcome,
            "stateWriteErrors": state_errors,
        }));
    }

    let audit_id = Uuid::new_v4().to_string();
    let record = json!({
        "window_start": window_start.to_rfc3339(),
        "window_end": window_end.to_rfc3339(),
        "dispatched": dispatched_ids.len(),
        "not_dispatched": failures,
        "followup_state_write_failures": followup_failures,
        "results": results,
        "run_at": now_str.clone(),
    });
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, severity, compliance_category, description, details, created_at) VALUES (?, 'automation', 'inactivity_nudge', 'nudge_run', 'automation_engine', 'info', 'audit_trail', ?, ?, ?)",
    )
    .bind(&audit_id)
    .bind(format!(
        "Inactivity nudge run: {} dispatched, {} not dispatched, {} with unstored follow-up state",
        dispatched_ids.len(),
        failures,
        followup_failures
    ))
    .bind(serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string()))
    .bind(&now_str)
    .execute(&state.db)
    .await;

    // Each truth gets its own sentence: a delivered nudge whose follow-up row
    // failed is reported as exactly that, never as a clean run.
    let mut note_parts: Vec<String> = Vec::new();
    if failures > 0 {
        note_parts.push(format!(
            "{failures} nudge(s) were written to the message log but NOT delivered -- each carries the real provider/config reason."
        ));
    }
    if followup_failures > 0 {
        note_parts.push(format!(
            "{followup_failures} sent nudge(s) reached the provider, but their follow-up rows (provider_message_id / last_outbound_at) were NOT stored; see stateWriteErrors on those entries."
        ));
    }
    if note_parts.is_empty() {
        note_parts.push(if dispatched_ids.is_empty() {
            "No conversation in the 18-24h inactivity window needed a nudge; nothing was sent.".to_string()
        } else {
            "Every listed nudge was accepted by the WhatsApp provider and its follow-up state was stored.".to_string()
        });
    }

    Json(json!({
        "status": if failures == 0 && followup_failures == 0 && !dispatched_ids.is_empty() {
            "ok"
        } else if dispatched_ids.is_empty() && failures == 0 && followup_failures == 0 {
            "no_candidates"
        } else {
            "partial"
        },
        "nudgedCount": dispatched_ids.len(),
        "conversationIds": dispatched_ids,
        "notDispatchedCount": failures,
        "followupStateWriteFailures": followup_failures,
        "results": results,
        "windowStart": window_start.to_rfc3339(),
        "windowEnd": window_end.to_rfc3339(),
        "executionRecordId": audit_id,
        "executionRecordStored": audit_res.is_ok(),
        "note": note_parts.join(" "),
    }))
    .into_response()
}

fn parse_candidate_ts(raw: &str) -> Option<chrono::DateTime<Utc>> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S%.f"))
        .ok()
        .map(|naive| naive.and_utc())
}
