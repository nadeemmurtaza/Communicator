//! Embed widget routes -- thin gRPC client for svc-embed-widget.
//! See crates/embed-widget and services/embed-widget. REST paths and
//! payload shapes are unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use newax_proto::embed_widget::v1::{
    embed_widget_service_client::EmbedWidgetServiceClient, CreateConfigRequest, Empty, GetEventsRequest, IdRequest,
    ListConfigsRequest, OrgRequest, RecordEventRequest, UpdateConfigRequest, UpdateSettingsRequest,
};
use serde::Deserialize;
use tonic::transport::Channel;

async fn client() -> Result<EmbedWidgetServiceClient<Channel>, (StatusCode, String)> {
    let url = std::env::var("EMBED_WIDGET_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50063".to_string());
    EmbedWidgetServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("embed-widget service unreachable: {e}")))
}

fn json_value(s: String) -> serde_json::Value {
    serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)
}

fn grpc_err(status: tonic::Status) -> (StatusCode, String) {
    let code = match status.code() {
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, status.message().to_string())
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
#[serde(rename_all = "camelCase")]
pub struct CreateEmbedWidgetRequest {
    pub name: String,
    pub widget_type: String,
    pub settings: Option<serde_json::Value>,
    pub styles: Option<serde_json::Value>,
    pub trigger_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWidgetEventRequest {
    pub instance_id: String,
    pub event_type: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigBody {
    pub name: Option<String>,
    pub status: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub styles: Option<serde_json::Value>,
    pub trigger_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsBody {
    pub enabled: Option<bool>,
    pub allow_custom_domains: Option<bool>,
    pub max_instances_per_domain: Option<i64>,
    pub analytics_enabled: Option<bool>,
    pub default_language: Option<String>,
}

pub async fn get_summary(State(_s): State<crate::db::AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_summary(Empty {}).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn list_configs(
    State(_s): State<crate::db::AppState>,
    Query(query): Query<ListConfigsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org_id = query.org_id.unwrap_or_else(|| "default".to_string());
    let mut c = client().await?;
    let r = c.list_configs(ListConfigsRequest { org_id, status: query.status }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_config(
    State(_s): State<crate::db::AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_config(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn create_config(
    State(_s): State<crate::db::AppState>,
    Json(req): Json<CreateEmbedWidgetRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = req.settings.unwrap_or_else(|| serde_json::json!({}));
    let styles = req.styles.unwrap_or_else(|| serde_json::json!({}));
    let trigger_config = req.trigger_config.unwrap_or_else(|| serde_json::json!({}));
    let mut c = client().await?;
    let r = c
        .create_config(CreateConfigRequest {
            org_id: "default".to_string(),
            name: req.name,
            widget_type: req.widget_type,
            settings_json: settings.to_string(),
            styles_json: styles.to_string(),
            trigger_config_json: trigger_config.to_string(),
        })
        .await
        .map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn update_config(
    State(_s): State<crate::db::AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateConfigBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut updates = serde_json::json!({});
    if let Some(name) = req.name { updates["name"] = serde_json::json!(name); }
    if let Some(status) = req.status { updates["status"] = serde_json::json!(status); }
    if let Some(settings) = req.settings { updates["settings"] = settings; }
    if let Some(styles) = req.styles { updates["styles"] = styles; }
    if let Some(trigger_config) = req.trigger_config { updates["trigger_config"] = trigger_config; }

    let mut c = client().await?;
    let r = c.update_config(UpdateConfigRequest { id, updates_json: updates.to_string() }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn delete_config(
    State(_s): State<crate::db::AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client().await?;
    c.delete_config(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_settings(State(_s): State<crate::db::AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_settings(OrgRequest { org_id: "default".to_string() }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn update_settings(
    State(_s): State<crate::db::AppState>,
    Json(req): Json<UpdateSettingsBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut updates = serde_json::json!({});
    if let Some(v) = req.enabled { updates["enabled"] = serde_json::json!(v); }
    if let Some(v) = req.allow_custom_domains { updates["allow_custom_domains"] = serde_json::json!(v); }
    if let Some(v) = req.max_instances_per_domain { updates["max_instances_per_domain"] = serde_json::json!(v); }
    if let Some(v) = req.analytics_enabled { updates["analytics_enabled"] = serde_json::json!(v); }
    if let Some(v) = req.default_language { updates["default_language"] = serde_json::json!(v); }

    let mut c = client().await?;
    let r = c
        .update_settings(UpdateSettingsRequest { org_id: "default".to_string(), updates_json: updates.to_string() })
        .await
        .map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn record_event(
    State(_s): State<crate::db::AppState>,
    Json(req): Json<RecordWidgetEventRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload = req.payload.unwrap_or_else(|| serde_json::json!({}));
    let mut c = client().await?;
    let r = c
        .record_event(RecordEventRequest {
            instance_id: req.instance_id,
            event_type: req.event_type,
            payload_json: payload.to_string(),
        })
        .await
        .map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_events(
    State(_s): State<crate::db::AppState>,
    Query(query): Query<GetEventsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c
        .get_events(GetEventsRequest {
            instance_id: query.instance_id,
            event_type: query.event_type,
            limit: query.limit.unwrap_or(50),
        })
        .await
        .map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_public_widget_data(
    State(_s): State<crate::db::AppState>,
    Path(config_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_public_widget_data(IdRequest { id: config_id }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}

pub async fn get_embed_script(
    State(_s): State<crate::db::AppState>,
    Path(config_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;
    let r = c.get_embed_script(IdRequest { id: config_id }).await.map_err(grpc_err)?;
    Ok(Json(json_value(r.into_inner().json)))
}
