from __future__ import annotations

import uuid

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.ai.claude_client import call_claude, parse_json_reply
from communicator.config import settings
from communicator.models.audit import AuditLog
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.draft import Draft, DraftStatus
from communicator.models.knowledge_base import KnowledgeBaseEntry
from communicator.models.lead import Lead
from communicator.models.message import Message
from communicator.services.alerts import send_hot_lead_alert
from communicator.services.integration_settings import get_settings_row
from communicator.services.memory import get_contact_memories, render_contact_memories_context
from communicator.services.service_catalog import get_matching_services, render_service_context

_DEFAULT_BUSINESS_NAME = "NEWAX Technologies"

_PROMPT_TEMPLATE = """You are a sales assistant for {business_name}, replying to a lead on behalf of the sales team.
{business_description}
Only make claims found in the SERVICES context below. Never invent pricing or contractual terms.
If the lead asks something outside the SERVICES context, or shows a strong buying signal, or is upset/complaining,
set intent to "escalate" and keep confidence low.
Detect the language of the lead's most recent message and write "reply" in that same language,
regardless of what language the rest of this prompt is in.
{agent_guidelines}
Respond with strict JSON only, no markdown fences:
{{"intent": "faq_question|objection|buying_signal|scheduling_request|complaint|unrelated|escalate",
 "sentiment": "positive|neutral|negative|urgent",
 "language": "BCP-47-ish short code of the lead's message, e.g. en, es, fr",
 "confidence": 0.0-1.0,
 "reply": "the message body to send back to the lead, written in `language`",
 "citations": ["service names referenced"]}}
"""


async def _build_system_prompt(session: AsyncSession) -> str:
    """
    Grounds the prompt in the Connections page's business profile +
    guidelines instead of a hardcoded business name, so the agent replies
    according to whichever business is actually connected.
    """
    row = await get_settings_row(session)
    return _PROMPT_TEMPLATE.format(
        business_name=row.business_name or _DEFAULT_BUSINESS_NAME,
        business_description=f"\n{row.business_description}\n" if row.business_description else "",
        agent_guidelines=f"\nADDITIONAL GUIDELINES:\n{row.agent_guidelines}\n"
        if row.agent_guidelines
        else "",
    )


_ALERT_INTENTS = ("buying_signal", "complaint", "escalate")
_ALERT_SENTIMENTS = ("negative", "urgent")


async def _render_knowledge_base(session: AsyncSession) -> str:
    entries = (await session.execute(select(KnowledgeBaseEntry))).scalars().all()
    if not entries:
        return ""
    body = "\n\n".join(f"### {e.title}\n{e.content}" for e in entries)
    return f"BUSINESS KNOWLEDGE BASE:\n{body}\n\n"


async def _build_context(
    session: AsyncSession, conversation: Conversation
) -> tuple[str, Contact, Lead, str]:
    contact = await session.get(Contact, conversation.contact_id)
    lead = await session.get(Lead, contact.lead_id)

    history_stmt = (
        select(Message).where(Message.conversation_id == conversation.id).order_by(Message.sent_at)
    )
    messages = (await session.execute(history_stmt)).scalars().all()
    history = "\n".join(f"{m.direction.value.upper()}: {m.body}" for m in messages)
    last_inbound_body = next(
        (m.body for m in reversed(messages) if m.direction.value == "inbound"), ""
    )

    services = await get_matching_services(session, lead.institution_type)
    service_context = render_service_context(services)
    knowledge_context = await _render_knowledge_base(session)

    # Ingest contact memories (Remembers) into prompt
    memories = await get_contact_memories(session, contact.id)
    memory_context = render_contact_memories_context(memories)

    user_prompt = (
        f"LEAD: {lead.name} ({lead.institution_type}, {lead.city})\n"
        f"QUALIFICATION: {lead.qualification_rationale}\n\n"
        f"SERVICES:\n{service_context}\n\n"
        f"{knowledge_context}"
        f"{memory_context}"
        f"CONVERSATION HISTORY:\n{history}\n\n"
        "Draft the next reply to send to this lead."
    )
    return user_prompt, contact, lead, last_inbound_body


async def generate_reply(session: AsyncSession, conversation_id: uuid.UUID) -> Draft | None:
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None or conversation.state in (
        ConversationState.OPTED_OUT,
        ConversationState.CLOSED_WON,
        ConversationState.CLOSED_LOST,
    ):
        return None

    user_prompt, contact, lead, last_inbound_body = await _build_context(session, conversation)
    system_prompt = await _build_system_prompt(session)
    raw = await call_claude(system_prompt, user_prompt)
    parsed = parse_json_reply(raw)

    intent = parsed.get("intent", "unrelated")
    sentiment = parsed.get("sentiment", "neutral")
    language = parsed.get("language", "en") or "en"
    confidence = float(parsed.get("confidence", 0.0))
    reply_body = parsed.get("reply", "")
    citations = parsed.get("citations", [])

    if language != contact.language and len(language) <= 10:
        contact.language = language

    escalate = intent in ("escalate", "complaint", "buying_signal")
    status = DraftStatus.PENDING_REVIEW
    if not escalate and confidence >= settings.reply_auto_send_confidence:
        status = DraftStatus.AUTO_APPROVED

    # Flag autonomous resolution (Resolved, no handoff) when AI answers with high confidence
    if status == DraftStatus.AUTO_APPROVED and not escalate and confidence >= 0.85:
        conversation.resolution_type = "autonomous_no_handoff"
    elif escalate:
        conversation.resolution_type = "escalated"

    draft = Draft(
        conversation_id=conversation.id,
        body=reply_body,
        confidence=confidence,
        citations=citations,
        status=status,
    )
    session.add(draft)

    needs_alert = intent in _ALERT_INTENTS or sentiment in _ALERT_SENTIMENTS
    if intent in ("buying_signal", "complaint"):
        conversation.state = ConversationState.HOT_LEAD

    session.add(
        AuditLog(
            entity_type="draft",
            entity_id=draft.id if draft.id else uuid.uuid4(),
            action="draft_generated",
            actor="ai",
            details={
                "intent": intent,
                "sentiment": sentiment,
                "confidence": confidence,
                "resolution_type": conversation.resolution_type,
            },
        )
    )

    if needs_alert:
        await send_hot_lead_alert(
            lead=lead,
            contact=contact,
            conversation=conversation,
            intent=intent,
            sentiment=sentiment,
            message_body=last_inbound_body,
        )

    await session.flush()
    return draft
