from __future__ import annotations

import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection
from communicator.models.team_member import TeamMember


@pytest.mark.asyncio
async def test_create_and_approve_refund_workflow(
    test_client: AsyncClient, db_session: AsyncSession
):
    # 1. Setup Lead, Contact, Agent, and Conversation
    lead = Lead(
        harvester_institution_id="test_inst_approval_1", name="Bruce Wayne", institution_type="vip"
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155550101",
        person_name="Bruce Wayne",
    )
    db_session.add(contact)
    await db_session.flush()

    agent = TeamMember(name="Alfred Pennyworth", email="alfred@waynecorp.com")
    db_session.add(agent)
    await db_session.flush()

    conv = Conversation(
        contact_id=contact.id,
        assigned_to_id=agent.id,
    )
    db_session.add(conv)
    await db_session.flush()

    # 2. AI Agent creates a pending Refund Approval Request
    create_res = await test_client.post(
        "/api/v1/copilot/approvals",
        json={
            "conversation_id": str(conv.id),
            "title": "Approve $89.00 Refund for Bruce Wayne",
            "proposed_reply": "Hi Bruce, I've reviewed your request and approved a full refund of $89.00 for order #ORD-4912. Funds will arrive in 3-5 days.",
            "request_type": "refund",
            "action_data": {
                "amount": 89.00,
                "currency": "USD",
                "order_id": "ORD-4912",
                "reason": "Damaged goods on delivery",
            },
            "internal_note": "Customer requested full refund. Policy >$50 requires manager sign-off before dispatch.",
        },
    )
    assert create_res.status_code == 200
    approval_data = create_res.json()
    assert approval_data["status"] == "pending"
    assert approval_data["requestType"] == "refund"
    assert approval_data["actionData"]["amount"] == 89.00
    approval_id = approval_data["id"]

    # Verify an internal private note was posted in conversation thread
    msg_stmt = select(Message).where(Message.conversation_id == conv.id, Message.is_private == True)
    notes = list((await db_session.execute(msg_stmt)).scalars().all())
    assert any("[Copilot Approval Requested]" in n.body for n in notes)

    # 3. Teammate reviews and approves: Agent finishes the conversation itself
    approve_res = await test_client.post(
        f"/api/v1/copilot/approvals/{approval_id}/approve",
        json={
            "reviewer_name": "Alfred Pennyworth",
            "execute_send": True,
        },
    )
    assert approve_res.status_code == 200
    approved_body = approve_res.json()
    assert approved_body["status"] == "approved"
    assert approved_body["reviewerName"] == "Alfred Pennyworth"
    assert approved_body["dispatchedMessageId"] is not None

    # Verify outbound customer message was dispatched to WhatsApp
    outbound_stmt = select(Message).where(
        Message.conversation_id == conv.id,
        Message.direction == MessageDirection.OUTBOUND,
        Message.is_private == False,
    )
    outbound_msgs = list((await db_session.execute(outbound_stmt)).scalars().all())
    assert len(outbound_msgs) >= 1
    dispatched = outbound_msgs[-1]
    assert "approved a full refund of $89.00" in dispatched.body
    assert dispatched.sender_type == "copilot_agent"


@pytest.mark.asyncio
async def test_teammate_amends_reply_and_approves(
    test_client: AsyncClient, db_session: AsyncSession
):
    lead = Lead(harvester_institution_id="test_inst_approval_2", name="Selina Kyle")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.EMAIL,
        value="selina@gotham.com",
        person_name="Selina Kyle",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    # Create approval
    create_res = await test_client.post(
        "/api/v1/copilot/approvals",
        json={
            "conversation_id": str(conv.id),
            "title": "Approve Discount for Selina Kyle",
            "proposed_reply": "Hi Selina, we can offer 10% off your purchase.",
            "request_type": "discount",
            "action_data": {"discount_percent": 10},
        },
    )
    assert create_res.status_code == 200
    approval_id = create_res.json()["id"]

    # Teammate amends reply to 15% VIP discount
    amended_text = (
        "Hi Selina, as a valued client we are pleased to offer you an exclusive 15% VIP discount!"
    )
    approve_res = await test_client.post(
        f"/api/v1/copilot/approvals/{approval_id}/approve",
        json={
            "reviewer_name": "Lucius Fox",
            "amended_reply": amended_text,
            "execute_send": True,
        },
    )
    assert approve_res.status_code == 200
    res_data = approve_res.json()
    assert res_data["status"] == "amended"
    assert res_data["amendedReply"] == amended_text

    # Verify amended reply was dispatched to customer
    outbound_stmt = select(Message).where(
        Message.conversation_id == conv.id,
        Message.direction == MessageDirection.OUTBOUND,
        Message.is_private == False,
    )
    outbound_msgs = list((await db_session.execute(outbound_stmt)).scalars().all())
    assert any("exclusive 15% VIP discount" in m.body for m in outbound_msgs)


