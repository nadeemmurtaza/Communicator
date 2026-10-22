from __future__ import annotations

import logging
import uuid
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.rbac import RoleDefinition
from communicator.models.team_member import TeamMember
from communicator.services import audit_service

logger = logging.getLogger(__name__)

STANDARD_PERMISSIONS: dict[str, dict[str, str]] = {
    # Conversations
    "conversations:view": {
        "label": "View Conversations",
        "description": "View live and historical customer message threads",
        "category": "Conversations",
    },
    "conversations:reply": {
        "label": "Send Replies",
        "description": "Send public inbound/outbound responses across any connected channel",
        "category": "Conversations",
    },
    "conversations:assign": {
        "label": "Assign Conversations",
        "description": "Assign conversations to specific team members or routing queues",
        "category": "Conversations",
    },
    "conversations:delete": {
        "label": "Delete Conversations",
        "description": "Permanently purge conversation threads and customer message history",
        "category": "Conversations",
    },
    "conversations:export": {
        "label": "Export Transcripts",
        "description": "Export conversation history to CSV or JSON formats",
        "category": "Conversations",
    },
    "conversations:private_notes": {
        "label": "Internal Private Notes",
        "description": "Post internal private team notes and @mentions invisible to customers",
        "category": "Conversations",
    },
    # Contacts & CRM
    "contacts:view": {
        "label": "View Contacts",
        "description": "Access contact directory, profiles, phone numbers, and attributes",
        "category": "Contacts & CRM",
    },
    "contacts:edit": {
        "label": "Edit Contacts",
        "description": "Modify customer properties, custom attributes, and contact tags",
        "category": "Contacts & CRM",
    },
    "contacts:delete": {
        "label": "Delete Contacts",
        "description": "Delete contact profiles from database",
        "category": "Contacts & CRM",
    },
    "contacts:export": {
        "label": "Bulk Export Contacts",
        "description": "Bulk export entire customer and leads database to CSV",
        "category": "Contacts & CRM",
    },
    # Inboxes & Channels
    "inboxes:view": {
        "label": "View Inboxes",
        "description": "View active communication channels and live chat status",
        "category": "Inboxes & Channels",
    },
    "inboxes:manage": {
        "label": "Manage Channels",
        "description": "Add, edit, or configure channel credentials and live chat widget appearance",
        "category": "Inboxes & Channels",
    },
    # AI & Automation
    "ai:view": {
        "label": "View AI Copilot",
        "description": "View Captain AI suggestions, confidence metrics, and knowledge insights",
        "category": "AI & Automation",
    },
    "ai:manage": {
        "label": "Configure AI Rules",
        "description": "Manage Captain AI model providers, prompt guidelines, and retraining jobs",
        "category": "AI & Automation",
    },
    "macros:manage": {
        "label": "Manage Macros & Canned Responses",
        "description": "Create, edit, or delete canned responses and automation workflows",
        "category": "AI & Automation",
    },
    "broadcasts:send": {
        "label": "Send Campaigns",
        "description": "Compose and broadcast multi-channel marketing campaigns",
        "category": "AI & Automation",
    },
    # Security, Compliance & Administration
    "billing:view": {
        "label": "View Billing",
        "description": "Inspect Stripe subscription tier, usage, and invoices",
        "category": "Administration",
    },
    "billing:manage": {
        "label": "Manage Subscriptions",
        "description": "Change payment methods, upgrade plans, or cancel Stripe subscriptions",
        "category": "Administration",
    },
    "team:manage": {
        "label": "Manage Team Members",
        "description": "Invite agents, adjust capacity limits, and assign RBAC roles",
        "category": "Administration",
    },
    "roles:manage": {
        "label": "Manage RBAC Roles",
        "description": "Create, edit, and define custom enterprise permissions matrices",
        "category": "Administration",
    },
    "audit_logs:view": {
        "label": "Access SOC 2 Audit Logs",
        "description": "Inspect immutable security activity trails and export SOC 2 reports",
        "category": "Administration",
    },
    "webhooks:manage": {
        "label": "Manage Webhooks",
        "description": "Configure outgoing webhook endpoints, secrets, and event subscriptions",
        "category": "Administration",
    },
    "sso:manage": {
        "label": "Enterprise SSO & SAML",
        "description": "Configure SAML 2.0 Identity Providers, Google Workspace, and Okta SSO",
        "category": "Administration",
    },
}

