//! WhatsApp Flows API routes — thin gRPC client forwarding to svc-whatsapp-flows.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::db::AppState;
use newax_proto::whatsapp_flows::v1::whats_app_flows_service_client::WhatsAppFlowsServiceClient;

async fn client(
    state: &AppState,
) -> Result<WhatsAppFlowsServiceClient<tonic::transport::Channel>, (StatusCode, Json<serde_json::Value>)> {
    let url = "http://127.0.0.1:50100".to_string();
    WhatsAppFlowsServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("whatsapp_flows gRPC connect failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "whatsapp_flows service unavailable" })),
            )
        })
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TemplateGroupsQuery {
    pub industry_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FlowsQuery {
    pub industry_id: Option<String>,
    pub group_id: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateIndustryRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIndustryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateGroupRequest {
    pub industry_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlowRequest {
    pub group_id: Option<String>,
    pub industry_id: Option<String>,
    pub name: String,
    pub flow_type: Option<String>,
    pub category: Option<String>,
    pub body: String,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub buttons: Option<String>,
    pub sections: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlowRequest {
    pub name: Option<String>,
    pub body: Option<String>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub buttons: Option<String>,
    pub sections: Option<String>,
    pub status: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DuplicateFlowRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlowStepRequest {
    pub step_number: i32,
    pub condition: Option<String>,
    pub action_type: String,
    pub action_config: Option<String>,
    pub next_step_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlowStepRequest {
    pub step_number: Option<i32>,
    pub condition: Option<String>,
    pub action_type: Option<String>,
    pub action_config: Option<String>,
    pub next_step_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Industries
// ---------------------------------------------------------------------------

pub async fn list_industries(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_industries(newax_proto::whatsapp_flows::v1::ListIndustriesRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn create_industry(
    State(state): State<AppState>,
    Json(body): Json<CreateIndustryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.create_industry(newax_proto::whatsapp_flows::v1::CreateIndustryRequest {
        name: body.name,
        description: body.description.unwrap_or_default(),
        icon: body.icon.clone().unwrap_or_default(),
        has_icon: body.icon.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn update_industry(
    State(state): State<AppState>,
    Path(industry_id): Path<String>,
    Json(body): Json<UpdateIndustryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.update_industry(newax_proto::whatsapp_flows::v1::UpdateIndustryRequest {
        industry_id,
        name: body.name.clone().unwrap_or_default(),
        has_name: body.name.is_some(),
        description: body.description.clone().unwrap_or_default(),
        has_description: body.description.is_some(),
        icon: body.icon.clone().unwrap_or_default(),
        has_icon: body.icon.is_some(),
        is_active: if body.is_active.is_some() { body.is_active.unwrap_or(false).to_string() } else { String::new() },
        has_is_active: body.is_active.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_industry(
    State(state): State<AppState>,
    Path(industry_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.delete_industry(newax_proto::whatsapp_flows::v1::DeleteIndustryRequest { industry_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

// ---------------------------------------------------------------------------
// Template Groups
// ---------------------------------------------------------------------------

pub async fn list_template_groups(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Query(params): Query<TemplateGroupsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_template_groups(newax_proto::whatsapp_flows::v1::ListTemplateGroupsRequest {
        brand_id,
        industry_id: params.industry_id.clone().unwrap_or_default(),
        has_industry_id: params.industry_id.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn create_template_group(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Json(body): Json<CreateTemplateGroupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.create_template_group(newax_proto::whatsapp_flows::v1::CreateTemplateGroupRequest {
        brand_id,
        industry_id: body.industry_id.clone().unwrap_or_default(),
        has_industry_id: body.industry_id.is_some(),
        name: body.name,
        description: body.description.unwrap_or_default(),
        sort_order: body.sort_order.unwrap_or(0),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn update_template_group(
    State(state): State<AppState>,
    Path((brand_id, group_id)): Path<(String, String)>,
    Json(body): Json<UpdateTemplateGroupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _brand_id = brand_id;
    let mut cli = client(&state).await?;
    let resp = cli.update_template_group(newax_proto::whatsapp_flows::v1::UpdateTemplateGroupRequest {
        group_id,
        name: body.name.clone().unwrap_or_default(),
        has_name: body.name.is_some(),
        description: body.description.clone().unwrap_or_default(),
        has_description: body.description.is_some(),
        sort_order: if body.sort_order.is_some() { body.sort_order.unwrap_or(0).to_string() } else { String::new() },
        has_sort_order: body.sort_order.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_template_group(
    State(state): State<AppState>,
    Path((brand_id, group_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _brand_id = brand_id;
    let mut cli = client(&state).await?;
    let resp = cli.delete_template_group(newax_proto::whatsapp_flows::v1::DeleteTemplateGroupRequest { group_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

// ---------------------------------------------------------------------------
// Template Flows
// ---------------------------------------------------------------------------

pub async fn list_flows(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Query(params): Query<FlowsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_flows(newax_proto::whatsapp_flows::v1::ListFlowsRequest {
        brand_id,
        industry_id: params.industry_id.clone().unwrap_or_default(),
        has_industry_id: params.industry_id.is_some(),
        group_id: params.group_id.clone().unwrap_or_default(),
        has_group_id: params.group_id.is_some(),
        category: params.category.clone().unwrap_or_default(),
        has_category: params.category.is_some(),
        status: params.status.clone().unwrap_or_default(),
        has_status: params.status.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn create_flow(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    Json(body): Json<CreateFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.create_flow(newax_proto::whatsapp_flows::v1::CreateFlowRequest {
        brand_id,
        group_id: body.group_id.clone().unwrap_or_default(),
        has_group_id: body.group_id.is_some(),
        industry_id: body.industry_id.clone().unwrap_or_default(),
        has_industry_id: body.industry_id.is_some(),
        name: body.name,
        flow_type: body.flow_type.unwrap_or_else(|| "template".to_string()),
        category: body.category.unwrap_or_else(|| "general".to_string()),
        body: body.body,
        header: body.header.clone().unwrap_or_default(),
        has_header: body.header.is_some(),
        footer: body.footer.clone().unwrap_or_default(),
        has_footer: body.footer.is_some(),
        buttons: body.buttons.clone().unwrap_or_default(),
        has_buttons: body.buttons.is_some(),
        sections: body.sections.clone().unwrap_or_default(),
        has_sections: body.sections.is_some(),
        language: body.language.unwrap_or_else(|| "en_US".to_string()),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn get_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.get_flow(newax_proto::whatsapp_flows::v1::GetFlowRequest { flow_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn update_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<UpdateFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.update_flow(newax_proto::whatsapp_flows::v1::UpdateFlowRequest {
        flow_id,
        name: body.name.clone().unwrap_or_default(),
        has_name: body.name.is_some(),
        body: body.body.clone().unwrap_or_default(),
        has_body: body.body.is_some(),
        header: body.header.clone().unwrap_or_default(),
        has_header: body.header.is_some(),
        footer: body.footer.clone().unwrap_or_default(),
        has_footer: body.footer.is_some(),
        buttons: body.buttons.clone().unwrap_or_default(),
        has_buttons: body.buttons.is_some(),
        sections: body.sections.clone().unwrap_or_default(),
        has_sections: body.sections.is_some(),
        status: body.status.clone().unwrap_or_default(),
        has_status: body.status.is_some(),
        is_active: if body.is_active.is_some() { body.is_active.unwrap_or(false).to_string() } else { String::new() },
        has_is_active: body.is_active.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.delete_flow(newax_proto::whatsapp_flows::v1::DeleteFlowRequest { flow_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn duplicate_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<DuplicateFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.duplicate_flow(newax_proto::whatsapp_flows::v1::DuplicateFlowRequest {
        flow_id,
        name: body.name,
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn validate_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.validate_flow(newax_proto::whatsapp_flows::v1::ValidateFlowRequest { flow_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

// ---------------------------------------------------------------------------
// Flow Steps
// ---------------------------------------------------------------------------

pub async fn list_flow_steps(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_flow_steps(newax_proto::whatsapp_flows::v1::ListFlowStepsRequest { flow_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn create_flow_step(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<CreateFlowStepRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.create_flow_step(newax_proto::whatsapp_flows::v1::CreateFlowStepRequest {
        flow_id,
        step_number: body.step_number,
        condition: body.condition.clone().unwrap_or_default(),
        has_condition: body.condition.is_some(),
        action_type: body.action_type,
        action_config: body.action_config.clone().unwrap_or_default(),
        has_action_config: body.action_config.is_some(),
        next_step_id: body.next_step_id.clone().unwrap_or_default(),
        has_next_step_id: body.next_step_id.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn update_flow_step(
    State(state): State<AppState>,
    Path((flow_id, step_id)): Path<(String, String)>,
    Json(body): Json<UpdateFlowStepRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _flow_id = flow_id;
    let mut cli = client(&state).await?;
    let resp = cli.update_flow_step(newax_proto::whatsapp_flows::v1::UpdateFlowStepRequest {
        step_id,
        step_number: if body.step_number.is_some() { body.step_number.unwrap_or(0).to_string() } else { String::new() },
        has_step_number: body.step_number.is_some(),
        condition: body.condition.clone().unwrap_or_default(),
        has_condition: body.condition.is_some(),
        action_type: body.action_type.clone().unwrap_or_default(),
        has_action_type: body.action_type.is_some(),
        action_config: body.action_config.clone().unwrap_or_default(),
        has_action_config: body.action_config.is_some(),
        next_step_id: body.next_step_id.clone().unwrap_or_default(),
        has_next_step_id: body.next_step_id.is_some(),
    }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_flow_step(
    State(state): State<AppState>,
    Path((flow_id, step_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _flow_id = flow_id;
    let mut cli = client(&state).await?;
    let resp = cli.delete_flow_step(newax_proto::whatsapp_flows::v1::DeleteFlowStepRequest { step_id }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value =
                serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            Ok(Json(val))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

// ---------------------------------------------------------------------------
// Router — mount these routes in main.rs or the parent router
// ---------------------------------------------------------------------------

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        // Industries
        .route("/brands/{brand_id}/whatsapp/industries", axum::routing::get(list_industries))
        .route("/brands/{brand_id}/whatsapp/industries", axum::routing::post(create_industry))
        .route("/brands/{brand_id}/whatsapp/industries/{industry_id}", axum::routing::put(update_industry))
        .route("/brands/{brand_id}/whatsapp/industries/{industry_id}", axum::routing::delete(delete_industry))
        // Template Groups
        .route("/brands/{brand_id}/whatsapp/groups", axum::routing::get(list_template_groups))
        .route("/brands/{brand_id}/whatsapp/groups", axum::routing::post(create_template_group))
        .route("/brands/{brand_id}/whatsapp/groups/{group_id}", axum::routing::put(update_template_group))
        .route("/brands/{brand_id}/whatsapp/groups/{group_id}", axum::routing::delete(delete_template_group))
        // Template Flows
        .route("/brands/{brand_id}/whatsapp/flows", axum::routing::get(list_flows))
        .route("/brands/{brand_id}/whatsapp/flows", axum::routing::post(create_flow))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}", axum::routing::get(get_flow))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}", axum::routing::put(update_flow))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}", axum::routing::delete(delete_flow))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}/duplicate", axum::routing::post(duplicate_flow))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}/validate", axum::routing::post(validate_flow))
        // Flow Steps
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}/steps", axum::routing::get(list_flow_steps))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}/steps", axum::routing::post(create_flow_step))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}/steps/{step_id}", axum::routing::put(update_flow_step))
        .route("/brands/{brand_id}/whatsapp/flows/{flow_id}/steps/{step_id}", axum::routing::delete(delete_flow_step))
}
