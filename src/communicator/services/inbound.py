"""
Turns a parsed inbound message (already extracted from a provider's
webhook payload -- see communicator.api.routes.webhooks) into a recorded
Message, running the compliance gate first. Intent classification and
AI-drafted replies are Phase 3 (see docs/ARCHITECTURE.md section 7); this
module only gets as far as recording the message and updating state.
"""
from __future__ import annotations

import httpx
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.compliance.optout import process_inbound_consent_signal
from communicator.harvester_client import HarvesterClient
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.services.messaging import get_or_create_conversation

_TERMINAL_STATES = (ConversationState.OPTED_OUT, ConversationState.CLOSED_WON, ConversationState.CLOSED_LOST)


class UnknownContactError(RuntimeError):
    """Raised when an inbound message doesn't match a Contact we know about."""


async def _find_contact(session: AsyncSession, channel: ChannelType, value: str) -> Contact:
    stmt = select(Contact).where(Contact.channel == channel, Contact.value == value)
    contact = (await session.execute(stmt)).scalar_one_or_none()
    if contact is None:
        raise UnknownContactError(f"No Contact found for {channel.value}:{value}")
    return contact


async def process_inbound_message(
    session: AsyncSession,
    *,
    channel: ChannelType,
    from_value: str,
    body: str,
    provider: str,
    provider_message_id: str,
    harvester_client: HarvesterClient,
) -> Message:
    contact = await _find_contact(session, channel, from_value)
    conversation = await get_or_create_conversation(session, contact)

    consent_change = await process_inbound_consent_signal(session, contact.id, body)

    message = Message(
        conversation_id=conversation.id,
        direction=MessageDirection.INBOUND,
        channel=channel,
        body=body,
        provider=provider,
        provider_message_id=provider_message_id,
        status=MessageStatus.RECEIVED,
    )
    session.add(message)

    if consent_change == ConsentStatus.OPTED_OUT:
        conversation.state = ConversationState.OPTED_OUT
    elif conversation.state not in _TERMINAL_STATES:
        conversation.state = ConversationState.ENGAGED

    await session.flush()

    lead = await session.get(Lead, contact.lead_id)
    if lead is not None:
        try:
            await harvester_client.push_outreach_outcome(
                institution_id=lead.harvester_institution_id,
                replied=True,
                reply_text=body,
            )
        except httpx.HTTPError:
            # Best-effort: an unreachable Harvester must never cost us the
            # inbound message we already have safely recorded.
            pass

    return message
