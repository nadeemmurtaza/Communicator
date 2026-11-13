#![allow(dead_code)]

//! Validation Layer — Phase 7 of the self-improvement flywheel.
//!
//! Before an AI-generated draft reply is stored or sent, this module runs a
//! second, lightweight Claude call that acts as an independent reviewer.  The
//! validator checks two things:
//!
//! 1. **Relevance** — Does the reply actually answer the customer's question?
//! 2. **Grounding** — Is every factual claim in the reply supported by the
//!    knowledge base / service catalog provided as context?
//!
//! The validator returns a structured JSON response with scores and reasons.
//! If either check fails below the configured threshold, the draft is flagged
//! for human review regardless of the auto-send confidence threshold.
//!
//! Design decisions:
//! - The validator is a *separate* call, not a self-referential prompt.  A
//!   single model call that both generates and validates tends to validate its
//!   own output blindly.  Two independent calls reduce this bias.
//! - The validator uses a lower max_tokens budget (512) since it only needs to
//!   produce a short structured verdict, not a full reply.
//! - If the validator call fails (API error, timeout), the draft is not
//!   rejected — it falls through to the normal confidence-based review.  This
//!   avoids blocking all replies when the validator is temporarily unavailable.

use crate::db::AppState;
use crate::services::claude_client::ask_json;
use serde_json::{json, Value};
use std::env;

/// Max tokens for the validation call — enough for a short structured verdict.
const VALIDATION_MAX_TOKENS: u32 = 512;

/// Default minimum score for relevance check (0.0–1.0).
const DEFAULT_RELEVANCE_THRESHOLD: f64 = 0.7;

/// Default minimum score for grounding check (0.0–1.0).
const DEFAULT_GROUNDING_THRESHOLD: f64 = 0.7;

/// Result of the validation check on a draft reply.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall pass/fail decision.
    pub passed: bool,

    /// Relevance score: how well the reply answers the customer's question (0–1).
    pub relevance_score: f64,

    /// Grounding score: how well the reply is supported by knowledge sources (0–1).
    pub grounding_score: f64,

    /// Human-readable reason if the validation failed.
    pub reason: Option<String>,

    /// List of specific issues found (e.g., "claims pricing not in knowledge base").
    pub issues: Vec<String>,

    /// Whether the validation was actually run (false if skipped due to error).
    pub validated: bool,

    /// Latency of the validation call in milliseconds.
    pub latency_ms: i64,
}

impl ValidationResult {
    /// Returns a result indicating validation was skipped.
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            passed: true, // Skipped validation does not block the draft
            relevance_score: 0.0,
            grounding_score: 0.0,
            reason: Some(reason.into()),
            issues: Vec::new(),
            validated: false,
            latency_ms: 0,
        }
    }

    /// Returns a result indicating validation was not configured.
    pub fn not_configured() -> Self {
        Self::skipped("validation layer is not configured")
    }

    pub fn to_json(&self) -> Value {
        json!({
            "passed": self.passed,
            "relevanceScore": self.relevance_score,
            "groundingScore": self.grounding_score,
            "reason": self.reason,
            "issues": self.issues,
            "validated": self.validated,
            "latencyMs": self.latency_ms,
        })
    }
}

/// Configuration for the validation layer, read from environment variables.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub relevance_threshold: f64,
    pub grounding_threshold: f64,
    pub enabled: bool,
}

impl ValidationConfig {
    pub fn from_env() -> Self {
        let relevance_threshold = env::var("VALIDATION_RELEVANCE_THRESHOLD")
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|v| *v >= 0.0 && *v <= 1.0)
            .unwrap_or(DEFAULT_RELEVANCE_THRESHOLD);

        let grounding_threshold = env::var("VALIDATION_GROUNDING_THRESHOLD")
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|v| *v >= 0.0 && *v <= 1.0)
            .unwrap_or(DEFAULT_GROUNDING_THRESHOLD);

        // Enabled by default; set VALIDATION_ENABLED=false to disable.
        let enabled = !matches!(
            env::var("VALIDATION_ENABLED")
                .ok()
                .as_deref()
                .map(|v| v.trim().to_ascii_lowercase()),
            Some(v) if v == "false" || v == "0" || v == "no"
        );

        Self {
            relevance_threshold,
            grounding_threshold,
            enabled,
        }
    }
}

