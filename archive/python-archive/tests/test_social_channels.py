import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.message import Message, MessageDirection


@pytest.mark.asyncio
async def test_facebook_channel_flow(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch Facebook channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/facebook")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "facebook"
    assert "pageProfile" in data
    assert data["pageProfile"]["pageId"] == "100234567890123"
    assert len(data["icebreakers"]) >= 2
    assert "webhookConfig" in data
    assert data["health"]["isConnected"] is True

    # 2. Update Facebook settings
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/facebook",
        json={
            "name": "Bay Works Global Support",
            "page_id": "100234567890123",
            "page_name": "Bay Works Enterprise Page",
            "page_access_token": "EAAB_test_mock_token_facebook",
            "greeting_enabled": True,
            "greeting_message": "Hello from Bay Works Messenger! We are ready to help.",
            "out_of_office_enabled": False,
            "csat_enabled": True,
            "icebreakers": [
                {"question": "How can I upgrade?", "payload": "UPGRADE"},
                {"question": "Pricing tiers", "payload": "PRICING"},
            ],
            "timezone": "America/New_York",
        },
    )
    assert update_res.status_code == 200
    updated = update_res.json()
    assert updated["inbox"]["name"] == "Bay Works Global Support"
    assert updated["pageProfile"]["pageName"] == "Bay Works Enterprise Page"
    assert len(updated["icebreakers"]) == 2

    # 3. Verify Facebook connection
    verify_res = await test_client.post("/api/v1/inboxes/channels/facebook/verify")
    assert verify_res.status_code == 200
    assert verify_res.json()["verified"] is True

    # 4. Dispatch Facebook diagnostic test message
    test_msg_res = await test_client.post(
        "/api/v1/inboxes/channels/facebook/test-message",
        json={
            "recipient_psid": "psid_test_902184",
            "recipient_name": "Sarah Jenkins",
            "message": "Testing automated messenger dispatch from Bay Works Support.",
        },
    )
    assert test_msg_res.status_code == 200
    msg_data = test_msg_res.json()
    assert msg_data["status"] == "sent"
    assert "mid" in msg_data
    assert msg_data["recipient"] == "psid_test_902184"

    # 5. List Facebook events
    events_res = await test_client.get("/api/v1/inboxes/channels/facebook/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 1
    assert any("Testing automated messenger" in e["body"] for e in events)

    # 6. Webhook handshake verification
    handshake_res = await test_client.get(
        "/api/v1/webhooks/meta/facebook",
        params={
            "hub.mode": "subscribe",
            "hub.verify_token": "meta_facebook_verify_token",
            "hub.challenge": "fb_challenge_998877",
        },
    )
    assert handshake_res.status_code == 200
    assert handshake_res.text == "fb_challenge_998877"

    # 7. Webhook inbound message processing
    webhook_inbound_res = await test_client.post(
        "/api/v1/webhooks/meta/facebook",
        json={
            "object": "page",
            "entry": [
                {
                    "id": "100234567890123",
                    "time": 1711000000000,
                    "messaging": [
                        {
                            "sender": {"id": "psid_inbound_cust_42"},
                            "recipient": {"id": "100234567890123"},
                            "timestamp": 1711000000000,
                            "message": {
                                "mid": "mid.fb_inbound_msg_001",
                                "text": "Hi! Can you provide your enterprise SLAs?",
                            },
                        }
                    ],
                }
            ],
        },
    )
    assert webhook_inbound_res.status_code == 200
    assert webhook_inbound_res.json()["status"] == "ok"

    # Verify contact was auto-provisioned and message recorded
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.FACEBOOK, Contact.value == "psid_inbound_cust_42"
    )
    contact = (await db_session.execute(contact_stmt)).scalar_one_or_none()
    assert contact is not None

    conv_stmt = select(Conversation).where(Conversation.contact_id == contact.id)
    conv = (await db_session.execute(conv_stmt)).scalar_one_or_none()
    assert conv is not None

    msg_stmt = select(Message).where(Message.provider_message_id == "mid.fb_inbound_msg_001")
    msg = (await db_session.execute(msg_stmt)).scalar_one_or_none()
    assert msg is not None
    assert msg.body == "Hi! Can you provide your enterprise SLAs?"
    assert msg.direction == MessageDirection.INBOUND


