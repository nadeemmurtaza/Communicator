//! CX QA Scoring Service - Quality assurance scoring for conversations
//!
//! Scores conversations against the `qa_scoring_rules` table (dimension +
//! condition + score_impact) using real signals pulled from `messages`,
//! `conversations` and `drafts`, and persists each run to `qa_scores`.
//! `get_cx_summary` and `batch_score_conversations` aggregate over that
//! real data instead of returning placeholders.

use crate::db::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CXSummary {
    pub avg_score: f64,
    pub total_conversations: i64,
    pub scored_conversations: i64,
    pub trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAScore {
    pub conversation_id: String,
    pub score: f64,
    pub feedback: String,
    pub rules_triggered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchScoringResult {
    pub total: i64,
    pub scored: i64,
    pub failed: i64,
    pub avg_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAScoringRule {
    pub id: String,
    pub name: String,
    pub is_enabled: bool,
    pub score_impact: f64,
}

struct ScoringRule {
    name: String,
    description: String,
    dimension: String,
    condition: Value,
    score_impact: f64,
}

const GREETING_WORDS: &[&str] = &[
    "hi ", "hi,", "hello", "hey", "dear ", "good morning", "good afternoon",
    "good evening", "welcome", "thanks for reaching", "thank you for reaching",
];
const CLOSING_WORDS: &[&str] = &[
    "anything else", "have a great day", "have a nice day", "take care",
    "best regards", "thank you for your patience", "thanks for your patience",
    "let us know if", "is there anything else",
];
const FRUSTRATION_WORDS: &[&str] = &[
    "frustrated", "angry", "annoyed", "terrible", "worst", "unacceptable",
    "disappointed", "ridiculous", "furious", "fed up",
];
const EMPATHY_WORDS: &[&str] = &[
    "sorry", "apolog", "understand", "i hear you", "appreciate your patience",
    "i know how", "that must be",
];
const POLICY_RED_FLAGS: &[&str] = &[
    "100% guarantee", "guaranteed refund", "i promise", "no questions asked refund",
    "free forever", "unlimited refund",
];

async fn fetch_enabled_rules(state: &AppState) -> Result<Vec<ScoringRule>, String> {
    let rows = sqlx::query(
        "SELECT name, description, dimension, condition, score_impact \
         FROM qa_scoring_rules WHERE is_enabled = true",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let condition_text: String = r.try_get("condition").unwrap_or_else(|_| "{}".to_string());
            let condition: Value = serde_json::from_str(&condition_text).unwrap_or(Value::Null);
            ScoringRule {
                name: r.try_get("name").unwrap_or_default(),
                description: r.try_get("description").unwrap_or_default(),
                dimension: r.try_get("dimension").unwrap_or_default(),
                condition,
                score_impact: r.try_get::<f32, _>("score_impact").unwrap_or(0.0) as f64,
            }
        })
        .collect())
}

pub async fn get_cx_summary(state: &AppState) -> Result<CXSummary, String> {
    let avg_score: Option<f64> = sqlx::query_scalar("SELECT AVG(overall_score) FROM qa_scores")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let total_conversations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let scored_conversations: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT conversation_id) FROM qa_scores")
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    let recent_avg: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(overall_score) FROM qa_scores WHERE created_at >= NOW() - INTERVAL '7 days'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let prior_avg: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(overall_score) FROM qa_scores \
         WHERE created_at >= NOW() - INTERVAL '14 days' AND created_at < NOW() - INTERVAL '7 days'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let trend = match (recent_avg, prior_avg) {
        (Some(r), Some(p)) if r - p > 2.0 => "improving",
        (Some(r), Some(p)) if r - p < -2.0 => "declining",
        _ => "stable",
    }
    .to_string();

    Ok(CXSummary {
        avg_score: avg_score.unwrap_or(0.0) as f64,
        total_conversations,
        scored_conversations,
        trend,
    })
}

