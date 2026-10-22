import uuid

import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.team_member import TeamMember
from communicator.services import rbac_service


@pytest.mark.asyncio
async def test_default_system_roles_initialization(db_session):
    roles = await rbac_service.list_roles(db_session)
    assert len(roles) >= 5

    role_names = [r["name"] for r in roles]
    assert "super_admin" in role_names
    assert "admin" in role_names
    assert "agent" in role_names
    assert "restricted_agent" in role_names
    assert "compliance_auditor" in role_names

    super_admin = next(r for r in roles if r["name"] == "super_admin")
    agent = next(r for r in roles if r["name"] == "agent")

    assert len(super_admin["permissions"]) > len(agent["permissions"])
    assert "billing:manage" in super_admin["permissions"]
    assert "billing:manage" not in agent["permissions"]


@pytest.mark.asyncio
async def test_create_and_update_custom_role(db_session):
    # 1. Create custom role
    role = await rbac_service.create_custom_role(
        db_session,
        name="billing_specialist",
        display_name="Billing Specialist",
        description="Assigned to finance team to manage customer subscriptions",
        permissions=["billing:view", "billing:manage", "conversations:view", "conversations:reply"],
        actor="admin_dave",
    )
    assert role.id is not None
    assert role.name == "billing_specialist"
    assert "billing:manage" in role.permissions

    # 2. Update custom role permissions
    updated = await rbac_service.update_role(
        db_session,
        "billing_specialist",
        {"permissions": ["billing:view", "conversations:view"]},
        actor="admin_dave",
    )
    assert "billing:manage" not in updated.permissions
    assert "billing:view" in updated.permissions

    # 3. System roles cannot be deleted
    with pytest.raises(ValueError, match="protected"):
        await rbac_service.delete_custom_role(db_session, "super_admin")

    # 4. Custom role can be deleted
    deleted = await rbac_service.delete_custom_role(db_session, "billing_specialist")
    assert deleted is True


@pytest.mark.asyncio
async def test_member_role_assignment_and_permission_check(db_session):
    # 1. Create TeamMember
    member = TeamMember(
        name="Rachel Green",
        email=f"rachel_{uuid.uuid4().hex[:6]}@bayworks.com",
        role="agent",
    )
    db_session.add(member)
    await db_session.flush()

    # 2. Check default agent permissions
    assert (
        await rbac_service.check_member_permission(db_session, member.id, "conversations:reply")
        is True
    )
    assert (
        await rbac_service.check_member_permission(db_session, member.id, "billing:manage") is False
    )

    # 3. Elevate to super_admin
    await rbac_service.assign_member_role(db_session, member.id, "super_admin")
    assert (
        await rbac_service.check_member_permission(db_session, member.id, "billing:manage") is True
    )

    # 4. Restrict to restricted_agent with custom export permission
    await rbac_service.assign_member_role(
        db_session,
        member.id,
        "restricted_agent",
        custom_permissions=["contacts:export"],
    )
    assert (
        await rbac_service.check_member_permission(db_session, member.id, "contacts:export") is True
    )
    assert (
        await rbac_service.check_member_permission(db_session, member.id, "conversations:delete")
        is False
    )


@pytest.mark.asyncio
async def test_roles_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. GET /api/v1/roles
        r_list = await client.get("/api/v1/roles")
        assert r_list.status_code == 200
        roles = r_list.json()
        assert len(roles) >= 5

        # 2. GET /api/v1/roles/permissions
        r_perms = await client.get("/api/v1/roles/permissions")
        assert r_perms.status_code == 200
        perms = r_perms.json()
        assert len(perms) >= 15
        assert any(p["key"] == "conversations:reply" for p in perms)

        # 3. POST /api/v1/roles (create custom role)
        r_create = await client.post(
            "/api/v1/roles",
            json={
                "name": "tier1_triage",
                "display_name": "Tier 1 Triage Specialist",
                "description": "Frontline intake and triage team",
                "permissions": ["conversations:view", "conversations:assign", "contacts:view"],
            },
        )
        assert r_create.status_code == 200
        created = r_create.json()
        assert created["name"] == "tier1_triage"

        # 4. Assign role to team member via API
        m = TeamMember(name="Monica Geller", email="monica@friends.com", role="agent")
        db_session.add(m)
        await db_session.flush()

        r_assign = await client.post(
            f"/api/v1/roles/members/{m.id}/assign",
            json={"role": "tier1_triage", "custom_permissions": ["contacts:edit"]},
        )
        assert r_assign.status_code == 200
        assign_data = r_assign.json()
        assert assign_data["role"] == "tier1_triage"

        # 5. Check permission via API
        r_check = await client.get(f"/api/v1/roles/members/{m.id}/check?permission=contacts:edit")
        assert r_check.status_code == 200
        assert r_check.json()["allowed"] is True

        # 6. DELETE custom role
        r_del = await client.delete("/api/v1/roles/tier1_triage")
        assert r_del.status_code == 200
        assert r_del.json()["success"] is True
