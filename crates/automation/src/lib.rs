//! Automation rule business logic — moved from backend-rs monolith.

use chrono::{NaiveDateTime, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// Application state for the automation service.
#[derive(Clone)]
pub struct AutomationState {
    pub pool: PgPool,
    pub http_client: reqwest::Client,
}

impl AutomationState {
    pub fn new(pool: PgPool, http_client: reqwest::Client) -> Self {
        Self { pool, http_client }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_keyword: String,
    pub match_type: String,
    pub action_type: String,
    pub action_value: String,
    pub priority: i32,
    pub is_active: bool,
    pub execution_count: Option<i32>,
    pub last_triggered_at: Option<String>,
    pub created_at: Option<String>,
}

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

async fn active_rules(state: &AutomationState, tenant: &str) -> Result<Vec<AutomationRule>, String> {
    sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE is_active = 1 AND COALESCE(tenant_key, '') = $1 ORDER BY priority DESC, created_at ASC",
    )
    .bind(tenant)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())
}

async fn conversation_target(state: &AutomationState, conversation_id: &str) -> Option<(Option<String>, String)> {
    sqlx::query_as::<_, (Option<String>, String)>(
        "SELECT COALESCE(NULLIF(c.contact_phone, ''), NULLIF(c.phone, ''), NULLIF(ct.value, '')) AS phone, c.channel AS channel \
         FROM conversations c LEFT JOIN contacts ct ON ct.id = c.contact_id WHERE c.id = $1 LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.pool)
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

async fn dispatch_whatsapp_text(state: &AutomationState, phone: &str, body: &str, _local_msg_id: &str) -> Value {
    let creds: Option<(String, String, String)> = sqlx::query_as(
        "SELECT whatsapp_phone_id, whatsapp_access_token, whatsapp_business_account_id \
         FROM gateway_settings WHERE id = 'whatsapp' LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let Some((phone_id, token, _account_id)) = creds else {
        return json!({ "dispatched": false, "reason": "WhatsApp gateway credentials not configured" });
    };

    if phone.is_empty() {
        return json!({ "dispatched": false, "reason": "No phone number available" });
    }

    let url = format!("https://graph.facebook.com/v18.0/{}/messages", phone_id);
    let payload = json!({
        "messaging_product": "whatsapp",
        "to": phone,
        "type": "text",
        "text": { "body": body },
    });

    let resp = state.http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(mut r) => {
            if r.status().is_success() {
                let body = r.json::<Value>().await.unwrap_or(Value::Null);
                let msg_id = body.get("messages").and_then(|m| m.get(0))
                    .and_then(|m| m.get("id")).and_then(|i| i.as_str()).unwrap_or("");
                json!({ "dispatched": true, "providerMessageId": msg_id, "response": body })
            } else {
                let status = r.status();
                let err_body = r.text().await.unwrap_or_default();
                json!({ "dispatched": false, "status": status.as_u16(), "providerError": err_body })
            }
        }
        Err(e) => json!({ "dispatched": false, "transportError": e.to_string() }),
    }
}

#[derive(sqlx::FromRow)]
struct NudgeCandidate {
    id: String,
    last_inbound_at: Option<String>,
    channel: String,
    phone: Option<String>,
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

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn list_automation_rules(state: &AutomationState, tenant: &str) -> Result<Value, String> {
    let rules = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE COALESCE(tenant_key, '') = $1 ORDER BY priority DESC, created_at ASC",
    )
    .bind(tenant)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let out: Vec<Value> = rules.iter().map(|r| rule_json(r)).collect();
    Ok(json!({ "rules": out, "count": out.len() }))
}

pub async fn create_automation_rule(state: &AutomationState, body: &Value) -> Result<Value, String> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = body.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let trigger_keyword = body.get("triggerKeyword").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let match_type = body.get("matchType").and_then(|v| v.as_str()).unwrap_or("contains").to_string();
    let action_type = body.get("actionType").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let action_value = body.get("actionValue").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let priority = body.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let is_active = body.get("isActive").and_then(|v| v.as_bool()).unwrap_or(true);

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO automation_rules (id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, created_at, tenant_key, channel) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, $10, '', '')",
    )
    .bind(&id)
    .bind(&name)
    .bind(&description)
    .bind(&trigger_keyword)
    .bind(&match_type)
    .bind(&action_type)
    .bind(&action_value)
    .bind(priority)
    .bind(is_active)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({
        "id": id, "name": name, "description": description,
        "triggerKeyword": trigger_keyword, "matchType": match_type,
        "actionType": action_type, "actionValue": action_value,
        "priority": priority, "isActive": is_active,
        "executionCount": 0, "lastTriggeredAt": null, "createdAt": now,
    }))
}

