//! Copilot for Human Reviewers — library crate.
//!
//! AI-powered context generation for draft approval reviews:
//! conversation summaries, customer profile enrichment, and suggested edits.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, FromRow};
use uuid::Uuid;
use chrono::Utc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotContext {
    pub id: String,
    pub draft_id: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub conversation_summary: Option<String>,
    pub customer_profile: Option<CustomerProfile>,
    pub suggested_edits: Vec<EditSuggestion>,
    pub key_topics: Vec<String>,
    pub customer_sentiment: Option<String>,
    pub generated: bool,
    pub generated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProfile {
    pub contact_name: Option<String>,
    pub contact_value: Option<String>,
    pub channel: Option<String>,
    pub role: Option<String>,
    pub language: Option<String>,
    pub lead_name: Option<String>,
    pub lead_institution_type: Option<String>,
    pub lead_city: Option<String>,
    pub lead_qualification: Option<String>,
    pub conversation_count: i64,
    pub message_count: i64,
    pub first_contact_at: Option<String>,
    pub last_message_at: Option<String>,
    pub conversation_state: Option<String>,
    pub consent_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSuggestion {
    pub kind: String,
    pub description: String,
    pub suggested_text: Option<String>,
    pub priority: String,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct CopilotReviewerState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
}

// ---------------------------------------------------------------------------
// Claude client types (embedded to avoid depending on backend-rs claude_client)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    system: Option<String>,
    messages: Vec<WireMessage>,
    temperature: Option<f64>,
    #[allow(dead_code)]
    expect_json: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WireMessage {
    role: String,
    content: String,
}

struct ClaudeReply {
    text: String,
}

async fn claude_complete(
    http_client: &reqwest::Client,
    api_key: &str,
    req: &MessageRequest,
) -> Result<ClaudeReply, String> {
    let mut body = serde_json::Map::new();
    body.insert("model".into(), serde_json::json!(req.model));
    body.insert("max_tokens".into(), serde_json::json!(req.max_tokens));
    if let Some(ref sys) = req.system {
        body.insert("system".into(), serde_json::json!(sys));
    }
    body.insert("messages".into(), serde_json::to_value(&req.messages).unwrap_or_default());
    if let Some(temp) = req.temperature {
        body.insert("temperature".into(), serde_json::json!(temp));
    }

    let resp = http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("Claude request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Claude returned HTTP {}", status.as_u16()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let text = json.get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    Ok(ClaudeReply { text })
}

// ---------------------------------------------------------------------------
// Core copilot context generation
// ---------------------------------------------------------------------------

struct DraftInfo {
    conversation_id: String,
    contact_id: String,
    body: String,
}

async fn fetch_draft_info(state: &CopilotReviewerState, draft_id: &str) -> Result<DraftInfo, String> {
    #[derive(FromRow)]
    struct DraftRow {
        conversation_id: String,
        body: String,
    }

    let row: DraftRow = sqlx::query_as(
        "SELECT conversation_id, body FROM drafts WHERE id = $1",
    )
    .bind(draft_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Draft not found: {}", e))?;

    let contact_id: String = sqlx::query_scalar(
        "SELECT contact_id FROM conversations WHERE id = $1",
    )
    .bind(&row.conversation_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Conversation not found: {}", e))?;

    Ok(DraftInfo {
        conversation_id: row.conversation_id,
        contact_id,
        body: row.body,
    })
}

async fn fetch_cached_context(state: &CopilotReviewerState, draft_id: &str) -> Option<CopilotContext> {
    let row = sqlx::query(
        "SELECT id, draft_id, conversation_id, contact_id, conversation_summary,
         customer_profile, suggested_edits, key_topics, customer_sentiment,
         generated, generated_at, created_at, updated_at
         FROM copilot_context WHERE draft_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(draft_id)
    .fetch_optional(&state.db)
    .await
    .ok()?;

    let row = row?;

    let customer_profile: Option<CustomerProfile> = row.try_get("customer_profile").ok()
        .and_then(|s: String| serde_json::from_str(&s).ok());

    let suggested_edits: Vec<EditSuggestion> = row.try_get("suggested_edits").ok()
        .and_then(|s: String| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let key_topics: Vec<String> = row.try_get("key_topics").ok()
        .and_then(|s: String| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Some(CopilotContext {
        id: row.get("id"),
        draft_id: row.get("draft_id"),
        conversation_id: row.get("conversation_id"),
        contact_id: row.get("contact_id"),
        conversation_summary: row.try_get("conversation_summary").ok(),
        customer_profile,
        suggested_edits,
        key_topics,
        customer_sentiment: row.try_get("customer_sentiment").ok(),
        generated: row.get("generated"),
        generated_at: row.try_get("generated_at").ok(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn is_recent(timestamp: &str) -> bool {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        now.signed_duration_since(dt.to_utc()).num_minutes() < 60
    } else {
        false
    }
}

async fn cache_context(state: &CopilotReviewerState, ctx: &CopilotContext) -> Result<(), String> {
    let profile_json = ctx.customer_profile
        .as_ref()
        .and_then(|p| serde_json::to_string(p).ok());

    let edits_json = if ctx.suggested_edits.is_empty() {
        None
    } else {
        serde_json::to_string(&ctx.suggested_edits).ok()
    };

    let topics_json = if ctx.key_topics.is_empty() {
        None
    } else {
        serde_json::to_string(&ctx.key_topics).ok()
    };

    sqlx::query(
        "INSERT INTO copilot_context (id, draft_id, conversation_id, contact_id,
         conversation_summary, customer_profile, suggested_edits, key_topics,
         customer_sentiment, generated, generated_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(&ctx.id)
    .bind(&ctx.draft_id)
    .bind(&ctx.conversation_id)
    .bind(&ctx.contact_id)
    .bind(ctx.conversation_summary.as_deref())
    .bind(profile_json.as_deref())
    .bind(edits_json.as_deref())
    .bind(topics_json.as_deref())
    .bind(ctx.customer_sentiment.as_deref())
    .bind(ctx.generated as i32)
    .bind(ctx.generated_at.as_deref())
    .bind(&ctx.created_at)
    .bind(&ctx.updated_at)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to cache copilot context: {}", e))?;

    Ok(())
}

/// Get or generate copilot context for a draft.
pub async fn get_copilot_context_for_draft(
    state: &CopilotReviewerState,
    draft_id: &str,
) -> Result<CopilotContext, String> {
    let cached = fetch_cached_context(state, draft_id).await;
    if let Some(ctx) = cached {
        if let Some(gen_at) = &ctx.generated_at {
            if is_recent(gen_at) {
                return Ok(ctx);
            }
        }
    }

    let draft_info = fetch_draft_info(state, draft_id).await?;

    let (summary, topics, sentiment, edits) = generate_ai_context(
        state,
        &draft_info.conversation_id,
        &draft_info.body,
    )
    .await;

    let profile = build_customer_profile(state, &draft_info.contact_id, &draft_info.conversation_id)
        .await?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let generated = summary.is_some() || !edits.is_empty();

    let ctx = CopilotContext {
        id: id.clone(),
        draft_id: draft_id.to_string(),
        conversation_id: draft_info.conversation_id,
        contact_id: draft_info.contact_id,
        conversation_summary: summary.clone(),
        customer_profile: Some(profile),
        suggested_edits: edits.clone(),
        key_topics: topics.clone(),
        customer_sentiment: sentiment.clone(),
        generated,
        generated_at: if generated { Some(now.clone()) } else { None },
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    cache_context(state, &ctx).await?;

    Ok(ctx)
}

/// Delete cached context for a draft (used by refresh).
pub async fn delete_cached_context(
    state: &CopilotReviewerState,
    draft_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM copilot_context WHERE draft_id = $1")
        .bind(draft_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete cached context: {}", e))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// AI-powered context generation
// ---------------------------------------------------------------------------

struct ConversationMessage {
    role: String,
    body: String,
}

async fn fetch_conversation_messages(
    state: &CopilotReviewerState,
    conversation_id: &str,
) -> Vec<ConversationMessage> {
    #[derive(FromRow)]
    struct MessageRow {
        sender: Option<String>,
        direction: Option<String>,
        body: String,
    }

    let rows: Vec<MessageRow> = sqlx::query_as(
        "SELECT sender, direction, body FROM messages
         WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 30",
    )
    .bind(conversation_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut rows = rows;
    rows.reverse();

    rows.into_iter().map(|row| {
        let role = if row.sender.as_deref() == Some("customer")
            || row.direction.as_deref() == Some("inbound")
        {
            "customer"
        } else {
            "agent"
        }.to_string();

        ConversationMessage {
            role,
            body: row.body,
        }
    }).collect()
}

async fn generate_ai_context(
    state: &CopilotReviewerState,
    conversation_id: &str,
    draft_body: &str,
) -> (Option<String>, Vec<String>, Option<String>, Vec<EditSuggestion>) {
    let messages = fetch_conversation_messages(state, conversation_id).await;

    if messages.is_empty() {
        return (None, Vec::new(), None, Vec::new());
    }

    let mut transcript_lines = Vec::new();
    for msg in &messages {
        let role_label = if msg.role == "customer" { "Customer" } else { "Agent" };
        transcript_lines.push(format!("{}: {}", role_label, msg.body));
    }
    let transcript = transcript_lines.join("\n");

    let (summary, topics, sentiment) = generate_summary_topics_sentiment(state, &transcript).await;
    let edits = generate_suggested_edits(state, &transcript, draft_body).await;

    (summary, topics, sentiment, edits)
}

async fn generate_summary_topics_sentiment(
    state: &CopilotReviewerState,
    transcript: &str,
) -> (Option<String>, Vec<String>, Option<String>) {
    let api_key = std::env::var("CLAUDE_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        tracing::warn!("CLAUDE_API_KEY not set, skipping copilot summary generation");
        return (None, Vec::new(), None);
    }

    let system_prompt = r#"You are an AI assistant that analyzes customer service conversations.
Produce a concise analysis in JSON format with three fields:
- "summary": A 2-3 sentence summary of the conversation, focusing on what the customer wants and where the conversation stands.
- "topics": An array of 3-5 key topic strings extracted from the conversation.
- "sentiment": The customer's overall sentiment: "positive", "neutral", "negative", or "frustrated".

Return ONLY valid JSON. No markdown, no explanation."#.to_string();

    let user_prompt = format!(
        "Analyze this conversation:\n---\n{}\n---",
        transcript.chars().take(8000).collect::<String>()
    );

    let request = MessageRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 1024,
        system: Some(system_prompt),
        messages: vec![
            WireMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: Some(0.3),
        expect_json: true,
    };

    match claude_complete(&state.http_client, &api_key, &request).await {
        Ok(reply) => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&reply.text) {
                let summary = parsed.get("summary").and_then(|v| v.as_str()).map(String::from);
                let topics: Vec<String> = parsed.get("topics")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let sentiment = parsed.get("sentiment").and_then(|v| v.as_str()).map(String::from);
                return (summary, topics, sentiment);
            }
        }
        Err(e) => {
            tracing::warn!("Claude unavailable for copilot summary: {}", e);
        }
    }

    (None, Vec::new(), None)
}

async fn generate_suggested_edits(
    state: &CopilotReviewerState,
    transcript: &str,
    draft_body: &str,
) -> Vec<EditSuggestion> {
    let api_key = std::env::var("CLAUDE_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        tracing::warn!("CLAUDE_API_KEY not set, skipping copilot edit suggestions");
        return Vec::new();
    }

    let system_prompt = r#"You are an AI writing coach reviewing a customer service response draft.
The draft will be sent to a customer. Review it and suggest specific improvements.

Focus on:
- "tone": Is the tone appropriate for the conversation context?
- "clarity": Is the message clear and easy to understand?
- "completeness": Does it address everything the customer asked?
- "accuracy": Are there any potentially misleading statements?
- "style": Grammar, formatting, and readability.

For each issue found, create a suggestion with:
- "kind": One of: tone, clarity, completeness, accuracy, style
- "description": What to change and why (1-2 sentences)
- "suggested_text": A specific text replacement (only if applicable)
- "priority": "high", "medium", or "low"

Return ONLY a JSON array of suggestion objects. If the draft is good, return an empty array."#.to_string();

    let user_prompt = format!(
        "Conversation context:\n---\n{}\n---\n\nDraft response:\n---\n{}\n---",
        transcript.chars().take(6000).collect::<String>(),
        draft_body
    );

    let request = MessageRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 2048,
        system: Some(system_prompt),
        messages: vec![
            WireMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: Some(0.3),
        expect_json: true,
    };

    match claude_complete(&state.http_client, &api_key, &request).await {
        Ok(reply) => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&reply.text) {
                if let Some(arr) = parsed.as_array() {
                    let mut edits = Vec::new();
                    for item in arr {
                        let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("clarity").to_string();
                        let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let suggested_text = item.get("suggested_text").and_then(|v| v.as_str()).map(String::from);
                        let priority = item.get("priority").and_then(|v| v.as_str()).unwrap_or("medium").to_string();

                        if !description.is_empty() {
                            edits.push(EditSuggestion {
                                kind,
                                description,
                                suggested_text,
                                priority,
                            });
                        }
                    }
                    return edits;
                }
            }
        }
        Err(e) => {
            tracing::warn!("Claude unavailable for copilot edits: {}", e);
        }
    }

    Vec::new()
}

// ---------------------------------------------------------------------------
// Customer profile enrichment
// ---------------------------------------------------------------------------

async fn build_customer_profile(
    state: &CopilotReviewerState,
    contact_id: &str,
    conversation_id: &str,
) -> Result<CustomerProfile, String> {
    #[derive(FromRow)]
    struct ContactRow {
        person_name: Option<String>,
        value: String,
        channel: String,
        role: Option<String>,
        language: Option<String>,
    }

    let contact: ContactRow = sqlx::query_as(
        "SELECT person_name, value, channel, role, language FROM contacts WHERE id = $1",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch contact: {}", e))?
    .unwrap_or_else(|| ContactRow {
        person_name: None,
        value: String::new(),
        channel: String::new(),
        role: None,
        language: None,
    });

    let lead_name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = $1)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let lead_institution_type: Option<String> = sqlx::query_scalar(
        "SELECT institution_type FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = $1)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let lead_city: Option<String> = sqlx::query_scalar(
        "SELECT city FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = $1)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let lead_qualification: Option<String> = sqlx::query_scalar(
        "SELECT qualification_rationale FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = $1)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let conversation_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversations WHERE contact_id = $1",
    )
    .bind(contact_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let message_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM messages WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let first_contact_at: Option<String> = sqlx::query_scalar(
        "SELECT MIN(created_at) FROM messages WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let last_message_at: Option<String> = sqlx::query_scalar(
        "SELECT MAX(created_at) FROM messages WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let conversation_state: Option<String> = sqlx::query_scalar(
        "SELECT state FROM conversations WHERE id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let consent_status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM consent_records WHERE contact_id = $1 ORDER BY recorded_at DESC LIMIT 1",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    Ok(CustomerProfile {
        contact_name: contact.person_name,
        contact_value: if contact.value.is_empty() { None } else { Some(contact.value) },
        channel: if contact.channel.is_empty() { None } else { Some(contact.channel) },
        role: contact.role,
        language: contact.language,
        lead_name,
        lead_institution_type,
        lead_city,
        lead_qualification,
        conversation_count,
        message_count,
        first_contact_at,
        last_message_at,
        conversation_state,
        consent_status,
    })
}

// ---------------------------------------------------------------------------
// Batch operations
// ---------------------------------------------------------------------------

/// Get copilot context for all pending review drafts.
pub async fn get_pending_drafts_context(
    state: &CopilotReviewerState,
) -> Result<Vec<CopilotContext>, String> {
    let draft_ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM drafts WHERE status = 'pending_review' ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch pending drafts: {}", e))?;

    let mut contexts = Vec::new();
    for draft_id in draft_ids {
        if let Ok(ctx) = get_copilot_context_for_draft(state, &draft_id).await {
            contexts.push(ctx);
        }
    }

    Ok(contexts)
}
