from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.conversation import Conversation
from communicator.models.message import Message

router = APIRouter(prefix="/conversations", tags=["Conversations"])


@router.get("")
async def list_conversations(session: AsyncSession = Depends(get_db_session)):
    conversations = (await session.execute(select(Conversation))).scalars().all()
    return [
        {
            "id": str(c.id),
            "contactId": str(c.contact_id),
            "state": c.state.value,
            "updatedAt": c.updated_at.isoformat(),
        }
        for c in conversations
    ]


@router.get("/{conversation_id}")
async def get_conversation(conversation_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")

    messages = (
        await session.execute(
            select(Message).where(Message.conversation_id == conversation_id).order_by(Message.sent_at)
        )
    ).scalars().all()

    return {
        "id": str(conversation.id),
        "state": conversation.state.value,
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
