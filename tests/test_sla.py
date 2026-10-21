import uuid
from datetime import UTC, datetime, timedelta
import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus


async def _create_test_lead_and_conv(
    db_session: AsyncSession,
    priority: str = "urgent",
    created_minutes_ago: int = 0,
) -> tuple[Lead, Contact, Conversation]:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="SLA Test Org")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value=f"+1555{uuid.uuid4().hex[:7]}",
        person_name="Morgan Chase",
    )
    db_session.add(contact)
    await db_session.flush()

    conv_created = datetime.now(UTC) - timedelta(minutes=created_minutes_ago)
    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.AWAITING_REPLY,
        priority=priority,
        created_at=conv_created,
        updated_at=conv_created,
    )
    db_session.add(conv)
    await db_session.flush()
    return lead, contact, conv


@pytest.mark.asyncio
async def test_list_and_seed_sla_policies(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/sla/policies")
    assert res.status_code == 200
    policies = res.json()
    assert isinstance(policies, list)
    assert len(policies) >= 3
    names = [p["name"] for p in policies]
    assert any("Critical VIP" in n for n in names)
    assert any("High Priority" in n for n in names)
    assert any("Standard Customer Support" in n for n in names)


@pytest.mark.asyncio
async def test_create_and_delete_sla_policy(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Create
    res = await test_client.post(
        "/api/v1/sla/policies",
        json={
            "name": f"Enterprise Tier SLA {uuid.uuid4().hex[:4]}",
            "description": "Custom SLA for tier-1 enterprise partners",
            "first_response_time_threshold": 20,
            "resolution_time_threshold": 180,
            "only_during_business_hours": True,
            "priorities": ["urgent", "high"],
        },
    )
    assert res.status_code == 200
    created = res.json()
    policy_id = created["id"]
    assert "Enterprise Tier SLA" in created["name"]
    assert created["firstResponseTimeThreshold"] == 20
    assert created["resolutionTimeThreshold"] == 180

    # 2. Get
    get_res = await test_client.get(f"/api/v1/sla/policies/{policy_id}")
    assert get_res.status_code == 200
    assert get_res.json()["id"] == policy_id

    # 3. Update
    put_res = await test_client.put(
        f"/api/v1/sla/policies/{policy_id}",
        json={"first_response_time_threshold": 10},
    )
    assert put_res.status_code == 200
    assert put_res.json()["firstResponseTimeThreshold"] == 10

    # 4. Delete
    del_res = await test_client.delete(f"/api/v1/sla/policies/{policy_id}")
    assert del_res.status_code == 200
    assert del_res.json()["deleted"] is True


@pytest.mark.asyncio
async def test_evaluate_conversation_sla_on_time_fulfillment(test_client: AsyncClient, db_session: AsyncSession):
    _, _, conv = await _create_test_lead_and_conv(db_session, priority="urgent", created_minutes_ago=10)

    # Agent replied 3 minutes after conversation creation
    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.OUTBOUND,
        channel="whatsapp",
        body="Hello Morgan, I am immediately investigating your issue.",
        provider="whatsapp",
        provider_message_id=f"msg_{uuid.uuid4().hex[:8]}",
        status=MessageStatus.SENT,
        sent_at=conv.created_at + timedelta(minutes=3),
    )
    db_session.add(msg)
    # Mark resolved
    conv.state = ConversationState.CLOSED_WON
    conv.updated_at = conv.created_at + timedelta(minutes=45)
    await db_session.flush()

    res = await test_client.post(f"/api/v1/conversations/{conv.id}/sla/evaluate")
    assert res.status_code == 200
    data = res.json()
    sla = data["sla"]

    assert sla["firstResponseStatus"] == "fulfilled"
    assert sla["firstResponseBreached"] is False
    assert sla["resolutionStatus"] == "fulfilled"
    assert sla["resolutionBreached"] is False
    assert sla["isBreached"] is False
    assert "sla-breached" not in data["tags"]


@pytest.mark.asyncio
async def test_evaluate_conversation_sla_breach_detection(test_client: AsyncClient, db_session: AsyncSession):
    # Created 45 minutes ago with urgent priority (15-min target) and no reply
    _, _, conv = await _create_test_lead_and_conv(db_session, priority="urgent", created_minutes_ago=45)

    res = await test_client.post(f"/api/v1/conversations/{conv.id}/sla/evaluate")
    assert res.status_code == 200
    data = res.json()
    sla = data["sla"]

    assert sla["firstResponseBreached"] is True
    assert sla["firstResponseStatus"] == "breached"
    assert sla["isBreached"] is True
    # Auto-applied breach tag
    assert "sla-breached" in data["tags"]

    # Verify persisted in database
    await db_session.refresh(conv)
    assert conv.custom_attributes["sla"]["isBreached"] is True
    assert "sla-breached" in conv.custom_attributes.get("tags", [])


@pytest.mark.asyncio
async def test_sla_metrics_and_active_breaches(test_client: AsyncClient, db_session: AsyncSession):
    # Seed breach
    _, _, conv = await _create_test_lead_and_conv(db_session, priority="urgent", created_minutes_ago=60)
    await test_client.post(f"/api/v1/conversations/{conv.id}/sla/evaluate")

    # 1. Metrics
    res_m = await test_client.get("/api/v1/sla/metrics")
    assert res_m.status_code == 200
    metrics = res_m.json()
    assert metrics["totalTracked"] >= 1
    assert metrics["activeBreachesCount"] >= 1
    assert "firstResponseFulfillmentRate" in metrics
    assert "policyBreakdown" in metrics

    # 2. Breaches list
    res_b = await test_client.get("/api/v1/sla/breaches")
    assert res_b.status_code == 200
    breaches = res_b.json()
    assert isinstance(breaches, list)
    assert len(breaches) >= 1
    match = next(b for b in breaches if b["conversationId"] == str(conv.id))
    assert match["isBreached"] is True
    assert match["contactName"] == "Morgan Chase"
    assert match["priority"] == "urgent"
