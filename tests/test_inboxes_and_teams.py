import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead


@pytest.mark.asyncio
async def test_inboxes_and_whatsapp_channel(test_client: AsyncClient, db_session: AsyncSession):
    # Test GET inboxes
    res = await test_client.get("/api/v1/inboxes")
    assert res.status_code == 200
    inboxes = res.json()
    assert len(inboxes) >= 1
    whatsapp_inbox = next(i for i in inboxes if i["channel_type"] == "whatsapp")
    assert "WhatsApp" in whatsapp_inbox["name"]

    # Test GET whatsapp channel details
    res = await test_client.get("/api/v1/inboxes/channels/whatsapp")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "whatsapp"
    assert data["health"]["messagingLimitTier"] == "TIER_100K"
    assert "webhookConfig" in data

    # Test POST update whatsapp channel
    res = await test_client.post(
        "/api/v1/inboxes/channels/whatsapp",
        json={
            "phone_number_id": "999888777",
            "waba_id": "111222333",
            "access_token": "EAAG_TEST_TOKEN",
            "greeting_enabled": True,
            "greeting_message": "Hello from testing!",
        },
    )
    assert res.status_code == 200
    updated = res.json()
    assert updated["inbox"]["credentials"]["phone_number_id"] == "999888777"
    assert updated["inbox"]["greeting_message"] == "Hello from testing!"

    # Test WhatsApp Verify endpoint
    res = await test_client.post("/api/v1/inboxes/channels/whatsapp/verify")
    assert res.status_code == 200
    assert res.json()["status"] == "ok"


@pytest.mark.asyncio
async def test_whatsapp_templates_and_test_message(test_client: AsyncClient):
    # Test GET templates
    res = await test_client.get("/api/v1/inboxes/channels/whatsapp/templates")
    assert res.status_code == 200
    templates = res.json()
    assert isinstance(templates, list)
    assert len(templates) >= 1
    assert any(t["status"] == "APPROVED" for t in templates)

    # Test sync templates
    res = await test_client.post("/api/v1/inboxes/channels/whatsapp/sync-templates")
    assert res.status_code == 200
    assert res.json()["status"] in ("ok", "synced")

    # Test sending test message
    res = await test_client.post(
        "/api/v1/inboxes/channels/whatsapp/test-message",
        json={"recipient_phone": "+14155552480", "message": "Test WhatsApp notification"},
    )
    assert res.status_code == 200
    assert res.json()["status"] in ("sent", "simulated_sent")

    # Test list events
    res = await test_client.get("/api/v1/inboxes/channels/whatsapp/events")
    assert res.status_code == 200
    assert isinstance(res.json(), list)

    # Test GET WhatsApp Business Profile
    res = await test_client.get("/api/v1/inboxes/channels/whatsapp/profile")
    assert res.status_code == 200
    profile = res.json()
    assert "about" in profile
    assert "websites" in profile

    # Test POST update WhatsApp Business Profile
    res = await test_client.post(
        "/api/v1/inboxes/channels/whatsapp/profile",
        json={
            "about": "Updated Bay Works Support",
            "address": "456 Mission St, San Francisco, CA",
            "description": "Enterprise customer engagement platform.",
            "email": "support@bayworks.com",
            "vertical": "PROF_SERVICES",
            "websites": ["https://bayworks.com"],
        },
    )
    assert res.status_code == 200
    assert res.json()["status"] == "ok"
    assert res.json()["profile"]["about"] == "Updated Bay Works Support"

    # Test phone registration with 2-step PIN
    res = await test_client.post(
        "/api/v1/inboxes/channels/whatsapp/register",
        json={"pin": "654321"},
    )
    assert res.status_code == 200
    assert res.json()["status"] == "registered"


@pytest.mark.asyncio
async def test_teams_crud(test_client: AsyncClient):
    # List teams (seeds default teams if none)
    res = await test_client.get("/api/v1/teams")
    assert res.status_code == 200
    teams = res.json()
    assert len(teams) >= 3
    team_names = [t["name"] for t in teams]
    assert "Customer Support" in team_names

    # Create team
    res = await test_client.post(
        "/api/v1/teams",
        json={
            "name": "VIP Concierge",
            "description": "High tier enterprise accounts",
            "auto_assign": True,
        },
    )
    assert res.status_code == 200
    created = res.json()
    assert created["name"] == "VIP Concierge"
    team_id = created["id"]

    # Delete team
    res = await test_client.delete(f"/api/v1/teams/{team_id}")
    assert res.status_code == 200
    assert res.json()["status"] == "deleted"


@pytest.mark.asyncio
async def test_canned_responses_crud(test_client: AsyncClient):
    # List canned responses (auto seeds defaults: /greet, /pricing, etc.)
    res = await test_client.get("/api/v1/canned-responses")
    assert res.status_code == 200
    canned = res.json()
    assert len(canned) >= 4
    short_codes = [c["short_code"] for c in canned]
    assert "greet" in short_codes

    # Create new canned response
    res = await test_client.post(
        "/api/v1/canned-responses",
        json={
            "short_code": "shipping",
            "content": "Our standard shipping takes 2-4 business days.",
            "category": "general",
        },
    )
    assert res.status_code == 200
    new_canned = res.json()
    assert new_canned["short_code"] == "shipping"

    # Delete canned response
    res = await test_client.delete(f"/api/v1/canned-responses/{new_canned['id']}")
    assert res.status_code == 200
    assert res.json()["status"] == "deleted"


@pytest.mark.asyncio
async def test_conversation_actions_and_notes(test_client: AsyncClient, db_session: AsyncSession):
    # Create lead, contact & conversation
    lead = Lead(harvester_institution_id="inst_vip_001", name="Luxury Living Decor")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155552480",
        person_name="Maya VIP",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(
        contact_id=contact.id,
        priority="medium",
        custom_attributes={"account_tier": "VIP"},
    )
    db_session.add(conv)
    await db_session.flush()

    # 1. Test Private Note vs Customer Message
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/messages",
        json={
            "body": "Customer asks for 20% discount on linen set.",
            "is_private": True,
            "sender_name": "Riley Brooks",
        },
    )
    assert res.status_code == 200
    note_data = res.json()
    assert note_data["is_private"] is True
    assert note_data["sender_type"] == "agent"

    # 2. Test Priority Update
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/priority",
        json={"priority": "urgent"},
    )
    assert res.status_code == 200
    assert res.json()["priority"] == "urgent"

    # 3. Test Assign Team
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/assign-team",
        json={"team_id": None},
    )
    assert res.status_code == 200
    assert res.json()["status"] == "ok"

    # 4. Test Snooze
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/snooze",
        json={"snoozed_until": "2026-12-01T10:00:00Z"},
    )
    assert res.status_code == 200
    assert res.json()["status"] == "snoozed"

    # 5. Test Custom Attributes
    res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/custom-attributes",
        json={"attributes": {"account_tier": "Diamond", "ltv": 15000}},
    )
    assert res.status_code == 200
    assert res.json()["custom_attributes"]["account_tier"] == "Diamond"

    # 6. Test Contact Notes
    res = await test_client.post(
        f"/api/v1/conversations/contacts/{contact.id}/notes",
        json={
            "body": "Important client: Always provide direct manager contact.",
            "author_name": "Riley",
        },
    )
    assert res.status_code == 200
    note = res.json()
    assert note["body"] == "Important client: Always provide direct manager contact."

    res = await test_client.get(f"/api/v1/conversations/contacts/{contact.id}/notes")
    assert res.status_code == 200
    notes = res.json()
    assert len(notes) == 1
    assert notes[0]["body"] == "Important client: Always provide direct manager contact."
