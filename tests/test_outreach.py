import uuid

import pytest

from communicator.channels.base import ChannelProvider, SendResult
from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.lead import Lead
from communicator.models.message import MessageStatus
from communicator.services import outreach as outreach_module
from communicator.services.messaging import NotSendableError
from communicator.services.outreach import send_first_touch


class FakeProvider(ChannelProvider):
    async def send(self, contact, body, subject=None):
        return SendResult(provider="fake", provider_message_id="fake-1", raw_response={})


class FakeHarvesterClient:
    def __init__(self):
        self.pushed: list[dict] = []

    async def push_outreach_outcome(self, **kwargs):
        self.pushed.append(kwargs)
        return {"id": "outcome-1"}


@pytest.mark.asyncio
async def test_send_first_touch_sends_and_reports_outcome(db_session, monkeypatch):
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Test Lead")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.EMAIL, value="lead@example.com")
    db_session.add(contact)
    await db_session.flush()
    await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")
    await db_session.flush()

    monkeypatch.setattr(outreach_module, "get_provider", lambda channel: FakeProvider())
    harvester_client = FakeHarvesterClient()

    message = await send_first_touch(
        db_session,
        lead,
        contact,
        "Hello! We help clinics like yours automate patient intake.",
        harvester_client,
        subject="Quick question",
    )

    assert message.status == MessageStatus.SENT
    assert len(harvester_client.pushed) == 1
    assert harvester_client.pushed[0]["institution_id"] == lead.harvester_institution_id
    assert harvester_client.pushed[0]["replied"] is False


@pytest.mark.asyncio
async def test_send_first_touch_does_not_report_outcome_without_consent(db_session, monkeypatch):
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Test Lead")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.EMAIL, value="lead@example.com")
    db_session.add(contact)
    await db_session.flush()
    # deliberately no consent record

    monkeypatch.setattr(outreach_module, "get_provider", lambda channel: FakeProvider())
    harvester_client = FakeHarvesterClient()

    with pytest.raises(NotSendableError):
        await send_first_touch(db_session, lead, contact, "Hello!", harvester_client)

    assert harvester_client.pushed == []
