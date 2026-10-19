from __future__ import annotations

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.config import settings
from communicator.models.contact import Contact


class TwilioWhatsAppProvider(ChannelProvider):
    """
    WhatsApp via Twilio's WhatsApp API.

    Sends freeform "session" messages only -- under WhatsApp Business
    policy these only succeed within 24h of the contact's last inbound
    message. Re-engaging outside that window requires a pre-approved
    template message (Twilio Content API), which isn't implemented yet
    (see docs/ARCHITECTURE.md section 6/9 -- that's follow-up-sequence
    territory, Phase 4). Sending outside the window fails at Twilio's end;
    this provider surfaces that as an httpx.HTTPStatusError rather than
    trying to guess or silently downgrade to another channel.
    """

    api_base = "https://api.twilio.com/2010-04-01"

    def __init__(
        self,
        account_sid: str | None = None,
        auth_token: str | None = None,
        from_number: str | None = None,
    ) -> None:
        self.account_sid = account_sid if account_sid is not None else settings.twilio_account_sid
        self.auth_token = auth_token if auth_token is not None else settings.twilio_auth_token
        self.from_number = from_number if from_number is not None else settings.twilio_whatsapp_from
        if not (self.account_sid and self.auth_token and self.from_number):
            raise RuntimeError(
                "TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, and TWILIO_WHATSAPP_FROM must be set to send WhatsApp."
            )

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        data = {
            "From": f"whatsapp:{self.from_number}",
            "To": f"whatsapp:{contact.value}",
            "Body": body,
        }

        async with httpx.AsyncClient(
            base_url=self.api_base, auth=(self.account_sid, self.auth_token), timeout=30.0
        ) as client:
            response = await client.post(f"/Accounts/{self.account_sid}/Messages.json", data=data)
            response.raise_for_status()
            payload = response.json()

        return SendResult(
            provider="twilio", provider_message_id=payload.get("sid", ""), raw_response=payload
        )


class MetaWhatsAppProvider(ChannelProvider):
    """
    WhatsApp via Meta's Graph/Cloud API directly (no Twilio in between).

    Ported from the standalone Node auto-reply bot (`reply/`), which talked
    to the Cloud API directly to get interactive quick-reply buttons and
    looser send rules than Twilio's WhatsApp wrapper. Same 24h session-window
    policy applies on Meta's side; this provider doesn't enforce it itself.
    """

    def __init__(
        self,
        access_token: str | None = None,
        phone_number_id: str | None = None,
        api_version: str | None = None,
    ) -> None:
        self.access_token = (
            access_token if access_token is not None else settings.meta_whatsapp_access_token
        )
        self.phone_number_id = (
            phone_number_id
            if phone_number_id is not None
            else settings.meta_whatsapp_phone_number_id
        )
        self.api_version = (
            api_version if api_version is not None else settings.meta_whatsapp_api_version
        )
        if not (self.access_token and self.phone_number_id):
            raise RuntimeError(
                "META_WHATSAPP_ACCESS_TOKEN and META_WHATSAPP_PHONE_NUMBER_ID must be set to send WhatsApp via Meta."
            )

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": contact.value,
            "type": "text",
            "text": {"preview_url": False, "body": body},
        }

        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                url,
                json=payload,
                headers={"Authorization": f"Bearer {self.access_token}"},
            )
            response.raise_for_status()
            data = response.json()

        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)
