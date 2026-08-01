from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.sequence import FollowUpSequence

router = APIRouter(prefix="/sequences", tags=["Sequences"])

_REPLIED_STATES = (
    ConversationState.ENGAGED,
    ConversationState.HOT_LEAD,
    ConversationState.HANDED_OFF,
    ConversationState.CLOSED_WON,
    ConversationState.CLOSED_LOST,
)


@router.get("")
async def list_sequences(session: AsyncSession = Depends(get_db_session)):
    sequences = (await session.execute(select(FollowUpSequence))).scalars().all()
    return [
        {"id": str(s.id), "name": s.name, "isActive": s.is_active}
        for s in sequences
    ]


@router.get("/{sequence_id}/variant-stats")
async def variant_stats(sequence_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    """
    Per-variant A/B results: how many Conversations were enrolled in each
    SequenceStep.variant, and how many progressed past a first reply /
    converted. Simple counts rather than statistical significance -- that's
    a call for whoever's reading the dashboard, not this endpoint.
    """
    sequence = await session.get(FollowUpSequence, sequence_id)
    if sequence is None:
        raise HTTPException(status_code=404, detail="FollowUpSequence not found")

    stmt = (
        select(
            Conversation.sequence_variant,
            func.count().label("enrolled"),
            func.count().filter(Conversation.state.in_(_REPLIED_STATES)).label("replied"),
            func.count()
            .filter(Conversation.state == ConversationState.CLOSED_WON)
            .label("closed_won"),
        )
        .where(Conversation.sequence_id == sequence_id)
        .group_by(Conversation.sequence_variant)
    )
    rows = (await session.execute(stmt)).all()

    return {
        "sequenceId": str(sequence.id),
        "variants": [
            {
                "variant": row.sequence_variant,
                "enrolled": row.enrolled,
                "replied": row.replied,
                "closedWon": row.closed_won,
            }
            for row in rows
        ],
    }
