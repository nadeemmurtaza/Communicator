from __future__ import annotations

import re
import uuid
from datetime import UTC, datetime

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.contact_memory import ContactMemory
from communicator.models.conversation import Conversation

# Regex heuristics for fast, zero-latency extraction of durable customer preferences
_PREFERENCE_PATTERNS = [
    # Channel preferences
    (
        r"(?:prefer|please only|only reach me via|reach me on|contact me via|email me instead of|don't call|do not call)\s+(email|calls?|phone|whatsapp|sms|text)",
        "channel_preference",
        lambda m: f"Prefers {m.group(1).lower()} over other channels",
    ),
    (
        r"(?:never call|no phone calls|don't call me|do not call me|text or email only|whatsapp only)",
        "channel_preference",
        lambda m: "Strict channel rule: Text/WhatsApp/Email only, never phone call",
    ),
    (
        r"(?:their team prefers|our team prefers|we prefer|i prefer)\s+([a-zA-Z0-9\s]{3,40}\s+over\s+[a-zA-Z0-9\s]{3,40})",
        "channel_preference",
        lambda m: f"Their team prefers {m.group(1).strip()}",
    ),
    # Language preferences
    (
        r"(?:prefer|only speak|please write in|respond in|hablo|hablamos|parlo|parle)\s+(english|spanish|español|french|français|german|deutsch|arabic|hindi|portuguese|português)",
        "language",
        lambda m: f"Language preference: {m.group(1).title()}",
    ),
    (
        r"(?:por favor responder en|favor de escribir en|responda en)\s+([a-zA-Záéíóúñ]+)",
        "language",
        lambda m: f"Language preference: {m.group(1).title()}",
    ),
    # Timing / Scheduling constraints
    (
        r"(?:available|reach me|call me|best time to reach me is)\s+(?:between|after|before|around)?\s*(\d{1,2}(?::\d{2})?\s*(?:am|pm)?\s*(?:to|-)\s*\d{1,2}(?::\d{2})?\s*(?:am|pm)?|\d{1,2}\s*(?:am|pm))",
        "schedule",
        lambda m: f"Best reachable time window: {m.group(1).strip()}",
    ),
    (
        r"(?:only on weekdays|weekdays only|no weekends|available monday to friday)",
        "schedule",
        lambda m: "Available on weekdays only; no weekends",
    ),
    (
        r"(?:never message on|don't message on|do not contact on|no calls on)\s+(weekends?|saturdays?|sundays?|fridays?|holidays?)",
        "schedule",
        lambda m: f"Restricted days: Do not contact on {m.group(1).lower()}",
    ),
    # Timezone
    (
        r"(?:timezone is|in timezone|based in)\s+([A-Z]{2,4}|[A-Za-z]+(?:\s+time)?)",
        "schedule",
        lambda m: f"Timezone: {m.group(1).strip()}",
    ),
    # Roles & Decision authority
    (
        r"(?:i am the|i'm the|as the)\s+(founder|ceo|cto|cfo|coo|vp of engineering|procurement manager|decision maker|owner|head of product)",
        "organization",
        lambda m: (
            f"Contact role: {m.group(1).upper() if len(m.group(1)) <= 4 else m.group(1).title()}"
        ),
    ),
    (
        r"(?:need approval from|need to check with|have to consult)\s+(?:our|my)?\s*([a-zA-Z\s]{3,30}\s+(?:team|board|director|partner|manager))",
        "constraint",
        lambda m: f"Decision constraint: Requires approval from {m.group(1).strip()}",
    ),
    # Constraints & Requirements
    (
        r"(?:require|need|must have)\s+((?:enterprise\s+)?sla|99\.\d+%\s+uptime|hipaa|soc\s*2|iso\s*27001|gdpr|data residency)",
        "constraint",
        lambda m: f"Requires {m.group(1).strip()} compliance / guarantee",
    ),
    (
        r"(?:budget is|maximum budget of|budget capped at)\s+([\$€£]?\s*\d+[,\d]*(?:\s*(?:k|kilo|thousand|usd|eur|gbp))?)",
        "constraint",
        lambda m: f"Budget parameter: {m.group(1).strip()}",
    ),
    (
        r"(?:we pay by|pay via|payment terms are|require)\s+((?:net\s*\d+|invoice|wire transfer|ach|purchase order|credit card))",
        "billing",
        lambda m: f"Payment terms/method: {m.group(1).upper()}",
    ),
    # Timeline & Urgency
    (
        r"(?:target launch|deadline is|need this by|launching by|go live by)\s+([a-zA-Z0-9\s]{3,30})",
        "timeline",
        lambda m: f"Project target/deadline: {m.group(1).strip()}",
    ),
    # Explicit "note that / remember that / keep in mind"
    (
        r"(?:please note that|note that|keep in mind that|remember that|fyi:?)\s+([^.!?\n]{8,80})",
        "preference",
        lambda m: m.group(1).strip().capitalize(),
    ),
]


