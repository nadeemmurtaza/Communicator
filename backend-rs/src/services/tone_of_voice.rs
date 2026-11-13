//! Tone of Voice & Guidance — Phase 33
//!
//! Configurable AI personality, tone, answer length control, and custom instructions
//! that coach the AI on support policies and communication style.
//! Core part of Intercom's TRAIN phase.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToneConfig {
    pub id: String,
    pub organization_id: String,
    pub tone_style: String,
    pub personality: String,
    pub answer_length: String,
    pub language: String,
    pub formality_level: i32,
    pub humor_enabled: bool,
    pub empathy_level: i32,
    pub conciseness_level: i32,
    pub custom_instructions: Option<String>,
    pub forbidden_phrases: Option<Value>,
    pub required_phrases: Option<Value>,
    pub greeting_template: String,
    pub closing_template: String,
    pub escalation_template: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuidanceRule {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub instruction: String,
    pub priority: i32,
    pub applies_to: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToneAnalytics {
    pub id: String,
    pub organization_id: String,
    pub tone_style: String,
    pub conversation_count: i32,
    pub avg_satisfaction: f64,
    pub avg_resolution_rate: f64,
    pub avg_response_length: i32,
    pub thumbs_up_count: i32,
    pub thumbs_down_count: i32,
    pub window_start: String,
    pub window_end: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TonePreset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tone_style: String,
    pub personality: String,
    pub answer_length: String,
    pub formality_level: i32,
    pub humor_enabled: bool,
    pub empathy_level: i32,
    pub conciseness_level: i32,
    pub greeting_template: String,
    pub closing_template: String,
    pub escalation_template: String,
    pub custom_instructions: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToneSummary {
    pub enabled: bool,
    pub current_tone_style: String,
    pub current_personality: String,
    pub current_answer_length: String,
    pub total_guidance_rules: i32,
    pub enabled_guidance_rules: i32,
    pub total_presets: i32,
    pub recent_analytics: Vec<ToneAnalytics>,
    pub guidance_rules: Vec<GuidanceRule>,
    pub presets: Vec<TonePreset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigResult {
    pub config_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuidanceResult {
    pub rule_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptAdditions {
    pub tone_instructions: String,
    pub guidance_instructions: String,
    pub greeting_template: String,
    pub closing_template: String,
    pub escalation_template: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsResult {
    pub analytics_id: String,
    pub message: String,
}

// ——— Core Functions ———

/// Get tone of voice summary dashboard data
pub async fn get_summary(state: &AppState) -> Result<ToneSummary, sqlx::Error> {
    // Config
    let config_row = sqlx::query(
        "SELECT enabled, tone_style, personality, answer_length \
         FROM tone_config WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let enabled = config_row.as_ref().map(|r| r.try_get::<bool, _>(0).unwrap_or(true)).unwrap_or(true);
    let current_tone_style = config_row.as_ref().map(|r| r.try_get::<String, _>(1).unwrap_or_default()).unwrap_or_default();
    let current_personality = config_row.as_ref().map(|r| r.try_get::<String, _>(2).unwrap_or_default()).unwrap_or_default();
    let current_answer_length = config_row.as_ref().map(|r| r.try_get::<String, _>(3).unwrap_or_default()).unwrap_or_default();

    // Total guidance rules
    let total_guidance_rules: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM guidance_rules"
    )
    .fetch_one(&state.db)
    .await?;

    // Enabled guidance rules
    let enabled_guidance_rules: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM guidance_rules WHERE enabled = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Total presets
    let total_presets: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tone_presets"
    )
    .fetch_one(&state.db)
    .await?;

    // Recent analytics
    let analytics_rows = sqlx::query(
        "SELECT id, organization_id, tone_style, conversation_count, avg_satisfaction, \
         avg_resolution_rate, avg_response_length, thumbs_up_count, thumbs_down_count, \
         window_start, window_end, created_at \
         FROM tone_analytics ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_analytics: Vec<ToneAnalytics> = analytics_rows
        .iter()
        .map(|row| ToneAnalytics {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            tone_style: row.try_get(2).unwrap_or_default(),
            conversation_count: row.try_get(3).unwrap_or(0i32),
            avg_satisfaction: row.try_get(4).unwrap_or(0.0f64),
            avg_resolution_rate: row.try_get(5).unwrap_or(0.0f64),
            avg_response_length: row.try_get(6).unwrap_or(0i32),
            thumbs_up_count: row.try_get(7).unwrap_or(0i32),
            thumbs_down_count: row.try_get(8).unwrap_or(0i32),
            window_start: row.try_get(9).unwrap_or_default(),
            window_end: row.try_get(10).unwrap_or_default(),
            created_at: row.try_get(11).unwrap_or_default(),
        })
        .collect();

    // Guidance rules
    let guidance_rows = sqlx::query(
        "SELECT id, organization_id, name, description, rule_type, instruction, \
         priority, applies_to, enabled, created_at, updated_at \
         FROM guidance_rules ORDER BY priority DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let guidance_rules: Vec<GuidanceRule> = guidance_rows
        .iter()
        .map(|row| GuidanceRule {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            rule_type: row.try_get(4).unwrap_or_default(),
            instruction: row.try_get(5).unwrap_or_default(),
            priority: row.try_get(6).unwrap_or(0i32),
            applies_to: row.try_get(7).unwrap_or_default(),
            enabled: row.try_get(8).unwrap_or(true),
            created_at: row.try_get(9).unwrap_or_default(),
            updated_at: row.try_get(10).unwrap_or_default(),
        })
        .collect();

    // Presets
    let preset_rows = sqlx::query(
        "SELECT id, name, description, tone_style, personality, answer_length, \
         formality_level, humor_enabled, empathy_level, conciseness_level, \
         greeting_template, closing_template, escalation_template, custom_instructions \
         FROM tone_presets ORDER BY name"
    )
    .fetch_all(&state.db)
    .await?;

    let presets: Vec<TonePreset> = preset_rows
        .iter()
        .map(|row| TonePreset {
            id: row.try_get(0).unwrap_or_default(),
            name: row.try_get(1).unwrap_or_default(),
            description: row.try_get(2).ok(),
            tone_style: row.try_get(3).unwrap_or_default(),
            personality: row.try_get(4).unwrap_or_default(),
            answer_length: row.try_get(5).unwrap_or_default(),
            formality_level: row.try_get(6).unwrap_or(0i32),
            humor_enabled: row.try_get(7).unwrap_or(false),
            empathy_level: row.try_get(8).unwrap_or(0i32),
            conciseness_level: row.try_get(9).unwrap_or(0i32),
            greeting_template: row.try_get(10).unwrap_or_default(),
            closing_template: row.try_get(11).unwrap_or_default(),
            escalation_template: row.try_get(12).unwrap_or_default(),
            custom_instructions: row.try_get(13).ok(),
        })
        .collect();

    Ok(ToneSummary {
        enabled,
        current_tone_style,
        current_personality,
        current_answer_length,
        total_guidance_rules,
        enabled_guidance_rules,
        total_presets,
        recent_analytics,
        guidance_rules,
        presets,
    })
}

/// Get current tone config
pub async fn get_config(state: &AppState) -> Result<ToneConfig, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, tone_style, personality, answer_length, language, \
         formality_level, humor_enabled, empathy_level, conciseness_level, \
         custom_instructions, forbidden_phrases, required_phrases, \
         greeting_template, closing_template, escalation_template, enabled, \
         created_at, updated_at \
         FROM tone_config WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(r) => Ok(ToneConfig {
            id: r.try_get(0).unwrap_or_default(),
            organization_id: r.try_get(1).unwrap_or_default(),
            tone_style: r.try_get(2).unwrap_or_default(),
            personality: r.try_get(3).unwrap_or_default(),
            answer_length: r.try_get(4).unwrap_or_default(),
            language: r.try_get(5).unwrap_or_default(),
            formality_level: r.try_get(6).unwrap_or(0i32),
            humor_enabled: r.try_get(7).unwrap_or(false),
            empathy_level: r.try_get(8).unwrap_or(0i32),
            conciseness_level: r.try_get(9).unwrap_or(0i32),
            custom_instructions: r.try_get(10).ok(),
            forbidden_phrases: r.try_get(11).ok(),
            required_phrases: r.try_get(12).ok(),
            greeting_template: r.try_get(13).unwrap_or_default(),
            closing_template: r.try_get(14).unwrap_or_default(),
            escalation_template: r.try_get(15).unwrap_or_default(),
            enabled: r.try_get(16).unwrap_or(true),
            created_at: r.try_get(17).unwrap_or_default(),
            updated_at: r.try_get(18).unwrap_or_default(),
        }),
        None => Err(sqlx::Error::RowNotFound),
    }
}

/// Update tone config
pub async fn update_config(state: &AppState, config: &ToneConfig) -> Result<ConfigResult, sqlx::Error> {
    sqlx::query(
        "UPDATE tone_config SET tone_style = $1, personality = $2, answer_length = $3, \
         language = $4, formality_level = $5, humor_enabled = $6, empathy_level = $7, \
         conciseness_level = $8, custom_instructions = $9, forbidden_phrases = $10, \
         required_phrases = $11, greeting_template = $12, closing_template = $13, \
         escalation_template = $14, enabled = $15, updated_at = NOW() \
         WHERE id = $16"
    )
    .bind(&config.tone_style)
    .bind(&config.personality)
    .bind(&config.answer_length)
    .bind(&config.language)
    .bind(&config.formality_level)
    .bind(&config.humor_enabled)
    .bind(&config.empathy_level)
    .bind(&config.conciseness_level)
    .bind(&config.custom_instructions)
    .bind(config.forbidden_phrases.as_ref().map(|v| v.to_string()))
    .bind(config.required_phrases.as_ref().map(|v| v.to_string()))
    .bind(&config.greeting_template)
    .bind(&config.closing_template)
    .bind(&config.escalation_template)
    .bind(&config.enabled)
    .bind(&config.id)
    .execute(&state.db)
    .await?;

    Ok(ConfigResult {
        config_id: config.id.clone(),
        message: "Tone config updated successfully".to_string(),
    })
}

/// Apply preset to current config
pub async fn apply_preset(state: &AppState, preset_id: &str) -> Result<ConfigResult, sqlx::Error> {
    // Get preset
    let preset_row = sqlx::query(
        "SELECT tone_style, personality, answer_length, formality_level, humor_enabled, \
         empathy_level, conciseness_level, greeting_template, closing_template, \
         escalation_template, custom_instructions \
         FROM tone_presets WHERE id = $1"
    )
    .bind(preset_id)
    .fetch_optional(&state.db)
    .await?;

    match preset_row {
        Some(p) => {
            let tone_style: String = p.try_get(0).unwrap_or_default();
            let personality: String = p.try_get(1).unwrap_or_default();
            let answer_length: String = p.try_get(2).unwrap_or_default();
            let formality_level: i32 = p.try_get(3).unwrap_or(0i32);
            let humor_enabled: bool = p.try_get(4).unwrap_or(false);
            let empathy_level: i32 = p.try_get(5).unwrap_or(0i32);
            let conciseness_level: i32 = p.try_get(6).unwrap_or(0i32);
            let greeting_template: String = p.try_get(7).unwrap_or_default();
            let closing_template: String = p.try_get(8).unwrap_or_default();
            let escalation_template: String = p.try_get(9).unwrap_or_default();
            let custom_instructions: Option<String> = p.try_get(10).ok();

            sqlx::query(
                "UPDATE tone_config SET tone_style = $1, personality = $2, answer_length = $3, \
                 formality_level = $4, humor_enabled = $5, empathy_level = $6, \
                 conciseness_level = $7, greeting_template = $8, closing_template = $9, \
                 escalation_template = $10, custom_instructions = $11, updated_at = NOW() \
                 WHERE organization_id = 'org-default'"
            )
            .bind(&tone_style)
            .bind(&personality)
            .bind(&answer_length)
            .bind(&formality_level)
            .bind(&humor_enabled)
            .bind(&empathy_level)
            .bind(&conciseness_level)
            .bind(&greeting_template)
            .bind(&closing_template)
            .bind(&escalation_template)
            .bind(&custom_instructions)
            .execute(&state.db)
            .await?;

            Ok(ConfigResult {
                config_id: "tone-config-default".to_string(),
                message: format!("Preset '{}' applied successfully", preset_id),
            })
        }
        None => Err(sqlx::Error::RowNotFound),
    }
}

/// List all guidance rules
pub async fn list_guidance_rules(state: &AppState) -> Result<Vec<GuidanceRule>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, organization_id, name, description, rule_type, instruction, \
         priority, applies_to, enabled, created_at, updated_at \
         FROM guidance_rules ORDER BY priority DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let rules: Vec<GuidanceRule> = rows
        .iter()
        .map(|row| GuidanceRule {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            rule_type: row.try_get(4).unwrap_or_default(),
            instruction: row.try_get(5).unwrap_or_default(),
            priority: row.try_get(6).unwrap_or(0i32),
            applies_to: row.try_get(7).unwrap_or_default(),
            enabled: row.try_get(8).unwrap_or(true),
            created_at: row.try_get(9).unwrap_or_default(),
            updated_at: row.try_get(10).unwrap_or_default(),
        })
        .collect();

    Ok(rules)
}

/// Create a new guidance rule
pub async fn create_guidance_rule(
    state: &AppState,
    name: &str,
    instruction: &str,
    rule_type: &str,
    priority: i32,
) -> Result<GuidanceResult, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO guidance_rules (organization_id, name, rule_type, instruction, priority, enabled) \
         VALUES ('org-default', $1, $2, $3, $4, true) \
         RETURNING id"
    )
    .bind(name)
    .bind(rule_type)
    .bind(instruction)
    .bind(priority)
    .fetch_one(&state.db)
    .await?;

    let id: String = row.try_get(0).unwrap_or_default();

    Ok(GuidanceResult {
        rule_id: id,
        message: "Guidance rule created successfully".to_string(),
    })
}

/// Update a guidance rule
pub async fn update_guidance_rule(
    state: &AppState,
    rule_id: &str,
    rule: &GuidanceRule,
) -> Result<GuidanceResult, sqlx::Error> {
    sqlx::query(
        "UPDATE guidance_rules SET name = $1, description = $2, rule_type = $3, \
         instruction = $4, priority = $5, applies_to = $6, enabled = $7, updated_at = NOW() \
         WHERE id = $8"
    )
    .bind(&rule.name)
    .bind(&rule.description)
    .bind(&rule.rule_type)
    .bind(&rule.instruction)
    .bind(&rule.priority)
    .bind(&rule.applies_to)
    .bind(&rule.enabled)
    .bind(rule_id)
    .execute(&state.db)
    .await?;

    Ok(GuidanceResult {
        rule_id: rule_id.to_string(),
        message: "Guidance rule updated successfully".to_string(),
    })
}

/// Delete a guidance rule
pub async fn delete_guidance_rule(state: &AppState, rule_id: &str) -> Result<GuidanceResult, sqlx::Error> {
    sqlx::query("DELETE FROM guidance_rules WHERE id = $1")
        .bind(rule_id)
        .execute(&state.db)
        .await?;

    Ok(GuidanceResult {
        rule_id: rule_id.to_string(),
        message: "Guidance rule deleted successfully".to_string(),
    })
}

/// Get prompt additions for the AI reply engine
/// This is the key function that injects tone and guidance into the AI system prompt
pub async fn get_prompt_additions(state: &AppState) -> Result<PromptAdditions, sqlx::Error> {
    // Get config
    let config_row = sqlx::query(
        "SELECT tone_style, personality, answer_length, formality_level, humor_enabled, \
         empathy_level, conciseness_level, custom_instructions, greeting_template, \
         closing_template, escalation_template, enabled \
         FROM tone_config WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let greeting_template = config_row.as_ref().map(|r| r.try_get::<String, _>(8).unwrap_or_default()).unwrap_or_else(|| "Hello! How can I help you today?".to_string());
    let closing_template = config_row.as_ref().map(|r| r.try_get::<String, _>(9).unwrap_or_default()).unwrap_or_else(|| "Is there anything else I can help you with?".to_string());
    let escalation_template = config_row.as_ref().map(|r| r.try_get::<String, _>(10).unwrap_or_default()).unwrap_or_else(|| "Let me connect you with a team member who can help.".to_string());

    let enabled = config_row.as_ref().map(|r| r.try_get::<bool, _>(11).unwrap_or(true)).unwrap_or(true);

    if !enabled {
        return Ok(PromptAdditions {
            tone_instructions: String::new(),
            guidance_instructions: String::new(),
            greeting_template,
            closing_template,
            escalation_template,
        });
    }

    let mut tone_parts = Vec::new();

    if let Some(r) = &config_row {
        let tone_style: String = r.try_get(0).unwrap_or_default();
        let personality: String = r.try_get(1).unwrap_or_default();
        let answer_length: String = r.try_get(2).unwrap_or_default();
        let formality_level: i32 = r.try_get(3).unwrap_or(5i32);
        let humor_enabled: bool = r.try_get(4).unwrap_or(false);
        let empathy_level: i32 = r.try_get(5).unwrap_or(5i32);
        let conciseness_level: i32 = r.try_get(6).unwrap_or(5i32);
        let custom_instructions: Option<String> = r.try_get(7).ok();

        tone_parts.push(format!("- Tone style: {}", tone_style));
        tone_parts.push(format!("- Personality: {}", personality));

        match answer_length.as_str() {
            "short" => tone_parts.push("- Answer length: Keep responses brief and to the point (1-3 sentences)".to_string()),
            "long" => tone_parts.push("- Answer length: Provide thorough, detailed responses with examples".to_string()),
            _ => tone_parts.push("- Answer length: Provide balanced responses (3-5 sentences)".to_string()),
        }

        if formality_level >= 7 {
            tone_parts.push("- Formality: Use formal, professional language".to_string());
        } else if formality_level <= 3 {
            tone_parts.push("- Formality: Use casual, conversational language".to_string());
        } else {
            tone_parts.push("- Formality: Use a balanced, semi-formal tone".to_string());
        }

        if humor_enabled {
            tone_parts.push("- Humor: Light humor is appropriate when it feels natural".to_string());
        } else {
            tone_parts.push("- Humor: Avoid humor; stay professional".to_string());
        }

        if empathy_level >= 7 {
            tone_parts.push("- Empathy: Always acknowledge feelings and show deep understanding".to_string());
        } else if empathy_level <= 3 {
            tone_parts.push("- Empathy: Focus on solutions rather than emotional acknowledgment".to_string());
        } else {
            tone_parts.push("- Empathy: Show moderate empathy when appropriate".to_string());
        }

        if conciseness_level >= 7 {
            tone_parts.push("- Conciseness: Be very direct; avoid unnecessary elaboration".to_string());
        } else if conciseness_level <= 3 {
            tone_parts.push("- Conciseness: Elaborate and provide context; don't worry about length".to_string());
        } else {
            tone_parts.push("- Conciseness: Balance detail with brevity".to_string());
        }

        if let Some(custom) = custom_instructions {
            tone_parts.push(format!("- Custom: {}", custom));
        }
    }

    // Get enabled guidance rules
    let guidance_rows = sqlx::query(
        "SELECT instruction FROM guidance_rules WHERE enabled = true ORDER BY priority DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let mut guidance_parts = Vec::new();
    for row in &guidance_rows {
        let instruction: String = row.try_get(0).unwrap_or_default();
        guidance_parts.push(instruction);
    }

    Ok(PromptAdditions {
        tone_instructions: if tone_parts.is_empty() {
            String::new()
        } else {
            format!("\n\nTone and Style Guidelines:\n{}", tone_parts.join("\n"))
        },
        guidance_instructions: if guidance_parts.is_empty() {
            String::new()
        } else {
            format!("\n\nGuidance Rules (follow in order of priority):\n{}", guidance_parts.join("\n"))
        },
        greeting_template,
        closing_template,
        escalation_template,
    })
}

/// Create analytics snapshot
pub async fn create_analytics_snapshot(
    state: &AppState,
    tone_style: &str,
    conversation_count: i32,
    avg_satisfaction: f64,
    avg_resolution_rate: f64,
    avg_response_length: i32,
    thumbs_up: i32,
    thumbs_down: i32,
) -> Result<AnalyticsResult, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO tone_analytics (organization_id, tone_style, conversation_count, \
         avg_satisfaction, avg_resolution_rate, avg_response_length, \
         thumbs_up_count, thumbs_down_count, window_start, window_end) \
         VALUES ('org-default', $1, $2, $3, $4, $5, $6, $7, NOW() - INTERVAL '7 days', NOW()) \
         RETURNING id"
    )
    .bind(tone_style)
    .bind(conversation_count)
    .bind(avg_satisfaction)
    .bind(avg_resolution_rate)
    .bind(avg_response_length)
    .bind(thumbs_up)
    .bind(thumbs_down)
    .fetch_one(&state.db)
    .await?;

    let id: String = row.try_get(0).unwrap_or_default();

    Ok(AnalyticsResult {
        analytics_id: id,
        message: "Analytics snapshot created".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_config_serialization() {
        let config = ToneConfig {
            id: "test".to_string(),
            organization_id: "org-1".to_string(),
            tone_style: "friendly".to_string(),
            personality: "enthusiastic".to_string(),
            answer_length: "medium".to_string(),
            language: "en".to_string(),
            formality_level: 4,
            humor_enabled: false,
            empathy_level: 8,
            conciseness_level: 5,
            custom_instructions: None,
            forbidden_phrases: None,
            required_phrases: None,
            greeting_template: "Hi there!".to_string(),
            closing_template: "Anything else?".to_string(),
            escalation_template: "Let me get help.".to_string(),
            enabled: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("friendly"));
        assert!(json.contains("enthusiastic"));
    }

    #[test]
    fn test_prompt_additions_serialization() {
        let additions = PromptAdditions {
            tone_instructions: "- Tone style: friendly".to_string(),
            guidance_instructions: "Be helpful".to_string(),
            greeting_template: "Hi!".to_string(),
            closing_template: "Bye!".to_string(),
            escalation_template: "Getting help.".to_string(),
        };
        let json = serde_json::to_string(&additions).unwrap();
        assert!(json.contains("friendly"));
        assert!(json.contains("Be helpful"));
    }

    #[test]
    fn test_guidance_rule_serialization() {
        let rule = GuidanceRule {
            id: "rule-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "Be Empathetic".to_string(),
            description: Some("Always show empathy".to_string()),
            rule_type: "style".to_string(),
            instruction: "Acknowledge feelings first".to_string(),
            priority: 10,
            applies_to: "all".to_string(),
            enabled: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Be Empathetic"));
        assert!(json.contains("10"));
    }
}
