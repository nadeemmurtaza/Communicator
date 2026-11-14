from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.contact import ChannelType
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.sequence import FollowUpSequence, SequenceStep

router = APIRouter(prefix="/sequences", tags=["FollowUpSequences"])

_REPLIED_STATES = (
    ConversationState.ENGAGED,
    ConversationState.HOT_LEAD,
    ConversationState.HANDED_OFF,
    ConversationState.CLOSED_WON,
    ConversationState.CLOSED_LOST,
)


class SequenceStepIn(BaseModel):
    step_number: int
    delay_days: int
    channel: ChannelType
    template: str = ""
    template_sid: str = ""


class SequenceIn(BaseModel):
    name: str
    description: str = ""
    steps: list[SequenceStepIn] = []


@router.get("")
async def list_sequences(session: AsyncSession = Depends(get_db_session)):
    sequences = (await session.execute(select(FollowUpSequence))).scalars().all()
    return [{"id": str(s.id), "name": s.name, "isActive": s.is_active} for s in sequences]


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


@router.post("")
async def create_sequence(payload: SequenceIn, session: AsyncSession = Depends(get_db_session)):
    sequence = FollowUpSequence(name=payload.name, description=payload.description)
    session.add(sequence)
    await session.flush()

    for step in payload.steps:
        session.add(
            SequenceStep(
                sequence_id=sequence.id,
                step_number=step.step_number,
                delay_days=step.delay_days,
                channel=step.channel,
                template=step.template,
                template_sid=step.template_sid,
            )
        )
    await session.flush()
    return {"id": str(sequence.id)}


@router.get("/{sequence_id}")
async def get_sequence(sequence_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    sequence = await session.get(FollowUpSequence, sequence_id)
    if sequence is None:
        raise HTTPException(status_code=404, detail="Sequence not found")

    steps = (
        await session.execute(
            select(SequenceStep)
            .where(SequenceStep.sequence_id == sequence_id)
            .order_by(SequenceStep.step_number)
        )
    ).scalars().all()

    return {
        "id": str(sequence.id),
        "name": sequence.name,
        "description": sequence.description,
        "isActive": sequence.is_active,
        "steps": [
            {
                "stepNumber": st.step_number,
                "delayDays": st.delay_days,
                "channel": st.channel.value,
                "template": st.template,
                "templateSid": st.template_sid,
            }
            for st in steps
        ],
    }