pub async fn update_automation_rule(state: &AutomationState, id: &str, body: &Value) -> Result<Value, String> {
    let existing = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE id = $1 AND COALESCE(tenant_key, '') = $2",
    )
    .bind(id)
    .bind("")
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or("Automation rule not found")?;

    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or(&existing.name).to_string();
    let description = body.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.description);
    let trigger_keyword = body.get("triggerKeyword").and_then(|v| v.as_str()).unwrap_or(&existing.trigger_keyword).to_string();
    let match_type = body.get("matchType").and_then(|v| v.as_str()).unwrap_or(&existing.match_type).to_string();
    let action_type = body.get("actionType").and_then(|v| v.as_str()).unwrap_or(&existing.action_type).to_string();
    let action_value = body.get("actionValue").and_then(|v| v.as_str()).unwrap_or(&existing.action_value).to_string();
    let priority = body.get("priority").and_then(|v| v.as_i64()).unwrap_or(existing.priority as i64) as i32;
    let is_active = body.get("isActive").and_then(|v| v.as_bool()).unwrap_or(existing.is_active);

    sqlx::query(
        "UPDATE automation_rules SET name = $1, description = $2, trigger_keyword = $3, match_type = $4, action_type = $5, action_value = $6, priority = $7, is_active = $8 WHERE id = $9",
    )
    .bind(&name).bind(&description).bind(&trigger_keyword).bind(&match_type)
    .bind(&action_type).bind(&action_value).bind(priority).bind(is_active).bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({
        "id": id, "name": name, "description": description,
        "triggerKeyword": trigger_keyword, "matchType": match_type,
        "actionType": action_type, "actionValue": action_value,
        "priority": priority, "isActive": is_active,
        "executionCount": existing.execution_count,
        "lastTriggeredAt": existing.last_triggered_at,
        "createdAt": existing.created_at,
    }))
}

pub async fn delete_automation_rule(state: &AutomationState, id: &str) -> Result<Value, String> {
    let res = sqlx::query("DELETE FROM automation_rules WHERE id = $1 AND COALESCE(tenant_key, '') = $2")
        .bind(id).bind("")
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    if res.rows_affected() == 0 {
        Err("Rule not found".to_string())
    } else {
        Ok(json!({ "status": "ok", "deleted": true }))
    }
}

pub async fn test_automation_rule(state: &AutomationState, body: &Value) -> Result<Value, String> {
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let payload_id = body.get("payloadId").and_then(|v| v.as_str()).map(|s| s.to_string())
        .or(body.get("payload_id").and_then(|v| v.as_str()).map(|s| s.to_string()));
    let intent = body.get("intent").and_then(|v| v.as_str()).map(|s| s.to_string());

    let probe = InboundProbe::new(&text, payload_id.as_deref(), intent.as_deref());
    let rules = active_rules(state, "").await?;

    for r in rules {
        if rule_matches(&r, &probe) {
            let preview = match serde_json::from_str::<Value>(&r.action_value) {
                Ok(parsed) => (parsed, Value::Null),
                Err(err) => (Value::Null, json!({ "error": format!("actionValue is not valid JSON: {}", err) })),
            };
            return Ok(json!({
                "matched": true, "applied": false,
                "rule": rule_json(&r),
                "actionType": r.action_type, "actionValue": r.action_value,
                "preview": preview.0, "previewError": preview.1,
                "note": "Dry-run match only.",
            }));
        }
    }

    Ok(json!({ "matched": false, "applied": false, "message": "No active automation rule matched this input." }))
}

