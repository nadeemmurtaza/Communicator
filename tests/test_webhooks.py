import uuid

import pytest
from sqlalchemy import select

from communicator.api.app import app
from communicator.api.dependencies import get_harvester_client
from communicator.api.routes import webhooks as webhooks_module
from communicator.compliance.optout import get_consent_status, record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message
from communicator.webhooks.twilio_signature import compute_twilio_signature


class FakeHarvesterClient:
    def __init__(self):
        self.pushed: list[dict] = []

    async def push_outreach_outcome(self, **kwargs):
        self.pushed.append(kwargs)
        return {"id": "outcome-1"}


@pytest.fixture
def fake_harvester_client():
    fake = FakeHarvesterClient()
    app.dependency_overrides[get_harvester_client] = lambda: fake
    yield fake
    app.dependency_overrides.pop(get_harvester_client, None)


@pytest.fixture(autouse=True)
def _configure_twilio_auth_token(monkeypatch):
    monkeypatch.setattr(webhooks_module.settings, "twilio_auth_token", "test-auth-token")


async def _seed_contact(db_session, channel: ChannelType, value: str) -> Contact:
    lead = Lead(harvester_institution_id=str(uuid.uuid4()), name="Test Lead")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=channel, value=value)
    db_session.add(contact)
    await db_session.flush()
    await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")
    await db_session.flush()

    return contact


@pytest.mark.asyncio
async def test_twilio_sms_webhook_rejects_missing_signature(test_client, fake_harvester_client, db_session):
    response = await test_client.post(
        "/api/v1/webhooks/twilio/sms", data={"From": "+15550000000", "Body": "hi", "MessageSid": "SM1"}
    )
    assert response.status_code == 403
    assert fake_harvester_client.pushed == []


@pytest.mark.asyncio
async def test_twilio_sms_webhook_rejects_bad_signature(test_client, fake_harvester_client, db_session):
    response = await test_client.post(
        "/api/v1/webhooks/twilio/sms",
        data={"From": "+15550000000", "Body": "hi", "MessageSid": "SM1"},
        headers={"X-Twilio-Signature": "not-a-real-signature"},
    )
    assert response.status_code == 403


@pytest.mark.asyncio
async def test_twilio_sms_webhook_accepts_valid_signature_and_records_message(
    test_client, fake_harvester_client, db_session
):
    contact = await _seed_contact(db_session, ChannelType.SMS, "+923001234567")
    lead = await db_session.get(Lead, contact.lead_id)

    url = "http://test/api/v1/webhooks/twilio/sms"
    form = {"From": contact.value, "Body": "Interested, tell me more.", "MessageSid": "SM100"}
    signature = compute_twilio_signature("test-auth-token", url, form)

    response = await test_client.post(
        "/api/v1/webhooks/twilio/sms", data=form, headers={"X-Twilio-Signature": signature}
    )

    assert response.status_code == 200
    messages = (
        await db_session.execute(select(Message).where(Message.provider_message_id == "SM100"))
    ).scalars().all()
    assert len(messages) == 1

    assert fake_harvester_client.pushed == [
        {"institution_id": lead.harvester_institution_id, "replied": True, "reply_text": "Interested, tell me more."}
    ]


@pytest.mark.asyncio
async def test_twilio_whatsapp_webhook_strips_prefix_and_matches_contact(
    test_client, fake_harvester_client, db_session
):
    contact = await _seed_contact(db_session, ChannelType.WHATSAPP, "+923001234567")

    url = "http://test/api/v1/webhooks/twilio/whatsapp"
    form = {"From": f"whatsapp:{contact.value}", "Body": "STOP", "MessageSid": "WA1"}
    signature = compute_twilio_signature("test-auth-token", url, form)

    response = await test_client.post(
        "/api/v1/webhooks/twilio/whatsapp", data=form, headers={"X-Twilio-Signature": signature}
    )

    assert response.status_code == 200
    status = await get_consent_status(db_session, contact.id)
    assert status == ConsentStatus.OPTED_OUT


@pytest.mark.asyncio
async def test_unknown_contact_still_acks_200(test_client, fake_harvester_client, db_session):
    url = "http://test/api/v1/webhooks/twilio/sms"
    form = {"From": "+15559999999", "Body": "hello", "MessageSid": "SM-unknown"}
    signature = compute_twilio_signature("test-auth-token", url, form)

    response = await test_client.post(
        "/api/v1/webhooks/twilio/sms", data=form, headers={"X-Twilio-Signature": signature}
    )

    assert response.status_code == 200
    assert fake_harvester_client.pushed == []


@pytest.mark.asyncio
async def test_postmark_webhook_requires_valid_token(test_client, fake_harvester_client, monkeypatch, db_session):
    monkeypatch.setattr(webhooks_module.settings, "postmark_inbound_webhook_secret", "shh-secret")

    response = await test_client.post(
        "/api/v1/webhooks/postmark/inbound?token=wrong",
        json={"FromFull": {"Email": "lead@example.com"}, "TextBody": "hi", "MessageID": "pm-1"},
    )
    assert response.status_code == 403


@pytest.mark.asyncio
async def test_postmark_webhook_accepts_valid_token_and_records_message(
    test_client, fake_harvester_client, monkeypatch, db_session
):
    monkeypatch.setattr(webhooks_module.settings, "postmark_inbound_webhook_secret", "shh-secret")
    contact = await _seed_contact(db_session, ChannelType.EMAIL, "lead@example.com")

    response = await test_client.post(
        "/api/v1/webhooks/postmark/inbound?token=shh-secret",
        json={"FromFull": {"Email": contact.value}, "TextBody": "What's the pricing?", "MessageID": "pm-2"},
    )

    assert response.status_code == 200
    messages = (await db_session.execute(select(Message).where(Message.provider_message_id == "pm-2"))).scalars().all()
    assert len(messages) == 1

    conversation = (
        await db_session.execute(select(Conversation).where(Conversation.id == messages[0].conversation_id))
    ).scalar_one()
    assert conversation.state == ConversationState.ENGAGED
