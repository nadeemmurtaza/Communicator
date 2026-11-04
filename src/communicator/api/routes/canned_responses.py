from __future__ import annotations

import re
import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.canned_response import CannedResponse
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.team_member import TeamMember

router = APIRouter(prefix="/canned-responses", tags=["Canned Responses (Saved Replies)"])


class CannedResponseIn(BaseModel):
    short_code: str
    content: str
    category: str = "general"


class CannedResponseUpdateIn(BaseModel):
    short_code: str | None = None
    content: str | None = None
    category: str | None = None


class CannedResponseExpandIn(BaseModel):
    short_code: str
    conversation_id: uuid.UUID | None = None
    custom_vars: dict[str, str] = {}


def _serialize(c: CannedResponse) -> dict[str, Any]:
    return {
        "id": str(c.id),
        "shortCode": c.short_code,
        "short_code": c.short_code,
        "content": c.content,
        "category": c.category,
        "createdAt": c.created_at.isoformat(),
        "created_at": c.created_at.isoformat(),
    }


def render_canned_variables(template: str, variables: dict[str, str]) -> str:
    """
    Replaces {{variable}} or {{ variable }} with resolved values.
    """
    rendered = template
    for k, v in variables.items():
        pattern = re.compile(rf"\{{\{{\s*{re.escape(k)}\s*\}}\}}", re.IGNORECASE)
        rendered = pattern.sub(v, rendered)
    return rendered


@router.get("")
async def list_canned_responses(
    q: str | None = Query(default=None),
    category: str | None = Query(default=None),
    session: AsyncSession = Depends(get_db_session),
):
    stmt = select(CannedResponse).order_by(CannedResponse.short_code)
    results = list((await session.execute(stmt)).scalars().all())

    # Filter by q (matches short_code or content)
    if q and q.strip():
        term = q.strip().lstrip("/").lower()
        results = [c for c in results if term in c.short_code.lower() or term in c.content.lower()]

    # Filter by category
    if category and category.strip() and category.lower() != "all":
        cat = category.strip().lower()
        results = [c for c in results if c.category.lower() == cat]

    return [_serialize(c) for c in results]


@router.post("")
async def create_canned_response(
    payload: CannedResponseIn, session: AsyncSession = Depends(get_db_session)
):
    code = payload.short_code.strip().lstrip("/")
    if not code:
        raise HTTPException(status_code=400, detail="Short code cannot be empty.")

    stmt = select(CannedResponse).where(CannedResponse.short_code == code)
    if (await session.execute(stmt)).scalar_one_or_none():
        raise HTTPException(status_code=400, detail=f"Short code /{code} already exists")

    item = CannedResponse(
        short_code=code,
        content=payload.content.strip(),
        category=payload.category.strip().lower() or "general",
    )
    session.add(item)
    await session.flush()
    return _serialize(item)


@router.put("/{id}")
async def update_canned_response(
    id: uuid.UUID,
    payload: CannedResponseUpdateIn,
    session: AsyncSession = Depends(get_db_session),
):
    item = await session.get(CannedResponse, id)
    if not item:
        raise HTTPException(status_code=404, detail="Canned response not found")

    if payload.short_code is not None:
        code = payload.short_code.strip().lstrip("/")
        if not code:
            raise HTTPException(status_code=400, detail="Short code cannot be empty.")
        if code != item.short_code:
            stmt = select(CannedResponse).where(CannedResponse.short_code == code)
            if (await session.execute(stmt)).scalar_one_or_none():
                raise HTTPException(status_code=400, detail=f"Short code /{code} already exists")
            item.short_code = code

    if payload.content is not None:
        item.content = payload.content.strip()

    if payload.category is not None:
        item.category = payload.category.strip().lower() or "general"

    await session.flush()
    return _serialize(item)


@router.delete("/{id}")
async def delete_canned_response(id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    item = await session.get(CannedResponse, id)
    if not item:
        raise HTTPException(status_code=404, detail="Canned response not found")
    await session.delete(item)
    await session.flush()
    return {"status": "deleted", "deleted": True, "id": str(id)}


@router.post("/expand")
async def expand_canned_response(
    payload: CannedResponseExpandIn, session: AsyncSession = Depends(get_db_session)
):
    """
    Renders dynamic placeholders in a canned response for a specific conversation.
    """
    code = payload.short_code.strip().lstrip("/")
    stmt = select(CannedResponse).where(CannedResponse.short_code == code)
    canned = (await session.execute(stmt)).scalar_one_or_none()
    if not canned:
        raise HTTPException(status_code=404, detail=f"Canned response /{code} not found")

    # Contextual variables dictionary
    variables = {
        "customer.name": "Customer",
        "customer_name": "Customer",
        "customer.first_name": "there",
        "first_name": "there",
        "agent.name": "Support Agent",
        "agent_name": "Support Agent",
        "channel": "Support",
        "ticket.id": "—",
        "ticket_id": "—",
        "conversation_id": "—",
    }
    variables.update(payload.custom_vars or {})

    if payload.conversation_id:
        conv = await session.get(Conversation, payload.conversation_id)
        if conv:
            variables["ticket.id"] = str(conv.id)[:8]
            variables["ticket_id"] = str(conv.id)[:8]
            variables["conversation_id"] = str(conv.id)

            if conv.assigned_to_id:
                agent = await session.get(TeamMember, conv.assigned_to_id)
                if agent:
                    variables["agent.name"] = agent.name
                    variables["agent_name"] = agent.name

            if conv.contact_id:
                contact = await session.get(Contact, conv.contact_id)
                if contact:
                    if contact.person_name:
                        variables["customer.name"] = contact.person_name
                        variables["customer_name"] = contact.person_name
                        variables["customer.first_name"] = contact.person_name.split()[0]
                        variables["first_name"] = contact.person_name.split()[0]
                    variables["channel"] = contact.channel.value.title()

    rendered = render_canned_variables(canned.content, variables)
    return {
        "shortCode": canned.short_code,
        "rawContent": canned.content,
        "renderedContent": rendered,
        "variables": variables,
    }
