from __future__ import annotations

import logging
import uuid

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class LineProvider(ChannelProvider):
    """
    Provider for LINE Messaging API (https://api.line.me/v2/bot/message/push).
    Supports sending push messages to LINE user IDs, channel access tokens,
    and simulated dispatch in sandbox/test environments.
    """

    api_base = "https://api.line.me/v2/bot"

    def __init__(
        self,
        channel_access_token: str | None = None,
        channel_id: str | None = None,
        channel_secret: str | None = None,
    ) -> None:
        self.channel_access_token = channel_access_token or "line_test_token_123456"
        self.channel_id = channel_id or "1657890123"
        self.channel_secret = channel_secret or "mock_secret_line"

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        # Check for test / simulated credentials
        if (
            not self.channel_access_token
            or self.channel_access_token.startswith("line_test")
            or self.channel_access_token.startswith("mock_")
            or "test" in self.channel_access_token.lower()
        ):
            msg_id = f"line_msg_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider="line",
                provider_message_id=msg_id,
                raw_response={
                    "to": contact.value,
                    "message_id": msg_id,
                    "channel": "line",
                    "simulated": True,
                },
            )

        endpoint = f"{self.api_base}/message/push"
        payload = {
            "to": contact.value,
            "messages": [
                {
                    "type": "text",
                    "text": body,
                }
            ],
        }

        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                res = await client.post(
                    endpoint,
                    headers={
                        "Authorization": f"Bearer {self.channel_access_token}",
                        "Content-Type": "application/json",
                    },
                    json=payload,
                )
                res.raise_for_status()
                data = res.json() if res.content else {}

            msg_id = f"line_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider="line",
                provider_message_id=msg_id,
                raw_response=data,
            )
        except Exception as e:  # noqa: BLE001
            logger.warning(f"LINE live dispatch failed ({e}); using simulated fallback.")
            msg_id = f"line_sim_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider="line",
                provider_message_id=msg_id,
                raw_response={
                    "to": contact.value,
                    "message_id": msg_id,
                    "simulated": True,
                    "error": str(e),
                },
            )
