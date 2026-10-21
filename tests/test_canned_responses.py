from __future__ import annotations

import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead
from communicator.models.team_member import TeamMember


@pytest.mark.asyncio
async def test_canned_responses_crud(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch defaults (auto-seeded)
    list_res = await test_client.get("/api/v1/canned-responses")
    assert list_res.status_code == 200
    defaults = list_res.json()
    assert len(defaults) >= 5
    assert any(c["shortCode"] == "greet" for c in defaults)
    assert any(c["shortCode"] == "pricing" for c in defaults)

    # 2. Filter by search query
    search_res = await test_client.get("/api/v1/canned-responses?q=pricing")
    assert search_res.status_code == 200
    search_items = search_res.json()
    assert len(search_items) >= 1
    assert any(c["shortCode"] == "pricing" for c in search_items)

    # 3. Filter by category
    cat_res = await test_client.get("/api/v1/canned-responses?category=greetings")
    assert cat_res.status_code == 200
    cat_items = cat_res.json()
    assert all(c["category"] == "greetings" for c in cat_items)

    # 4. Create new canned snippet
    create_res = await test_client.post(
        "/api/v1/canned-responses",
        json={
            "short_code": "shipping",
            "content": "Hi {{customer.first_name}}, orders are dispatched within 24 hours via express courier.",
            "category": "shipping",
        },
    )
    assert create_res.status_code == 200
    created = create_res.json()
    assert created["shortCode"] == "shipping"
    c_id = created["id"]

    # 5. Duplicate shortcode returns 400
    dup_res = await test_client.post(
        "/api/v1/canned-responses",
        json={
            "short_code": "/shipping",
            "content": "Duplicate content",
            "category": "shipping",
        },
    )
    assert dup_res.status_code == 400

    # 6. Update snippet
    update_res = await test_client.put(
        f"/api/v1/canned-responses/{c_id}",
        json={
            "content": "Hi {{customer.first_name}}, orders are dispatched within 12 hours with tracking.",
        },
    )
    assert update_res.status_code == 200
    assert "12 hours" in update_res.json()["content"]

    # 7. Delete snippet
    del_res = await test_client.delete(f"/api/v1/canned-responses/{c_id}")
    assert del_res.status_code == 200
    assert del_res.json()["deleted"] is True


@pytest.mark.asyncio
async def test_canned_response_variable_expansion(test_client: AsyncClient, db_session: AsyncSession):
    # Setup contact, lead, and agent
    lead = Lead(harvester_institution_id="test_lead_cr", name="Bruce Wayne", institution_type="vip")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155550999",
        person_name="Bruce Wayne",
    )
    db_session.add(contact)
    await db_session.flush()

    agent = TeamMember(name="Alfred Pennyworth", email="alfred@wayne.com")
    db_session.add(agent)
    await db_session.flush()

    conv = Conversation(
        contact_id=contact.id,
        assigned_to_id=agent.id,
    )
    db_session.add(conv)
    await db_session.flush()

    # Expand /greet with conversation context
    expand_res = await test_client.post(
        "/api/v1/canned-responses/expand",
        json={
            "short_code": "greet",
            "conversation_id": str(conv.id),
        },
    )
    assert expand_res.status_code == 200
    exp = expand_res.json()
    assert exp["shortCode"] == "greet"
    assert "Hi Bruce!" in exp["renderedContent"]
    assert exp["variables"]["customer.name"] == "Bruce Wayne"
    assert exp["variables"]["customer.first_name"] == "Bruce"
    assert exp["variables"]["agent.name"] == "Alfred Pennyworth"
    assert exp["variables"]["channel"] == "Whatsapp"

    # Expand /closing with custom variables
    expand_closing = await test_client.post(
        "/api/v1/canned-responses/expand",
        json={
            "short_code": "closing",
            "conversation_id": str(conv.id),
            "custom_vars": {"agent.name": "Alfred"},
        },
    )
    assert expand_closing.status_code == 200
    assert "-- Alfred" in expand_closing.json()["renderedContent"]
