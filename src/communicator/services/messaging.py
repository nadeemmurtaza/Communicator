"""
The only path allowed to actually send something to a Contact -- see
docs/ARCHITECTURE.md section 7 step 1. Every call here checks the
compliance gate before touching a channel provider, and always records a
Message row, whether the send succeeds or fails.
"""

from __future__ import annotations

import uuid
from datetime import UTC, datetime, timedelta

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels.base import ChannelProvider
from communicator.compliance.optout import is_sendable
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.message import Message, MessageDirection, MessageStatus

_TERMINAL_STATES = (
    ConversationState.OPTED_OUT,
    ConversationState.CLOSED_WON,
    ConversationState.CLOSED_LOST,
)

# WhatsApp Business policy: freeform "session" messages only succeed
# within 24h of the contact's last inbound message. See
# docs/ARCHITECTURE.md section 6/8.
WHATSAPP_SESSION_WINDOW = timedelta(hours=24)


class NotSendableError(RuntimeError):
    """Raised when a Contact has no OPTED_IN consent record."""


class WhatsAppTemplateRequiredError(RuntimeError):
    """
    Raised when a freeform WhatsApp send is attempted outside the 24h
    customer-initiated session window without a template (content_sid).
    """


async def whatsapp_session_open(session: AsyncSession, conversation_id: uuid.UUID) -> bool:
    stmt = (
        select(Message.sent_at)
        .where(
            Message.conversation_id == conversation_id,
            Message.direction == MessageDirection.INBOUND,
        )
        .order_by(Message.sent_at.desc())
        .limit(1)
    )
    last_inbound_at = (await session.execute(stmt)).scalar_one_or_none()
    if last_inbound_at is None:
        return False
    return datetime.now(UTC) - last_inbound_at < WHATSAPP_SESSION_WINDOW


async def get_or_create_conversation(session: AsyncSession, contact: Contact) -> Conversation:
    # Queried explicitly rather than read off contact.conversations: new
    # Conversation rows below are created via the raw contact_id FK, not
    # the ORM relationship attribute, so that in-memory collection would
    # go stale within a single session/request.
    stmt = (
        select(Conversation)
        .where(Conversation.contact_id == contact.id, Conversation.state.not_in(_TERMINAL_STATES))
        .order_by(Conversation.created_at.desc())
        .limit(1)
    )
    conversation = (await session.execute(stmt)).scalar_one_or_none()
    if conversation is not None:
        return conversation

    conversation = Conversation(contact_id=contact.id)
    session.add(conversation)
    await session.flush()
    return conversation


async def send_outbound_message(
    session: AsyncSession,
    contact: Contact,
    provider: ChannelProvider,
    body: str,
    subject: str | None = None,
    content_sid: str | None = None,
) -> Message:
    if not await is_sendable(session, contact.id):
        raise NotSendableError(f"Contact {contact.id} is not OPTED_IN; refusing to send.")

    conversation = await get_or_create_conversation(session, contact)

    if (
        contact.channel == ChannelType.WHATSAPP
        and not content_sid
        and not await whatsapp_session_open(session, conversation.id)
    ):
        raise WhatsAppTemplateRequiredError(
            f"Contact {contact.id} has no open WhatsApp session (>24h since last inbound "
            "message, or none yet); a pre-approved template (content_sid) is required."
        )

    message = Message(
        conversation_id=conversation.id,
        direction=MessageDirection.OUTBOUND,
        channel=contact.channel,
        body=body,
        status=MessageStatus.QUEUED,
    )
    session.add(message)

    try:
        result = await provider.send(contact, body, subject=subject, content_sid=content_sid)
    except Exception as exc:
        message.status = MessageStatus.FAILED
        message.provider_metadata = {"error": str(exc)}
        await session.flush()
        raise

    message.status = MessageStatus.SENT
    message.provider = result.provider
    message.provider_message_id = result.provider_message_id
    message.provider_metadata = result.raw_response
    conversation.state = ConversationState.AWAITING_REPLY

    await session.flush()
    return message
