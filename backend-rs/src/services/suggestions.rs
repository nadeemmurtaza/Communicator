#![allow(dead_code)]

//! AI-Powered Suggestions — Phase 9 of the self-improvement flywheel.
//!
//! Connects the feedback loop (Phase 6) to knowledge gap detection and
//! agent insight generation. When a customer gives negative feedback
//! (thumbs down), the system automatically analyzes the conversation
//! and suggests:
//! 1. New knowledge base entries for uncovered questions
//! 2. FAQ additions for recurring topics
//! 3. Agent insights for pattern-based improvements
//!
//! Design decisions:
//! - Suggestions are triggered automatically when negative feedback is
//!   submitted, not just on manual scan.
//! - Claude is used to analyze the conversation context and generate
//!   structured suggestions with grounded content.
//! - Each suggestion has a status lifecycle: suggested → approved/dismissed.
//! - Duplicate detection prevents the same suggestion from being created
//!   multiple times.

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

/// A suggestion generated from negative feedback or gap analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub suggestion_type: String,  // 'knowledge_gap', 'faq_addition', 'agent_insight'
    pub title: String,
    pub description: String,
    pub suggested_content: Option<String>,  // JSON: draft KB entry, FAQ Q/A, or insight payload
    pub source_feedback_id: Option<String>,  // message_feedback.id that triggered this
    pub source_conversation_id: Option<String>,
    pub source_message_body: Option<String>,  // the customer message that triggered it
    pub confidence: f64,  // 0.0-1.0, how confident the AI is in this suggestion
    pub category: Option<String>,  // e.g., 'pricing', 'technical', 'policy'
    pub status: String,  // 'suggested', 'approved', 'dismissed', 'applied'
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Summary of suggestion analytics.
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

// ---------------------------------------------------------------------------
// Core suggestion generation
// ---------------------------------------------------------------------------

