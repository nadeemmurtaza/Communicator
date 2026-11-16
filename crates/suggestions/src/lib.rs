//! AI-Powered Suggestions — Phase 9 of the self-improvement flywheel.
//!
//! Connects the feedback loop to knowledge gap detection and
//! agent insight generation. When a customer gives negative feedback
//! (thumbs down), the system automatically analyzes the conversation
//! and suggests new KB entries, FAQ additions, and agent insights.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct SuggestionsState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
    pub suggested_content: Option<String>,
    pub source_feedback_id: Option<String>,
    pub source_conversation_id: Option<String>,
    pub source_message_body: Option<String>,
    pub confidence: f64,
    pub category: Option<String>,
    pub status: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionSummary {
    pub total_suggestions: i64,
    pub open_suggestions: i64,
    pub approved_suggestions: i64,
    pub dismissed_suggestions: i64,
    pub by_type: Vec<SuggestionTypeCount>,
    pub by_category: Vec<SuggestionCategoryCount>,
    pub recent_suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionTypeCount {
    pub suggestion_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionCategoryCount {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisResult {
    pub feedback_analyzed: usize,
    pub suggestions_created: usize,
    pub errors: usize,
    pub error_details: Vec<String>,
}

// ---------------------------------------------------------------------------
// Minimal Claude client (extracted from claude_client)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize)]
struct WireMessage {
    role: &'static str,
    content: String,
}

#[derive(Clone, Debug, serde::Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing)]
    expect_json: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(default)]
    content: Vec<ContentBlock>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    usage: Option<ClaudeUsage>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ContentBlock {
    #[serde(rename = "type", default)]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ClaudeUsage {
    #[serde(default)]
    input_tokens: Option<i64>,
    #[serde(default)]
    output_tokens: Option<i64>,
}

struct ClaudeReply {
    text: String,
    provider_message_id: String,
}

async fn claude_complete(
    client: &reqwest::Client,
    api_key: &str,
    request: &MessageRequest,
) -> Result<ClaudeReply, String> {
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await
        .map_err(|e| format!("Claude request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Claude returned {}: {}", status.as_u16(), text));
    }

    let body: ClaudeResponse = resp.json().await.map_err(|e| format!("Failed to parse Claude response: {}", e))?;

    let text = body.content.iter()
        .filter_map(|b| b.text.clone())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(ClaudeReply {
        text,
        provider_message_id: body.id,
    })
}

// ---------------------------------------------------------------------------
// Core suggestion generation
// ---------------------------------------------------------------------------

/// Analyze negative feedback and generate suggestions automatically.
pub async fn analyze_negative_feedback(
    state: &SuggestionsState,
    feedback_id: &str,
) -> Result<Vec<Suggestion>, String> {
    #[derive(sqlx::FromRow)]
    struct FeedbackRow {
        id: String,
        message_id: String,
        conversation_id: String,
        rating: i32,
        comment: Option<String>,
        author_type: String,
        created_at: String,
    }

    let feedback: FeedbackRow = sqlx::query_as(
        "SELECT id, message_id, conversation_id, rating, comment, author_type, created_at
         FROM message_feedback WHERE id = $1",
    )
    .bind(feedback_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Feedback not found: {}", e))?;

    if feedback.rating != -1 {
        return Ok(Vec::new());
    }

    let conversation_context = get_conversation_context(state, &feedback.conversation_id).await?;

    let existing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM suggestions WHERE source_feedback_id = $1",
    )
    .bind(feedback_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    if existing > 0 {
        return Ok(Vec::new());
    }

    let suggestions = generate_suggestions_with_ai(
        state,
        &conversation_context,
        &feedback.comment,
        &feedback.author_type,
    )
    .await?;

    let mut created = Vec::new();
    for suggestion in suggestions {
        let stored = store_suggestion(
            state,
            &suggestion,
            &feedback.id,
            &feedback.conversation_id,
            &conversation_context.last_customer_message,
        )
        .await?;
        created.push(stored);
    }

    Ok(created)
}

struct ConversationContext {
    pub messages: Vec<ConversationMessage>,
    pub last_customer_message: Option<String>,
    pub contact_name: Option<String>,
    pub contact_channel: Option<String>,
}

struct ConversationMessage {
    pub role: String,
    pub body: String,
    pub created_at: String,
}

async fn get_conversation_context(
    state: &SuggestionsState,
    conversation_id: &str,
) -> Result<ConversationContext, String> {
    #[derive(sqlx::FromRow)]
    struct MessageRow {
        id: String,
        body: String,
        sender: Option<String>,
        direction: Option<String>,
        created_at: String,
    }

    let messages: Vec<MessageRow> = sqlx::query_as(
        "SELECT id, body, sender, direction, created_at
         FROM messages WHERE conversation_id = $1
         ORDER BY created_at ASC LIMIT 50",
    )
    .bind(conversation_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch messages: {}", e))?;

    let mut conv_messages = Vec::new();
    let mut last_customer_msg = None;

    for msg in &messages {
        let role = if msg.sender.as_deref() == Some("customer")
            || msg.direction.as_deref() == Some("inbound")
        {
            "customer"
        } else {
            "agent"
        }.to_string();

        if role == "customer" {
            last_customer_msg = Some(msg.body.clone());
        }

        conv_messages.push(ConversationMessage {
            role,
            body: msg.body.clone(),
            created_at: msg.created_at.clone(),
        });
    }

    let contact_name: Option<String> = sqlx::query_scalar(
        "SELECT c.person_name FROM contacts c
         JOIN conversations cv ON cv.contact_id = c.id
         WHERE cv.id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let contact_channel: Option<String> = sqlx::query_scalar(
        "SELECT c.channel FROM contacts c
         JOIN conversations cv ON cv.contact_id = c.id
         WHERE cv.id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    Ok(ConversationContext {
        messages: conv_messages,
        last_customer_message: last_customer_msg,
        contact_name,
        contact_channel,
    })
}

async fn generate_suggestions_with_ai(
    state: &SuggestionsState,
    context: &ConversationContext,
    feedback_comment: &Option<String>,
    author_type: &str,
) -> Result<Vec<Value>, String> {
    let mut transcript_lines = Vec::new();
    for msg in &context.messages {
        let role_label = if msg.role == "customer" { "Customer" } else { "Agent" };
        transcript_lines.push(format!("{}: {}", role_label, msg.body));
    }
    let transcript = transcript_lines.join("\n");

    let last_msg = context.last_customer_message.as_deref().unwrap_or("");
    let comment = feedback_comment.as_deref().unwrap_or("");

    let system_prompt = r#"You are an AI assistant that analyzes customer conversations where the customer gave negative feedback (thumbs down) on the AI's response.

Your job is to generate actionable suggestions to improve the system. Analyze the conversation and determine:

1. Was there a knowledge gap? (The AI didn't know the answer)
2. Was there a missing FAQ? (A common question not in the catalog)
3. Was there an agent behavior issue? (The AI responded poorly)

For each issue found, create a suggestion with:
- type: "knowledge_gap", "faq_addition", or "agent_insight"
- title: A concise title for the suggestion
- description: What the issue is and why it matters
- suggested_content: For knowledge gaps, a draft answer. For FAQs, the Q/A pair. For insights, the improvement recommendation.
- confidence: 0.0-1.0, how confident you are this is a valid suggestion
- category: The topic category (e.g., "pricing", "technical", "policy", "product", "billing")

Return ONLY a JSON array of suggestion objects. Be concise and actionable.
If no issues are found, return an empty array."#.to_string();

    let user_prompt = format!(
        "Conversation transcript:\n---\n{}\n---\n\nCustomer's last message: {}\nFeedback from {}: {}",
        transcript,
        last_msg,
        author_type,
        if comment.is_empty() { "thumbs down (no comment)" } else { comment }
    );

    let api_key = std::env::var("CLAUDE_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        tracing::warn!("CLAUDE_API_KEY not set; skipping suggestion generation");
        return Ok(Vec::new());
    }

    let request = MessageRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 2048,
        system: Some(system_prompt),
        messages: vec![
            WireMessage {
                role: "user",
                content: user_prompt,
            },
        ],
        temperature: Some(0.3),
        expect_json: true,
    };

    match claude_complete(&state.http_client, &api_key, &request).await {
        Ok(reply) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(&reply.text) {
                match parsed {
                    Value::Array(arr) => Ok(arr),
                    _ => Ok(vec![parsed]),
                }
            } else {
                Ok(Vec::new())
            }
        }
        Err(e) => {
            tracing::warn!("Claude unavailable for suggestion generation: {}", e);
            Ok(Vec::new())
        }
    }
}

async fn store_suggestion(
    state: &SuggestionsState,
    suggestion: &Value,
    source_feedback_id: &str,
    source_conversation_id: &str,
    source_message_body: &Option<String>,
) -> Result<Suggestion, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let suggestion_type = suggestion.get("type")
        .and_then(Value::as_str)
        .unwrap_or("knowledge_gap")
        .to_string();

    let title = suggestion.get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if title.is_empty() {
        return Err("Suggestion has no title".to_string());
    }

    let description = suggestion.get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let suggested_content = suggestion.get("suggested_content")
        .and_then(|v| {
            let s = v.to_string();
            if s == "null" || s.is_empty() { None } else { Some(s) }
        });

    let confidence = suggestion.get("confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.5);

    let category = suggestion.get("category")
        .and_then(Value::as_str)
        .map(String::from);

    let duplicate: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM suggestions WHERE title = $1 AND status = 'suggested'",
    )
    .bind(&title)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    if duplicate > 0 {
        return Err(format!("Duplicate suggestion: {}", title));
    }

    let content_json = suggested_content.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("");

    sqlx::query(
        "INSERT INTO suggestions (id, suggestion_type, title, description, suggested_content,
         source_feedback_id, source_conversation_id, source_message_body, confidence, category,
         status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'suggested', $11, $12)",
    )
    .bind(&id)
    .bind(&suggestion_type)
    .bind(&title)
    .bind(&description)
    .bind(content_json)
    .bind(source_feedback_id)
    .bind(source_conversation_id)
    .bind(source_message_body.as_deref())
    .bind(confidence)
    .bind(category.as_deref())
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to store suggestion: {}", e))?;

    Ok(Suggestion {
        id,
        suggestion_type,
        title,
        description,
        suggested_content,
        source_feedback_id: Some(source_feedback_id.to_string()),
        source_conversation_id: Some(source_conversation_id.to_string()),
        source_message_body: source_message_body.clone(),
        confidence,
        category,
        status: "suggested".to_string(),
        approved_by: None,
        approved_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

// ---------------------------------------------------------------------------
// Suggestion management
// ---------------------------------------------------------------------------

pub async fn list_suggestions(
    state: &SuggestionsState,
    status: Option<&str>,
    suggestion_type: Option<&str>,
    limit: i64,
) -> Result<Vec<Suggestion>, String> {
    let mut query = String::from("SELECT id, suggestion_type, title, description, suggested_content, source_feedback_id, source_conversation_id, source_message_body, confidence, category, status, approved_by, approved_at, created_at, updated_at FROM suggestions WHERE 1=1");
    let mut idx = 1;

    if status.is_some() {
        query.push_str(&format!(" AND status = ${}", idx));
        idx += 1;
    }
    if suggestion_type.is_some() {
        query.push_str(&format!(" AND suggestion_type = ${}", idx));
        idx += 1;
    }
    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", idx));

    let mut q = sqlx::query(&query);
    if let Some(s) = status {
        q = q.bind(s);
    }
    if let Some(t) = suggestion_type {
        q = q.bind(t);
    }
    q = q.bind(limit);

    let rows = q
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("Failed to list suggestions: {}", e))?;

    let mut suggestions = Vec::new();
    for row in rows {
        let suggestion = Suggestion {
            id: row.try_get("id").unwrap_or_default(),
            suggestion_type: row.try_get("suggestion_type").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            description: row.try_get("description").unwrap_or_default(),
            suggested_content: row.try_get("suggested_content").ok(),
            source_feedback_id: row.try_get("source_feedback_id").ok(),
            source_conversation_id: row.try_get("source_conversation_id").ok(),
            source_message_body: row.try_get("source_message_body").ok(),
            confidence: row.try_get("confidence").unwrap_or(0.0),
            category: row.try_get("category").ok(),
            status: row.try_get("status").unwrap_or_default(),
            approved_by: row.try_get("approved_by").ok(),
            approved_at: row.try_get("approved_at").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        };
        suggestions.push(suggestion);
    }

    Ok(suggestions)
}

pub async fn get_suggestion_summary(state: &SuggestionsState) -> Result<SuggestionSummary, String> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suggestions")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suggestions WHERE status = 'suggested'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let approved: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suggestions WHERE status = 'approved'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let dismissed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suggestions WHERE status = 'dismissed'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let type_rows = sqlx::query(
        "SELECT suggestion_type, COUNT(*) as count FROM suggestions GROUP BY suggestion_type",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let by_type: Vec<SuggestionTypeCount> = type_rows.iter().map(|row| {
        SuggestionTypeCount {
            suggestion_type: row.try_get("suggestion_type").unwrap_or_default(),
            count: row.try_get("count").unwrap_or(0),
        }
    }).collect();

    let cat_rows = sqlx::query(
        "SELECT category, COUNT(*) as count FROM suggestions WHERE category IS NOT NULL GROUP BY category",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let by_category: Vec<SuggestionCategoryCount> = cat_rows.iter().map(|row| {
        SuggestionCategoryCount {
            category: row.try_get("category").unwrap_or_default(),
            count: row.try_get("count").unwrap_or(0),
        }
    }).collect();

    let recent = list_suggestions(state, None, None, 10).await.unwrap_or_default();

    Ok(SuggestionSummary {
        total_suggestions: total,
        open_suggestions: open,
        approved_suggestions: approved,
        dismissed_suggestions: dismissed,
        by_type,
        by_category,
        recent_suggestions: recent,
    })
}

pub async fn approve_suggestion(
    state: &SuggestionsState,
    suggestion_id: &str,
    approved_by: &str,
) -> Result<Suggestion, String> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE suggestions SET status = 'approved', approved_by = $1, approved_at = $2, updated_at = $3 WHERE id = $4",
    )
    .bind(approved_by)
    .bind(&now)
    .bind(&now)
    .bind(suggestion_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to approve suggestion: {}", e))?;

    get_suggestion_by_id(state, suggestion_id).await
}

pub async fn dismiss_suggestion(
    state: &SuggestionsState,
    suggestion_id: &str,
) -> Result<Suggestion, String> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE suggestions SET status = 'dismissed', updated_at = $1 WHERE id = $2",
    )
    .bind(&now)
    .bind(suggestion_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to dismiss suggestion: {}", e))?;

    get_suggestion_by_id(state, suggestion_id).await
}

async fn get_suggestion_by_id(state: &SuggestionsState, id: &str) -> Result<Suggestion, String> {
    let row = sqlx::query(
        "SELECT id, suggestion_type, title, description, suggested_content, source_feedback_id, source_conversation_id, source_message_body, confidence, category, status, approved_by, approved_at, created_at, updated_at FROM suggestions WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Suggestion not found: {}", e))?;

    Ok(Suggestion {
        id: row.try_get("id").unwrap_or_default(),
        suggestion_type: row.try_get("suggestion_type").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        suggested_content: row.try_get("suggested_content").ok(),
        source_feedback_id: row.try_get("source_feedback_id").ok(),
        source_conversation_id: row.try_get("source_conversation_id").ok(),
        source_message_body: row.try_get("source_message_body").ok(),
        confidence: row.try_get("confidence").unwrap_or(0.0),
        category: row.try_get("category").ok(),
        status: row.try_get("status").unwrap_or_default(),
        approved_by: row.try_get("approved_by").ok(),
        approved_at: row.try_get("approved_at").ok(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// Batch analysis
// ---------------------------------------------------------------------------

pub async fn batch_analyze_negative_feedback(
    state: &SuggestionsState,
    hours_back: i64,
) -> Result<BatchAnalysisResult, String> {
    let feedback_ids: Vec<String> = sqlx::query_scalar(
        "SELECT mf.id FROM message_feedback mf
         WHERE mf.rating = -1
         AND mf.created_at >= datetime('now', $1 || ' hours')
         AND NOT EXISTS (
             SELECT 1 FROM suggestions s WHERE s.source_feedback_id = mf.id
         )
         LIMIT 50",
    )
    .bind(&format!("{}-{}", hours_back, ""))
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch feedback: {}", e))?;

    let mut total_suggestions = 0;
    let mut processed = 0;
    let mut errors = Vec::new();

    for fid in &feedback_ids {
        match analyze_negative_feedback(state, fid).await {
            Ok(suggestions) => {
                total_suggestions += suggestions.len();
                processed += 1;
            }
            Err(e) => {
                errors.push(format!("Feedback {}: {}", fid, e));
            }
        }
    }

    Ok(BatchAnalysisResult {
        feedback_analyzed: processed,
        suggestions_created: total_suggestions,
        errors: errors.len(),
        error_details: errors,
    })
}
