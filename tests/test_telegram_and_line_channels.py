import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.message import Message, MessageDirection


@pytest.mark.asyncio
async def test_telegram_channel_flow(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch Telegram channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/telegram")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "telegram"
    assert "botProfile" in data
    assert data["botProfile"]["botUsername"] == "@bayworks_support_bot"
    assert len(data["commands"]) >= 2
    assert "webhookConfig" in data
    assert data["health"]["isConnected"] is True

    # 2. Update Telegram settings
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/telegram",
        json={
            "name": "Bay Works Global Telegram",
            "bot_token": "bot_test_token_custom_999",
            "bot_username": "@bayworks_enterprise_bot",
            "secret_token": "tg_custom_secret_777",
            "greeting_enabled": True,
            "greeting_message": "Hello from Bay Works Telegram! How can we assist you today?",
            "out_of_office_enabled": False,
            "csat_enabled": True,
            "commands": [
                {"command": "/start", "description": "Start conversation"},
                {"command": "/quote", "description": "Request instant quote"},
                {"command": "/agent", "description": "Speak with live support"},
            ],
            "timezone": "America/New_York",
        },
    )
    assert update_res.status_code == 200
    updated = update_res.json()
    assert updated["inbox"]["name"] == "Bay Works Global Telegram"
    assert updated["botProfile"]["botUsername"] == "@bayworks_enterprise_bot"
    assert len(updated["commands"]) == 3

    # 3. Verify Telegram bot connection
    verify_res = await test_client.post("/api/v1/inboxes/channels/telegram/verify")
    assert verify_res.status_code == 200
    assert verify_res.json()["verified"] is True

    # 4. Dispatch Telegram diagnostic test message
    test_msg_res = await test_client.post(
        "/api/v1/inboxes/channels/telegram/test-message",
        json={
            "recipient_chat_id": "987654321",
            "recipient_name": "Dmitri Rostov",
            "message": "Testing automated Telegram dispatch from Bay Works Support.",
        },
    )
    assert test_msg_res.status_code == 200
    msg_data = test_msg_res.json()
    assert msg_data["status"] == "sent"
    assert "messageId" in msg_data
    assert msg_data["recipient"] == "987654321"

    # 5. List Telegram events
    events_res = await test_client.get("/api/v1/inboxes/channels/telegram/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 1
    assert any("Testing automated Telegram dispatch" in e["body"] for e in events)

    # 6. Webhook inbound message processing
    webhook_msg_res = await test_client.post(
        "/api/v1/webhooks/telegram",
        headers={"X-Telegram-Bot-Api-Secret-Token": "tg_custom_secret_777"},
        json={
            "update_id": 9001,
            "message": {
                "message_id": 4412,
                "date": 1711000000,
                "chat": {
                    "id": 987654321,
                    "type": "private",
                    "first_name": "Dmitri",
                    "username": "dmitri_r",
                },
                "from": {
                    "id": 987654321,
                    "first_name": "Dmitri",
                    "username": "dmitri_r",
                },
                "text": "Hi, I have a question about your enterprise API limits.",
            },
        },
    )
    assert webhook_msg_res.status_code == 200
    assert webhook_msg_res.json()["status"] == "ok"

    # Verify contact was auto-provisioned and message recorded
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.TELEGRAM, Contact.value == "987654321"
    )
    contact = (await db_session.execute(contact_stmt)).scalar_one_or_none()
    assert contact is not None

    conv_stmt = select(Conversation).where(Conversation.contact_id == contact.id)
    conv = (await db_session.execute(conv_stmt)).scalar_one_or_none()
    assert conv is not None

    msg_stmt = select(Message).where(Message.provider_message_id == "tg_4412")
    msg = (await db_session.execute(msg_stmt)).scalar_one_or_none()
    assert msg is not None
    assert msg.body == "Hi, I have a question about your enterprise API limits."
    assert msg.direction == MessageDirection.INBOUND

    # 7. Webhook callback query (inline button click)
    webhook_cb_res = await test_client.post(
        "/api/v1/webhooks/telegram",
        headers={"X-Telegram-Bot-Api-Secret-Token": "tg_custom_secret_777"},
        json={
            "update_id": 9002,
            "callback_query": {
                "id": "cb_opt_99",
                "from": {"id": 987654321, "first_name": "Dmitri"},
                "message": {"message_id": 4412, "chat": {"id": 987654321}},
                "data": "ACTION_SPEAK_TO_AGENT",
            },
        },
    )
    assert webhook_cb_res.status_code == 200
    assert webhook_cb_res.json()["status"] == "ok"


