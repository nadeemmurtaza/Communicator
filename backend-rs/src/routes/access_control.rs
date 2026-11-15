//! Thin gRPC client forwarding to svc-access-control (port 50093).
//!
//! Replaces the in-process `AccessControlService` calls with tonic gRPC
//! calls. The REST contract is preserved byte-for-byte.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use axum::Json;
use newax_proto::access_control::v1::{
    access_control_client::AccessControlClient, *,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::AppState;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Deserialize)]
struct AccessGrantsQuery {
    tenant_id: String,
    org_id: Option<Uuid>,
    brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct MyPermissionsQuery {
    team_member_id: String,
    tenant_id: String,
    org_id: Option<Uuid>,
    brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct InvitationsQuery {
    tenant_id: String,
    status: Option<String>,
}

#[derive(Deserialize)]
struct AcceptInvitationQuery {
    team_member_id: String,
}

// ============================================================================
// Client helper
// ============================================================================

async fn client(state: &AppState) -> Result<AccessControlClient<tonic::transport::Channel>, StatusCode> {
    let url = if state.config.access_control_grpc_url.is_empty() {
        "http://127.0.0.1:50093".to_string()
    } else {
        state.config.access_control_grpc_url.clone()
    };
    AccessControlClient::connect(url)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

fn parse_response(payload: String, error: String) -> Result<Json<serde_json::Value>, StatusCode> {
    if !error.is_empty() {
        return Ok(Json(serde_json::json!({ "error": error })));
    }
    serde_json::from_str(&payload).map(Json).map_err(|_| StatusCode::BAD_GATEWAY)
}

// ============================================================================
// Route Handlers
// ============================================================================

// Layer 1: Access Grants

pub async fn list_access_grants(
    State(state): State<AppState>,
    Query(query): Query<AccessGrantsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .list_access_grants(ListAccessGrantsRequest {
            tenant_id: query.tenant_id,
            org_id: query.org_id.map(|u| u.to_string()).unwrap_or_default(),
            brand_id: query.brand_id.map(|u| u.to_string()).unwrap_or_default(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn create_access_grant(
    State(state): State<AppState>,
    Query(query): Query<AccessGrantsQuery>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .create_access_grant(CreateAccessGrantRequest {
            tenant_id: query.tenant_id,
            org_id: query.org_id.map(|u| u.to_string()).unwrap_or_default(),
            brand_id: query.brand_id.map(|u| u.to_string()).unwrap_or_default(),
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn update_access_grant(
    State(state): State<AppState>,
    Path((_tenant_id, grant_id)): Path<(String, Uuid)>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .update_access_grant(UpdateAccessGrantRequest {
            tenant_id: _tenant_id,
            grant_id: grant_id.to_string(),
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn revoke_access_grant(
    State(state): State<AppState>,
    Path((_tenant_id, grant_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .revoke_access_grant(RevokeAccessGrantRequest {
            tenant_id: _tenant_id,
            grant_id: grant_id.to_string(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn transfer_ownership(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .transfer_ownership(TransferOwnershipRequest {
            tenant_id,
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn add_partner(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .add_partner(AddPartnerRequest {
            tenant_id,
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn get_my_permissions(
    State(state): State<AppState>,
    Query(query): Query<MyPermissionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .get_my_permissions(GetMyPermissionsRequest {
            team_member_id: query.team_member_id,
            tenant_id: query.tenant_id,
            org_id: query.org_id.map(|u| u.to_string()).unwrap_or_default(),
            brand_id: query.brand_id.map(|u| u.to_string()).unwrap_or_default(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

// Layer 2: Departments

pub async fn list_departments(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .list_departments(ListDepartmentsRequest { tenant_id })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn create_department(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .create_department(CreateDepartmentRequest {
            tenant_id,
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn update_department(
    State(state): State<AppState>,
    Path((_tenant_id, dept_id)): Path<(String, Uuid)>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .update_department(UpdateDepartmentRequest {
            tenant_id: _tenant_id,
            dept_id: dept_id.to_string(),
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn assign_to_department(
    State(state): State<AppState>,
    Path(dept_id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .assign_to_department(AssignToDepartmentRequest {
            dept_id: dept_id.to_string(),
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn remove_department_assignment(
    State(state): State<AppState>,
    Path((_tenant_id, assignment_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .remove_department_assignment(RemoveDepartmentAssignmentRequest {
            tenant_id: _tenant_id,
            assignment_id: assignment_id.to_string(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

// Permission Catalog

pub async fn get_permission_catalog(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .get_permission_catalog(GetPermissionCatalogRequest {
            tenant_id: String::new(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn get_all_permissions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .get_all_permissions(GetAllPermissionsRequest {
            tenant_id: String::new(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn get_permissions_by_group(
    State(state): State<AppState>,
    Path(group_name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .get_permissions_by_group(GetPermissionsByGroupRequest {
            group_name,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

// Invitations

pub async fn list_invitations(
    State(state): State<AppState>,
    Query(query): Query<InvitationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .list_invitations(ListInvitationsRequest {
            tenant_id: query.tenant_id,
            status: query.status.unwrap_or_default(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn send_email_invitation(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .send_email_invitation(SendEmailInvitationRequest {
            tenant_id,
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn send_username_invitation(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = req.to_string();
    let resp = cli
        .send_username_invitation(SendUsernameInvitationRequest {
            tenant_id,
            body,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn accept_invitation(
    State(state): State<AppState>,
    Path(invitation_id): Path<Uuid>,
    Query(query): Query<AcceptInvitationQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .accept_invitation(AcceptInvitationRequest {
            invitation_id: invitation_id.to_string(),
            team_member_id: query.team_member_id,
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn revoke_invitation(
    State(state): State<AppState>,
    Path((_tenant_id, invitation_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let resp = cli
        .revoke_invitation(RevokeInvitationRequest {
            tenant_id: _tenant_id,
            invitation_id: invitation_id.to_string(),
        })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

// Webhook Handlers

pub async fn handle_nid_access_webhook(
    State(state): State<AppState>,
    Json(webhook): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = webhook.to_string();
    let resp = cli
        .handle_nid_access_webhook(HandleNidAccessWebhookRequest { body })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn handle_nid_department_webhook(
    State(state): State<AppState>,
    Json(webhook): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = webhook.to_string();
    let resp = cli
        .handle_nid_department_webhook(HandleNidDepartmentWebhookRequest { body })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

pub async fn handle_nid_role_webhook(
    State(state): State<AppState>,
    Json(webhook): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut cli = client(&state).await?;
    let body = webhook.to_string();
    let resp = cli
        .handle_nid_role_webhook(HandleNidRoleWebhookRequest { body })
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let msg = resp.into_inner();
    parse_response(msg.payload, msg.error)
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn router() -> axum::Router<AppState> {
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
