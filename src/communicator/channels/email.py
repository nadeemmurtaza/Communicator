from __future__ import annotations

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.config import settings
from communicator.models.contact import Contact


class PostmarkEmailProvider(ChannelProvider):
    """Email via Postmark's REST API (the provider docs/ARCHITECTURE.md recommends)."""

    api_base = "https://api.postmarkapp.com"

    def __init__(self, api_key: str | None = None, from_address: str | None = None) -> None:
        self.api_key = api_key if api_key is not None else settings.email_provider_api_key
        self.from_address = (
            from_address if from_address is not None else settings.email_from_address
        )
        if not self.api_key or not self.from_address:
            raise RuntimeError(
                "EMAIL_PROVIDER_API_KEY and EMAIL_FROM_ADDRESS must be set to send email."
            )

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        payload = {
            "From": self.from_address,
            "To": contact.value,
            "Subject": subject or "",
            "TextBody": body,
            "MessageStream": "outbound",
        }
        headers = {"X-Postmark-Server-Token": self.api_key, "Accept": "application/json"}

        async with httpx.AsyncClient(
            base_url=self.api_base, headers=headers, timeout=30.0
        ) as client:
            response = await client.post("/email", json=payload)
            response.raise_for_status()
            data = response.json()

        return SendResult(
            provider="postmark", provider_message_id=data.get("MessageID", ""), raw_response=data
        )
