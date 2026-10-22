from __future__ import annotations

import re
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.models.approval import ApprovalStatus, ApprovalType, CopilotApprovalRequest
from communicator.models.contact import ChannelType
from communicator.models.conversation import Conversation
from communicator.models.message import Message, MessageDirection, MessageStatus


def serialize_approval(a: CopilotApprovalRequest) -> dict[str, Any]:
    return {
        "id": str(a.id),
        "conversationId": str(a.conversation_id),
        "conversation_id": str(a.conversation_id),
        "contactId": str(a.contact_id) if a.contact_id else None,
        "contact_id": str(a.contact_id) if a.contact_id else None,
        "requestType": a.request_type,
        "request_type": a.request_type,
        "status": a.status,
        "title": a.title,
        "proposedReply": a.proposed_reply,
        "proposed_reply": a.proposed_reply,
        "amendedReply": a.amended_reply,
        "amended_reply": a.amended_reply,
        "actionData": a.action_data or {},
        "action_data": a.action_data or {},
        "internalNote": a.internal_note,
        "internal_note": a.internal_note,
        "questionToTeammate": a.question_to_teammate,
        "question_to_teammate": a.question_to_teammate,
        "teammateAnswer": a.teammate_answer,
        "teammate_answer": a.teammate_answer,
        "assignedToId": str(a.assigned_to_id) if a.assigned_to_id else None,
        "reviewerName": a.reviewer_name,
        "reviewer_name": a.reviewer_name,
        "reviewerFeedback": a.reviewer_feedback,
        "reviewer_feedback": a.reviewer_feedback,
        "dispatchedMessageId": str(a.dispatched_message_id) if a.dispatched_message_id else None,
        "createdAt": a.created_at.isoformat(),
        "created_at": a.created_at.isoformat(),
        "updatedAt": a.updated_at.isoformat(),
        "updated_at": a.updated_at.isoformat(),
        "resolvedAt": a.resolved_at.isoformat() if a.resolved_at else None,
        "resolved_at": a.resolved_at.isoformat() if a.resolved_at else None,
    }


async def create_approval_request(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    title: str,
    proposed_reply: str,
    request_type: str = ApprovalType.REFUND.value,
    action_data: dict[str, Any] | None = None,
    internal_note: str = "",
    question_to_teammate: str | None = None,
    assigned_to_id: uuid.UUID | None = None,
) -> dict[str, Any]:
    """
    Creates a pending AI Copilot approval or info request.
    AI Agent asks a teammate before answering the customer, pausing outbound
    delivery until human sign-off.
    """
    conv = await session.get(
        Conversation,
        conversation_id,
        options=[selectinload(Conversation.contact)],
    )
    if not conv:
        raise ValueError(f"Conversation {conversation_id} not found")

    approval = CopilotApprovalRequest(
        conversation_id=conversation_id,
        contact_id=conv.contact_id,
        request_type=request_type.lower(),
        status=ApprovalStatus.PENDING.value,
        title=title,
        proposed_reply=proposed_reply,
        action_data=action_data or {},
        internal_note=internal_note
        or f"AI Agent requests teammate authorization for {request_type} action.",
        question_to_teammate=question_to_teammate,
        assigned_to_id=assigned_to_id or conv.assigned_to_id,
    )
    session.add(approval)
    await session.flush()

    # Post an internal private note in the shared conversation record
    note_body = (
        f"🤖 [Copilot Approval Requested] {title}\n"
        f"Type: {request_type.upper()} • Status: PENDING TEAMMATE APPROVAL\n"
        f"Note: {approval.internal_note}\n"
        f'Proposed Reply: "{proposed_reply}"'
    )
    if question_to_teammate:
        note_body += f"\n❓ Question for teammate: {question_to_teammate}"

    channel = conv.contact.channel if conv.contact else ChannelType.WHATSAPP
    internal_msg = Message(
        conversation_id=conversation_id,
        direction=MessageDirection.OUTBOUND,
        channel=channel,
        body=note_body,
        is_private=True,
        sender_type="copilot_agent",
        sender_name="Captain AI Copilot",
        status=MessageStatus.SENT,
    )
    session.add(internal_msg)
    await session.flush()

    return serialize_approval(approval)


async def list_approval_requests(
    session: AsyncSession,
    conversation_id: uuid.UUID | None = None,
    status: str | None = None,
    request_type: str | None = None,
) -> list[dict[str, Any]]:
    stmt = select(CopilotApprovalRequest).order_by(desc(CopilotApprovalRequest.created_at))

    if conversation_id:
        stmt = stmt.where(CopilotApprovalRequest.conversation_id == conversation_id)
    if status and status.lower() != "all":
        stmt = stmt.where(CopilotApprovalRequest.status == status.lower())
    if request_type and request_type.lower() != "all":
        stmt = stmt.where(CopilotApprovalRequest.request_type == request_type.lower())

    results = (await session.execute(stmt)).scalars().all()
    return [serialize_approval(a) for a in results]


