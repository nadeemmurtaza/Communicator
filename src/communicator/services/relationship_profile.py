"""
Customer relationship profile: a compact, provenance-aware summary of
everything NEWAX (or any tenant) already knows about a contact -- not
the full message log, which is expensive and eventually hits context
limits, but a short retrieval layer the agent loads before responding.

Reuses existing records instead of adding a parallel memory system:
- Identity: Contact
- Business memory: Contact + latest Inquiry
- Relationship memory: Conversation history across ALL of the contact's
  threads (not just the current one) + ContactMemory facts
- Commercial memory: Inquiry records (status, capability, qualification)
- Service memory: Ticket records, when present

Everything returned carries its own source so the AI prompt (and any
human-facing profile view) can tell a verified fact from an inference.
"""

from __future__ import annotations

import uuid

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.contact_memory import ContactMemory
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.inquiry import Inquiry

_ACTIVE_STATES = (
    ConversationState.NEW,
    ConversationState.AWAITING_REPLY,
    ConversationState.ENGAGED,
)


async def _contact_ids_for_customer(
    session: AsyncSession, contact_id: uuid.UUID
) -> list[uuid.UUID]:
    """
    Resolves every channel identity (Contact) belonging to the same
    Customer as the given contact, so the relationship is read across
    WhatsApp, Instagram, etc. -- not just the one channel the current
    message came in on. Falls back to the single contact if it isn't
    linked to a Customer yet.
    """
    contact = await session.get(Contact, contact_id)
    if contact is None or contact.customer_id is None:
        return [contact_id]
    stmt = select(Contact.id).where(Contact.customer_id == contact.customer_id)
    return list((await session.execute(stmt)).scalars().all())


async def get_relationship_summary(
    session: AsyncSession,
    contact_id: uuid.UUID,
    exclude_conversation_id: uuid.UUID | None = None,
) -> dict:
    """
    exclude_conversation_id should be the conversation the caller is
    currently acting within, when there is one. By the time this runs on
    an inbound message, that conversation already has a live state
    (NEW/ENGAGED/...) -- counting it toward "relationship_stage" makes
    every first-time contact look like a returning active prospect.
    """
    contact_ids = await _contact_ids_for_customer(session, contact_id)

    conversations = (
        (
            await session.execute(
                select(Conversation)
                .where(Conversation.contact_id.in_(contact_ids))
                .order_by(Conversation.created_at.desc())
            )
        )
        .scalars()
        .all()
    )
    prior_conversations = (
        [c for c in conversations if c.id != exclude_conversation_id]
        if exclude_conversation_id
        else conversations
    )
    inquiries = (
        (
            await session.execute(
                select(Inquiry)
                .where(Inquiry.contact_id.in_(contact_ids))
                .order_by(Inquiry.created_at.desc())
            )
        )
        .scalars()
        .all()
    )
    memories = (
        (
            await session.execute(
                select(ContactMemory)
                .where(ContactMemory.contact_id.in_(contact_ids))
                .order_by(ContactMemory.noted_at.desc())
                .limit(10)
            )
        )
        .scalars()
        .all()
    )

    closed_states = {ConversationState.CLOSED_WON, ConversationState.CLOSED_LOST}
    prior_closed = [c for c in prior_conversations if c.state in closed_states]

    if not prior_conversations:
        stage = "new_prospect"
    elif any(c.state == ConversationState.CLOSED_WON for c in prior_conversations):
        stage = "client"
    elif any(
        c.state in _ACTIVE_STATES or c.state == ConversationState.HOT_LEAD
        for c in prior_conversations
    ):
        stage = "active_prospect"
    else:
        stage = "dormant"

    latest_inquiry = inquiries[0] if inquiries else None
    last_conversation = conversations[0] if conversations else None

    channel_rows = (
        await session.execute(
            select(Contact.channel, Contact.value).where(Contact.id.in_(contact_ids))
        )
    ).all()

    return {
        "contact_id": str(contact_id),
        "channels_seen": [{"channel": c.value, "value": v} for c, v in channel_rows],
        "relationship_stage": stage,
        "total_conversations": len(conversations),
        "total_inquiries": len(inquiries),
        "last_interaction_at": last_conversation.updated_at.isoformat()
        if last_conversation
        else None,
        "last_interaction_outcome": last_conversation.resolution_type
        if last_conversation
        else None,
        "latest_inquiry": (
            {
                "organization_name": latest_inquiry.organization_name,
                "industry": latest_inquiry.industry,
                "problem_description": latest_inquiry.problem_description,
                "capability_mapping": latest_inquiry.capability_mapping,
                "status": latest_inquiry.status.value,
                "qualification_score": latest_inquiry.qualification_score,
                "source": "inquiry_record",
            }
            if latest_inquiry
            else None
        ),
        "prior_closed_outcomes": [
            {
                "state": c.state.value,
                "resolution_type": c.resolution_type,
                "source": "conversation_record",
            }
            for c in prior_closed[:3]
        ],
        "memories": [
            {
                "fact": m.fact,
                "category": m.category,
                "confidence": m.confidence,
                "noted_at": m.noted_at.isoformat(),
                "source": "contact_memory",
            }
            for m in memories
        ],
    }


def render_relationship_context(summary: dict) -> str:
    """Renders the relationship summary into the AI reply prompt. Returns ''
    for a genuinely new contact, so the prompt doesn't fabricate history."""
    if summary["relationship_stage"] == "new_prospect" and not summary["memories"]:
        return ""

    lines = [f"Relationship stage: {summary['relationship_stage']}"]
    lines.append(f"Prior conversations: {summary['total_conversations']}")

    inquiry = summary.get("latest_inquiry")
    if inquiry:
        lines.append(
            f"Most recent documented need ({inquiry['status']}): {inquiry['problem_description']} "
            f"[industry: {inquiry['industry'] or 'unknown'}, capability: "
            f"{', '.join(inquiry['capability_mapping']) or 'unmapped'}] -- source: inquiry record"
        )

    for outcome in summary.get("prior_closed_outcomes", []):
        lines.append(
            f"Previous conversation closed as {outcome['state']} -- source: conversation record"
        )

    for mem in summary.get("memories", [])[:5]:
        lines.append(
            f"{mem['noted_at'][:10]}: {mem['fact']} [{mem['category']}] -- source: contact memory"
        )

    body = "\n".join(f"- {line}" for line in lines)
    return (
        "CUSTOMER RELATIONSHIP (verified records only -- do not invent anything beyond this):\n"
        f"{body}\n\n"
    )
