use crate::{db::AppState, models::AutomationRule};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub async fn list_automation_rules(State(state): State<AppState>) -> impl IntoResponse {
    let rules = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules ORDER BY priority DESC, created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(rules)
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
    Json(payload): Json<CreateRulePayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let priority = payload.priority.unwrap_or(0);
    let is_active = payload.is_active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        "INSERT INTO automation_rules (id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, created_at, tenant_key, channel) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, 'newax', '')",
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
    Path(id): Path<String>,
    Json(payload): Json<UpdateRulePayload>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE id = ?",
    )
    .bind(&id)
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
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM automation_rules WHERE id = ?")
        .bind(&id)
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

pub async fn seed_default_automations(State(state): State<AppState>) -> impl IntoResponse {
    let existing_keywords: Vec<String> = sqlx::query_scalar("SELECT trigger_keyword FROM automation_rules")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let existing_set: std::collections::HashSet<String> = existing_keywords
        .into_iter()
        .map(|k| k.trim().to_lowercase())
        .collect();

    let defaults = vec![
        (
            "Main Interactive Menu",
            "Opens the main navigation list menu with services, pricing, and live support",
            "menu",
            "exact",
            "interactive_buttons",
            r#"{"body":"Welcome to our WhatsApp Center! Please select an option to get started immediately:","buttons":[{"id":"menu:services","title":"Browse Services"},{"id":"menu:pricing","title":"Pricing & Plans"},{"id":"action:agent","title":"Talk to Human"}],"header":"Main Menu","footer":"Select an option or reply with your question"}"#,
            100,
        ),
        (
            "Live Human Support Handoff",
            "Transfers conversation to live human agent and pauses the bot",
            "agent",
            "contains",
            "handoff",
            "Customer requested human agent assistance.",
            95,
        ),
        (
            "Human Agent Alias (Operator)",
            "Transfers conversation to live human agent when typing 'human' or 'operator'",
            "human",
            "contains",
            "handoff",
            "Customer requested human agent.",
            94,
        ),
        (
            "Services Catalog Quick View",
            "Displays interactive catalog of our core services",
            "services",
            "contains",
            "interactive_buttons",
            r#"{"body":"We specialize in custom web software, AI automations, and digital marketing systems.","buttons":[{"id":"menu:pricing","title":"Pricing & Rates"},{"id":"action:agent","title":"Speak with Sales"},{"id":"menu:main","title":"Main Menu"}],"header":"Services Catalog"}"#,
            85,
        ),
        (
            "Pricing & Rates Inquiry",
            "Provides transparent pricing breakdown and quote request options",
            "pricing",
            "contains",
            "interactive_buttons",
            r#"{"body":"Our solutions start with flexible monthly plans, or fixed-price milestones with 100% upfront quotes.","buttons":[{"id":"action:agent","title":"Get Custom Quote"},{"id":"menu:services","title":"Our Services"},{"id":"menu:main","title":"Main Menu"}],"header":"Pricing Information"}"#,
            80,
        ),
        (
            "Book a Meeting / Demo CTA",
            "Sends clickable Call-To-Action link to book a discovery call",
            "demo",
            "contains",
            "send_cta_url",
            r#"{"body":"Ready to discuss your project? Schedule a 15-minute discovery call directly on our calendar:","display_text":"Book Free Demo","url":"https://calendly.com","header":"Schedule Discovery"}"#,
            75,
        ),
        (
            "Office Location & Timings",
            "Shares office address and operating hours",
            "location",
            "contains",
            "auto_reply",
            "Office Address: Main Commercial Hub\nWorking Hours: Monday - Friday, 9:00 AM - 6:00 PM\nWhatsApp inquiries are accepted 24/7!",
            70,
        ),
        (
            "Post-Resolution Feedback / CSAT Survey",
            "Prompts customer with interactive star rating upon resolution",
            "csat",
            "exact",
            "send_csat",
            "How was your experience with our team today?",
            65,
        ),
        (
            "High-Intent Lead Auto-Tagging",
            "Automatically tags lead as 'high_intent' when asking about contracts or quotes",
            "quote",
            "contains",
            "add_tag",
            "high_intent",
            60,
        ),
        (
            "VIP Customer Auto-Tagging",
            "Automatically tags lead as 'vip' when enterprise inquiry occurs",
            "enterprise",
            "contains",
            "add_tag",
            "enterprise_lead",
            55,
        ),
        (
            "Opt-Out Compliance (STOP)",
            "Automated opt-out compliance handler when user requests to stop receiving messages",
            "stop",
            "exact",
            "auto_reply",
            "You have been successfully opted out from automated WhatsApp updates. Reply START at any time to resume.",
            110,
        ),
    ];

    let mut installed_count = 0;
    let now = Utc::now().to_rfc3339();

    for (name, desc, kw, match_t, action_t, action_v, prio) in defaults {
        if !existing_set.contains(&kw.to_lowercase()) {
            let id = Uuid::new_v4().to_string();
            let _ = sqlx::query(
                "INSERT INTO automation_rules (id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, created_at, tenant_key, channel) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, 0, ?, 'newax', '')",
            )
            .bind(&id)
            .bind(name)
            .bind(desc)
            .bind(kw)
            .bind(match_t)
            .bind(action_t)
            .bind(action_v)
            .bind(prio)
            .bind(&now)
            .execute(&state.db)
            .await;

            installed_count += 1;
        }
    }

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM automation_rules")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Json(json!({
        "status": "ok",
        "installedCount": installed_count,
        "message": if installed_count > 0 {
            format!("Successfully installed {} default WhatsApp automations (Total: {}).", installed_count, total)
        } else {
            format!("All 11 standard WhatsApp automations already installed (Total: {}).", total)
        }
    }))
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
    Json(payload): Json<TestRulePayload>,
) -> impl IntoResponse {
    let text_norm = payload.text.trim().to_lowercase();
    let payload_norm = payload.payload_id.as_deref().unwrap_or("").trim().to_lowercase();
    let intent_norm = payload.intent.as_deref().unwrap_or("").trim().to_lowercase();

    let rules = sqlx::query_as::<_, AutomationRule>(
        "SELECT id, name, description, trigger_keyword, match_type, action_type, action_value, priority, is_active, execution_count, last_triggered_at, created_at FROM automation_rules WHERE is_active = 1 ORDER BY priority DESC, created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for r in rules {
        let kw = r.trigger_keyword.trim().to_lowercase();
        if kw.is_empty() {
            continue;
        }

        let matched = match r.match_type.as_str() {
            "exact" => text_norm == kw,
            "contains" => !text_norm.is_empty() && text_norm.contains(&kw),
            "starts_with" => !text_norm.is_empty() && text_norm.starts_with(&kw),
            "regex" => {
                if let Ok(re) = regex::RegexBuilder::new(&r.trigger_keyword).case_insensitive(true).build() {
                    re.is_match(&payload.text)
                } else {
                    false
                }
            }
            "button_click" | "list_select" => {
                payload_norm == kw || (!payload_norm.is_empty() && payload_norm.contains(&kw)) || text_norm == kw
            }
            "intent" => {
                !intent_norm.is_empty() && intent_norm == kw
            }
            _ => !text_norm.is_empty() && text_norm.contains(&kw),
        };

        if matched {
            let preview: serde_json::Value = serde_json::from_str(&r.action_value)
                .unwrap_or_else(|_| json!({}));

            return Json(json!({
                "matched": true,
                "rule": {
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
                },
                "actionType": r.action_type,
                "actionValue": r.action_value,
                "preview": preview
            }));
        }
    }

    Json(json!({
        "matched": false,
        "message": "No automation rule matched. AI reply engine will handle this message."
    }))
}