@pytest.mark.asyncio
async def test_instagram_channel_flow(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch Instagram channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/instagram")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "instagram"
    assert "accountProfile" in data
    assert data["accountProfile"]["igUsername"] == "@bayworks"
    assert data["storyMentions"]["enabled"] is True
    assert "webhookConfig" in data
    assert data["health"]["isConnected"] is True

    # 2. Update Instagram settings
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/instagram",
        json={
            "name": "Bay Works Instagram Support",
            "ig_username": "@bayworks_official",
            "ig_id": "17841405822345678",
            "connected_page_id": "100234567890123",
            "access_token": "EAAB_test_mock_token_instagram",
            "story_mentions_enabled": True,
            "story_reply_message": "Thanks for sharing us in your story! Drop us a DM anytime. ✨",
            "greeting_enabled": True,
            "greeting_message": "Hey! Welcome to Bay Works on Instagram.",
            "out_of_office_enabled": False,
            "csat_enabled": True,
            "icebreakers": [
                {"question": "Product Catalog", "payload": "CATALOG"},
                {"question": "Partner Inquiry", "payload": "PARTNER"},
            ],
            "timezone": "UTC",
        },
    )
    assert update_res.status_code == 200
    updated = update_res.json()
    assert updated["inbox"]["name"] == "Bay Works Instagram Support"
    assert updated["accountProfile"]["igUsername"] == "@bayworks_official"
    assert "Drop us a DM anytime" in updated["storyMentions"]["replyMessage"]

    # 3. Verify Instagram connection
    verify_res = await test_client.post("/api/v1/inboxes/channels/instagram/verify")
    assert verify_res.status_code == 200
    assert verify_res.json()["verified"] is True

    # 4. Dispatch Instagram diagnostic test Direct Message
    test_msg_res = await test_client.post(
        "/api/v1/inboxes/channels/instagram/test-message",
        json={
            "recipient_igsid": "igsid_test_549120",
            "recipient_username": "clara_design",
            "message": "Testing automated Instagram DM dispatch from Bay Works.",
        },
    )
    assert test_msg_res.status_code == 200
    msg_data = test_msg_res.json()
    assert msg_data["status"] == "sent"
    assert "mid" in msg_data
    assert msg_data["recipient"] == "igsid_test_549120"

    # 5. List Instagram events
    events_res = await test_client.get("/api/v1/inboxes/channels/instagram/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 1
    assert any("Testing automated Instagram DM" in e["body"] for e in events)

    # 6. Webhook handshake verification
    handshake_res = await test_client.get(
        "/api/v1/webhooks/meta/instagram",
        params={
            "hub.mode": "subscribe",
            "hub.verify_token": "meta_instagram_verify_token",
            "hub.challenge": "ig_challenge_332211",
        },
    )
    assert handshake_res.status_code == 200
    assert handshake_res.text == "ig_challenge_332211"

    # 7. Webhook inbound message processing
    webhook_inbound_res = await test_client.post(
        "/api/v1/webhooks/meta/instagram",
        json={
            "object": "instagram",
            "entry": [
                {
                    "id": "17841405822345678",
                    "time": 1711000000000,
                    "messaging": [
                        {
                            "sender": {"id": "igsid_inbound_fan_88"},
                            "recipient": {"id": "17841405822345678"},
                            "timestamp": 1711000000000,
                            "message": {
                                "mid": "mid.ig_inbound_dm_999",
                                "text": "Hey! Saw your recent demo video, do you offer free trials?",
                            },
                        }
                    ],
                }
            ],
        },
    )
    assert webhook_inbound_res.status_code == 200
    assert webhook_inbound_res.json()["status"] == "ok"

    # Verify contact was auto-provisioned and message recorded
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.INSTAGRAM, Contact.value == "igsid_inbound_fan_88"
    )
    contact = (await db_session.execute(contact_stmt)).scalar_one_or_none()
    assert contact is not None

    conv_stmt = select(Conversation).where(Conversation.contact_id == contact.id)
    conv = (await db_session.execute(conv_stmt)).scalar_one_or_none()
    assert conv is not None

    msg_stmt = select(Message).where(Message.provider_message_id == "mid.ig_inbound_dm_999")
    msg = (await db_session.execute(msg_stmt)).scalar_one_or_none()
    assert msg is not None
    assert msg.body == "Hey! Saw your recent demo video, do you offer free trials?"
    assert msg.direction == MessageDirection.INBOUND
