import uuid

import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection


async def _create_test_lead_and_contact(
    session: AsyncSession, name: str, channel: ChannelType, value: str
) -> tuple[Lead, Contact]:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name=f"Org for {name}")
    session.add(lead)
    await session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=channel,
        value=value,
        person_name=name,
    )
    session.add(contact)
    await session.flush()
    return lead, contact


@pytest.mark.asyncio
async def test_captain_suggested_reply(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Create a test contact & conversation
    _, contact = await _create_test_lead_and_contact(
        db_session, "Maya Chen", ChannelType.WHATSAPP, f"+1555000{uuid.uuid4().hex[:4]}"
    )

    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.AWAITING_REPLY,
    )
    db_session.add(conv)
    await db_session.flush()

    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.WHATSAPP,
        body="Does the spring discount cover the new linen collection?",
        provider="meta",
        provider_message_id=f"wamid_{uuid.uuid4().hex[:8]}",
    )
    db_session.add(msg)
    await db_session.flush()

    # Call suggested-reply endpoint
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/ai-copilot/suggested-reply",
        json={"instruction": "Mention VIP early access"},
    )
    assert res.status_code == 200
    data = res.json()
    assert data["conversationId"] == str(conv.id)
    assert "suggestedReply" in data
    assert len(data["suggestedReply"]) > 10
    assert data["confidence"] > 0.8
    assert "groundedSources" in data
    assert len(data["suggestedActions"]) >= 1


@pytest.mark.asyncio
async def test_captain_thread_summarization(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Create conversation with multi-turn messages
    _, contact = await _create_test_lead_and_contact(
        db_session,
        "Lorrie Trosdall",
        ChannelType.WEBSITE,
        f"test_{uuid.uuid4().hex[:6]}@example.com",
    )

    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.ENGAGED,
    )
    db_session.add(conv)
    await db_session.flush()

    m1 = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.WEBSITE,
        body="I'm getting a 401 Unauthorized error when accessing my dashboard session.",
        provider="web_widget",
        provider_message_id=f"m1_{uuid.uuid4().hex[:6]}",
    )
    m2 = Message(
        conversation_id=conv.id,
        direction=MessageDirection.OUTBOUND,
        channel=ChannelType.WEBSITE,
        body="Let me clear your active web session token from our server cache.",
        provider="web_widget",
        provider_message_id=f"m2_{uuid.uuid4().hex[:6]}",
    )
    m3 = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.WEBSITE,
        body="Thank you! It works perfectly now.",
        provider="web_widget",
        provider_message_id=f"m3_{uuid.uuid4().hex[:6]}",
    )
    db_session.add_all([m1, m2, m3])
    await db_session.flush()

    # Call summarize endpoint
    res = await test_client.post(f"/api/v1/conversations/{conv.id}/ai-copilot/summarize")
    assert res.status_code == 200
    data = res.json()
    assert data["conversationId"] == str(conv.id)
    assert "summary" in data
    assert len(data["bulletPoints"]) >= 2
    assert data["sentiment"] in ["positive", "neutral", "negative", "urgent"]
    assert "keyTopics" in data


@pytest.mark.asyncio
async def test_captain_tone_rewriter(test_client: AsyncClient):
    sample_text = "we fixed the bug and your token is working"

    # 1. Professional
    res_prof = await test_client.post(
        "/api/v1/ai-copilot/rewrite",
        json={"text": sample_text, "tone": "professional"},
    )
    assert res_prof.status_code == 200
    data_prof = res_prof.json()
    assert "Dear customer" in data_prof["rewritten"]
    assert data_prof["tone"] == "professional"

    # 2. Friendly
    res_fr = await test_client.post(
        "/api/v1/ai-copilot/rewrite",
        json={"text": sample_text, "tone": "friendly"},
    )
    assert res_fr.status_code == 200
    assert "😊" in res_fr.json()["rewritten"]

    # 3. Shorten
    res_sh = await test_client.post(
        "/api/v1/ai-copilot/rewrite",
        json={"text": "Please be advised that we fixed the bug.", "tone": "shorten"},
    )
    assert res_sh.status_code == 200
    assert "fixed the bug" in res_sh.json()["rewritten"]

    # 4. Expand
    res_ex = await test_client.post(
        "/api/v1/ai-copilot/rewrite",
        json={"text": sample_text, "tone": "expand"},
    )
    assert res_ex.status_code == 200
    assert "Furthermore" in res_ex.json()["rewritten"]


@pytest.mark.asyncio
async def test_captain_triage_engine(test_client: AsyncClient, db_session: AsyncSession):
    # Create an urgent billing dispute conversation
    _, contact = await _create_test_lead_and_contact(
        db_session,
        "Harrison Sterling",
        ChannelType.EMAIL,
        f"billing_{uuid.uuid4().hex[:6]}@corp.com",
    )

    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.AWAITING_REPLY,
        priority="medium",
    )
    db_session.add(conv)
    await db_session.flush()

    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.EMAIL,
        body="URGENT: We were charged double for our annual invoice! Please process a refund immediately.",
        provider="postmark",
        provider_message_id=f"msg_bill_{uuid.uuid4().hex[:6]}",
    )
    db_session.add(msg)
    await db_session.flush()

    # 1. Dry run triage (apply=False)
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/ai-copilot/triage",
        json={"apply": False},
    )
    assert res.status_code == 200
    triage = res.json()
    assert triage["priority"] == "urgent"
    assert triage["suggestedTeam"] == "Sales & Billing"
    assert "billing" in triage["suggestedTags"]
    assert triage["applied"] is False

    # 2. Apply triage (apply=True)
    res_apply = await test_client.post(
        f"/api/v1/conversations/{conv.id}/ai-copilot/triage",
        json={"apply": True},
    )
    assert res_apply.status_code == 200
    applied_data = res_apply.json()
    assert applied_data["applied"] is True

    # Check conversation was actually updated
    await db_session.refresh(conv)
    assert conv.priority == "urgent"
    assert "captain_triage" in conv.custom_attributes


@pytest.mark.asyncio
async def test_captain_monitors_api(test_client: AsyncClient):
    # 1. Fetch monitors
    res = await test_client.get("/api/v1/captain/monitors")
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert len(data["activeMonitors"]) >= 3
    assert len(data["emergingThemes"]) >= 2
    assert "positive" in data["sentimentTelemetry"]

    # 2. Create custom monitor
    res_create = await test_client.post(
        "/api/v1/captain/monitors",
        json={
            "name": "Checkout Cart Abandonment Alert",
            "topic": "ecommerce",
            "keywords": ["cart", "checkout", "abandon", "stripe error"],
            "threshold": 4,
        },
    )
    assert res_create.status_code == 200
    created = res_create.json()
    assert created["name"] == "Checkout Cart Abandonment Alert"
    assert created["threshold"] == 4

    # Verify custom monitor appears in list
    res_after = await test_client.get("/api/v1/captain/monitors")
    assert any(
        m["name"] == "Checkout Cart Abandonment Alert" for m in res_after.json()["activeMonitors"]
    )
