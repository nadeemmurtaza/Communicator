from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import rbac_service

router = APIRouter(prefix="/roles", tags=["Role-Based Access Control (RBAC)"])


class CreateRoleIn(BaseModel):
    name: str = Field(..., description="Unique slug identifier, e.g. billing_specialist")
    display_name: str = Field(..., description="Display label, e.g. Billing Specialist")
    description: str = ""
    permissions: list[str] = Field(
        default_factory=list, description="List of granted permission keys"
    )


class UpdateRoleIn(BaseModel):
    display_name: str | None = None
    description: str | None = None
    permissions: list[str] | None = None


class AssignMemberRoleIn(BaseModel):
    role: str = Field(..., description="Role slug name e.g. super_admin, admin, agent")
    custom_permissions: list[str] | None = None
    actor: str = "admin"


@router.get("")
async def list_roles(
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    """Lists all system and custom RBAC roles with assigned permissions and member counts."""
    return await rbac_service.list_roles(session)


@router.get("/permissions")
async def get_permissions_catalog() -> list[dict[str, Any]]:
    """Returns the full master catalog of all system permissions grouped by category."""
    return rbac_service.get_permissions_catalog()


@router.get("/{role_id_or_name}")
async def get_role(
    role_id_or_name: str,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Retrieves full details and assigned members for a specific role."""
    role = await rbac_service.get_role(session, role_id_or_name)
    if not role:
        raise HTTPException(status_code=404, detail="Role not found")
    return role


@router.post("")
async def create_role(
    payload: CreateRoleIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Creates a custom enterprise RBAC role definition."""
    try:
        role = await rbac_service.create_custom_role(
            session,
            name=payload.name,
            display_name=payload.display_name,
            description=payload.description,
            permissions=payload.permissions,
        )
        return {
            "success": True,
            "id": str(role.id),
            "name": role.name,
            "display_name": role.display_name,
            "permissions": role.permissions,
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.put("/{role_id_or_name}")
async def update_role(
    role_id_or_name: str,
    payload: UpdateRoleIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Updates role permissions or display metadata."""
    try:
        updates = payload.model_dump(exclude_unset=True)
        role = await rbac_service.update_role(session, role_id_or_name, updates)
        return {
            "success": True,
            "id": str(role.id),
            "name": role.name,
            "display_name": role.display_name,
            "permissions": role.permissions,
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.delete("/{role_id_or_name}")
async def delete_role(
    role_id_or_name: str,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Deletes custom role. System roles (super_admin, admin, agent) are protected."""
    try:
        res = await rbac_service.delete_custom_role(session, role_id_or_name)
        return {"success": res, "deleted_role": role_id_or_name}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/members/{member_id}/assign")
async def assign_member_role(
    member_id: str,
    payload: AssignMemberRoleIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Assigns an RBAC role and optional custom permissions to a team member."""
    try:
        member = await rbac_service.assign_member_role(
            session,
            member_id=member_id,
            role_name=payload.role,
            custom_permissions=payload.custom_permissions,
            actor=payload.actor,
        )
        return {
            "success": True,
            "member_id": str(member.id),
            "name": member.name,
            "role": member.role,
            "custom_permissions": member.custom_permissions,
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/members/{member_id}/check")
async def check_member_permission(
    member_id: str,
    permission: str = Query(..., description="Permission key e.g. billing:view"),
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Evaluates whether the team member has a specific permission."""
    allowed = await rbac_service.check_member_permission(session, member_id, permission)
    return {
        "member_id": member_id,
        "permission": permission,
        "allowed": allowed,
    }
