from __future__ import annotations

from datetime import UTC, datetime, timedelta

import pytest
from httpx import AsyncClient, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels.whatsapp import MetaWhatsAppProvider
from communicator.compliance.optout import record_consent
from communicator.models.broadcast import (
    Broadcast,
    BroadcastRecipient,
    BroadcastStatus,
    RecipientStatus,
)
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead
from communicator.models.message import Message
from communicator.services.memory import extract_contact_memories
from communicator.services.tags import add_tag_to_contact


@pytest.mark.asyncio
async def test_meta_whatsapp_api_error_and_retry(monkeypatch):
    """Verifies MetaWhatsAppProvider retries on transient errors and parses error details."""
    provider = MetaWhatsAppProvider(
        access_token="test_token",
        phone_number_id="1234567890",
    )

    attempt_count = 0

    async def mock_post(client, url, json=None, headers=None):
        nonlocal attempt_count
        attempt_count += 1
        if attempt_count == 1:
            # Simulate transient 429 rate limit
            return Response(
                429,
                json={
                    "error": {
                        "message": "Rate limit hit",
                        "type": "OAuthException",
                        "code": 130429,
                        "error_subcode": 0,
                    }
                },
            )
        # Succeeded on retry
        return Response(
            200,
            json={"messages": [{"id": "wamid.retry_success"}]},
        )

    # Monkeypatch client.post
    from httpx import AsyncClient as HttpxAsyncClient

    monkeypatch.setattr(HttpxAsyncClient, "post", mock_post)

    res = await provider.send_text("+14155551234", "Hello world!")
    assert res.provider_message_id == "wamid.retry_success"
    assert attempt_count == 2


@pytest.mark.asyncio
async def test_webhook_message_deduplication(
    test_client: AsyncClient, db_session: AsyncSession, monkeypatch
):
    """Verifies identical Meta webhook messages are not processed multiple times."""
    # Create lead and contact
    lead = Lead(
        harvester_institution_id="test:dedup-lead", name="Dedup Corp", institution_type="tech"
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155559999",
        person_name="Dedup User",
    )
    db_session.add(contact)
    await db_session.flush()

    # Bypass meta signature check in test
    import communicator.api.routes.webhooks as webhooks_module

    async def mock_require_sig(session, body, sig):
        return None

    monkeypatch.setattr(webhooks_module, "_require_meta_signature", mock_require_sig)

    webhook_payload = {
        "object": "whatsapp_business_account",
        "entry": [
            {
                "id": "biz_123",
                "changes": [
                    {
                        "value": {
                            "messaging_product": "whatsapp",
                            "messages": [
                                {
                                    "id": "wamid.unique_12345",
                                    "from": "14155559999",
                                    "timestamp": "1710000000",
                                    "type": "text",
                                    "text": {"body": "Testing deduplication message"},
                                }
                            ],
                        }
                    }
                ],
            }
        ],
    }

    # Send 1st webhook
    res1 = await test_client.post("/api/v1/webhooks/meta/whatsapp", json=webhook_payload)
    assert res1.status_code == 200

    # Send 2nd identical webhook (duplicate delivery from Meta)
    res2 = await test_client.post("/api/v1/webhooks/meta/whatsapp", json=webhook_payload)
    assert res2.status_code == 200

    # Verify only 1 message row exists
    msgs = (
        (
            await db_session.execute(
                select(Message).where(Message.provider_message_id == "wamid.unique_12345")
            )
        )
        .scalars()
        .all()
    )
    assert len(msgs) == 1


