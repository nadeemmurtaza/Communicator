from __future__ import annotations

import hashlib
import hmac
import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels.tiktok import TikTokProvider
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.inbox import Inbox
from communicator.models.message import Message, MessageDirection


@pytest.mark.asyncio
async def test_tiktok_provider_unit():
    provider = TikTokProvider(
        access_token="tiktok_test_token_12345",
        app_id="tt_app_mock_6789",
        app_secret="mock_secret_tiktok",
        business_account_id="tt_biz_acc_1001",
    )

    # 1. Test simulated send
    contact = Contact(
        channel=ChannelType.TIKTOK,
        value="tt_user_open_id_1001",
        person_name="TikTok User",
    )
    res = await provider.send(contact, "Hello from test unit")
    assert res.provider == "tiktok"
    assert res.provider_message_id.startswith("tt_msg_")
    assert res.raw_response.get("simulated") is True

    # 2. Test simulated comment reply
    cmt_res = await provider.reply_video_comment("c_12345", "Thank you for commenting!")
    assert cmt_res["status"] == "success"
    assert cmt_res["comment_id"] == "c_12345"
    assert cmt_res["simulated"] is True

    # 3. Test signature verification
    payload = b'{"event":"test"}'
    secret = "my_secret_key"
    sig = hmac.new(secret.encode("utf-8"), payload, hashlib.sha256).hexdigest()
    assert TikTokProvider.verify_signature(payload, sig, secret) is True
    assert TikTokProvider.verify_signature(payload, "invalid_sig", secret) is False
    assert TikTokProvider.verify_signature(payload, sig, "wrong_secret") is False


