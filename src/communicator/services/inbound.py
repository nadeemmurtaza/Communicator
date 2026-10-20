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

from communicator.channels import get_provider
from communicator.compliance.optout import process_inbound_consent_signal
from communicator.harvester_client import HarvesterClient
from communicator.models.automation_rule import RuleActionType
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.services.automation import match_rule
from communicator.services.business_hours import is_within_business_hours
from communicator.services.integration_settings import get_settings_row
from communicator.services.messaging import get_or_create_conversation, send_outbound_message
from communicator.services.reply_engine import generate_reply
from communicator.services.tags import add_tag_to_contact

_TERMINAL_STATES = (
    ConversationState.OPTED_OUT,
    ConversationState.CLOSED_WON,
    ConversationState.CLOSED_LOST,
)


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

    if conversation.state not in _TERMINAL_STATES:
        await _route_reply(session, conversation, contact, body)

    return message


async def _route_reply(
    session: AsyncSession, conversation: Conversation, contact: Contact, body: str
) -> None:
    """
    Decides what, if anything, answers this inbound message: a human
    takeover or disabled auto-reply means nothing; outside business hours
    sends the away message; a matching AutomationRule short-circuits the
    AI; otherwise the AI reply engine drafts a response as before.
    """
    integration_settings = await get_settings_row(session)

    if conversation.human_takeover or not integration_settings.auto_reply_enabled:
        return

    if not is_within_business_hours(integration_settings) and integration_settings.away_message:
        try:
            channel_provider = await get_provider(contact.channel, session)
            await send_outbound_message(
                session, contact, channel_provider, integration_settings.away_message
            )
        except Exception:  # noqa: BLE001, S110 -- best-effort; never lose the inbound message
            pass
        return

    rule = await match_rule(session, body)
    if rule is not None:
        if rule.action_type == RuleActionType.AUTO_REPLY and rule.action_value:
            try:
                channel_provider = await get_provider(contact.channel, session)
                await send_outbound_message(session, contact, channel_provider, rule.action_value)
            except Exception:  # noqa: BLE001, S110
                pass
        elif rule.action_type == RuleActionType.HANDOFF:
            conversation.human_takeover = True
            await session.flush()
        elif rule.action_type == RuleActionType.ADD_TAG and rule.action_value:
            await add_tag_to_contact(session, contact, rule.action_value)
        return

    try:
        await generate_reply(session, conversation.id)
    except Exception:  # noqa: BLE001, S110 -- a drafting failure (LLM/network/parse)
        pass  # must never lose the inbound message already recorded above.