pub async fn get_automation_stats(State(state): State<AppState>) -> impl IntoResponse {
    let total_rules: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_rules")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let active_rules: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_rules WHERE is_active = 1")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_executions: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(execution_count), 0) FROM automation_rules")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let csat_row: (i64, Option<f64>) = sqlx::query_as(
        "SELECT COUNT(*), AVG(csat_score) FROM conversations WHERE csat_score IS NOT NULL",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((0, None));

    let csat_responses_count = csat_row.0;
    let csat_average = csat_row.1.map(|avg| (avg * 100.0).round() / 100.0);

    // Dynamically estimated: ~5 minutes saved per automation rule execution
    let time_saved_hours = ((total_executions as f64 * 5.0) / 60.0 * 10.0).round() / 10.0;

    let nudges_sent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM messages WHERE provider_message_id = 'inactivity_nudge' OR intent = 'inactivity_nudge'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Json(json!({
        "totalRules": total_rules,
        "activeRules": active_rules,
        "totalExecutions": total_executions,
        "csatAverage": csat_average,
        "csatResponsesCount": csat_responses_count,
        "timeSavedHours": time_saved_hours,
        "nudgesSent": nudges_sent
    }))
}

#[derive(sqlx::FromRow)]
struct NudgeCandidate {
    id: String,
    last_inbound_at: Option<String>,
}

