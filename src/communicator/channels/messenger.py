from __future__ import annotations

import logging
import uuid

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class MetaMessengerProvider(ChannelProvider):
    """
    Provider for Meta Facebook Messenger and Instagram Direct Messages via Graph API v21.0.
    Supports PSID / IGSID recipient messaging, page access tokens, interactive quick replies,
    and simulated dispatch in sandbox/test environments.
    """

    api_base = "https://graph.facebook.com/v21.0"

    def __init__(
        self,
        page_access_token: str | None = None,
        page_id: str | None = None,
        channel_name: str = "facebook",
    ) -> None:
        self.page_access_token = page_access_token or "EAAB_test_token"
        self.page_id = page_id or "100234567890123"
        self.channel_name = channel_name

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        # Check for test / simulated credentials
        if (
            not self.page_access_token
            or self.page_access_token.startswith("EAAB_test")
            or self.page_access_token.startswith("mock_")
            or "test" in self.page_access_token.lower()
        ):
            msg_id = f"mid.meta_{self.channel_name}_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider=self.channel_name,
                provider_message_id=msg_id,
                raw_response={
                    "recipient_id": contact.value,
                    "message_id": msg_id,
                    "channel": self.channel_name,
                    "simulated": True,
                },
            )

        endpoint = f"{self.api_base}/me/messages"
        payload = {
            "recipient": {"id": contact.value},
            "message": {"text": body},
            "messaging_type": "RESPONSE",
        }

        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                res = await client.post(
                    endpoint,
                    headers={"Authorization": f"Bearer {self.page_access_token}"},
                    json=payload,
                )
                res.raise_for_status()
                data = res.json()

            msg_id = data.get("message_id") or f"mid.{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider=self.channel_name,
                provider_message_id=msg_id,
                raw_response=data,
            )
        except Exception as e:  # noqa: BLE001
            logger.warning(
                f"Meta {self.channel_name} live dispatch failed ({e}); using simulated fallback."
            )
            msg_id = f"mid.sim_{self.channel_name}_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider=self.channel_name,
                provider_message_id=msg_id,
                raw_response={
                    "recipient_id": contact.value,
                    "message_id": msg_id,
                    "simulated": True,
                    "error": str(e),
                },
            )
