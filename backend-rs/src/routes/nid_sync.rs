use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::db::AppState;
use crate::services::nid_sync::NidSyncService;

#[derive(Debug, Deserialize)]
pub struct AccountQuery {
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    pub org_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SetPlatformRequest {
    pub order_id: String,
    pub platform_name: String,
    pub platform_type: String,
    pub brand_name: String,
    pub shop_name: Option<String>,
}

// ---- Launchpad ----

pub async fn get_launchpad_overview(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<crate::services::nid_sync::LaunchpadOverview>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_launchpad_overview(&query.account_id).await {
        Ok(overview) => Ok(Json(overview)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Personal Profile ----

pub async fn get_personal_profile(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::services::nid_sync::PersonalProfileOverview>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_personal_profile_overview(&account_id).await {
        Ok(overview) => Ok(Json(overview)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Work Profile ----

pub async fn get_work_profile(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<String>,
) -> Result<Json<crate::services::nid_sync::WorkProfileOverview>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_work_profile_overview(&org_id).await {
        Ok(overview) => Ok(Json(overview)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Organizations ----

pub async fn get_user_organizations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<Vec<crate::services::nid_sync::NidOrganization>>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_user_organizations(&query.account_id).await {
        Ok(orgs) => Ok(Json(orgs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Memberships ----

pub async fn get_user_memberships(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<Vec<crate::services::nid_sync::NidMembership>>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_user_memberships(&query.account_id).await {
        Ok(memberships) => Ok(Json(memberships)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Invoices ----

pub async fn get_invoices(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<crate::services::nid_sync::NidInvoice>>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_invoices_by_tenant(&query.tenant_id).await {
        Ok(invoices) => Ok(Json(invoices)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Payments ----

pub async fn get_payments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<crate::services::nid_sync::NidPayment>>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_payments_by_tenant(&query.tenant_id).await {
        Ok(payments) => Ok(Json(payments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Subscriptions ----

pub async fn get_subscriptions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<crate::services::nid_sync::NidSubscription>>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_subscriptions_by_tenant(&query.tenant_id).await {
        Ok(subs) => Ok(Json(subs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Order Platform Info ----

pub async fn set_order_platform_info(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetPlatformRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.set_order_platform_info(
        &req.order_id,
        &req.platform_name,
        &req.platform_type,
        &req.brand_name,
        req.shop_name.as_deref(),
    )
    .await
    {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ---- Tenants ----

pub async fn get_tenants(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::services::nid_sync::NidTenant>>, (StatusCode, String)> {
    let service = NidSyncService::new(state.db.clone());
    match service.get_tenants().await {
        Ok(tenants) => Ok(Json(tenants)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
