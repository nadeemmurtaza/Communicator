from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.harvester_client import HarvesterClient
from communicator.models.draft import Draft, DraftStatus
from communicator.services.draft_actions import approve_and_send, reject_draft
from communicator.services.reply_engine import generate_reply

router = APIRouter(prefix="/drafts", tags=["Drafts"])


@router.get("")
async def list_drafts(status: str = "pending_review", session: AsyncSession = Depends(get_db_session)):
    stmt = select(Draft).where(Draft.status == DraftStatus(status))
    drafts = (await session.execute(stmt)).scalars().all()
    return [
        {
            "id": str(d.id),
            "conversationId": str(d.conversation_id),
            "body": d.body,
            "confidence": d.confidence,
            "citations": d.citations,
            "contentSid": d.content_sid,
            "status": d.status.value,
            "createdAt": d.created_at.isoformat(),
        }
        for d in drafts
    ]


@router.post("/{draft_id}/approve")
async def approve_draft(
    draft_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
):
    draft = await session.get(Draft, draft_id)
    if draft is None:
        raise HTTPException(status_code=404, detail="Draft not found")
    draft = await approve_and_send(session, draft, harvester_client)
    return {"id": str(draft.id), "status": draft.status.value}


@router.post("/{draft_id}/reject")
async def reject_draft_endpoint(
    draft_id: uuid.UUID, feedback: str = "", session: AsyncSession = Depends(get_db_session)
):
    draft = await session.get(Draft, draft_id)
    if draft is None:
        raise HTTPException(status_code=404, detail="Draft not found")
    draft = await reject_draft(session, draft, feedback)
    return {"id": str(draft.id), "status": draft.status.value}


@router.post("/generate/{conversation_id}")
async def generate_draft(
    conversation_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
):
    draft = await generate_reply(session, conversation_id, harvester_client)
    if draft is None:
        raise HTTPException(status_code=400, detail="Conversation not eligible for a reply")
    return {"id": str(draft.id), "status": draft.status.value, "body": draft.body}
