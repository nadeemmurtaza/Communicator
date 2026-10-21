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

    Full enterprise automation capabilities:
    - Text messages with preview URLs
    - Interactive Quick Reply buttons (up to 3 buttons)
    - Interactive List menus (sections and rows, up to 10 choices)
    - Interactive Call-To-Action (CTA) URL buttons
    - Rich media (images, documents/PDFs, audio, video)
    - Location sharing
    - Emoji reactions
    - Message read receipts (mark_as_read)
    - Meta pre-approved template messages (re-engaging outside 24h window)
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
        return await self.send_text(contact.value, body)

    async def send_text(self, to: str, body: str, preview_url: bool = False) -> SendResult:
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "text",
            "text": {"preview_url": preview_url, "body": body},
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

    async def send_interactive_buttons(
        self,
        to: str,
        body_text: str,
        buttons: list[dict[str, str]],
        header_text: str | None = None,
        footer_text: str | None = None,
    ) -> SendResult:
        """
        Sends interactive quick-reply buttons (up to 3 buttons).
        Each button in `buttons` has:
          - id: str (payload returned on click, e.g. "pricing_catalog")
          - title: str (button label, max 20 chars)
        """
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        formatted_buttons = [
            {
                "type": "reply",
                "reply": {
                    "id": str(btn.get("id", f"btn_{i}"))[:256],
                    "title": str(btn.get("title", ""))[:20],
                },
            }
            for i, btn in enumerate(buttons[:3])
        ]

        interactive_payload: dict = {
            "type": "button",
            "body": {"text": body_text},
            "action": {"buttons": formatted_buttons},
        }
        if header_text:
            interactive_payload["header"] = {"type": "text", "text": header_text[:60]}
        if footer_text:
            interactive_payload["footer"] = {"text": footer_text[:60]}

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": interactive_payload,
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

    async def send_interactive_list(
        self,
        to: str,
        body_text: str,
        button_title: str,
        sections: list[dict],
        header_text: str | None = None,
        footer_text: str | None = None,
    ) -> SendResult:
        """
        Sends an interactive list menu (up to 10 rows total across sections).
        `sections`: [
          {
            "title": "Section Title",
            "rows": [
              {"id": "svc_1", "title": "Service 1", "description": "Details..."}
            ]
          }
        ]
        """
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        formatted_sections = []
        for sec in sections:
            rows = [
                {
                    "id": str(r.get("id", ""))[:200],
                    "title": str(r.get("title", ""))[:24],
                    **({"description": str(r.get("description", ""))[:72]} if r.get("description") else {}),
                }
                for r in sec.get("rows", [])
            ]
            formatted_sections.append({
                "title": str(sec.get("title", ""))[:24],
                "rows": rows,
            })

        interactive_payload: dict = {
            "type": "list",
            "body": {"text": body_text},
            "action": {
                "button": (button_title or "Options")[:20],
                "sections": formatted_sections,
            },
        }
        if header_text:
            interactive_payload["header"] = {"type": "text", "text": header_text[:60]}
        if footer_text:
            interactive_payload["footer"] = {"text": footer_text[:60]}

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": interactive_payload,
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

    async def send_interactive_cta_url(
        self,
        to: str,
        body_text: str,
        display_text: str,
        url: str,
        header_text: str | None = None,
        footer_text: str | None = None,
    ) -> SendResult:
        """Sends an interactive Call-to-Action button with a clickable web link."""
        graph_url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        interactive_payload: dict = {
            "type": "cta_url",
            "body": {"text": body_text},
            "action": {
                "name": "cta_url",
                "parameters": {
                    "display_text": display_text[:20],
                    "url": url,
                },
            },
        }
        if header_text:
            interactive_payload["header"] = {"type": "text", "text": header_text[:60]}
        if footer_text:
            interactive_payload["footer"] = {"text": footer_text[:60]}

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": interactive_payload,
        }

        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                graph_url,
                json=payload,
                headers={"Authorization": f"Bearer {self.access_token}"},
            )
            response.raise_for_status()
            data = response.json()

        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_media(
        self,
        to: str,
        media_type: str,
        media_url_or_id: str,
        caption: str | None = None,
        filename: str | None = None,
    ) -> SendResult:
        """Sends media: 'image', 'document', 'audio', 'video'."""
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        is_link = media_url_or_id.startswith("http://") or media_url_or_id.startswith("https://")
        media_payload: dict = {"link" if is_link else "id": media_url_or_id}
        if caption and media_type in ("image", "document", "video"):
            media_payload["caption"] = caption
        if filename and media_type == "document":
            media_payload["filename"] = filename

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": media_type,
            media_type: media_payload,
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

    async def send_location(
        self,
        to: str,
        latitude: float,
        longitude: float,
        name: str | None = None,
        address: str | None = None,
    ) -> SendResult:
        """Sends GPS / office / store location coordinates."""
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        loc_payload: dict = {
            "latitude": latitude,
            "longitude": longitude,
        }
        if name:
            loc_payload["name"] = name
        if address:
            loc_payload["address"] = address

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "location",
            "location": loc_payload,
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

    async def send_reaction(self, to: str, message_id: str, emoji: str) -> SendResult:
        """Reacts to a user's message with an emoji (e.g. '👍', '❤️')."""
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "reaction",
            "reaction": {
                "message_id": message_id,
                "emoji": emoji,
            },
        }

        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                url,
                json=payload,
                headers={"Authorization": f"Bearer {self.access_token}"},
            )
            response.raise_for_status()
            data = response.json()

        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def mark_as_read(self, message_id: str) -> bool:
        """Marks user's incoming message as read (blue double check marks on WhatsApp)."""
        if not message_id:
            return False
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        payload = {
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id,
        }
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                res = await client.post(
                    url,
                    json=payload,
                    headers={"Authorization": f"Bearer {self.access_token}"},
                )
                return res.status_code in (200, 201)
        except Exception:
            return False

    async def send_template(
        self,
        to: str,
        template_name: str,
        language: str = "en_US",
        body_params: list[str] | None = None,
    ) -> SendResult:
        """
        Sends an approved template message -- the only kind Meta allows
        outside a contact's 24h session window, so this is what broadcasts
        use instead of `send`.
        """
        url = f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"
        components = []
        if body_params:
            components.append(
                {
                    "type": "body",
                    "parameters": [{"type": "text", "text": p} for p in body_params],
                }
            )

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "template",
            "template": {
                "name": template_name,
                "language": {"code": language},
                **({"components": components} if components else {}),
            },
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