@pytest.mark.asyncio
async def test_line_channel_flow(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch LINE channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/line")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "line"
    assert "accountProfile" in data
    assert data["accountProfile"]["lineAccountId"] == "@bayworks"
    assert len(data["quickReplies"]) >= 2
    assert "webhookConfig" in data
    assert data["health"]["isConnected"] is True

    # 2. Update LINE settings
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/line",
        json={
            "name": "Bay Works Asia Support",
            "channel_id": "1657890123",
            "channel_secret": "mock_secret_line",
            "channel_access_token": "line_test_token_asia_123",
            "line_account_id": "@bayworks_asia",
            "greeting_enabled": True,
            "greeting_message": "Welcome to Bay Works LINE Support! How can we assist you today?",
            "out_of_office_enabled": False,
            "csat_enabled": True,
            "quick_replies": [
                {"label": "Plans", "text": "What are your subscription plans?"},
                {"label": "Billing", "text": "I have an invoice inquiry"},
            ],
            "timezone": "Asia/Tokyo",
        },
    )
    assert update_res.status_code == 200
    updated = update_res.json()
    assert updated["inbox"]["name"] == "Bay Works Asia Support"
    assert updated["accountProfile"]["lineAccountId"] == "@bayworks_asia"
    assert len(updated["quickReplies"]) == 2

    # 3. Verify LINE connection
    verify_res = await test_client.post("/api/v1/inboxes/channels/line/verify")
    assert verify_res.status_code == 200
    assert verify_res.json()["verified"] is True

    # 4. Dispatch LINE diagnostic push message
    test_msg_res = await test_client.post(
        "/api/v1/inboxes/channels/line/test-message",
        json={
            "recipient_user_id": "U4af4980629fbcc4c3fffed0cfa255d6",
            "recipient_name": "Kenji Sato",
            "message": "Testing automated LINE push message from Bay Works.",
        },
    )
    assert test_msg_res.status_code == 200
    msg_data = test_msg_res.json()
    assert msg_data["status"] == "sent"
    assert "messageId" in msg_data
    assert msg_data["recipient"] == "U4af4980629fbcc4c3fffed0cfa255d6"

    # 5. List LINE events
    events_res = await test_client.get("/api/v1/inboxes/channels/line/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 1
    assert any("Testing automated LINE push message" in e["body"] for e in events)

    # 6. Webhook inbound message processing
    webhook_inbound_res = await test_client.post(
        "/api/v1/webhooks/line",
        json={
            "destination": "U4af4980629fbcc4c3fffed0cfa255d6",
            "events": [
                {
                    "type": "message",
                    "message": {
                        "type": "text",
                        "id": "line_in_msg_881",
                        "text": "Hello, I would like to schedule a product walkthrough.",
                    },
                    "timestamp": 1711000000000,
                    "source": {
                        "type": "user",
                        "userId": "U4af4980629fbcc4c3fffed0cfa255d6",
                    },
                    "replyToken": "token_dummy_reply_123",
                }
            ],
        },
    )
    assert webhook_inbound_res.status_code == 200
    assert webhook_inbound_res.json()["status"] == "ok"

    # Verify contact was auto-provisioned and message recorded
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.LINE,
        Contact.value == "U4af4980629fbcc4c3fffed0cfa255d6",
    )
    contact = (await db_session.execute(contact_stmt)).scalar_one_or_none()
    assert contact is not None

    conv_stmt = select(Conversation).where(Conversation.contact_id == contact.id)
    conv = (await db_session.execute(conv_stmt)).scalar_one_or_none()
    assert conv is not None

    msg_stmt = select(Message).where(Message.provider_message_id == "line_in_msg_881")
    msg = (await db_session.execute(msg_stmt)).scalar_one_or_none()
    assert msg is not None
    assert msg.body == "Hello, I would like to schedule a product walkthrough."
    assert msg.direction == MessageDirection.INBOUND


@pytest.mark.asyncio
async def test_telegram_and_line_shared_inbox_reply(
    test_client: AsyncClient, db_session: AsyncSession
):
    # 1. Create a Telegram conversation via test message
    tg_send = await test_client.post(
        "/api/v1/inboxes/channels/telegram/test-message",
        json={
            "recipient_chat_id": "777888999",
            "recipient_name": "Dmitri Shared",
            "message": "Initial diagnostic ping to Dmitri.",
        },
    )
    assert tg_send.status_code == 200

    tg_contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.TELEGRAM, Contact.value == "777888999"
    )
    tg_contact = (await db_session.execute(tg_contact_stmt)).scalar_one_or_none()
    assert tg_contact is not None

    tg_conv_stmt = select(Conversation).where(Conversation.contact_id == tg_contact.id)
    tg_conv = (await db_session.execute(tg_conv_stmt)).scalar_one_or_none()
    assert tg_conv is not None

    agent_tg_reply = await test_client.post(
        f"/api/v1/conversations/{tg_conv.id}/messages",
        json={
            "body": "Hi Dmitri, our enterprise tier includes unlimited API throughput.",
            "sender_name": "Riley Brooks",
            "is_private": False,
        },
    )
    assert agent_tg_reply.status_code == 200
    tg_reply_data = agent_tg_reply.json()
    assert tg_reply_data["body"] == "Hi Dmitri, our enterprise tier includes unlimited API throughput."
    assert tg_reply_data["senderName"] == "Riley Brooks"

    # 2. Create a LINE conversation via test message
    line_send = await test_client.post(
        "/api/v1/inboxes/channels/line/test-message",
        json={
            "recipient_user_id": "Uline_shared_12345",
            "recipient_name": "Kenji Shared",
            "message": "Initial diagnostic ping to Kenji.",
        },
    )
    assert line_send.status_code == 200

    line_contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.LINE,
        Contact.value == "Uline_shared_12345",
    )
    line_contact = (await db_session.execute(line_contact_stmt)).scalar_one_or_none()
    assert line_contact is not None

    line_conv_stmt = select(Conversation).where(Conversation.contact_id == line_contact.id)
    line_conv = (await db_session.execute(line_conv_stmt)).scalar_one_or_none()
    assert line_conv is not None

    agent_line_reply = await test_client.post(
        f"/api/v1/conversations/{line_conv.id}/messages",
        json={
            "body": "Hello Kenji, we would love to schedule a walkthrough! Here is our calendar link.",
            "sender_name": "Kenji Support Agent",
            "is_private": False,
        },
    )
    assert agent_line_reply.status_code == 200
    line_reply_data = agent_line_reply.json()
    assert (
        line_reply_data["body"]
        == "Hello Kenji, we would love to schedule a walkthrough! Here is our calendar link."
    )
    assert line_reply_data["senderName"] == "Kenji Support Agent"
