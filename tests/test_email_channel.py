import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.inbox import Inbox
from communicator.models.message import Message


@pytest.mark.asyncio
async def test_email_channel_config_and_verification(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Verify Email inbox exists in /inboxes listing
    res = await test_client.get("/api/v1/inboxes")
    assert res.status_code == 200
    inboxes = res.json()
    email_inbox = next((i for i in inboxes if i["channel_type"] == "email"), None)
    assert email_inbox is not None
    assert "Email" in email_inbox["name"]

    # 2. Get Email channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/email")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "email"
    assert "smtpConfig" in data
    assert "imapConfig" in data
    assert "webhookConfig" in data
    assert "templateConfig" in data
    assert data["smtpConfig"]["port"] == 587
    assert data["imapConfig"]["port"] == 993

    # 3. Update Email channel settings
    res = await test_client.post(
        "/api/v1/inboxes/channels/email",
        json={
            "name": "Bay Works Global Support Email",
            "email_from_address": "help@bayworks.co",
            "email_from_name": "Bay Works Global Helpdesk",
            "email_reply_to": "help@bayworks.co",
            "smtp_host": "smtp.bayworks.co",
            "smtp_port": 587,
            "smtp_username": "help@bayworks.co",
            "smtp_encryption": "tls",
            "imap_host": "imap.bayworks.co",
            "imap_port": 993,
            "imap_username": "help@bayworks.co",
            "imap_ssl": True,
            "imap_folder": "INBOX",
            "email_signature": "--\nBest regards,\n{{agent.name}}\nBay Works Global Support",
            "email_footer": "Bay Works Global Inc. · 100 Market St, San Francisco, CA",
            "greeting_enabled": True,
            "greeting_message": "Thank you for reaching out to Bay Works Global Support.",
        },
    )
    assert res.status_code == 200
    updated = res.json()
    assert updated["inbox"]["name"] == "Bay Works Global Support Email"
    assert updated["smtpConfig"]["fromAddress"] == "help@bayworks.co"
    assert updated["smtpConfig"]["fromName"] == "Bay Works Global Helpdesk"

    # 4. Verify SMTP connection & SPF/DKIM
    res_verify = await test_client.post("/api/v1/inboxes/channels/email/verify")
    assert res_verify.status_code == 200
    verify_data = res_verify.json()
    assert verify_data["status"] == "ok"
    assert verify_data["smtpVerified"] is True
    assert verify_data["spfDkimVerified"] is True

    # 5. Trigger IMAP mailbox sync
    res_sync = await test_client.post("/api/v1/inboxes/channels/email/sync-imap")
    assert res_sync.status_code == 200
    sync_data = res_sync.json()
    assert sync_data["status"] == "ok"
    assert "INBOX" in sync_data["mailbox"]


@pytest.mark.asyncio
async def test_email_inbound_webhook_and_threading(test_client: AsyncClient, db_session: AsyncSession):
    # Inbound email webhook payload
    res = await test_client.post(
        "/api/v1/webhooks/email/inbound",
        json={
            "From": '"Victoria Vance" <victoria.vance@vancetech.io>',
            "FromName": "Victoria Vance",
            "To": "support@bayworks.co",
            "Subject": "Urgent: API rate limits during weekend launch",
            "TextBody": (
                "Hi Bay Works Support team,\n\n"
                "We are launching our mobile app this Saturday and anticipate a 5x spike in webhook traffic. "
                "Can we request a temporary rate limit increase?\n\n"
                "Thanks,\nVictoria\n\n"
                "On Thu, Oct 15, 2026 at 10:00 AM Bay Works <support@bayworks.co> wrote:\n"
                "> Welcome to Bay Works developer portal.\n"
            ),
            "MessageID": "<msg_vance_20261021@vancetech.io>",
        },
    )
    assert res.status_code == 200
    inbound_data = res.json()
    assert inbound_data["status"] == "ok"
    assert inbound_data["from"] == "victoria.vance@vancetech.io"
    conv_id = inbound_data["conversationId"]

    # Verify contact was auto-provisioned
    stmt = select(Contact).where(Contact.value == "victoria.vance@vancetech.io")
    contact = (await db_session.execute(stmt)).scalars().first()
    assert contact is not None
    assert contact.channel == ChannelType.EMAIL
    assert contact.person_name == "Victoria Vance"

    # Verify message body has quoted header stripped
    msg_stmt = select(Message).where(Message.provider_message_id == "<msg_vance_20261021@vancetech.io>")
    msg = (await db_session.execute(msg_stmt)).scalars().first()
    assert msg is not None
    assert "spike in webhook traffic" in msg.body
    # Quoted text below "On Thu, Oct 15 ... wrote:" should be removed
    assert "Welcome to Bay Works developer portal" not in msg.body


@pytest.mark.asyncio
async def test_email_outbound_and_shared_inbox_reply(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Send outbound test email via channel diagnostics
    res_test = await test_client.post(
        "/api/v1/inboxes/channels/email/test-message",
        json={
            "recipient_email": "daniel.okafor@fintech.ng",
            "recipient_name": "Daniel Okafor",
            "subject": "Scheduled Maintenance Notice",
            "message": "Hello Daniel, please be advised our core API will undergo 15 minutes of routine maintenance on Sunday.",
        },
    )
    assert res_test.status_code == 200
    test_data = res_test.json()
    assert test_data["status"] == "ok"
    assert test_data["recipient"] == "daniel.okafor@fintech.ng"
    conv_id = test_data["conversationId"]

    # 2. Agent replies inside the shared inbox conversation thread
    res_reply = await test_client.post(
        f"/api/v1/conversations/{conv_id}/messages",
        json={
            "body": "Hi Daniel, following up on our maintenance schedule -- maintenance has completed with zero downtime.",
            "sender_name": "Riley Brooks",
        },
    )
    assert res_reply.status_code == 200
    reply_data = res_reply.json()
    assert reply_data["status"] in ("queued", "sent", "delivered")

    # 3. Retrieve live email events
    res_events = await test_client.get("/api/v1/inboxes/channels/email/events")
    assert res_events.status_code == 200
    events = res_events.json()
    assert len(events) >= 1
    assert any("daniel.okafor@fintech.ng" in str(ev.get("body", "")) or "Maintenance" in str(ev.get("body", "")) for ev in events)
