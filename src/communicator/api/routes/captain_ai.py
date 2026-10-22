from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import captain_ai

router = APIRouter(tags=["Captain AI Copilot & Monitors"])


class SuggestedReplyIn(BaseModel):
    instruction: str | None = None


class RewriteIn(BaseModel):
    text: str
    tone: str = "professional"  # professional | friendly | shorten | fix_spelling | expand


class TriageIn(BaseModel):
    apply: bool = False


class MonitorCreateIn(BaseModel):
    name: str
    topic: str
    keywords: list[str]
    threshold: int = 5


@router.post("/conversations/{conversation_id}/ai-copilot/suggested-reply")
async def get_suggested_reply(
    conversation_id: uuid.UUID,
    payload: SuggestedReplyIn | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Captain AI Copilot: Generates a contextual suggested reply grounded in the
    knowledge base, service catalog, and past customer messages.
    """
    try:
        instruction = payload.instruction if payload else None
        return await captain_ai.generate_copilot_reply(session, conversation_id, instruction)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/conversations/{conversation_id}/ai-copilot/summarize")
async def summarize_thread(
    conversation_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Captain AI Copilot: Generates an executive bulleted summary of the full conversation thread.
    """
    try:
        return await captain_ai.summarize_conversation(session, conversation_id)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/ai-copilot/rewrite")
async def rewrite_draft(
    payload: RewriteIn,
) -> dict[str, Any]:
    """
    Captain AI Copilot Tone Adjuster / Rewriter:
    Rephrases a draft in professional, friendly, shorten, fix_spelling, or expand tones.
    """
    if not payload.text.strip():
        raise HTTPException(status_code=400, detail="Text cannot be empty")
    return captain_ai.rewrite_text(payload.text, payload.tone)


@router.post("/conversations/{conversation_id}/ai-copilot/triage")
async def triage_conversation(
    conversation_id: uuid.UUID,
    payload: TriageIn | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Captain AI Triage:
    Infers intent, customer urgency/priority, suggested tags, and team routing.
    Optionally applies the classification to the conversation if apply=True.
    """
    try:
        apply_changes = payload.apply if payload else False
        return await captain_ai.triage_conversation(
            session, conversation_id, apply_changes=apply_changes
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.get("/captain/monitors")
async def list_captain_monitors(
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Captain Monitors: Returns real-time conversation theme intelligence,
    spike alerts, and sentiment telemetry.
    """
    return await captain_ai.get_captain_monitors(session)


@router.post("/captain/monitors")
async def create_captain_monitor(
    payload: MonitorCreateIn,
) -> dict[str, Any]:
    """
    Captain Monitors: Adds a new custom topic monitor rule.
    """
    if not payload.name.strip():
        raise HTTPException(status_code=400, detail="Monitor name cannot be empty")
    if not payload.keywords:
        raise HTTPException(status_code=400, detail="At least one keyword is required")
    return captain_ai.add_custom_monitor(
        payload.name, payload.topic, payload.keywords, payload.threshold
    )
