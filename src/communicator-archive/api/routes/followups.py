from __future__ import annotations

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services.followup_scheduler import run_due_followups

router = APIRouter(prefix="/followups", tags=["Follow-ups"])


@router.post("/run")
async def run_followups(session: AsyncSession = Depends(get_db_session)):
    """Manually fire one scheduler tick -- the worker (communicator.worker) calls the same function on a poll loop."""
    drafts = await run_due_followups(session)
    return {"draftsCreated": len(drafts), "draftIds": [str(d.id) for d in drafts]}
