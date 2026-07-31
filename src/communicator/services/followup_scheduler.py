"""
Advances Conversations enrolled in a FollowUpSequence once their
next_followup_at has passed -- see docs/ARCHITECTURE.md section 8.

Generated copy goes through the same Draft/approval gate as reactive
replies (communicator.services.reply_engine): this module only ever
creates PENDING_REVIEW drafts, the existing /drafts/{id}/approve endpoint
is what actually sends.
"""

from __future__ import annotations

import logging
import uuid
from datetime import UTC, datetime, timedelta

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.compliance.optout import is_sendable
from communicator.models.audit import AuditLog
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.draft import Draft, DraftStatus
from communicator.models.lead import Lead
from communicator.models.sequence import FollowUpSequence, SequenceStep
from communicator.services.messaging import whatsapp_session_open

logger = logging.getLogger(__name__)


async def _ordered_steps(session: AsyncSession, sequence_id: uuid.UUID) -> list[SequenceStep]:
    stmt = (
        select(SequenceStep)
        .where(SequenceStep.sequence_id == sequence_id)
        .order_by(SequenceStep.step_number)
    )
    return list((await session.execute(stmt)).scalars().all())


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