async def get_approval_request(session: AsyncSession, approval_id: uuid.UUID) -> dict[str, Any]:
    approval = await session.get(CopilotApprovalRequest, approval_id)
    if not approval:
        raise ValueError(f"Approval request {approval_id} not found")
    return serialize_approval(approval)


async def approve_request(
    session: AsyncSession,
    approval_id: uuid.UUID,
    reviewer_name: str = "Teammate",
    amended_reply: str | None = None,
    execute_send: bool = True,
) -> dict[str, Any]:
    """
    Teammate approves the request. Once approved, the AI Agent finishes
    the conversation itself by dispatching the reply to the customer on their channel.
    """
    approval = await session.get(
        CopilotApprovalRequest,
        approval_id,
        options=[
            selectinload(CopilotApprovalRequest.conversation).selectinload(Conversation.contact)
        ],
    )
    if not approval:
        raise ValueError(f"Approval request {approval_id} not found")

    if approval.status != ApprovalStatus.PENDING.value:
        raise ValueError(f"Approval request is already {approval.status}")

    conv = approval.conversation
    reply_to_send = (
        amended_reply.strip()
        if amended_reply and amended_reply.strip()
        else approval.proposed_reply
    )

    approval.status = (
        ApprovalStatus.AMENDED.value
        if amended_reply and amended_reply.strip() != approval.proposed_reply
        else ApprovalStatus.APPROVED.value
    )
    approval.amended_reply = amended_reply.strip() if amended_reply else None
    approval.reviewer_name = reviewer_name
    approval.resolved_at = datetime.now(UTC)

    # Agent finishes the conversation itself: Dispatches the approved reply to the customer
    if execute_send and reply_to_send:
        channel = conv.contact.channel if conv.contact else ChannelType.WHATSAPP
        outbound_msg = Message(
            conversation_id=conv.id,
            direction=MessageDirection.OUTBOUND,
            channel=channel,
            body=reply_to_send,
            is_private=False,
            sender_type="copilot_agent",
            sender_name="Captain AI (Approved)",
            status=MessageStatus.SENT,
        )
        session.add(outbound_msg)
        await session.flush()
        approval.dispatched_message_id = outbound_msg.id

        # Post internal confirmation audit note
        audit_note = Message(
            conversation_id=conv.id,
            direction=MessageDirection.OUTBOUND,
            channel=channel,
            body=f"✅ [Copilot Approved] {reviewer_name} approved {approval.request_type.upper()}. AI Agent dispatched reply to customer and completed the workflow.",
            is_private=True,
            sender_type="copilot_agent",
            sender_name="Captain AI Copilot",
            status=MessageStatus.SENT,
        )
        session.add(audit_note)
        await session.flush()

    return serialize_approval(approval)


async def reject_request(
    session: AsyncSession,
    approval_id: uuid.UUID,
    reviewer_name: str = "Teammate",
    feedback: str = "",
) -> dict[str, Any]:
    approval = await session.get(
        CopilotApprovalRequest,
        approval_id,
        options=[
            selectinload(CopilotApprovalRequest.conversation).selectinload(Conversation.contact)
        ],
    )
    if not approval:
        raise ValueError(f"Approval request {approval_id} not found")

    if approval.status != ApprovalStatus.PENDING.value:
        raise ValueError(f"Approval request is already {approval.status}")

    approval.status = ApprovalStatus.REJECTED.value
    approval.reviewer_name = reviewer_name
    approval.reviewer_feedback = feedback
    approval.resolved_at = datetime.now(UTC)

    # Post internal rejection note in the shared conversation record
    conv = approval.conversation
    channel = conv.contact.channel if conv.contact else ChannelType.WHATSAPP
    audit_note = Message(
        conversation_id=conv.id,
        direction=MessageDirection.OUTBOUND,
        channel=channel,
        body=f"❌ [Copilot Rejected] {reviewer_name} rejected the {approval.request_type.upper()} request. Reason: {feedback or 'No reason provided'}. AI Agent paused and will not dispatch the proposed reply.",
        is_private=True,
        sender_type="copilot_agent",
        sender_name="Captain AI Copilot",
        status=MessageStatus.SENT,
    )
    session.add(audit_note)
    await session.flush()

    return serialize_approval(approval)