pub async fn score_conversation(state: &AppState, conversation_id: &str) -> Result<QAScore, String> {
    let conv_row = sqlx::query("SELECT state, resolution_type FROM conversations WHERE id = $1")
        .bind(conversation_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("No messages found: conversation '{}' does not exist", conversation_id))?;

    let conv_state: String = conv_row.try_get("state").unwrap_or_default();
    let resolution_type: Option<String> = conv_row.try_get("resolution_type").ok();

    let msg_rows = sqlx::query(
        "SELECT direction, body, sent_at FROM messages \
         WHERE conversation_id = $1 AND COALESCE(is_private, 0) = 0 \
         ORDER BY sent_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if msg_rows.is_empty() {
        return Err(format!("No messages found for conversation '{}'", conversation_id));
    }

    let messages: Vec<(String, String, String)> = msg_rows
        .iter()
        .map(|r| {
            (
                r.try_get::<String, _>("direction").unwrap_or_default(),
                r.try_get::<String, _>("body").unwrap_or_default(),
                r.try_get::<String, _>("sent_at").unwrap_or_default(),
            )
        })
        .collect();

    let outbound_bodies: Vec<&str> = messages
        .iter()
        .filter(|(dir, _, _)| dir == "outbound")
        .map(|(_, body, _)| body.as_str())
        .collect();

    let has_greeting = outbound_bodies
        .first()
        .map(|b| {
            let lb = b.to_lowercase();
            GREETING_WORDS.iter().any(|w| lb.contains(w))
        })
        .unwrap_or(false);

    let has_closing = outbound_bodies
        .last()
        .map(|b| {
            let lb = b.to_lowercase();
            CLOSING_WORDS.iter().any(|w| lb.contains(w))
        })
        .unwrap_or(false);

    let policy_violation_found = outbound_bodies.iter().any(|b| {
        let lb = b.to_lowercase();
        POLICY_RED_FLAGS.iter().any(|w| lb.contains(w))
    });

    let mut frustration_without_empathy = false;
    let mut response_times_secs: Vec<f64> = Vec::new();
    for i in 0..messages.len() {
        let (dir, body, sent_at) = &messages[i];
        if dir != "inbound" {
            continue;
        }
        if let Some((_, reply_body, reply_sent_at)) =
            messages[i + 1..].iter().find(|(d, _, _)| d == "outbound")
        {
            // Response time between the inbound message and the next agent reply.
            if let (Ok(t1), Ok(t2)) = (
                chrono::DateTime::parse_from_rfc3339(sent_at),
                chrono::DateTime::parse_from_rfc3339(reply_sent_at),
            ) {
                let diff = (t2 - t1).num_seconds() as f64;
                if diff >= 0.0 {
                    response_times_secs.push(diff);
                }
            }

            // Empathy check: customer frustration followed by a reply with no empathetic language.
            let lb_in = body.to_lowercase();
            if FRUSTRATION_WORDS.iter().any(|w| lb_in.contains(w)) {
                let lb_out = reply_body.to_lowercase();
                if !EMPATHY_WORDS.iter().any(|w| lb_out.contains(w)) {
                    frustration_without_empathy = true;
                }
            }
        }
    }
    let avg_response_seconds = if response_times_secs.is_empty() {
        None
    } else {
        Some(response_times_secs.iter().sum::<f64>() / response_times_secs.len() as f64)
    };

    let draft_confidence: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(confidence) FROM drafts WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let is_resolved = conv_state == "resolved" || conv_state == "closed";

    let rules = fetch_enabled_rules(state).await?;

    let mut accuracy = 100.0f64;
    let mut empathy = 100.0f64;
    let mut resolution = 100.0f64;
    let mut compliance = 100.0f64;
    let mut timeliness = 100.0f64;
    let mut issues: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();
    let mut rules_triggered: Vec<String> = Vec::new();

    for rule in &rules {
        let check = rule.condition.get("check").and_then(|v| v.as_str()).unwrap_or("");
        let required = rule.condition.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
        let threshold = rule.condition.get("threshold").and_then(|v| v.as_f64());

        let triggered = match check {
            "resolution_type" => required && resolution_type.is_none() && !is_resolved,
            "response_seconds" => {
                let limit = threshold.unwrap_or(300.0);
                avg_response_seconds.map(|s| s > limit).unwrap_or(false)
            }
            "has_greeting" => required && !has_greeting,
            "has_closing" => required && !has_closing,
            "sentiment_mismatch" => frustration_without_empathy,
            "policy_violation" => policy_violation_found,
            "grounding_score" => {
                let limit = threshold.unwrap_or(0.7);
                draft_confidence.map(|c| c < limit).unwrap_or(false)
            }
            _ => false,
        };

        if triggered {
            match rule.dimension.as_str() {
                "accuracy" => accuracy += rule.score_impact,
                "empathy" => empathy += rule.score_impact,
                "resolution" => resolution += rule.score_impact,
                "compliance" => compliance += rule.score_impact,
                "timeliness" => timeliness += rule.score_impact,
                _ => {}
            }
            issues.push(format!("{}: {}", rule.name, rule.description));
            recommendations.push(rule.description.clone());
            rules_triggered.push(rule.name.clone());
        }
    }

    accuracy = accuracy.clamp(0.0, 100.0);
    empathy = empathy.clamp(0.0, 100.0);
    resolution = resolution.clamp(0.0, 100.0);
    compliance = compliance.clamp(0.0, 100.0);
    timeliness = timeliness.clamp(0.0, 100.0);

    let overall = (accuracy + empathy + resolution + compliance + timeliness) / 5.0;

    let feedback = if issues.is_empty() {
        "No issues found; conversation passed all active QA checks.".to_string()
    } else {
        issues.join("; ")
    };

    let qa_id = format!("qa-{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO qa_scores \
         (id, conversation_id, overall_score, accuracy_score, empathy_score, resolution_score, \
          compliance_score, timeliness_score, evaluation_summary, issues, recommendations, \
          is_auto_scored, scored_by, scored_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, 'auto-qa-engine', $12)",
    )
    .bind(&qa_id)
    .bind(conversation_id)
    .bind(overall as f32)
    .bind(accuracy as f32)
    .bind(empathy as f32)
    .bind(resolution as f32)
    .bind(compliance as f32)
    .bind(timeliness as f32)
    .bind(&feedback)
    .bind(serde_json::to_string(&issues).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&recommendations).unwrap_or_else(|_| "[]".to_string()))
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(QAScore {
        conversation_id: conversation_id.to_string(),
        score: overall,
        feedback,
        rules_triggered,
    })
}

