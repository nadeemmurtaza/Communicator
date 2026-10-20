"""
Keyword-triggered rules evaluated on every inbound message before the AI
reply engine runs -- same job as the standalone reply bot's rulesEngine:
cheap, deterministic shortcuts ("menu", "agent", "STOP"-adjacent) that a
business wants guaranteed literal behavior for, not an LLM's best guess.
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.automation_rule import AutomationRule, RuleMatchType


async def match_rule(session: AsyncSession, message_body: str) -> AutomationRule | None:
    text = message_body.strip().lower()
    if not text:
        return None

    stmt = (
        select(AutomationRule)
        .where(AutomationRule.is_active.is_(True))
        .order_by(AutomationRule.priority.desc())
    )
    rules = (await session.execute(stmt)).scalars().all()

    for rule in rules:
        keyword = rule.trigger_keyword.strip().lower()
        if not keyword:
            continue
        if rule.match_type == RuleMatchType.EXACT and text == keyword:
            return rule
        if rule.match_type == RuleMatchType.CONTAINS and keyword in text:
            return rule

    return None