@pytest.mark.asyncio
async def test_teammate_rejects_approval(test_client: AsyncClient, db_session: AsyncSession):
    lead = Lead(harvester_institution_id="test_inst_approval_3", name="Edward Nygma")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155550303",
        person_name="Edward Nygma",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    create_res = await test_client.post(
        "/api/v1/copilot/approvals",
        json={
            "conversation_id": str(conv.id),
            "title": "Approve $500 Replacement Credit",
            "proposed_reply": "Hi Edward, we have credited $500 to your account.",
            "request_type": "policy_exception",
        },
    )
    approval_id = create_res.json()["id"]

    reject_res = await test_client.post(
        f"/api/v1/copilot/approvals/{approval_id}/reject",
        json={
            "reviewer_name": "James Gordon",
            "feedback": "Item was purchased over 90 days ago. Not eligible for replacement credit.",
        },
    )
    assert reject_res.status_code == 200
    rejected_body = reject_res.json()
    assert rejected_body["status"] == "rejected"
    assert (
        rejected_body["reviewerFeedback"]
        == "Item was purchased over 90 days ago. Not eligible for replacement credit."
    )

    # Verify no customer outbound message was dispatched
    outbound_stmt = select(Message).where(
        Message.conversation_id == conv.id,
        Message.direction == MessageDirection.OUTBOUND,
        Message.is_private == False,
    )
    outbound_msgs = list((await db_session.execute(outbound_stmt)).scalars().all())
    assert len(outbound_msgs) == 0


@pytest.mark.asyncio
async def test_ai_asks_teammate_info_and_finishes_itself(
    test_client: AsyncClient, db_session: AsyncSession
):
    lead = Lead(harvester_institution_id="test_inst_approval_4", name="Barbara Gordon")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155550404",
        person_name="Barbara Gordon",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    # Agent asks teammate for information
    create_res = await test_client.post(
        "/api/v1/copilot/approvals",
        json={
            "conversation_id": str(conv.id),
            "title": "Stock Check for SKU-TITAN-X",
            "proposed_reply": "",
            "request_type": "info_request",
            "question_to_teammate": "Does our central warehouse have replacement units for SKU-TITAN-X?",
            "internal_note": "Customer wants to know if they can get immediate replacement.",
        },
    )
    assert create_res.status_code == 200
    approval_id = create_res.json()["id"]

    # Teammate answers the question
    info_res = await test_client.post(
        f"/api/v1/copilot/approvals/{approval_id}/provide-info",
        json={
            "reviewer_name": "Lucius Fox",
            "teammate_answer": "Yes, we have 6 units in Bay 4 ready for next-day dispatch.",
        },
    )
    assert info_res.status_code == 200
    info_body = info_res.json()
    assert info_body["status"] == "approved"
    assert (
        info_body["teammateAnswer"] == "Yes, we have 6 units in Bay 4 ready for next-day dispatch."
    )

    # Verify Agent finished the conversation itself by sending synthesized reply to customer
    outbound_stmt = select(Message).where(
        Message.conversation_id == conv.id,
        Message.direction == MessageDirection.OUTBOUND,
        Message.is_private == False,
    )
    outbound_msgs = list((await db_session.execute(outbound_stmt)).scalars().all())
    assert len(outbound_msgs) >= 1
    assert "6 units in Bay 4 ready for next-day dispatch" in outbound_msgs[-1].body
    assert outbound_msgs[-1].sender_type == "copilot_agent"


@pytest.mark.asyncio
async def test_evaluate_and_trigger_refund_approval(
    test_client: AsyncClient, db_session: AsyncSession
):
    lead = Lead(harvester_institution_id="test_inst_approval_5", name="Harvey Dent")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155550505",
        person_name="Harvey Dent",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    # Evaluate incoming message requesting a refund
    trigger_res = await test_client.post(
        "/api/v1/copilot/approvals/evaluate-trigger",
        json={
            "conversation_id": str(conv.id),
            "customer_message": "Hello, my item arrived broken. I need a full refund of $150 for order ORD-8812 immediately.",
        },
    )
    assert trigger_res.status_code == 200
    trig_data = trigger_res.json()
    assert trig_data["triggered"] is True
    approval = trig_data["approval"]
    assert approval["requestType"] == "refund"
    assert approval["actionData"]["amount"] == 150.0
    assert approval["actionData"]["order_id"] == "ORD-8812"
    assert "150.00" in approval["proposedReply"]
    assert "ORD-8812" in approval["proposedReply"]
