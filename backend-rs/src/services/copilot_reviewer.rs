#![allow(dead_code)]

//! Copilot for Human Reviewers — Phase 10 of the self-improvement flywheel.
//!
//! When a draft sits in the approval queue, the human reviewer sees not just
//! the raw draft text, but an AI-generated context package:
//!
//! 1. **Conversation Summary** — Claude condenses the full conversation history
//!    into a brief summary so the reviewer understands context quickly.
//! 2. **Customer Profile Enrichment** — Lead info, contact details, conversation
//!    statistics, and historical engagement data.
//! 3. **Suggested Edits** — Claude reviews the draft body and suggests specific
//!    improvements (tone, clarity, missing information, etc.).
//!
//! Design decisions:
//! - Context is generated on demand when the reviewer opens a draft, not
//!   pre-computed, to ensure freshness.
//! - Results are cached in `copilot_context` table to avoid regenerating
//!   on every page refresh.
//! - If Claude is unavailable, the service returns whatever data it can
//!   collect (profile enrichment works without AI).

use crate::db::AppState;
use crate::services::claude_client::{self, MessageRequest};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Row, FromRow};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

/// Complete copilot context for a draft approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotContext {
    pub id: String,
    pub draft_id: String,
    pub conversation_id: String,
    pub contact_id: String,

    /// AI-generated summary of the conversation history.
    pub conversation_summary: Option<String>,

    /// Enriched customer profile data.
    pub customer_profile: Option<CustomerProfile>,

    /// Suggested edits for the draft.
    pub suggested_edits: Vec<EditSuggestion>,

    /// Key topics extracted from the conversation.
    pub key_topics: Vec<String>,

    /// Customer sentiment in the conversation.
    pub customer_sentiment: Option<String>,

    /// Whether the AI-generated parts are available.
    pub generated: bool,

    pub generated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Enriched customer profile with lead info, contact details, and stats.
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

/// A suggested edit for improving the draft.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSuggestion {
    pub kind: String,       // 'tone', 'clarity', 'completeness', 'accuracy', 'style'
    pub description: String, // What to change and why
    pub suggested_text: Option<String>, // Specific text replacement suggestion
    pub priority: String,    // 'high', 'medium', 'low'
}

// ---------------------------------------------------------------------------
// Core copilot context generation
// ---------------------------------------------------------------------------

/// Get or generate copilot context for a draft.
/// If context already exists and is recent (< 1 hour), return cached version.
/// Otherwise, generate fresh context.
pub async fn get_copilot_context_for_draft(
    state: &AppState,
    draft_id: &str,
) -> Result<CopilotContext, String> {
    // Check for existing cached context
    let cached = fetch_cached_context(state, draft_id).await;
    if let Some(ctx) = cached {
        // Use cached if generated within the last hour
        if let Some(gen_at) = &ctx.generated_at {
            if is_recent(gen_at) {
                return Ok(ctx);
            }
        }
    }

    // Fetch the draft to get conversation_id and contact_id
    let draft_info = fetch_draft_info(state, draft_id).await?;

    // Generate context
    let (summary, topics, sentiment, edits) = generate_ai_context(
        state,
        &draft_info.conversation_id,
        &draft_info.body,
    )
    .await;

    // Build customer profile (doesn't require AI)
    let profile = build_customer_profile(state, &draft_info.contact_id, &draft_info.conversation_id)
        .await?;

    // Assemble the context
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

    // Cache in database
    cache_context(state, &ctx).await?;

    Ok(ctx)
}

/// Internal struct to hold draft info needed for context generation.
struct DraftInfo {
    conversation_id: String,
    contact_id: String,
    body: String,
}

