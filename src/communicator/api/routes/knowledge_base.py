from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException, UploadFile
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.knowledge_base import KnowledgeBaseEntry
from communicator.services.knowledge_gaps import (
    approve_knowledge_gap,
    detect_knowledge_gaps,
    dismiss_knowledge_gap,
    list_knowledge_gaps,
)
from communicator.services.knowledge_hub_service import (
    apply_recommendation,
    dismiss_recommendation,
    ensure_default_hub_content,
    get_knowledge_hub_overview,
    list_recommendations,
    serialize_hub_entry,
)
from communicator.services.knowledge_ingest import IngestionError, ingest_document_bytes, ingest_url

router = APIRouter(prefix="/knowledge-base", tags=["Knowledge Base"])


def _serialize_entry(e: KnowledgeBaseEntry) -> dict:
    return serialize_hub_entry(e)


class EntryIn(BaseModel):
    title: str
    content: str
    topic: str = "General"
    source_type: str = "note"
    target_touchpoints: list[str] = ["agent", "copilot", "help_center"]


class IngestUrlIn(BaseModel):
    url: str
    topic: str = "General"


class ApproveGapIn(BaseModel):
    custom_answer: str | None = None


class ApplyRecommendationIn(BaseModel):
    custom_content: str | None = None


@router.get("/hub")
async def get_hub_overview(
    q: str | None = None,
    topic: str | None = None,
    touchpoint: str | None = None,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Centralized Knowledge Hub endpoint: Returns sources grouped by topic,
    cross-touchpoint deployment tracking (Agent, Copilot, Help Center), and resolution metrics.
    """
    return await get_knowledge_hub_overview(
        session=session,
        search_query=q,
        topic_filter=topic,
        touchpoint_filter=touchpoint,
    )


@router.get("/recommendations")
async def get_content_recommendations(
    status: str = "open",
    gap_type: str | None = None,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Returns AI recommendations highlighting content, customer data, and action gaps
    to improve Agent Fin's automated resolution rate.
    """
    return await list_recommendations(session=session, status=status, gap_type=gap_type)


@router.post("/recommendations/{rec_id}/apply")
async def apply_content_recommendation(
    rec_id: uuid.UUID,
    payload: ApplyRecommendationIn | None = None,
    session: AsyncSession = Depends(get_db_session),
):
    """
    1-Click Action: Converts an AI gap recommendation directly into a published
    knowledge base article to improve the resolution rate.
    """
    custom = payload.custom_content if payload else None
    try:
        return await apply_recommendation(
            session=session, recommendation_id=rec_id, custom_content=custom
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/recommendations/{rec_id}/dismiss")
async def dismiss_content_recommendation(
    rec_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    """Dismisses an AI gap recommendation."""
    try:
        return await dismiss_recommendation(session=session, recommendation_id=rec_id)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.get("")
async def list_entries(session: AsyncSession = Depends(get_db_session)):
    await ensure_default_hub_content(session)
    entries = (
        (
            await session.execute(
                select(KnowledgeBaseEntry).order_by(KnowledgeBaseEntry.created_at.desc())
            )
        )
        .scalars()
        .all()
    )
    return [_serialize_entry(e) for e in entries]


@router.post("")
async def create_entry(payload: EntryIn, session: AsyncSession = Depends(get_db_session)):
    entry = KnowledgeBaseEntry(
        title=payload.title,
        content=payload.content,
        topic=payload.topic or "General",
        source_type=payload.source_type,
        target_touchpoints=payload.target_touchpoints or ["agent", "copilot", "help_center"],
    )
    session.add(entry)
    await session.flush()
    return _serialize_entry(entry)


@router.post("/ingest-url")
async def ingest_web_url(payload: IngestUrlIn, session: AsyncSession = Depends(get_db_session)):
    """
    Crawls and scrapes an external help center or documentation URL (e.g., helpcenter.com)
    and stores it in the Knowledge Base ("Learns").
    """
    try:
        entry = await ingest_url(session, payload.url)
        return _serialize_entry(entry)
    except IngestionError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/upload")
async def upload_document(file: UploadFile, session: AsyncSession = Depends(get_db_session)):
    """
    Extracts text from uploaded documents (PDF, Markdown, TXT) and adds them
    as grounded knowledge entries ("Learns").
    """
    content_bytes = await file.read()
    try:
        entry = ingest_document_bytes(
            session, filename=file.filename or "uploaded-doc.pdf", content_bytes=content_bytes
        )
        await session.flush()
        return _serialize_entry(entry)
    except IngestionError as e:
        raise HTTPException(status_code=400, detail=str(e))


# --- Knowledge Gap Detection ("Fills gaps" / ✨ Suggested FAQ) ---


@router.get("/gaps")
async def get_gaps(status: str = "suggested", session: AsyncSession = Depends(get_db_session)):
    """Lists auto-detected knowledge gaps / suggested FAQs."""
    gaps = await list_knowledge_gaps(session, status=status)
    return [
        {
            "id": str(g.id),
            "question": g.question,
            "suggestedAnswer": g.suggested_answer,
            "sourceQuery": g.source_query,
            "frequency": g.frequency,
            "status": g.status,
            "createdAt": g.created_at.isoformat(),
        }
        for g in gaps
    ]


@router.post("/gaps/scan")
async def scan_gaps(session: AsyncSession = Depends(get_db_session)):
    """Triggers gap detection across recent customer questions and low-confidence replies."""
    detected = await detect_knowledge_gaps(session)
    return {
        "status": "success",
        "detectedCount": len(detected),
        "gaps": [
            {
                "id": str(g.id),
                "question": g.question,
                "suggestedAnswer": g.suggested_answer,
                "frequency": g.frequency,
            }
            for g in detected
        ],
    }


@router.post("/gaps/{gap_id}/approve")
async def approve_gap(
    gap_id: uuid.UUID,
    payload: ApproveGapIn | None = None,
    session: AsyncSession = Depends(get_db_session),
):
    """Converts a suggested FAQ into an active Knowledge Base article with 1 click."""
    custom_ans = payload.custom_answer if payload else None
    try:
        entry = await approve_knowledge_gap(session, gap_id, custom_answer=custom_ans)
        return _serialize_entry(entry)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/gaps/{gap_id}/dismiss")
async def dismiss_gap(gap_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    """Dismisses a suggested knowledge gap."""
    await dismiss_knowledge_gap(session, gap_id)
    return {"status": "dismissed"}


@router.delete("/{entry_id}")
async def delete_entry(entry_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    entry = await session.get(KnowledgeBaseEntry, entry_id)
    if entry is None:
        raise HTTPException(status_code=404, detail="Entry not found")
    await session.delete(entry)
    await session.flush()
    return {"status": "deleted"}