/// Scores every unresolved conversation that has at least one message and
/// hasn't been auto-scored in the last 24 hours.
pub async fn batch_score_conversations(state: &AppState) -> Result<BatchScoringResult, String> {
    let candidates: Vec<String> = sqlx::query_scalar(
        "SELECT c.id FROM conversations c \
         WHERE c.state NOT IN ('resolved', 'closed') \
           AND EXISTS (SELECT 1 FROM messages m WHERE m.conversation_id = c.id) \
           AND NOT EXISTS ( \
               SELECT 1 FROM qa_scores q \
               WHERE q.conversation_id = c.id AND q.created_at > NOW() - INTERVAL '24 hours' \
           ) \
         ORDER BY c.updated_at DESC LIMIT 200",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let mut scored = 0i64;
    let mut failed = 0i64;
    let mut score_sum = 0.0f64;

    for conversation_id in &candidates {
        match score_conversation(state, conversation_id).await {
            Ok(qa) => {
                scored += 1;
                score_sum += qa.score;
            }
            Err(_) => failed += 1,
        }
    }

    let avg_score = if scored > 0 { score_sum / scored as f64 } else { 0.0 };

    Ok(BatchScoringResult {
        total: candidates.len() as i64,
        scored,
        failed,
        avg_score,
    })
}

pub async fn list_qa_rules(state: &AppState) -> Result<Vec<QAScoringRule>, String> {
    let rows = sqlx::query(
        "SELECT id, name, is_enabled, score_impact FROM qa_scoring_rules ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| QAScoringRule {
            id: r.try_get("id").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            is_enabled: r.try_get("is_enabled").unwrap_or(false),
            score_impact: r.try_get::<f32, _>("score_impact").unwrap_or(0.0) as f64,
        })
        .collect())
}

pub async fn update_qa_rule(
    state: &AppState,
    rule_id: &str,
    is_enabled: Option<bool>,
    score_impact: Option<f64>,
) -> Result<QAScoringRule, String> {
    let existing = sqlx::query(
        "SELECT name, is_enabled, score_impact FROM qa_scoring_rules WHERE id = $1",
    )
    .bind(rule_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("QA scoring rule '{}' not found", rule_id))?;

    let name: String = existing.try_get("name").unwrap_or_default();
    let current_enabled: bool = existing.try_get("is_enabled").unwrap_or(true);
    let current_impact: f64 = existing.try_get::<f32, _>("score_impact").unwrap_or(0.0) as f64;

    let new_enabled = is_enabled.unwrap_or(current_enabled);
    let new_impact = score_impact.unwrap_or(current_impact);

    sqlx::query(
        "UPDATE qa_scoring_rules SET is_enabled = $1, score_impact = $2, updated_at = NOW() WHERE id = $3",
    )
    .bind(new_enabled)
    .bind(new_impact as f32)
    .bind(rule_id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(QAScoringRule {
        id: rule_id.to_string(),
        name,
        is_enabled: new_enabled,
        score_impact: new_impact,
    })
}
