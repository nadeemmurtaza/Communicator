from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.api.dependencies import get_db_session
from communicator.channels import get_provider
from communicator.models.broadcast import Broadcast
from communicator.models.contact import ChannelType, Contact
from communicator.models.contact_note import ContactNote
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.models.sequence import FollowUpSequence
from communicator.models.team import Team
from communicator.models.team_member import TeamMember
from communicator.services.followup_scheduler import enroll_conversation
from communicator.services.messaging import NotSendableError, send_outbound_message

router = APIRouter(prefix="/conversations", tags=["Conversations"])


class MessageSendIn(BaseModel):
    body: str
    is_private: bool = False
    sender_name: str = "Agent"


class PriorityIn(BaseModel):
    priority: str  # urgent | high | medium | low


class AssignTeamIn(BaseModel):
    team_id: uuid.UUID | None


class AssignAgentIn(BaseModel):
    agent_id: uuid.UUID | None


class SnoozeIn(BaseModel):
    snoozed_until: datetime | None


class CustomAttributesIn(BaseModel):
    attributes: dict[str, Any]


class ContactNoteIn(BaseModel):
    body: str
    author_name: str = "Agent"


def _serialize_conversation(c: Conversation) -> dict[str, Any]:
    return {
        "id": str(c.id),
        "contactId": str(c.contact_id),
        "contactName": c.contact.person_name if c.contact and c.contact.person_name else "Contact",
        "contactPhone": c.contact.value if c.contact else "",
        "channel": c.contact.channel.value if c.contact else "whatsapp",
        "state": c.state.value,
        "priority": c.priority or "medium",
        "resolutionType": c.resolution_type,
        "broadcastId": str(c.broadcast_id) if c.broadcast_id else None,
        "broadcastName": c.broadcast.name if c.broadcast else None,
        "inboxId": str(c.inbox_id) if c.inbox_id else None,
        "teamId": str(c.team_id) if c.team_id else None,
        "teamName": c.team.name if c.team else None,
        "csatScore": c.csat_score,
        "humanTakeover": c.human_takeover,
        "assignedToId": str(c.assigned_to_id) if c.assigned_to_id else None,
        "assignedToName": c.assigned_to.name if c.assigned_to else None,
        "snoozedUntil": c.snoozed_until.isoformat() if c.snoozed_until else None,
        "customAttributes": c.custom_attributes or {},
        "language": c.contact.language if c.contact and c.contact.language else "en",
        "updatedAt": c.updated_at.isoformat(),
    }


@router.get("")
async def list_conversations(session: AsyncSession = Depends(get_db_session)):
    stmt = (
        select(Conversation)
        .options(
            selectinload(Conversation.contact),
            selectinload(Conversation.broadcast),
            selectinload(Conversation.team),
            selectinload(Conversation.assigned_to),
        )
        .order_by(Conversation.updated_at.desc())
    )
    conversations = (await session.execute(stmt)).scalars().all()
    return [_serialize_conversation(c) for c in conversations]


