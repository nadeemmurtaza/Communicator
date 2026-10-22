from __future__ import annotations

import hashlib
import hmac
import logging
import uuid
from typing import Any

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class TikTokProvider(ChannelProvider):
    """
    Provider for TikTok Direct Messages & Video Comment Interactions.
    Supports sending DMs via the TikTok Business API (v2),
    replying to customer video comments, and simulated dispatch in test environments.
    """

    api_base = "https://open.tiktokapis.com/v2"

    def __init__(
        self,
        access_token: str | None = None,
        app_id: str | None = None,
        app_secret: str | None = None,
        business_account_id: str | None = None,
    ) -> None:
        self.access_token = access_token or "tiktok_test_token_12345"
        self.app_id = app_id or "tt_app_mock_6789"
        self.app_secret = app_secret or "mock_secret_tiktok"
        self.business_account_id = business_account_id or "tt_biz_acc_1001"

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        """
        Sends an outbound TikTok direct message to the user identified by open_id.
        """
        open_id = contact.value

        # Check for test / simulated credentials
        if (
            not self.access_token
            or self.access_token.startswith("tiktok_test")
            or self.access_token.startswith("mock_")
            or "test" in self.access_token.lower()
        ):
            msg_id = f"tt_msg_{uuid.uuid4().hex[:16]}"
            return SendResult(
                provider="tiktok",
                provider_message_id=msg_id,
                raw_response={
                    "open_id": open_id,
                    "message_id": msg_id,
                    "channel": "tiktok",
                    "simulated": True,
                },
            )

        endpoint = f"{self.api_base}/im/message/send/"
        headers = {
            "Authorization": f"Bearer {self.access_token}",
            "Content-Type": "application/json",
        }
        payload = {
            "to_user_id": open_id,
            "message_type": "text",
            "content": {
                "text": body,
            },
        }

        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.post(endpoint, json=payload, headers=headers)
            if resp.status_code != 200:
                logger.error("TikTok API error %d: %s", resp.status_code, resp.text)
                resp.raise_for_status()

            data = resp.json()
            msg_id = data.get("data", {}).get("message_id") or f"tt_{uuid.uuid4().hex[:12]}"
            return SendResult(
                provider="tiktok",
                provider_message_id=msg_id,
                raw_response=data,
            )

    async def reply_video_comment(self, comment_id: str, body: str) -> dict[str, Any]:
        """Replies to a user comment on a TikTok video."""
        if (
            not self.access_token
            or self.access_token.startswith("tiktok_test")
            or self.access_token.startswith("mock_")
            or "test" in self.access_token.lower()
        ):
            return {
                "status": "success",
                "comment_id": comment_id,
                "reply_id": f"tt_cmt_reply_{uuid.uuid4().hex[:12]}",
                "simulated": True,
            }

        endpoint = f"{self.api_base}/research/video/comment/reply/"
        headers = {
            "Authorization": f"Bearer {self.access_token}",
            "Content-Type": "application/json",
        }
        payload = {
            "comment_id": comment_id,
            "text": body,
        }

        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.post(endpoint, json=payload, headers=headers)
            resp.raise_for_status()
            return resp.json()

    @staticmethod
    def verify_signature(payload_bytes: bytes, signature: str, app_secret: str) -> bool:
        """
        Validates TikTok webhook HMAC-SHA256 signature against X-TikTok-Signature header.
        """
        if not signature or not app_secret:
            return False
        expected = hmac.new(app_secret.encode("utf-8"), payload_bytes, hashlib.sha256).hexdigest()
        return hmac.compare_digest(expected, signature)
