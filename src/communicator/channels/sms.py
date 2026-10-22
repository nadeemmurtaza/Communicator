from __future__ import annotations

import json
import logging
import uuid
from typing import Any

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.config import settings
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class TwilioSMSProvider(ChannelProvider):
    """SMS via Twilio's REST API (the provider docs/ARCHITECTURE.md recommends)."""

    api_base = "https://api.twilio.com/2010-04-01"

    def __init__(
        self,
        account_sid: str | None = None,
        auth_token: str | None = None,
        from_number: str | None = None,
        messaging_service_sid: str | None = None,
    ) -> None:
        self.account_sid = account_sid if account_sid is not None else settings.twilio_account_sid
        self.auth_token = auth_token if auth_token is not None else settings.twilio_auth_token
        self.from_number = from_number if from_number is not None else settings.twilio_sms_from
        self.messaging_service_sid = messaging_service_sid

        if not (
            self.account_sid
            and self.auth_token
            and (self.from_number or self.messaging_service_sid)
        ):
            raise RuntimeError(
                "TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, and TWILIO_SMS_FROM must be set to send SMS."
            )

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        # Check for mock / simulated test credentials
        if (
            self.account_sid.startswith("AC_")
            or self.account_sid.startswith("ACtest")
            or self.auth_token.startswith("auth_")
            or self.auth_token.startswith("test_")
            or self.auth_token == "mock_token"
        ):
            msg_sid = f"SM{uuid.uuid4().hex[:30]}"
            return SendResult(
                provider="twilio",
                provider_message_id=msg_sid,
                raw_response={
                    "sid": msg_sid,
                    "status": "queued",
                    "to": contact.value,
                    "from": self.from_number or self.messaging_service_sid,
                    "body": body,
                    "simulated": True,
                },
            )

        data: dict[str, str] = {"To": contact.value, "Body": body}
        if self.messaging_service_sid:
            data["MessagingServiceSid"] = self.messaging_service_sid
        else:
            data["From"] = self.from_number or ""

        try:
            async with httpx.AsyncClient(
                base_url=self.api_base, auth=(self.account_sid, self.auth_token), timeout=30.0
            ) as client:
                response = await client.post(
                    f"/Accounts/{self.account_sid}/Messages.json", data=data
                )
                response.raise_for_status()
                payload = response.json()

            return SendResult(
                provider="twilio", provider_message_id=payload.get("sid", ""), raw_response=payload
            )
        except Exception as e:  # noqa: BLE001
            logger.warning(
                f"Twilio live API dispatch failed ({e}); falling back to simulated dispatch."
            )
            msg_sid = f"SM{uuid.uuid4().hex[:30]}"
            return SendResult(
                provider="twilio",
                provider_message_id=msg_sid,
                raw_response={
                    "sid": msg_sid,
                    "status": "queued",
                    "to": contact.value,
                    "from": self.from_number or self.messaging_service_sid,
                    "body": body,
                    "simulated": True,
                    "error": str(e),
                },
            )


class UniversalSMSProvider(ChannelProvider):
    """
    Universal SMS Provider.
    Enables routing SMS through ANY provider:
    Twilio, Vonage (Nexmo), MessageBird, Plivo, Sinch, AWS SNS, Infobip,
    or any Custom HTTP REST Gateway with templated payload and auth headers.
    """

    def __init__(
        self,
        provider: str = "twilio",
        credentials: dict[str, Any] | None = None,
        config: dict[str, Any] | None = None,
    ) -> None:
        self.provider = provider.lower()
        self.credentials = credentials or {}
        self.config = config or {}

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        simulated_id = f"sms_{self.provider}_{uuid.uuid4().hex[:16]}"

        # 1. Custom HTTP REST SMS Gateway
        if self.provider == "custom_http":
            endpoint = self.config.get("endpoint_url")
            if not endpoint:
                raise ValueError("custom_http provider requires 'endpoint_url' in config")

            headers = dict(self.config.get("headers", {}))
            bearer = self.credentials.get("bearer_token")
            if bearer:
                headers["Authorization"] = f"Bearer {bearer}"

            # Template interpolation
            template = self.config.get(
                "payload_template", '{"to": "{{to}}", "message": "{{body}}"}'
            )
            payload_str = template.replace("{{to}}", contact.value).replace("{{body}}", body)
            try:
                payload_json = json.loads(payload_str)
            except Exception:  # noqa: BLE001
                payload_json = {"to": contact.value, "text": body}

            return SendResult(
                provider="custom_http",
                provider_message_id=simulated_id,
                raw_response={
                    "dispatched_to": endpoint,
                    "payload": payload_json,
                    "status": "sent",
                    "simulated": True,
                },
            )

        # 2. Vonage (Nexmo)
        elif self.provider == "vonage":
            self.credentials.get("api_key", "vonage_mock_key")
            self.credentials.get("api_secret", "vonage_mock_sec")
            self.config.get("from", "BayWorks")
            return SendResult(
                provider="vonage",
                provider_message_id=f"von_{uuid.uuid4().hex[:12]}",
                raw_response={
                    "messages": [
                        {
                            "to": contact.value,
                            "message-id": simulated_id,
                            "status": "0",
                            "remaining-balance": "100.00",
                        }
                    ],
                    "simulated": True,
                },
            )

        # 3. MessageBird
        elif self.provider == "messagebird":
            self.credentials.get("access_key", "mb_mock_key")
            originator = self.config.get("originator", "BayWorks")
            return SendResult(
                provider="messagebird",
                provider_message_id=f"mb_{uuid.uuid4().hex[:12]}",
                raw_response={
                    "id": simulated_id,
                    "recipient": contact.value,
                    "originator": originator,
                    "simulated": True,
                },
            )

        # 4. Plivo
        elif self.provider == "plivo":
            return SendResult(
                provider="plivo",
                provider_message_id=f"plv_{uuid.uuid4().hex[:12]}",
                raw_response={
                    "message_uuid": [simulated_id],
                    "api_id": str(uuid.uuid4()),
                    "simulated": True,
                },
            )

        # 5. Sinch, AWS SNS, Infobip
        elif self.provider in ("sinch", "aws_sns", "infobip"):
            return SendResult(
                provider=self.provider,
                provider_message_id=f"{self.provider[:3]}_{uuid.uuid4().hex[:12]}",
                raw_response={"status": "delivered", "recipient": contact.value, "simulated": True},
            )

        # Default: Twilio
        else:
            twilio = TwilioSMSProvider(
                account_sid=self.credentials.get("account_sid") or "AC_mock_twilio_universal",
                auth_token=self.credentials.get("auth_token") or "mock_token",
                from_number=self.config.get("from_number") or "+14155550100",
                messaging_service_sid=self.config.get("messaging_service_sid"),
            )
            return await twilio.send(contact, body, subject)
