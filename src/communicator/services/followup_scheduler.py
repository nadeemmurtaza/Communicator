"""
Phase 4 follow-up scheduler -- see docs/ARCHITECTURE.md section 8.

Advances Conversations through their FollowUpSequence on a cadence. Not
wired to a task queue yet (no Celery/APScheduler dependency in this repo);
``run_followup_scheduler`` is a plain async function meant to be invoked by
an external cron hitting POST /api/v1/scheduler/run-followups, or a queue
worker later, without callers needing to change.
"""

from __future__ import annotations

import random
import uuid
from datetime import UTC, datetime, timedelta

import httpx
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.audit import AuditLog
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection
from communicator.models.sequence import FollowUpSequence, SequenceStep
from communicator.services.messaging import send_outbound_message
from communicator.services.translation import translate_text

_ELIGIBLE_STATES = (ConversationState.NEW, ConversationState.AWAITING_REPLY)


def within_working_hours(now: datetime | None = None) -> bool:
    """
    Placeholder business-hours check (UTC, Mon-Fri, configured hour range).
    Per-lead timezone isn't modeled yet -- see docs/ARCHITECTURE.md's
    `within_working_hours` reference to Harvester's existing concept.
    """
    now = now or datetime.now(UTC)
    if now.weekday() >= 5:
        return False
    return settings.follow_up_quiet_hours_start <= now.hour < settings.follow_up_quiet_hours_end


def enroll_conversation(
    conversation: Conversation, sequence: FollowUpSequence, now: datetime | None = None
) -> None:
    """
    Attach a Conversation to a FollowUpSequence, randomly assign it to one
    of the sequence's variants (equal weight across whatever distinct
    `variant` labels exist -- "control" only, if the sequence isn't running
    an A/B test), and schedule its first step.
    """
    now = now or datetime.now(UTC)
    variants = sorted({step.variant for step in sequence.steps}) or ["control"]
    variant = random.choice(variants)

    conversation.sequence_id = sequence.id
    conversation.sequence_variant = variant
    conversation.sequence_step = 0

    first_step = min(
        (s for s in sequence.steps if s.variant == variant),
        key=lambda s: s.step_number,
        default=None,
    )
    conversation.next_followup_at = (
        now + timedelta(days=first_step.delay_days) if first_step else None
    )


async def _weekly_send_count(session: AsyncSession, contact_id: uuid.UUID, now: datetime) -> int:
    stmt = (
        select(func.count())
        .select_from(Message)
        .join(Conversation, Message.conversation_id == Conversation.id)
        .where(
            Conversation.contact_id == contact_id,
            Message.direction == MessageDirection.OUTBOUND,
            Message.sent_at >= now - timedelta(days=7),
        )
    )
    return (await session.execute(stmt)).scalar_one()


async def _last_inbound_at(session: AsyncSession, conversation_id: uuid.UUID) -> datetime | None:
    stmt = (
        select(Message.sent_at)
        .where(
            Message.conversation_id == conversation_id,
            Message.direction == MessageDirection.INBOUND,
        )
        .order_by(Message.sent_at.desc())
        .limit(1)
    )
    return (await session.execute(stmt)).scalar_one_or_none()


async def _due_conversations(session: AsyncSession, now: datetime) -> list[Conversation]:
    stmt = select(Conversation).where(
        Conversation.sequence_id.is_not(None),
        Conversation.next_followup_at.is_not(None),
        Conversation.next_followup_at <= now,
        Conversation.state.in_(_ELIGIBLE_STATES),
    )
    return list((await session.execute(stmt)).scalars().all())


async def _advance_conversation(
    session: AsyncSession,
    conversation: Conversation,
    harvester_client: HarvesterClient,
    now: datetime,
) -> Message | None:
    if not within_working_hours(now):
        conversation.next_followup_at = now.replace(
            hour=settings.follow_up_quiet_hours_start, minute=0, second=0, microsecond=0
        ) + timedelta(days=1)
        return None

    contact = await session.get(Contact, conversation.contact_id)
    if await _weekly_send_count(session, contact.id, now) >= settings.follow_up_weekly_message_cap:
        conversation.next_followup_at = now + timedelta(days=1)
        return None

    next_step_number = conversation.sequence_step + 1
    stmt = select(SequenceStep).where(
        SequenceStep.sequence_id == conversation.sequence_id,
        SequenceStep.step_number == next_step_number,
        SequenceStep.variant == conversation.sequence_variant,
    )
    step = (await session.execute(stmt)).scalar_one_or_none()
    if step is None:
        conversation.next_followup_at = None
        return None

    if step.channel == ChannelType.WHATSAPP:
        last_inbound = await _last_inbound_at(session, conversation.id)
        window = timedelta(hours=settings.whatsapp_session_window_hours)
        if last_inbound is None or now - last_inbound > window:
            # Outside Meta's 24h session window; freeform sends will be
            # rejected and a template-message provider isn't built yet
            # (see channels/whatsapp.py). Defer and retry rather than
            # attempting a doomed send.
            session.add(
                AuditLog(
                    entity_type="conversation",
                    entity_id=conversation.id,
                    action="followup_deferred_whatsapp_window",
                    actor="system",
                )
            )
            conversation.next_followup_at = now + timedelta(days=1)
            return None

    body = await translate_text(step.template, contact.language)
    provider = await get_provider(step.channel, session)
    message = await send_outbound_message(session, contact, provider, body)

    conversation.sequence_step = next_step_number
    next_stmt = select(SequenceStep).where(
        SequenceStep.sequence_id == conversation.sequence_id,
        SequenceStep.step_number == next_step_number + 1,
        SequenceStep.variant == conversation.sequence_variant,
    )
    following = (await session.execute(next_stmt)).scalar_one_or_none()
    conversation.next_followup_at = (
        now + timedelta(days=following.delay_days) if following else None
    )

    session.add(
        AuditLog(
            entity_type="conversation",
            entity_id=conversation.id,
            action="followup_sent",
            actor="system",
            details={"step_number": next_step_number},
        )
    )

    lead = await session.get(Lead, contact.lead_id)
    if lead is not None:
        try:
            await harvester_client.push_outreach_outcome(
                institution_id=lead.harvester_institution_id
            )
        except httpx.HTTPError:
            pass

    return message


async def run_followup_scheduler(
    session: AsyncSession, harvester_client: HarvesterClient
) -> list[Message]:
    now = datetime.now(UTC)
    sent: list[Message] = []
    for conversation in await _due_conversations(session, now):
        message = await _advance_conversation(session, conversation, harvester_client, now)
        if message is not None:
            sent.append(message)
    await session.flush()
    return sent
