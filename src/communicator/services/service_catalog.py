from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.service import Service


async def get_matching_services(session: AsyncSession, institution_type: str) -> list[Service]:
    stmt = select(Service)
    services = list((await session.execute(stmt)).scalars().all())
    if not institution_type:
        return services
    matched = [
        s
        for s in services
        if not s.target_institution_types or institution_type in s.target_institution_types
    ]
    return matched or services


def render_service_context(services: list[Service]) -> str:
    blocks = []
    for s in services:
        faq_lines = "\n".join(f"Q: {f.get('question')}\nA: {f.get('answer')}" for f in s.faq)
        blocks.append(
            f"### {s.name}\n{s.summary}\n{s.details}\nPricing notes: {s.pricing_notes}\n{faq_lines}"
        )
    return "\n\n".join(blocks)
