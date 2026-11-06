#![allow(dead_code)]

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::AppState;

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub timezone: Option<String>,
    pub settings: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateBrandRequest {
    pub organization_id: String,
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub tagline: Option<String>,
    pub color_scheme: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub social_links: Option<String>,
    pub products: Option<String>,
    pub services: Option<String>,
    pub prices: Option<String>,
    pub policies: Option<String>,
    pub rules: Option<String>,
    pub tone_guidelines: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBrandRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub tagline: Option<String>,
    pub color_scheme: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub social_links: Option<String>,
    pub products: Option<String>,
    pub services: Option<String>,
    pub prices: Option<String>,
    pub policies: Option<String>,
    pub rules: Option<String>,
    pub tone_guidelines: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Deserialize)]
pub struct UpsertCredentialsRequest {
    pub channel_type: String,
    pub credentials: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct UpsertAIConfigRequest {
    pub ai_provider: Option<String>,
    pub api_key: Option<String>,
    pub model_id: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i64>,
    pub system_prompt: Option<String>,
    pub rag_enabled: Option<bool>,
    pub embedding_provider: Option<String>,
    pub embedding_api_key: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateKnowledgeEntryRequest {
    pub title: String,
    pub content: String,
    pub source_type: Option<String>,
    pub topic: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateKnowledgeEntryRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub source_type: Option<String>,
    pub topic: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchKnowledgeQuery {
    pub q: Option<String>,
    pub topic: Option<String>,
}

#[derive(Serialize)]
pub struct BrandProfileResponse {
    pub brand: serde_json::Value,
    pub organization: serde_json::Value,
    pub ai_config: Option<serde_json::Value>,
    pub knowledge_count: i64,
}

// ---------------------------------------------------------------------------
// Enhanced profile request/response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct UpsertProfileSourceRequest {
    pub field_name: String,
    pub source_platform: String,
    pub source_user_id: Option<String>,
    pub source_user_name: Option<String>,
    pub source_record_id: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpsertSyncStatusRequest {
    pub platform: String,
    pub is_connected: Option<bool>,
    pub sync_interval_seconds: Option<i64>,
}

#[derive(Deserialize)]
pub struct ResolveConflictRequest {
    pub resolved_value: String,
    pub resolved_by: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCustomSectionRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub section_name: String,
    pub section_type: String,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpsertSectionDataRequest {
    pub row_key: Option<String>,
    pub field_key: String,
    pub field_value: String,
    pub field_type: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreatePackageRequest {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub currency: Option<String>,
    pub duration_type: Option<String>,
    pub duration_value: Option<i64>,
    pub features: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdatePackageRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub duration_type: Option<String>,
    pub duration_value: Option<i64>,
    pub features: Option<String>,
    pub is_active: Option<bool>,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreatePromotionRequest {
    pub name: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: f64,
    pub start_date: String,
    pub end_date: String,
    pub applicable_packages: Option<String>,
    pub usage_limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdatePromotionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub applicable_packages: Option<String>,
    pub is_active: Option<bool>,
    pub usage_limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpsertVoucherSettingsRequest {
    pub auto_generate: Option<bool>,
    pub voucher_prefix: Option<String>,
    pub voucher_length: Option<i64>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub validity_days: Option<i64>,
    pub trigger_event: Option<String>,
    pub max_uses_per_customer: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateMessageTriggerRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_event: Option<String>,
    pub trigger_schedule: Option<String>,
    pub trigger_conditions: Option<String>,
    pub preferred_channel: Option<String>,
    pub message_template: String,
}

#[derive(Deserialize)]
pub struct UpdateMessageTriggerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_event: Option<String>,
    pub trigger_schedule: Option<String>,
    pub trigger_conditions: Option<String>,
    pub preferred_channel: Option<String>,
    pub message_template: Option<String>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        // Organizations
        .route("/brands/organizations", get(list_organizations))
        .route("/brands/organizations", post(create_organization))
        .route("/brands/organizations/{id}", get(get_organization))
        .route("/brands/organizations/{id}", put(update_organization))
        .route("/brands/organizations/{id}", delete(delete_organization))
        // Brands
        .route("/brands", get(list_brands))
        .route("/brands", post(create_brand))
        .route("/brands/{id}", get(get_brand))
        .route("/brands/{id}", put(update_brand))
        .route("/brands/{id}", delete(delete_brand))
        .route("/brands/{id}/profile", get(get_brand_profile))
        // Brand credentials
        .route("/brands/{id}/credentials", get(list_credentials))
        .route("/brands/{id}/credentials/{channel}", put(upsert_credentials))
        .route("/brands/{id}/credentials/{channel}", delete(delete_credentials))
        .route("/brands/{id}/credentials/{channel}/verify", post(verify_credentials))
        // Brand AI config
        .route("/brands/{id}/ai-config", get(get_ai_config))
        .route("/brands/{id}/ai-config", put(upsert_ai_config))
        // Brand knowledge base
        .route("/brands/{id}/knowledge", get(list_knowledge_entries))
        .route("/brands/{id}/knowledge", post(create_knowledge_entry))
        .route("/brands/{id}/knowledge/{entry_id}", get(get_knowledge_entry))
        .route("/brands/{id}/knowledge/{entry_id}", put(update_knowledge_entry))
        .route("/brands/{id}/knowledge/{entry_id}", delete(delete_knowledge_entry))
        // ===== Enhanced Profile: Data Sources =====
        .route("/brands/{id}/sources", get(list_profile_sources))
        .route("/brands/{id}/sources", post(upsert_profile_source))
        .route("/brands/{id}/sources/{source_id}", delete(delete_profile_source))
        // ===== Enhanced Profile: Sync Status =====
        .route("/brands/{id}/sync-status", get(list_sync_status))
        .route("/brands/{id}/sync-status", post(upsert_sync_status))
        .route("/brands/{id}/sync/{platform}", post(sync_now))
        // ===== Enhanced Profile: Conflicts =====
        .route("/brands/{id}/conflicts", get(list_conflicts))
        .route("/brands/{id}/conflicts/{conflict_id}/resolve", post(resolve_conflict))
        // ===== Enhanced Profile: Custom Sections =====
        .route("/brands/{id}/sections", get(list_custom_sections))
        .route("/brands/{id}/sections", post(create_custom_section))
        .route("/brands/{id}/sections/{section_id}", get(get_custom_section))
        .route("/brands/{id}/sections/{section_id}", put(update_custom_section))
        .route("/brands/{id}/sections/{section_id}", delete(delete_custom_section))
        .route("/brands/{id}/sections/{section_id}/data", get(get_section_data))
        .route("/brands/{id}/sections/{section_id}/data", post(upsert_section_data))
        .route("/brands/{id}/sections/{section_id}/data/{data_id}", delete(delete_section_data))
        // ===== Enhanced Profile: Packages =====
        .route("/brands/{id}/packages", get(list_packages))
        .route("/brands/{id}/packages", post(create_package))
        .route("/brands/{id}/packages/{package_id}", get(get_package))
        .route("/brands/{id}/packages/{package_id}", put(update_package))
        .route("/brands/{id}/packages/{package_id}", delete(delete_package))
        // ===== Enhanced Profile: Promotions =====
        .route("/brands/{id}/promotions", get(list_promotions))
        .route("/brands/{id}/promotions", post(create_promotion))
        .route("/brands/{id}/promotions/{promo_id}", get(get_promotion))
        .route("/brands/{id}/promotions/{promo_id}", put(update_promotion))
        .route("/brands/{id}/promotions/{promo_id}", delete(delete_promotion))
        // ===== Enhanced Profile: Voucher Settings =====
        .route("/brands/{id}/voucher-settings", get(get_voucher_settings))
        .route("/brands/{id}/voucher-settings", put(upsert_voucher_settings))
        // ===== Enhanced Profile: Message Triggers =====
        .route("/brands/{id}/triggers", get(list_message_triggers))
        .route("/brands/{id}/triggers", post(create_message_trigger))
        .route("/brands/{id}/triggers/{trigger_id}", get(get_message_trigger))
        .route("/brands/{id}/triggers/{trigger_id}", put(update_message_trigger))
        .route("/brands/{id}/triggers/{trigger_id}", delete(delete_message_trigger))
        .route("/brands/{id}/triggers/{trigger_id}/logs", get(get_trigger_logs))
        // ===== Enhanced Profile: Platform Data Cache =====
        .route("/brands/{id}/cache", get(list_platform_cache))
        .route("/brands/{id}/cache/{platform}", get(get_platform_cache))
        // ===== Organization Enhanced Profile =====
        .route("/brands/organizations/{id}/sources", get(list_org_profile_sources))
        .route("/brands/organizations/{id}/sources", post(upsert_org_profile_source))
        .route("/brands/organizations/{id}/sync-status", get(list_org_sync_status))
        .route("/brands/organizations/{id}/sync-status", post(upsert_org_sync_status))
        .route("/brands/organizations/{id}/conflicts", get(list_org_conflicts))
        .route("/brands/organizations/{id}/sections", get(list_org_custom_sections))
        .route("/brands/organizations/{id}/sections", post(create_org_custom_section))
        .route("/brands/organizations/{id}/sections/{section_id}", get(get_org_custom_section))
        .route("/brands/organizations/{id}/sections/{section_id}", put(update_org_custom_section))
        .route("/brands/organizations/{id}/sections/{section_id}", delete(delete_org_custom_section))
        .route("/brands/organizations/{id}/sections/{section_id}/data", get(get_org_section_data))
        .route("/brands/organizations/{id}/sections/{section_id}/data", post(upsert_org_section_data))
        .route("/brands/organizations/{id}/sections/{section_id}/data/{data_id}", delete(delete_org_section_data))
        // ===== WhatsApp Channel =====
        .route("/brands/{id}/whatsapp/health", get(whatsapp_health))
        .route("/brands/{id}/whatsapp/templates", get(list_whatsapp_templates))
        .route("/brands/{id}/whatsapp/sync-templates", post(sync_whatsapp_templates))
        .route("/brands/{id}/whatsapp/sync-profile", post(sync_whatsapp_profile_now))
        .route("/brands/{id}/whatsapp/customers", get(list_whatsapp_customers))
        .route("/brands/{id}/whatsapp/send-test", post(send_whatsapp_test))
        .route("/brands/{id}/whatsapp/automation-rules", get(list_whatsapp_rules))
        .route("/brands/{id}/whatsapp/automation-rules", post(create_whatsapp_rule))
        .route("/brands/{id}/whatsapp/automation-rules/{rule_id}", put(update_whatsapp_rule))
        .route("/brands/{id}/whatsapp/automation-rules/{rule_id}", delete(delete_whatsapp_rule))
        .route("/brands/{id}/whatsapp/nudges", post(run_whatsapp_nudges))
        // ===== WhatsApp Campaigns, Flows, Industries =====
        // Industries
        .route("/brands/{id}/whatsapp/industries", get(list_industries))
        .route("/brands/{id}/whatsapp/industries", post(create_industry))
        .route("/brands/{id}/whatsapp/industries/{industry_id}", put(update_industry))
        .route("/brands/{id}/whatsapp/industries/{industry_id}", delete(delete_industry))
        // Template Groups
        .route("/brands/{id}/whatsapp/groups", get(list_template_groups))
        .route("/brands/{id}/whatsapp/groups", post(create_template_group))
        .route("/brands/{id}/whatsapp/groups/{group_id}", put(update_template_group))
        .route("/brands/{id}/whatsapp/groups/{group_id}", delete(delete_template_group))
        // Template Flows
        .route("/brands/{id}/whatsapp/flows", get(list_flows))
        .route("/brands/{id}/whatsapp/flows", post(create_flow))
        .route("/brands/{id}/whatsapp/flows/{flow_id}", get(get_flow))
        .route("/brands/{id}/whatsapp/flows/{flow_id}", put(update_flow))
        .route("/brands/{id}/whatsapp/flows/{flow_id}", delete(delete_flow))
        .route("/brands/{id}/whatsapp/flows/{flow_id}/duplicate", post(duplicate_flow))
        .route("/brands/{id}/whatsapp/flows/{flow_id}/validate", post(validate_flow))
        // Flow Steps
        .route("/brands/{id}/whatsapp/flows/{flow_id}/steps", get(list_flow_steps))
        .route("/brands/{id}/whatsapp/flows/{flow_id}/steps", post(create_flow_step))
        .route("/brands/{id}/whatsapp/flows/{flow_id}/steps/{step_id}", put(update_flow_step))
        .route("/brands/{id}/whatsapp/flows/{flow_id}/steps/{step_id}", delete(delete_flow_step))
        // Campaigns
        .route("/brands/{id}/whatsapp/campaigns", get(list_campaigns))
        .route("/brands/{id}/whatsapp/campaigns", post(create_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}", get(get_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}", delete(delete_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/start", post(start_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/pause", post(pause_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/resume", post(resume_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/cancel", post(cancel_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/contacts", get(get_campaign_contacts))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/contacts", post(add_campaign_contacts))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/retry", post(retry_campaign))
        .route("/brands/{id}/whatsapp/campaigns/{campaign_id}/analytics", get(get_campaign_analytics))
        // Message Limits
        .route("/brands/{id}/whatsapp/message-limits", get(get_message_limits))
        .route("/brands/{id}/whatsapp/sync-limits", post(sync_message_limits))
        // Template Sync
        .route("/brands/{id}/whatsapp/sync-templates-now", post(sync_templates_now))
        // AI Campaign
        .route("/brands/{id}/whatsapp/campaigns/ai-suggest", post(ai_suggest_campaign))
        .route("/brands/{id}/whatsapp/campaigns/ai-generate", post(ai_generate_campaign))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gen_id() -> String {
    Uuid::new_v4().to_string()
}

fn slug_from_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

async fn resolve_tenant_key(
    headers: &axum::http::HeaderMap,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    headers
        .get("X-Tenant-Key")
        .or_else(|| headers.get("x-tenant-key"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing X-Tenant-Key header" })),
            )
        })
}

// ---------------------------------------------------------------------------
// Organizations
// ---------------------------------------------------------------------------

pub async fn list_organizations(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let rows = sqlx::query("SELECT * FROM organizations WHERE tenant_key = ? ORDER BY name")
        .bind(&tenant_key)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let orgs: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "tenantKey": r.try_get::<String, _>("tenant_key").unwrap_or_default(),
                "name": r.try_get::<String, _>("name").unwrap_or_default(),
                "description": r.try_get::<String, _>("description").unwrap_or_default(),
                "industry": r.try_get::<String, _>("industry").unwrap_or_default(),
                "website": r.try_get::<String, _>("website").unwrap_or_default(),
                "email": r.try_get::<String, _>("email").unwrap_or_default(),
                "phone": r.try_get::<String, _>("phone").unwrap_or_default(),
                "address": r.try_get::<String, _>("address").unwrap_or_default(),
                "timezone": r.try_get::<String, _>("timezone").unwrap_or_default(),
                "createdAt": r.try_get::<String, _>("created_at").unwrap_or_default(),
                "updatedAt": r.try_get::<String, _>("updated_at").unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "organizations": orgs })))
}

pub async fn create_organization(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let id = gen_id();
    let slug = slug_from_name(&body.name);

    sqlx::query(
        "INSERT INTO organizations (id, tenant_key, name, description, industry, website, email, phone, address, timezone, settings)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&tenant_key)
    .bind(&body.name)
    .bind(body.description.as_deref().unwrap_or(""))
    .bind(body.industry.as_deref().unwrap_or(""))
    .bind(body.website.as_deref().unwrap_or(""))
    .bind(body.email.as_deref().unwrap_or(""))
    .bind(body.phone.as_deref().unwrap_or(""))
    .bind(body.address.as_deref().unwrap_or(""))
    .bind(body.timezone.as_deref().unwrap_or("UTC"))
    .bind("{}")
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // Create a default brand for the new organization
    let brand_id = gen_id();
    let brand_slug = format!("{}-brand", slug);
    sqlx::query(
        "INSERT INTO brands (id, organization_id, name, slug, description) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&brand_id)
    .bind(&id)
    .bind(format!("{} Primary", body.name))
    .bind(&brand_slug)
    .bind("")
    .execute(&state.db)
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "id": id,
        "tenantKey": tenant_key,
        "name": body.name,
        "defaultBrandId": brand_id,
        "createdAt": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn get_organization(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let row = sqlx::query("SELECT * FROM organizations WHERE id = ? AND tenant_key = ?")
        .bind(&id)
        .bind(&tenant_key)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Organization not found" })),
        ));
    };

    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "tenantKey": row.try_get::<String, _>("tenant_key").unwrap_or_default(),
        "name": row.try_get::<String, _>("name").unwrap_or_default(),
        "description": row.try_get::<String, _>("description").unwrap_or_default(),
        "industry": row.try_get::<String, _>("industry").unwrap_or_default(),
        "website": row.try_get::<String, _>("website").unwrap_or_default(),
        "email": row.try_get::<String, _>("email").unwrap_or_default(),
        "phone": row.try_get::<String, _>("phone").unwrap_or_default(),
        "address": row.try_get::<String, _>("address").unwrap_or_default(),
        "timezone": row.try_get::<String, _>("timezone").unwrap_or_default(),
        "settings": row.try_get::<String, _>("settings").unwrap_or_default(),
        "createdAt": row.try_get::<String, _>("created_at").unwrap_or_default(),
        "updatedAt": row.try_get::<String, _>("updated_at").unwrap_or_default(),
    })))
}

pub async fn update_organization(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdateOrganizationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let mut sets = Vec::new();
    if let Some(name) = &body.name {
        sets.push(format!("name = '{}'", name.replace('\'', "''")));
    }
    if let Some(desc) = &body.description {
        sets.push(format!("description = '{}'", desc.replace('\'', "''")));
    }
    if let Some(ind) = &body.industry {
        sets.push(format!("industry = '{}'", ind.replace('\'', "''")));
    }
    if let Some(web) = &body.website {
        sets.push(format!("website = '{}'", web.replace('\'', "''")));
    }
    if let Some(email) = &body.email {
        sets.push(format!("email = '{}'", email.replace('\'', "''")));
    }
    if let Some(phone) = &body.phone {
        sets.push(format!("phone = '{}'", phone.replace('\'', "''")));
    }
    if let Some(addr) = &body.address {
        sets.push(format!("address = '{}'", addr.replace('\'', "''")));
    }
    if let Some(tz) = &body.timezone {
        sets.push(format!("timezone = '{}'", tz.replace('\'', "''")));
    }
    if let Some(settings) = &body.settings {
        sets.push(format!("settings = '{}'", settings.replace('\'', "''")));
    }
    sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string());