@pytest.mark.asyncio
async def test_tiktok_channel_inbox_flow(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch TikTok channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/tiktok")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "tiktok"
    assert "webhookConfig" in data
    assert "businessProfile" in data
    assert data["businessProfile"]["appId"] == "tt_app_mock_6789"
    assert data["commentAutoReply"]["enabled"] is True
    assert data["health"]["isConnected"] is True

    # 2. Update TikTok channel settings
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/tiktok",
        json={
            "name": "Bay Works TikTok Support",
            "app_id": "tt_app_custom_888",
            "app_secret": "custom_tt_secret_999",
            "access_token": "tiktok_test_custom_token",
            "business_account_id": "tt_biz_acc_custom_777",
            "webhook_verify_token": "custom_tt_verify_token",
            "comment_auto_reply_enabled": True,
            "comment_auto_reply_text": "Thanks for your comment! DM us for prompt support. 🚀",
            "greeting_enabled": True,
            "greeting_message": "Welcome to Bay Works on TikTok! How can we help?",
            "out_of_office_enabled": False,
            "csat_enabled": True,
            "timezone": "America/Los_Angeles",
        },
    )
    assert update_res.status_code == 200
    updated = update_res.json()
    assert updated["inbox"]["name"] == "Bay Works TikTok Support"
    assert updated["businessProfile"]["appId"] == "tt_app_custom_888"
    assert updated["commentAutoReply"]["template"] == "Thanks for your comment! DM us for prompt support. 🚀"

    # 3. Verify TikTok connection
    verify_res = await test_client.post("/api/v1/inboxes/channels/tiktok/verify")
    assert verify_res.status_code == 200
    assert verify_res.json()["verified"] is True
    assert verify_res.json()["appId"] == "tt_app_custom_888"

    # 4. Dispatch TikTok diagnostic test direct message
    test_msg_res = await test_client.post(
        "/api/v1/inboxes/channels/tiktok/test-message",
        json={
            "recipient_open_id": "tt_recipient_open_id_4455",
            "recipient_name": "Maya Lin",
            "message": "Testing automated direct message from Bay Works TikTok agent.",
        },
    )
    assert test_msg_res.status_code == 200
    msg_data = test_msg_res.json()
    assert msg_data["status"] == "sent"
    assert "messageId" in msg_data
    assert msg_data["recipient"] == "tt_recipient_open_id_4455"

    # 5. Dispatch TikTok diagnostic video comment reply
    test_cmt_res = await test_client.post(
        "/api/v1/inboxes/channels/tiktok/test-comment-reply",
        json={
            "comment_id": "c_video_comment_8899",
            "video_id": "v_product_launch_101",
            "recipient_open_id": "tt_recipient_open_id_4455",
            "recipient_name": "Maya Lin",
            "reply_text": "Thanks for watching Maya! The product is available on our website.",
        },
    )
    assert test_cmt_res.status_code == 200
    cmt_data = test_cmt_res.json()
    assert cmt_data["status"] == "sent"
    assert cmt_data["commentId"] == "c_video_comment_8899"
    assert cmt_data["videoId"] == "v_product_launch_101"

    # 6. List TikTok events
    events_res = await test_client.get("/api/v1/inboxes/channels/tiktok/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 2
    assert any("Testing automated direct message" in e["body"] for e in events)
    assert any("Thanks for watching Maya" in e["body"] for e in events)


@pytest.mark.asyncio
async def test_tiktok_webhook_verification(test_client: AsyncClient):
    # Challenge parameter verification
    res1 = await test_client.get("/api/v1/webhooks/tiktok?challenge=tt_challenge_abc_123")
    assert res1.status_code == 200
    assert res1.text == "tt_challenge_abc_123"

    # hub.challenge alias verification
    res2 = await test_client.get("/api/v1/webhooks/tiktok?hub.challenge=tt_hub_token_456")
    assert res2.status_code == 200
    assert res2.text == "tt_hub_token_456"

    # Default fallback verification
    res3 = await test_client.get("/api/v1/webhooks/tiktok")
    assert res3.status_code == 200
    assert res3.text == "tiktok_challenge_ok"


@pytest.mark.asyncio
async def test_tiktok_webhook_inbound_dm(test_client: AsyncClient, db_session: AsyncSession):
    # Inbound direct message
    res = await test_client.post(
        "/api/v1/webhooks/tiktok",
        json={
            "event": "im.direct_message.receive",
            "content": {
                "open_id": "tt_inbound_user_7788",
                "text": "Hey! Can I customize the enterprise subscription?",
                "message_id": "tt_msg_inbound_9900",
            },
            "user": {
                "nickname": "Elena Rostova",
                "open_id": "tt_inbound_user_7788",
            },
        },
    )
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert data["openId"] == "tt_inbound_user_7788"
    assert data["isComment"] is False

    # Verify contact was auto-provisioned
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.TIKTOK, Contact.value == "tt_inbound_user_7788"
    )
    contact = (await db_session.execute(contact_stmt)).scalar_one_or_none()
    assert contact is not None
    assert contact.person_name == "Elena Rostova"

    # Verify conversation and message
    conv_stmt = select(Conversation).where(Conversation.contact_id == contact.id)
    conv = (await db_session.execute(conv_stmt)).scalar_one_or_none()
    assert conv is not None

    msg_stmt = select(Message).where(Message.provider_message_id == "tt_msg_inbound_9900")
    msg = (await db_session.execute(msg_stmt)).scalar_one_or_none()
    assert msg is not None
    assert msg.body == "Hey! Can I customize the enterprise subscription?"
    assert msg.direction == MessageDirection.INBOUND
    assert msg.channel == ChannelType.TIKTOK


@pytest.mark.asyncio
async def test_tiktok_webhook_inbound_comment(test_client: AsyncClient, db_session: AsyncSession):
    # Inbound video comment
    res = await test_client.post(
        "/api/v1/webhooks/tiktok",
        json={
            "event": "video.comment.receive",
            "content": {
                "video_id": "v_growth_hack_500",
                "comment_id": "c_cmt_inbound_1234",
                "open_id": "tt_commenter_4433",
                "text": "Awesome demonstration video!",
            },
            "user": {
                "nickname": "Zach Morris",
                "open_id": "tt_commenter_4433",
            },
        },
    )
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert data["isComment"] is True

    # Verify message recorded with video comment prefix
    msg_stmt = select(Message).where(Message.provider_message_id == "c_cmt_inbound_1234")
    msg = (await db_session.execute(msg_stmt)).scalar_one_or_none()
    assert msg is not None
    assert "[TikTok Video Comment #v_growth_hack_500]: Awesome demonstration video!" in msg.body
    assert msg.direction == MessageDirection.INBOUND
