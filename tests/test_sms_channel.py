import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.message import Message, MessageDirection
from communicator.config import settings


@pytest.mark.asyncio
async def test_sms_channel_integration(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch SMS channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/sms")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "sms"
    assert "webhookConfig" in data
    assert "compliance10dlc" in data
    assert data["compliance10dlc"]["status"] in ("VERIFIED", "REGISTERED")
    assert "optOutKeywords" in data["compliance10dlc"]
    assert "STOP" in data["compliance10dlc"]["optOutKeywords"]

    # 2. Update SMS channel configuration
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/sms",
        json={
            "name": "Bay Works SMS Center",
            "account_sid": "AC_test_account_sid_12345",
            "auth_token": "auth_test_token_secret_67890",
            "phone_number": "+14155559988",
            "messaging_service_sid": "MG_test_service_sid_345",
            "a2p_10dlc_status": "VERIFIED",
            "brand_name": "Bay Works LLC",
            "brand_tier": "Standard High Throughput (10DLC)",
            "opt_out_message": "You have unsubscribed. Reply START to resubscribe.",
            "greeting_enabled": True,
            "greeting_message": "Welcome to Bay Works SMS support!",
            "out_of_office_enabled": True,
            "out_of_office_message": "Our agents are currently offline.",
            "csat_enabled": True,
            "working_hours_enabled": True,
            "working_hours_start": 8,
            "working_hours_end": 19,
            "timezone": "America/New_York",
        },
    )
    assert update_res.status_code == 200
    updated_data = update_res.json()
    assert updated_data["inbox"]["name"] == "Bay Works SMS Center"
    assert updated_data["compliance10dlc"]["brandName"] == "Bay Works LLC"
    assert updated_data["inbox"]["credentials"]["fromNumber"] == "+14155559988"

    # 3. Verify Twilio connection
    verify_res = await test_client.post("/api/v1/inboxes/channels/sms/verify")
    assert verify_res.status_code == 200
    verify_data = verify_res.json()
    assert verify_data["status"] == "ok"
    assert verify_data["verified"] is True
    assert "AC_test" in verify_data["accountSid"]

    # 4. Dispatch diagnostic test SMS
    test_sms_res = await test_client.post(
        "/api/v1/inboxes/channels/sms/test-message",
        json={
            "recipient_phone": "+14155550199",
            "message": "Twilio 10DLC verification test message",
        },
    )
    assert test_sms_res.status_code == 200
    sms_res = test_sms_res.json()
    assert sms_res["status"] == "sent"
    assert "sid" in sms_res
    assert sms_res["sid"].startswith("SM")

    # 5. List SMS events
    events_res = await test_client.get("/api/v1/inboxes/channels/sms/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 1
    assert any("verification test" in e["body"] for e in events)


@pytest.mark.asyncio
async def test_sms_outbound_reply_from_shared_inbox(test_client: AsyncClient, db_session: AsyncSession):
    # Send a diagnostic test message to seed contact and conversation
    res = await test_client.post(
        "/api/v1/inboxes/channels/sms/test-message",
        json={
            "recipient_phone": "+14155550288",
            "message": "Initial test inquiry",
        },
    )
    assert res.status_code == 200

    # Retrieve conversation for this contact
    convs_res = await test_client.get("/api/v1/conversations")
    assert convs_res.status_code == 200
    convs = convs_res.json()
    sms_conv = next(c for c in convs if c["contactPhone"] == "+14155550288")
    conv_id = sms_conv["id"]

    # 1. Agent sends an internal team note (private)
    note_res = await test_client.post(
        f"/api/v1/conversations/{conv_id}/messages",
        json={
            "body": "INTERNAL NOTE: Lead requested SMS quote details.",
            "is_private": True,
            "sender_name": "Riley Brooks",
        },
    )
    assert note_res.status_code == 200
    note_data = note_res.json()
    assert note_data["isPrivate"] is True

    # 2. Agent sends an outbound SMS reply to customer
    reply_res = await test_client.post(
        f"/api/v1/conversations/{conv_id}/messages",
        json={
            "body": "Hi there! Our enterprise pricing starts at $49/mo. Reply 1 for details.",
            "is_private": False,
            "sender_name": "Riley Brooks",
        },
    )
    assert reply_res.status_code == 200
    reply_data = reply_res.json()
    assert reply_data["isPrivate"] is False
    assert "enterprise pricing" in reply_data["body"]