    if sets.is_empty() {
        return Ok(Json(serde_json::json!({ "message": "No changes" })));
    }

    let query = format!(
        "UPDATE organizations SET {} WHERE id = ? AND tenant_key = ?",
        sets.join(", ")
    );

    sqlx::query(&query)
        .bind(&id)
        .bind(&tenant_key)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({
        "id": id,
        "updatedAt": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn delete_organization(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    sqlx::query("DELETE FROM organizations WHERE id = ? AND tenant_key = ?")
        .bind(&id)
        .bind(&tenant_key)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// Brands
// ---------------------------------------------------------------------------

pub async fn list_brands(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let org_id = params.get("organizationId");
    let query = if org_id.is_some() {
        "SELECT b.*, o.tenant_key FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE o.tenant_key = ? AND b.organization_id = ?
         ORDER BY b.name"
    } else {
        "SELECT b.*, o.tenant_key FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE o.tenant_key = ?
         ORDER BY b.name"
    };

    let mut q = sqlx::query(query).bind(&tenant_key);
    if let Some(ref org_id) = org_id {
        q = q.bind(org_id);
    }

    let rows = q.fetch_all(&state.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let brands: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "organizationId": r.try_get::<String, _>("organization_id").unwrap_or_default(),
                "name": r.try_get::<String, _>("name").unwrap_or_default(),
                "slug": r.try_get::<String, _>("slug").unwrap_or_default(),
                "description": r.try_get::<String, _>("description").unwrap_or_default(),
                "logoUrl": r.try_get::<String, _>("logo_url").unwrap_or_default(),
                "tagline": r.try_get::<String, _>("tagline").unwrap_or_default(),
                "createdAt": r.try_get::<String, _>("created_at").unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "brands": brands })))
}

pub async fn create_brand(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateBrandRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    // Verify organization belongs to tenant
    let org = sqlx::query("SELECT id FROM organizations WHERE id = ? AND tenant_key = ?")
        .bind(&body.organization_id)
        .bind(&tenant_key)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if org.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Organization not found" })),
        ));
    }

    let id = gen_id();
    let slug = body
        .slug
        .unwrap_or_else(|| slug_from_name(&body.name));

    sqlx::query(
        "INSERT INTO brands (id, organization_id, name, slug, description, logo_url, tagline,
         color_scheme, bio, website, email, phone, address, social_links, products, services,
         prices, policies, rules, tone_guidelines, keywords)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&body.organization_id)
    .bind(&body.name)
    .bind(&slug)
    .bind(body.description.as_deref().unwrap_or(""))
    .bind(body.logo_url.as_deref().unwrap_or(""))
    .bind(body.tagline.as_deref().unwrap_or(""))
    .bind(body.color_scheme.as_deref().unwrap_or("{}"))
    .bind(body.bio.as_deref().unwrap_or(""))
    .bind(body.website.as_deref().unwrap_or(""))
    .bind(body.email.as_deref().unwrap_or(""))
    .bind(body.phone.as_deref().unwrap_or(""))
    .bind(body.address.as_deref().unwrap_or(""))
    .bind(body.social_links.as_deref().unwrap_or("{}"))
    .bind(body.products.as_deref().unwrap_or("[]"))
    .bind(body.services.as_deref().unwrap_or("[]"))
    .bind(body.prices.as_deref().unwrap_or("{}"))
    .bind(body.policies.as_deref().unwrap_or("{}"))
    .bind(body.rules.as_deref().unwrap_or("{}"))
    .bind(body.tone_guidelines.as_deref().unwrap_or(""))
    .bind(body.keywords.as_deref().unwrap_or("[]"))
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(serde_json::json!({
        "id": id,
        "organizationId": body.organization_id,
        "name": body.name,
        "slug": slug,
        "createdAt": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn get_brand(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let row = sqlx::query(
        "SELECT b.*, o.tenant_key FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    };

    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "organizationId": row.try_get::<String, _>("organization_id").unwrap_or_default(),
        "name": row.try_get::<String, _>("name").unwrap_or_default(),
        "slug": row.try_get::<String, _>("slug").unwrap_or_default(),
        "description": row.try_get::<String, _>("description").unwrap_or_default(),
        "logoUrl": row.try_get::<String, _>("logo_url").unwrap_or_default(),
        "tagline": row.try_get::<String, _>("tagline").unwrap_or_default(),
        "colorScheme": row.try_get::<String, _>("color_scheme").unwrap_or_default(),
        "bio": row.try_get::<String, _>("bio").unwrap_or_default(),
        "website": row.try_get::<String, _>("website").unwrap_or_default(),
        "email": row.try_get::<String, _>("email").unwrap_or_default(),
        "phone": row.try_get::<String, _>("phone").unwrap_or_default(),
        "address": row.try_get::<String, _>("address").unwrap_or_default(),
        "socialLinks": row.try_get::<String, _>("social_links").unwrap_or_default(),
        "products": row.try_get::<String, _>("products").unwrap_or_default(),
        "services": row.try_get::<String, _>("services").unwrap_or_default(),
        "prices": row.try_get::<String, _>("prices").unwrap_or_default(),
        "policies": row.try_get::<String, _>("policies").unwrap_or_default(),
        "rules": row.try_get::<String, _>("rules").unwrap_or_default(),
        "toneGuidelines": row.try_get::<String, _>("tone_guidelines").unwrap_or_default(),
        "keywords": row.try_get::<String, _>("keywords").unwrap_or_default(),
        "createdAt": row.try_get::<String, _>("created_at").unwrap_or_default(),
        "updatedAt": row.try_get::<String, _>("updated_at").unwrap_or_default(),
    })))
}

