import uuid

import httpx
import pytest

from communicator.channels.email import PostmarkEmailProvider
from communicator.channels.sms import TwilioSMSProvider
from communicator.models.contact import ChannelType, Contact


def _patch_async_client(monkeypatch, handler):
    """Redirect httpx.AsyncClient's transport to a MockTransport, capturing the request."""
    transport = httpx.MockTransport(handler)
    real_client = httpx.AsyncClient

    class PatchedClient(real_client):
        def __init__(self, *args, **kwargs):
            kwargs["transport"] = transport
            super().__init__(*args, **kwargs)

    monkeypatch.setattr(httpx, "AsyncClient", PatchedClient)


def _email_contact() -> Contact:
    return Contact(id=uuid.uuid4(), lead_id=uuid.uuid4(), channel=ChannelType.EMAIL, value="lead@example.com")


def _sms_contact() -> Contact:
    return Contact(id=uuid.uuid4(), lead_id=uuid.uuid4(), channel=ChannelType.SMS, value="+15551234567")


@pytest.mark.asyncio
async def test_postmark_provider_sends_and_returns_message_id(monkeypatch):
    captured = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured["request"] = request
        return httpx.Response(200, json={"MessageID": "pm-abc-123", "ErrorCode": 0})

    _patch_async_client(monkeypatch, handler)

    provider = PostmarkEmailProvider(api_key="test-key", from_address="sales@newax.example")
    result = await provider.send(_email_contact(), "Hello there", subject="Quick question")

    assert result.provider == "postmark"
    assert result.provider_message_id == "pm-abc-123"
    assert captured["request"].headers["X-Postmark-Server-Token"] == "test-key"
    assert captured["request"].url.path == "/email"


@pytest.mark.asyncio
async def test_postmark_provider_requires_credentials():
    with pytest.raises(RuntimeError):
        PostmarkEmailProvider(api_key="", from_address="")


@pytest.mark.asyncio
async def test_postmark_provider_raises_on_http_error(monkeypatch):
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(422, json={"ErrorCode": 300, "Message": "Invalid email"})

    _patch_async_client(monkeypatch, handler)

    provider = PostmarkEmailProvider(api_key="test-key", from_address="sales@newax.example")
    with pytest.raises(httpx.HTTPStatusError):
        await provider.send(_email_contact(), "Hello there")


@pytest.mark.asyncio
async def test_twilio_provider_sends_and_returns_sid(monkeypatch):
    captured = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured["request"] = request
        return httpx.Response(201, json={"sid": "SMxxxxx", "status": "queued"})

    _patch_async_client(monkeypatch, handler)

    provider = TwilioSMSProvider(account_sid="ACxxx", auth_token="secret", from_number="+15559876543")
    result = await provider.send(_sms_contact(), "Hi! Quick follow-up on your inquiry.")

    assert result.provider == "twilio"
    assert result.provider_message_id == "SMxxxxx"
    assert captured["request"].url.path == "/2010-04-01/Accounts/ACxxx/Messages.json"


@pytest.mark.asyncio
async def test_twilio_provider_requires_credentials():
    with pytest.raises(RuntimeError):
        TwilioSMSProvider(account_sid="", auth_token="", from_number="")
