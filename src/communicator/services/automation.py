"""
Keyword-triggered rules evaluated on every inbound message before the AI
reply engine runs -- cheap, deterministic shortcuts ("menu", "agent", "STOP"-adjacent)
that a business wants guaranteed literal behavior for, not an LLM's best guess.
"""

from __future__ import annotations

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.automation_rule import AutomationRule
from communicator.services.whatsapp_automations import match_advanced_rule


async def match_rule(session: AsyncSession, message_body: str) -> AutomationRule | None:
    return await match_advanced_rule(session, message_body)
