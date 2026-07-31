"""
The compliance gate: this module must run before any reply-engine logic
touches an inbound message, and before any send touches a Contact. See
docs/ARCHITECTURE.md section 7, step 1.
"""
from __future__ import annotations

import re
import uuid

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.consent import ConsentRecord, ConsentStatus

# Word-boundary match against the *whole* message, not a substring check, so
# "please help me stop paying too much" doesn't trip it. Channel providers
# (Twilio, etc.) apply their own backstop handling of these keywords too --
# this is our own durable record of consent state, independent of the
# provider.
_OPT_OUT_PATTERN = re.compile(r"^\s*(stop|unsubscribe|opt\s*out|cancel|remove\s*me)\s*$", re.IGNORECASE)
_OPT_IN_PATTERN = re.compile(r"^\s*(start|subscribe|opt\s*in|yes)\s*$", re.IGNORECASE)


def is_opt_out_message(body: str) -> bool:
    return bool(_OPT_OUT_PATTERN.match(body.strip()))


def is_opt_in_message(body: str) -> bool:
    return bool(_OPT_IN_PATTERN.match(body.strip()))


async def get_consent_status(session: AsyncSession, contact_id: uuid.UUID) -> ConsentStatus:
    stmt = (
        select(ConsentRecord)
        .where(ConsentRecord.contact_id == contact_id)
        .order_by(ConsentRecord.recorded_at.desc())
        .limit(1)
    )
    record = (await session.execute(stmt)).scalar_one_or_none()
    return record.status if record else ConsentStatus.UNKNOWN


async def record_consent(
    session: AsyncSession,
    contact_id: uuid.UUID,
    status: ConsentStatus,
    source: str,
    note: str = "",
) -> ConsentRecord:
    record = ConsentRecord(contact_id=contact_id, status=status, source=source, note=note)
    session.add(record)
    return record


async def is_sendable(session: AsyncSession, contact_id: uuid.UUID) -> bool:
    """
    True only if the contact's latest consent record is OPTED_IN. UNKNOWN is
    deliberately *not* sendable -- every channel adapter's first send to a
    new Contact must first write an explicit OPTED_IN record (e.g. sourced
    from "harvester_sync" for a lead we're already cleared to contact)
    before anything goes out.
    """
    status = await get_consent_status(session, contact_id)
    return status == ConsentStatus.OPTED_IN


async def process_inbound_consent_signal(
    session: AsyncSession, contact_id: uuid.UUID, body: str
) -> ConsentStatus | None:
    """
    Call this first on every inbound message, before any AI/reply logic.
    Returns the new status if the message changed consent, else None.
    """
    if is_opt_out_message(body):
        await record_consent(session, contact_id, ConsentStatus.OPTED_OUT, source="inbound_stop_keyword")
        return ConsentStatus.OPTED_OUT
    if is_opt_in_message(body):
        await record_consent(session, contact_id, ConsentStatus.OPTED_IN, source="inbound_start_keyword")
        return ConsentStatus.OPTED_IN
    return None