@pytest.mark.asyncio
async def test_webhook_status_failure_parsing(
    test_client: AsyncClient, db_session: AsyncSession, monkeypatch
):
    """Verifies detailed error messages like 'Number is not on WhatsApp' are attached to recipients."""
    lead = Lead(
        harvester_institution_id="test:status-lead", name="Status Corp", institution_type="tech"
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+14155558888")
    db_session.add(contact)
    await db_session.flush()

    broadcast = Broadcast(
        name="Test Failure Parsing",
        template_name="hello_world",
        tag_name="test",
        status=BroadcastStatus.SENDING,
    )
    db_session.add(broadcast)
    await db_session.flush()

    recipient = BroadcastRecipient(
        broadcast_id=broadcast.id,
        contact_id=contact.id,
        status=RecipientStatus.SENT,
        provider_message_id="wamid.fail_123",
    )
    db_session.add(recipient)
    await db_session.flush()

    import communicator.api.routes.webhooks as webhooks_module

    async def mock_require_sig(session, body, sig):
        return None

    monkeypatch.setattr(webhooks_module, "_require_meta_signature", mock_require_sig)

    status_payload = {
        "object": "whatsapp_business_account",
        "entry": [
            {
                "id": "biz_123",
                "changes": [
                    {
                        "value": {
                            "messaging_product": "whatsapp",
                            "statuses": [
                                {
                                    "id": "wamid.fail_123",
                                    "status": "failed",
                                    "timestamp": "1710001000",
                                    "errors": [
                                        {
                                            "code": 131026,
                                            "title": "Message undeliverable",
                                            "message": "Number is not on WhatsApp",
                                        }
                                    ],
                                }
                            ],
                        }
                    }
                ],
            }
        ],
    }

    res = await test_client.post("/api/v1/webhooks/meta/whatsapp", json=status_payload)
    assert res.status_code == 200

    await db_session.refresh(recipient)
    assert recipient.status == RecipientStatus.FAILED
    assert recipient.error == "Number is not on WhatsApp"


@pytest.mark.asyncio
async def test_scheduled_campaign_processor(
    test_client: AsyncClient, db_session: AsyncSession, monkeypatch
):
    """Verifies that scheduled campaigns with past timestamps are dispatched automatically."""
    from communicator.channels.base import SendResult

    async def mock_send_template(*args, **kwargs):
        return SendResult(provider="meta", provider_message_id="wamid.mock_sched_123")

    monkeypatch.setattr(MetaWhatsAppProvider, "send_template", mock_send_template)

    lead = Lead(
        harvester_institution_id="test:sched-lead", name="Sched Corp", institution_type="tech"
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155557777",
        person_name="Alex Rivera",
    )
    db_session.add(contact)
    await db_session.flush()
    await add_tag_to_contact(db_session, contact, "vip-scheduled")
    await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test_setup")
    await db_session.flush()

    past_time = datetime.now(UTC) - timedelta(minutes=10)
    campaign = Broadcast(
        name="Scheduled Flash Sale",
        template_name="flash_sale",
        tag_name="vip-scheduled",
        status=BroadcastStatus.SCHEDULED,
        scheduled_at=past_time,
        template_params={"1": "Alex", "2": "30%"},
        body_preview="Hi {{1}}, get {{2}} off today!",
    )
    db_session.add(campaign)
    await db_session.flush()

    # Trigger process-scheduled endpoint
    res = await test_client.post("/api/v1/broadcasts/process-scheduled")
    assert res.status_code == 200
    data = res.json()
    assert data["processed"] >= 1

    await db_session.refresh(campaign)
    assert campaign.status == BroadcastStatus.COMPLETED
    assert campaign.audience_count >= 1
    assert campaign.sent_count >= 1


@pytest.mark.asyncio
async def test_enhanced_contact_memory_extraction(db_session: AsyncSession):
    """Tests multi-lingual, role, negative constraints, and billing preference extraction."""
    lead = Lead(harvester_institution_id="test:mem-lead", name="Mem Corp", institution_type="tech")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+14155556666", person_name="Elena"
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    # Multi-lingual & role
    msg1 = "Hola, hablo español y por favor responder en español. I am the CTO of our organization."
    extracted1 = await extract_contact_memories(db_session, contact, conv, msg1)
    facts1 = [m.fact for m in extracted1]
    assert any("Spanish" in f or "Español" in f for f in facts1)
    assert any("CTO" in f for f in facts1)

    # Negative schedule constraint & billing
    msg2 = "Please keep in mind: never message on weekends and we pay by invoice."
    extracted2 = await extract_contact_memories(db_session, contact, conv, msg2)
    facts2 = [m.fact for m in extracted2]
    assert any("weekends" in f.lower() for f in facts2)
    assert any("INVOICE" in f for f in facts2)