pub async fn apply_automation_rules(state: &AutomationState, body: &Value) -> Result<Value, String> {
    let conversation_id = body.get("conversationId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let payload_id = body.get("payloadId").and_then(|v| v.as_str()).map(|s| s.to_string())
        .or(body.get("payload_id").and_then(|v| v.as_str()).map(|s| s.to_string()));
    let intent = body.get("intent").and_then(|v| v.as_str()).map(|s| s.to_string());

    let conv: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, channel, custom_attributes FROM conversations WHERE id = $1 LIMIT 1",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let (conv_id, conv_channel, attrs_raw) = conv.ok_or(format!("Conversation '{}' does not exist", conversation_id))?;

    let probe = InboundProbe::new(&text, payload_id.as_deref(), intent.as_deref());
    let rule = active_rules(state, "").await?.into_iter().find(|r| rule_matches(r, &probe))
        .ok_or_else(|| json!({ "matched": false, "status": "no_match", "conversationId": conv_id }).to_string())?;

    let mut results: Vec<Value> = Vec::new();
    let mut performed = 0usize;
    let mut not_performed = 0usize;
    let now = Utc::now().to_rfc3339();

    let action_value_text = rule.action_value.clone();
    let parsed_action: Value = serde_json::from_str(&action_value_text).unwrap_or(Value::Null);

    match rule.action_type.as_str() {
        "auto_reply" => {
            let body_text = if parsed_action.is_string() {
                parsed_action.as_str().unwrap_or("").to_string()
            } else if let Some(b) = parsed_action.get("body").and_then(|v| v.as_str()) {
                b.to_string()
            } else {
                action_value_text.clone()
            };

            let (phone, _) = conversation_target(state, &conv_id).await.unwrap_or((None, conv_channel.clone()));
            let msg_id = Uuid::new_v4().to_string();
            let outcome = if conv_channel.eq_ignore_ascii_case("whatsapp") {
                dispatch_whatsapp_text(state, phone.as_deref().unwrap_or(""), &body_text, &msg_id).await
            } else {
                json!({ "dispatched": false, "reason": format!("No outbound dispatcher for channel '{}'", conv_channel) })
            };

            let dispatched = outcome["dispatched"].as_bool().unwrap_or(false);
            let delivery_state = if dispatched { "sent" } else { "failed" };

            sqlx::query(
                "INSERT INTO messages (id, conversation_id, sender, sender_name, direction, channel, body, status, delivery_status, sent_at, created_at, provider, intent) VALUES ($1, $2, 'bot', $3, 'outbound', $4, $5, $6, $7, $8, $9, 'automation_rule', $10)",
            )
            .bind(&msg_id).bind(&conv_id).bind(format!("automation:{}", rule.name))
            .bind(&conv_channel).bind(&body_text).bind(delivery_state).bind(delivery_state)
            .bind(&now).bind(&now).bind(&rule.id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

            if dispatched {
                let mut state_errors: Vec<String> = Vec::new();
                let stamp_res = sqlx::query("UPDATE conversations SET last_outbound_at = $1, updated_at = $2 WHERE id = $3")
                    .bind(&now).bind(&now).bind(&conv_id)
                    .execute(&state.pool).await;
                if let Ok(report) = &stamp_res {
                    if report.rows_affected() != 1 {
                        state_errors.push(format!("last_outbound_at not recorded: {} rows", report.rows_affected()));
                    }
                } else if let Err(e) = stamp_res {
                    state_errors.push(format!("last_outbound_at not recorded: {}", e));
                }

                let provider_message_id = outcome["providerMessageId"].as_str().unwrap_or("");
                let provider_res = sqlx::query("UPDATE messages SET provider_message_id = $1 WHERE id = $2")
                    .bind(provider_message_id).bind(&msg_id)
                    .execute(&state.pool).await;
                if let Ok(report) = &provider_res {
                    if report.rows_affected() != 1 {
                        state_errors.push(format!("provider_message_id not stored: {} rows", report.rows_affected()));
                    }
                } else if let Err(e) = provider_res {
                    state_errors.push(format!("provider_message_id not stored: {}", e));
                }

                if !state_errors.is_empty() {
                    not_performed += 1;
                    results.push(json!({ "actionType": "outbound_state_write", "status": "failed", "error": state_errors.join("; ") }));
                }
                results.push(json!({ "actionType": rule.action_type, "messageId": msg_id, "status": "dispatched", "text": body_text, "dispatch": outcome, "stateWriteErrors": state_errors }));
                performed += 1;
            } else {
                results.push(json!({ "actionType": rule.action_type, "messageId": msg_id, "status": "stored_not_dispatched", "text": body_text, "dispatch": outcome }));
                not_performed += 1;
            }
        }
        "handoff" => {
            sqlx::query("UPDATE conversations SET human_takeover = 1, updated_at = $1 WHERE id = $2")
                .bind(&now).bind(&conv_id)
                .execute(&state.pool)
                .await
                .map_err(|e| e.to_string())?;
            results.push(json!({ "actionType": rule.action_type, "status": "applied" }));
            performed += 1;
        }
        "add_tag" => {
            let mut attrs: Value = serde_json::from_str(attrs_raw.as_deref().unwrap_or("{}")).unwrap_or_else(|_| json!({}));
            if !attrs.is_object() { attrs = json!({}); }
            let tag = if parsed_action.is_string() {
                parsed_action.as_str().unwrap_or("").to_string()
            } else {
                action_value_text.clone()
            };
            let tag = tag.trim().to_lowercase();
            let obj = attrs.as_object_mut().unwrap();
            if !obj.contains_key("tags") { obj.insert("tags".to_string(), json!([])); }
            let list = obj.get_mut("tags").unwrap();
            let mut added = false;
            if let Some(arr) = list.as_array_mut() {
                if !arr.iter().any(|v| v.as_str().map(|s| s.to_lowercase()).as_deref() == Some(tag.as_str())) {
                    arr.push(json!(tag.clone()));
                    added = true;
                }
            }
            let serialized = serde_json::to_string(obj).unwrap_or_else(|_| "{}".to_string());
            sqlx::query("UPDATE conversations SET custom_attributes = $1, updated_at = $2 WHERE id = $3")
                .bind(&serialized).bind(&now).bind(&conv_id)
                .execute(&state.pool)
                .await
                .map_err(|e| e.to_string())?;
            results.push(json!({ "actionType": rule.action_type, "tag": tag, "addedNow": added, "status": "applied" }));
            performed += 1;
        }
        "interactive_buttons" | "list_reply" | "send_cta_url" | "send_csat" => {
            not_performed += 1;
            results.push(json!({ "actionType": rule.action_type, "status": "deferred", "reason": format!("Outbound {} builder not ported", rule.action_type), "wouldHaveSent": parsed_action }));
        }
        other => {
            not_performed += 1;
            results.push(json!({ "actionType": other, "status": "unsupported", "reason": format!("Action type '{}' has no executor", other) }));
        }
    }

    let counter_bumped = performed > 0;
    let mut execution_count_after: Option<i64> = None;
    if counter_bumped {
        sqlx::query(
            "UPDATE automation_rules SET execution_count = COALESCE(execution_count, 0) + 1, last_triggered_at = $1 WHERE id = $2",
        )
        .bind(&now).bind(&rule.id)
        .execute(&state.pool)
        .await
        .ok();
        execution_count_after = sqlx::query_scalar::<_, i64>("SELECT execution_count FROM automation_rules WHERE id = $1")
            .bind(&rule.id)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();
    }

    let record = json!({
        "rule_id": rule.id, "rule_name": rule.name, "action_type": rule.action_type,
        "conversation_id": conv_id, "matched_text": text,
        "performed": performed, "not_performed": not_performed,
        "results": results, "executed_at": now,
    });
    let audit_id = Uuid::new_v4().to_string();
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, severity, compliance_category, description, details, created_at) VALUES ($1, 'automation_rule', $2, 'automation_rule_applied', 'automation_engine', 'info', 'audit_trail', $3, $4, $5)",
    )
    .bind(&audit_id).bind(&rule.id)
    .bind(format!("Automation '{}' executed on conversation {}", rule.name, conv_id))
    .bind(serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string()))
    .bind(&now)
    .execute(&state.pool)
    .await;

    let status = if performed > 0 && not_performed == 0 { "ok" } else if performed > 0 { "partial" } else { "no_effect" };

    Ok(json!({
        "matched": true, "status": status, "rule": rule_json(&rule),
        "conversationId": conv_id,
        "performedActionCount": performed, "notPerformedActionCount": not_performed,
        "results": results,
        "executionCountBefore": rule.execution_count, "executionCountAfter": execution_count_after,
        "counterBumped": counter_bumped,
        "executionRecordId": audit_id, "executionRecordStored": audit_res.is_ok(),
        "executedAt": now,
    }))
}