pub async fn update_brand(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdateBrandRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    // Verify brand belongs to tenant
    let existing = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if existing.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let mut sets = Vec::new();
    if let Some(name) = &body.name {
        sets.push(format!("name = '{}'", name.replace('\'', "''")));
    }
    if let Some(slug) = &body.slug {
        sets.push(format!("slug = '{}'", slug.replace('\'', "''")));
    }
    if let Some(desc) = &body.description {
        sets.push(format!("description = '{}'", desc.replace('\'', "''")));
    }
    if let Some(logo) = &body.logo_url {
        sets.push(format!("logo_url = '{}'", logo.replace('\'', "''")));
    }
    if let Some(tagline) = &body.tagline {
        sets.push(format!("tagline = '{}'", tagline.replace('\'', "''")));
    }
    if let Some(cs) = &body.color_scheme {
        sets.push(format!("color_scheme = '{}'", cs.replace('\'', "''")));
    }
    if let Some(bio) = &body.bio {
        sets.push(format!("bio = '{}'", bio.replace('\'', "''")));
    }
    if let Some(web) = &body.website {
        sets.push(format!("website = '{}'", web.replace('\'', "''")));
    }
    if let Some(email) = &body.email {
        sets.push(format!("email = '{}'", email.replace('\'', "''")));
    }
    if let Some(phone) = &body.phone {
        sets.push(format!("phone = '{}'", phone.replace('\'', "''")));
    }
    if let Some(addr) = &body.address {
        sets.push(format!("address = '{}'", addr.replace('\'', "''")));
    }
    if let Some(sl) = &body.social_links {
        sets.push(format!("social_links = '{}'", sl.replace('\'', "''")));
    }
    if let Some(pr) = &body.products {
        sets.push(format!("products = '{}'", pr.replace('\'', "''")));
    }
    if let Some(sv) = &body.services {
        sets.push(format!("services = '{}'", sv.replace('\'', "''")));
    }
    if let Some(pr) = &body.prices {
        sets.push(format!("prices = '{}'", pr.replace('\'', "''")));
    }
    if let Some(pol) = &body.policies {
        sets.push(format!("policies = '{}'", pol.replace('\'', "''")));
    }
    if let Some(rules) = &body.rules {
        sets.push(format!("rules = '{}'", rules.replace('\'', "''")));
    }
    if let Some(tone) = &body.tone_guidelines {
        sets.push(format!("tone_guidelines = '{}'", tone.replace('\'', "''")));
    }
    if let Some(kw) = &body.keywords {
        sets.push(format!("keywords = '{}'", kw.replace('\'', "''")));
    }
    sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string());

    if sets.is_empty() {
        return Ok(Json(serde_json::json!({ "message": "No changes" })));
    }

    let query = format!("UPDATE brands SET {} WHERE id = ?", sets.join(", "));
    sqlx::query(&query)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({
        "id": id,
        "updatedAt": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn delete_brand(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let existing = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if existing.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    sqlx::query("DELETE FROM brands WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn get_brand_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<BrandProfileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    // Fetch brand
    let brand_row = sqlx::query(
        "SELECT b.*, o.tenant_key FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let Some(brand_row) = brand_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    };

    let brand: serde_json::Value = serde_json::json!({
        "id": brand_row.try_get::<String, _>("id").unwrap_or_default(),
        "organizationId": brand_row.try_get::<String, _>("organization_id").unwrap_or_default(),
        "name": brand_row.try_get::<String, _>("name").unwrap_or_default(),
        "slug": brand_row.try_get::<String, _>("slug").unwrap_or_default(),
        "description": brand_row.try_get::<String, _>("description").unwrap_or_default(),
        "logoUrl": brand_row.try_get::<String, _>("logo_url").unwrap_or_default(),
        "tagline": brand_row.try_get::<String, _>("tagline").unwrap_or_default(),
        "colorScheme": brand_row.try_get::<String, _>("color_scheme").unwrap_or_default(),
        "bio": brand_row.try_get::<String, _>("bio").unwrap_or_default(),
        "website": brand_row.try_get::<String, _>("website").unwrap_or_default(),
        "email": brand_row.try_get::<String, _>("email").unwrap_or_default(),
        "phone": brand_row.try_get::<String, _>("phone").unwrap_or_default(),
        "address": brand_row.try_get::<String, _>("address").unwrap_or_default(),
        "socialLinks": brand_row.try_get::<String, _>("social_links").unwrap_or_default(),
        "products": brand_row.try_get::<String, _>("products").unwrap_or_default(),
        "services": brand_row.try_get::<String, _>("services").unwrap_or_default(),
        "prices": brand_row.try_get::<String, _>("prices").unwrap_or_default(),
        "policies": brand_row.try_get::<String, _>("policies").unwrap_or_default(),
        "rules": brand_row.try_get::<String, _>("rules").unwrap_or_default(),
        "toneGuidelines": brand_row.try_get::<String, _>("tone_guidelines").unwrap_or_default(),
        "keywords": brand_row.try_get::<String, _>("keywords").unwrap_or_default(),
    });

    // Fetch organization
    let org_id_val = brand_row.try_get::<String, _>("organization_id").unwrap_or_default();
    let org_row = sqlx::query("SELECT * FROM organizations WHERE id = ?")
        .bind(&org_id_val)
        .fetch_optional(&state.db)
        .await
        .ok();

    let organization: serde_json::Value = match org_row {
        Some(Some(r)) => serde_json::json!({
            "id": r.try_get::<String, _>("id").unwrap_or_default(),
            "name": r.try_get::<String, _>("name").unwrap_or_default(),
            "description": r.try_get::<String, _>("description").unwrap_or_default(),
            "industry": r.try_get::<String, _>("industry").unwrap_or_default(),
            "website": r.try_get::<String, _>("website").unwrap_or_default(),
            "email": r.try_get::<String, _>("email").unwrap_or_default(),
            "phone": r.try_get::<String, _>("phone").unwrap_or_default(),
        }),
        _ => serde_json::json!({}),
    };

    // Fetch AI config
    let ai_row = sqlx::query("SELECT * FROM brand_ai_config WHERE brand_id = ? AND is_active = 1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .ok();

    let ai_config: Option<serde_json::Value> = match ai_row {
        Some(Some(r)) => Some(serde_json::json!({
            "aiProvider": r.try_get::<String, _>("ai_provider").unwrap_or_default(),
            "modelId": r.try_get::<String, _>("model_id").unwrap_or_default(),
            "temperature": r.try_get::<f64, _>("temperature").unwrap_or(0.7),
            "maxTokens": r.try_get::<i64, _>("max_tokens").unwrap_or(4096),
            "systemPrompt": r.try_get::<String, _>("system_prompt").unwrap_or_default(),
            "ragEnabled": r.try_get::<bool, _>("rag_enabled").unwrap_or(false),
        })),
        _ => None,
    };

    // Count knowledge entries
    let kb_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM brand_knowledge_entries WHERE brand_id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Ok(Json(BrandProfileResponse {
        brand,
        organization,
        ai_config,
        knowledge_count: kb_count,
    }))
}

// ---------------------------------------------------------------------------
// Brand Credentials (BYOK)
// ---------------------------------------------------------------------------

pub async fn list_credentials(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    // Verify brand belongs to tenant
    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let rows = sqlx::query(
        "SELECT id, brand_id, channel_type, is_active, last_verified_at, created_at, updated_at
         FROM brand_credentials WHERE brand_id = ?",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let creds: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "channelType": r.try_get::<String, _>("channel_type").unwrap_or_default(),
                "isActive": r.try_get::<bool, _>("is_active").unwrap_or(true),
                "lastVerifiedAt": r.try_get::<Option<String>, _>("last_verified_at").unwrap_or(None),
                "createdAt": r.try_get::<String, _>("created_at").unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "credentials": creds })))
}

pub async fn upsert_credentials(
    State(state): State<AppState>,
    Path((id, channel)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    // Verify brand belongs to tenant
    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let credentials_json = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());

    // Check if credentials already exist for this channel
    let existing = sqlx::query(
        "SELECT id FROM brand_credentials WHERE brand_id = ? AND channel_type = ?",
    )
    .bind(&id)
    .bind(&channel)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if let Some(row) = existing {
        let cred_id = row.try_get::<String, _>("id").unwrap_or_default();
        sqlx::query(
            "UPDATE brand_credentials SET credentials = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(&credentials_json)
        .bind(&cred_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        Ok(Json(serde_json::json!({
            "id": cred_id,
            "brandId": id,
            "channelType": channel,
            "updated": true,
        })))
    } else {
        let cred_id = gen_id();
        sqlx::query(
            "INSERT INTO brand_credentials (id, brand_id, channel_type, credentials) VALUES (?, ?, ?, ?)",
        )
        .bind(&cred_id)
        .bind(&id)
        .bind(&channel)
        .bind(&credentials_json)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        Ok(Json(serde_json::json!({
            "id": cred_id,
            "brandId": id,
            "channelType": channel,
            "created": true,
        })))
    }
}

pub async fn delete_credentials(
    State(state): State<AppState>,
    Path((id, channel)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    sqlx::query("DELETE FROM brand_credentials WHERE brand_id = ? AND channel_type = ?")
        .bind(&id)
        .bind(&channel)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn verify_credentials(
    State(state): State<AppState>,
    Path((id, channel)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    // Verify brand belongs to tenant
    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    // Fetch credentials for this brand + channel
    let row = sqlx::query(
        "SELECT credentials FROM brand_credentials WHERE brand_id = ? AND channel_type = ? AND is_active = 1",
    )
    .bind(&id)
    .bind(&channel)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "No credentials found for this channel" })),
        ));
    };

    let creds_str: String = row.try_get("credentials").unwrap_or_else(|_| "{}".to_string());
    let creds: serde_json::Value = serde_json::from_str(&creds_str).unwrap_or(serde_json::json!({}));

    // Verify based on channel type
    let result = match channel.to_lowercase().as_str() {
        "whatsapp" => verify_whatsapp_brand(&state, &creds).await,
        "sms" | "twilio" => verify_twilio_brand(&state, &creds).await,
        "telegram" => verify_telegram_brand(&state, &creds).await,
        "line" => verify_line_brand(&state, &creds).await,
        "ai" | "anthropic" => verify_ai_brand(&state, &creds).await,
        _ => serde_json::json!({
            "connected": false,
            "error": format!("Channel '{}' verification not implemented", channel)
        }),
    };

    // Update last_verified_at
    sqlx::query(
        "UPDATE brand_credentials SET last_verified_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE brand_id = ? AND channel_type = ?",
    )
    .bind(&id)
    .bind(&channel)
    .execute(&state.db)
    .await
    .ok();

    Ok(Json(result))
}

// Brand-specific verification functions (use brand's own credentials)

async fn verify_whatsapp_brand(state: &AppState, creds: &serde_json::Value) -> serde_json::Value {
    let token = creds["accessToken"].as_str().unwrap_or("");
    let phone_id = creds["phoneNumberId"].as_str().unwrap_or("");

    if token.is_empty() || phone_id.is_empty() {
        return serde_json::json!({ "connected": false, "error": "Missing WhatsApp credentials" });
    }

    match state
        .http_client
        .get(format!("https://graph.facebook.com/v21.0/{}", phone_id))
        .query(&[("access_token", token)])
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body: Result<serde_json::Value, _> = resp.json().await;
            serde_json::json!({
                "connected": true,
                "providerInfo": body.ok(),
                "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
            })
        }
        Ok(resp) => serde_json::json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => serde_json::json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_twilio_brand(state: &AppState, creds: &serde_json::Value) -> serde_json::Value {
    let sid = creds["accountSid"].as_str().unwrap_or("");
    let token = creds["authToken"].as_str().unwrap_or("");

    if sid.is_empty() || token.is_empty() {
        return serde_json::json!({ "connected": false, "error": "Missing Twilio credentials" });
    }

    match state
        .http_client
        .get("https://api.twilio.com/2010-04-01/Accounts.json")
        .basic_auth(sid, Some(token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => serde_json::json!({
            "connected": true,
            "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
        }),
        Ok(resp) => serde_json::json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => serde_json::json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_telegram_brand(state: &AppState, creds: &serde_json::Value) -> serde_json::Value {
    let token = creds["botToken"].as_str().unwrap_or("");

    if token.is_empty() {
        return serde_json::json!({ "connected": false, "error": "Missing Telegram bot token" });
    }

    match state
        .http_client
        .get(format!("https://api.telegram.org/bot{}/getMe", token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => serde_json::json!({
            "connected": true,
            "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
        }),
        Ok(resp) => serde_json::json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => serde_json::json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_line_brand(state: &AppState, creds: &serde_json::Value) -> serde_json::Value {
    let token = creds["channelAccessToken"].as_str().unwrap_or("");

    if token.is_empty() {
        return serde_json::json!({ "connected": false, "error": "Missing LINE token" });
    }

    match state
        .http_client
        .get("https://api.line.me/v2/bot/info")
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => serde_json::json!({
            "connected": true,
            "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
        }),
        Ok(resp) => serde_json::json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => serde_json::json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

async fn verify_ai_brand(state: &AppState, creds: &serde_json::Value) -> serde_json::Value {
    let api_key = creds["apiKey"].as_str().unwrap_or("");

    if api_key.is_empty() {
        return serde_json::json!({ "connected": false, "error": "Missing AI API key" });
    }

    // Test with a minimal request to Anthropic
    match state
        .http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "ok"}]
        })
        .to_string())
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => serde_json::json!({
            "connected": true,
            "lastConnectedAt": chrono::Utc::now().to_rfc3339(),
        }),
        Ok(resp) => serde_json::json!({
            "connected": false,
            "error": format!("HTTP {}", resp.status()),
        }),
        Err(e) => serde_json::json!({ "connected": false, "error": format!("Request failed: {}", e) }),
    }
}

// ---------------------------------------------------------------------------
// Brand AI Config
// ---------------------------------------------------------------------------

pub async fn get_ai_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let row = sqlx::query(
        "SELECT * FROM brand_ai_config WHERE brand_id = ? ORDER BY is_active DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "id": r.try_get::<String, _>("id").unwrap_or_default(),
            "brandId": r.try_get::<String, _>("brand_id").unwrap_or_default(),
            "aiProvider": r.try_get::<String, _>("ai_provider").unwrap_or_default(),
            "modelId": r.try_get::<String, _>("model_id").unwrap_or_default(),
            "baseUrl": r.try_get::<String, _>("base_url").unwrap_or_default(),
            "temperature": r.try_get::<f64, _>("temperature").unwrap_or(0.7),
            "maxTokens": r.try_get::<i64, _>("max_tokens").unwrap_or(4096),
            "systemPrompt": r.try_get::<String, _>("system_prompt").unwrap_or_default(),
            "ragEnabled": r.try_get::<bool, _>("rag_enabled").unwrap_or(false),
            "embeddingProvider": r.try_get::<String, _>("embedding_provider").unwrap_or_default(),
            "embeddingModel": r.try_get::<String, _>("embedding_model").unwrap_or_default(),
            "isActive": r.try_get::<bool, _>("is_active").unwrap_or(true),
            "createdAt": r.try_get::<String, _>("created_at").unwrap_or_default(),
            "updatedAt": r.try_get::<String, _>("updated_at").unwrap_or_default(),
        }))),
        None => Ok(Json(serde_json::json!({
            "brandId": id,
            "aiProvider": "anthropic",
            "modelId": "claude-sonnet-4-20250514",
            "temperature": 0.7,
            "maxTokens": 4096,
            "systemPrompt": "",
            "ragEnabled": false,
        }))),
    }
}

