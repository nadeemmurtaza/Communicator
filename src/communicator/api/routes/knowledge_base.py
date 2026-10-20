from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.knowledge_base import KnowledgeBaseEntry

router = APIRouter(prefix="/knowledge-base", tags=["Knowledge Base"])


class EntryIn(BaseModel):
    title: str
    content: str
    source_type: str = "note"


@router.get("")
async def list_entries(session: AsyncSession = Depends(get_db_session)):
    entries = (
        (
            await session.execute(
                select(KnowledgeBaseEntry).order_by(KnowledgeBaseEntry.created_at.desc())
            )
        )
        .scalars()
        .all()
    )
    return [
        {
            "id": str(e.id),
            "title": e.title,
            "content": e.content,
            "sourceType": e.source_type,
            "createdAt": e.created_at.isoformat(),
        }
        for e in entries
    ]


@router.post("")
async def create_entry(payload: EntryIn, session: AsyncSession = Depends(get_db_session)):
    entry = KnowledgeBaseEntry(
        title=payload.title, content=payload.content, source_type=payload.source_type
    )
    session.add(entry)
    await session.flush()
    return {"id": str(entry.id)}


@router.delete("/{entry_id}")
async def delete_entry(entry_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    entry = await session.get(KnowledgeBaseEntry, entry_id)
    if entry is None:
        raise HTTPException(status_code=404, detail="Entry not found")
    await session.delete(entry)
    await session.flush()
    return {"status": "deleted"}