pub async fn get_automation_stats(state: &AutomationState, _tenant: &str) -> Result<Value, String> {
    async fn scalar_i64(pool: &PgPool, sql: &str) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(sql).fetch_optional(pool).await
    }

    let total_rules = scalar_i64(&state.pool, "SELECT COUNT(*) FROM automation_rules WHERE COALESCE(tenant_key, '') = ''").await;
    let active_rules = scalar_i64(&state.pool, "SELECT COUNT(*) FROM automation_rules WHERE is_active = 1 AND COALESCE(tenant_key, '') = ''").await;
    let total_executions = scalar_i64(&state.pool, "SELECT COALESCE(SUM(execution_count), 0) FROM automation_rules WHERE COALESCE(tenant_key, '') = ''").await;
    let nudges_sent = scalar_i64(&state.pool, "SELECT COUNT(*) FROM messages WHERE provider = 'inactivity_nudge' AND delivery_status = 'sent'").await;
    let nudges_failed = scalar_i64(&state.pool, "SELECT COUNT(*) FROM messages WHERE provider = 'inactivity_nudge' AND delivery_status = 'failed'").await;
    let execution_records = scalar_i64(&state.pool, "SELECT COUNT(*) FROM audit_logs WHERE action = 'automation_rule_applied'").await;

    let total_executions_value = total_executions.as_ref().ok().copied().flatten();
    let time_saved_hours = total_executions_value.map(|execs| ((execs as f64 * 5.0) / 60.0 * 10.0).round() / 10.0);

    Ok(json!({
        "totalRules": total_rules.ok().flatten(),
        "activeRules": active_rules.ok().flatten(),
        "totalExecutions": total_executions.ok().flatten(),
        "timeSavedHours": time_saved_hours,
        "nudgesSent": nudges_sent.ok().flatten(),
        "nudgesFailedToDispatch": nudges_failed.ok().flatten(),
        "ruleExecutionAuditRecords": execution_records.ok().flatten(),
    }))
}