async def provide_teammate_info(
    session: AsyncSession,
    approval_id: uuid.UUID,
    reviewer_name: str = "Teammate",
    teammate_answer: str = "",
) -> dict[str, Any]:
    """
    Teammate provides requested piece of info; Agent synthesizes and finishes the conversation itself.
    """
    approval = await session.get(
        CopilotApprovalRequest,
        approval_id,
        options=[
            selectinload(CopilotApprovalRequest.conversation).selectinload(Conversation.contact)
        ],
    )
    if not approval:
        raise ValueError(f"Approval request {approval_id} not found")

    conv = approval.conversation
    contact_name = (
        conv.contact.person_name.split()[0]
        if conv.contact and conv.contact.person_name
        else "there"
    )

    # AI synthesizes the answer from teammate and customer context
    final_reply = (
        f"Hi {contact_name}! I checked with our team regarding your question: {teammate_answer.strip()} "
        "Please let us know if you have any questions or if there is anything else I can do for you!"
    )

    approval.status = ApprovalStatus.APPROVED.value
    approval.reviewer_name = reviewer_name
    approval.teammate_answer = teammate_answer.strip()
    approval.amended_reply = final_reply
    approval.resolved_at = datetime.now(UTC)

    channel = conv.contact.channel if conv.contact else ChannelType.WHATSAPP
    outbound_msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.OUTBOUND,
        channel=channel,
        body=final_reply,
        is_private=False,
        sender_type="copilot_agent",
        sender_name="Captain AI (Finished)",
        status=MessageStatus.SENT,
    )
    session.add(outbound_msg)
    await session.flush()
    approval.dispatched_message_id = outbound_msg.id

    audit_note = Message(
        conversation_id=conv.id,
        direction=MessageDirection.OUTBOUND,
        channel=channel,
        body=f'✅ [Teammate Info Answered] {reviewer_name} answered: "{teammate_answer.strip()}". Agent Captain finalized and dispatched response to customer.',
        is_private=True,
        sender_type="copilot_agent",
        sender_name="Captain AI Copilot",
        status=MessageStatus.SENT,
    )
    session.add(audit_note)
    await session.flush()

    return serialize_approval(approval)


async def evaluate_and_trigger_refund_approval(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    customer_message: str,
) -> dict[str, Any] | None:
    """
    Checks if an incoming customer message requests a refund or financial credit.
    If detected, automatically creates a pending approval request asking a teammate
    to approve the refund reply before answering the customer.
    """
    text_lower = customer_message.lower()
    is_refund = any(
        kw in text_lower
        for kw in ["refund", "money back", "reimburse", "cancel order", "defective item"]
    )
    if not is_refund:
        return None

    conv = await session.get(
        Conversation,
        conversation_id,
        options=[selectinload(Conversation.contact)],
    )
    if not conv:
        return None

    # Check if there's already an active pending approval for this conversation
    existing_stmt = select(CopilotApprovalRequest).where(
        CopilotApprovalRequest.conversation_id == conversation_id,
        CopilotApprovalRequest.status == ApprovalStatus.PENDING.value,
    )
    existing = (await session.execute(existing_stmt)).scalar_one_or_none()
    if existing:
        return serialize_approval(existing)

    contact_name = (
        conv.contact.person_name if conv.contact and conv.contact.person_name else "Valued Customer"
    )
    first_name = contact_name.split()[0] if contact_name else "there"

    # Extract or simulate refund amount
    amount_match = re.search(r"\$?(\d+(?:\.\d{2})?)", customer_message)
    amount = float(amount_match.group(1)) if amount_match else 89.00

    order_match = re.search(r"(?:order|ord|#)\s*([A-Za-z0-9-]+)", customer_message, re.IGNORECASE)
    order_id = order_match.group(1) if order_match else "ORD-4912"

    proposed_reply = (
        f"Hi {first_name}, I have reviewed your request and approved a full refund of ${amount:.2f} "
        f"for order #{order_id}. The funds have been initiated back to your original payment method "
        "and will appear in 3 to 5 business days. Please let me know if I can assist with anything else!"
    )

    action_data = {
        "amount": amount,
        "currency": "USD",
        "order_id": order_id,
        "reason": "Customer reported product issue / requested return",
        "policy_rule": "Refunds > $50.00 require human teammate sign-off prior to execution",
    }

    internal_note = (
        f"Customer {contact_name} requested a refund of ${amount:.2f} for order #{order_id}. "
        "Company policy requires human teammate approval before answering customer and processing payment return."
    )

    return await create_approval_request(
        session=session,
        conversation_id=conversation_id,
        title=f"Approve ${amount:.2f} Refund for {contact_name}",
        proposed_reply=proposed_reply,
        request_type=ApprovalType.REFUND.value,
        action_data=action_data,
        internal_note=internal_note,
    )