/// Build the system prompt for the validator.  The validator acts as an
/// independent reviewer, not the original author.
fn build_validator_system_prompt() -> String {
    String::from(
        "You are an independent quality reviewer for customer-facing messages. \
         Your job is to evaluate whether a draft reply is appropriate to send to a customer. \
         You are NOT the author of the reply — you are an unbiased reviewer.\n\
         Evaluate two dimensions:\n\
         1. RELEVANCE (0.0–1.0): Does the reply directly answer the customer's question or \
         address their concern? A reply that ignores the question scores 0.0. A reply that \
         fully and accurately answers scores 1.0.\n\
         2. GROUNDING (0.0–1.0): Is every factual claim in the reply supported by the provided \
         knowledge base and service catalog? If the reply makes claims about pricing, features, \
         or policies that are NOT in the knowledge base, lower the grounding score. A reply \
         that only uses information from the knowledge base scores 1.0. A reply that invents \
         facts scores 0.0.\n\
         Respond with strict JSON only, no markdown fences:\n\
         {\"relevance_score\": 0.0-1.0,\n\
         \"grounding_score\": 0.0-1.0,\n\
         \"reason\": \"one sentence explaining any failure, or null if both pass\",\n\
         \"issues\": [\"specific issue 1\", \"specific issue 2\"]}\n"
    )
}

/// Build the user prompt for the validator, containing the customer's question,
/// the draft reply, and the knowledge context used to generate it.
fn build_validator_user_prompt(
    customer_question: &str,
    draft_reply: &str,
    knowledge_context: &str,
) -> String {
    let kb_section = if knowledge_context.trim().is_empty() {
        "KNOWLEDGE BASE & SERVICES:\n(No knowledge base entries were provided to the generator.)\n\n"
            .to_string()
    } else {
        format!("KNOWLEDGE BASE & SERVICES:\n{}\n\n", knowledge_context)
    };

    format!(
        "CUSTOMER'S QUESTION:\n{}\n\n\
         DRAFT REPLY TO REVIEW:\n{}\n\n\
         {kb_section}\
         Review the draft reply against the customer's question and the knowledge base above. \
         Score relevance and grounding, then list any specific issues.",
        customer_question,
        draft_reply
    )
}

/// Parse the validator's JSON response into a structured result.
fn parse_validation_result(parsed: &Value, latency_ms: i64) -> ValidationResult {
    let relevance_score = parsed
        .get("relevance_score")
        .and_then(|v| v.as_f64())
        .filter(|v| *v >= 0.0 && *v <= 1.0)
        .unwrap_or(0.0);

    let grounding_score = parsed
        .get("grounding_score")
        .and_then(|v| v.as_f64())
        .filter(|v| *v >= 0.0 && *v <= 1.0)
        .unwrap_or(0.0);

    let reason = parsed
        .get("reason")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string());

    let issues: Vec<String> = parsed
        .get("issues")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    ValidationResult {
        relevance_score,
        grounding_score,
        reason,
        issues,
        validated: true,
        latency_ms,
        // The pass/fail decision is made by the caller using thresholds
        passed: true,
    }
}

/// Validate a draft reply against the customer's question and knowledge context.
///
/// Returns a `ValidationResult` that indicates whether the draft passes the
/// quality checks.  If validation is disabled or the API call fails, returns
/// a skipped result that does not block the draft.
///
/// # Arguments
/// * `state` — Application state (for AI provider access)
/// * `config` — Validation thresholds and enabled flag
/// * `customer_question` — The last inbound message from the customer
/// * `draft_reply` — The AI-generated reply text to validate
/// * `knowledge_context` — The knowledge base and service catalog context
///   that was provided to the generator (so the validator can check grounding)
pub async fn validate_draft(
    state: &AppState,
    config: &ValidationConfig,
    customer_question: &str,
    draft_reply: &str,
    knowledge_context: &str,
) -> ValidationResult {
    // If validation is disabled, skip it.
    if !config.enabled {
        return ValidationResult::skipped("validation is disabled (VALIDATION_ENABLED=false)");
    }

    // If the draft is empty, there's nothing to validate.
    if draft_reply.trim().is_empty() {
        return ValidationResult::skipped("draft reply is empty, nothing to validate");
    }

    // If the customer question is empty, we can't check relevance.
    if customer_question.trim().is_empty() {
        return ValidationResult::skipped(
            "no customer question available, cannot validate relevance",
        );
    }

    let system_prompt = build_validator_system_prompt();
    let user_prompt = build_validator_user_prompt(
        customer_question,
        draft_reply,
        knowledge_context,
    );

    let start = std::time::Instant::now();

    let parsed = match ask_json(state, &system_prompt, &user_prompt, VALIDATION_MAX_TOKENS).await {
        Ok(value) => value,
        Err(err) => {
            tracing::warn!(
                "validation call failed (draft will proceed without validation): {}",
                err.message()
            );
            return ValidationResult::skipped(format!(
                "validation API call failed: {}",
                err.message()
            ));
        }
    };

    let latency_ms = start.elapsed().as_millis() as i64;
    let mut result = parse_validation_result(&parsed, latency_ms);

    // Apply thresholds to determine pass/fail.
    let relevance_pass = result.relevance_score >= config.relevance_threshold;
    let grounding_pass = result.grounding_score >= config.grounding_threshold;

    if !relevance_pass || !grounding_pass {
        result.passed = false;

        // If the validator didn't provide a reason, synthesize one.
        if result.reason.is_none() {
            let mut reasons = Vec::new();
            if !relevance_pass {
                reasons.push(format!(
                    "relevance score {:.2} is below threshold {:.2}",
                    result.relevance_score, config.relevance_threshold
                ));
            }
            if !grounding_pass {
                reasons.push(format!(
                    "grounding score {:.2} is below threshold {:.2}",
                    result.grounding_score, config.grounding_threshold
                ));
            }
            result.reason = Some(reasons.join("; "));
        }
    }

    result
}