pub async fn run_inactivity_nudges(state: &AutomationState) -> Result<Value, String> {
    let now = Utc::now();
    let window_start = now - chrono::Duration::hours(24);
    let window_end = now - chrono::Duration::hours(18);

    let candidates = sqlx::query_as::<_, NudgeCandidate>(
        "SELECT c.id AS id, c.last_inbound_at AS last_inbound_at, c.channel AS channel, \
         COALESCE(NULLIF(c.contact_phone, ''), NULLIF(c.phone, ''), NULLIF(ct.value, '')) AS phone \
         FROM conversations c LEFT JOIN contacts ct ON ct.id = c.contact_id \
         WHERE (c.state = 'ENGAGED' OR c.state = 'AWAITING_REPLY' OR c.state = 'engaged' OR c.state = 'awaiting_reply') \
         AND (c.human_takeover = 0 OR c.human_takeover IS NULL) AND c.last_inbound_at IS NOT NULL",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("nudge candidates could not be read: {}", e))?;

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
        if !should_nudge { continue; }

        let window_start_naive = window_start.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let recent_nudge: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = $1 AND provider = 'inactivity_nudge' AND delivery_status = 'sent' AND (created_at >= $2 OR created_at >= $3)",
        )
        .bind(&cand.id).bind(window_start.to_rfc3339()).bind(&window_start_naive)
        .fetch_one(&state.pool).await.unwrap_or(0);
        if recent_nudge > 0 { continue; }

        let body = "Hi there, just checking in to see if you needed any more assistance or had further questions before our active chat session wraps up!";
        let msg_id = Uuid::new_v4().to_string();

        let outcome = if cand.channel.eq_ignore_ascii_case("whatsapp") {
            dispatch_whatsapp_text(state, cand.phone.as_deref().unwrap_or(""), body, &msg_id).await
        } else {
            json!({ "dispatched": false, "reason": format!("Channel '{}' has no outbound dispatcher", cand.channel) })
        };

        let dispatched = outcome["dispatched"].as_bool().unwrap_or(false);
        let delivery_state = if dispatched { "sent" } else { "failed" };

        if sqlx::query(
            "INSERT INTO messages (id, conversation_id, sender, sender_name, direction, channel, body, status, delivery_status, sent_at, created_at, provider, intent) VALUES ($1, $2, 'bot', 'inactivity_nudge', 'outbound', $3, $4, $5, $6, $7, $8, 'inactivity_nudge', 'inactivity_nudge')",
        )
        .bind(&msg_id).bind(&cand.id).bind(&cand.channel).bind(body)
        .bind(delivery_state).bind(delivery_state).bind(&now_str).bind(&now_str)
        .execute(&state.pool).await.is_err()
        {
            failures += 1;
            results.push(json!({ "conversationId": cand.id, "status": "failed", "error": "The nudge row could not be written." }));
            continue;
        }

        let mut state_errors: Vec<String> = Vec::new();
        if dispatched {
            let provider_res = sqlx::query("UPDATE messages SET provider_message_id = $1 WHERE id = $2")
                .bind(outcome["providerMessageId"].as_str().unwrap_or(""))
                .bind(&msg_id).execute(&state.pool).await;
            if let Ok(report) = &provider_res {
                if report.rows_affected() != 1 {
                    state_errors.push(format!("provider_message_id not stored: {} rows", report.rows_affected()));
                }
            } else if let Err(e) = provider_res {
                state_errors.push(format!("provider_message_id not stored: {}", e));
            }

            let stamp_res = sqlx::query("UPDATE conversations SET last_outbound_at = $1, updated_at = $2 WHERE id = $3")
                .bind(&now_str).bind(&now_str).bind(&cand.id).execute(&state.pool).await;
            if let Ok(report) = &stamp_res {
                if report.rows_affected() != 1 {
                    state_errors.push(format!("last_outbound_at not recorded: {} rows", report.rows_affected()));
                }
            } else if let Err(e) = stamp_res {
                state_errors.push(format!("last_outbound_at not recorded: {}", e));
            }

            dispatched_ids.push(cand.id.clone());
            if !state_errors.is_empty() { followup_failures += 1; }
        } else {
            failures += 1;
        }

        results.push(json!({
            "conversationId": cand.id, "messageId": msg_id,
            "status": if dispatched { "dispatched" } else { "stored_not_dispatched" },
            "dispatch": outcome, "stateWriteErrors": state_errors,
        }));
    }

    let audit_id = Uuid::new_v4().to_string();
    let record = json!({
        "window_start": window_start.to_rfc3339(), "window_end": window_end.to_rfc3339(),
        "dispatched": dispatched_ids.len(), "not_dispatched": failures,
        "followup_state_write_failures": followup_failures,
        "results": results, "run_at": now_str.clone(),
    });
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, severity, compliance_category, description, details, created_at) VALUES ($1, 'automation', 'inactivity_nudge', 'nudge_run', 'automation_engine', 'info', 'audit_trail', $2, $3, $4)",
    )
    .bind(&audit_id)
    .bind(format!("Inactivity nudge run: {} dispatched, {} not dispatched", dispatched_ids.len(), failures))
    .bind(serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string()))
    .bind(&now_str)
    .execute(&state.pool).await;

    let status = if failures == 0 && followup_failures == 0 && !dispatched_ids.is_empty() { "ok" }
        else if dispatched_ids.is_empty() && failures == 0 && followup_failures == 0 { "no_candidates" }
        else { "partial" };

    Ok(json!({
        "status": status, "nudgedCount": dispatched_ids.len(), "conversationIds": dispatched_ids,
        "notDispatchedCount": failures, "followupStateWriteFailures": followup_failures,
        "results": results,
        "windowStart": window_start.to_rfc3339(), "windowEnd": window_end.to_rfc3339(),
        "executionRecordId": audit_id, "executionRecordStored": audit_res.is_ok(),
    }))
}
