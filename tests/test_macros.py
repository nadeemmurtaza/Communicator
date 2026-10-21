import uuid
import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead


async def _create_test_lead_and_conv(db_session: AsyncSession) -> tuple[Lead, Contact, Conversation]:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Macro Org")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value=f"+1555{uuid.uuid4().hex[:7]}",
        person_name="Alex Customer",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.AWAITING_REPLY,
        priority="medium",
    )
    db_session.add(conv)
    await db_session.flush()
    return lead, contact, conv


@pytest.mark.asyncio
async def test_list_and_seed_macros(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/macros")
    assert res.status_code == 200
    macros = res.json()
    assert isinstance(macros, list)
    assert len(macros) >= 4
    names = [m["name"] for m in macros]
    assert any("Resolve Ticket" in n for n in names)
    assert any("Escalate to Tier 2" in n for n in names)


@pytest.mark.asyncio
async def test_create_and_delete_macro(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Create custom macro
    res = await test_client.post(
        "/api/v1/macros",
        json={
            "name": f"VIP Discount Follow-up {uuid.uuid4().hex[:4]}",
            "description": "Applies VIP discount tag and changes priority to high",
            "actions": [
                {"type": "change_priority", "priority": "high"},
                {"type": "add_tags", "tags": ["vip", "discount-applied"]},
                {"type": "add_private_note", "note": "VIP discount approved by manager."},
            ],
        },
    )
    assert res.status_code == 200
    created = res.json()
    macro_id = created["id"]
    assert "VIP Discount" in created["name"]
    assert len(created["actions"]) == 3

    # 2. Get by ID
    get_res = await test_client.get(f"/api/v1/macros/{macro_id}")
    assert get_res.status_code == 200
    assert get_res.json()["id"] == macro_id

    # 3. Update macro
    put_res = await test_client.put(
        f"/api/v1/macros/{macro_id}",
        json={"description": "Updated description for VIP macro"},
    )
    assert put_res.status_code == 200
    assert put_res.json()["description"] == "Updated description for VIP macro"

    # 4. Delete macro
    del_res = await test_client.delete(f"/api/v1/macros/{macro_id}")
    assert del_res.status_code == 200
    assert del_res.json()["deleted"] is True


@pytest.mark.asyncio
async def test_execute_resolve_and_csat_macro(test_client: AsyncClient, db_session: AsyncSession):
    _, _, conv = await _create_test_lead_and_conv(db_session)

    # Get the Resolve macro
    res_list = await test_client.get("/api/v1/macros")
    macros = res_list.json()
    resolve_macro = next(m for m in macros if "Resolve Ticket" in m["name"])

    # Execute macro on conversation
    exec_res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/macros/{resolve_macro['id']}/execute",
        json={"agent_name": "Riley Brooks"},
    )
    assert exec_res.status_code == 200
    data = exec_res.json()
    assert data["status"] == "ok"
    assert data["conversationState"] == "closed_won"
    assert "resolved" in data["conversationTags"]
    assert data["executedActionsCount"] >= 3

    # Refresh DB conversation and verify
    await db_session.refresh(conv)
    assert conv.state == ConversationState.CLOSED_WON
    assert conv.resolution_type == "human_assisted"
    assert "csat_dispatched_at" in conv.custom_attributes
    assert "resolved" in conv.custom_attributes.get("tags", [])


@pytest.mark.asyncio
async def test_execute_escalation_macro(test_client: AsyncClient, db_session: AsyncSession):
    _, _, conv = await _create_test_lead_and_conv(db_session)

    res_list = await test_client.get("/api/v1/macros")
    macros = res_list.json()
    escalate_macro = next(m for m in macros if "Escalate to Tier 2" in m["name"])

    exec_res = await test_client.post(
        f"/api/v1/conversations/{conv.id}/macros/{escalate_macro['id']}/execute",
        json={"agent_name": "Riley Brooks"},
    )
    assert exec_res.status_code == 200
    data = exec_res.json()
    assert data["conversationPriority"] == "urgent"
    assert "escalated" in data["conversationTags"]

    await db_session.refresh(conv)
    assert conv.priority == "urgent"
    assert "last_macro_executed" in conv.custom_attributes
