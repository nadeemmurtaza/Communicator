"""
Phase 4 trigger points. No Celery/APScheduler worker is wired up yet -- these
are meant to be hit by an external cron (or a scheduler added later) rather
than run in-process. See docs/ARCHITECTURE.md section 8/9.
"""

from __future__ import annotations

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.harvester_client import HarvesterClient
from communicator.services.draft_actions import dispatch_auto_approved_drafts
from communicator.services.followup_scheduler import run_followup_scheduler

router = APIRouter(prefix="/scheduler", tags=["Scheduler"])


@router.post("/run-followups")
async def run_followups(
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
):
    sent = await run_followup_scheduler(session, harvester_client)
    return {"sent": len(sent)}


@router.post("/dispatch-auto-approved")
async def dispatch_auto_approved(
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
):
    drafts = await dispatch_auto_approved_drafts(session, harvester_client)
    return {"sent": len(drafts)}
