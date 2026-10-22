import uuid
from datetime import UTC, datetime, timedelta

import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.team_member import TeamMember


async def _create_test_conv(
    db_session: AsyncSession, assigned_to_id: uuid.UUID | None = None
) -> Conversation:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Routing Org")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value=f"+1555{uuid.uuid4().hex[:7]}",
        person_name="Routing Customer",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.AWAITING_REPLY,
        priority="medium",
        assigned_to_id=assigned_to_id,
    )
    db_session.add(conv)
    await db_session.flush()
    return conv


@pytest.mark.asyncio
async def test_list_and_seed_routing_agents(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/routing/agents")
    assert res.status_code == 200
    agents = res.json()
    assert isinstance(agents, list)
    assert len(agents) >= 4
    names = [a["name"] for a in agents]
    assert "Riley Brooks" in names
    assert "Jamie Cole" in names
    for a in agents:
        assert "availabilityStatus" in a
        assert "maxCapacity" in a
        assert "activeConversationsCount" in a
        assert "availableSlots" in a


@pytest.mark.asyncio
async def test_update_agent_status_and_capacity(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/routing/agents")
    agent = res.json()[0]
    agent_id = agent["id"]

    # 1. Update status to busy
    put_s = await test_client.put(
        f"/api/v1/routing/agents/{agent_id}/status",
        json={"status": "busy"},
    )
    assert put_s.status_code == 200
    assert put_s.json()["availabilityStatus"] == "busy"

    # 2. Update capacity
    put_c = await test_client.put(
        f"/api/v1/routing/agents/{agent_id}/capacity",
        json={"max_capacity": 12},
    )
    assert put_c.status_code == 200
    assert put_c.json()["maxCapacity"] == 12

    # 3. Invalid status check
    bad_res = await test_client.put(
        f"/api/v1/routing/agents/{agent_id}/status",
        json={"status": "sleeping"},
    )
    assert bad_res.status_code == 400


@pytest.mark.asyncio
async def test_auto_assign_capacity_based_least_loaded(
    test_client: AsyncClient, db_session: AsyncSession
):
    # Create two fresh dedicated online agents
    agent_busy = TeamMember(
        name=f"Busy Rep {uuid.uuid4().hex[:4]}",
        email="busy@test.com",
        availability_status="online",
        max_concurrent_conversations=5,
    )
    agent_free = TeamMember(
        name=f"Free Rep {uuid.uuid4().hex[:4]}",
        email="free@test.com",
        availability_status="online",
        max_concurrent_conversations=5,
    )
    db_session.add(agent_busy)
    db_session.add(agent_free)
    await db_session.flush()

    # Assign 2 active conversations to agent_busy
    await _create_test_conv(db_session, assigned_to_id=agent_busy.id)
    await _create_test_conv(db_session, assigned_to_id=agent_busy.id)

    # Create unassigned conversation to route
    target_conv = await _create_test_conv(db_session, assigned_to_id=None)

    # Route using capacity_based
    res = await test_client.post(
        f"/api/v1/conversations/{target_conv.id}/auto-assign",
        json={"strategy": "capacity_based"},
    )
    assert res.status_code == 200
    data = res.json()
    assert data["assigned"] is True
    # The least-loaded agent with 0 active conversations must be chosen
    assert data["agentId"] == str(agent_free.id)
    assert data["agentName"] == agent_free.name


@pytest.mark.asyncio
async def test_auto_assign_round_robin(test_client: AsyncClient, db_session: AsyncSession):
    now = datetime.now(UTC)
    agent_older = TeamMember(
        name=f"Older Rep {uuid.uuid4().hex[:4]}",
        email="older@test.com",
        availability_status="online",
        max_concurrent_conversations=5,
        last_assigned_at=now - timedelta(minutes=30),
    )
    agent_recent = TeamMember(
        name=f"Recent Rep {uuid.uuid4().hex[:4]}",
        email="recent@test.com",
        availability_status="online",
        max_concurrent_conversations=5,
        last_assigned_at=now - timedelta(minutes=5),
    )
    db_session.add(agent_older)
    db_session.add(agent_recent)
    await db_session.flush()

    target_conv = await _create_test_conv(db_session, assigned_to_id=None)

    # Auto assign with round_robin
    res = await test_client.post(
        f"/api/v1/conversations/{target_conv.id}/auto-assign",
        json={"strategy": "round_robin"},
    )
    assert res.status_code == 200
    data = res.json()
    assert data["assigned"] is True
    # The agent who received a conversation longer ago must be selected
    assert data["agentId"] == str(agent_older.id)


@pytest.mark.asyncio
async def test_auto_assign_capacity_overflow_fallback(
    test_client: AsyncClient, db_session: AsyncSession
):
    # Set all existing agents to offline or max capacity 0
    agents_res = await test_client.get("/api/v1/routing/agents")
    for a in agents_res.json():
        await test_client.put(
            f"/api/v1/routing/agents/{a['id']}/status",
            json={"status": "offline"},
        )

    # Create unassigned conversation
    conv = await _create_test_conv(db_session, assigned_to_id=None)

    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/auto-assign",
        json={"strategy": "capacity_based"},
    )
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "overflow"
    assert data["assigned"] is False
    assert "team-overflow" in data["tags"]

    # Verify persisted in database
    await db_session.refresh(conv)
    assert "team-overflow" in conv.custom_attributes.get("tags", [])


@pytest.mark.asyncio
async def test_batch_auto_assign_and_capacity_overview(
    test_client: AsyncClient, db_session: AsyncSession
):
    # Make at least one agent online
    agents_res = await test_client.get("/api/v1/routing/agents")
    agent_id = agents_res.json()[0]["id"]
    await test_client.put(
        f"/api/v1/routing/agents/{agent_id}/status",
        json={"status": "online"},
    )
    await test_client.put(
        f"/api/v1/routing/agents/{agent_id}/capacity",
        json={"max_capacity": 10},
    )

    # Create 3 unassigned conversations
    await _create_test_conv(db_session, assigned_to_id=None)
    await _create_test_conv(db_session, assigned_to_id=None)

    # Batch assign
    res_b = await test_client.post(
        "/api/v1/routing/auto-assign-batch",
        json={"strategy": "capacity_based"},
    )
    assert res_b.status_code == 200
    batch_data = res_b.json()
    assert batch_data["totalProcessed"] >= 2
    assert batch_data["assignedCount"] >= 1

    # Capacity overview
    res_o = await test_client.get("/api/v1/routing/capacity-overview")
    assert res_o.status_code == 200
    overview = res_o.json()
    assert overview["totalAgents"] >= 1
    assert "onlineCapacity" in overview
    assert "currentActiveConversations" in overview
