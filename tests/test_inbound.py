import uuid

import pytest
from sqlalchemy import select

from communicator.compliance.optout import get_consent_status, record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import MessageDirection, MessageStatus
from communicator.services.inbound import UnknownContactError, process_inbound_message


class FakeHarvesterClient:
    def __init__(self):
        self.pushed: list[dict] = []

    async def push_outreach_outcome(self, **kwargs):
        self.pushed.append(kwargs)
        return {"id": "outcome-1"}


async def _make_lead_and_contact(db_session, opted_in: bool = True) -> tuple[Lead, Contact]:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Test Lead")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.SMS, value="+923001234567")
    db_session.add(contact)
    await db_session.flush()

    if opted_in:
        await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")
        await db_session.flush()

    return lead, contact


@pytest.mark.asyncio
async def test_unknown_contact_raises(db_session):
    harvester_client = FakeHarvesterClient()
    with pytest.raises(UnknownContactError):
        await process_inbound_message(
            db_session,
            channel=ChannelType.SMS,
            from_value="+15550000000",
            body="Hello",
            provider="twilio",
            provider_message_id="SM1",
            harvester_client=harvester_client,
        )


@pytest.mark.asyncio
async def test_regular_reply_records_message_and_engages_conversation(db_session):
    lead, contact = await _make_lead_and_contact(db_session)
    harvester_client = FakeHarvesterClient()

    message = await process_inbound_message(
        db_session,
        channel=ChannelType.SMS,
        from_value=contact.value,
        body="Yes, tell me more about pricing.",
        provider="twilio",
        provider_message_id="SM2",
        harvester_client=harvester_client,
    )

    assert message.direction == MessageDirection.INBOUND
    assert message.status == MessageStatus.RECEIVED

    conversation = (
        await db_session.execute(
            select(Conversation).where(Conversation.id == message.conversation_id)
        )
    ).scalar_one()
    assert conversation.state == ConversationState.ENGAGED

    assert len(harvester_client.pushed) == 1
    assert harvester_client.pushed[0]["institution_id"] == lead.harvester_institution_id
    assert harvester_client.pushed[0]["replied"] is True


@pytest.mark.asyncio
async def test_stop_keyword_opts_out_and_closes_conversation(db_session):
    _lead, contact = await _make_lead_and_contact(db_session)
    harvester_client = FakeHarvesterClient()

    message = await process_inbound_message(
        db_session,
        channel=ChannelType.SMS,
        from_value=contact.value,
        body="STOP",
        provider="twilio",
        provider_message_id="SM3",
        harvester_client=harvester_client,
    )

    consent_status = await get_consent_status(db_session, contact.id)
    assert consent_status == ConsentStatus.OPTED_OUT

    conversation = (
        await db_session.execute(
            select(Conversation).where(Conversation.id == message.conversation_id)
        )
    ).scalar_one()
    assert conversation.state == ConversationState.OPTED_OUT


@pytest.mark.asyncio
async def test_harvester_outage_does_not_lose_the_message(db_session):
    _lead, contact = await _make_lead_and_contact(db_session)

    class FailingHarvesterClient:
        async def push_outreach_outcome(self, **kwargs):
            import httpx

            raise httpx.ConnectError("connection refused")

    message = await process_inbound_message(
        db_session,
        channel=ChannelType.SMS,
        from_value=contact.value,
        body="Still interested, what's next?",
        provider="twilio",
        provider_message_id="SM4",
        harvester_client=FailingHarvesterClient(),
    )

    assert message.id is not None
    assert message.status == MessageStatus.RECEIVED
