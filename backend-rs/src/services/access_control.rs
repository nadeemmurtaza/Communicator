use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// Data Models
// ============================================================================

#[derive(FromRow, Serialize, Clone)]
pub struct AccessGrant {
    pub id: Uuid,
    pub team_member_id: String,

    pub tenant_id: String,

    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub access_type: String,
    pub partnership_percentage: Option<f64>,
    pub granted_by: String,

    pub status: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize, Clone)]
pub struct Department {
    pub id: Uuid,
    pub tenant_id: String,

    pub name: String,
    pub slug: String,
    pub default_permissions: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub inherited_from_tenant: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize, Clone)]
pub struct DepartmentAssignment {
    pub id: Uuid,
    pub team_member_id: String,

    pub department_id: Uuid,
    pub tenant_id: String,

    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub scope: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize, Clone)]
pub struct RoleAssignment {
    pub id: Uuid,
    pub team_member_id: String,

    pub role_definition_id: Uuid,
    pub tenant_id: String,

    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub scope: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize, Clone, Debug)]
pub struct PermissionCatalogEntry {
    pub id: Uuid,
    pub group_name: String,
    pub service_name: String,
    pub permission_key: String,
    pub label: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize, Clone)]
pub struct Invitation {
    pub id: Uuid,
    pub tenant_id: String,

    pub email: Option<String>,
    pub username: Option<String>,
    pub invited_by: String,

    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub scope: String,
    pub role_definition_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub status: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub accepted_by: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct PermissionCatalogGroup {
    pub group_name: String,
    pub services: Vec<PermissionCatalogService>,
}

#[derive(Serialize)]
pub struct PermissionCatalogService {
    pub service_name: String,
    pub permissions: Vec<PermissionCatalogEntry>,
}

#[derive(Serialize)]
pub struct EffectivePermissions {
    pub team_member_id: String,

    pub tenant_id: String,

    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub access_type: String,
    pub roles: Vec<String>,
    pub departments: Vec<String>,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Deserialize)]
pub struct CreateAccessGrantRequest {
    pub team_member_id: String,

