//! Thin gRPC client forwarding brand override requests to `svc-brand-overrides` (port 50090).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::brand_overrides::v1::{
    brand_overrides_service_client::BrandOverridesServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("BRAND_OVERRIDES_GRPC").unwrap_or("http://127.0.0.1:50090")
}

async fn client() -> BrandOverridesServiceClient<tonic::transport::Channel> {
    BrandOverridesServiceClient::connect(svc_addr().to_string())
        .await
        .expect("brand-overrides gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn err_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

#[derive(Deserialize, Debug)]
pub struct ListRulesQuery {
    pub organization_id: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub to_brand_id: Option<String>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct UpdateSettingsRequest {
    pub enabled: Option<bool>,
    pub allow_cross_org_brands: Option<bool>,
    pub require_approval: Option<bool>,
    pub log_all_switches: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct AddHierarchyRequest {
    pub parent_brand_id: String,
    pub sub_brand_id: String,
    pub relationship_type: Option<String>,
    pub priority: Option<i32>,
}

pub async fn summary(State(_state): State<AppState>) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_rules(
    State(_state): State<AppState>,
    Query(params): Query<ListRulesQuery>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::ListRulesRequest::default();
    req.organization_id = params.organization_id.clone().unwrap_or_default();
    req.has_is_active = params.is_active.is_some();
    req.is_active = params.is_active.unwrap_or_default();
    req.limit = params.limit.unwrap_or(50);
    req.offset = params.offset.unwrap_or(0);
    match client().await.list_rules(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn create_rule(
    State(_state): State<AppState>,
    Json(body): Json<CreateRuleRequest>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::CreateRuleRequest::default();
    req.organization_id = body.organization_id;
    req.name = body.name;
    req.trigger_type = body.trigger_type.unwrap_or_else(|| "workflow".to_string());
    req.from_brand_id = body.from_brand_id.unwrap_or_default();
    req.to_brand_id = body.to_brand_id;
    req.auto_switch_kb = body.auto_switch_kb.unwrap_or(true);
    req.auto_switch_ai_config = body.auto_switch_ai_config.unwrap_or(true);
    req.auto_switch_messenger = body.auto_switch_messenger.unwrap_or(true);
    match client().await.create_rule(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn update_rule(
    State(_state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(body): Json<UpdateRuleRequest>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::UpdateRuleRequest::default();
    req.rule_id = rule_id;
    req.name = body.name.unwrap_or_default();
    req.has_is_active = body.is_active.is_some();
    req.is_active = body.is_active.unwrap_or_default();
    req.to_brand_id = body.to_brand_id.unwrap_or_default();
    match client().await.update_rule(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn delete_rule(
    State(_state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::DeleteRuleRequest::default();
    req.rule_id = rule_id;
    match client().await.delete_rule(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn switch_brand(
    State(_state): State<AppState>,
    Json(body): Json<SwitchBrandRequest>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::SwitchBrandRequest::default();
    req.conversation_id = body.conversation_id;
    req.contact_id = body.contact_id;
    req.from_brand_id = body.from_brand_id;
    req.to_brand_id = body.to_brand_id;
    req.trigger_type = body.trigger_type.unwrap_or_else(|| "manual".to_string());
    req.trigger_source = body.trigger_source.unwrap_or_default();
    req.rule_id = body.rule_id.unwrap_or_default();
    req.switched_by = body.switched_by.unwrap_or_else(|| "system".to_string());
    match client().await.switch_brand(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn switch_log(
    State(_state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::GetSwitchLogRequest::default();
    req.conversation_id = conversation_id;
    match client().await.get_switch_log(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_settings(State(_state): State<AppState>) -> Response {
    match client().await.get_settings(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Json(body): Json<UpdateSettingsRequest>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::UpdateSettingsRequest::default();
    req.has_enabled = body.enabled.is_some();
    req.enabled = body.enabled.unwrap_or_default();
    req.has_allow_cross_org_brands = body.allow_cross_org_brands.is_some();
    req.allow_cross_org_brands = body.allow_cross_org_brands.unwrap_or_default();
    req.has_require_approval = body.require_approval.is_some();
    req.require_approval = body.require_approval.unwrap_or_default();
    req.has_log_all_switches = body.log_all_switches.is_some();
    req.log_all_switches = body.log_all_switches.unwrap_or_default();
    match client().await.update_settings(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn list_hierarchy(State(_state): State<AppState>) -> Response {
    match client().await.list_hierarchy(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn add_hierarchy(
    State(_state): State<AppState>,
    Json(body): Json<AddHierarchyRequest>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::AddHierarchyRequest::default();
    req.parent_brand_id = body.parent_brand_id;
    req.sub_brand_id = body.sub_brand_id;
    req.relationship_type = body.relationship_type.unwrap_or_else(|| "sub_brand".to_string());
    req.priority = body.priority.unwrap_or(0);
    match client().await.add_hierarchy(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn remove_hierarchy(
    State(_state): State<AppState>,
    Json(body): Json<AddHierarchyRequest>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::RemoveHierarchyRequest::default();
    req.parent_brand_id = body.parent_brand_id;
    req.sub_brand_id = body.sub_brand_id;
    match client().await.remove_hierarchy(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn find_matching_rules(
    State(_state): State<AppState>,
    Query(params): Query<ListRulesQuery>,
) -> Response {
    let from_brand = params.organization_id.clone().unwrap_or_default();
    let org_id = params.organization_id.unwrap_or_else(|| "org-default".to_string());
    let mut req = newax_proto::brand_overrides::v1::FindMatchingRulesRequest::default();
    req.from_brand_id = from_brand;
    req.organization_id = org_id;
    match client().await.find_matching_rules(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_sub_brands(
    State(_state): State<AppState>,
    Path(parent_brand_id): Path<String>,
) -> Response {
    let mut req = newax_proto::brand_overrides::v1::GetSubBrandsRequest::default();
    req.parent_brand_id = parent_brand_id;
    match client().await.get_sub_brands(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}
