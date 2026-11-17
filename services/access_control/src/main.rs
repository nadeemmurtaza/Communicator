mod generated {
    tonic::include_proto!("newax.access_control.v1");
}

use generated::access_control_server::{AccessControl, AccessControlServer};
use generated::*;
// Proto messages share names with the request structs in newax_access_control.
// Explicit imports win over the two glob imports; handlers that deserialize the
// JSON body qualify the library type as `newax_access_control::...`.
use generated::{
    AddPartnerRequest, CreateAccessGrantRequest, CreateDepartmentRequest,
    SendEmailInvitationRequest, SendUsernameInvitationRequest, TransferOwnershipRequest,
    UpdateAccessGrantRequest, UpdateDepartmentRequest,
};
use newax_access_control::*;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AccessControlServiceImpl {
    state: Arc<AccessControlState>,
}

impl AccessControlServiceImpl {
    fn new(pool: PgPool) -> Self {
        Self {
            state: Arc::new(AccessControlState { db: pool }),
        }
    }
}

fn err_response<E: std::fmt::Display>(e: E) -> serde_json::Value {
    serde_json::json!({ "error": format!("{}", e) })
}

fn ok_response<T: serde::Serialize>(val: T) -> serde_json::Value {
    serde_json::to_value(val).unwrap_or_else(|e| err_response(e))
}

