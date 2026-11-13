//! Autonomous Resolution — Phase 29
//!
//! Enables the AI agent to resolve conversations autonomously without human approval.
//! Mirrors Intercom's "Fin AI Agent" that "resolves conversations autonomously across all channels."
//! Includes confidence-based auto-send, escalation rules, and resolution analytics.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolutionConfig {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub auto_send_threshold: f64,
    pub require_validation: bool,
    pub validation_threshold: f64,
    pub max_autonomous_attempts: i32,
    pub escalate_on_low_confidence: bool,
    pub low_confidence_threshold: f64,
    pub escalate_on_sensitive_topic: bool,
    pub sensitive_topics: Option<Value>,
    pub allowed_channels: Option<Value>,
    pub cooldown_seconds: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EscalationRule {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub condition_type: String,
    pub condition_value: f64,
    pub condition_topic: Option<String>,
    pub condition_keyword: Option<String>,
    pub action_type: String,
    pub assign_to_agent_id: Option<String>,
    pub assign_to_team_id: Option<String>,
    pub notify_webhook: Option<String>,
    pub notification_message: Option<String>,
    pub is_active: bool,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolutionLog {
    pub id: String,
    pub organization_id: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub channel: String,
    pub attempt_number: i32,
    pub ai_confidence: f64,
    pub validation_score: Option<f64>,
    pub validation_passed: Option<bool>,
    pub resolution_type: String,
    pub escalated: bool,
    pub escalation_rule_id: Option<String>,
    pub escalation_reason: Option<String>,
    pub message_sent: bool,
    pub message_id: Option<String>,
    pub customer_resolved: Option<bool>,
    pub customer_feedback_rating: Option<i32>,
    pub outcome: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolutionAnalytics {
    pub id: String,
    pub organization_id: String,
    pub window_start: String,
    pub window_end: String,
    pub total_attempts: i32,
    pub successful_resolutions: i32,
    pub escalated_count: i32,
    pub auto_sent_count: i32,
    pub avg_confidence: f64,
    pub avg_validation_score: Option<f64>,
    pub resolution_rate: f64,
    pub escalation_rate: f64,
    pub customer_satisfaction_rate: Option<f64>,
    pub top_escalation_reasons: Option<Value>,
    pub resolution_by_channel: Option<Value>,
    pub resolution_by_topic: Option<Value>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPolicy {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: String,
    pub condition: String,
    pub action: String,
    pub is_active: bool,
    pub priority: i32,
    pub metadata: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolutionSummary {
    pub enabled: bool,
    pub total_attempts_7d: i32,
    pub total_attempts_30d: i32,
    pub successful_resolutions_7d: i32,
    pub escalated_count_7d: i32,
    pub resolution_rate_7d: f64,
    pub escalation_rate_7d: f64,
    pub avg_confidence_7d: f64,
    pub auto_send_threshold: f64,
    pub active_escalation_rules: i32,
    pub active_policies: i32,
    pub recent_logs: Vec<ResolutionLog>,
    pub top_escalation_reasons: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolutionDecision {
    pub should_auto_send: bool,
    pub reason: String,
    pub escalation_rule: Option<EscalationRule>,
    pub policies_violated: Vec<AgentPolicy>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigResult {
    pub config_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EscalationResult {
    pub rule_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsSnapshot {
    pub snapshot_id: String,
    pub window_start: String,
    pub window_end: String,
    pub resolution_rate: f64,
    pub escalation_rate: f64,
}

// ——— Core Functions ———

/// Get resolution summary dashboard data
pub async fn get_resolution_summary(state: &AppState) -> Result<ResolutionSummary, sqlx::Error> {
    // Config
    let config_row = sqlx::query(
        "SELECT enabled, auto_send_threshold FROM resolution_config WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let enabled = config_row
        .as_ref()
        .map(|r| r.try_get::<bool, _>(0).unwrap_or(false))
        .unwrap_or(false);

    let auto_send_threshold = config_row
        .map(|r| r.try_get::<f64, _>(1).unwrap_or(0.8))
        .unwrap_or(0.8);

    // Total attempts in last 7 days
    let total_attempts_7d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resolution_logs WHERE created_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total attempts in last 30 days
    let total_attempts_30d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resolution_logs WHERE created_at >= NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Successful resolutions in last 7 days
    let successful_resolutions_7d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resolution_logs WHERE created_at >= NOW() - INTERVAL '7 days' AND outcome = 'resolved'"
    )
    .fetch_one(&state.db)
    .await?;

    // Escalated count in last 7 days
    let escalated_count_7d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resolution_logs WHERE created_at >= NOW() - INTERVAL '7 days' AND escalated = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Resolution rate
    let resolution_rate_7d = if total_attempts_7d > 0 {
        successful_resolutions_7d as f64 / total_attempts_7d as f64
    } else {
        0.0
    };

    // Escalation rate
    let escalation_rate_7d = if total_attempts_7d > 0 {
        escalated_count_7d as f64 / total_attempts_7d as f64
    } else {
        0.0
    };

    // Average confidence in last 7 days
    let avg_confidence_row: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(ai_confidence) FROM resolution_logs WHERE created_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_optional(&state.db)
    .await?;

    let avg_confidence_7d = avg_confidence_row.map(|r| r.0).unwrap_or(0.0);

    // Active escalation rules
    let active_escalation_rules: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM escalation_rules WHERE is_active = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Active policies
    let active_policies: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resolution_agent_policies WHERE is_active = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Recent logs
    let log_rows = sqlx::query(
        "SELECT id, organization_id, conversation_id, contact_id, channel, attempt_number, \
         ai_confidence, validation_score, validation_passed, resolution_type, escalated, \
         escalation_rule_id, escalation_reason, message_sent, message_id, customer_resolved, \
         customer_feedback_rating, outcome, error_message, duration_ms, created_at, completed_at \
         FROM resolution_logs ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_logs: Vec<ResolutionLog> = log_rows
        .iter()
        .map(|row| ResolutionLog {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            conversation_id: row.try_get(2).unwrap_or_default(),
            contact_id: row.try_get(3).unwrap_or_default(),
            channel: row.try_get(4).unwrap_or_default(),
            attempt_number: row.try_get(5).unwrap_or(1i32),
            ai_confidence: row.try_get(6).unwrap_or(0.0),
            validation_score: row.try_get(7).ok(),
            validation_passed: row.try_get(8).ok(),
            resolution_type: row.try_get(9).unwrap_or_default(),
            escalated: row.try_get(10).unwrap_or(false),
            escalation_rule_id: row.try_get(11).ok(),
            escalation_reason: row.try_get(12).ok(),
            message_sent: row.try_get(13).unwrap_or(false),
            message_id: row.try_get(14).ok(),
            customer_resolved: row.try_get(15).ok(),
            customer_feedback_rating: row.try_get(16).ok(),
            outcome: row.try_get(17).unwrap_or_default(),
            error_message: row.try_get(18).ok(),
            duration_ms: row.try_get(19).ok(),
            created_at: row.try_get(20).unwrap_or_default(),
            completed_at: row.try_get(21).ok(),
        })
        .collect();

    // Top escalation reasons
    let reason_rows = sqlx::query(
        "SELECT escalation_reason, COUNT(*) as cnt FROM resolution_logs \
         WHERE escalated = true AND escalation_reason IS NOT NULL \
         AND created_at >= NOW() - INTERVAL '7 days' \
         GROUP BY escalation_reason ORDER BY cnt DESC LIMIT 5"
    )
    .fetch_all(&state.db)
    .await?;

    let top_escalation_reasons: Vec<Value> = reason_rows
        .iter()
        .map(|row| {
            json!({
                "reason": row.try_get::<String, _>(0).unwrap_or_default(),
                "count": row.try_get::<i64, _>(1).unwrap_or(0i64),
            })
        })
        .collect();

    Ok(ResolutionSummary {
        enabled,
        total_attempts_7d,
        total_attempts_30d,
        successful_resolutions_7d,
        escalated_count_7d,
        resolution_rate_7d,
        escalation_rate_7d,
        avg_confidence_7d,
        auto_send_threshold,
        active_escalation_rules,
        active_policies,
        recent_logs,
        top_escalation_reasons,
    })
}

/// Get resolution config
pub async fn get_resolution_config(state: &AppState) -> Result<ResolutionConfig, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, auto_send_threshold, require_validation, \
         validation_threshold, max_autonomous_attempts, escalate_on_low_confidence, \
         low_confidence_threshold, escalate_on_sensitive_topic, sensitive_topics, \
         allowed_channels, cooldown_seconds, created_at, updated_at \
         FROM resolution_config WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(ResolutionConfig {
        id: row.try_get(0).unwrap_or_default(),
        organization_id: row.try_get(1).unwrap_or_default(),
        enabled: row.try_get(2).unwrap_or(false),
        auto_send_threshold: row.try_get(3).unwrap_or(0.8),
        require_validation: row.try_get(4).unwrap_or(true),
        validation_threshold: row.try_get(5).unwrap_or(0.7),
        max_autonomous_attempts: row.try_get(6).unwrap_or(3i32),
        escalate_on_low_confidence: row.try_get(7).unwrap_or(true),
        low_confidence_threshold: row.try_get(8).unwrap_or(0.4),
        escalate_on_sensitive_topic: row.try_get(9).unwrap_or(true),
        sensitive_topics: row.try_get(10).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
        allowed_channels: row.try_get(11).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
        cooldown_seconds: row.try_get(12).unwrap_or(300i32),
        created_at: row.try_get(13).unwrap_or_default(),
        updated_at: row.try_get(14).unwrap_or_default(),
    })
}

/// Update resolution config
pub async fn update_resolution_config(
    state: &AppState,
    config: &ResolutionConfig,
) -> Result<ConfigResult, sqlx::Error> {
    let sensitive_topics = serde_json::to_string(&config.sensitive_topics).unwrap_or_default();
    let allowed_channels = serde_json::to_string(&config.allowed_channels).unwrap_or_default();

    sqlx::query(
        "UPDATE resolution_config SET \
         enabled = $1, auto_send_threshold = $2, require_validation = $3, \
         validation_threshold = $4, max_autonomous_attempts = $5, \
         escalate_on_low_confidence = $6, low_confidence_threshold = $7, \
         escalate_on_sensitive_topic = $8, sensitive_topics = $9, \
         allowed_channels = $10, cooldown_seconds = $11, updated_at = NOW() \
         WHERE id = $12"
    )
    .bind(config.enabled)
    .bind(config.auto_send_threshold)
    .bind(config.require_validation)
    .bind(config.validation_threshold)
    .bind(config.max_autonomous_attempts)
    .bind(config.escalate_on_low_confidence)
    .bind(config.low_confidence_threshold)
    .bind(config.escalate_on_sensitive_topic)
    .bind(sensitive_topics)
    .bind(allowed_channels)
    .bind(config.cooldown_seconds)
    .bind(&config.id)
    .execute(&state.db)
    .await?;

    Ok(ConfigResult {
        config_id: config.id.clone(),
        message: "Resolution config updated successfully".to_string(),
    })
}

/// List escalation rules
pub async fn list_escalation_rules(
    state: &AppState,
    is_active: Option<bool>,
) -> Result<Vec<EscalationRule>, sqlx::Error> {
    let where_clause = if is_active.is_some() {
        " WHERE is_active = $1"
    } else {
        ""
    };

    let query = format!(
        "SELECT id, organization_id, name, description, condition_type, condition_value, \
         condition_topic, condition_keyword, action_type, assign_to_agent_id, \
         assign_to_team_id, notify_webhook, notification_message, is_active, priority, \
         created_at, updated_at \
         FROM escalation_rules{} ORDER BY priority DESC",
        where_clause
    );

    let rows = if is_active.is_some() {
        sqlx::query(&query)
            .bind(is_active.unwrap())
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query(&query)
            .fetch_all(&state.db)
            .await?
    };

    let rules: Vec<EscalationRule> = rows
        .iter()
        .map(|row| EscalationRule {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            condition_type: row.try_get(4).unwrap_or_default(),
            condition_value: row.try_get(5).unwrap_or(0.0),
            condition_topic: row.try_get(6).ok(),
            condition_keyword: row.try_get(7).ok(),
            action_type: row.try_get(8).unwrap_or_default(),
            assign_to_agent_id: row.try_get(9).ok(),
            assign_to_team_id: row.try_get(10).ok(),
            notify_webhook: row.try_get(11).ok(),
            notification_message: row.try_get(12).ok(),
            is_active: row.try_get(13).unwrap_or(true),
            priority: row.try_get(14).unwrap_or(0i32),
            created_at: row.try_get(15).unwrap_or_default(),
            updated_at: row.try_get(16).unwrap_or_default(),
        })
        .collect();

    Ok(rules)
}

/// Create escalation rule
pub async fn create_escalation_rule(
    state: &AppState,
    name: &str,
    condition_type: &str,
    condition_value: f64,
    action_type: &str,
    description: Option<&str>,
    condition_topic: Option<&str>,
    condition_keyword: Option<&str>,
    assign_to_agent_id: Option<&str>,
    assign_to_team_id: Option<&str>,
    notify_webhook: Option<&str>,
    notification_message: Option<&str>,
    priority: i32,
) -> Result<EscalationResult, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO escalation_rules (id, organization_id, name, description, condition_type, \
         condition_value, condition_topic, condition_keyword, action_type, assign_to_agent_id, \
         assign_to_team_id, notify_webhook, notification_message, is_active, priority) \
         VALUES ($1, 'org-default', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, true, $13)"
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(condition_type)
    .bind(condition_value)
    .bind(condition_topic)
    .bind(condition_keyword)
    .bind(action_type)
    .bind(assign_to_agent_id)
    .bind(assign_to_team_id)
    .bind(notify_webhook)
    .bind(notification_message)
    .bind(priority)
    .execute(&state.db)
    .await?;

    Ok(EscalationResult {
        rule_id: id,
        message: "Escalation rule created successfully".to_string(),
    })
}

/// Update escalation rule
pub async fn update_escalation_rule(
    state: &AppState,
    rule: &EscalationRule,
) -> Result<EscalationResult, sqlx::Error> {
    sqlx::query(
        "UPDATE escalation_rules SET \
         name = $1, description = $2, condition_type = $3, condition_value = $4, \
         condition_topic = $5, condition_keyword = $6, action_type = $7, \
         assign_to_agent_id = $8, assign_to_team_id = $9, notify_webhook = $10, \
         notification_message = $11, is_active = $12, priority = $13, updated_at = NOW() \
         WHERE id = $14"
    )
    .bind(&rule.name)
    .bind(&rule.description)
    .bind(&rule.condition_type)
    .bind(rule.condition_value)
    .bind(&rule.condition_topic)
    .bind(&rule.condition_keyword)
    .bind(&rule.action_type)
    .bind(&rule.assign_to_agent_id)
    .bind(&rule.assign_to_team_id)
    .bind(&rule.notify_webhook)
    .bind(&rule.notification_message)
    .bind(rule.is_active)
    .bind(rule.priority)
    .bind(&rule.id)
    .execute(&state.db)
    .await?;

    Ok(EscalationResult {
        rule_id: rule.id.clone(),
        message: "Escalation rule updated successfully".to_string(),
    })
}

/// Delete escalation rule
pub async fn delete_escalation_rule(
    state: &AppState,
    rule_id: &str,
) -> Result<EscalationResult, sqlx::Error> {
    sqlx::query("DELETE FROM escalation_rules WHERE id = $1")
        .bind(rule_id)
        .execute(&state.db)
        .await?;

    Ok(EscalationResult {
        rule_id: rule_id.to_string(),
        message: "Escalation rule deleted successfully".to_string(),
    })
}

/// Evaluate whether to auto-send or escalate based on confidence and rules
pub async fn evaluate_resolution_decision(
    state: &AppState,
    conversation_id: &str,
    ai_confidence: f64,
    validation_score: Option<f64>,
    topic: Option<&str>,
) -> Result<ResolutionDecision, sqlx::Error> {
    // Get config
    let config = get_resolution_config(state).await?;

    // If not enabled, require human review
    if !config.enabled {
        return Ok(ResolutionDecision {
            should_auto_send: false,
            reason: "Autonomous resolution is not enabled".to_string(),
            escalation_rule: None,
            policies_violated: vec![],
        });
    }

    // Check confidence threshold
    if ai_confidence < config.auto_send_threshold {
        // Find matching escalation rule
        let rules = list_escalation_rules(state, Some(true)).await?;
        let matching_rule = rules.iter().find(|r| {
            r.condition_type == "confidence_below" && r.is_active
        });

        return Ok(ResolutionDecision {
            should_auto_send: false,
            reason: format!(
                "AI confidence {:.2} below auto-send threshold {:.2}",
                ai_confidence, config.auto_send_threshold
            ),
            escalation_rule: matching_rule.cloned(),
            policies_violated: vec![],
        });
    }

    // Check validation
    if config.require_validation {
        if let Some(score) = validation_score {
            if score < config.validation_threshold {
                return Ok(ResolutionDecision {
                    should_auto_send: false,
                    reason: format!(
                        "Validation score {:.2} below threshold {:.2}",
                        score, config.validation_threshold
                    ),
                    escalation_rule: None,
                    policies_violated: vec![],
                });
            }
        }
    }

    // Check sensitive topics
    if config.escalate_on_sensitive_topic {
        if let Some(topics_json) = &config.sensitive_topics {
            if let Value::Array(topics) = topics_json {
                if let Some(topic_str) = topic {
                    for topic_val in topics {
                        if let Value::String(t) = topic_val {
                            if t.to_lowercase() == topic_str.to_lowercase() {
                                let rules = list_escalation_rules(state, Some(true)).await?;
                                let matching_rule = rules.iter().find(|r| {
                                    r.condition_type == "topic_match" && r.is_active
                                });

                                return Ok(ResolutionDecision {
                                    should_auto_send: false,
                                    reason: format!("Sensitive topic '{}' requires human review", topic_str),
                                    escalation_rule: matching_rule.cloned(),
                                    policies_violated: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Check low confidence escalation
    if config.escalate_on_low_confidence && ai_confidence < config.low_confidence_threshold {
        let rules = list_escalation_rules(state, Some(true)).await?;
        let matching_rule = rules.iter().find(|r| {
            r.condition_type == "confidence_below" && r.is_active
        });

        return Ok(ResolutionDecision {
            should_auto_send: false,
            reason: format!(
                "AI confidence {:.2} below low-confidence threshold {:.2}",
                ai_confidence, config.low_confidence_threshold
            ),
            escalation_rule: matching_rule.cloned(),
            policies_violated: vec![],
        });
    }

    // All checks passed — auto-send
    Ok(ResolutionDecision {
        should_auto_send: true,
        reason: "All checks passed — confidence and validation meet thresholds".to_string(),
        escalation_rule: None,
        policies_violated: vec![],
    })
}

/// Log a resolution attempt
pub async fn log_resolution_attempt(
    state: &AppState,
    conversation_id: &str,
    contact_id: &str,
    channel: &str,
    ai_confidence: f64,
    validation_score: Option<f64>,
    validation_passed: Option<bool>,
    decision: &ResolutionDecision,
) -> Result<ResolutionLog, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();

    let escalated = !decision.should_auto_send;
    let escalation_rule_id = decision.escalation_rule.as_ref().map(|r| r.id.clone());
    let escalation_reason = if escalated {
        Some(decision.reason.clone())
    } else {
        None
    };

    let outcome = if decision.should_auto_send {
        "auto_sent"
    } else {
        "escalated"
    };

    sqlx::query(
        "INSERT INTO resolution_logs (id, organization_id, conversation_id, contact_id, \
         channel, attempt_number, ai_confidence, validation_score, validation_passed, \
         resolution_type, escalated, escalation_rule_id, escalation_reason, message_sent, \
         outcome, created_at) \
         VALUES ($1, 'org-default', $2, $3, $4, 1, $5, $6, $7, $8, $9, $10, $11, $12, NOW())"
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(contact_id)
    .bind(channel)
    .bind(ai_confidence)
    .bind(validation_score)
    .bind(validation_passed)
    .bind(if decision.should_auto_send { "auto_send" } else { "escalated" })
    .bind(escalated)
    .bind(&escalation_rule_id)
    .bind(&escalation_reason)
    .bind(outcome)
    .execute(&state.db)
    .await?;

    // Fetch the created log
    let row = sqlx::query(
        "SELECT id, organization_id, conversation_id, contact_id, channel, attempt_number, \
         ai_confidence, validation_score, validation_passed, resolution_type, escalated, \
         escalation_rule_id, escalation_reason, message_sent, message_id, customer_resolved, \
         customer_feedback_rating, outcome, error_message, duration_ms, created_at, completed_at \
         FROM resolution_logs WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    Ok(ResolutionLog {
        id: row.try_get(0).unwrap_or_default(),
        organization_id: row.try_get(1).unwrap_or_default(),
        conversation_id: row.try_get(2).unwrap_or_default(),
        contact_id: row.try_get(3).unwrap_or_default(),
        channel: row.try_get(4).unwrap_or_default(),
        attempt_number: row.try_get(5).unwrap_or(1i32),
        ai_confidence: row.try_get(6).unwrap_or(0.0),
        validation_score: row.try_get(7).ok(),
        validation_passed: row.try_get(8).ok(),
        resolution_type: row.try_get(9).unwrap_or_default(),
        escalated: row.try_get(10).unwrap_or(false),
        escalation_rule_id: row.try_get(11).ok(),
        escalation_reason: row.try_get(12).ok(),
        message_sent: row.try_get(13).unwrap_or(false),
        message_id: row.try_get(14).ok(),
        customer_resolved: row.try_get(15).ok(),
        customer_feedback_rating: row.try_get(16).ok(),
        outcome: row.try_get(17).unwrap_or_default(),
        error_message: row.try_get(18).ok(),
        duration_ms: row.try_get(19).ok(),
        created_at: row.try_get(20).unwrap_or_default(),
        completed_at: row.try_get(21).ok(),
    })
}

/// Generate analytics snapshot
pub async fn generate_analytics_snapshot(
    state: &AppState,
    days: i32,
) -> Result<AnalyticsSnapshot, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let window_start = format!("NOW() - INTERVAL '{}' days", days);
    let window_end = "NOW()".to_string();

    // Collect metrics
    let metrics_row = sqlx::query(
        &format!(
            "SELECT \
             COUNT(*) as total, \
             COUNT(*) FILTER (WHERE outcome = 'resolved') as resolved, \
             COUNT(*) FILTER (WHERE escalated = true) as escalated, \
             COUNT(*) FILTER (WHERE outcome = 'auto_sent') as auto_sent, \
             AVG(ai_confidence) as avg_confidence, \
             AVG(validation_score) as avg_validation \
             FROM resolution_logs \
             WHERE created_at >= {}::TIMESTAMPTZ",
            window_start
        )
    )
    .fetch_one(&state.db)
    .await?;

    let total: i64 = metrics_row.try_get(0).unwrap_or(0i64);
    let resolved: i64 = metrics_row.try_get(1).unwrap_or(0i64);
    let escalated: i64 = metrics_row.try_get(2).unwrap_or(0i64);
    let auto_sent: i64 = metrics_row.try_get(3).unwrap_or(0i64);
    let avg_confidence: Option<f64> = metrics_row.try_get(4).ok();
    let avg_validation: Option<f64> = metrics_row.try_get(5).ok();

    let resolution_rate = if total > 0 { resolved as f64 / total as f64 } else { 0.0 };
    let escalation_rate = if total > 0 { escalated as f64 / total as f64 } else { 0.0 };

    // Top escalation reasons
    let reason_rows = sqlx::query(
        &format!(
            "SELECT escalation_reason, COUNT(*) as cnt FROM resolution_logs \
             WHERE escalated = true AND escalation_reason IS NOT NULL \
             AND created_at >= {}::TIMESTAMPTZ \
             GROUP BY escalation_reason ORDER BY cnt DESC LIMIT 5",
            window_start
        )
    )
    .fetch_all(&state.db)
    .await?;

    let top_reasons: Vec<Value> = reason_rows
        .iter()
        .map(|row| {
            json!({
                "reason": row.try_get::<String, _>(0).unwrap_or_default(),
                "count": row.try_get::<i64, _>(1).unwrap_or(0i64),
            })
        })
        .collect();

    // Resolution by channel
    let channel_rows = sqlx::query(
        &format!(
            "SELECT channel, COUNT(*) as cnt FROM resolution_logs \
             WHERE created_at >= {}::TIMESTAMPTZ \
             GROUP BY channel",
            window_start
        )
    )
    .fetch_all(&state.db)
    .await?;

    let by_channel: Value = json!(
        channel_rows.iter()
            .map(|row| {
                (
                    row.try_get::<String, _>(0).unwrap_or_default(),
                    row.try_get::<i64, _>(1).unwrap_or(0i64),
                )
            })
            .collect::<std::collections::HashMap<_, _>>()
    );

    // Insert snapshot
    sqlx::query(
        "INSERT INTO resolution_analytics (id, organization_id, window_start, window_end, \
         total_attempts, successful_resolutions, escalated_count, auto_sent_count, \
         avg_confidence, avg_validation_score, resolution_rate, escalation_rate, \
         top_escalation_reasons, resolution_by_channel) \
         VALUES ($1, 'org-default', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
    )
    .bind(&id)
    .bind(&window_start)
    .bind(&window_end)
    .bind(total as i32)
    .bind(resolved as i32)
    .bind(escalated as i32)
    .bind(auto_sent as i32)
    .bind(avg_confidence.unwrap_or(0.0))
    .bind(avg_validation)
    .bind(resolution_rate)
    .bind(escalation_rate)
    .bind(serde_json::to_string(&top_reasons).unwrap_or_default())
    .bind(serde_json::to_string(&by_channel).unwrap_or_default())
    .execute(&state.db)
    .await?;

    Ok(AnalyticsSnapshot {
        snapshot_id: id,
        window_start,
        window_end,
        resolution_rate,
        escalation_rate,
    })
}

/// List agent policies
pub async fn list_agent_policies(
    state: &AppState,
    is_active: Option<bool>,
) -> Result<Vec<AgentPolicy>, sqlx::Error> {
    let where_clause = if is_active.is_some() {
        " WHERE is_active = $1"
    } else {
        ""
    };

    let query = format!(
        "SELECT id, organization_id, name, description, policy_type, condition, action, \
         is_active, priority, metadata, created_at, updated_at \
         FROM resolution_agent_policies{} ORDER BY priority DESC",
        where_clause
    );

    let rows = if is_active.is_some() {
        sqlx::query(&query)
            .bind(is_active.unwrap())
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query(&query)
            .fetch_all(&state.db)
            .await?
    };

    let policies: Vec<AgentPolicy> = rows
        .iter()
        .map(|row| AgentPolicy {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            policy_type: row.try_get(4).unwrap_or_default(),
            condition: row.try_get(5).unwrap_or_default(),
            action: row.try_get(6).unwrap_or_default(),
            is_active: row.try_get(7).unwrap_or(true),
            priority: row.try_get(8).unwrap_or(0i32),
            metadata: row.try_get(9).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            created_at: row.try_get(10).unwrap_or_default(),
            updated_at: row.try_get(11).unwrap_or_default(),
        })
        .collect();

    Ok(policies)
}

/// List resolution logs
pub async fn list_resolution_logs(
    state: &AppState,
    outcome: Option<&str>,
    escalated: Option<bool>,
    limit: i32,
    offset: i32,
) -> Result<Vec<ResolutionLog>, sqlx::Error> {
    let limit = limit.max(1).min(100);
    let offset = offset.max(0);

    let mut conditions = Vec::new();
    let mut param_count = 1;

    if let Some(out) = outcome {
        conditions.push(format!("outcome = ${}", param_count));
        param_count += 1;
    }
    if let Some(esc) = escalated {
        conditions.push(format!("escalated = ${}", param_count));
        param_count += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT id, organization_id, conversation_id, contact_id, channel, attempt_number, \
         ai_confidence, validation_score, validation_passed, resolution_type, escalated, \
         escalation_rule_id, escalation_reason, message_sent, message_id, customer_resolved, \
         customer_feedback_rating, outcome, error_message, duration_ms, created_at, completed_at \
         FROM resolution_logs{} \
         ORDER BY created_at DESC \
         LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(out) = outcome {
        query_builder = query_builder.bind(out);
    }
    if let Some(esc) = escalated {
        query_builder = query_builder.bind(esc);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(&state.db).await?;

    let logs: Vec<ResolutionLog> = rows
        .iter()
        .map(|row| ResolutionLog {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            conversation_id: row.try_get(2).unwrap_or_default(),
            contact_id: row.try_get(3).unwrap_or_default(),
            channel: row.try_get(4).unwrap_or_default(),
            attempt_number: row.try_get(5).unwrap_or(1i32),
            ai_confidence: row.try_get(6).unwrap_or(0.0),
            validation_score: row.try_get(7).ok(),
            validation_passed: row.try_get(8).ok(),
            resolution_type: row.try_get(9).unwrap_or_default(),
            escalated: row.try_get(10).unwrap_or(false),
            escalation_rule_id: row.try_get(11).ok(),
            escalation_reason: row.try_get(12).ok(),
            message_sent: row.try_get(13).unwrap_or(false),
            message_id: row.try_get(14).ok(),
            customer_resolved: row.try_get(15).ok(),
            customer_feedback_rating: row.try_get(16).ok(),
            outcome: row.try_get(17).unwrap_or_default(),
            error_message: row.try_get(18).ok(),
            duration_ms: row.try_get(19).ok(),
            created_at: row.try_get(20).unwrap_or_default(),
            completed_at: row.try_get(21).ok(),
        })
        .collect();

    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let summary = ResolutionSummary {
            enabled: true,
            total_attempts_7d: 100,
            total_attempts_30d: 400,
            successful_resolutions_7d: 80,
            escalated_count_7d: 20,
            resolution_rate_7d: 0.8,
            escalation_rate_7d: 0.2,
            avg_confidence_7d: 0.85,
            auto_send_threshold: 0.8,
            active_escalation_rules: 3,
            active_policies: 3,
            recent_logs: vec![],
            top_escalation_reasons: vec![],
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"resolution_rate_7d\":0.8"));
    }

    #[test]
    fn test_decision_serialization() {
        let decision = ResolutionDecision {
            should_auto_send: true,
            reason: "All checks passed".to_string(),
            escalation_rule: None,
            policies_violated: vec![],
        };

        let json = serde_json::to_string(&decision).unwrap();
        assert!(json.contains("\"should_auto_send\":true"));
    }

    #[test]
    fn test_analytics_snapshot() {
        let snapshot = AnalyticsSnapshot {
            snapshot_id: "test-id".to_string(),
            window_start: "NOW() - INTERVAL '7' days".to_string(),
            window_end: "NOW()".to_string(),
            resolution_rate: 0.75,
            escalation_rate: 0.25,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"resolution_rate\":0.75"));
    }
}
