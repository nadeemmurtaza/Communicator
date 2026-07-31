from __future__ import annotations

import uuid

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.ai.claude_client import call_claude, parse_json_reply
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.audit import AuditLog
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.draft import Draft, DraftStatus
from communicator.models.lead import Lead
from communicator.models.message import Message
from communicator.services.draft_actions import approve_and_send
from communicator.services.service_catalog import get_matching_services, render_service_context

SYSTEM_PROMPT = """You are a sales assistant for NEWAX Technologies, replying to a lead on behalf of the sales team.
Only make claims found in the SERVICES context below. Never invent pricing or contractual terms.
If the lead asks something outside the SERVICES context, or shows a strong buying signal, or is upset/complaining,
set intent to "escalate" and keep confidence low.

Respond with strict JSON only, no markdown fences:
{"intent": "faq_question|objection|buying_signal|scheduling_request|complaint|unrelated|escalate",
 "confidence": 0.0-1.0,
 "reply": "the message body to send back to the lead",
 "citations": ["service names referenced"]}
"""


async def _build_context(session: AsyncSession, conversation: Conversation) -> tuple[str, str]:
    contact = await session.get(Contact, conversation.contact_id)
    lead = await session.get(Lead, contact.lead_id)

    history_stmt = (
        select(Message).where(Message.conversation_id == conversation.id).order_by(Message.sent_at)
    )
    messages = (await session.execute(history_stmt)).scalars().all()
    history = "\n".join(f"{m.direction.value.upper()}: {m.body}" for m in messages)

    services = await get_matching_services(session, lead.institution_type)
    service_context = render_service_context(services)

    user_prompt = (
        f"LEAD: {lead.name} ({lead.institution_type}, {lead.city})\n"
        f"QUALIFICATION: {lead.qualification_rationale}\n\n"
        f"SERVICES:\n{service_context}\n\n"
        f"CONVERSATION HISTORY:\n{history}\n\n"
        "Draft the next reply to send to this lead."
    )
    return user_prompt, lead.harvester_institution_id


async def generate_reply(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    harvester_client: HarvesterClient | None = None,
) -> Draft | None:
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None or conversation.state in (
        ConversationState.OPTED_OUT,
        ConversationState.CLOSED_WON,
        ConversationState.CLOSED_LOST,
    ):
        return None

    user_prompt, _institution_id = await _build_context(session, conversation)
    raw = await call_claude(SYSTEM_PROMPT, user_prompt)
    parsed = parse_json_reply(raw)

    intent = parsed.get("intent", "unrelated")
    confidence = float(parsed.get("confidence", 0.0))
    reply_body = parsed.get("reply", "")
    citations = parsed.get("citations", [])

    escalate = intent in ("escalate", "complaint", "buying_signal")
    status = DraftStatus.PENDING_REVIEW
    if not escalate and confidence >= settings.reply_auto_send_confidence:
        status = DraftStatus.AUTO_APPROVED

    draft = Draft(
        conversation_id=conversation.id,
        body=reply_body,
        confidence=confidence,
        citations=citations,
        status=status,
    )
    session.add(draft)

    if intent in ("buying_signal", "complaint"):
        conversation.state = ConversationState.HOT_LEAD

    session.add(
        AuditLog(
            entity_type="draft",
            entity_id=draft.id if draft.id else uuid.uuid4(),
            action="draft_generated",
            actor="ai",
            details={"intent": intent, "confidence": confidence},
        )
    )

    await session.flush()

    if draft.status == DraftStatus.AUTO_APPROVED and harvester_client is not None:
        draft = await approve_and_send(session, draft, harvester_client, actor="ai")

    return draft
