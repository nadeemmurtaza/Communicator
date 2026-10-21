import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.inbox import Inbox
from communicator.models.message import Message


@pytest.mark.asyncio
async def test_voice_channel_config_and_webrtc_token(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Verify Voice inbox is listed in /inboxes
    res = await test_client.get("/api/v1/inboxes")
    assert res.status_code == 200
    inboxes = res.json()
    voice_inbox = next((i for i in inboxes if i["channel_type"] == "voice"), None)
    assert voice_inbox is not None
    assert "Voice" in voice_inbox["name"]

    # 2. Get Voice channel configuration
    res = await test_client.get("/api/v1/inboxes/channels/voice")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "voice"
    assert "sipConfig" in data
    assert "webrtcConfig" in data
    assert "ivrConfig" in data
    assert data["ivrConfig"]["enabled"] is True
    assert len(data["ivrConfig"]["menu"]) >= 3

    # 3. Update Voice channel settings
    res = await test_client.post(
        "/api/v1/inboxes/channels/voice",
        json={
            "name": "Bay Works Live Voice Hotline",
            "voice_number": "+14155550299",
            "caller_id": "+14155550299",
            "ivr_enabled": True,
            "ivr_prompt": "Welcome to Bay Works Hotline. Press 1 for Sales, 2 for Support, 3 for Voicemail.",
            "recording_enabled": True,
            "recording_channels": "dual",
            "transcription_enabled": True,
            "webrtc_enabled": True,
        },
    )
    assert res.status_code == 200
    updated = res.json()
    assert updated["inbox"]["name"] == "Bay Works Live Voice Hotline"
    assert updated["inbox"]["phone_number"] == "+14155550299"
    assert "Welcome to Bay Works Hotline" in updated["ivrConfig"]["prompt"]

    # 4. Verify Voice credentials
    res = await test_client.post("/api/v1/inboxes/channels/voice/verify")
    assert res.status_code == 200
    verify_data = res.json()
    assert verify_data["status"] == "ok"
    assert verify_data["verified"] is True

    # 5. Generate WebRTC capability token for browser softphone
    res = await test_client.post(
        "/api/v1/inboxes/channels/voice/token",
        json={"identity": "agent-sarah"},
    )
    assert res.status_code == 200
    token_data = res.json()
    assert token_data["identity"] == "agent-sarah"
    assert token_data["token"].startswith("vctok_")
    assert "sipDomain" in token_data


@pytest.mark.asyncio
async def test_voice_inbound_call_and_ivr_flow(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Incoming call with no digits (initial ring) -> Returns Gather IVR TwiML
    res = await test_client.post(
        "/api/v1/webhooks/voice/inbound",
        data={
            "CallSid": "CA_test_ivr_12345",
            "From": "+14155559988",
            "To": "+14155550199",
            "CallStatus": "ringing",
            "Direction": "inbound",
        },
    )
    assert res.status_code == 200
    assert "text/xml" in res.headers.get("content-type", "") or "application/xml" in res.headers.get("content-type", "")
    xml_content = res.text
    assert "<Response>" in xml_content
    assert "<Gather" in xml_content
    assert "Thank you for calling" in xml_content

    # Verify conversation & message were auto-provisioned
    msg_stmt = select(Message).where(Message.provider_message_id == "CA_test_ivr_12345")
    msg = (await db_session.execute(msg_stmt)).scalars().first()
    assert msg is not None
    assert msg.channel == ChannelType.VOICE
    assert "+14155559988" in msg.body

    # 2. Caller presses '1' (Sales) -> Connects to sales queue
    res_sales = await test_client.post(
        "/api/v1/webhooks/voice/inbound",
        data={
            "CallSid": "CA_test_ivr_12345",
            "From": "+14155559988",
            "To": "+14155550199",
            "Digits": "1",
        },
    )
    assert res_sales.status_code == 200
    assert "sales-queue" in res_sales.text

    # 3. Caller presses '2' (Technical Support) -> Connects to support queue
    res_support = await test_client.post(
        "/api/v1/webhooks/voice/inbound",
        data={
            "CallSid": "CA_test_ivr_12345",
            "From": "+14155559988",
            "To": "+14155550199",
            "Digits": "2",
        },
    )
    assert res_support.status_code == 200
    assert "support-queue" in res_support.text

    # 4. Caller presses '3' (Voicemail) -> Drops into recording
    res_voicemail = await test_client.post(
        "/api/v1/webhooks/voice/inbound",
        data={
            "CallSid": "CA_test_ivr_12345",
            "From": "+14155559988",
            "To": "+14155550199",
            "Digits": "3",
        },
    )
    assert res_voicemail.status_code == 200
    assert "<Record" in res_voicemail.text


@pytest.mark.asyncio
async def test_voice_status_and_recording_callbacks(test_client: AsyncClient, db_session: AsyncSession):
    call_sid = "CA_rec_callback_789"
    # Seed incoming call first
    await test_client.post(
        "/api/v1/webhooks/voice/inbound",
        data={
            "CallSid": call_sid,
            "From": "+14155557766",
            "To": "+14155550199",
            "Digits": "1",
        },
    )

    # 1. Status callback: call completed after 145 seconds
    res_status = await test_client.post(
        "/api/v1/webhooks/voice/status",
        data={
            "CallSid": call_sid,
            "CallStatus": "completed",
            "CallDuration": "145",
        },
    )
    assert res_status.status_code == 200
    assert res_status.json()["status"] == "ok"
    assert res_status.json()["duration"] == 145

    # 2. Recording callback: dual-channel audio link
    rec_url = "https://api.twilio.com/2010-04-01/Accounts/AC123/Recordings/RE998877.mp3"
    res_rec = await test_client.post(
        "/api/v1/webhooks/voice/recording",
        data={
            "CallSid": call_sid,
            "RecordingUrl": rec_url,
            "RecordingDuration": "145",
            "RecordingSid": "RE998877",
        },
    )
    assert res_rec.status_code == 200
    assert res_rec.json()["recording_url"] == rec_url

    # 3. Transcription callback: speech-to-text text
    transcript = "Hello, I am calling to confirm our enterprise subscription renewal."
    res_trans = await test_client.post(
        "/api/v1/webhooks/voice/transcription",
        data={
            "CallSid": call_sid,
            "TranscriptionText": transcript,
            "TranscriptionStatus": "completed",
        },
    )
    assert res_trans.status_code == 200
    assert res_trans.json()["status"] == "ok"

    # Verify message metadata contains status, duration, recording URL, and transcript
    msg_stmt = select(Message).where(Message.provider_message_id == call_sid)
    msg = (await db_session.execute(msg_stmt)).scalars().first()
    assert msg is not None
    assert msg.provider_metadata["duration"] == 145
    assert msg.provider_metadata["recording_url"] == rec_url
    assert msg.provider_metadata["transcription_text"] == transcript
    assert "Transcript:" in msg.body


@pytest.mark.asyncio
async def test_voice_outbound_call_and_shared_inbox(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Outbound call via dialpad
    res_call = await test_client.post(
        "/api/v1/inboxes/channels/voice/call",
        json={
            "recipient_phone": "+14155553322",
            "agent_name": "Riley Brooks",
            "message": "Calling to follow up on your support ticket #8291.",
        },
    )
    assert res_call.status_code == 200
    call_data = res_call.json()
    assert call_data["status"] == "ok"
    assert call_data["callSid"].startswith("CA")
    conv_id = call_data["conversationId"]

    # 2. Hangup call
    res_hangup = await test_client.post(
        "/api/v1/inboxes/channels/voice/hangup",
        json={"call_sid": call_data["callSid"]},
    )
    assert res_hangup.status_code == 200
    assert res_hangup.json()["status"] == "ok"

    # 3. Retrieve call logs
    res_logs = await test_client.get("/api/v1/inboxes/channels/voice/call-logs")
    assert res_logs.status_code == 200
    logs = res_logs.json()
    assert len(logs) >= 1
    assert any(l["callSid"] == call_data["callSid"] for l in logs)

    # 4. Agent sends follow-up message in the Voice conversation thread
    res_reply = await test_client.post(
        f"/api/v1/conversations/{conv_id}/messages",
        json={
            "body": "Follow-up note left after voice call. Customer agreed to Monday review.",
            "sender_name": "Riley Brooks",
        },
    )
    assert res_reply.status_code == 200
    reply_data = res_reply.json()
    assert reply_data["status"] in ("queued", "sent", "delivered")

    # 5. Retrieve voice events
    res_events = await test_client.get("/api/v1/inboxes/channels/voice/events")
    assert res_events.status_code == 200
    events = res_events.json()
    assert len(events) >= 1
