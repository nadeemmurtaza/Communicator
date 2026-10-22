import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType
from communicator.models.message import Message, MessageDirection


@pytest.mark.asyncio
async def test_live_chat_channel_integration(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch live chat channel config
    res = await test_client.get("/api/v1/inboxes/channels/live-chat")
    assert res.status_code == 200
    data = res.json()
    assert data["inbox"]["channel_type"] == "website"
    assert "websiteToken" in data
    assert "embedScript" in data
    assert data["branding"]["widgetColor"] == "#1570EF"
    assert data["preChatForm"]["enabled"] is True
    assert len(data["preChatForm"]["fields"]) >= 3

    original_token = data["websiteToken"]

    # 2. Update live chat settings
    update_res = await test_client.post(
        "/api/v1/inboxes/channels/live-chat",
        json={
            "name": "Acme Live Chat",
            "website_domain": "example.org",
            "widget_color": "#7F56D9",
            "greeting_title": "Hello from Support! 👋",
            "greeting_tagline": "Online now.",
            "widget_position": "left",
            "greeting_enabled": True,
            "greeting_message": "Welcome to Acme!",
            "out_of_office_enabled": True,
            "out_of_office_message": "We are currently offline.",
            "csat_enabled": True,
            "pre_chat_form_enabled": True,
            "pre_chat_fields": [
                {"name": "fullName", "label": "Full Name", "required": True},
                {"name": "email", "label": "Email Address", "required": True},
            ],
            "identity_validation": True,
            "hmac_secret": "whsec_customsecret123456",
        },
    )
    assert update_res.status_code == 200
    updated_data = update_res.json()
    assert updated_data["inbox"]["name"] == "Acme Live Chat"
    assert updated_data["branding"]["widgetColor"] == "#7F56D9"
    assert updated_data["branding"]["greetingTitle"] == "Hello from Support! 👋"
    assert updated_data["branding"]["widgetPosition"] == "left"
    assert updated_data["security"]["identityValidation"] is True

    # 3. Regenerate widget token
    regen_res = await test_client.post("/api/v1/inboxes/channels/live-chat/regenerate-token")
    assert regen_res.status_code == 200
    regen_data = regen_res.json()
    new_token = regen_data["websiteToken"]
    assert new_token != original_token
    assert new_token in regen_data["embedScript"]

    # 4. Trigger test visitor conversation
    test_vis_res = await test_client.post(
        "/api/v1/inboxes/channels/live-chat/test-visitor",
        json={
            "name": "Sarah Connor",
            "email": "sarah.connor@cyberdyne.org",
            "phone": "+1 (555) 000-1111",
            "company": "Resistance HQ",
            "initial_message": "Need info about your AI defensive products.",
        },
    )
    assert test_vis_res.status_code == 200
    conv_data = test_vis_res.json()
    assert conv_data["status"] == "created"
    assert "conversationId" in conv_data

    # 5. List live chat events
    events_res = await test_client.get("/api/v1/inboxes/channels/live-chat/events")
    assert events_res.status_code == 200
    events = events_res.json()
    assert len(events) >= 1
    assert any("Need info" in e["body"] for e in events)


@pytest.mark.asyncio
async def test_public_widget_flow(test_client: AsyncClient, db_session: AsyncSession):
    # Fetch live chat channel to get valid token
    res = await test_client.get("/api/v1/inboxes/channels/live-chat")
    assert res.status_code == 200
    token = res.json()["websiteToken"]

    # 1. Public config fetch
    cfg_res = await test_client.get(f"/api/v1/widget/config?token={token}")
    assert cfg_res.status_code == 200
    cfg = cfg_res.json()
    assert cfg["websiteToken"] == token
    assert "widgetColor" in cfg

    # 2. Register visitor contact
    contact_res = await test_client.post(
        "/api/v1/widget/contact",
        json={
            "website_token": token,
            "email": "visitor99@webtest.com",
            "name": "Jane Visitor",
            "phone": "+1 (555) 333-4444",
            "custom_attributes": {"referrer": "google.com", "plan": "pro"},
        },
    )
    assert contact_res.status_code == 200
    visitor = contact_res.json()
    conv_id = visitor["conversationId"]
    assert conv_id

    # 3. Post inbound message from visitor
    msg_res = await test_client.post(
        "/api/v1/widget/messages",
        json={"conversation_id": conv_id, "body": "Hello, I am asking via widget!"},
    )
    assert msg_res.status_code == 200
    msg = msg_res.json()
    assert msg["body"] == "Hello, I am asking via widget!"

    # 4. Fetch visitor messages (test private notes exclusion)
    # Add a private internal agent note directly to conversation
    private_note = Message(
        conversation_id=conv_id,
        direction=MessageDirection.OUTBOUND,
        channel=ChannelType.WEBSITE,
        body="INTERNAL ONLY: Do not reveal discount code 50OFF.",
        provider="web_widget",
        provider_message_id="priv_note_1",
        is_private=True,
    )
    db_session.add(private_note)
    await db_session.flush()

    messages_res = await test_client.get(f"/api/v1/widget/conversations/{conv_id}/messages")
    assert messages_res.status_code == 200
    messages = messages_res.json()
    # Visitor should see their message
    assert any(m["body"] == "Hello, I am asking via widget!" for m in messages)
    # Visitor must NEVER see private internal note!
    assert all("INTERNAL ONLY" not in m["body"] for m in messages)

    # 5. Heartbeat presence
    presence_res = await test_client.post(
        "/api/v1/widget/presence",
        json={
            "conversation_id": conv_id,
            "status": "online",
            "page_url": "https://example.org/pricing",
        },
    )
    assert presence_res.status_code == 200
    assert presence_res.json()["status"] == "ok"

    # 6. Agent in Shared Inbox replies to visitor
    agent_reply_res = await test_client.post(
        f"/api/v1/conversations/{conv_id}/messages",
        json={
            "body": "Hello Jane, thanks for reaching out! How can our support team assist you today?",
            "sender_name": "Riley Brooks",
            "is_private": False,
        },
    )
    assert agent_reply_res.status_code == 200
    agent_msg = agent_reply_res.json()
    assert (
        agent_msg["body"]
        == "Hello Jane, thanks for reaching out! How can our support team assist you today?"
    )
    assert agent_msg["isPrivate"] is False

    # 7. Visitor fetches updated chat history to receive agent reply
    updated_messages_res = await test_client.get(f"/api/v1/widget/conversations/{conv_id}/messages")
    assert updated_messages_res.status_code == 200
    updated_messages = updated_messages_res.json()
    assert any("Hello Jane, thanks for reaching out!" in m["body"] for m in updated_messages)


@pytest.mark.asyncio
async def test_live_chat_hmac_identity_verification():
    from communicator.channels.live_chat import WebLiveChatProvider

    provider = WebLiveChatProvider(hmac_secret="secret_signing_key_999")
    email = "customer@example.com"
    token = provider.generate_identity_hash(email)
    assert token
    assert isinstance(token, str)
    assert len(token) == 64  # sha256 hex length

    # Verify matching hash returns True
    assert provider.verify_identity_hash(email, token) is True

    # Verify altered hash returns False
    assert provider.verify_identity_hash(email, "invalid_forged_hash") is False
    assert provider.verify_identity_hash("impostor@example.com", token) is False