@router.get("/{conversation_id}")
async def get_conversation(
    conversation_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    stmt = (
        select(Conversation)
        .options(
            selectinload(Conversation.contact),
            selectinload(Conversation.broadcast),
            selectinload(Conversation.team),
            selectinload(Conversation.assigned_to),
        )
        .where(Conversation.id == conversation_id)
    )
    conversation = (await session.execute(stmt)).scalar_one_or_none()
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")

    messages = (
        (
            await session.execute(
                select(Message)
                .where(Message.conversation_id == conversation_id)
                .order_by(Message.sent_at)
            )
        )
        .scalars()
        .all()
    )

    campaign_info = None
    if conversation.broadcast:
        campaign_info = {
            "id": str(conversation.broadcast.id),
            "name": conversation.broadcast.name,
            "templateName": conversation.broadcast.template_name,
            "sentAt": conversation.broadcast.created_at.isoformat(),
            "status": conversation.broadcast.status.value,
        }

    data = _serialize_conversation(conversation)
    data["campaign"] = campaign_info
    data["messages"] = [
        {
            "id": str(m.id),
            "direction": m.direction.value,
            "channel": m.channel.value,
            "body": m.body,
            "intent": m.intent,
            "status": m.status.value,
            "isPrivate": m.is_private,
            "senderType": m.sender_type,
            "senderName": m.sender_name,
            "attachments": m.attachments or [],
            "sentAt": m.sent_at.isoformat(),
            "translation": (m.provider_metadata or {}).get("translation"),
            "providerMetadata": m.provider_metadata or {},
        }
        for m in messages
    ]
    return data


@router.post("/{conversation_id}/messages")
async def send_conversation_message(
    conversation_id: uuid.UUID,
    payload: MessageSendIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Sends an outbound message to customer, or creates an internal Private Note
    visible to the team only.
    """
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")

    contact = await session.get(Contact, conversation.contact_id)
    now = datetime.now(UTC)

    # 1. Internal Private Note (Team collaboration)
    if payload.is_private:
        note_msg = Message(
            conversation_id=conversation.id,
            direction=MessageDirection.OUTBOUND,
            channel=contact.channel,
            body=payload.body,
            provider="internal_note",
            provider_message_id=f"note_{uuid.uuid4().hex[:12]}",
            status=MessageStatus.DELIVERED,
            is_private=True,
            sender_type="agent",
            sender_name=payload.sender_name or "Agent",
            sent_at=now,
        )
        session.add(note_msg)
        conversation.updated_at = now
        await session.flush()
        return {
            "id": str(note_msg.id),
            "direction": note_msg.direction.value,
            "body": note_msg.body,
            "isPrivate": True,
            "is_private": True,
            "senderType": note_msg.sender_type,
            "sender_type": note_msg.sender_type,
            "senderName": note_msg.sender_name,
            "sender_name": note_msg.sender_name,
            "status": note_msg.status.value,
            "sentAt": note_msg.sent_at.isoformat(),
        }

    # 2. Outbound Customer Message
    if contact.channel in (
        ChannelType.WEBSITE,
        ChannelType.FACEBOOK,
        ChannelType.INSTAGRAM,
        ChannelType.TELEGRAM,
        ChannelType.LINE,
        ChannelType.VOICE,
        ChannelType.EMAIL,
    ):
        from communicator.compliance.optout import is_sendable, record_consent
        from communicator.models.consent import ConsentStatus

        if not await is_sendable(session, contact.id):
            await record_consent(
                session, contact.id, ConsentStatus.OPTED_IN, source=contact.channel.value
            )

    provider = await get_provider(contact.channel, session)
    try:
        msg = await send_outbound_message(session, contact, provider, payload.body)
    except NotSendableError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    msg.sender_type = "agent"
    msg.sender_name = payload.sender_name or "Agent"
    conversation.last_outbound_at = now
    conversation.updated_at = now
    await session.flush()

    return {
        "id": str(msg.id),
        "direction": msg.direction.value,
        "body": msg.body,
        "isPrivate": False,
        "is_private": False,
        "senderType": msg.sender_type,
        "sender_type": msg.sender_type,
        "senderName": msg.sender_name,
        "sender_name": msg.sender_name,
        "status": msg.status.value,
        "sentAt": msg.sent_at.isoformat(),
    }


@router.post("/{conversation_id}/priority")
async def update_priority(
    conversation_id: uuid.UUID,
    payload: PriorityIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Sets urgency priority: urgent | high | medium | low."""
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.priority = payload.priority.lower()
    conversation.updated_at = datetime.now(UTC)
    await session.flush()
    return {"id": str(conversation.id), "priority": conversation.priority}


@router.post("/{conversation_id}/assign-team")
async def assign_team(
    conversation_id: uuid.UUID,
    payload: AssignTeamIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Routes conversation to a specific team queue."""
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.team_id = payload.team_id
    conversation.updated_at = datetime.now(UTC)
    await session.flush()
    return {
        "status": "ok",
        "id": str(conversation.id),
        "teamId": str(conversation.team_id) if conversation.team_id else None,
        "team_id": str(conversation.team_id) if conversation.team_id else None,
    }


@router.post("/{conversation_id}/assign-agent")
async def assign_agent(
    conversation_id: uuid.UUID,
    payload: AssignAgentIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Assigns conversation to an agent."""
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.assigned_to_id = payload.agent_id
    conversation.updated_at = datetime.now(UTC)
    await session.flush()
    return {
        "status": "ok",
        "id": str(conversation.id),
        "assignedToId": str(conversation.assigned_to_id) if conversation.assigned_to_id else None,
        "assigned_to_id": str(conversation.assigned_to_id) if conversation.assigned_to_id else None,
    }


@router.post("/{conversation_id}/snooze")
async def snooze_conversation(
    conversation_id: uuid.UUID,
    payload: SnoozeIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Snoozes conversation until a future date/time."""
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.snoozed_until = payload.snoozed_until
    conversation.updated_at = datetime.now(UTC)
    await session.flush()
    return {
        "status": "snoozed",
        "id": str(conversation.id),
        "snoozedUntil": conversation.snoozed_until.isoformat() if conversation.snoozed_until else None,
        "snoozed_until": conversation.snoozed_until.isoformat() if conversation.snoozed_until else None,
    }


@router.post("/{conversation_id}/custom-attributes")
async def update_custom_attributes(
    conversation_id: uuid.UUID,
    payload: CustomAttributesIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Updates custom metadata attributes on the conversation."""
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    current = dict(conversation.custom_attributes or {})
    current.update(payload.attributes)
    conversation.custom_attributes = current
    conversation.updated_at = datetime.now(UTC)
    await session.flush()
    return {
        "status": "ok",
        "id": str(conversation.id),
        "customAttributes": conversation.custom_attributes,
        "custom_attributes": conversation.custom_attributes,
    }


@router.post("/{conversation_id}/resolve")
async def resolve_conversation(
    conversation_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
):
    """Marks a conversation resolved."""
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.state = ConversationState.CLOSED_WON
    if not conversation.resolution_type:
        conversation.resolution_type = "human_assisted"
    conversation.updated_at = datetime.now(UTC)
    await session.flush()
    return {"id": str(conversation.id), "state": conversation.state.value, "resolutionType": conversation.resolution_type}


@router.post("/{conversation_id}/enroll-sequence")
async def enroll_sequence(
    conversation_id: uuid.UUID,
    sequence_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
):
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")

    sequence = await session.get(FollowUpSequence, sequence_id)
    if sequence is None:
        raise HTTPException(status_code=404, detail="FollowUpSequence not found")

    await session.refresh(sequence, attribute_names=["steps"])
    enroll_conversation(conversation, sequence)
    await session.flush()

    return {
        "id": str(conversation.id),
        "sequenceId": str(sequence.id),
        "nextFollowupAt": conversation.next_followup_at.isoformat()
        if conversation.next_followup_at
        else None,
    }


# --- Contact Profile Notes ---


@router.get("/contacts/{contact_id}/notes")
async def list_contact_notes(
    contact_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    """Retrieves all internal notes on a customer profile."""
    stmt = (
        select(ContactNote)
        .where(ContactNote.contact_id == contact_id)
        .order_by(ContactNote.created_at.desc())
    )
    notes = (await session.execute(stmt)).scalars().all()
    return [
        {
            "id": str(n.id),
            "contactId": str(n.contact_id),
            "authorName": n.author_name,
            "body": n.body,
            "createdAt": n.created_at.isoformat(),
        }
        for n in notes
    ]


@router.post("/contacts/{contact_id}/notes")
async def add_contact_note(
    contact_id: uuid.UUID,
    payload: ContactNoteIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Adds a persistent internal note to a customer profile."""
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")

    note = ContactNote(
        contact_id=contact.id,
        author_name=payload.author_name or "Agent",
        body=payload.body.strip(),
    )
    session.add(note)
    await session.flush()
    return {
        "id": str(note.id),
        "contactId": str(note.contact_id),
        "authorName": note.author_name,
        "body": note.body,
        "createdAt": note.created_at.isoformat(),
    }