async def extract_contact_memories(
    session: AsyncSession,
    contact: Contact,
    conversation: Conversation,
    message_body: str,
    message_id: uuid.UUID | None = None,
) -> list[ContactMemory]:
    """
    Listens to inbound conversation text and extracts durable facts or
    preferences to persist to the Contact record.
    """
    if not message_body or len(message_body.strip()) < 5:
        return []

    # Fetch existing memories for this contact to avoid duplicate facts
    existing_stmt = select(ContactMemory).where(ContactMemory.contact_id == contact.id)
    existing_memories = (await session.execute(existing_stmt)).scalars().all()
    existing_facts_lower = {m.fact.lower().strip() for m in existing_memories}

    extracted: list[ContactMemory] = []
    text = message_body.strip()

    for pattern, category, fact_builder in _PREFERENCE_PATTERNS:
        match = re.search(pattern, text, re.IGNORECASE)
        if match:
            fact_text = fact_builder(match)
            if fact_text.lower().strip() not in existing_facts_lower:
                memory = ContactMemory(
                    contact_id=contact.id,
                    fact=fact_text,
                    category=category,
                    confidence=0.95,
                    source_message_id=message_id,
                    noted_at=datetime.now(UTC),
                )
                session.add(memory)
                extracted.append(memory)
                existing_facts_lower.add(fact_text.lower().strip())

    if extracted:
        await session.flush()

    return extracted


async def get_contact_memories(session: AsyncSession, contact_id: uuid.UUID) -> list[ContactMemory]:
    """Retrieves all memories for a contact in chronological order."""
    stmt = (
        select(ContactMemory)
        .where(ContactMemory.contact_id == contact_id)
        .order_by(ContactMemory.noted_at.desc())
    )
    return list((await session.execute(stmt)).scalars().all())


def render_contact_memories_context(memories: list[ContactMemory]) -> str:
    """Renders remembered contact facts into Claude's prompt context."""
    if not memories:
        return ""
    lines = []
    for m in memories:
        date_str = m.noted_at.strftime("%b %d")
        lines.append(f"- Noted · {date_str}: {m.fact} [{m.category}]")
    content = "\n".join(lines)
    return (
        f"CUSTOMER MEMORY & REMEMBERED PREFERENCES (REMEMBERS):\n"
        f"{content}\n"
        f"Always respect and comply with these customer preferences in your reply.\n\n"
    )


async def add_manual_memory(
    session: AsyncSession,
    contact_id: uuid.UUID,
    fact: str,
    category: str = "preference",
) -> ContactMemory:
    """Manually records a memory / preference for a contact from the UI."""
    memory = ContactMemory(
        contact_id=contact_id,
        fact=fact.strip(),
        category=category.strip() or "preference",
        confidence=1.0,
        noted_at=datetime.now(UTC),
    )
    session.add(memory)
    await session.flush()
    return memory


async def delete_contact_memory(
    session: AsyncSession, contact_id: uuid.UUID, memory_id: uuid.UUID
) -> bool:
    """Deletes a contact memory."""
    stmt = select(ContactMemory).where(
        ContactMemory.id == memory_id, ContactMemory.contact_id == contact_id
    )
    mem = (await session.execute(stmt)).scalar_one_or_none()
    if mem is None:
        return False
    await session.delete(mem)
    await session.flush()
    return True