/// Quick validation that only checks if the AI provider is configured.
/// Returns `true` if validation can run, `false` if it will be skipped.
pub fn is_validation_available(state: &AppState) -> bool {
    let config = ValidationConfig::from_env();
    if !config.enabled {
        return false;
    }
    // Check if AI provider is configured.
    crate::services::claude_client::ClaudeConfig::from_state(state).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_config_defaults() {
        // Clear any env overrides.
        std::env::remove_var("VALIDATION_RELEVANCE_THRESHOLD");
        std::env::remove_var("VALIDATION_GROUNDING_THRESHOLD");
        std::env::remove_var("VALIDATION_ENABLED");

        let config = ValidationConfig::from_env();
        assert_eq!(config.relevance_threshold, DEFAULT_RELEVANCE_THRESHOLD);
        assert_eq!(config.grounding_threshold, DEFAULT_GROUNDING_THRESHOLD);
        assert!(config.enabled);
    }

    #[test]
    fn test_validation_config_disabled() {
        std::env::set_var("VALIDATION_ENABLED", "false");
        let config = ValidationConfig::from_env();
        assert!(!config.enabled);
        std::env::remove_var("VALIDATION_ENABLED");
    }

    #[test]
    fn test_validation_result_skipped() {
        let result = ValidationResult::skipped("test reason");
        assert!(!result.validated);
        assert!(result.passed); // Skipped should not block
        assert_eq!(result.reason.as_deref(), Some("test reason"));
    }

    #[test]
    fn test_parse_validation_result_valid() {
        let parsed = json!({
            "relevance_score": 0.9,
            "grounding_score": 0.8,
            "reason": null,
            "issues": []
        });
        let result = parse_validation_result(&parsed, 100);
        assert!(result.validated);
        assert_eq!(result.relevance_score, 0.9);
        assert_eq!(result.grounding_score, 0.8);
        assert!(result.reason.is_none());
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_parse_validation_result_with_issues() {
        let parsed = json!({
            "relevance_score": 0.5,
            "grounding_score": 0.3,
            "reason": "Reply does not address the customer's question about pricing",
            "issues": ["Does not mention pricing", "Claims features not in knowledge base"]
        });
        let result = parse_validation_result(&parsed, 200);
        assert!(result.validated);
        assert_eq!(result.relevance_score, 0.5);
        assert_eq!(result.grounding_score, 0.3);
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_parse_validation_result_invalid_scores() {
        // Scores out of range should default to 0.0.
        let parsed = json!({
            "relevance_score": 1.5,
            "grounding_score": -0.5,
            "reason": null,
            "issues": []
        });
        let result = parse_validation_result(&parsed, 50);
        assert_eq!(result.relevance_score, 0.0);
        assert_eq!(result.grounding_score, 0.0);
    }

    #[test]
    fn test_build_validator_user_prompt() {
        let prompt = build_validator_user_prompt(
            "What are your hours?",
            "We're open 9-5 Monday to Friday.",
            "BUSINESS HOURS: Monday-Friday 9am-5pm",
        );
        assert!(prompt.contains("What are your hours?"));
        assert!(prompt.contains("We're open 9-5 Monday to Friday."));
        assert!(prompt.contains("BUSINESS HOURS"));
    }

    #[test]
    fn test_build_validator_user_prompt_empty_kb() {
        let prompt = build_validator_user_prompt(
            "Question?",
            "Answer.",
            "",
        );
        assert!(prompt.contains("No knowledge base entries were provided"));
    }
}
