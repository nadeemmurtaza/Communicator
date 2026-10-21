from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.api.dependencies import get_db_session
from communicator.models.broadcast import Broadcast
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.message import Message
from communicator.models.sequence import FollowUpSequence
from communicator.services.followup_scheduler import enroll_conversation

router = APIRouter(prefix="/conversations", tags=["Conversations"])


@router.get("")
async def list_conversations(session: AsyncSession = Depends(get_db_session)):
    stmt = (
        select(Conversation)
        .options(selectinload(Conversation.contact), selectinload(Conversation.broadcast))
        .order_by(Conversation.updated_at.desc())
    )
    conversations = (await session.execute(stmt)).scalars().all()
    return [
        {
            "id": str(c.id),
            "contactId": str(c.contact_id),
            "contactName": c.contact.person_name if c.contact and c.contact.person_name else "Contact",
            "contactPhone": c.contact.value if c.contact else "",
            "channel": c.contact.channel.value if c.contact else "whatsapp",
            "state": c.state.value,
            "resolutionType": c.resolution_type,
            "broadcastId": str(c.broadcast_id) if c.broadcast_id else None,
            "broadcastName": c.broadcast.name if c.broadcast else None,
            "csatScore": c.csat_score,
            "humanTakeover": c.human_takeover,
            "assignedToId": str(c.assigned_to_id) if c.assigned_to_id else None,
            "updatedAt": c.updated_at.isoformat(),
        }
        for c in conversations
    ]


@router.get("/{conversation_id}")
async def get_conversation(
    conversation_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    stmt = (
        select(Conversation)
        .options(selectinload(Conversation.contact), selectinload(Conversation.broadcast))
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

    return {
        "id": str(conversation.id),
        "contactId": str(conversation.contact_id),
        "contactName": conversation.contact.person_name if conversation.contact and conversation.contact.person_name else "Contact",
        "contactPhone": conversation.contact.value if conversation.contact else "",
        "channel": conversation.contact.channel.value if conversation.contact else "whatsapp",
        "state": conversation.state.value,
        "resolutionType": conversation.resolution_type,
        "campaign": campaign_info,
        "csatScore": conversation.csat_score,
        "humanTakeover": conversation.human_takeover,
        "assignedToId": str(conversation.assigned_to_id) if conversation.assigned_to_id else None,
        "messages": [
            {
                "id": str(m.id),
                "direction": m.direction.value,
                "channel": m.channel.value,
                "body": m.body,
                "intent": m.intent,
                "status": m.status.value,
                "sentAt": m.sent_at.isoformat(),
            }
            for m in messages
        ],
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