#[tonic::async_trait]
impl AccessControl for AccessControlServiceImpl {
    async fn list_access_grants(
        &self,
        request: Request<ListAccessGrantsRequest>,
    ) -> Result<Response<ListAccessGrantsResponse>, Status> {
        let msg = request.into_inner();
        let org_id = parse_uuid(&msg.org_id);
        let brand_id = parse_uuid(&msg.brand_id);
        match list_access_grants(&self.state, msg.tenant_id, org_id, brand_id).await {
            Ok(v) => Ok(Response::new(ListAccessGrantsResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ListAccessGrantsResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn create_access_grant(
        &self,
        request: Request<CreateAccessGrantRequest>,
    ) -> Result<Response<CreateAccessGrantResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_access_control::CreateAccessGrantRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let granted_by = req.team_member_id.clone();
        match create_access_grant(&self.state, msg.tenant_id, &req, granted_by).await {
            Ok(v) => Ok(Response::new(CreateAccessGrantResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CreateAccessGrantResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn update_access_grant(
        &self,
        request: Request<UpdateAccessGrantRequest>,
    ) -> Result<Response<UpdateAccessGrantResponse>, Status> {
        let msg = request.into_inner();
        let grant_id = parse_uuid_required(&msg.grant_id)?;
        let req: newax_access_control::UpdateAccessGrantRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match update_access_grant(&self.state, grant_id, &req).await {
            Ok(v) => Ok(Response::new(UpdateAccessGrantResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UpdateAccessGrantResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn revoke_access_grant(
        &self,
        request: Request<RevokeAccessGrantRequest>,
    ) -> Result<Response<RevokeAccessGrantResponse>, Status> {
        let msg = request.into_inner();
        let grant_id = parse_uuid_required(&msg.grant_id)?;
        match revoke_access_grant(&self.state, grant_id).await {
            Ok(_) => Ok(Response::new(RevokeAccessGrantResponse {
                payload: serde_json::json!({ "message": "Access grant revoked" }).to_string(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RevokeAccessGrantResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn transfer_ownership(
        &self,
        request: Request<TransferOwnershipRequest>,
    ) -> Result<Response<TransferOwnershipResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_access_control::TransferOwnershipRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match transfer_ownership(&self.state, msg.tenant_id, &req).await {
            Ok(v) => Ok(Response::new(TransferOwnershipResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(TransferOwnershipResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn add_partner(
        &self,
        request: Request<AddPartnerRequest>,
    ) -> Result<Response<AddPartnerResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_access_control::AddPartnerRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let granted_by = req.team_member_id.clone();
        match add_partner(&self.state, msg.tenant_id, &req, granted_by).await {
            Ok(v) => Ok(Response::new(AddPartnerResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(AddPartnerResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn get_my_permissions(
        &self,
        request: Request<GetMyPermissionsRequest>,
    ) -> Result<Response<GetMyPermissionsResponse>, Status> {
        let msg = request.into_inner();
        let org_id = parse_uuid(&msg.org_id);
        let brand_id = parse_uuid(&msg.brand_id);
        match get_my_permissions(&self.state, msg.team_member_id, msg.tenant_id, org_id, brand_id).await {
            Ok(v) => Ok(Response::new(GetMyPermissionsResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(GetMyPermissionsResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn list_departments(
        &self,
        request: Request<ListDepartmentsRequest>,
    ) -> Result<Response<ListDepartmentsResponse>, Status> {
        let msg = request.into_inner();
        match list_departments(&self.state, msg.tenant_id).await {
            Ok(v) => Ok(Response::new(ListDepartmentsResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ListDepartmentsResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn create_department(
        &self,
        request: Request<CreateDepartmentRequest>,
    ) -> Result<Response<CreateDepartmentResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_access_control::CreateDepartmentRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match create_department(&self.state, msg.tenant_id, &req).await {
            Ok(v) => Ok(Response::new(CreateDepartmentResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CreateDepartmentResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn update_department(
        &self,
        request: Request<UpdateDepartmentRequest>,
    ) -> Result<Response<UpdateDepartmentResponse>, Status> {
        let msg = request.into_inner();
        let dept_id = parse_uuid_required(&msg.dept_id)?;
        let req: newax_access_control::UpdateDepartmentRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match update_department(&self.state, dept_id, &req).await {
            Ok(v) => Ok(Response::new(UpdateDepartmentResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UpdateDepartmentResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn assign_to_department(
        &self,
        request: Request<AssignToDepartmentRequest>,
    ) -> Result<Response<AssignToDepartmentResponse>, Status> {
        let msg = request.into_inner();
        let dept_id = parse_uuid_required(&msg.dept_id)?;
        let req: newax_access_control::AssignDepartmentRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match assign_to_department(&self.state, dept_id, &req).await {
            Ok(v) => Ok(Response::new(AssignToDepartmentResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(AssignToDepartmentResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn remove_department_assignment(
        &self,
        request: Request<RemoveDepartmentAssignmentRequest>,
    ) -> Result<Response<RemoveDepartmentAssignmentResponse>, Status> {
        let msg = request.into_inner();
        let assignment_id = parse_uuid_required(&msg.assignment_id)?;
        match remove_department_assignment(&self.state, assignment_id).await {
            Ok(_) => Ok(Response::new(RemoveDepartmentAssignmentResponse {
                payload: serde_json::json!({ "message": "Assignment removed" }).to_string(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RemoveDepartmentAssignmentResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn get_permission_catalog(
        &self,
        _request: Request<GetPermissionCatalogRequest>,
    ) -> Result<Response<GetPermissionCatalogResponse>, Status> {
        match get_permission_catalog(&self.state).await {
            Ok(v) => Ok(Response::new(GetPermissionCatalogResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(GetPermissionCatalogResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn get_all_permissions(
        &self,
        _request: Request<GetAllPermissionsRequest>,
    ) -> Result<Response<GetAllPermissionsResponse>, Status> {
        match get_all_permissions(&self.state).await {
            Ok(v) => Ok(Response::new(GetAllPermissionsResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(GetAllPermissionsResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn get_permissions_by_group(
        &self,
        request: Request<GetPermissionsByGroupRequest>,
    ) -> Result<Response<GetPermissionsByGroupResponse>, Status> {
        let msg = request.into_inner();
        match get_permissions_by_group(&self.state, &msg.group_name).await {
            Ok(v) => Ok(Response::new(GetPermissionsByGroupResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(GetPermissionsByGroupResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn list_invitations(
        &self,
        request: Request<ListInvitationsRequest>,
    ) -> Result<Response<ListInvitationsResponse>, Status> {
        let msg = request.into_inner();
        let status = if msg.status.is_empty() {
            None
        } else {
            Some(msg.status.as_str())
        };
        match list_invitations(&self.state, msg.tenant_id, status).await {
            Ok(v) => Ok(Response::new(ListInvitationsResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ListInvitationsResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn send_email_invitation(
        &self,
        request: Request<SendEmailInvitationRequest>,
    ) -> Result<Response<SendEmailInvitationResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_access_control::SendEmailInvitationRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let invited_by = "system".to_string();
        match send_email_invitation(&self.state, msg.tenant_id, &req, invited_by).await {
            Ok(v) => Ok(Response::new(SendEmailInvitationResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(SendEmailInvitationResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn send_username_invitation(
        &self,
        request: Request<SendUsernameInvitationRequest>,
    ) -> Result<Response<SendUsernameInvitationResponse>, Status> {
        let msg = request.into_inner();
        let req: newax_access_control::SendUsernameInvitationRequest =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let invited_by = "system".to_string();
        match send_username_invitation(&self.state, msg.tenant_id, &req, invited_by).await {
            Ok(v) => Ok(Response::new(SendUsernameInvitationResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(SendUsernameInvitationResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn accept_invitation(
        &self,
        request: Request<AcceptInvitationRequest>,
    ) -> Result<Response<AcceptInvitationResponse>, Status> {
        let msg = request.into_inner();
        let invitation_id = parse_uuid_required(&msg.invitation_id)?;
        match accept_invitation(&self.state, invitation_id, msg.team_member_id).await {
            Ok(v) => Ok(Response::new(AcceptInvitationResponse {
                payload: serde_json::to_string(&v).unwrap_or_default(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(AcceptInvitationResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn revoke_invitation(
        &self,
        request: Request<RevokeInvitationRequest>,
    ) -> Result<Response<RevokeInvitationResponse>, Status> {
        let msg = request.into_inner();
        let invitation_id = parse_uuid_required(&msg.invitation_id)?;
        match revoke_invitation(&self.state, invitation_id).await {
            Ok(_) => Ok(Response::new(RevokeInvitationResponse {
                payload: serde_json::json!({ "message": "Invitation revoked" }).to_string(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RevokeInvitationResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn handle_nid_access_webhook(
        &self,
        request: Request<HandleNidAccessWebhookRequest>,
    ) -> Result<Response<HandleNidAccessWebhookResponse>, Status> {
        let msg = request.into_inner();
        let webhook: NidAccessWebhook =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match handle_nid_access_webhook(&self.state, &webhook).await {
            Ok(_) => Ok(Response::new(HandleNidAccessWebhookResponse {
                payload: serde_json::json!({ "status": "ok" }).to_string(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(HandleNidAccessWebhookResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn handle_nid_department_webhook(
        &self,
        request: Request<HandleNidDepartmentWebhookRequest>,
    ) -> Result<Response<HandleNidDepartmentWebhookResponse>, Status> {
        let msg = request.into_inner();
        let webhook: NidDepartmentWebhook =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match handle_nid_department_webhook(&self.state, &webhook).await {
            Ok(_) => Ok(Response::new(HandleNidDepartmentWebhookResponse {
                payload: serde_json::json!({ "status": "ok" }).to_string(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(HandleNidDepartmentWebhookResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }

    async fn handle_nid_role_webhook(
        &self,
        request: Request<HandleNidRoleWebhookRequest>,
    ) -> Result<Response<HandleNidRoleWebhookResponse>, Status> {
        let msg = request.into_inner();
        let webhook: NidRoleWebhook =
            serde_json::from_str(&msg.body).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match handle_nid_role_webhook(&self.state, &webhook).await {
            Ok(_) => Ok(Response::new(HandleNidRoleWebhookResponse {
                payload: serde_json::json!({ "status": "ok" }).to_string(),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(HandleNidRoleWebhookResponse {
                payload: String::new(),
                error: e.to_http().to_string(),
            })),
        }
    }
}

fn parse_uuid(s: &str) -> Option<uuid::Uuid> {
    if s.is_empty() {
        return None;
    }
    s.parse().ok()
}

fn parse_uuid_required(s: &str) -> Result<uuid::Uuid, Status> {
    parse_uuid(s).ok_or_else(|| Status::invalid_argument("invalid UUID"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://newax:newax@localhost/newax".to_string());
    let pool = PgPool::connect(&database_url).await?;
    let addr = "[::1]:50093".parse()?;
    let service = AccessControlServiceImpl::new(pool);
    tracing::info!("access-control listening on {}", addr);
    tonic::transport::Server::builder()
        .add_service(AccessControlServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
