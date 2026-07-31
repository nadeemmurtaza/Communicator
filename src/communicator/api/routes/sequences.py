from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.contact import ChannelType
from communicator.models.sequence import FollowUpSequence, SequenceStep

router = APIRouter(prefix="/sequences", tags=["FollowUpSequences"])


class SequenceStepIn(BaseModel):
    step_number: int
    delay_days: int
    channel: ChannelType
    template: str = ""


class SequenceIn(BaseModel):
    name: str
    description: str = ""
    steps: list[SequenceStepIn] = []


@router.get("")
async def list_sequences(session: AsyncSession = Depends(get_db_session)):
    sequences = (await session.execute(select(FollowUpSequence))).scalars().all()
    return [{"id": str(s.id), "name": s.name, "isActive": s.is_active} for s in sequences]


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
            }
            for st in steps
        ],
    }