pub async fn run_inactivity_nudges(State(state): State<AppState>) -> impl IntoResponse {
    let now = Utc::now();
    let window_start = now - chrono::Duration::hours(24);
    let window_end = now - chrono::Duration::hours(18);

    let candidates = sqlx::query_as::<_, NudgeCandidate>(
        "SELECT id, last_inbound_at FROM conversations WHERE (state = 'ENGAGED' OR state = 'AWAITING_REPLY' OR state = 'engaged' OR state = 'awaiting_reply') AND (human_takeover = 0 OR human_takeover IS NULL) AND last_inbound_at IS NOT NULL",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut nudged_ids = Vec::new();

    for cand in candidates {
        let should_nudge = if let Some(ref inbound_str) = cand.last_inbound_at {
            if let Ok(inbound_dt) = chrono::DateTime::parse_from_rfc3339(inbound_str) {
                let inbound_utc = inbound_dt.with_timezone(&Utc);
                inbound_utc >= window_start && inbound_utc <= window_end
            } else {
                false
            }
        } else {
            false
        };

        if should_nudge {
            // Check if already nudged in last 24h
            let recent_nudge: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM messages WHERE conversation_id = ? AND (provider_message_id = 'inactivity_nudge' OR intent = 'inactivity_nudge') AND created_at >= ?",
            )
            .bind(&cand.id)
            .bind(window_start.to_rfc3339())
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

            if recent_nudge == 0 {
                let msg_id = Uuid::new_v4().to_string();
                let body = "Hi there, just checking in to see if you needed any more assistance or had further questions before our active chat session wraps up!";
                let now_str = now.to_rfc3339();

                let _ = sqlx::query(
                    "INSERT INTO messages (id, conversation_id, sender, body, channel, delivery_status, created_at, provider_message_id, intent) VALUES (?, ?, 'bot', ?, 'whatsapp', 'sent', ?, 'inactivity_nudge', 'inactivity_nudge')",
                )
                .bind(&msg_id)
                .bind(&cand.id)
                .bind(body)
                .bind(&now_str)
                .execute(&state.db)
                .await;

                let _ = sqlx::query("UPDATE conversations SET last_outbound_at = ? WHERE id = ?")
                    .bind(&now_str)
                    .bind(&cand.id)
                    .execute(&state.db)
                    .await;

                nudged_ids.push(cand.id);
            }
        }
    }

    Json(json!({
        "status": "ok",
        "nudgedCount": nudged_ids.len(),
        "conversationIds": nudged_ids
    }))
}