DEFAULT_SYSTEM_ROLES: dict[str, dict[str, Any]] = {
    "super_admin": {
        "display_name": "Super Administrator",
        "description": "Unrestricted access across all workspace settings, billing, API keys, and security controls.",
        "permissions": list(STANDARD_PERMISSIONS.keys()),
    },
    "admin": {
        "display_name": "Administrator",
        "description": "Comprehensive operational control over channels, team members, macros, and routing. Excludes direct billing.",
        "permissions": [p for p in STANDARD_PERMISSIONS if not p.startswith("billing:")],
    },
    "agent": {
        "display_name": "Support Agent",
        "description": "Standard frontline agent: view and reply to conversations, add private notes, view contacts and AI suggestions.",
        "permissions": [
            "conversations:view",
            "conversations:reply",
            "conversations:assign",
            "conversations:private_notes",
            "contacts:view",
            "contacts:edit",
            "inboxes:view",
            "ai:view",
        ],
    },
    "restricted_agent": {
        "display_name": "Restricted Agent",
        "description": "Limited agent role: can reply to assigned conversations but cannot assign queues, delete data, or export records.",
        "permissions": [
            "conversations:view",
            "conversations:reply",
            "conversations:private_notes",
            "contacts:view",
            "inboxes:view",
        ],
    },
    "compliance_auditor": {
        "display_name": "Compliance Auditor",
        "description": "Read-only auditor access for SOC 2 reviews, activity trail inspections, and compliance certifications.",
        "permissions": [
            "audit_logs:view",
            "conversations:view",
            "contacts:view",
            "inboxes:view",
        ],
    },
}


async def initialize_default_roles(session: AsyncSession) -> None:
    """Seeds default enterprise RBAC roles if they do not yet exist in the database."""
    stmt = select(RoleDefinition)
    existing = {r.name: r for r in (await session.execute(stmt)).scalars().all()}

    for role_name, defn in DEFAULT_SYSTEM_ROLES.items():
        if role_name not in existing:
            role_obj = RoleDefinition(
                name=role_name,
                display_name=defn["display_name"],
                description=defn["description"],
                is_system_role=True,
                permissions=defn["permissions"],
            )
            session.add(role_obj)
        else:
            # Keep system role permissions synchronized
            role_obj = existing[role_name]
            role_obj.is_system_role = True
            if not role_obj.display_name:
                role_obj.display_name = defn["display_name"]
            if not role_obj.description:
                role_obj.description = defn["description"]

    await session.flush()


def get_permissions_catalog() -> list[dict[str, Any]]:
    """Returns the full master catalog of available system permissions grouped by category."""
    catalog = []
    for key, meta in STANDARD_PERMISSIONS.items():
        catalog.append(
            {
                "key": key,
                "label": meta["label"],
                "description": meta["description"],
                "category": meta["category"],
            }
        )
    return catalog


async def list_roles(session: AsyncSession) -> list[dict[str, Any]]:
    """Lists all role definitions with permission lists and member count."""
    await initialize_default_roles(session)

    roles = (
        (
            await session.execute(
                select(RoleDefinition).order_by(
                    RoleDefinition.is_system_role.desc(), RoleDefinition.name
                )
            )
        )
        .scalars()
        .all()
    )
    members = (await session.execute(select(TeamMember))).scalars().all()

    # Count members per role
    role_counts: dict[str, int] = {}
    for m in members:
        r_name = getattr(m, "role", "agent") or "agent"
        role_counts[r_name] = role_counts.get(r_name, 0) + 1

    return [
        {
            "id": str(r.id),
            "name": r.name,
            "display_name": r.display_name,
            "description": r.description,
            "is_system_role": r.is_system_role,
            "permissions": r.permissions,
            "permissions_count": len(r.permissions),
            "assigned_members_count": role_counts.get(r.name, 0),
            "created_at": r.created_at.isoformat() if r.created_at else "",
            "updated_at": r.updated_at.isoformat() if r.updated_at else "",
        }
        for r in roles
    ]