pub async fn upsert_ai_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertAIConfigRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    // Check if config already exists
    let existing = sqlx::query("SELECT id FROM brand_ai_config WHERE brand_id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if let Some(row) = existing {
        let config_id = row.try_get::<String, _>("id").unwrap_or_default();
        let mut sets = Vec::new();
        if let Some(provider) = &body.ai_provider {
            sets.push(format!("ai_provider = '{}'", provider.replace('\'', "''")));
        }
        if let Some(key) = &body.api_key {
            sets.push(format!("api_key = '{}'", key.replace('\'', "''")));
        }
        if let Some(model) = &body.model_id {
            sets.push(format!("model_id = '{}'", model.replace('\'', "''")));
        }
        if let Some(url) = &body.base_url {
            sets.push(format!("base_url = '{}'", url.replace('\'', "''")));
        }
        if let Some(temp) = body.temperature {
            sets.push(format!("temperature = {}", temp));
        }
        if let Some(tokens) = body.max_tokens {
            sets.push(format!("max_tokens = {}", tokens));
        }
        if let Some(prompt) = &body.system_prompt {
            sets.push(format!("system_prompt = '{}'", prompt.replace('\'', "''")));
        }
        if let Some(rag) = body.rag_enabled {
            sets.push(format!("rag_enabled = {}", if rag { 1 } else { 0 }));
        }
        if let Some(ep) = &body.embedding_provider {
            sets.push(format!("embedding_provider = '{}'", ep.replace('\'', "''")));
        }
        if let Some(ekey) = &body.embedding_api_key {
            sets.push(format!("embedding_api_key = '{}'", ekey.replace('\'', "''")));
        }
        if let Some(em) = &body.embedding_model {
            sets.push(format!("embedding_model = '{}'", em.replace('\'', "''")));
        }
        sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string());

        let query = format!(
            "UPDATE brand_ai_config SET {} WHERE id = ?",
            sets.join(", ")
        );
        sqlx::query(&query)
            .bind(&config_id)
            .execute(&state.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

        Ok(Json(serde_json::json!({
            "id": config_id,
            "brandId": id,
            "updated": true,
        })))
    } else {
        let config_id = gen_id();
        sqlx::query(
            "INSERT INTO brand_ai_config (id, brand_id, ai_provider, api_key, model_id, base_url,
             temperature, max_tokens, system_prompt, rag_enabled, embedding_provider,
             embedding_api_key, embedding_model)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&config_id)
        .bind(&id)
        .bind(body.ai_provider.as_deref().unwrap_or("anthropic"))
        .bind(body.api_key.as_deref().unwrap_or(""))
        .bind(body.model_id.as_deref().unwrap_or("claude-sonnet-4-20250514"))
        .bind(body.base_url.as_deref().unwrap_or(""))
        .bind(body.temperature.unwrap_or(0.7))
        .bind(body.max_tokens.unwrap_or(4096))
        .bind(body.system_prompt.as_deref().unwrap_or(""))
        .bind(if body.rag_enabled.unwrap_or(false) { 1i32 } else { 0i32 })
        .bind(body.embedding_provider.as_deref().unwrap_or(""))
        .bind(body.embedding_api_key.as_deref().unwrap_or(""))
        .bind(body.embedding_model.as_deref().unwrap_or(""))
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        Ok(Json(serde_json::json!({
            "id": config_id,
            "brandId": id,
            "created": true,
        })))
    }
}

// ---------------------------------------------------------------------------
// Brand Knowledge Base (for RAG)
// ---------------------------------------------------------------------------

pub async fn list_knowledge_entries(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Query(params): Query<SearchKnowledgeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let mut query = String::from(
        "SELECT * FROM brand_knowledge_entries WHERE brand_id = ?",
    );

    if let Some(ref _q) = params.q {
        query.push_str(" AND (title LIKE ? OR content LIKE ?)");
    }
    if let Some(ref _topic) = params.topic {
        query.push_str(" AND topic = ?");
    }
    query.push_str(" ORDER BY created_at DESC");

    let rows = sqlx::query(&query)
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "title": r.try_get::<String, _>("title").unwrap_or_default(),
                "content": r.try_get::<String, _>("content").unwrap_or_default(),
                "sourceType": r.try_get::<String, _>("source_type").unwrap_or_default(),
                "topic": r.try_get::<String, _>("topic").unwrap_or_default(),
                "tags": r.try_get::<String, _>("tags").unwrap_or_default(),
                "createdAt": r.try_get::<String, _>("created_at").unwrap_or_default(),
                "updatedAt": r.try_get::<String, _>("updated_at").unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "entries": entries })))
}

pub async fn create_knowledge_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateKnowledgeEntryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let entry_id = gen_id();
    sqlx::query(
        "INSERT INTO brand_knowledge_entries (id, brand_id, title, content, source_type, topic, tags)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&entry_id)
    .bind(&id)
    .bind(&body.title)
    .bind(&body.content)
    .bind(body.source_type.as_deref().unwrap_or("manual"))
    .bind(body.topic.as_deref().unwrap_or(""))
    .bind(body.tags.as_deref().unwrap_or("[]"))
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(serde_json::json!({
        "id": entry_id,
        "brandId": id,
        "title": body.title,
        "createdAt": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn get_knowledge_entry(
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let row = sqlx::query(
        "SELECT * FROM brand_knowledge_entries WHERE id = ? AND brand_id = ?",
    )
    .bind(&entry_id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Knowledge entry not found" })),
        ));
    };

    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "brandId": row.try_get::<String, _>("brand_id").unwrap_or_default(),
        "title": row.try_get::<String, _>("title").unwrap_or_default(),
        "content": row.try_get::<String, _>("content").unwrap_or_default(),
        "sourceType": row.try_get::<String, _>("source_type").unwrap_or_default(),
        "topic": row.try_get::<String, _>("topic").unwrap_or_default(),
        "tags": row.try_get::<String, _>("tags").unwrap_or_default(),
        "createdAt": row.try_get::<String, _>("created_at").unwrap_or_default(),
        "updatedAt": row.try_get::<String, _>("updated_at").unwrap_or_default(),
    })))
}

pub async fn update_knowledge_entry(
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdateKnowledgeEntryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    let mut sets = Vec::new();
    if let Some(title) = &body.title {
        sets.push(format!("title = '{}'", title.replace('\'', "''")));
    }
    if let Some(content) = &body.content {
        sets.push(format!("content = '{}'", content.replace('\'', "''")));
    }
    if let Some(st) = &body.source_type {
        sets.push(format!("source_type = '{}'", st.replace('\'', "''")));
    }
    if let Some(topic) = &body.topic {
        sets.push(format!("topic = '{}'", topic.replace('\'', "''")));
    }
    if let Some(tags) = &body.tags {
        sets.push(format!("tags = '{}'", tags.replace('\'', "''")));
    }
    sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string());

    if sets.is_empty() {
        return Ok(Json(serde_json::json!({ "message": "No changes" })));
    }

    let query = format!(
        "UPDATE brand_knowledge_entries SET {} WHERE id = ? AND brand_id = ?",
        sets.join(", ")
    );
    sqlx::query(&query)
        .bind(&entry_id)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({
        "id": entry_id,
        "updatedAt": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn delete_knowledge_entry(
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;

    let brand = sqlx::query(
        "SELECT b.id FROM brands b
         JOIN organizations o ON b.organization_id = o.id
         WHERE b.id = ? AND o.tenant_key = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if brand.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        ));
    }

    sqlx::query(
        "DELETE FROM brand_knowledge_entries WHERE id = ? AND brand_id = ?",
    )
    .bind(&entry_id)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// Enhanced Profile: Data Sources
// ---------------------------------------------------------------------------

pub async fn list_profile_sources(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM brand_profile_sources WHERE brand_id = ? ORDER BY field_name")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let sources: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "fieldName": r.try_get::<String, _>("field_name").unwrap_or_default(),
        "sourcePlatform": r.try_get::<String, _>("source_platform").unwrap_or_default(),
        "sourceUserName": r.try_get::<String, _>("source_user_name").unwrap_or_default(),
        "isVerified": r.try_get::<i64, _>("is_verified").unwrap_or(0) == 1,
        "confidenceScore": r.try_get::<f64, _>("confidence_score").unwrap_or(1.0),
    })).collect();
    Ok(Json(serde_json::json!(sources)))
}

pub async fn upsert_profile_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertProfileSourceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let source_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO brand_profile_sources (id, brand_id, field_name, source_platform, source_user_id, source_user_name, source_record_id, source_url, synced_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&source_id).bind(&id).bind(&body.field_name).bind(&body.source_platform)
        .bind(body.source_user_id.as_deref().unwrap_or("")).bind(body.source_user_name.as_deref().unwrap_or(""))
        .bind(body.source_record_id.as_deref().unwrap_or("")).bind(body.source_url.as_deref().unwrap_or(""))
        .bind(&now).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": source_id, "fieldName": body.field_name, "sourcePlatform": body.source_platform, "createdAt": now })))
}

pub async fn delete_profile_source(
    State(state): State<AppState>,
    Path((id, source_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    sqlx::query("DELETE FROM brand_profile_sources WHERE id = ? AND brand_id = ?")
        .bind(&source_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// Enhanced Profile: Sync Status
// ---------------------------------------------------------------------------

pub async fn list_sync_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM brand_sync_status WHERE brand_id = ? ORDER BY platform")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let statuses: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "platform": r.try_get::<String, _>("platform").unwrap_or_default(),
        "isConnected": r.try_get::<i64, _>("is_connected").unwrap_or(0) == 1,
        "connectionStatus": r.try_get::<String, _>("connection_status").unwrap_or_default(),
        "lastSyncedAt": r.try_get::<Option<String>, _>("last_synced_at").ok(),
        "lastError": r.try_get::<String, _>("last_error").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(statuses)))
}

pub async fn upsert_sync_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertSyncStatusRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let now = chrono::Utc::now().to_rfc3339();
    let sync_id = gen_id();
    sqlx::query("INSERT OR REPLACE INTO brand_sync_status (id, brand_id, platform, sync_interval_seconds, next_sync_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&sync_id).bind(&id).bind(&body.platform)
        .bind(body.sync_interval_seconds.unwrap_or(3600)).bind(&now).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": sync_id, "platform": body.platform, "createdAt": now })))
}

pub async fn sync_now(
    State(state): State<AppState>,
    Path((id, platform)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let now = chrono::Utc::now().to_rfc3339();
    let result = match platform.as_str() {
        "whatsapp" | "meta" => sync_whatsapp_profile(&state, &id).await,
        "telegram" => sync_telegram_profile(&state, &id).await,
        "line" => sync_line_profile(&state, &id).await,
        _ => Ok(serde_json::json!({ "status": "skipped", "message": format!("Sync not implemented for {}", platform) })),
    };
    match result {
        Ok(data) => {
            sqlx::query("UPDATE brand_sync_status SET last_synced_at = ?, connection_status = 'connected', platform_data = ?, updated_at = ? WHERE brand_id = ? AND platform = ?")
                .bind(&now).bind(serde_json::to_string(&data).unwrap_or_default()).bind(&now)
                .bind(&id).bind(&platform).execute(&state.db).await.ok();
            Ok(Json(serde_json::json!({ "status": "synced", "platform": platform, "data": data, "syncedAt": now })))
        }
        Err(e) => {
            sqlx::query("UPDATE brand_sync_status SET last_synced_at = ?, connection_status = 'error', last_error = ?, updated_at = ? WHERE brand_id = ? AND platform = ?")
                .bind(&now).bind(e.to_string()).bind(&now).bind(&id).bind(&platform).execute(&state.db).await.ok();
            let mut map = serde_json::Map::new();
            map.insert("error".to_string(), serde_json::json!(e.to_string()));
            map.insert("platform".to_string(), serde_json::json!(platform));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::Value::Object(map))))
        }
    }
}

