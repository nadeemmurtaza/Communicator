"""
Inbound message processor for all channels with enterprise WhatsApp automation.

Handles:
1. Contact resolution & Conversation creation
2. Mandatory compliance gate (opt-in / opt-out / STOP / START)
3. 24-hour WhatsApp session window timestamp tracking
4. In-chat agent commands (#bot, #resolve, #csat, #tag)
5. CSAT feedback rating responses
6. Interactive WhatsApp navigation tree & catalog menus
7. Business hours / after-hours away messaging
8. Advanced keyword, regex, button click, and intent automations (13 action types)
9. Welcome greeting with interactive quick replies
10. Contact Memory & preference extraction ("Remembers")
11. AI reply engine fallback with autonomous resolution ("Responds")
"""

from __future__ import annotations

from datetime import UTC, datetime

import httpx
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.channels.whatsapp import MetaWhatsAppProvider
from communicator.compliance.optout import process_inbound_consent_signal
from communicator.harvester_client import HarvesterClient
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.draft import DraftStatus
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.services.business_hours import is_within_business_hours
from communicator.services.draft_actions import approve_and_send
from communicator.services.integration_settings import get_settings_row
from communicator.services.memory import extract_contact_memories
from communicator.services.messaging import get_or_create_conversation, send_outbound_message
from communicator.services.reply_engine import generate_reply
from communicator.services.whatsapp_automations import (
    execute_rule_action,
    handle_agent_commands,
    handle_csat_response,
    handle_navigation_menu,
    match_advanced_rule,
    send_welcome_greeting,
    should_send_welcome_greeting,
)

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
    interactive_payload_id: str | None = None,
) -> Message:
    if provider_message_id:
        existing_stmt = select(Message).where(Message.provider_message_id == provider_message_id)
        existing_msg = (await session.execute(existing_stmt)).scalar_one_or_none()
        if existing_msg is not None:
            return existing_msg

    contact = await _find_contact(session, channel, from_value)
    conversation = await get_or_create_conversation(session, contact)

    now = datetime.now(UTC)
    is_welcome_candidate = await should_send_welcome_greeting(session, conversation, now)
    conversation.last_inbound_at = now

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

    # Automatically extract durable contact preferences / memories ("Remembers")
    try:
        await extract_contact_memories(
            session, contact, conversation, body, message_id=message.id
        )
    except Exception:
        pass

    # Automatically mark message as read on WhatsApp if Meta Cloud API is used
    if channel == ChannelType.WHATSAPP and provider == "meta" and provider_message_id:
        try:
            channel_provider = await get_provider(channel, session)
            if isinstance(channel_provider, MetaWhatsAppProvider):
                await channel_provider.mark_as_read(provider_message_id)
        except Exception:
            pass

    lead = await session.get(Lead, contact.lead_id)
    if lead is not None:
        try:
            await harvester_client.push_outreach_outcome(
                institution_id=lead.harvester_institution_id,
                replied=True,
                reply_text=body,
            )
        except httpx.HTTPError:
            pass

    if conversation.state not in _TERMINAL_STATES:
        await _route_reply(
            session,
            conversation,
            contact,
            body,
            interactive_payload_id=interactive_payload_id,
            is_welcome_candidate=is_welcome_candidate,
            harvester_client=harvester_client,
        )

    return message


async def _route_reply(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    body: str,
    interactive_payload_id: str | None = None,
    is_welcome_candidate: bool = False,
    harvester_client: HarvesterClient | None = None,
) -> None:
    """
    Evaluates in order:
    1. Agent commands (#bot, #resolve, #csat, #tag)
    2. CSAT rating response
    3. WhatsApp interactive menu navigation
    4. Human takeover check
    5. After-hours / Business hours away message
    6. Advanced automation rules (keywords, regex, button click, list select, 13 action types)
    7. Welcome greeting automation (first touch or >24h idle)
    8. AI reply engine fallback with autonomous resolution ("Responds")
    """
    # 1. In-chat agent control commands
    if await handle_agent_commands(session, conversation, contact, body):
        return

    # 2. CSAT rating survey responses
    if await handle_csat_response(session, conversation, contact, body, interactive_payload_id):
        return

    # 3. WhatsApp Interactive Navigation Menus (services catalog, pricing, location)
    if contact.channel == ChannelType.WHATSAPP:
        if await handle_navigation_menu(
            session, conversation, contact, interactive_payload_id, body
        ):
            return

    integration_settings = await get_settings_row(session)

    # 4. Human takeover / Auto-reply disabled check
    if conversation.human_takeover or not integration_settings.auto_reply_enabled:
        return

    # 5. After-Hours / Business Hours away message
    if not is_within_business_hours(integration_settings) and integration_settings.away_message:
        try:
            channel_provider = await get_provider(contact.channel, session)
            await send_outbound_message(
                session, contact, channel_provider, integration_settings.away_message
            )
        except Exception:
            pass
        return

    # 6. Advanced Automation Rules Engine
    rule = await match_advanced_rule(session, body, payload_id=interactive_payload_id)
    if rule is not None:
        try:
            if await execute_rule_action(session, rule, conversation, contact):
                return
        except Exception:
            pass

    # 7. Welcome / Greeting Automation for WhatsApp
    if contact.channel == ChannelType.WHATSAPP and is_welcome_candidate:
        try:
            await send_welcome_greeting(session, conversation, contact)
            return
        except Exception:
            pass

    # 8. AI reply engine fallback
    try:
        draft = await generate_reply(session, conversation.id)
        if (
            draft is not None
            and draft.status == DraftStatus.AUTO_APPROVED
            and harvester_client is not None
        ):
            await approve_and_send(session, draft, harvester_client, actor="ai")
    except Exception:
        pass