async def get_role(session: AsyncSession, role_id_or_name: str) -> dict[str, Any] | None:
    """Finds role by UUID or slug name."""
    await initialize_default_roles(session)

    try:
        r_uuid = uuid.UUID(role_id_or_name)
        stmt = select(RoleDefinition).where(RoleDefinition.id == r_uuid)
    except ValueError:
        stmt = select(RoleDefinition).where(RoleDefinition.name == role_id_or_name.strip())

    role = (await session.execute(stmt)).scalar_one_or_none()
    if not role:
        return None

    members = (
        (await session.execute(select(TeamMember).where(TeamMember.role == role.name)))
        .scalars()
        .all()
    )

    return {
        "id": str(role.id),
        "name": role.name,
        "display_name": role.display_name,
        "description": role.description,
        "is_system_role": role.is_system_role,
        "permissions": role.permissions,
        "permissions_count": len(role.permissions),
        "assigned_members_count": len(members),
        "created_at": role.created_at.isoformat() if role.created_at else "",
        "updated_at": role.updated_at.isoformat() if role.updated_at else "",
    }


async def create_custom_role(
    session: AsyncSession,
    name: str,
    display_name: str,
    description: str,
    permissions: list[str],
    actor: str = "admin",
) -> RoleDefinition:
    """Creates a custom enterprise RBAC role definition."""
    await initialize_default_roles(session)
    slug_name = name.strip().lower().replace(" ", "_")

    # Validate duplicate
    existing = (
        await session.execute(select(RoleDefinition).where(RoleDefinition.name == slug_name))
    ).scalar_one_or_none()
    if existing:
        raise ValueError(f"Role with identifier '{slug_name}' already exists.")

    # Validate permissions against master catalog
    valid_perms = [p for p in permissions if p in STANDARD_PERMISSIONS]

    role = RoleDefinition(
        name=slug_name,
        display_name=display_name.strip() or slug_name,
        description=description.strip(),
        is_system_role=False,
        permissions=valid_perms,
    )
    session.add(role)
    await session.flush()

    await audit_service.record_audit_event(
        session,
        action="role.created",
        entity_type="role",
        entity_id=role.id,
        actor=actor,
        severity="notice",
        compliance_category="access_control",
        description=f"Created custom enterprise role '{role.display_name}' with {len(valid_perms)} permissions",
        details={"role_name": role.name, "permissions": valid_perms},
    )

    return role


async def update_role(
    session: AsyncSession,
    role_id_or_name: str,
    updates: dict[str, Any],
    actor: str = "admin",
) -> RoleDefinition:
    """Updates role display name, description, or permissions set."""
    await initialize_default_roles(session)
    try:
        r_uuid = uuid.UUID(role_id_or_name)
        stmt = select(RoleDefinition).where(RoleDefinition.id == r_uuid)
    except ValueError:
        stmt = select(RoleDefinition).where(RoleDefinition.name == role_id_or_name.strip())

    role = (await session.execute(stmt)).scalar_one_or_none()
    if not role:
        raise ValueError(f"Role '{role_id_or_name}' not found.")

    if updates.get("display_name"):
        role.display_name = str(updates["display_name"]).strip()
    if "description" in updates and updates["description"] is not None:
        role.description = str(updates["description"]).strip()

    if "permissions" in updates and isinstance(updates["permissions"], list):
        # Do not allow removing all permissions from super_admin
        if role.name == "super_admin":
            role.permissions = list(STANDARD_PERMISSIONS.keys())
        else:
            valid_perms = [p for p in updates["permissions"] if p in STANDARD_PERMISSIONS]
            role.permissions = valid_perms

    await session.flush()

    await audit_service.record_audit_event(
        session,
        action="role.updated",
        entity_type="role",
        entity_id=role.id,
        actor=actor,
        severity="notice",
        compliance_category="access_control",
        description=f"Updated RBAC permissions for role '{role.display_name}'",
        details={"role_name": role.name, "permissions_count": len(role.permissions)},
    )

    return role