async fn sync_whatsapp_profile(state: &AppState, brand_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let creds = sqlx::query("SELECT credentials FROM brand_credentials WHERE brand_id = ? AND channel_type = 'whatsapp' AND is_active = 1")
        .bind(brand_id).fetch_optional(&state.db).await?;
    let Some(creds) = creds else { return Err("No WhatsApp credentials found".into()); };
    let cred_json: serde_json::Value = serde_json::from_str(&creds.try_get::<String, _>("credentials").unwrap_or_default())?;
    let token = cred_json.get("access_token").and_then(|v| v.as_str()).ok_or("Missing access_token")?;
    let phone_id = cred_json.get("phone_number_id").and_then(|v| v.as_str()).ok_or("Missing phone_number_id")?;
    let resp = state.http_client.get(format!("https://graph.facebook.com/v21.0/{}", phone_id))
        .query(&[("access_token", token), ("fields", "name,verified_name,quality_rating,messaging_limit")])
        .send().await?.error_for_status()?;
    let data: serde_json::Value = resp.json().await?;
    let now = chrono::Utc::now().to_rfc3339();
    if let Some(name) = data.get("name") {
        cache_platform_data(&state.db, "brand", brand_id, "whatsapp", "brand_name", name.to_string(), &now).await;
    }
    if let Some(verified) = data.get("verified_name") {
        cache_platform_data(&state.db, "brand", brand_id, "whatsapp", "verified_name", verified.to_string(), &now).await;
    }
    if let Some(limit) = data.get("messaging_limit") {
        cache_platform_data(&state.db, "brand", brand_id, "whatsapp", "messaging_limit", limit.to_string(), &now).await;
    }
    Ok(data)
}

async fn sync_telegram_profile(state: &AppState, brand_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let creds = sqlx::query("SELECT credentials FROM brand_credentials WHERE brand_id = ? AND channel_type = 'telegram' AND is_active = 1")
        .bind(brand_id).fetch_optional(&state.db).await?;
    let Some(creds) = creds else { return Err("No Telegram credentials found".into()); };
    let cred_json: serde_json::Value = serde_json::from_str(&creds.try_get::<String, _>("credentials").unwrap_or_default())?;
    let token = cred_json.get("bot_token").and_then(|v| v.as_str()).ok_or("Missing bot_token")?;
    let resp = state.http_client.get(format!("https://api.telegram.org/bot{}/getMe", token))
        .send().await?.error_for_status()?;
    let data: serde_json::Value = resp.json().await?;
    Ok(data)
}

async fn sync_line_profile(state: &AppState, brand_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let creds = sqlx::query("SELECT credentials FROM brand_credentials WHERE brand_id = ? AND channel_type = 'line' AND is_active = 1")
        .bind(brand_id).fetch_optional(&state.db).await?;
    let Some(creds) = creds else { return Err("No LINE credentials found".into()); };
    let cred_json: serde_json::Value = serde_json::from_str(&creds.try_get::<String, _>("credentials").unwrap_or_default())?;
    let token = cred_json.get("channel_access_token").and_then(|v| v.as_str()).ok_or("Missing channel_access_token")?;
    let resp = state.http_client.get("https://api.line.me/v2/bot/info")
        .bearer_auth(token).send().await?.error_for_status()?;
    let data: serde_json::Value = resp.json().await?;
    Ok(data)
}

async fn cache_platform_data(pool: &sqlx::SqlitePool, entity_type: &str, entity_id: &str, platform: &str, data_key: &str, data_value: String, synced_at: &str) {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO platform_data_cache (id, entity_type, entity_id, platform, data_key, data_value, synced_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(entity_type).bind(entity_id).bind(platform)
        .bind(data_key).bind(&data_value).bind(synced_at).bind(&now)
        .execute(pool).await.ok();
}

// ---------------------------------------------------------------------------
// Enhanced Profile: Conflicts
// ---------------------------------------------------------------------------

pub async fn list_conflicts(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM profile_conflicts WHERE entity_type = 'brand' AND entity_id = ? ORDER BY created_at DESC")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let conflicts: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "fieldName": r.try_get::<String, _>("field_name").unwrap_or_default(),
        "conflictType": r.try_get::<String, _>("conflict_type").unwrap_or_default(),
        "sourceAPlatform": r.try_get::<String, _>("source_a_platform").unwrap_or_default(),
        "sourceAValue": r.try_get::<String, _>("source_a_value").unwrap_or_default(),
        "sourceBPlatform": r.try_get::<String, _>("source_b_platform").unwrap_or_default(),
        "sourceBValue": r.try_get::<String, _>("source_b_value").unwrap_or_default(),
        "resolutionStatus": r.try_get::<String, _>("resolution_status").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(conflicts)))
}

pub async fn resolve_conflict(
    State(state): State<AppState>,
    Path((id, conflict_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<ResolveConflictRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let now = chrono::Utc::now().to_rfc3339();
    let resolved_by = body.resolved_by.as_deref().unwrap_or("system");
    let status = if resolved_by == "system" { "auto_resolved" } else { "manual_resolved" };
    sqlx::query("UPDATE profile_conflicts SET resolution_status = ?, resolved_value = ?, resolved_by = ?, resolved_at = ? WHERE id = ? AND entity_type = 'brand' AND entity_id = ?")
        .bind(status).bind(&body.resolved_value).bind(resolved_by).bind(&now)
        .bind(&conflict_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "conflictId": conflict_id, "resolvedValue": body.resolved_value, "resolvedBy": resolved_by, "resolvedAt": now })))
}

// ---------------------------------------------------------------------------
// Enhanced Profile: Custom Sections
// ---------------------------------------------------------------------------

pub async fn list_custom_sections(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM profile_custom_sections WHERE entity_type = 'brand' AND entity_id = ? AND is_active = 1 ORDER BY display_order")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let sections: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "sectionName": r.try_get::<String, _>("section_name").unwrap_or_default(),
        "sectionType": r.try_get::<String, _>("section_type").unwrap_or_default(),
        "displayOrder": r.try_get::<i64, _>("display_order").unwrap_or(0),
    })).collect();
    Ok(Json(serde_json::json!(sections)))
}

pub async fn create_custom_section(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateCustomSectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let section_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO profile_custom_sections (id, entity_type, entity_id, section_name, section_type, display_order, created_at, updated_at) VALUES (?, 'brand', ?, ?, ?, ?, ?, ?)")
        .bind(&section_id).bind(&id).bind(&body.section_name).bind(&body.section_type)
        .bind(body.display_order.unwrap_or(0)).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": section_id, "sectionName": body.section_name, "createdAt": now })))
}

pub async fn get_custom_section(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let row = sqlx::query("SELECT * FROM profile_custom_sections WHERE id = ? AND entity_type = 'brand' AND entity_id = ?")
        .bind(&section_id).bind(&id).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let Some(row) = row else { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Section not found" })))); };
    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "sectionName": row.try_get::<String, _>("section_name").unwrap_or_default(),
        "sectionType": row.try_get::<String, _>("section_type").unwrap_or_default(),
    })))
}

pub async fn update_custom_section(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string()];
    if let Some(name) = body.get("sectionName").and_then(|v| v.as_str()) {
        sets.push(format!("section_name = '{}'", name.replace('\'', "''")));
    }
    let query = format!("UPDATE profile_custom_sections SET {} WHERE id = ? AND entity_id = ?", sets.join(", "));
    sqlx::query(&query).bind(&section_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": section_id, "updatedAt": chrono::Utc::now().to_rfc3339() })))
}

pub async fn delete_custom_section(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    sqlx::query("DELETE FROM profile_custom_sections WHERE id = ? AND entity_id = ?")
        .bind(&section_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn get_section_data(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM profile_custom_section_data WHERE section_id = ? ORDER BY display_order, row_key")
        .bind(&section_id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let data: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "rowKey": r.try_get::<String, _>("row_key").unwrap_or_default(),
        "fieldKey": r.try_get::<String, _>("field_key").unwrap_or_default(),
        "fieldValue": r.try_get::<String, _>("field_value").unwrap_or_default(),
        "fieldType": r.try_get::<String, _>("field_type").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(data)))
}

pub async fn upsert_section_data(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertSectionDataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let data_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO profile_custom_section_data (id, section_id, row_key, field_key, field_value, field_type, display_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&data_id).bind(&section_id)
        .bind(body.row_key.as_deref().unwrap_or(""))
        .bind(&body.field_key).bind(&body.field_value)
        .bind(body.field_type.as_deref().unwrap_or("text"))
        .bind(body.display_order.unwrap_or(0)).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": data_id, "createdAt": now })))
}

pub async fn delete_section_data(
    State(state): State<AppState>,
    Path((id, section_id, data_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    sqlx::query("DELETE FROM profile_custom_section_data WHERE id = ? AND section_id = ?")
        .bind(&data_id).bind(&section_id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// Enhanced Profile: Packages, Promotions, Vouchers, Triggers, Cache
// ---------------------------------------------------------------------------

pub async fn list_packages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM brand_packages WHERE brand_id = ? AND is_active = 1 ORDER BY display_order")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let packages: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "name": r.try_get::<String, _>("name").unwrap_or_default(),
        "price": r.try_get::<f64, _>("price").unwrap_or(0.0),
        "currency": r.try_get::<String, _>("currency").unwrap_or_default(),
        "features": r.try_get::<String, _>("features").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(packages)))
}

pub async fn create_package(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreatePackageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let pkg_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO brand_packages (id, brand_id, name, description, price, currency, duration_type, duration_value, features, display_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&pkg_id).bind(&id).bind(&body.name).bind(body.description.as_deref().unwrap_or(""))
        .bind(body.price).bind(body.currency.as_deref().unwrap_or("USD"))
        .bind(body.duration_type.as_deref().unwrap_or(""))
        .bind(body.duration_value.unwrap_or(1))
        .bind(body.features.as_deref().unwrap_or("[]"))
        .bind(body.display_order.unwrap_or(0)).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": pkg_id, "name": body.name, "createdAt": now })))
}

pub async fn get_package(
    State(state): State<AppState>,
    Path((id, package_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let row = sqlx::query("SELECT * FROM brand_packages WHERE id = ? AND brand_id = ?")
        .bind(&package_id).bind(&id).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let Some(row) = row else { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Package not found" })))); };
    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "name": row.try_get::<String, _>("name").unwrap_or_default(),
        "price": row.try_get::<f64, _>("price").unwrap_or(0.0),
    })))
}

pub async fn update_package(
    State(state): State<AppState>,
    Path((id, package_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdatePackageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string()];
    if let Some(name) = &body.name { sets.push(format!("name = '{}'", name.replace('\'', "''"))); }
    if let Some(price) = body.price { sets.push(format!("price = {}", price)); }
    let query = format!("UPDATE brand_packages SET {} WHERE id = ? AND brand_id = ?", sets.join(", "));
    sqlx::query(&query).bind(&package_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": package_id, "updatedAt": chrono::Utc::now().to_rfc3339() })))
}

pub async fn delete_package(
    State(state): State<AppState>,
    Path((id, package_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    sqlx::query("DELETE FROM brand_packages WHERE id = ? AND brand_id = ?")
        .bind(&package_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn list_promotions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM brand_promotions WHERE brand_id = ? ORDER BY start_date DESC")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let promos: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "name": r.try_get::<String, _>("name").unwrap_or_default(),
        "discountType": r.try_get::<String, _>("discount_type").unwrap_or_default(),
        "discountValue": r.try_get::<f64, _>("discount_value").unwrap_or(0.0),
        "startDate": r.try_get::<String, _>("start_date").unwrap_or_default(),
        "endDate": r.try_get::<String, _>("end_date").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(promos)))
}

pub async fn create_promotion(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreatePromotionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let promo_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO brand_promotions (id, brand_id, name, description, discount_type, discount_value, start_date, end_date, applicable_packages, usage_limit, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&promo_id).bind(&id).bind(&body.name).bind(body.description.as_deref().unwrap_or(""))
        .bind(&body.discount_type).bind(body.discount_value)
        .bind(&body.start_date).bind(&body.end_date)
        .bind(body.applicable_packages.as_deref().unwrap_or("[]"))
        .bind(body.usage_limit.unwrap_or(0)).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": promo_id, "name": body.name, "createdAt": now })))
}

pub async fn get_promotion(
    State(state): State<AppState>,
    Path((id, promo_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let row = sqlx::query("SELECT * FROM brand_promotions WHERE id = ? AND brand_id = ?")
        .bind(&promo_id).bind(&id).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let Some(row) = row else { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Promotion not found" })))); };
    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "name": row.try_get::<String, _>("name").unwrap_or_default(),
        "discountValue": row.try_get::<f64, _>("discount_value").unwrap_or(0.0),
    })))
}

pub async fn update_promotion(
    State(state): State<AppState>,
    Path((id, promo_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdatePromotionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string()];
    if let Some(name) = &body.name { sets.push(format!("name = '{}'", name.replace('\'', "''"))); }
    let query = format!("UPDATE brand_promotions SET {} WHERE id = ? AND brand_id = ?", sets.join(", "));
    sqlx::query(&query).bind(&promo_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": promo_id, "updatedAt": chrono::Utc::now().to_rfc3339() })))
}

pub async fn delete_promotion(
    State(state): State<AppState>,
    Path((id, promo_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    sqlx::query("DELETE FROM brand_promotions WHERE id = ? AND brand_id = ?")
        .bind(&promo_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn get_voucher_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let row = sqlx::query("SELECT * FROM brand_voucher_settings WHERE brand_id = ?")
        .bind(&id).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "autoGenerate": r.try_get::<i64, _>("auto_generate").unwrap_or(0) == 1,
            "voucherPrefix": r.try_get::<String, _>("voucher_prefix").unwrap_or_default(),
            "discountValue": r.try_get::<f64, _>("discount_value").unwrap_or(10.0),
        }))),
        None => Ok(Json(serde_json::json!({ "autoGenerate": false, "voucherPrefix": "VOUCHER", "discountValue": 10.0 }))),
    }
}

pub async fn upsert_voucher_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertVoucherSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let now = chrono::Utc::now().to_rfc3339();
    let vs_id = gen_id();
    sqlx::query("INSERT OR REPLACE INTO brand_voucher_settings (id, brand_id, auto_generate, voucher_prefix, voucher_length, discount_type, discount_value, validity_days, trigger_event, max_uses_per_customer, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&vs_id).bind(&id)
        .bind(if body.auto_generate.unwrap_or(false) { 1i64 } else { 0 })
        .bind(body.voucher_prefix.as_deref().unwrap_or("VOUCHER"))
        .bind(body.voucher_length.unwrap_or(10))
        .bind(body.discount_type.as_deref().unwrap_or("percentage"))
        .bind(body.discount_value.unwrap_or(10.0))
        .bind(body.validity_days.unwrap_or(30))
        .bind(body.trigger_event.as_deref().unwrap_or(""))
        .bind(body.max_uses_per_customer.unwrap_or(1))
        .bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": vs_id, "updatedAt": now })))
}

pub async fn list_message_triggers(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM message_triggers WHERE brand_id = ? ORDER BY created_at DESC")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let triggers: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "name": r.try_get::<String, _>("name").unwrap_or_default(),
        "triggerType": r.try_get::<String, _>("trigger_type").unwrap_or_default(),
        "preferredChannel": r.try_get::<String, _>("preferred_channel").unwrap_or_default(),
        "isActive": r.try_get::<i64, _>("is_active").unwrap_or(1) == 1,
    })).collect();
    Ok(Json(serde_json::json!(triggers)))
}

