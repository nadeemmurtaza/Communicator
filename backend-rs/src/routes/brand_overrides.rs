//! Brand Overrides API Routes — Phase 27

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db::AppState;
use crate::services::brand_overrides;

// ——— Request/Response Types ———

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub organization_id: String,
    pub name: String,
    pub trigger_type: Option<String>,
    pub from_brand_id: Option<String>,
    pub to_brand_id: String,
    pub auto_switch_kb: Option<bool>,
    pub auto_switch_ai_config: Option<bool>,
    pub auto_switch_messenger: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub to_brand_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SwitchBrandRequest {
    pub conversation_id: String,
    pub contact_id: String,
    pub from_brand_id: String,
    pub to_brand_id: String,
    pub trigger_type: Option<String>,
    pub trigger_source: Option<String>,
    pub rule_id: Option<String>,
    pub switched_by: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub enabled: Option<bool>,
    pub allow_cross_org_brands: Option<bool>,
    pub require_approval: Option<bool>,
    pub log_all_switches: Option<bool>,
}

#[derive(Deserialize)]
pub struct AddHierarchyRequest {
    pub parent_brand_id: String,
    pub sub_brand_id: String,
    pub relationship_type: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Deserialize)]
pub struct ListRulesQuery {
    pub organization_id: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ——— Handlers ———

/// GET /brand-overrides/summary
pub async fn summary(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let summary = brand_overrides::get_brand_override_summary(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

/// GET /brand-overrides/rules
pub async fn list_rules(
    State(state): State<AppState>,
    Query(params): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rules = brand_overrides::list_brand_override_rules(
        &state,
        params.organization_id.as_deref(),
        params.is_active,
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(rules).unwrap_or_default()))
}

/// POST /brand-overrides/rules
pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let result = brand_overrides::create_brand_override_rule(
        &state,
        &req.organization_id,
        &req.name,
        req.trigger_type.as_deref().unwrap_or("workflow"),
        req.from_brand_id.as_deref(),
        &req.to_brand_id,
        req.auto_switch_kb.unwrap_or(true),
        req.auto_switch_ai_config.unwrap_or(true),
        req.auto_switch_messenger.unwrap_or(true),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

/// PUT /brand-overrides/rules/{rule_id}
pub async fn update_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateRuleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    brand_overrides::update_brand_override_rule(
        &state,
        &rule_id,
        req.name.as_deref(),
        req.is_active,
        req.to_brand_id.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({"message": "Rule updated"})))
}

/// DELETE /brand-overrides/rules/{rule_id}
pub async fn delete_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    brand_overrides::delete_brand_override_rule(&state, &rule_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({"message": "Rule deleted"})))
}

/// POST /brand-overrides/switch
pub async fn switch_brand(
    State(state): State<AppState>,
    Json(req): Json<SwitchBrandRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = brand_overrides::switch_brand(
        &state,
        &req.conversation_id,
        &req.contact_id,
        &req.from_brand_id,
        &req.to_brand_id,
        req.trigger_type.as_deref().unwrap_or("manual"),
        req.trigger_source.as_deref().unwrap_or(""),
        req.rule_id.as_deref(),
        req.switched_by.as_deref().unwrap_or("system"),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /brand-overrides/switch-log/{conversation_id}
pub async fn switch_log(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let log = brand_overrides::get_switch_log_for_conversation(&state, &conversation_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(log).unwrap_or_default()))
}

/// GET /brand-overrides/settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = brand_overrides::get_brand_override_settings(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap_or_default()))
}

/// PUT /brand-overrides/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = brand_overrides::update_brand_override_settings(
        &state,
        req.enabled,
        req.allow_cross_org_brands,
        req.require_approval,
        req.log_all_switches,
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(settings).unwrap_or_default()))
}

/// GET /brand-overrides/hierarchy
pub async fn list_hierarchy(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let hierarchy = brand_overrides::list_brand_hierarchy(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(hierarchy).unwrap_or_default()))
}

/// POST /brand-overrides/hierarchy
pub async fn add_hierarchy(
    State(state): State<AppState>,
    Json(req): Json<AddHierarchyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    brand_overrides::add_brand_hierarchy(
        &state,
        &req.parent_brand_id,
        &req.sub_brand_id,
        req.relationship_type.as_deref().unwrap_or("sub_brand"),
        req.priority.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({"message": "Hierarchy added"})))
}

/// DELETE /brand-overrides/hierarchy
pub async fn remove_hierarchy(
    State(state): State<AppState>,
    Json(req): Json<AddHierarchyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    brand_overrides::remove_brand_hierarchy(&state, &req.parent_brand_id, &req.sub_brand_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({"message": "Hierarchy removed"})))
}

/// GET /brand-overrides/matching
pub async fn find_matching_rules(
    State(state): State<AppState>,
    Query(params): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let from_brand = params
        .organization_id
        .clone()
        .unwrap_or_default();
    let org_id = params.organization_id.unwrap_or_else(|| "org-default".to_string());

    let rules = brand_overrides::find_matching_rules(&state, &from_brand, &org_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(rules).unwrap_or_default()))
}

/// GET /brand-overrides/sub-brands/{parent_brand_id}
pub async fn get_sub_brands(
    State(state): State<AppState>,
    Path(parent_brand_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let sub_brands = brand_overrides::get_sub_brands(&state, &parent_brand_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(sub_brands).unwrap_or_default()))
}
