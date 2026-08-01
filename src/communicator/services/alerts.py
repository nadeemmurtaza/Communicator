"""
Phase 5 sentiment-based hot-lead alerts -- see docs/ARCHITECTURE.md
section 7. reply_engine already flags a Conversation HOT_LEAD in the DB;
this is the piece that actually pages a human about it instead of relying
on someone polling /drafts or /conversations.
"""

from __future__ import annotations

import httpx

from communicator.config import settings
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead


async def send_hot_lead_alert(
    *,
    lead: Lead,
    contact: Contact,
    conversation: Conversation,
    intent: str,
    sentiment: str,
    message_body: str,
) -> None:
    """
    Best-effort notification to sales. Posts to a generic Slack-compatible
    incoming webhook (SALES_ALERT_WEBHOOK_URL) rather than assuming a
    specific paging tool -- swap the POST body/target for PagerDuty, email,
    etc. behind this same function later without touching callers.
    """
    if not settings.sales_alert_webhook_url:
        return

    text = (
        f":rotating_light: Hot lead: *{lead.name}* ({lead.institution_type}, {lead.city})\n"
        f"Intent: `{intent}` | Sentiment: `{sentiment}` | Channel: `{contact.channel.value}`\n"
        f"> {message_body}\n"
        f"Conversation: {conversation.id}"
    )

    async with httpx.AsyncClient(timeout=10.0) as client:
        try:
            await client.post(settings.sales_alert_webhook_url, json={"text": text})
        except httpx.HTTPError:
            # Best-effort: a down webhook must never block the reply
            # pipeline or lose the underlying HOT_LEAD state, which is
            # already durably recorded on the Conversation/AuditLog.
            pass