pub async fn create_message_trigger(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateMessageTriggerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let trigger_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO message_triggers (id, brand_id, name, description, trigger_type, trigger_event, trigger_schedule, trigger_conditions, preferred_channel, message_template, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&trigger_id).bind(&id).bind(&body.name).bind(body.description.as_deref().unwrap_or(""))
        .bind(&body.trigger_type).bind(body.trigger_event.as_deref().unwrap_or(""))
        .bind(body.trigger_schedule.as_deref().unwrap_or(""))
        .bind(body.trigger_conditions.as_deref().unwrap_or("{}"))
        .bind(body.preferred_channel.as_deref().unwrap_or("whatsapp"))
        .bind(&body.message_template).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": trigger_id, "name": body.name, "createdAt": now })))
}

pub async fn get_message_trigger(
    State(state): State<AppState>,
    Path((id, trigger_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let row = sqlx::query("SELECT * FROM message_triggers WHERE id = ? AND brand_id = ?")
        .bind(&trigger_id).bind(&id).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let Some(row) = row else { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Trigger not found" })))); };
    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "name": row.try_get::<String, _>("name").unwrap_or_default(),
        "messageTemplate": row.try_get::<String, _>("message_template").unwrap_or_default(),
    })))
}

pub async fn update_message_trigger(
    State(state): State<AppState>,
    Path((id, trigger_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdateMessageTriggerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string()];
    if let Some(name) = &body.name { sets.push(format!("name = '{}'", name.replace('\'', "''"))); }
    let query = format!("UPDATE message_triggers SET {} WHERE id = ? AND brand_id = ?", sets.join(", "));
    sqlx::query(&query).bind(&trigger_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": trigger_id, "updatedAt": chrono::Utc::now().to_rfc3339() })))
}

pub async fn delete_message_trigger(
    State(state): State<AppState>,
    Path((id, trigger_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    sqlx::query("DELETE FROM message_triggers WHERE id = ? AND brand_id = ?")
        .bind(&trigger_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn get_trigger_logs(
    State(state): State<AppState>,
    Path((id, trigger_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM message_trigger_logs WHERE trigger_id = ? ORDER BY executed_at DESC LIMIT 100")
        .bind(&trigger_id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let logs: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "executedAt": r.try_get::<String, _>("executed_at").unwrap_or_default(),
        "status": r.try_get::<String, _>("status").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(logs)))
}

pub async fn list_platform_cache(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM platform_data_cache WHERE entity_type = 'brand' AND entity_id = ? ORDER BY synced_at DESC")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let cache: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "platform": r.try_get::<String, _>("platform").unwrap_or_default(),
        "dataKey": r.try_get::<String, _>("data_key").unwrap_or_default(),
        "dataValue": r.try_get::<String, _>("data_value").unwrap_or_default(),
        "syncedAt": r.try_get::<String, _>("synced_at").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(cache)))
}

pub async fn get_platform_cache(
    State(state): State<AppState>,
    Path((id, platform)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }
    let rows = sqlx::query("SELECT * FROM platform_data_cache WHERE entity_type = 'brand' AND entity_id = ? AND platform = ? ORDER BY synced_at DESC")
        .bind(&id).bind(&platform).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let mut map = serde_json::Map::new();
    for r in &rows {
        let key = r.try_get::<String, _>("data_key").unwrap_or_default();
        let value = r.try_get::<String, _>("data_value").unwrap_or_default();
        map.insert(key, serde_json::Value::String(value));
    }
    Ok(Json(serde_json::Value::Object(map)))
}

// ---------------------------------------------------------------------------
// Organization Enhanced Profile
// ---------------------------------------------------------------------------

pub async fn list_org_profile_sources(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let rows = sqlx::query("SELECT * FROM org_profile_sources WHERE organization_id = ? ORDER BY field_name")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let sources: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "fieldName": r.try_get::<String, _>("field_name").unwrap_or_default(),
        "sourcePlatform": r.try_get::<String, _>("source_platform").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(sources)))
}

pub async fn upsert_org_profile_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertProfileSourceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let source_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO org_profile_sources (id, organization_id, field_name, source_platform, source_user_id, source_user_name, source_record_id, source_url, synced_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&source_id).bind(&id).bind(&body.field_name).bind(&body.source_platform)
        .bind(body.source_user_id.as_deref().unwrap_or("")).bind(body.source_user_name.as_deref().unwrap_or(""))
        .bind(body.source_record_id.as_deref().unwrap_or("")).bind(body.source_url.as_deref().unwrap_or(""))
        .bind(&now).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": source_id, "createdAt": now })))
}

pub async fn list_org_sync_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let rows = sqlx::query("SELECT * FROM org_sync_status WHERE organization_id = ? ORDER BY platform")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let statuses: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "platform": r.try_get::<String, _>("platform").unwrap_or_default(),
        "isConnected": r.try_get::<i64, _>("is_connected").unwrap_or(0) == 1,
        "connectionStatus": r.try_get::<String, _>("connection_status").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(statuses)))
}

pub async fn upsert_org_sync_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertSyncStatusRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let now = chrono::Utc::now().to_rfc3339();
    let sync_id = gen_id();
    sqlx::query("INSERT OR REPLACE INTO org_sync_status (id, organization_id, platform, sync_interval_seconds, next_sync_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&sync_id).bind(&id).bind(&body.platform)
        .bind(body.sync_interval_seconds.unwrap_or(3600)).bind(&now).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": sync_id, "createdAt": now })))
}

pub async fn list_org_conflicts(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let rows = sqlx::query("SELECT * FROM profile_conflicts WHERE entity_type = 'organization' AND entity_id = ? ORDER BY created_at DESC")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let conflicts: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "fieldName": r.try_get::<String, _>("field_name").unwrap_or_default(),
        "conflictType": r.try_get::<String, _>("conflict_type").unwrap_or_default(),
        "resolutionStatus": r.try_get::<String, _>("resolution_status").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(conflicts)))
}

pub async fn list_org_custom_sections(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let rows = sqlx::query("SELECT * FROM profile_custom_sections WHERE entity_type = 'organization' AND entity_id = ? AND is_active = 1 ORDER BY display_order")
        .bind(&id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let sections: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "sectionName": r.try_get::<String, _>("section_name").unwrap_or_default(),
        "sectionType": r.try_get::<String, _>("section_type").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(sections)))
}

pub async fn create_org_custom_section(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateCustomSectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let section_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO profile_custom_sections (id, entity_type, entity_id, section_name, section_type, display_order, created_at, updated_at) VALUES (?, 'organization', ?, ?, ?, ?, ?, ?)")
        .bind(&section_id).bind(&id).bind(&body.section_name).bind(&body.section_type)
        .bind(body.display_order.unwrap_or(0)).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": section_id, "createdAt": now })))
}

pub async fn get_org_custom_section(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let row = sqlx::query("SELECT * FROM profile_custom_sections WHERE id = ? AND entity_type = 'organization' AND entity_id = ?")
        .bind(&section_id).bind(&id).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let Some(row) = row else { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Section not found" })))); };
    Ok(Json(serde_json::json!({
        "id": row.try_get::<String, _>("id").unwrap_or_default(),
        "sectionName": row.try_get::<String, _>("section_name").unwrap_or_default(),
    })))
}

pub async fn update_org_custom_section(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_string()];
    if let Some(name) = body.get("sectionName").and_then(|v| v.as_str()) {
        sets.push(format!("section_name = '{}'", name.replace('\'', "''")));
    }
    let query = format!("UPDATE profile_custom_sections SET {} WHERE id = ? AND entity_id = ?", sets.join(", "));
    sqlx::query(&query).bind(&section_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": section_id, "updatedAt": chrono::Utc::now().to_rfc3339() })))
}

pub async fn delete_org_custom_section(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    sqlx::query("DELETE FROM profile_custom_sections WHERE id = ? AND entity_id = ?")
        .bind(&section_id).bind(&id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn get_org_section_data(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let rows = sqlx::query("SELECT * FROM profile_custom_section_data WHERE section_id = ? ORDER BY display_order, row_key")
        .bind(&section_id).fetch_all(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let data: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "fieldKey": r.try_get::<String, _>("field_key").unwrap_or_default(),
        "fieldValue": r.try_get::<String, _>("field_value").unwrap_or_default(),
    })).collect();
    Ok(Json(serde_json::json!(data)))
}

pub async fn upsert_org_section_data(
    State(state): State<AppState>,
    Path((id, section_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpsertSectionDataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    let data_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO profile_custom_section_data (id, section_id, row_key, field_key, field_value, field_type, display_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&data_id).bind(&section_id)
        .bind(body.row_key.as_deref().unwrap_or(""))
        .bind(&body.field_key).bind(&body.field_value)
        .bind(body.field_type.as_deref().unwrap_or("text"))
        .bind(body.display_order.unwrap_or(0)).bind(&now).bind(&now)
        .execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "id": data_id, "createdAt": now })))
}

pub async fn delete_org_section_data(
    State(state): State<AppState>,
    Path((id, section_id, data_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let org = sqlx::query("SELECT o.id FROM organizations o WHERE o.id = ? AND o.tenant_key = ?")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if org.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" })))); }
    sqlx::query("DELETE FROM profile_custom_section_data WHERE id = ? AND section_id = ?")
        .bind(&data_id).bind(&section_id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// WhatsApp Channel Endpoints
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SendWhatsAppTestRequest {
    pub to: String,
    #[serde(default)]
    pub message_type: String,
    pub body: Option<String>,
    pub template_name: Option<String>,
    pub language: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateWhatsAppRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_value: Option<String>,
    pub action_type: String,
    pub action_config: Option<String>,
    pub priority: Option<i64>,
}

pub async fn whatsapp_health(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let brand_row = sqlx::query_as::<_, (String, String)>("SELECT b.id, b.name FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let brand_name = match &brand_row {
        Some((_, name)) => name.clone(),
        None => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))),
    };

    let creds: Option<(String,)> = sqlx::query_as(
        "SELECT credentials FROM brand_credentials WHERE brand_id = ?1 AND channel_type = 'whatsapp' AND is_active = 1 LIMIT 1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    let mut is_connected = false;
    let mut connection_error: Option<String> = None;
    let mut provider_info: Option<serde_json::Value> = None;

    if let Some((creds_json,)) = creds {
        if let Ok(creds_val) = serde_json::from_str::<serde_json::Value>(&creds_json) {
            let token = creds_val["access_token"].as_str().unwrap_or("");
            let phone_id = creds_val["phone_number_id"].as_str().unwrap_or("");
            let api_version = creds_val["api_version"].as_str().unwrap_or("v18.0");

            if !token.is_empty() && !phone_id.is_empty() {
                let client = reqwest::Client::new();
                let url = format!("https://graph.facebook.com/{}/{}", api_version, phone_id);
                match client.get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            is_connected = true;
                            if let Ok(body) = resp.text().await {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                    provider_info = Some(json.clone());
                                }
                            }
                        } else {
                            connection_error = Some(format!("Meta API returned {}", resp.status()));
                        }
                    }
                    Err(e) => {
                        connection_error = Some(format!("Connection failed: {}", e));
                    }
                }
            } else {
                connection_error = Some("Incomplete credentials".to_string());
            }
        }
    } else {
        if !state.config.meta_whatsapp_access_token.is_empty() && !state.config.meta_whatsapp_phone_number_id.is_empty() {
            connection_error = Some("Using global config — set brand-level credentials for best results".to_string());
            is_connected = true;
        } else {
            connection_error = Some("No WhatsApp credentials configured".to_string());
        }
    }

    let profile: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT verified_name, messaging_limit_tier, quality_rating FROM whatsapp_business_profile_cache WHERE brand_id = ?1 LIMIT 1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    Ok(Json(serde_json::json!({
        "brandId": id,
        "brandName": brand_name,
        "isConnected": is_connected,
        "connectionError": connection_error,
        "providerInfo": provider_info,
        "verifiedName": profile.as_ref().and_then(|p| p.0.clone()),
        "messagingLimitTier": profile.as_ref().and_then(|p| p.1.clone()),
        "qualityRating": profile.as_ref().and_then(|p| p.2.clone()),
    })))
}

