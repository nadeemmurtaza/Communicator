from __future__ import annotations

from datetime import datetime
from typing import Any

from sqlalchemy import func, or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.models.contact import ChannelType, Contact
from communicator.models.segment import ContactSegment
from communicator.models.tag import Tag


def build_segment_filter_stmt(filters: dict[str, Any]):
    """
    Constructs a SQLAlchemy select query applying dynamic criteria:
    channels, tags (any/all), search text, and creation date bounds.
    """
    stmt = select(Contact).options(selectinload(Contact.tags))

    # 1. Filter by channels
    channels = filters.get("channels")
    if channels and isinstance(channels, list) and len(channels) > 0:
        valid_channels = []
        for c in channels:
            try:
                valid_channels.append(ChannelType(c.lower()))
            except ValueError:
                pass
        if valid_channels:
            stmt = stmt.where(Contact.channel.in_(valid_channels))

    # 2. Filter by tags
    tags = filters.get("tags")
    tag_mode = filters.get("tag_match_mode", "any")
    if tags and isinstance(tags, list) and len(tags) > 0:
        clean_tags = [str(t).strip() for t in tags if str(t).strip()]
        if clean_tags:
            if tag_mode == "all":
                for t_name in clean_tags:
                    stmt = stmt.where(Contact.tags.any(Tag.name == t_name))
            else:
                stmt = stmt.where(Contact.tags.any(Tag.name.in_(clean_tags)))

    # 3. Filter by search query
    search = filters.get("search")
    if search and isinstance(search, str) and search.strip():
        term = f"%{search.strip()}%"
        stmt = stmt.where(or_(Contact.person_name.ilike(term), Contact.value.ilike(term)))

    # 4. Filter by creation date range
    created_after = filters.get("created_after")
    if created_after:
        try:
            dt = datetime.fromisoformat(str(created_after))
            stmt = stmt.where(Contact.created_at >= dt)
        except Exception:  # noqa: BLE001, S110
            pass

    created_before = filters.get("created_before")
    if created_before:
        try:
            dt = datetime.fromisoformat(str(created_before))
            stmt = stmt.where(Contact.created_at <= dt)
        except Exception:  # noqa: BLE001, S110
            pass

    return stmt


async def get_segment_contacts(
    session: AsyncSession, segment: ContactSegment, limit: int = 100, offset: int = 0
) -> list[Contact]:
    """Retrieves contacts matching the dynamic segment filters."""
    stmt = build_segment_filter_stmt(segment.filters or {})
    stmt = stmt.order_by(Contact.created_at.desc()).limit(limit).offset(offset)
    result = await session.execute(stmt)
    return list(result.scalars().unique().all())


async def count_segment_contacts(session: AsyncSession, segment: ContactSegment) -> int:
    """Calculates total count of contacts matching the segment."""
    base_stmt = build_segment_filter_stmt(segment.filters or {})
    subq = base_stmt.subquery()
    count_stmt = select(func.count()).select_from(subq)
    result = await session.execute(count_stmt)
    return int(result.scalar_one_or_none() or 0)


async def seed_default_segments(session: AsyncSession) -> list[ContactSegment]:
    """Initializes standard dynamic audience segments if none exist."""
    existing = (await session.execute(select(ContactSegment))).scalars().first()
    if existing is not None:
        return list(
            (await session.execute(select(ContactSegment).order_by(ContactSegment.name)))
            .scalars()
            .all()
        )

    defaults = [
        ContactSegment(
            name="All Omnichannel Contacts",
            description="Entire unified customer database across all communication channels.",
            filters={},
        ),
        ContactSegment(
            name="WhatsApp VIP Audience",
            description="High-value VIP contacts opted-in for official WhatsApp communications.",
            filters={"channels": ["whatsapp"], "tags": ["vip"], "tag_match_mode": "any"},
        ),
        ContactSegment(
            name="Email & SMS Subscribers",
            description="Direct outreach audience on Email and SMS messaging channels.",
            filters={"channels": ["email", "sms"], "tag_match_mode": "any"},
        ),
        ContactSegment(
            name="Social & TikTok Community",
            description="Social media followers across TikTok, Instagram, and Facebook.",
            filters={"channels": ["tiktok", "instagram", "facebook"], "tag_match_mode": "any"},
        ),
    ]

    for seg in defaults:
        session.add(seg)
    await session.flush()
    return defaults
