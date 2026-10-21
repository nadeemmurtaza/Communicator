from __future__ import annotations

import logging
import uuid
from typing import Any
import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class TelegramProvider(ChannelProvider):
    """
    Provider for Telegram Bot API (https://api.telegram.org).
    Supports sending text messages to Telegram chat IDs, custom bot tokens,
    and simulated dispatch in sandbox/test environments.
    """

    api_base = "https://api.telegram.org"

    def __init__(
        self,
        bot_token: str | None = None,
        bot_username: str | None = None,
    ) -> None:
        self.bot_token = bot_token or "bot_test_token_123456"
        self.bot_username = bot_username or "@bayworks_support_bot"

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        # Check for test / simulated credentials
        if (
            not self.bot_token
            or self.bot_token.startswith("bot_test")
            or self.bot_token.startswith("mock_")
            or "test" in self.bot_token.lower()
        ):
            msg_id = f"tg_msg_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider="telegram",
                provider_message_id=msg_id,
                raw_response={
                    "chat_id": contact.value,
                    "message_id": msg_id,
                    "channel": "telegram",
                    "simulated": True,
                },
            )

        endpoint = f"{self.api_base}/bot{self.bot_token}/sendMessage"
        payload = {
            "chat_id": contact.value,
            "text": body,
        }

        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                res = await client.post(endpoint, json=payload)
                res.raise_for_status()
                data = res.json()

            telegram_msg_id = str((data.get("result") or {}).get("message_id") or uuid.uuid4().hex[:16])
            return SendResult(
                provider="telegram",
                provider_message_id=telegram_msg_id,
                raw_response=data,
            )
        except Exception as e:
            logger.warning(
                f"Telegram live dispatch failed ({e}); using simulated fallback."
            )
            msg_id = f"tg_sim_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider="telegram",
                provider_message_id=msg_id,
                raw_response={
                    "chat_id": contact.value,
                    "message_id": msg_id,
                    "simulated": True,
                    "error": str(e),
                },
            )
