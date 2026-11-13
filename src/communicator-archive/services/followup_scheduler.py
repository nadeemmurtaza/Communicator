"""
Phase 4 follow-up scheduler -- see docs/ARCHITECTURE.md section 8.

Advances Conversations enrolled in a FollowUpSequence once their
next_followup_at has passed.

Generated copy goes through the same Draft/approval gate as reactive
replies (communicator.services.reply_engine): this module only ever
creates PENDING_REVIEW drafts, the existing /drafts/{id}/approve endpoint
is what actually sends.
"""

from __future__ import annotations

import logging
import random
import uuid
from datetime import UTC, datetime, timedelta

import httpx
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.channels.whatsapp import MetaWhatsAppProvider
from communicator.compliance.optout import is_sendable
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.audit import AuditLog
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.draft import Draft, DraftStatus
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.models.sequence import FollowUpSequence, SequenceStep
from communicator.services.messaging import send_outbound_message, whatsapp_session_open
from communicator.services.translation import translate_text

logger = logging.getLogger(__name__)

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


async def _ordered_steps(session: AsyncSession, sequence_id: uuid.UUID) -> list[SequenceStep]:
    stmt = (
        select(SequenceStep)
        .where(SequenceStep.sequence_id == sequence_id)
        .order_by(SequenceStep.step_number)
    )
    return list((await session.execute(stmt)).scalars().all())


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
            provider = await get_provider(step.channel, session)
            if isinstance(provider, MetaWhatsAppProvider) and step.template.startswith("template:"):
                template_name = step.template.removeprefix("template:").strip()
                try:
                    res = await provider.send_template(
                        to=contact.value,
                        template_name=template_name,
                        language=contact.language or "en_US",
                        body_params=[contact.person_name or "there"],
                    )
                    message = Message(
                        conversation_id=conversation.id,
                        direction=MessageDirection.OUTBOUND,
                        channel=step.channel,
                        body=f"[Template: {template_name}]",
                        provider="meta",
                        provider_message_id=res.provider_message_id,
                        status=MessageStatus.SENT,
                        intent="followup_template",
                    )
                    session.add(message)
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
                            action="followup_template_sent",
                            actor="system",
                            details={"step_number": next_step_number, "template": template_name},
                        )
                    )
                    return message
                except Exception:  # noqa: BLE001, S110
                    pass

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


def _render(template: str, lead: Lead, contact: Contact) -> str:
    return template.format(
        lead_name=lead.name,
        city=lead.city,
        contact_name=contact.person_name or lead.name,
    )


async def enroll_in_sequence(
    session: AsyncSession, conversation: Conversation, sequence: FollowUpSequence
) -> None:
    """
    Call right after a first-touch send. Step 1 (index 0) is the message
    just sent, so this schedules step 2 (index 1), if the sequence has one.
    """
    steps = await _ordered_steps(session, sequence.id)
    conversation.sequence_id = sequence.id
    conversation.sequence_step = 1
    conversation.next_followup_at = (
        datetime.now(UTC) + timedelta(days=steps[1].delay_days) if len(steps) > 1 else None
    )


async def run_due_followups(session: AsyncSession, now: datetime | None = None) -> list[Draft]:
    """
    Scans conversations whose next_followup_at has passed and drafts the
    next SequenceStep for each. Stops (clears next_followup_at, drafts
    nothing) the moment a conversation is no longer eligible -- opted out,
    replied, or sequence exhausted -- checked here at run time rather than
    only when it was first scheduled, per docs/ARCHITECTURE.md section 8.
    """
    now = now or datetime.now(UTC)

    stmt = select(Conversation).where(
        Conversation.state == ConversationState.AWAITING_REPLY,
        Conversation.sequence_id.is_not(None),
        Conversation.next_followup_at <= now,
    )
    due = (await session.execute(stmt)).scalars().all()

    drafts: list[Draft] = []
    for conversation in due:
        if conversation.state != ConversationState.AWAITING_REPLY:
            continue

        contact = await session.get(Contact, conversation.contact_id)
        if not await is_sendable(session, contact.id):
            conversation.next_followup_at = None
            continue

        steps = await _ordered_steps(session, conversation.sequence_id)
        step_index = conversation.sequence_step
        if step_index >= len(steps):
            conversation.next_followup_at = None
            continue

        step = steps[step_index]

        if (
            step.channel == ChannelType.WHATSAPP
            and not step.template_sid
            and not await whatsapp_session_open(session, conversation.id)
        ):
            logger.warning(
                "conversation %s step %d is a WhatsApp send outside the 24h session window "
                "with no template_sid configured on the sequence step; skipping until one is "
                "attached or the contact replies",
                conversation.id,
                step.step_number,
            )
            continue

        try:
            lead = await session.get(Lead, contact.lead_id)
            body = _render(step.template, lead, contact)
        except Exception:
            logger.exception(
                "follow-up render failed for conversation %s step %d; will retry next tick",
                conversation.id,
                step.step_number,
            )
            continue

        draft = Draft(
            conversation_id=conversation.id,
            body=body,
            confidence=0.0,
            citations=[],
            status=DraftStatus.PENDING_REVIEW,
            content_sid=step.template_sid,
        )
        session.add(draft)
        drafts.append(draft)

        conversation.sequence_step = step_index + 1
        next_step = steps[step_index + 1] if step_index + 1 < len(steps) else None
        conversation.next_followup_at = (
            now + timedelta(days=next_step.delay_days) if next_step else None
        )

        session.add(
            AuditLog(
                entity_type="conversation",
                entity_id=conversation.id,
                action="followup_step_drafted",
                actor="scheduler",
                details={
                    "step_number": step.step_number,
                    "sequence_id": str(conversation.sequence_id),
                },
            )
        )

    await session.flush()
    return drafts
