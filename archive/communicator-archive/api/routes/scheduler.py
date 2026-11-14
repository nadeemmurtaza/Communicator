"""
Phase 4 trigger points and scheduled background runners.
Supports running follow-ups, auto-approved AI drafts, WhatsApp campaigns,
and 24-hour inactivity session nudges.
"""

from __future__ import annotations

from datetime import UTC, datetime

from fastapi import APIRouter, Depends
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.api.routes.broadcasts import dispatch_broadcast
from communicator.harvester_client import HarvesterClient
from communicator.models.broadcast import Broadcast, BroadcastStatus
from communicator.services.draft_actions import dispatch_auto_approved_drafts
from communicator.services.followup_scheduler import run_followup_scheduler
from communicator.services.whatsapp_automations import run_inactivity_nudges

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


@router.post("/run-nudges")
async def run_nudges(session: AsyncSession = Depends(get_db_session)):
    """Runs 24-hour session window inactivity reminders."""
    nudged = await run_inactivity_nudges(session)
    return {"nudged": len(nudged), "conversationIds": [str(c) for c in nudged]}


@router.post("/process-campaigns")
async def process_campaigns(session: AsyncSession = Depends(get_db_session)):
    """Processes any campaigns scheduled in the past that have not yet dispatched."""
    now = datetime.now(UTC)
    stmt = (
        select(Broadcast)
        .where(Broadcast.status == BroadcastStatus.SCHEDULED)
        .where(Broadcast.scheduled_at.is_not(None))
        .where(Broadcast.scheduled_at <= now)
    )
    due = (await session.execute(stmt)).scalars().all()
    dispatched = []
    for camp in due:
        res = await dispatch_broadcast(camp.id, session)
        dispatched.append(res)
    return {"dispatched": len(dispatched), "campaigns": dispatched}


@router.post("/run-all")
async def run_all(
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
):
    """Executes all background schedulers in a single unified run."""
    followups = await run_followup_scheduler(session, harvester_client)
    drafts = await dispatch_auto_approved_drafts(session, harvester_client)
    nudges = await run_inactivity_nudges(session)

    # Due campaigns
    now = datetime.now(UTC)
    stmt = (
        select(Broadcast)
        .where(Broadcast.status == BroadcastStatus.SCHEDULED)
        .where(Broadcast.scheduled_at.is_not(None))
        .where(Broadcast.scheduled_at <= now)
    )
    due = (await session.execute(stmt)).scalars().all()
    campaigns = []
    for camp in due:
        res = await dispatch_broadcast(camp.id, session)
        campaigns.append(res)

    return {
        "followupsSent": len(followups),
        "draftsAutoSent": len(drafts),
        "inactivityNudges": len(nudges),
        "campaignsDispatched": len(campaigns),
    }