async def delete_custom_role(
    session: AsyncSession,
    role_id_or_name: str,
    actor: str = "admin",
) -> bool:
    """Deletes custom role. System roles (e.g. super_admin, agent) cannot be deleted."""
    await initialize_default_roles(session)
    try:
        r_uuid = uuid.UUID(role_id_or_name)
        stmt = select(RoleDefinition).where(RoleDefinition.id == r_uuid)
    except ValueError:
        stmt = select(RoleDefinition).where(RoleDefinition.name == role_id_or_name.strip())

    role = (await session.execute(stmt)).scalar_one_or_none()
    if not role:
        raise ValueError(f"Role '{role_id_or_name}' not found.")

    if role.is_system_role or role.name in DEFAULT_SYSTEM_ROLES:
        raise ValueError(f"System role '{role.display_name}' is protected and cannot be deleted.")

    # Reassign members with this role to standard agent role
    members = (
        (await session.execute(select(TeamMember).where(TeamMember.role == role.name)))
        .scalars()
        .all()
    )
    for m in members:
        m.role = "agent"

    role_id_str = str(role.id)
    role_name = role.name
    await session.delete(role)
    await session.flush()

    await audit_service.record_audit_event(
        session,
        action="role.deleted",
        entity_type="role",
        entity_id=role_id_str,
        actor=actor,
        severity="warning",
        compliance_category="access_control",
        description=f"Deleted custom RBAC role '{role_name}' and migrated {len(members)} members to standard agent",
        details={"deleted_role": role_name, "reassigned_members": len(members)},
    )

    return True


async def check_member_permission(
    session: AsyncSession,
    member_id: uuid.UUID | str,
    permission: str,
) -> bool:
    """
    Evaluates whether team member holds the specified permission (combining role permissions + custom perms).
    """
    await initialize_default_roles(session)
    safe_id = uuid.UUID(str(member_id))
    member = await session.get(TeamMember, safe_id)
    if not member:
        return False

    member_role = getattr(member, "role", "agent") or "agent"
    if member_role == "super_admin":
        return True

    # Retrieve permissions for assigned role
    role_stmt = select(RoleDefinition).where(RoleDefinition.name == member_role)
    role_obj = (await session.execute(role_stmt)).scalar_one_or_none()

    role_perms = set(role_obj.permissions if role_obj else [])
    custom_perms = set(getattr(member, "custom_permissions", []) or [])

    all_perms = role_perms.union(custom_perms)
    return permission in all_perms


async def assign_member_role(
    session: AsyncSession,
    member_id: uuid.UUID | str,
    role_name: str,
    custom_permissions: list[str] | None = None,
    actor: str = "admin",
) -> TeamMember:
    """Assigns an RBAC role and optional granular permission overrides to a team member."""
    await initialize_default_roles(session)
    safe_id = uuid.UUID(str(member_id))
    member = await session.get(TeamMember, safe_id)
    if not member:
        raise ValueError(f"Team member with ID '{member_id}' not found.")

    # Verify role exists
    role_stmt = select(RoleDefinition).where(RoleDefinition.name == role_name.strip())
    role_obj = (await session.execute(role_stmt)).scalar_one_or_none()
    if not role_obj:
        raise ValueError(f"Role '{role_name}' does not exist.")

    previous_role = getattr(member, "role", "agent")
    member.role = role_name.strip()
    if custom_permissions is not None:
        member.custom_permissions = [p for p in custom_permissions if p in STANDARD_PERMISSIONS]

    await session.flush()

    await audit_service.record_audit_event(
        session,
        action="member.role_assigned",
        entity_type="team_member",
        entity_id=member.id,
        actor=actor,
        severity="notice",
        compliance_category="access_control",
        description=f"Assigned role '{role_obj.display_name}' to {member.name} ({member.email})",
        details={
            "member_id": str(member.id),
            "member_name": member.name,
            "previous_role": previous_role,
            "new_role": role_name,
            "custom_permissions": getattr(member, "custom_permissions", []),
        },
    )

    return member
