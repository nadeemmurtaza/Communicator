from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.api.dependencies import get_db_session
from communicator.models.team import Team
from communicator.models.team_member import TeamMember

router = APIRouter(prefix="/teams", tags=["Teams"])


class TeamIn(BaseModel):
    name: str
    description: str = ""
    auto_assign: bool = True


class TeamMemberAddIn(BaseModel):
    member_id: uuid.UUID


def _serialize_team(t: Team) -> dict[str, Any]:
    return {
        "id": str(t.id),
        "name": t.name,
        "description": t.description,
        "autoAssign": t.auto_assign,
        "membersCount": len(t.members) if t.members else 0,
        "members": [
            {"id": str(m.id), "name": m.name, "email": m.email} for m in (t.members or [])
        ],
        "createdAt": t.created_at.isoformat(),
    }


@router.get("")
async def list_teams(session: AsyncSession = Depends(get_db_session)):
    stmt = select(Team).options(selectinload(Team.members)).order_by(Team.name)
    teams = (await session.execute(stmt)).scalars().all()

    # Pre-seed starter teams if empty
    if not teams:
        defaults = [
            Team(name="Sales & Growth", description="Handles new customer inquiries and deals", auto_assign=True),
            Team(name="Customer Support", description="Resolves technical questions and account issues", auto_assign=True),
            Team(name="VIP Accounts", description="Priority queue for enterprise customers", auto_assign=False),
        ]
        session.add_all(defaults)
        await session.flush()
        # Fetch seeded teams with selectinload
        teams = (await session.execute(stmt)).scalars().all()

    return [_serialize_team(t) for t in teams]


@router.post("")
async def create_team(payload: TeamIn, session: AsyncSession = Depends(get_db_session)):
    name = payload.name.strip()
    stmt = select(Team).where(Team.name == name)
    if (await session.execute(stmt)).scalar_one_or_none():
        raise HTTPException(status_code=400, detail=f"Team '{name}' already exists")

    team = Team(
        name=name,
        description=payload.description.strip(),
        auto_assign=payload.auto_assign,
    )
    session.add(team)
    await session.flush()
    await session.refresh(team, attribute_names=["members"])
    return _serialize_team(team)


@router.delete("/{id}")
async def delete_team(id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    team = await session.get(Team, id)
    if not team:
        raise HTTPException(status_code=404, detail="Team not found")
    await session.delete(team)
    await session.flush()
    return {"status": "deleted"}


@router.post("/{id}/members")
async def add_member_to_team(
    id: uuid.UUID, payload: TeamMemberAddIn, session: AsyncSession = Depends(get_db_session)
):
    stmt = select(Team).options(selectinload(Team.members)).where(Team.id == id)
    team = (await session.execute(stmt)).scalar_one_or_none()
    if not team:
        raise HTTPException(status_code=404, detail="Team not found")

    member = await session.get(TeamMember, payload.member_id)
    if not member:
        raise HTTPException(status_code=404, detail="Team member not found")

    if member not in team.members:
        team.members.append(member)
        await session.flush()

    return _serialize_team(team)


@router.delete("/{id}/members/{member_id}")
async def remove_member_from_team(
    id: uuid.UUID, member_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    stmt = select(Team).options(selectinload(Team.members)).where(Team.id == id)
    team = (await session.execute(stmt)).scalar_one_or_none()
    if not team:
        raise HTTPException(status_code=404, detail="Team not found")

    team.members = [m for m in team.members if m.id != member_id]
    await session.flush()
    return _serialize_team(team)
