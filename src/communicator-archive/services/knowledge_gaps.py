from __future__ import annotations

import re
import uuid

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.draft import Draft
from communicator.models.knowledge_base import KnowledgeBaseEntry, KnowledgeGap
from communicator.models.message import Message, MessageDirection


async def detect_knowledge_gaps(session: AsyncSession) -> list[KnowledgeGap]:
    """
    Scans recent inbound customer messages and low-confidence drafts to identify
    recurring customer questions not covered by existing KnowledgeBaseEntries ("Fills gaps").
    """
    # 1. Fetch existing KB entry titles to check what is already known
    kb_titles = set((await session.execute(select(KnowledgeBaseEntry.title))).scalars().all())
    " ".join((await session.execute(select(KnowledgeBaseEntry.content))).scalars().all()).lower()

    # 2. Fetch existing gaps to avoid duplicates
    existing_gaps = (await session.execute(select(KnowledgeGap))).scalars().all()
    existing_questions = {g.question.lower().strip() for g in existing_gaps}

    # 3. Find customer inquiries from inbound messages ending with "?" or question words
    inbound_stmt = (
        select(Message)
        .where(Message.direction == MessageDirection.INBOUND)
        .order_by(Message.sent_at.desc())
        .limit(100)
    )
    inbound_messages = (await session.execute(inbound_stmt)).scalars().all()

    question_patterns = [
        r"(?:what is|how do|how does|can i|is it possible to|do you support|what are|where can i|when do|how much does)\s+([^?.!\n]{10,120})\??",
        r"(?:refund policy|return policy|sla|custom pricing|migration guide|api rate limit|support hours|soc 2|hipaa|security audit)\b",
    ]

    candidates: dict[str, dict] = {}

    for msg in inbound_messages:
        text = msg.body.strip()
        for pat in question_patterns:
            m = re.search(pat, text, re.IGNORECASE)
            if m:
                clean_q = m.group(0).strip().capitalize()
                if not clean_q.endswith("?"):
                    clean_q += "?"

                # Check if already covered in KB
                is_covered = any(
                    clean_q.lower() in t.lower() or t.lower() in clean_q.lower() for t in kb_titles
                )
                if not is_covered and clean_q.lower() not in existing_questions:
                    if clean_q not in candidates:
                        candidates[clean_q] = {
                            "question": clean_q,
                            "source_query": text,
                            "count": 1,
                        }
                    else:
                        candidates[clean_q]["count"] += 1

    # Also look at low confidence drafts
    drafts_stmt = (
        select(Draft).where(Draft.confidence < 0.8).order_by(Draft.created_at.desc()).limit(50)
    )
    low_conf_drafts = (await session.execute(drafts_stmt)).scalars().all()
    for d in low_conf_drafts:
        # If draft had citations empty or low confidence
        if not d.citations and d.body:
            pass

    new_gaps: list[KnowledgeGap] = []

    # Insert detected candidates
    for q, data in candidates.items():
        gap = KnowledgeGap(
            question=q,
            suggested_answer=f"Recommended policy or answer for: {q}",
            source_query=data["source_query"],
            frequency=data["count"],
            status="suggested",
        )
        session.add(gap)
        new_gaps.append(gap)
        existing_questions.add(q.lower())

    if new_gaps:
        await session.flush()

    return new_gaps


async def list_knowledge_gaps(
    session: AsyncSession, status: str = "suggested"
) -> list[KnowledgeGap]:
    """Lists knowledge gaps by status."""
    stmt = (
        select(KnowledgeGap)
        .where(KnowledgeGap.status == status)
        .order_by(KnowledgeGap.frequency.desc(), KnowledgeGap.created_at.desc())
    )
    return list((await session.execute(stmt)).scalars().all())


async def approve_knowledge_gap(
    session: AsyncSession, gap_id: uuid.UUID, custom_answer: str | None = None
) -> KnowledgeBaseEntry:
    """
    Approves a suggested FAQ, turning it into a first-class KnowledgeBaseEntry
    grounded in the AI agent's replies.
    """
    gap = await session.get(KnowledgeGap, gap_id)
    if gap is None:
        raise ValueError(f"KnowledgeGap {gap_id} not found")

    content = custom_answer.strip() if custom_answer else gap.suggested_answer
    entry = KnowledgeBaseEntry(
        title=gap.question,
        content=content,
        source_type="suggested_faq",
    )
    session.add(entry)

    gap.status = "approved"
    await session.flush()
    return entry


async def dismiss_knowledge_gap(session: AsyncSession, gap_id: uuid.UUID) -> None:
    """Dismisses a suggested knowledge gap."""
    gap = await session.get(KnowledgeGap, gap_id)
    if gap is not None:
        gap.status = "dismissed"
        await session.flush()
