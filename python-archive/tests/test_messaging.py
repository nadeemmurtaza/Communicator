import uuid

import pytest
from sqlalchemy import select

from communicator.channels.base import ChannelProvider, SendResult
from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageStatus
from communicator.services.messaging import NotSendableError, send_outbound_message


class FakeProvider(ChannelProvider):
    def __init__(self, should_fail: bool = False):
        self.should_fail = should_fail
        self.sent: list[tuple[str, str, str | None]] = []

    async def send(self, contact, body, subject=None):
        if self.should_fail:
            raise RuntimeError("provider unavailable")
        self.sent.append((contact.value, body, subject))
        return SendResult(
            provider="fake", provider_message_id="fake-123", raw_response={"ok": True}
        )


async def _make_contact(db_session, opted_in: bool) -> Contact:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Test Lead")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.EMAIL, value="lead@example.com")
    db_session.add(contact)
    await db_session.flush()

    if opted_in:
        await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")
        await db_session.flush()

    return contact


@pytest.mark.asyncio
async def test_send_refuses_without_consent(db_session):
    contact = await _make_contact(db_session, opted_in=False)
    provider = FakeProvider()

    with pytest.raises(NotSendableError):
        await send_outbound_message(db_session, contact, provider, "Hello")

    assert provider.sent == []


@pytest.mark.asyncio
async def test_send_records_message_and_advances_conversation(db_session):
    contact = await _make_contact(db_session, opted_in=True)
    provider = FakeProvider()

    message = await send_outbound_message(
        db_session, contact, provider, "Hello there", subject="Hi"
    )

    assert message.status == MessageStatus.SENT
    assert message.provider == "fake"
    assert message.provider_message_id == "fake-123"
    assert provider.sent == [("lead@example.com", "Hello there", "Hi")]

    conversation = (
        await db_session.execute(
            select(Conversation).where(Conversation.id == message.conversation_id)
        )
    ).scalar_one()
    assert conversation.state == ConversationState.AWAITING_REPLY


@pytest.mark.asyncio
async def test_send_marks_message_failed_on_provider_error(db_session):
    contact = await _make_contact(db_session, opted_in=True)
    provider = FakeProvider(should_fail=True)

    with pytest.raises(RuntimeError):
        await send_outbound_message(db_session, contact, provider, "Hello there")

    conversation_ids = select(Conversation.id).where(Conversation.contact_id == contact.id)
    messages = (
        (
            await db_session.execute(
                select(Message).where(Message.conversation_id.in_(conversation_ids))
            )
        )
        .scalars()
        .all()
    )

    assert len(messages) == 1
    assert messages[0].status == MessageStatus.FAILED
    assert "provider unavailable" in messages[0].provider_metadata["error"]


@pytest.mark.asyncio
async def test_second_send_reuses_open_conversation(db_session):
    contact = await _make_contact(db_session, opted_in=True)
    provider = FakeProvider()

    first = await send_outbound_message(db_session, contact, provider, "First touch")
    second = await send_outbound_message(db_session, contact, provider, "Follow-up")

    assert first.conversation_id == second.conversation_id