/// Fetch draft info from the database.
async fn fetch_draft_info(state: &AppState, draft_id: &str) -> Result<DraftInfo, String> {
    #[derive(FromRow)]
    struct DraftRow {
        conversation_id: String,
        body: String,
    }

    let row: DraftRow = sqlx::query_as(
        "SELECT conversation_id, body FROM drafts WHERE id = ?",
    )
    .bind(draft_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Draft not found: {}", e))?;

    // Get contact_id from conversation
    let contact_id: String = sqlx::query_scalar(
        "SELECT contact_id FROM conversations WHERE id = ?",
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

/// Fetch cached copilot context if it exists.
async fn fetch_cached_context(state: &AppState, draft_id: &str) -> Option<CopilotContext> {
    let row = sqlx::query(
        "SELECT id, draft_id, conversation_id, contact_id, conversation_summary,
         customer_profile, suggested_edits, key_topics, customer_sentiment,
         generated, generated_at, created_at, updated_at
         FROM copilot_context WHERE draft_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(draft_id)
    .fetch_optional(&state.db)
    .await?;

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

/// Check if a timestamp is within the last hour.
fn is_recent(timestamp: &str) -> bool {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        now.signed_duration_since(dt.to_utc()).num_minutes() < 60
    } else {
        false
    }
}

/// Cache copilot context in the database.
async fn cache_context(state: &AppState, ctx: &CopilotContext) -> Result<(), String> {
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
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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

// ---------------------------------------------------------------------------
// AI-powered context generation
// ---------------------------------------------------------------------------

/// Generate AI-powered context: summary, topics, sentiment, and suggested edits.
async fn generate_ai_context(
    state: &AppState,
    conversation_id: &str,
    draft_body: &str,
) -> (Option<String>, Vec<String>, Option<String>, Vec<EditSuggestion>) {
    // Fetch conversation messages
    let messages = fetch_conversation_messages(state, conversation_id).await;

    if messages.is_empty() {
        return (None, Vec::new(), None, Vec::new());
    }

    // Build transcript
    let mut transcript_lines = Vec::new();
    for msg in &messages {
        let role_label = if msg.role == "customer" { "Customer" } else { "Agent" };
        transcript_lines.push(format!("{}: {}", role_label, msg.body));
    }
    let transcript = transcript_lines.join("\n");

    // Generate summary, topics, sentiment in one call
    let (summary, topics, sentiment) = generate_summary_topics_sentiment(state, &transcript).await;

    // Generate suggested edits in a separate call
    let edits = generate_suggested_edits(state, &transcript, draft_body).await;

    (summary, topics, sentiment, edits)
}

#[derive(Debug, Clone)]
struct ConversationMessage {
    role: String,
    body: String,
}

/// Fetch the last 30 messages from a conversation.
async fn fetch_conversation_messages(state: &AppState, conversation_id: &str) -> Vec<ConversationMessage> {
    #[derive(FromRow)]
    struct MessageRow {
        sender: Option<String>,
        direction: Option<String>,
        body: String,
    }

    let rows: Vec<MessageRow> = sqlx::query_as(
        "SELECT sender, direction, body FROM messages
         WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 30",
    )
    .bind(conversation_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Reverse to get chronological order
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

/// Generate conversation summary, key topics, and sentiment using Claude.
async fn generate_summary_topics_sentiment(
    state: &AppState,
    transcript: &str,
) -> (Option<String>, Vec<String>, Option<String>) {
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
            claude_client::WireMessage {
                role: "user",
                content: user_prompt,
            },
        ],
        temperature: Some(0.3),
        expect_json: true,
    };

    match claude_client::complete(state, &request).await {
        Ok(reply) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(&reply.text) {
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

/// Generate suggested edits for the draft using Claude.
async fn generate_suggested_edits(
    state: &AppState,
    transcript: &str,
    draft_body: &str,
) -> Vec<EditSuggestion> {
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
            claude_client::WireMessage {
                role: "user",
                content: user_prompt,
            },
        ],
        temperature: Some(0.3),
        expect_json: true,
    };

    match claude_client::complete(state, &request).await {
        Ok(reply) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(&reply.text) {
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
// Customer profile enrichment (no AI required)
// ---------------------------------------------------------------------------

/// Build enriched customer profile from database data.
async fn build_customer_profile(
    state: &AppState,
    contact_id: &str,
    conversation_id: &str,
) -> Result<CustomerProfile, String> {
    // Contact info
    #[derive(FromRow)]
    struct ContactRow {
        person_name: Option<String>,
        value: String,
        channel: String,
        role: Option<String>,
        language: Option<String>,
    }

    let contact: ContactRow = sqlx::query_as(
        "SELECT person_name, value, channel, role, language FROM contacts WHERE id = ?",
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

    // Lead info
    let lead_name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = ?)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let lead_institution_type: Option<String> = sqlx::query_scalar(
        "SELECT institution_type FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = ?)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let lead_city: Option<String> = sqlx::query_scalar(
        "SELECT city FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = ?)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let lead_qualification: Option<String> = sqlx::query_scalar(
        "SELECT qualification_rationale FROM leads WHERE id = (SELECT lead_id FROM contacts WHERE id = ?)",
    )
    .bind(contact_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    // Conversation stats
    let conversation_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversations WHERE contact_id = ?",
    )
    .bind(contact_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let message_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM messages WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let first_contact_at: Option<String> = sqlx::query_scalar(
        "SELECT MIN(created_at) FROM messages WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let last_message_at: Option<String> = sqlx::query_scalar(
        "SELECT MAX(created_at) FROM messages WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let conversation_state: Option<String> = sqlx::query_scalar(
        "SELECT state FROM conversations WHERE id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let consent_status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM consent_records WHERE contact_id = ? ORDER BY recorded_at DESC LIMIT 1",
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
    state: &AppState,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_profile_serialization() {
        let profile = CustomerProfile {
            contact_name: Some("John Doe".to_string()),
            contact_value: Some("john@example.com".to_string()),
            channel: Some("email".to_string()),
            role: Some("Decision Maker".to_string()),
            language: Some("en".to_string()),
            lead_name: Some("Acme Corp".to_string()),
            lead_institution_type: Some("University".to_string()),
            lead_city: Some("New York".to_string()),
            lead_qualification: Some("High interest in LMS".to_string()),
            conversation_count: 3,
            message_count: 15,
            first_contact_at: Some("2025-01-01T00:00:00Z".to_string()),
            last_message_at: Some("2025-01-15T00:00:00Z".to_string()),
            conversation_state: Some("engaged".to_string()),
            consent_status: Some("opted_in".to_string()),
        };

        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("John Doe"));
        assert!(json.contains("Acme Corp"));
    }

    #[test]
    fn test_edit_suggestion_serialization() {
        let edit = EditSuggestion {
            kind: "tone".to_string(),
            description: "The tone is too formal for this conversation".to_string(),
            suggested_text: Some("Consider a friendlier greeting".to_string()),
            priority: "high".to_string(),
        };

        let json = serde_json::to_string(&edit).unwrap();
        assert!(json.contains("tone"));
    }

    #[test]
    fn test_is_recent() {
        let now = Utc::now().to_rfc3339();
        assert!(is_recent(&now));

        let old = "2020-01-01T00:00:00+00:00";
        assert!(!is_recent(old));

        let invalid = "not-a-date";
        assert!(!is_recent(invalid));
    }
}
