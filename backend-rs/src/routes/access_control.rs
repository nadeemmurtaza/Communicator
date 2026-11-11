use crate::db::DbPool;
use crate::services::access_control::*;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post, put};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Deserialize)]
struct AccessGrantsQuery {
    tenant_id: Uuid,
    org_id: Option<Uuid>,
    brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct MyPermissionsQuery {
    team_member_id: Uuid,
    tenant_id: Uuid,
    org_id: Option<Uuid>,
    brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct InvitationsQuery {
    tenant_id: Uuid,
    status: Option<String>,
}

#[derive(Deserialize)]
struct AcceptInvitationQuery {
    team_member_id: Uuid,
}

// ============================================================================
// Route Handlers
// ============================================================================

// Layer 1: Access Grants

pub async fn list_access_grants(
    State(pool): State<DbPool>,
    Query(query): Query<AccessGrantsQuery>,
) -> Result<Json<Vec<AccessGrant>>, StatusCode> {
    let grants = AccessControlService::list_access_grants(&pool, query.tenant_id, query.org_id, query.brand_id).await?;
    Ok(Json(grants))
}

pub async fn create_access_grant(
    State(pool): State<DbPool>,
    Query(query): Query<AccessGrantsQuery>,
    Json(req): Json<CreateAccessGrantRequest>,
) -> Result<Json<AccessGrant>, StatusCode> {
    // Use the requester as granted_by (simplified — in production, extract from auth)
    let granted_by = req.team_member_id;
    let grant = AccessControlService::create_access_grant(&pool, query.tenant_id, &req, granted_by).await?;
    Ok(Json(grant))
}

pub async fn update_access_grant(
    State(pool): State<DbPool>,
    Path((_tenant_id, grant_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateAccessGrantRequest>,
) -> Result<Json<AccessGrant>, StatusCode> {
    let grant = AccessControlService::update_access_grant(&pool, grant_id, &req).await?;
    Ok(Json(grant))
}

pub async fn revoke_access_grant(
    State(pool): State<DbPool>,
    Path((_tenant_id, grant_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    AccessControlService::revoke_access_grant(&pool, grant_id).await?;
    Ok(Json(serde_json::json!({ "message": "Access grant revoked" })))
}

pub async fn transfer_ownership(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<Json<AccessGrant>, StatusCode> {
    let grant = AccessControlService::transfer_ownership(&pool, tenant_id, &req).await?;
    Ok(Json(grant))
}

pub async fn add_partner(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<AddPartnerRequest>,
) -> Result<Json<AccessGrant>, StatusCode> {
    let granted_by = req.team_member_id;
    let grant = AccessControlService::add_partner(&pool, tenant_id, &req, granted_by).await?;
    Ok(Json(grant))
}

pub async fn get_my_permissions(
    State(pool): State<DbPool>,
    Query(query): Query<MyPermissionsQuery>,
) -> Result<Json<EffectivePermissions>, StatusCode> {
    let perms = AccessControlService::resolve_effective_permissions(
        &pool,
        query.team_member_id,
        query.tenant_id,
        query.org_id,
        query.brand_id,
    ).await?;
    Ok(Json(perms))
}

// Layer 2: Departments

pub async fn list_departments(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<Department>>, StatusCode> {
    let depts = AccessControlService::list_departments(&pool, tenant_id).await?;
    Ok(Json(depts))
}

pub async fn create_department(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<Department>, StatusCode> {
    let dept = AccessControlService::create_department(&pool, tenant_id, &req).await?;
    Ok(Json(dept))
}

pub async fn update_department(
    State(pool): State<DbPool>,
    Path((tenant_id, dept_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<Department>, StatusCode> {
    let dept = AccessControlService::update_department(&pool, dept_id, &req).await?;
    Ok(Json(dept))
}

pub async fn assign_to_department(
    State(pool): State<DbPool>,
    Path((tenant_id, dept_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AssignDepartmentRequest>,
) -> Result<Json<DepartmentAssignment>, StatusCode> {
    let assignment = AccessControlService::assign_to_department(&pool, dept_id, &req).await?;
    Ok(Json(assignment))
}

pub async fn remove_department_assignment(
    State(pool): State<DbPool>,
    Path((tenant_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    AccessControlService::remove_department_assignment(&pool, assignment_id).await?;
    Ok(Json(serde_json::json!({ "message": "Department assignment removed" })))
}

// Layer 3: Roles & Permissions

pub async fn get_permission_catalog(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<PermissionCatalogGroup>>, StatusCode> {
    let catalog = AccessControlService::get_permission_catalog(&pool).await?;
    Ok(Json(catalog))
}

pub async fn get_all_permissions(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<PermissionCatalogEntry>>, StatusCode> {
    let perms = AccessControlService::get_all_permissions(&pool).await?;
    Ok(Json(perms))
}

pub async fn get_permissions_by_group(
    State(pool): State<DbPool>,
    Path(group_name): Path<String>,
) -> Result<Json<Vec<PermissionCatalogEntry>>, StatusCode> {
    let perms = AccessControlService::get_permissions_by_group(&pool, &group_name).await?;
    Ok(Json(perms))
}

// Invitations

pub async fn list_invitations(
    State(pool): State<DbPool>,
    Query(query): Query<InvitationsQuery>,
) -> Result<Json<Vec<Invitation>>, StatusCode> {
    let invitations = AccessControlService::list_invitations(&pool, query.tenant_id, query.status.as_deref()).await?;
    Ok(Json(invitations))
}

pub async fn send_email_invitation(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<SendEmailInvitationRequest>,
) -> Result<Json<Invitation>, StatusCode> {
    let invited_by = Uuid::nil(); // Simplified — extract from auth in production
    let invitation = AccessControlService::send_email_invitation(&pool, tenant_id, &req, invited_by).await?;
    Ok(Json(invitation))
}

pub async fn send_username_invitation(
    State(pool): State<DbPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<SendUsernameInvitationRequest>,
) -> Result<Json<Invitation>, StatusCode> {
    let invited_by = Uuid::nil(); // Simplified — extract from auth in production
    let invitation = AccessControlService::send_username_invitation(&pool, tenant_id, &req, invited_by).await?;
    Ok(Json(invitation))
}

pub async fn accept_invitation(
    State(pool): State<DbPool>,
    Path(invitation_id): Path<Uuid>,
    Query(query): Query<AcceptInvitationQuery>,
) -> Result<Json<Invitation>, StatusCode> {
    let invitation = AccessControlService::accept_invitation(&pool, invitation_id, query.team_member_id).await?;
    Ok(Json(invitation))
}

pub async fn revoke_invitation(
    State(pool): State<DbPool>,
    Path((tenant_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    AccessControlService::revoke_invitation(&pool, invitation_id).await?;
    Ok(Json(serde_json::json!({ "message": "Invitation revoked" })))
}

// Webhook Handlers

pub async fn handle_nid_access_webhook(
    State(pool): State<DbPool>,
    Json(webhook): Json<NidAccessWebhook>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    AccessControlService::handle_nid_access_webhook(&pool, &webhook).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn handle_nid_department_webhook(
    State(pool): State<DbPool>,
    Json(webhook): Json<NidDepartmentWebhook>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    AccessControlService::handle_nid_department_webhook(&pool, &webhook).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn handle_nid_role_webhook(
    State(pool): State<DbPool>,
    Json(webhook): Json<NidRoleWebhook>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    AccessControlService::handle_nid_role_webhook(&pool, &webhook).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn router() -> axum::Router<DbPool> {
    axum::Router::new()
        // Access Grants
        .route("/api/v1/access/grants", get(list_access_grants))
        .route("/api/v1/access/grant", post(create_access_grant))
        .route("/api/v1/access/:tenant_id/grant/:grant_id", patch(update_access_grant))
        .route("/api/v1/access/:tenant_id/grant/:grant_id", delete(revoke_access_grant))
        .route("/api/v1/access/:tenant_id/transfer-ownership", post(transfer_ownership))
        .route("/api/v1/access/:tenant_id/partner", post(add_partner))
        .route("/api/v1/access/my-permissions", get(get_my_permissions))
        
        // Departments
        .route("/api/v1/departments/:tenant_id", get(list_departments))
        .route("/api/v1/departments/:tenant_id", post(create_department))
        .route("/api/v1/departments/:tenant_id/:dept_id", patch(update_department))
        .route("/api/v1/departments/:tenant_id/:dept_id/assign", post(assign_to_department))
        .route("/api/v1/departments/:tenant_id/assignment/:assignment_id", delete(remove_department_assignment))
        
        // Permission Catalog
        .route("/api/v1/permissions/catalog", get(get_permission_catalog))
        .route("/api/v1/permissions/all", get(get_all_permissions))
        .route("/api/v1/permissions/group/:group_name", get(get_permissions_by_group))
        
        // Invitations
        .route("/api/v1/invitations/:tenant_id", get(list_invitations))
        .route("/api/v1/invitations/:tenant_id/email", post(send_email_invitation))
        .route("/api/v1/invitations/:tenant_id/username", post(send_username_invitation))
        .route("/api/v1/invitations/:invitation_id/accept", post(accept_invitation))
        .route("/api/v1/invitations/:tenant_id/:invitation_id", delete(revoke_invitation))
        
        // nID Webhooks
        .route("/api/v1/nid/access/webhook", post(handle_nid_access_webhook))
        .route("/api/v1/nid/department/webhook", post(handle_nid_department_webhook))
        .route("/api/v1/nid/role/webhook", post(handle_nid_role_webhook))
}
