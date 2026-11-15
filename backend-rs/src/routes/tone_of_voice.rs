//! Tone of Voice & Guidance Routes — Phase 33
//!
//! Thin gRPC client forwarding to svc-tone-of-voice; REST shape unchanged.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db::AppState;

use newax_proto::tone_of_voice::v1::{
    tone_of_voice_service_client::ToneOfVoiceServiceClient, ApplyPresetRequest as GrpcApplyPresetRequest,
    CreateAnalyticsRequest as GrpcCreateAnalyticsRequest, CreateGuidanceRequest as GrpcCreateGuidanceRequest, Empty,
    RuleIdRequest,
};

async fn client() -> Result<ToneOfVoiceServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("TONE_OF_VOICE_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50070".to_string());
    ToneOfVoiceServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("tone-of-voice service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

// ——— Request/Response Types (wire shape mirrors the old service types) ———

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
    pub forbidden_phrases: Option<serde_json::Value>,
    pub required_phrases: Option<serde_json::Value>,
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

// ——— proto <-> wire conversions ———

fn config_from_grpc(c: newax_proto::tone_of_voice::v1::ToneConfig) -> ToneConfig {
    ToneConfig {
        id: c.id,
        organization_id: c.organization_id,
        tone_style: c.tone_style,
        personality: c.personality,
        answer_length: c.answer_length,
        language: c.language,
        formality_level: c.formality_level,
        humor_enabled: c.humor_enabled,
        empathy_level: c.empathy_level,
        conciseness_level: c.conciseness_level,
        custom_instructions: c.custom_instructions,
        forbidden_phrases: c.forbidden_phrases_json.and_then(|s| serde_json::from_str(&s).ok()),
        required_phrases: c.required_phrases_json.and_then(|s| serde_json::from_str(&s).ok()),
        greeting_template: c.greeting_template,
        closing_template: c.closing_template,
        escalation_template: c.escalation_template,
        enabled: c.enabled,
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

fn config_to_grpc(c: ToneConfig) -> newax_proto::tone_of_voice::v1::ToneConfig {
    newax_proto::tone_of_voice::v1::ToneConfig {
        id: c.id,
        organization_id: c.organization_id,
        tone_style: c.tone_style,
        personality: c.personality,
        answer_length: c.answer_length,
        language: c.language,
        formality_level: c.formality_level,
        humor_enabled: c.humor_enabled,
        empathy_level: c.empathy_level,
        conciseness_level: c.conciseness_level,
        custom_instructions: c.custom_instructions,
        forbidden_phrases_json: c.forbidden_phrases.map(|v| v.to_string()),
        required_phrases_json: c.required_phrases.map(|v| v.to_string()),
        greeting_template: c.greeting_template,
        closing_template: c.closing_template,
        escalation_template: c.escalation_template,
        enabled: c.enabled,
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

fn rule_from_grpc(r: newax_proto::tone_of_voice::v1::GuidanceRule) -> GuidanceRule {
    GuidanceRule {
        id: r.id,
        organization_id: r.organization_id,
        name: r.name,
        description: r.description,
        rule_type: r.rule_type,
        instruction: r.instruction,
        priority: r.priority,
        applies_to: r.applies_to,
        enabled: r.enabled,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

fn rule_to_grpc(r: GuidanceRule) -> newax_proto::tone_of_voice::v1::GuidanceRule {
    newax_proto::tone_of_voice::v1::GuidanceRule {
        id: r.id,
        organization_id: r.organization_id,
        name: r.name,
        description: r.description,
        rule_type: r.rule_type,
        instruction: r.instruction,
        priority: r.priority,
        applies_to: r.applies_to,
        enabled: r.enabled,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

fn analytics_from_grpc(a: newax_proto::tone_of_voice::v1::ToneAnalytics) -> ToneAnalytics {
    ToneAnalytics {
        id: a.id,
        organization_id: a.organization_id,
        tone_style: a.tone_style,
        conversation_count: a.conversation_count,
        avg_satisfaction: a.avg_satisfaction,
        avg_resolution_rate: a.avg_resolution_rate,
        avg_response_length: a.avg_response_length,
        thumbs_up_count: a.thumbs_up_count,
        thumbs_down_count: a.thumbs_down_count,
        window_start: a.window_start,
        window_end: a.window_end,
        created_at: a.created_at,
    }
}

fn preset_from_grpc(p: newax_proto::tone_of_voice::v1::TonePreset) -> TonePreset {
    TonePreset {
        id: p.id,
        name: p.name,
        description: p.description,
        tone_style: p.tone_style,
        personality: p.personality,
        answer_length: p.answer_length,
        formality_level: p.formality_level,
        humor_enabled: p.humor_enabled,
        empathy_level: p.empathy_level,
        conciseness_level: p.conciseness_level,
        greeting_template: p.greeting_template,
        closing_template: p.closing_template,
        escalation_template: p.escalation_template,
        custom_instructions: p.custom_instructions,
    }
}

// ——— Handlers ———

/// GET /tone-voice/summary
pub async fn get_summary(State(_state): State<AppState>) -> Result<Json<ToneSummary>, (StatusCode, String)> {
    let mut c = client().await?;
    let s = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(ToneSummary {
        enabled: s.enabled,
        current_tone_style: s.current_tone_style,
        current_personality: s.current_personality,
        current_answer_length: s.current_answer_length,
        total_guidance_rules: s.total_guidance_rules,
        enabled_guidance_rules: s.enabled_guidance_rules,
        total_presets: s.total_presets,
        recent_analytics: s.recent_analytics.into_iter().map(analytics_from_grpc).collect(),
        guidance_rules: s.guidance_rules.into_iter().map(rule_from_grpc).collect(),
        presets: s.presets.into_iter().map(preset_from_grpc).collect(),
    }))
}

/// GET /tone-voice/config
pub async fn get_config(State(_state): State<AppState>) -> Result<Json<ToneConfig>, (StatusCode, String)> {
    let mut c = client().await?;
    let config = c.get_config(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(config_from_grpc(config)))
}

/// PUT /tone-voice/config
pub async fn update_config(
    State(_state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let current = c.get_config(Empty {}).await.map_err(grpc_err)?.into_inner();
    let mut config = config_from_grpc(current);

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

    let result = c.update_config(config_to_grpc(config)).await.map_err(grpc_err)?.into_inner();
    Ok(Json(ConfigResult { config_id: result.config_id, message: result.message }))
}

/// POST /tone-voice/apply-preset
pub async fn apply_preset(
    State(_state): State<AppState>,
    Json(req): Json<ApplyPresetRequest>,
) -> Result<Json<ConfigResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .apply_preset(GrpcApplyPresetRequest { preset_id: req.preset_id })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(ConfigResult { config_id: result.config_id, message: result.message }))
}

/// GET /tone-voice/guidance
pub async fn list_guidance(State(_state): State<AppState>) -> Result<Json<Vec<GuidanceRule>>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.list_guidance(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(resp.rules.into_iter().map(rule_from_grpc).collect()))
}

/// POST /tone-voice/guidance
pub async fn create_guidance(
    State(_state): State<AppState>,
    Json(req): Json<CreateGuidanceRequest>,
) -> Result<Json<GuidanceResult>, (StatusCode, String)> {
    let rule_type = req.rule_type.unwrap_or_else(|| "style".to_string());
    let priority = req.priority.unwrap_or(0);
    let mut c = client().await?;
    let result = c
        .create_guidance(GrpcCreateGuidanceRequest { name: req.name, instruction: req.instruction, rule_type, priority })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(GuidanceResult { rule_id: result.rule_id, message: result.message }))
}

/// PUT /tone-voice/guidance/:id
pub async fn update_guidance(
    State(_state): State<AppState>,
    axum::extract::Path(rule_id): axum::extract::Path<String>,
    Json(req): Json<UpdateGuidanceRequest>,
) -> Result<Json<GuidanceResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let existing = c.list_guidance(Empty {}).await.map_err(grpc_err)?.into_inner();
    let mut rule = existing
        .rules
        .into_iter()
        .map(rule_from_grpc)
        .find(|r| r.id == rule_id)
        .ok_or((StatusCode::NOT_FOUND, "Guidance rule not found".to_string()))?;

    if let Some(v) = req.name { rule.name = v; }
    if let Some(v) = req.description { rule.description = Some(v); }
    if let Some(v) = req.rule_type { rule.rule_type = v; }
    if let Some(v) = req.instruction { rule.instruction = v; }
    if let Some(v) = req.priority { rule.priority = v; }
    if let Some(v) = req.applies_to { rule.applies_to = v; }
    if let Some(v) = req.enabled { rule.enabled = v; }

    let result = c.update_guidance(rule_to_grpc(rule)).await.map_err(grpc_err)?.into_inner();
    Ok(Json(GuidanceResult { rule_id: result.rule_id, message: result.message }))
}

/// DELETE /tone-voice/guidance/:id
pub async fn delete_guidance(
    State(_state): State<AppState>,
    axum::extract::Path(rule_id): axum::extract::Path<String>,
) -> Result<Json<GuidanceResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c.delete_guidance(RuleIdRequest { rule_id }).await.map_err(grpc_err)?.into_inner();
    Ok(Json(GuidanceResult { rule_id: result.rule_id, message: result.message }))
}

/// GET /tone-voice/prompt-additions
pub async fn get_prompt_additions(State(_state): State<AppState>) -> Result<Json<PromptAdditions>, (StatusCode, String)> {
    let mut c = client().await?;
    let a = c.get_prompt_additions(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(PromptAdditions {
        tone_instructions: a.tone_instructions,
        guidance_instructions: a.guidance_instructions,
        greeting_template: a.greeting_template,
        closing_template: a.closing_template,
        escalation_template: a.escalation_template,
    }))
}

/// POST /tone-voice/analytics
pub async fn create_analytics(
    State(_state): State<AppState>,
    Json(req): Json<CreateAnalyticsRequest>,
) -> Result<Json<AnalyticsResult>, (StatusCode, String)> {
    let mut c = client().await?;
    let result = c
        .create_analytics_snapshot(GrpcCreateAnalyticsRequest {
            tone_style: req.tone_style,
            conversation_count: req.conversation_count,
            avg_satisfaction: req.avg_satisfaction,
            avg_resolution_rate: req.avg_resolution_rate,
            avg_response_length: req.avg_response_length,
            thumbs_up: req.thumbs_up,
            thumbs_down: req.thumbs_down,
        })
        .await
        .map_err(grpc_err)?
        .into_inner();
    Ok(Json(AnalyticsResult { analytics_id: result.analytics_id, message: result.message }))
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