    pub access_type: String,
    pub partnership_percentage: Option<f64>,
    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateAccessGrantRequest {
    pub access_type: Option<String>,
    pub partnership_percentage: Option<f64>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct TransferOwnershipRequest {
    pub new_owner_id: String,

    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct AddPartnerRequest {
    pub team_member_id: String,

    pub partnership_percentage: f64,
    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub slug: Option<String>,
    pub default_permissions: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub default_permissions: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct AssignDepartmentRequest {
    pub team_member_id: String,

    pub scope: Option<String>,
    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub team_member_id: String,

    pub scope: Option<String>,
    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct SendEmailInvitationRequest {
    pub email: String,
    pub scope: Option<String>,
    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub role_definition_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct SendUsernameInvitationRequest {
    pub username: String,
    pub scope: Option<String>,
    pub org_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub role_definition_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct NidAccessWebhook {
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct NidDepartmentWebhook {
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct NidRoleWebhook {
    pub event: String,
    pub data: serde_json::Value,
}

// ============================================================================
// Access Control Service
// ============================================================================

pub struct AccessControlService;

impl AccessControlService {
    // ---- Layer 1: Entity Access ----

    pub async fn list_access_grants(
        pool: &PgPool,
        tenant_id: String,

        org_id: Option<Uuid>,
        brand_id: Option<Uuid>,
    ) -> Result<Vec<AccessGrant>, StatusCode> {
        let grants = sqlx::query_as!(
            AccessGrant,
            r#"SELECT id, team_member_id, tenant_id, org_id, brand_id, access_type,
                      partnership_percentage, granted_by, status, granted_at
               FROM access_grants
               WHERE tenant_id = $1 AND status = 'active'
                 AND (org_id IS NULL OR org_id = $2)
                 AND (brand_id IS NULL OR brand_id = $3)
               ORDER BY granted_at DESC"#,
            tenant_id,
            org_id,
            brand_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(grants)
    }

    pub async fn create_access_grant(
        pool: &PgPool,
        tenant_id: String,

        req: &CreateAccessGrantRequest,
        granted_by: String,

    ) -> Result<AccessGrant, StatusCode> {
        if req.access_type != "owner" && req.access_type != "partner" && req.access_type != "admin" && req.access_type != "member" {
            return Err(StatusCode::BAD_REQUEST);
        }
        if req.access_type == "partner" && (req.partnership_percentage.is_none() || req.partnership_percentage.unwrap() <= 0.0 || req.partnership_percentage.unwrap() > 100.0) {
            return Err(StatusCode::BAD_REQUEST);
        }
        let grant = sqlx::query_as!(
            AccessGrant,
            r#"INSERT INTO access_grants (team_member_id, tenant_id, org_id, brand_id, access_type, partnership_percentage, granted_by, status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'active')
               RETURNING id, team_member_id, tenant_id, org_id, brand_id, access_type, partnership_percentage, granted_by, status, granted_at"#,
            req.team_member_id,
            tenant_id,
            req.org_id,
            req.brand_id,
            req.access_type,
            req.partnership_percentage,
            granted_by
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(grant)
    }

    pub async fn update_access_grant(
        pool: &PgPool,
        grant_id: Uuid,
        req: &UpdateAccessGrantRequest,
    ) -> Result<AccessGrant, StatusCode> {
        let mut sets = Vec::new();
        let mut vals = Vec::new();
        let mut counter = 1;

        if let Some(ref access_type) = req.access_type {
            sets.push(format!("access_type = ${}", counter));
            vals.push(format!("{}", access_type));
            counter += 1;
        }
        if let Some(partnership_percentage) = req.partnership_percentage {
            sets.push(format!("partnership_percentage = ${}", counter));
            vals.push(format!("{:.2}", partnership_percentage));
            counter += 1;
        }
        if let Some(ref status) = req.status {
            sets.push(format!("status = ${}", counter));
            vals.push(format!("{}", status));
            counter += 1;
        }

        if sets.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        sets.push(format!("id = ${}", counter));
        vals.push(format!("{}", grant_id));

        let query = format!(
            "UPDATE access_grants SET {} WHERE id = $1 RETURNING id, team_member_id, tenant_id, org_id, brand_id, access_type, partnership_percentage, granted_by, status, granted_at",
            sets.join(", ")
        );

        let grant: AccessGrant = sqlx::query_as(&query)
            .fetch_one(pool)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(grant)
    }

    pub async fn revoke_access_grant(pool: &PgPool, grant_id: Uuid) -> Result<(), StatusCode> {
        sqlx::query!("UPDATE access_grants SET status = 'revoked' WHERE id = $1", grant_id)
            .execute(pool)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(())
    }

    pub async fn transfer_ownership(
        pool: &PgPool,
        tenant_id: String,

        req: &TransferOwnershipRequest,
    ) -> Result<AccessGrant, StatusCode> {
        let grant = sqlx::query_as!(
            AccessGrant,
            r#"INSERT INTO access_grants (team_member_id, tenant_id, org_id, brand_id, access_type, granted_by, status)
               SELECT $1, $2, $3, $4, 'owner', $1, 'active'
               WHERE NOT EXISTS (
                   SELECT 1 FROM access_grants
                   WHERE team_member_id = $1 AND tenant_id = $2 AND access_type = 'owner' AND status = 'active'
               )
               RETURNING id, team_member_id, tenant_id, org_id, brand_id, access_type, partnership_percentage, granted_by, status, granted_at"#,
            req.new_owner_id,
            tenant_id,
            req.org_id,
            req.brand_id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
        Ok(grant)
    }

    pub async fn add_partner(
        pool: &PgPool,
        tenant_id: String,

        req: &AddPartnerRequest,
        granted_by: String,

    ) -> Result<AccessGrant, StatusCode> {
        let grant = sqlx::query_as!(
            AccessGrant,
            r#"INSERT INTO access_grants (team_member_id, tenant_id, org_id, brand_id, access_type, partnership_percentage, granted_by, status)
               VALUES ($1, $2, $3, $4, 'partner', $5, $6, 'active')
               RETURNING id, team_member_id, tenant_id, org_id, brand_id, access_type, partnership_percentage, granted_by, status, granted_at"#,
            req.team_member_id,
            tenant_id,
            req.org_id,
            req.brand_id,
            req.partnership_percentage,
            granted_by
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(grant)
    }

    // ---- Layer 2: Department Access ----

    pub async fn list_departments(
        pool: &PgPool,
        tenant_id: String,

    ) -> Result<Vec<Department>, StatusCode> {
        let depts = sqlx::query_as!(
            Department,
            r#"SELECT id, tenant_id, name, slug, default_permissions, icon, color, inherited_from_tenant, created_at, updated_at
               FROM departments
               WHERE tenant_id = $1
               ORDER BY name"#,
            tenant_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(depts)
    }

    pub async fn create_department(
        pool: &PgPool,
        tenant_id: String,

        req: &CreateDepartmentRequest,
    ) -> Result<Department, StatusCode> {
        let slug = req.slug.clone().unwrap_or_else(|| {
            req.name.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .map(|c| if c.is_whitespace() { '-' } else { c })
                .collect()
        });
        let dept = sqlx::query_as!(
            Department,
            r#"INSERT INTO departments (tenant_id, name, slug, default_permissions, icon, color)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, tenant_id, name, slug, default_permissions, icon, color, inherited_from_tenant, created_at, updated_at"#,
            tenant_id,
            req.name,
            slug,
            req.default_permissions.clone(),
            req.icon.as_deref(),
            req.color.as_deref()
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(dept)
    }

    pub async fn update_department(
        pool: &PgPool,
        dept_id: Uuid,
        req: &UpdateDepartmentRequest,
    ) -> Result<Department, StatusCode> {
        let mut sets = Vec::new();
        if let Some(ref name) = req.name {
            sets.push(format!("name = '{}'", name.replace('\'', "''")));
        }
        if let Some(ref perms) = req.default_permissions {
            sets.push(format!("default_permissions = {}", perms));
        }
        if let Some(ref icon) = req.icon {
            sets.push(format!("icon = '{}'", icon.replace('\'', "''")));
        }
        if let Some(ref color) = req.color {
            sets.push(format!("color = '{}'", color.replace('\'', "''")));
        }
        sets.push("updated_at = NOW()".to_string());

        if sets.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let query = format!(
            "UPDATE departments SET {} WHERE id = '{}' RETURNING id, tenant_id, name, slug, default_permissions, icon, color, inherited_from_tenant, created_at, updated_at",
            sets.join(", "),
            dept_id.to_string().replace('\'', "''")
        );

        let dept: Department = sqlx::query_as(&query)
            .fetch_one(pool)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(dept)
    }

    pub async fn assign_to_department(
        pool: &PgPool,
        dept_id: Uuid,
        req: &AssignDepartmentRequest,
    ) -> Result<DepartmentAssignment, StatusCode> {
        // Get department tenant_id
        let tenant_id: String = sqlx::query_scalar!("SELECT tenant_id FROM departments WHERE id = $1", dept_id)
            .fetch_one(pool)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        let scope = req.scope.clone().unwrap_or_else(|| {
            if req.brand_id.is_some() { "brand".to_string() }
            else if req.org_id.is_some() { "organization".to_string() }
            else { "tenant".to_string() }
        });

        let assignment = sqlx::query_as!(
            DepartmentAssignment,
            r#"INSERT INTO department_assignments (team_member_id, department_id, tenant_id, org_id, brand_id, scope)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, team_member_id, department_id, tenant_id, org_id, brand_id, scope, created_at"#,
            req.team_member_id,
            dept_id,
            tenant_id,
            req.org_id,
            req.brand_id,
            scope
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(assignment)
    }

    pub async fn remove_department_assignment(
        pool: &PgPool,
        assignment_id: Uuid,
    ) -> Result<(), StatusCode> {
        sqlx::query!("DELETE FROM department_assignments WHERE id = $1", assignment_id)
            .execute(pool)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(())
    }

    // ---- Layer 3: Role & Permission Access ----

    pub async fn check_entity_access(
        pool: &PgPool,
        team_member_id: String,

        tenant_id: String,

        org_id: Option<Uuid>,
        brand_id: Option<Uuid>,
    ) -> Result<Option<AccessGrant>, StatusCode> {
        let grant = sqlx::query_as!(
            AccessGrant,
            r#"SELECT id, team_member_id, tenant_id, org_id, brand_id, access_type,
                      partnership_percentage, granted_by, status, granted_at
               FROM access_grants
               WHERE team_member_id = $1 AND tenant_id = $2 AND status = 'active'
                 AND (org_id IS NULL OR org_id = $3)
                 AND (brand_id IS NULL OR brand_id = $4)
               ORDER BY
                   CASE access_type WHEN 'owner' THEN 1 WHEN 'partner' THEN 2 WHEN 'admin' THEN 3 WHEN 'member' THEN 4 END
               LIMIT 1"#,
            team_member_id,
            tenant_id,
            org_id,
            brand_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(grant)
    }

    pub async fn check_department_access(
        pool: &PgPool,
        team_member_id: String,

        tenant_id: String,

        department_slug: &str,
    ) -> Result<bool, StatusCode> {
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COALESCE(COUNT(*), 0) AS "count!" FROM department_assignments da
               JOIN departments d ON da.department_id = d.id
               WHERE da.team_member_id = $1 AND d.tenant_id = $2 AND d.slug = $3"#,
            team_member_id,
            tenant_id,
            department_slug
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(count > 0)
    }

    pub async fn check_permission(
        pool: &PgPool,
        team_member_id: String,

        tenant_id: String,

        permission_key: &str,
        org_id: Option<Uuid>,
        brand_id: Option<Uuid>,
    ) -> Result<bool, StatusCode> {
        // Check if member has owner/partner access (bypasses permission checks)
        let is_owner: i64 = sqlx::query_scalar!(
            r#"SELECT COALESCE(COUNT(*), 0) AS "count!" FROM access_grants
               WHERE team_member_id = $1 AND tenant_id = $2 AND status = 'active'
                 AND access_type IN ('owner', 'partner')
                 AND (org_id IS NULL OR org_id = $3)
                 AND (brand_id IS NULL OR brand_id = $4)"#,
            team_member_id,
            tenant_id,
            org_id,
            brand_id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if is_owner > 0 {
            return Ok(true);
        }

        // Check role-based permissions
        let has_perm: i64 = sqlx::query_scalar!(
            r#"SELECT COALESCE(COUNT(*), 0) AS "count!"
               FROM role_assignments ra
               JOIN tenant_role_definitions rd ON ra.role_definition_id = rd.id
               WHERE ra.team_member_id = $1 AND ra.tenant_id = $2
                 AND (ra.org_id IS NULL OR ra.org_id = $3)
                 AND (ra.brand_id IS NULL OR ra.brand_id = $4)
                 AND jsonb_exists(rd.permissions, $5)"#,
            team_member_id,
            tenant_id,
            org_id,
            brand_id,
            permission_key
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(has_perm > 0)
    }

    pub async fn resolve_effective_permissions(
        pool: &PgPool,
        team_member_id: String,

        tenant_id: String,

        org_id: Option<Uuid>,
        brand_id: Option<Uuid>,
    ) -> Result<EffectivePermissions, StatusCode> {
        // Get access type
        let grant = Self::check_entity_access(pool, team_member_id.clone(), tenant_id.clone(), org_id, brand_id).await?;
        let access_type = grant.as_ref().map(|g| g.access_type.clone()).unwrap_or_else(|| "none".to_string());

        // If owner/partner, return all permissions
        if access_type == "owner" || access_type == "partner" {
            let all_perms: Vec<String> = sqlx::query_scalar!(
                r#"SELECT DISTINCT permission_key FROM permission_catalog ORDER BY permission_key"#
            )
            .fetch_all(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(EffectivePermissions {
                team_member_id: team_member_id.clone(),
                tenant_id: tenant_id.clone(),
                org_id,
                brand_id,
                permissions: all_perms,
                access_type,
                roles: Vec::new(),
                departments: Vec::new(),
            });
        }

        // Collect permissions from all roles
        let perms: Vec<Option<String>> = sqlx::query_scalar!(
            r#"SELECT DISTINCT jsonb_array_elements_text(rd.permissions)::TEXT
               FROM role_assignments ra
               JOIN tenant_role_definitions rd ON ra.role_definition_id = rd.id
               WHERE ra.team_member_id = $1 AND ra.tenant_id = $2
                 AND (ra.org_id IS NULL OR ra.org_id = $3)
                 AND (ra.brand_id IS NULL OR ra.brand_id = $4)
               ORDER BY 1"#,
            team_member_id.clone(),
            tenant_id.clone(),
            org_id,
            brand_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let perms: Vec<String> = perms.into_iter().flatten().collect();

        // Collect role names
        let roles: Vec<String> = sqlx::query_scalar!(
            r#"SELECT DISTINCT rd.name
               FROM role_assignments ra
               JOIN tenant_role_definitions rd ON ra.role_definition_id = rd.id
               WHERE ra.team_member_id = $1 AND ra.tenant_id = $2
                 AND (ra.org_id IS NULL OR ra.org_id = $3)
                 AND (ra.brand_id IS NULL OR ra.brand_id = $4)"#,
            team_member_id.clone(),
            tenant_id.clone(),
            org_id,
            brand_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Collect department names
        let departments: Vec<String> = sqlx::query_scalar!(
            r#"SELECT DISTINCT d.name
               FROM department_assignments da
               JOIN departments d ON da.department_id = d.id
               WHERE da.team_member_id = $1 AND d.tenant_id = $2"#,
            team_member_id.clone(),
            tenant_id.clone()
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(EffectivePermissions {
            team_member_id,
            tenant_id,
            org_id,
            brand_id,
            permissions: perms,
            access_type,
            roles,
            departments,
        })
    }

    // ---- Permission Catalog ----

    pub async fn get_permission_catalog(pool: &PgPool) -> Result<Vec<PermissionCatalogGroup>, StatusCode> {
        let entries = sqlx::query_as!(
            PermissionCatalogEntry,
            r#"SELECT id, group_name, service_name, permission_key, label, description, created_at
               FROM permission_catalog
               ORDER BY group_name, service_name, permission_key"#
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Group by group_name -> service_name -> permissions
        let mut groups: HashMap<String, HashMap<String, Vec<PermissionCatalogEntry>>> = HashMap::new();
        for entry in entries {
            groups
                .entry(entry.group_name.clone())
                .or_default()
                .entry(entry.service_name.clone())
                .or_default()
                .push(entry);
        }

        let mut result = Vec::new();
        for (group_name, services) in groups {
            let mut service_list: Vec<PermissionCatalogService> = services
                .into_iter()
                .map(|(service_name, permissions)| PermissionCatalogService {
                    service_name,
                    permissions,
                })
                .collect();
            service_list.sort_by(|a, b| a.service_name.cmp(&b.service_name));
            result.push(PermissionCatalogGroup {
                group_name,
                services: service_list,
            });
        }
        result.sort_by(|a, b| a.group_name.cmp(&b.group_name));
        Ok(result)
    }

    pub async fn get_all_permissions(pool: &PgPool) -> Result<Vec<PermissionCatalogEntry>, StatusCode> {
        let entries = sqlx::query_as!(
            PermissionCatalogEntry,
            r#"SELECT id, group_name, service_name, permission_key, label, description, created_at
               FROM permission_catalog
               ORDER BY group_name, service_name, permission_key"#
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(entries)
    }

    pub async fn get_permissions_by_group(
        pool: &PgPool,
        group_name: &str,
    ) -> Result<Vec<PermissionCatalogEntry>, StatusCode> {
        let entries = sqlx::query_as!(
            PermissionCatalogEntry,
            r#"SELECT id, group_name, service_name, permission_key, label, description, created_at
               FROM permission_catalog
               WHERE group_name = $1
               ORDER BY service_name, permission_key"#,
            group_name
        )
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(entries)
    }

    // ---- Invitations ----

    pub async fn list_invitations(
        pool: &PgPool,
        tenant_id: String,

        status: Option<&str>,
    ) -> Result<Vec<Invitation>, StatusCode> {
        let invitations = if let Some(status) = status {
            sqlx::query_as!(
                Invitation,
                r#"SELECT id, tenant_id, email, username, invited_by, org_id, brand_id, scope,
                          role_definition_id, department_id, status, expires_at, accepted_at, accepted_by, created_at
                   FROM invitations
                   WHERE tenant_id = $1 AND status = $2
                   ORDER BY created_at DESC"#,
                tenant_id,
                status
            )
            .fetch_all(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        } else {
            sqlx::query_as!(
                Invitation,
                r#"SELECT id, tenant_id, email, username, invited_by, org_id, brand_id, scope,
                          role_definition_id, department_id, status, expires_at, accepted_at, accepted_by, created_at
                   FROM invitations
                   WHERE tenant_id = $1
                   ORDER BY created_at DESC"#,
                tenant_id
            )
            .fetch_all(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        };
        Ok(invitations)
    }

    pub async fn send_email_invitation(
        pool: &PgPool,
        tenant_id: String,

        req: &SendEmailInvitationRequest,
        invited_by: String,

    ) -> Result<Invitation, StatusCode> {
        let scope = req.scope.clone().unwrap_or_else(|| "tenant".to_string());
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        let invitation = sqlx::query_as!(
            Invitation,
            r#"INSERT INTO invitations (tenant_id, email, invited_by, org_id, brand_id, scope, role_definition_id, department_id, status, expires_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9)
               RETURNING id, tenant_id, email, username, invited_by, org_id, brand_id, scope, role_definition_id, department_id, status, expires_at, accepted_at, accepted_by, created_at"#,
            tenant_id,
            req.email,
            invited_by,
            req.org_id,
            req.brand_id,
            scope,
            req.role_definition_id,
            req.department_id,
            expires_at
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(invitation)
    }

    pub async fn send_username_invitation(
        pool: &PgPool,
        tenant_id: String,

        req: &SendUsernameInvitationRequest,
        invited_by: String,

    ) -> Result<Invitation, StatusCode> {
        let scope = req.scope.clone().unwrap_or_else(|| "tenant".to_string());
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        let invitation = sqlx::query_as!(
            Invitation,
            r#"INSERT INTO invitations (tenant_id, username, invited_by, org_id, brand_id, scope, role_definition_id, department_id, status, expires_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9)
               RETURNING id, tenant_id, email, username, invited_by, org_id, brand_id, scope, role_definition_id, department_id, status, expires_at, accepted_at, accepted_by, created_at"#,
            tenant_id,
            req.username,
            invited_by,
            req.org_id,
            req.brand_id,
            scope,
            req.role_definition_id,
            req.department_id,
            expires_at
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(invitation)
    }

    pub async fn accept_invitation(
        pool: &PgPool,
        invitation_id: Uuid,
        accepted_by: String,

    ) -> Result<Invitation, StatusCode> {
        let invitation = sqlx::query_as!(
            Invitation,
            r#"UPDATE invitations SET status = 'accepted', accepted_at = NOW(), accepted_by = $2
               WHERE id = $1 AND status = 'pending' AND expires_at > NOW()
               RETURNING id, tenant_id, email, username, invited_by, org_id, brand_id, scope, role_definition_id, department_id, status, expires_at, accepted_at, accepted_by, created_at"#,
            invitation_id,
            accepted_by
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(invitation)
    }

    pub async fn revoke_invitation(pool: &PgPool, invitation_id: Uuid) -> Result<(), StatusCode> {
        sqlx::query!("UPDATE invitations SET status = 'revoked' WHERE id = $1 AND status = 'pending'", invitation_id)
            .execute(pool)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(())
    }

    // ---- Webhook Handlers ----

    pub async fn handle_nid_access_webhook(
        pool: &PgPool,
        webhook: &NidAccessWebhook,
    ) -> Result<(), StatusCode> {
        let event = &webhook.event;
        let data = &webhook.data;

        if event == "access.grant.created" || event == "access.grant.updated" {
            // Mirror the access grant from nID to Communicator
            if let Some(tenant_id) = data.get("tenantId").and_then(|v| v.as_str()) {
                if let Some(team_member_id) = data.get("accountId").and_then(|v| v.as_str()) {
                    let access_type = data.get("accessType").and_then(|v| v.as_str()).unwrap_or("member");
                    let partnership_percentage = data.get("partnershipPercentage").and_then(|v| v.as_f64());

                    // Upsert the access grant
                    let _ = sqlx::query(
                        r#"INSERT INTO access_grants (team_member_id, tenant_id, access_type, partnership_percentage, granted_by, status)
                           VALUES ($1, $2, $3, $4, $1, 'active')
                           ON CONFLICT (team_member_id, tenant_id, org_id, brand_id)
                           DO UPDATE SET access_type = $3, partnership_percentage = $4, status = 'active'"#,
                    )
                    .bind(team_member_id)
                    .bind(tenant_id)
                    .bind(access_type)
                    .bind(partnership_percentage)
                    .execute(pool)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                }
            }
        }

        Ok(())
    }

    pub async fn handle_nid_department_webhook(
        pool: &PgPool,
        webhook: &NidDepartmentWebhook,
    ) -> Result<(), StatusCode> {
        let event = &webhook.event;
        let data = &webhook.data;

        if event == "department.created" || event == "department.updated" {
            if let Some(tenant_id) = data.get("tenantId").and_then(|v| v.as_str()) {
                let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let name_lower = name.to_lowercase();
                let slug = data.get("slug").and_then(|v| v.as_str()).unwrap_or(name_lower.as_str());
                let icon = data.get("icon").and_then(|v| v.as_str());
                let color = data.get("color").and_then(|v| v.as_str());

                let _ = sqlx::query(
                    r#"INSERT INTO departments (tenant_id, name, slug, icon, color)
                       VALUES ($1, $2, $3, $4, $5)
                       ON CONFLICT (tenant_id, slug)
                       DO UPDATE SET name = $2, icon = $4, color = $5, updated_at = NOW()"#,
                )
                .bind(tenant_id)
                .bind(name)
                .bind(slug)
                .bind(icon)
                .bind(color)
                .execute(pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }

        Ok(())
    }

    pub async fn handle_nid_role_webhook(
        pool: &PgPool,
        webhook: &NidRoleWebhook,
    ) -> Result<(), StatusCode> {
        let event = &webhook.event;
        let data = &webhook.data;

        if event == "role.created" || event == "role.updated" {
            if let Some(tenant_id) = data.get("tenantId").and_then(|v| v.as_str()) {
                let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let name_lower = name.to_lowercase();
                let slug = data.get("slug").and_then(|v| v.as_str()).unwrap_or(name_lower.as_str());
                let description = data.get("description").and_then(|v| v.as_str());
                let permissions = data.get("permissions").cloned().unwrap_or_else(|| serde_json::json!([]));

                let _ = sqlx::query(
                    r#"INSERT INTO tenant_role_definitions (tenant_id, name, slug, description, permissions, is_built_in)
                       VALUES ($1, $2, $3, $4, $5, false)
                       ON CONFLICT (tenant_id, slug)
                       DO UPDATE SET name = $2, description = $4, permissions = $5, updated_at = NOW()"#,
                )
                .bind(tenant_id)
                .bind(name)
                .bind(slug)
                .bind(description)
                .bind(&permissions)
                .execute(pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }

        Ok(())
    }
}
