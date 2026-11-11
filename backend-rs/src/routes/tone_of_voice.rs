//! Tone of Voice & Guidance Routes — Phase 33
//!
//! API endpoints for managing AI tone configuration, guidance rules, presets,
//! and retrieving prompt additions for the reply engine.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db::AppState;
use crate::services::tone_of_voice;

// ——— Request/Response Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateConfigRequest {
    pub tone_style: Option<String>,
    pub personality: Option<String>,
    pub answer_length: Option<String>,
    pub language: Option<String>,
    pub formality_level: Option<i32>,
    pub humor_enabled: Option<bool>,
    pub empathy_level: Option<i32>,
    pub conciseness_level: Option<i32>,
    pub custom_instructions: Option<String>,
    pub greeting_template: Option<String>,
    pub closing_template: Option<String>,
    pub escalation_template: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateGuidanceRequest {
    pub name: String,
    pub instruction: String,
    pub rule_type: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateGuidanceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rule_type: Option<String>,
    pub instruction: Option<String>,
    pub priority: Option<i32>,
    pub applies_to: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplyPresetRequest {
    pub preset_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateAnalyticsRequest {
    pub tone_style: String,
    pub conversation_count: i32,
    pub avg_satisfaction: f64,
    pub avg_resolution_rate: f64,
    pub avg_response_length: i32,
    pub thumbs_up: i32,
    pub thumbs_down: i32,
}

// ——— Handlers ———

/// GET /tone-voice/summary
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<tone_of_voice::ToneSummary>, (StatusCode, String)> {
    let summary = tone_of_voice::get_summary(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(summary))
}

/// GET /tone-voice/config
pub async fn get_config(State(state): State<AppState>) -> Result<Json<tone_of_voice::ToneConfig>, (StatusCode, String)> {
    let config = tone_of_voice::get_config(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(config))
}

/// PUT /tone-voice/config
pub async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<tone_of_voice::ConfigResult>, (StatusCode, String)> {
    let mut config = tone_of_voice::get_config(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(v) = req.tone_style { config.tone_style = v; }
    if let Some(v) = req.personality { config.personality = v; }
    if let Some(v) = req.answer_length { config.answer_length = v; }
    if let Some(v) = req.language { config.language = v; }
    if let Some(v) = req.formality_level { config.formality_level = v; }
    if let Some(v) = req.humor_enabled { config.humor_enabled = v; }
    if let Some(v) = req.empathy_level { config.empathy_level = v; }
    if let Some(v) = req.conciseness_level { config.conciseness_level = v; }
    if let Some(v) = req.custom_instructions { config.custom_instructions = Some(v); }
    if let Some(v) = req.greeting_template { config.greeting_template = v; }
    if let Some(v) = req.closing_template { config.closing_template = v; }
    if let Some(v) = req.escalation_template { config.escalation_template = v; }
    if let Some(v) = req.enabled { config.enabled = v; }

    let result = tone_of_voice::update_config(&state, &config).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

/// POST /tone-voice/apply-preset
pub async fn apply_preset(
    State(state): State<AppState>,
    Json(req): Json<ApplyPresetRequest>,
) -> Result<Json<tone_of_voice::ConfigResult>, (StatusCode, String)> {
    let result = tone_of_voice::apply_preset(&state, &req.preset_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

/// GET /tone-voice/guidance
pub async fn list_guidance(State(state): State<AppState>) -> Result<Json<Vec<tone_of_voice::GuidanceRule>>, (StatusCode, String)> {
    let rules = tone_of_voice::list_guidance_rules(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rules))
}

/// POST /tone-voice/guidance
pub async fn create_guidance(
    State(state): State<AppState>,
    Json(req): Json<CreateGuidanceRequest>,
) -> Result<Json<tone_of_voice::GuidanceResult>, (StatusCode, String)> {
    let rule_type = req.rule_type.unwrap_or_else(|| "style".to_string());
    let priority = req.priority.unwrap_or(0);
    let result = tone_of_voice::create_guidance_rule(&state, &req.name, &req.instruction, &rule_type, priority).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

/// PUT /tone-voice/guidance/:id
pub async fn update_guidance(
    State(state): State<AppState>,
    axum::extract::Path(rule_id): axum::extract::Path<String>,
    Json(req): Json<UpdateGuidanceRequest>,
) -> Result<Json<tone_of_voice::GuidanceResult>, (StatusCode, String)> {
    let mut rule = tone_of_voice::list_guidance_rules(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .find(|r| r.id == rule_id)
        .ok_or((StatusCode::NOT_FOUND, "Guidance rule not found".to_string()))?;

    if let Some(v) = req.name { rule.name = v; }
    if let Some(v) = req.description { rule.description = Some(v); }
    if let Some(v) = req.rule_type { rule.rule_type = v; }
    if let Some(v) = req.instruction { rule.instruction = v; }
    if let Some(v) = req.priority { rule.priority = v; }
    if let Some(v) = req.applies_to { rule.applies_to = v; }
    if let Some(v) = req.enabled { rule.enabled = v; }

    let result = tone_of_voice::update_guidance_rule(&state, &rule_id, &rule).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

/// DELETE /tone-voice/guidance/:id
pub async fn delete_guidance(
    State(state): State<AppState>,
    axum::extract::Path(rule_id): axum::extract::Path<String>,
) -> Result<Json<tone_of_voice::GuidanceResult>, (StatusCode, String)> {
    let result = tone_of_voice::delete_guidance_rule(&state, &rule_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

/// GET /tone-voice/prompt-additions
pub async fn get_prompt_additions(State(state): State<AppState>) -> Result<Json<tone_of_voice::PromptAdditions>, (StatusCode, String)> {
    let additions = tone_of_voice::get_prompt_additions(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(additions))
}

/// POST /tone-voice/analytics
pub async fn create_analytics(
    State(state): State<AppState>,
    Json(req): Json<CreateAnalyticsRequest>,
) -> Result<Json<tone_of_voice::AnalyticsResult>, (StatusCode, String)> {
    let result = tone_of_voice::create_analytics_snapshot(
        &state,
        &req.tone_style,
        req.conversation_count,
        req.avg_satisfaction,
        req.avg_resolution_rate,
        req.avg_response_length,
        req.thumbs_up,
        req.thumbs_down,
    ).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

// ——— Router ———

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/tone-voice/summary", axum::routing::get(get_summary))
        .route("/tone-voice/config", axum::routing::get(get_config))
        .route("/tone-voice/config", axum::routing::put(update_config))
        .route("/tone-voice/apply-preset", axum::routing::post(apply_preset))
        .route("/tone-voice/guidance", axum::routing::get(list_guidance))
        .route("/tone-voice/guidance", axum::routing::post(create_guidance))
        .route("/tone-voice/guidance/:id", axum::routing::put(update_guidance))
        .route("/tone-voice/guidance/:id", axum::routing::delete(delete_guidance))
        .route("/tone-voice/prompt-additions", axum::routing::get(get_prompt_additions))
        .route("/tone-voice/analytics", axum::routing::post(create_analytics))
}
