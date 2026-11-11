use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::models::{CreateEmbedWidgetRequest, RecordWidgetEventRequest};
use crate::services::embed_widget::EmbedWidgetService;

#[derive(Debug, Serialize)]
pub struct EmbedWidgetState {
    pub embed_widget: EmbedWidgetService,
}

#[derive(Debug, Deserialize)]
pub struct ListConfigsQuery {
    pub org_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetEventsQuery {
    pub instance_id: Option<String>,
    pub event_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub styles: Option<serde_json::Value>,
    pub trigger_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub enabled: Option<bool>,
    pub allow_custom_domains: Option<bool>,
    pub max_instances_per_domain: Option<i64>,
    pub analytics_enabled: Option<bool>,
    pub default_language: Option<String>,
}

// GET /embed-widget/summary
pub async fn get_summary(State(state): State<EmbedWidgetState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let summary = state.embed_widget.get_summary().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(summary).unwrap()))
}

// GET /embed-widget/configs
pub async fn list_configs(
    State(state): State<EmbedWidgetState>,
    Query(query): Query<ListConfigsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org_id = query.org_id.unwrap_or_else(|| "default".to_string());
    let configs = state.embed_widget.list_configs(&org_id, query.status.as_deref())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(configs).unwrap()))
}

// GET /embed-widget/configs/:id
pub async fn get_config(
    State(state): State<EmbedWidgetState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let config = state.embed_widget.get_config(&id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match config {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        None => Err((StatusCode::NOT_FOUND, "Config not found".to_string())),
    }
}

// POST /embed-widget/configs
pub async fn create_config(
    State(state): State<EmbedWidgetState>,
    Json(req): Json<CreateEmbedWidgetRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org_id = "default".to_string();
    let settings = req.settings.unwrap_or_else(|| serde_json::json!({}));
    let styles = req.styles.unwrap_or_else(|| serde_json::json!({}));
    let trigger_config = req.trigger_config.unwrap_or_else(|| serde_json::json!({}));

    let result = state.embed_widget.create_config(
        &org_id, &req.name, &req.widget_type, &settings, &styles, &trigger_config
    ).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

// PUT /embed-widget/configs/:id
pub async fn update_config(
    State(state): State<EmbedWidgetState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut updates = serde_json::json!({});
    if let Some(name) = req.name {
        updates["name"] = serde_json::json!(name);
    }
    if let Some(status) = req.status {
        updates["status"] = serde_json::json!(status);
    }
    if let Some(settings) = req.settings {
        updates["settings"] = settings;
    }
    if let Some(styles) = req.styles {
        updates["styles"] = styles;
    }
    if let Some(trigger_config) = req.trigger_config {
        updates["trigger_config"] = trigger_config;
    }

    let config = state.embed_widget.update_config(&id, &updates)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(config).unwrap()))
}

// DELETE /embed-widget/configs/:id
pub async fn delete_config(
    State(state): State<EmbedWidgetState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.embed_widget.delete_config(&id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /embed-widget/settings
pub async fn get_settings(
    State(state): State<EmbedWidgetState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org_id = "default".to_string();
    let settings = state.embed_widget.get_settings(&org_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap()))
}

// PUT /embed-widget/settings
pub async fn update_settings(
    State(state): State<EmbedWidgetState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org_id = "default".to_string();
    let mut updates = serde_json::json!({});
    if let Some(enabled) = req.enabled {
        updates["enabled"] = serde_json::json!(enabled);
    }
    if let Some(allow_custom_domains) = req.allow_custom_domains {
        updates["allow_custom_domains"] = serde_json::json!(allow_custom_domains);
    }
    if let Some(max_instances) = req.max_instances_per_domain {
        updates["max_instances_per_domain"] = serde_json::json!(max_instances);
    }
    if let Some(analytics_enabled) = req.analytics_enabled {
        updates["analytics_enabled"] = serde_json::json!(analytics_enabled);
    }
    if let Some(default_language) = req.default_language {
        updates["default_language"] = serde_json::json!(default_language);
    }

    let settings = state.embed_widget.update_settings(&org_id, &updates)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap()))
}

// POST /embed-widget/events
pub async fn record_event(
    State(state): State<EmbedWidgetState>,
    Json(req): Json<RecordWidgetEventRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload = req.payload.unwrap_or_else(|| serde_json::json!({}));
    let event_id = state.embed_widget.record_event(&req.instance_id, &req.event_type, &payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "event_id": event_id })))
}

// GET /embed-widget/events
pub async fn get_events(
    State(state): State<EmbedWidgetState>,
    Query(query): Query<GetEventsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50);
    let events = state.embed_widget.get_events(
        query.instance_id.as_deref(),
        query.event_type.as_deref(),
        limit,
    ).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(events).unwrap()))
}

// GET /embed-widget/public/:config_id
pub async fn get_public_widget_data(
    State(state): State<EmbedWidgetState>,
    Path(config_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let data = state.embed_widget.get_public_widget_data(&config_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(data).unwrap()))
}

// GET /embed-widget/script/:config_id
pub async fn get_embed_script(
    State(state): State<EmbedWidgetState>,
    Path(config_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let script = state.embed_widget.generate_embed_script(&config_id);
    Ok(Json(serde_json::json!({ "script": script, "config_id": config_id })))
}
