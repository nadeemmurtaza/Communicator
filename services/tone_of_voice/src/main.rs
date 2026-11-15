use newax_tone_of_voice as tov;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.tone_of_voice.v1");
}

use proto::{
    tone_of_voice_service_server::{ToneOfVoiceService, ToneOfVoiceServiceServer},
    AnalyticsResult as ProtoAnalyticsResult, ApplyPresetRequest, ConfigResult as ProtoConfigResult,
    CreateAnalyticsRequest, CreateGuidanceRequest, Empty, GuidanceResult as ProtoGuidanceResult,
    GuidanceRule as ProtoGuidanceRule, GuidanceRulesResponse, PromptAdditions as ProtoPromptAdditions, RuleIdRequest,
    ToneAnalytics as ProtoAnalytics, ToneConfig as ProtoConfig, TonePreset as ProtoPreset, ToneSummary as ProtoSummary,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn config_to_proto(c: tov::ToneConfig) -> ProtoConfig {
    ProtoConfig {
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

fn config_from_proto(c: ProtoConfig) -> tov::ToneConfig {
    tov::ToneConfig {
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

fn rule_to_proto(r: tov::GuidanceRule) -> ProtoGuidanceRule {
    ProtoGuidanceRule {
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

fn rule_from_proto(r: ProtoGuidanceRule) -> tov::GuidanceRule {
    tov::GuidanceRule {
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

fn analytics_to_proto(a: tov::ToneAnalytics) -> ProtoAnalytics {
    ProtoAnalytics {
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

fn preset_to_proto(p: tov::TonePreset) -> ProtoPreset {
    ProtoPreset {
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

#[tonic::async_trait]
impl ToneOfVoiceService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<ProtoSummary>, Status> {
        let s = tov::get_summary(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(ProtoSummary {
            enabled: s.enabled,
            current_tone_style: s.current_tone_style,
            current_personality: s.current_personality,
            current_answer_length: s.current_answer_length,
            total_guidance_rules: s.total_guidance_rules,
            enabled_guidance_rules: s.enabled_guidance_rules,
            total_presets: s.total_presets,
            recent_analytics: s.recent_analytics.into_iter().map(analytics_to_proto).collect(),
            guidance_rules: s.guidance_rules.into_iter().map(rule_to_proto).collect(),
            presets: s.presets.into_iter().map(preset_to_proto).collect(),
        }))
    }

    async fn get_config(&self, _req: Request<Empty>) -> Result<Response<ProtoConfig>, Status> {
        let c = tov::get_config(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(config_to_proto(c)))
    }

    async fn update_config(&self, req: Request<ProtoConfig>) -> Result<Response<ProtoConfigResult>, Status> {
        let config = config_from_proto(req.into_inner());
        let r = tov::update_config(&self.pool, &config).await.map_err(to_status)?;
        Ok(Response::new(ProtoConfigResult { config_id: r.config_id, message: r.message }))
    }

    async fn apply_preset(&self, req: Request<ApplyPresetRequest>) -> Result<Response<ProtoConfigResult>, Status> {
        let preset_id = req.into_inner().preset_id;
        let r = tov::apply_preset(&self.pool, &preset_id).await.map_err(to_status)?;
        Ok(Response::new(ProtoConfigResult { config_id: r.config_id, message: r.message }))
    }

    async fn list_guidance(&self, _req: Request<Empty>) -> Result<Response<GuidanceRulesResponse>, Status> {
        let rules = tov::list_guidance_rules(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(GuidanceRulesResponse { rules: rules.into_iter().map(rule_to_proto).collect() }))
    }

    async fn create_guidance(&self, req: Request<CreateGuidanceRequest>) -> Result<Response<ProtoGuidanceResult>, Status> {
        let r = req.into_inner();
        let result = tov::create_guidance_rule(&self.pool, &r.name, &r.instruction, &r.rule_type, r.priority)
            .await
            .map_err(to_status)?;
        Ok(Response::new(ProtoGuidanceResult { rule_id: result.rule_id, message: result.message }))
    }

    async fn update_guidance(&self, req: Request<ProtoGuidanceRule>) -> Result<Response<ProtoGuidanceResult>, Status> {
        let proto_rule = req.into_inner();
        let rule_id = proto_rule.id.clone();
        let rule = rule_from_proto(proto_rule);
        let result = tov::update_guidance_rule(&self.pool, &rule_id, &rule).await.map_err(to_status)?;
        Ok(Response::new(ProtoGuidanceResult { rule_id: result.rule_id, message: result.message }))
    }

    async fn delete_guidance(&self, req: Request<RuleIdRequest>) -> Result<Response<ProtoGuidanceResult>, Status> {
        let rule_id = req.into_inner().rule_id;
        let result = tov::delete_guidance_rule(&self.pool, &rule_id).await.map_err(to_status)?;
        Ok(Response::new(ProtoGuidanceResult { rule_id: result.rule_id, message: result.message }))
    }

    async fn get_prompt_additions(&self, _req: Request<Empty>) -> Result<Response<ProtoPromptAdditions>, Status> {
        let a = tov::get_prompt_additions(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(ProtoPromptAdditions {
            tone_instructions: a.tone_instructions,
            guidance_instructions: a.guidance_instructions,
            greeting_template: a.greeting_template,
            closing_template: a.closing_template,
            escalation_template: a.escalation_template,
        }))
    }

    async fn create_analytics_snapshot(
        &self,
        req: Request<CreateAnalyticsRequest>,
    ) -> Result<Response<ProtoAnalyticsResult>, Status> {
        let r = req.into_inner();
        let result = tov::create_analytics_snapshot(
            &self.pool,
            &r.tone_style,
            r.conversation_count,
            r.avg_satisfaction,
            r.avg_resolution_rate,
            r.avg_response_length,
            r.thumbs_up,
            r.thumbs_down,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(ProtoAnalyticsResult { analytics_id: result.analytics_id, message: result.message }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_tone_of_voice=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("TONE_OF_VOICE_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50070);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-tone-of-voice starting (gRPC)");
    Server::builder()
        .add_service(ToneOfVoiceServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
