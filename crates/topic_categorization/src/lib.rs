//! Topic Categorization — Phase 11 of the self-improvement flywheel.
//!
//! Every conversation is automatically categorized into topics and subtopics
//! using Claude. This enables trend analysis, content gap identification,
//! agent coaching, and routing optimization.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row, FromRow};
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct TopicCategorizationState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
    pub claude_api_key: String,
}

impl TopicCategorizationState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            claude_api_key: std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTopic {
    pub id: String,
    pub conversation_id: String,
    pub topic: String,
    pub subtopic: Option<String>,
    pub confidence: f64,
    pub keywords: Vec<String>,
    pub is_overridden: bool,
    pub overridden_by: Option<String>,
    pub categorized_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTrend {
    pub id: String,
    pub topic: String,
    pub subtopic: Option<String>,
    pub window_start: String,
    pub window_end: String,
    pub conversation_count: i64,
    pub avg_confidence: f64,
    pub avg_resolution_rate: Option<f64>,
    pub avg_response_time_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub total_conversations: i64,
    pub categorized_count: i64,
    pub uncategorized_count: i64,
    pub topic_counts: Vec<TopicCount>,
    pub top_keywords: Vec<KeywordCount>,
    pub trending_topics: Vec<TrendingTopic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCount {
    pub topic: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordCount {
    pub keyword: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTopic {
    pub topic: String,
    pub current_count: i64,
    pub previous_count: i64,
    pub change_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCategorizationResult {
    pub total: usize,
    pub categorized: usize,
    pub failed: usize,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

struct AiCategorizationResult {
    topic: String,
    subtopic: Option<String>,
    confidence: f64,
    keywords: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConversationMessage {
    role: String,
    body: String,
}

// ---------------------------------------------------------------------------
// Core categorization
// ---------------------------------------------------------------------------

pub async fn categorize_conversation(
    state: &TopicCategorizationState,
    conversation_id: &str,
) -> Result<ConversationTopic, String> {
    let existing = fetch_existing_topic(state, conversation_id).await;

    if let Some(topic) = &existing {
        if !topic.is_overridden && is_recent(&topic.categorized_at) {
            return Ok(topic.clone());
        }
    }

    let messages = fetch_conversation_messages(state, conversation_id).await;
    if messages.is_empty() {
        return Err("No messages found in conversation".to_string());
    }

    let result = categorize_with_ai(state, &messages).await;

    let (topic, subtopic, confidence, keywords) = if let Ok(ai_result) = result {
        (ai_result.topic, ai_result.subtopic, ai_result.confidence, ai_result.keywords)
    } else {
        let fallback = categorize_by_keywords(&messages);
        (fallback.topic, fallback.subtopic, fallback.confidence, fallback.keywords)
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let conv_topic = ConversationTopic {
        id: id.clone(),
        conversation_id: conversation_id.to_string(),
        topic: topic.clone(),
        subtopic: subtopic.clone(),
        confidence,
        keywords: keywords.clone(),
        is_overridden: false,
        overridden_by: None,
        categorized_at: now.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    store_topic(state, &conv_topic).await?;

    Ok(conv_topic)
}

async fn fetch_conversation_messages(
    state: &TopicCategorizationState,
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

async fn categorize_with_ai(
    state: &TopicCategorizationState,
    messages: &[ConversationMessage],
) -> Result<AiCategorizationResult, String> {
    if state.claude_api_key.is_empty() {
        return Err("Claude API key not configured".to_string());
    }

    let mut transcript_lines = Vec::new();
    for msg in messages {
        let role_label = if msg.role == "customer" { "Customer" } else { "Agent" };
        transcript_lines.push(format!("{}: {}", role_label, msg.body));
    }
    let transcript = transcript_lines.join("\n");

    let system_prompt = r#"You are an AI assistant that categorizes customer service conversations into topics and subtopics.

Available top-level topics:
- "product_inquiry" — Questions about products, features, capabilities
- "pricing" — Questions about pricing, quotes, discounts, billing
- "support" — Technical support, troubleshooting, bugs
- "sales" — Buying signals, demos, trials, proposals
- "complaint" — Complaints, escalations, dissatisfaction
- "refund" — Refund requests, returns, chargebacks
- "scheduling" — Meeting requests, appointments, availability
- "account" — Account management, login, profile changes
- "feedback" — Customer feedback, suggestions, reviews
- "general" — Conversations that don't fit other categories

For each topic, choose the most specific subtopic that fits.
Extract 3-5 keywords from the conversation that led to your classification.

Return ONLY valid JSON with these fields:
- "topic": The top-level topic (must be one of the listed topics)
- "subtopic": The specific subtopic (string)
- "confidence": Confidence score 0.0-1.0
- "keywords": Array of 3-5 keyword strings

Return ONLY valid JSON. No markdown, no explanation."#.to_string();

    let user_prompt = format!(
        "Categorize this conversation:\n---\n{}\n---",
        transcript.chars().take(8000).collect::<String>()
    );

    let body = json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 512,
        "system": system_prompt,
        "messages": [{
            "role": "user",
            "content": user_prompt
        }],
        "temperature": 0.1,
    });

    let resp = state.http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &state.claude_api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Claude request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Claude response read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Claude API error: {} - {}", status, text));
    }

    let parsed: Value = serde_json::from_str(&text)
        .map_err(|e| format!("Invalid JSON from Claude: {}", e))?;

    let content = parsed.get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    let parsed = serde_json::from_str::<Value>(content)
        .map_err(|e| format!("Invalid JSON in Claude content: {}", e))?;

    let topic = parsed.get("topic")
        .and_then(|v| v.as_str())
        .unwrap_or("general")
        .to_string();

    let subtopic = parsed.get("subtopic")
        .and_then(|v| v.as_str())
        .map(String::from);

    let confidence = parsed.get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7);

    let keywords: Vec<String> = parsed.get("keywords")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Ok(AiCategorizationResult {
        topic,
        subtopic,
        confidence,
        keywords,
    })
}

fn categorize_by_keywords(messages: &[ConversationMessage]) -> AiCategorizationResult {
    let combined: String = messages.iter().map(|m| m.body.to_lowercase()).collect();

    let mut keywords = Vec::new();
    let mut topic = "general".to_string();
    let mut subtopic = None;
    let mut confidence = 0.5;

    if combined.contains("refund") || combined.contains("return") || combined.contains("chargeback") {
        topic = "refund".to_string();
        subtopic = Some("refund_request".to_string());
        keywords.push("refund".to_string());
        confidence = 0.7;
    } else if combined.contains("complaint") || combined.contains("terrible") || combined.contains("awful")
        || combined.contains("disappointed") || combined.contains("unacceptable") {
        topic = "complaint".to_string();
        subtopic = Some("service_quality".to_string());
        keywords.push("complaint".to_string());
        confidence = 0.7;
    } else if combined.contains("price") || combined.contains("cost") || combined.contains("quote")
        || combined.contains("billing") || combined.contains("invoice") {
        topic = "pricing".to_string();
        subtopic = Some("quote_request".to_string());
        keywords.push("pricing".to_string());
        confidence = 0.7;
    } else if combined.contains("demo") || combined.contains("trial") || combined.contains("buy")
        || combined.contains("purchase") || combined.contains("order") {
        topic = "sales".to_string();
        subtopic = Some("demo_request".to_string());
        keywords.push("sales".to_string());
        confidence = 0.7;
    } else if combined.contains("bug") || combined.contains("error") || combined.contains("broken")
        || combined.contains("issue") || combined.contains("problem") {
        topic = "support".to_string();
        subtopic = Some("bug_report".to_string());
        keywords.push("support".to_string());
        confidence = 0.7;
    } else if combined.contains("meeting") || combined.contains("schedule") || combined.contains("appointment")
        || combined.contains("available") {
        topic = "scheduling".to_string();
        subtopic = Some("meeting_request".to_string());
        keywords.push("scheduling".to_string());
        confidence = 0.7;
    } else if combined.contains("feature") || combined.contains("capability") || combined.contains("how does")
        || combined.contains("what is") {
        topic = "product_inquiry".to_string();
        subtopic = Some("features".to_string());
        keywords.push("product".to_string());
        confidence = 0.6;
    }

    let common_words = ["help", "need", "want", "question", "info", "support", "service", "product"];
    for word in &common_words {
        if combined.contains(word) && keywords.len() < 5 {
            keywords.push(word.to_string());
        }
    }

    AiCategorizationResult {
        topic,
        subtopic,
        confidence,
        keywords,
    }
}

fn is_recent(timestamp: &str) -> bool {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        now.signed_duration_since(dt.to_utc()).num_minutes() < 60
    } else {
        false
    }
}

async fn fetch_existing_topic(state: &TopicCategorizationState, conversation_id: &str) -> Option<ConversationTopic> {
    let row = sqlx::query(
        "SELECT id, conversation_id, topic, subtopic, confidence, keywords,
         is_overridden, overridden_by, categorized_at, created_at, updated_at
         FROM conversation_topics WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .ok()?;

    let row = row?;

    let keywords: Vec<String> = row.try_get("keywords").ok()
        .and_then(|s: String| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Some(ConversationTopic {
        id: row.get("id"),
        conversation_id: row.get("conversation_id"),
        topic: row.get("topic"),
        subtopic: row.try_get("subtopic").ok(),
        confidence: row.get("confidence"),
        keywords,
        is_overridden: row.get("is_overridden"),
        overridden_by: row.try_get("overridden_by").ok(),
        categorized_at: row.get("categorized_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn store_topic(state: &TopicCategorizationState, topic: &ConversationTopic) -> Result<(), String> {
    let keywords_json = serde_json::to_string(&topic.keywords)
        .unwrap_or_else(|_| "[]".to_string());

    let _ = sqlx::query(
        "DELETE FROM conversation_topics WHERE conversation_id = $1 AND is_overridden = 0",
    )
    .bind(&topic.conversation_id)
    .execute(&state.db)
    .await;

    sqlx::query(
        "INSERT INTO conversation_topics (id, conversation_id, topic, subtopic, confidence,
         keywords, is_overridden, overridden_by, categorized_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(&topic.id)
    .bind(&topic.conversation_id)
    .bind(&topic.topic)
    .bind(topic.subtopic.as_deref())
    .bind(topic.confidence)
    .bind(&keywords_json)
    .bind(topic.is_overridden as i32)
    .bind(topic.overridden_by.as_deref())
    .bind(&topic.categorized_at)
    .bind(&topic.created_at)
    .bind(&topic.updated_at)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to store topic categorization: {}", e))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Batch categorization
// ---------------------------------------------------------------------------

pub async fn categorize_uncategorized(
    state: &TopicCategorizationState,
) -> Result<BatchCategorizationResult, String> {
    #[derive(FromRow)]
    struct ConvIdRow {
        id: String,
    }

    let conversations: Vec<ConvIdRow> = sqlx::query_as(
        "SELECT c.id FROM conversations c
         LEFT JOIN conversation_topics ct ON c.id = ct.conversation_id
         WHERE ct.id IS NULL
         AND c.state NOT IN ('opted_out', 'closed_won', 'closed_lost')
         ORDER BY c.updated_at DESC
         LIMIT 100",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch uncategorized conversations: {}", e))?;

    let total = conversations.len();
    let mut categorized = 0;
    let mut failed = 0;

    for conv in &conversations {
        match categorize_conversation(state, &conv.id).await {
            Ok(_) => categorized += 1,
            Err(_) => failed += 1,
        }
    }

    Ok(BatchCategorizationResult {
        total,
        categorized,
        failed,
    })
}

// ---------------------------------------------------------------------------
// Topic summary & analytics
// ---------------------------------------------------------------------------

pub async fn get_topic_summary(state: &TopicCategorizationState) -> Result<TopicSummary, String> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversations",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to count conversations: {}", e))?;

    let categorized: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT conversation_id) FROM conversation_topics",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to count categorized: {}", e))?;

    let uncategorized = total - categorized;

    #[derive(FromRow)]
    struct TopicCountRow {
        topic: String,
        count: i64,
    }

    let rows: Vec<TopicCountRow> = sqlx::query_as(
        "SELECT topic, COUNT(DISTINCT conversation_id) as count
         FROM conversation_topics
         GROUP BY topic
         ORDER BY count DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to query topic counts: {}", e))?;

    let topic_counts: Vec<TopicCount> = rows.into_iter().map(|row| {
        let percentage = if categorized > 0 {
            (row.count as f64 / categorized as f64 * 1000.0).round() / 10.0
        } else {
            0.0
        };
        TopicCount {
            topic: row.topic,
            count: row.count,
            percentage,
        }
    }).collect();

    let keywords_raw: Vec<String> = sqlx::query_scalar(
        "SELECT keywords FROM conversation_topics ORDER BY categorized_at DESC LIMIT 100",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut keyword_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for kw_json in keywords_raw {
        if let Ok(kws) = serde_json::from_str::<Vec<String>>(&kw_json) {
            for kw in kws {
                *keyword_map.entry(kw).or_insert(0) += 1;
            }
        }
    }

    let mut keyword_counts: Vec<KeywordCount> = keyword_map.into_iter()
        .map(|(keyword, count)| KeywordCount { keyword, count })
        .collect();
    keyword_counts.sort_by(|a, b| b.count.cmp(&a.count));
    keyword_counts.truncate(20);

    let seven_days_ago = (Utc::now() - chrono::Duration::days(7)).to_rfc3339();
    let fourteen_days_ago = (Utc::now() - chrono::Duration::days(14)).to_rfc3339();

    let current_rows: Vec<TopicCountRow> = sqlx::query_as(
        "SELECT topic, COUNT(DISTINCT conversation_id) as count
         FROM conversation_topics WHERE categorized_at >= $1
         GROUP BY topic",
    )
    .bind(&seven_days_ago)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let previous_rows: Vec<TopicCountRow> = sqlx::query_as(
        "SELECT topic, COUNT(DISTINCT conversation_id) as count
         FROM conversation_topics WHERE categorized_at >= $1 AND categorized_at < $2
         GROUP BY topic",
    )
    .bind(&fourteen_days_ago)
    .bind(&seven_days_ago)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let previous_map: std::collections::HashMap<String, i64> = previous_rows.into_iter()
        .map(|r| (r.topic, r.count))
        .collect();

    let trending_topics: Vec<TrendingTopic> = current_rows.into_iter().map(|row| {
        let prev_count = *previous_map.get(&row.topic).unwrap_or(&0);
        let change = if prev_count > 0 {
            ((row.count as f64 - prev_count as f64) / prev_count as f64 * 1000.0).round() / 10.0
        } else if row.count > 0 {
            100.0
        } else {
            0.0
        };
        TrendingTopic {
            topic: row.topic,
            current_count: row.count,
            previous_count: prev_count,
            change_percentage: change,
        }
    }).collect();

    Ok(TopicSummary {
        total_conversations: total,
        categorized_count: categorized,
        uncategorized_count: uncategorized,
        topic_counts,
        top_keywords: keyword_counts,
        trending_topics,
    })
}

// ---------------------------------------------------------------------------
// Override topic
// ---------------------------------------------------------------------------

pub async fn override_topic(
    state: &TopicCategorizationState,
    conversation_id: &str,
    new_topic: &str,
    new_subtopic: Option<&str>,
    overridden_by: &str,
) -> Result<ConversationTopic, String> {
    let existing = fetch_existing_topic(state, conversation_id).await;

    let id = existing.as_ref().map(|t| t.id.clone()).unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now().to_rfc3339();

    let keywords = existing.as_ref().map(|t| t.keywords.clone()).unwrap_or_default();
    let confidence = existing.as_ref().map(|t| t.confidence).unwrap_or(1.0);

    let conv_topic = ConversationTopic {
        id: id.clone(),
        conversation_id: conversation_id.to_string(),
        topic: new_topic.to_string(),
        subtopic: new_subtopic.map(String::from),
        confidence,
        keywords: keywords.clone(),
        is_overridden: true,
        overridden_by: Some(overridden_by.to_string()),
        categorized_at: now.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let _ = sqlx::query(
        "DELETE FROM conversation_topics WHERE conversation_id = $1 AND is_overridden = 0",
    )
    .bind(conversation_id)
    .execute(&state.db)
    .await;

    store_topic(state, &conv_topic).await?;

    Ok(conv_topic)
}

// ---------------------------------------------------------------------------
// Get conversation topic (exposed for gRPC)
// ---------------------------------------------------------------------------

pub async fn get_conversation_topic(
    state: &TopicCategorizationState,
    conversation_id: &str,
) -> Result<ConversationTopic, String> {
    match fetch_existing_topic(state, conversation_id).await {
        Some(topic) => Ok(topic),
        None => Err("No topic categorization found for this conversation".to_string()),
    }
}