pub async fn list_whatsapp_templates(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let rows = sqlx::query("SELECT * FROM whatsapp_templates WHERE brand_id = ?1 ORDER BY name")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    let templates: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "name": r.try_get::<String, _>("name").unwrap_or_default(),
        "category": r.try_get::<Option<String>, _>("category").unwrap_or(None),
        "language": r.try_get::<Option<String>, _>("language").unwrap_or(None),
        "status": r.try_get::<String, _>("status").unwrap_or_default(),
        "body": r.try_get::<String, _>("body").unwrap_or_default(),
        "lastSyncedAt": r.try_get::<Option<String>, _>("last_synced_at").unwrap_or(None),
    })).collect();

    Ok(Json(serde_json::json!(templates)))
}

pub async fn sync_whatsapp_templates(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let creds = resolve_whatsapp_creds(&state.db, &id).await?;
    let token = creds["access_token"].as_str().unwrap_or("");
    let phone_id = creds["phone_number_id"].as_str().unwrap_or("");
    let api_version = creds["api_version"].as_str().unwrap_or("v18.0");

    if token.is_empty() || phone_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "No WhatsApp credentials configured" }))));
    }

    let client = reqwest::Client::new();
    let url = format!("https://graph.facebook.com/{}/{}", api_version, phone_id);
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("fields", "name,verified_name,messaging_product_quality,messaging_limit,tier")])
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("API error: {}", e) }))))?;

    if !resp.status().is_success() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("Meta API returned {}", resp.status()) }))));
    }

    let body = resp.text().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let data: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|_| serde_json::json!({}));

    let now = chrono::Utc::now().to_rfc3339();
    let cache_id = gen_id();
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO whatsapp_business_profile_cache (id, brand_id, verified_name, messaging_limit_tier, quality_rating, updated_at, last_synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)"
    )
    .bind(&cache_id).bind(&id)
    .bind(data.get("verified_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("tier").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("messaging_product_quality").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(&now).bind(&now)
    .execute(&state.db).await;

    let waba_id = creds["waba_id"].as_str().unwrap_or("");
    let mut templates_synced = 0;
    if !waba_id.is_empty() {
        let templates_url = format!("https://graph.facebook.com/{}/{}", api_version, waba_id);
        let tresp = client.get(&templates_url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[("fields", "name,category,language,status,components")])
            .send().await;

        if let Ok(tresp) = tresp {
            if tresp.status().is_success() {
                let tbody = tresp.text().await.unwrap_or_default();
                if let Ok(tdata) = serde_json::from_str::<serde_json::Value>(&tbody) {
                    if let Some(arr) = tdata["data"].as_array() {
                        for tmpl in arr {
                            let name = tmpl["name"].as_str().unwrap_or("");
                            let category = tmpl["category"].as_str().unwrap_or("");
                            let language = tmpl["language"].as_str().unwrap_or("");
                            let status = tmpl["status"].as_str().unwrap_or("approved");
                            let body_text = tmpl["components"].as_array().and_then(|c| {
                                c.iter().find(|comp| comp["type"] == "BODY").and_then(|b| b["text"].as_str())
                            }).unwrap_or("");
                            let components = serde_json::to_string(&tmpl["components"]).unwrap_or_default();

                            let tmpl_id = gen_id();
                            let _ = sqlx::query(
                                "INSERT OR REPLACE INTO whatsapp_templates (id, brand_id, name, category, language, status, body, components, updated_at, last_synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)"
                            )
                            .bind(&tmpl_id).bind(&id).bind(name).bind(category).bind(language)
                            .bind(status).bind(body_text).bind(&components).bind(&now).bind(&now)
                            .execute(&state.db).await;
                            templates_synced += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "templatesSynced": templates_synced,
        "profileSynced": true,
        "syncedAt": now,
    })))
}

pub async fn sync_whatsapp_profile_now(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let creds = resolve_whatsapp_creds(&state.db, &id).await?;
    let token = creds["access_token"].as_str().unwrap_or("");
    let phone_id = creds["phone_number_id"].as_str().unwrap_or("");
    let api_version = creds["api_version"].as_str().unwrap_or("v18.0");

    if token.is_empty() || phone_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "No WhatsApp credentials configured" }))));
    }

    let client = reqwest::Client::new();
    let url = format!("https://graph.facebook.com/{}/{}", api_version, phone_id);
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("fields", "name,about,address,description,email,websites,vertical,profile_picture_url,verified_name,messaging_product_quality,messaging_limit,tier,account_mode")])
        .send().await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("API error: {}", e) }))))?;

    if !resp.status().is_success() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("Meta API returned {}", resp.status()) }))));
    }

    let body = resp.text().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let data: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|_| serde_json::json!({}));

    let now = chrono::Utc::now().to_rfc3339();
    let cache_id = gen_id();
    let websites = data.get("websites").and_then(|w| w.as_array()).map(|w| {
        w.iter().filter_map(|ws| ws["url"].as_str()).collect::<Vec<_>>().join(", ")
    });

    let _ = sqlx::query(
        "INSERT OR REPLACE INTO whatsapp_business_profile_cache (id, brand_id, verified_name, about, address, description, email, websites, vertical, profile_picture_url, messaging_limit_tier, quality_rating, account_mode, updated_at, last_synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)"
    )
    .bind(&cache_id).bind(&id)
    .bind(data.get("verified_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("about").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("address").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("email").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(websites)
    .bind(data.get("vertical").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("profile_picture_url").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("tier").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("messaging_product_quality").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(data.get("account_mode").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(&now).bind(&now)
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({
        "synced": true,
        "verifiedName": data.get("verified_name").and_then(|v| v.as_str()),
        "messagingLimitTier": data.get("tier").and_then(|v| v.as_str()),
        "qualityRating": data.get("messaging_product_quality").and_then(|v| v.as_str()),
        "syncedAt": now,
    })))
}

pub async fn list_whatsapp_customers(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let rows = sqlx::query("SELECT * FROM whatsapp_customer_profiles WHERE brand_id = ?1 ORDER BY last_fetched_at DESC")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    let customers: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "waId": r.try_get::<String, _>("wa_id").unwrap_or_default(),
        "profileName": r.try_get::<Option<String>, _>("profile_name").unwrap_or(None),
        "profilePictureUrl": r.try_get::<Option<String>, _>("profile_picture_url").unwrap_or(None),
        "about": r.try_get::<Option<String>, _>("about").unwrap_or(None),
        "lastFetchedAt": r.try_get::<Option<String>, _>("last_fetched_at").unwrap_or(None),
    })).collect();

    Ok(Json(serde_json::json!(customers)))
}

pub async fn send_whatsapp_test(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SendWhatsAppTestRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let creds = resolve_whatsapp_creds(&state.db, &id).await?;
    let token = creds["access_token"].as_str().unwrap_or("");
    let phone_id = creds["phone_number_id"].as_str().unwrap_or("");
    let api_version = creds["api_version"].as_str().unwrap_or("v18.0");

    if token.is_empty() || phone_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "No WhatsApp credentials configured" }))));
    }

    let client = reqwest::Client::new();
    let url = format!("https://graph.facebook.com/{}/{}", api_version, phone_id);

    let message_type = if body.message_type.is_empty() { "text".to_string() } else { body.message_type.clone() };

    let payload = match message_type.as_str() {
        "template" => {
            let name = body.template_name.as_deref().unwrap_or("");
            let lang = body.language.as_deref().unwrap_or("en_US");
            serde_json::json!({
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": body.to.trim_start_matches('+'),
                "type": "template",
                "template": { "name": name, "language": { "code": lang } }
            })
        }
        _ => {
            serde_json::json!({
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": body.to.trim_start_matches('+'),
                "type": "text",
                "text": { "body": body.body.as_deref().unwrap_or("Test message"), "preview_url": false }
            })
        }
    };

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send().await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("API error: {}", e) }))))?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Meta API returned {}: {}", status, body_text)
        }))));
    }

    let result: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_else(|_| serde_json::json!({}));
    let msg_id = result["messages"][0]["id"].as_str().unwrap_or("");

    Ok(Json(serde_json::json!({
        "success": true,
        "messageId": msg_id,
        "messageType": message_type,
    })))
}

pub async fn list_whatsapp_rules(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let rows = sqlx::query("SELECT * FROM whatsapp_automation_rules WHERE brand_id = ?1 ORDER BY priority DESC")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    let rules: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "name": r.try_get::<String, _>("name").unwrap_or_default(),
        "triggerType": r.try_get::<String, _>("trigger_type").unwrap_or_default(),
        "triggerValue": r.try_get::<Option<String>, _>("trigger_value").unwrap_or(None),
        "actionType": r.try_get::<String, _>("action_type").unwrap_or_default(),
        "actionConfig": r.try_get::<Option<String>, _>("action_config").unwrap_or(None),
        "isActive": r.try_get::<bool, _>("is_active").unwrap_or(true),
        "priority": r.try_get::<i64, _>("priority").unwrap_or(0),
    })).collect();

    Ok(Json(serde_json::json!(rules)))
}

pub async fn create_whatsapp_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateWhatsAppRuleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let rule_id = gen_id();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO whatsapp_automation_rules (id, brand_id, name, description, trigger_type, trigger_value, action_type, action_config, is_active, priority, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?10)"
    )
    .bind(&rule_id).bind(&id).bind(&body.name).bind(body.description.as_deref())
    .bind(&body.trigger_type).bind(body.trigger_value.as_deref())
    .bind(&body.action_type).bind(body.action_config.as_deref())
    .bind(body.priority.unwrap_or(0)).bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    Ok(Json(serde_json::json!({ "id": rule_id, "createdAt": now })))
}

