from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.conversation import Conversation
from communicator.models.team_member import TeamMember

router = APIRouter(prefix="/team", tags=["Team"])


class TeamMemberIn(BaseModel):
    name: str
    email: str = ""


@router.get("")
async def list_team(session: AsyncSession = Depends(get_db_session)):
    members = (await session.execute(select(TeamMember))).scalars().all()
    return [{"id": str(m.id), "name": m.name, "email": m.email} for m in members]


@router.post("")
async def create_team_member(
    payload: TeamMemberIn, session: AsyncSession = Depends(get_db_session)
):
    member = TeamMember(name=payload.name, email=payload.email)
    session.add(member)
    await session.flush()
    return {"id": str(member.id)}


@router.delete("/{member_id}")
async def delete_team_member(member_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    member = await session.get(TeamMember, member_id)
    if member is None:
        raise HTTPException(status_code=404, detail="Team member not found")
    await session.delete(member)
    await session.flush()
    return {"status": "deleted"}


@router.post("/conversations/{conversation_id}/assign")
async def assign_conversation(
    conversation_id: uuid.UUID,
    member_id: uuid.UUID | None = None,
    session: AsyncSession = Depends(get_db_session),
):
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.assigned_to_id = member_id
    await session.flush()
    return {"id": str(conversation.id), "assignedToId": str(member_id) if member_id else None}


@router.post("/conversations/{conversation_id}/takeover")
async def set_human_takeover(
    conversation_id: uuid.UUID,
    enabled: bool = True,
    session: AsyncSession = Depends(get_db_session),
):
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation not found")
    conversation.human_takeover = enabled
    await session.flush()
    return {"id": str(conversation.id), "humanTakeover": conversation.human_takeover}