/// Analyze negative feedback and generate suggestions automatically.
/// Returns the number of suggestions created.
pub async fn analyze_negative_feedback(
    state: &AppState,
    feedback_id: &str,
) -> Result<Vec<Suggestion>, String> {
    // Fetch the feedback record
    #[derive(FromRow)]
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

    // Only process negative feedback
    if feedback.rating != -1 {
        return Ok(Vec::new());
    }

    // Fetch the conversation context
    let conversation_context = get_conversation_context(state, &feedback.conversation_id).await?;

    // Check for existing suggestions from this feedback
    let existing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM suggestions WHERE source_feedback_id = $1",
    )
    .bind(feedback_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    if existing > 0 {
        return Ok(Vec::new());  // Already processed
    }

    // Use Claude to analyze and generate suggestions
    let suggestions = generate_suggestions_with_ai(
        state,
        &conversation_context,
        &feedback.comment,
        &feedback.author_type,
    )
    .await?;

    // Store suggestions in the database
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

/// Conversation context for suggestion generation.
#[derive(Debug, Clone)]
struct ConversationContext {
    pub messages: Vec<ConversationMessage>,
    pub last_customer_message: Option<String>,
    pub contact_name: Option<String>,
    pub contact_channel: Option<String>,
}

#[derive(Debug, Clone)]
struct ConversationMessage {
    pub role: String,  // 'customer' or 'agent'
    pub body: String,
    pub created_at: String,
}

/// Fetch the conversation context for suggestion analysis.
async fn get_conversation_context(
    state: &AppState,
    conversation_id: &str,
) -> Result<ConversationContext, String> {
    #[derive(FromRow)]
    struct MessageRow {
        id: String,
        body: String,
        sender: Option<String>,
        direction: Option<String>,
        created_at: String,
    }

    let messages: Vec<MessageRow> = sqlx::query_as(
        "SELECT id, body, sender, direction, created_at
         FROM messages WHERE conversation_id = ?
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

    // Fetch contact info
    let contact_name: Option<String> = sqlx::query_scalar(
        "SELECT c.person_name FROM contacts c
         JOIN conversations cv ON cv.contact_id = c.id
         WHERE cv.id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let contact_channel: Option<String> = sqlx::query_scalar(
        "SELECT c.channel FROM contacts c
         JOIN conversations cv ON cv.contact_id = c.id
         WHERE cv.id = ?",
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

/// Use Claude to analyze the conversation and generate structured suggestions.
async fn generate_suggestions_with_ai(
    state: &AppState,
    context: &ConversationContext,
    feedback_comment: &Option<String>,
    author_type: &str,
) -> Result<Vec<Value>, String> {
    // Build the conversation transcript
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
            // Parse the JSON array from the response
            if let Ok(parsed) = serde_json::from_str::<Value>(&reply.text) {
                match parsed {
                    Value::Array(arr) => Ok(arr),
                    _ => Ok(vec![parsed]),
                }
            } else {
                // If parsing fails, return empty suggestions
                Ok(Vec::new())
            }
        }
        Err(e) => {
            // If Claude is unavailable, don't block the feedback submission
            tracing::warn!("Claude unavailable for suggestion generation: {}", e);
            Ok(Vec::new())
        }
    }
}

/// Store a suggestion in the database.
async fn store_suggestion(
    state: &AppState,
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

    // Check for duplicates
    let duplicate: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM suggestions WHERE title = ? AND status = 'suggested'",
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
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'suggested', ?, ?)",
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

/// Get all suggestions with optional filtering.
pub async fn list_suggestions(
    state: &AppState,
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

/// Get suggestion summary analytics.
pub async fn get_suggestion_summary(state: &AppState) -> Result<SuggestionSummary, String> {
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

    // Count by type
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

    // Count by category
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

    // Recent suggestions
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

/// Approve a suggestion.
pub async fn approve_suggestion(
    state: &AppState,
    suggestion_id: &str,
    approved_by: &str,
) -> Result<Suggestion, String> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE suggestions SET status = 'approved', approved_by = ?, approved_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(approved_by)
    .bind(&now)
    .bind(&now)
    .bind(suggestion_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to approve suggestion: {}", e))?;

    // Fetch the updated suggestion
    get_suggestion_by_id(state, suggestion_id).await
}

/// Dismiss a suggestion.
pub async fn dismiss_suggestion(
    state: &AppState,
    suggestion_id: &str,
) -> Result<Suggestion, String> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE suggestions SET status = 'dismissed', updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(suggestion_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to dismiss suggestion: {}", e))?;

    get_suggestion_by_id(state, suggestion_id).await
}

/// Get a suggestion by ID.
async fn get_suggestion_by_id(state: &AppState, id: &str) -> Result<Suggestion, String> {
    let row = sqlx::query(
        "SELECT id, suggestion_type, title, description, suggested_content, source_feedback_id, source_conversation_id, source_message_body, confidence, category, status, approved_by, approved_at, created_at, updated_at FROM suggestions WHERE id = ?",
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
// Batch analysis for auto-learning scan
// ---------------------------------------------------------------------------

/// Analyze all recent negative feedback and generate suggestions in batch.
/// This is the real implementation of the auto-learning scan.
pub async fn batch_analyze_negative_feedback(
    state: &AppState,
    hours_back: i64,
) -> Result<BatchAnalysisResult, String> {
    // Get unprocessed negative feedback
    let feedback_ids: Vec<String> = sqlx::query_scalar(
        "SELECT mf.id FROM message_feedback mf
         WHERE mf.rating = -1
         AND mf.created_at >= datetime('now', ? || ' hours')
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisResult {
    pub feedback_analyzed: usize,
    pub suggestions_created: usize,
    pub errors: usize,
    pub error_details: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_serialization() {
        let suggestion = Suggestion {
            id: "test-123".to_string(),
            suggestion_type: "knowledge_gap".to_string(),
            title: "Test suggestion".to_string(),
            description: "Test description".to_string(),
            suggested_content: None,
            source_feedback_id: None,
            source_conversation_id: None,
            source_message_body: None,
            confidence: 0.8,
            category: Some("pricing".to_string()),
            status: "suggested".to_string(),
            approved_by: None,
            approved_at: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&suggestion).unwrap();
        assert!(json.contains("knowledge_gap"));
        assert!(json.contains("Test suggestion"));
    }

    #[test]
    fn test_batch_result_serialization() {
        let result = BatchAnalysisResult {
            feedback_analyzed: 10,
            suggestions_created: 5,
            errors: 0,
            error_details: Vec::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("10"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_summary_serialization() {
        let summary = SuggestionSummary {
            total_suggestions: 100,
            open_suggestions: 10,
            approved_suggestions: 80,
            dismissed_suggestions: 10,
            by_type: vec![],
            by_category: vec![],
            recent_suggestions: vec![],
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("100"));
    }
}