pub async fn update_whatsapp_rule(
    State(state): State<AppState>,
    Path((id, rule_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateWhatsAppRuleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE whatsapp_automation_rules SET name = ?1, description = ?2, trigger_type = ?3, trigger_value = ?4, action_type = ?5, action_config = ?6, priority = ?7, updated_at = ?8 WHERE id = ?9 AND brand_id = ?10"
    )
    .bind(&body.name).bind(body.description.as_deref()).bind(&body.trigger_type)
    .bind(body.trigger_value.as_deref()).bind(&body.action_type).bind(body.action_config.as_deref())
    .bind(body.priority.unwrap_or(0)).bind(&now).bind(&rule_id).bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    Ok(Json(serde_json::json!({ "id": rule_id, "updatedAt": now })))
}

pub async fn delete_whatsapp_rule(
    State(state): State<AppState>,
    Path((id, rule_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    sqlx::query("DELETE FROM whatsapp_automation_rules WHERE id = ?1 AND brand_id = ?2")
        .bind(&rule_id).bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn run_whatsapp_nudges(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = sqlx::query("SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2")
        .bind(&id).bind(&tenant_key).fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    if _brand.is_none() { return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Brand not found" })))); }

    let now = chrono::Utc::now().to_rfc3339();
    let expired: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, contact_id FROM whatsapp_session_windows WHERE brand_id = ?1 AND is_active = 1 AND expires_at < ?2"
    )
    .bind(&id).bind(&now)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    let mut count = 0;
    for (_id, contact_id) in &expired {
        let _ = sqlx::query(
            "UPDATE whatsapp_session_windows SET is_active = 0 WHERE brand_id = ?1 AND contact_id = ?2"
        )
        .bind(&id).bind(contact_id)
        .execute(&state.db).await;
        count += 1;
    }

    Ok(Json(serde_json::json!({
        "nudgesSent": count,
        "runAt": now,
    })))
}

// ---------------------------------------------------------------------------
// WhatsApp Campaigns, Flows, Industries — Handlers
// ---------------------------------------------------------------------------

// --- Industries ---

pub async fn list_industries(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.list_industries().await {
        Ok(industries) => Ok(Json(serde_json::json!({ "industries": industries }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct CreateIndustryRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

pub async fn create_industry(
    State(state): State<AppState>,
    Json(body): Json<CreateIndustryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .create_industry(&body.name, body.description.as_deref().unwrap_or(""), body.icon.as_deref())
        .await
    {
        Ok(industry) => Ok(Json(serde_json::json!(industry))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct UpdateIndustryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn update_industry(
    State(state): State<AppState>,
    Path(industry_id): Path<String>,
    Json(body): Json<UpdateIndustryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .update_industry(
            &industry_id,
            body.name.as_deref(),
            body.description.as_deref(),
            body.icon.as_deref(),
            body.is_active,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({ "updated": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn delete_industry(
    State(state): State<AppState>,
    Path(industry_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.delete_industry(&industry_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- Template Groups ---

pub async fn list_template_groups(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    let industry_id = params.get("industry_id").map(|s| s.as_str());
    match svc.list_template_groups(&id, industry_id).await {
        Ok(groups) => Ok(Json(serde_json::json!({ "groups": groups }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct CreateTemplateGroupRequest {
    pub industry_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

pub async fn create_template_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateTemplateGroupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .create_template_group(
            &id,
            body.industry_id.as_deref(),
            &body.name,
            body.description.as_deref().unwrap_or(""),
            body.sort_order.unwrap_or(0),
        )
        .await
    {
        Ok(group) => Ok(Json(serde_json::json!(group))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct UpdateTemplateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

pub async fn update_template_group(
    State(state): State<AppState>,
    Path((id, group_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdateTemplateGroupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .update_template_group(
            &group_id,
            body.name.as_deref(),
            body.description.as_deref(),
            body.sort_order,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({ "updated": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn delete_template_group(
    State(state): State<AppState>,
    Path((id, group_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.delete_template_group(&group_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- Template Flows ---

pub async fn list_flows(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    let industry_id = params.get("industry_id").map(|s| s.as_str());
    let group_id = params.get("group_id").map(|s| s.as_str());
    let category = params.get("category").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());
    match svc
        .list_template_flows(&id, industry_id, group_id, category, status)
        .await
    {
        Ok(flows) => Ok(Json(serde_json::json!({ "flows": flows }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
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

pub async fn create_flow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .create_template_flow(
            &id,
            body.group_id.as_deref(),
            body.industry_id.as_deref(),
            &body.name,
            body.flow_type.as_deref().unwrap_or("template"),
            body.category.as_deref().unwrap_or("general"),
            &body.body,
            body.header.as_deref(),
            body.footer.as_deref(),
            body.buttons.as_deref(),
            body.sections.as_deref(),
            body.language.as_deref().unwrap_or("en_US"),
        )
        .await
    {
        Ok(flow) => Ok(Json(serde_json::json!(flow))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn get_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.get_template_flow(&flow_id).await {
        Ok(flow_with_steps) => Ok(Json(serde_json::json!(flow_with_steps))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
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

pub async fn update_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<UpdateFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .update_template_flow(
            &flow_id,
            body.name.as_deref(),
            body.body.as_deref(),
            body.header.as_deref(),
            body.footer.as_deref(),
            body.buttons.as_deref(),
            body.sections.as_deref(),
            body.status.as_deref(),
            body.is_active,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({ "updated": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn delete_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.delete_template_flow(&flow_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct DuplicateFlowRequest {
    pub name: String,
}

pub async fn duplicate_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<DuplicateFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.duplicate_flow(&flow_id, &body.name).await {
        Ok(new_id) => Ok(Json(serde_json::json!({ "id": new_id }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn validate_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.validate_flow(&flow_id).await {
        Ok(errors) => {
            if errors.is_empty() {
                Ok(Json(serde_json::json!({ "valid": true })))
            } else {
                Ok(Json(serde_json::json!({ "valid": false, "errors": errors })))
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- Flow Steps ---

pub async fn list_flow_steps(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.list_flow_steps(&flow_id).await {
        Ok(steps) => Ok(Json(serde_json::json!({ "steps": steps }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct CreateFlowStepRequest {
    pub step_number: i32,
    pub condition: Option<String>,
    pub action_type: String,
    pub action_config: Option<String>,
    pub next_step_id: Option<String>,
}

pub async fn create_flow_step(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<CreateFlowStepRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .create_flow_step(
            &flow_id,
            body.step_number,
            body.condition.as_deref(),
            &body.action_type,
            body.action_config.as_deref(),
            body.next_step_id.as_deref(),
        )
        .await
    {
        Ok(step) => Ok(Json(serde_json::json!(step))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct UpdateFlowStepRequest {
    pub step_number: Option<i32>,
    pub condition: Option<String>,
    pub action_type: Option<String>,
    pub action_config: Option<String>,
    pub next_step_id: Option<String>,
}

pub async fn update_flow_step(
    State(state): State<AppState>,
    Path((flow_id, step_id)): Path<(String, String)>,
    Json(body): Json<UpdateFlowStepRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .update_flow_step(
            &step_id,
            body.step_number,
            body.condition.as_deref(),
            body.action_type.as_deref(),
            body.action_config.as_deref(),
            body.next_step_id.as_deref(),
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({ "updated": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn delete_flow_step(
    State(state): State<AppState>,
    Path((flow_id, step_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _flow_id = flow_id;
    let svc = crate::services::whatsapp_flows::WhatsAppFlowsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.delete_flow_step(&step_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- Campaigns ---

pub async fn list_campaigns(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    let status = params.get("status").map(|s| s.as_str());
    match svc.list_campaigns(&id, status).await {
        Ok(campaigns) => Ok(Json(serde_json::json!({ "campaigns": campaigns }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub campaign_type: Option<String>,
    pub template_flow_id: Option<String>,
    pub scheduled_at: Option<String>,
}

pub async fn create_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateCampaignRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .create_campaign(
            &id,
            &body.name,
            body.campaign_type.as_deref().unwrap_or("whatsapp"),
            body.template_flow_id.as_deref(),
            body.scheduled_at.as_deref(),
        )
        .await
    {
        Ok(campaign) => Ok(Json(serde_json::json!(campaign))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn get_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.get_campaign(&campaign_id).await {
        Ok(campaign) => Ok(Json(serde_json::json!(campaign))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn delete_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.delete_campaign(&campaign_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn start_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.start_campaign(&campaign_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "status": "running" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn pause_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.pause_campaign(&campaign_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "status": "paused" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn resume_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.resume_campaign(&campaign_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "status": "running" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn cancel_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.cancel_campaign(&campaign_id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "status": "cancelled" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn get_campaign_contacts(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.get_campaign_contacts(&campaign_id).await {
        Ok(contacts) => Ok(Json(serde_json::json!({ "contacts": contacts }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[derive(Deserialize)]
pub struct AddCampaignContactsRequest {
    pub contact_ids: Vec<String>,
}

pub async fn add_campaign_contacts(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
    Json(body): Json<AddCampaignContactsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc
        .add_contacts_to_campaign(&campaign_id, &body.contact_ids)
        .await
    {
        Ok(count) => Ok(Json(serde_json::json!({ "added": count }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn retry_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.retry_failed_messages(&campaign_id).await {
        Ok(count) => Ok(Json(serde_json::json!({ "retried": count }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn get_campaign_analytics(
    State(state): State<AppState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let svc = crate::services::whatsapp_campaigns::WhatsAppCampaignsService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.get_campaign_analytics(&campaign_id).await {
        Ok(analytics) => Ok(Json(serde_json::json!(analytics))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- Message Limits ---

pub async fn get_message_limits(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_template_sync::TemplateSyncService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.get_remaining_limits(&id).await {
        Ok(limits) => Ok(Json(limits)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn sync_message_limits(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_template_sync::TemplateSyncService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.update_message_limits_in_db(&id).await {
        Ok(result) => Ok(Json(serde_json::json!(result))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- Template Sync ---

pub async fn sync_templates_now(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    let svc = crate::services::whatsapp_template_sync::TemplateSyncService::new(
        state.db.clone(),
        state.config.as_ref().clone(),
    );
    match svc.sync_templates_from_meta(&id).await {
        Ok(result) => Ok(Json(serde_json::json!(result))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// --- AI Campaign ---

#[derive(Deserialize)]
pub struct AISuggestRequest {
    pub industry: Option<String>,
    pub audience_size: Option<i64>,
}

pub async fn ai_suggest_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AISuggestRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    // Get brand profile for context
    let brand_name: Option<String> = sqlx::query_scalar("SELECT name FROM brands WHERE id = ?1 LIMIT 1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let industry = body.industry.as_deref().unwrap_or("general");
    let audience = body.audience_size.unwrap_or(100);

    // Generate suggestions based on brand data
    let suggestions = vec![
        serde_json::json!({
            "title": "Welcome Series",
            "description": "Send a welcome message to new contacts with brand introduction",
            "type": "whatsapp",
            "category": "onboarding",
            "estimated_contacts": audience,
            "recommended_template": "welcome_message",
        }),
        serde_json::json!({
            "title": "Promotional Campaign",
            "description": format!("Promote latest offers to your {industry} audience"),
            "type": "whatsapp",
            "category": "marketing",
            "estimated_contacts": audience / 2,
            "recommended_template": "promotional_offer",
        }),
        serde_json::json!({
            "title": "Follow-up Sequence",
            "description": "Re-engage inactive customers with personalized follow-ups",
            "type": "whatsapp",
            "category": "re-engagement",
            "estimated_contacts": audience / 4,
            "recommended_template": "follow_up",
        }),
    ];

    Ok(Json(serde_json::json!({
        "brand": brand_name,
        "industry": industry,
        "suggestions": suggestions,
    })))
}

#[derive(Deserialize)]
pub struct AIGenerateRequest {
    pub campaign_type: Option<String>,
    pub audience: Option<String>,
    pub goal: Option<String>,
}

pub async fn ai_generate_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AIGenerateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tenant_key = resolve_tenant_key(&headers).await?;
    let _brand = verify_brand(&state, &id, &tenant_key).await?;

    // Get brand profile for context
    let brand_name: Option<String> = sqlx::query_scalar("SELECT name FROM brands WHERE id = ?1 LIMIT 1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
    let brand_bio: Option<String> = sqlx::query_scalar("SELECT bio FROM brands WHERE id = ?1 LIMIT 1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let campaign_type = body.campaign_type.as_deref().unwrap_or("promotional");
    let audience = body.audience.as_deref().unwrap_or("existing customers");
    let goal = body.goal.as_deref().unwrap_or("increase engagement");

    // Try AI generation via reply_engine, fall back to template
    let ai_body = match crate::services::reply_engine::generate_reply(
        &state,
        "",
        Some(&format!(
            "Generate a WhatsApp campaign message. Brand: {}, Bio: {}, Type: {}, Audience: {}, Goal: {}",
            brand_name.as_deref().unwrap_or("Our Brand"),
            brand_bio.as_deref().unwrap_or(""),
            campaign_type,
            audience,
            goal
        )),
    )
    .await
    {
        Ok(draft) if !draft.body.is_empty() => draft.body,
        _ => format!(
            "Hello from {}! We have something special for you. Check out our latest offers and updates. Reply to learn more!",
            brand_name.as_deref().unwrap_or("our team")
        ),
    };

    Ok(Json(serde_json::json!({
        "brand": brand_name,
        "campaignType": campaign_type,
        "audience": audience,
        "goal": goal,
        "generatedContent": {
            "body": ai_body,
            "header": format!("🎉 Special Offer from {}", brand_name.as_deref().unwrap_or("Us")),
            "footer": "Reply STOP to unsubscribe",
            "buttons": [
                {"type": "quick_reply", "text": "🛒 Shop Now"},
                {"type": "quick_reply", "text": "ℹ️ Learn More"},
            ]
        }
    })))
}

// --- Brand verification helper ---

async fn verify_brand(
    state: &AppState,
    brand_id: &str,
    tenant_key: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let brand_id_db: Option<String> = sqlx::query_scalar(
        "SELECT b.id FROM brands b JOIN organizations o ON b.organization_id = o.id WHERE b.id = ?1 AND o.tenant_key = ?2"
    )
    .bind(brand_id)
    .bind(tenant_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    match brand_id_db {
        Some(id) => Ok(id),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Brand not found" })),
        )),
    }
}

// ---------------------------------------------------------------------------
// WhatsApp helpers
// ---------------------------------------------------------------------------

async fn resolve_whatsapp_creds(
    db: &sqlx::SqlitePool,
    brand_id: &str,
) -> Result<serde_json::Value, (StatusCode, Json<serde_json::Value>)> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT credentials FROM brand_credentials WHERE brand_id = ?1 AND channel_type = 'whatsapp' AND is_active = 1 LIMIT 1"
    )
    .bind(brand_id)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    if let Some((creds_json,)) = row {
        if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&creds_json) {
            return Ok(creds);
        }
    }

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT credentials FROM integration_settings WHERE key = 'whatsapp_meta' LIMIT 1"
    )
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;

    if let Some((creds_json,)) = row {
        if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&creds_json) {
            return Ok(creds);
        }
    }

    Ok(serde_json::json!({}))
}
