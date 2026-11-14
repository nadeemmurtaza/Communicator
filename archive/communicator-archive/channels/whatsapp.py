from __future__ import annotations

import asyncio
import logging
from typing import Any

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.config import settings
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class MetaWhatsAppAPIError(Exception):
    """Raised when Meta's Graph API returns an error response."""

    def __init__(
        self,
        message: str,
        code: int | None = None,
        subcode: int | None = None,
        status_code: int = 400,
        raw_error: dict[str, Any] | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        self.subcode = subcode
        self.status_code = status_code
        self.raw_error = raw_error or {}


class TwilioWhatsAppProvider(ChannelProvider):
    """
    WhatsApp via Twilio's WhatsApp API.

    Sends freeform "session" messages only -- under WhatsApp Business
    policy these only succeed within 24h of the contact's last inbound
    message. Re-engaging outside that window requires a pre-approved
    template message (Twilio Content API).
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
    - Text messages with preview URLs and 4096-char guard
    - Interactive Quick Reply buttons (up to 3 buttons)
    - Interactive List menus (sections and rows, up to 10 choices)
    - Interactive Call-To-Action (CTA) URL buttons
    - Interactive Location Requests
    - WhatsApp Commerce: Catalog messages, Single product, and Multi-product messages
    - Rich media (images, documents/PDFs, audio, video)
    - Location sharing
    - Emoji reactions
    - Message read receipts (mark_as_read)
    - Meta pre-approved template messages with dynamic header, body, and button params
    - Resilient exponential backoff retry loop on rate limits (429) and transient errors
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

    @property
    def messages_url(self) -> str:
        return f"https://graph.facebook.com/{self.api_version}/{self.phone_number_id}/messages"

    async def _post_with_retry(
        self,
        url: str,
        payload: dict[str, Any],
        max_retries: int = 3,
        initial_delay: float = 0.5,
    ) -> dict[str, Any]:
        """
        Sends an HTTP POST to Meta Graph API with exponential backoff on
        rate limits (429) or transient server errors (500, 502, 503, 504).
        """
        delay = initial_delay
        headers = {"Authorization": f"Bearer {self.access_token}"}
        last_exception: Exception | None = None

        for attempt in range(max_retries + 1):
            try:
                async with httpx.AsyncClient(timeout=30.0) as client:
                    response = await client.post(url, json=payload, headers=headers)
                    if response.status_code in (200, 201):
                        return response.json()

                    # Parse detailed Meta error if available
                    err_dict: dict[str, Any] = {}
                    err_msg = response.text
                    err_code: int | None = None
                    err_subcode: int | None = None
                    try:
                        err_json = response.json()
                        if isinstance(err_json, dict) and "error" in err_json:
                            err_dict = err_json["error"]
                            err_msg = err_dict.get("message", response.text)
                            err_code = err_dict.get("code")
                            err_subcode = err_dict.get("error_subcode")
                    except Exception:  # noqa: BLE001, S110
                        pass

                    # Rate limited or transient server error: back off and retry
                    if response.status_code in (429, 500, 502, 503, 504) and attempt < max_retries:
                        logger.warning(
                            f"Meta API error {response.status_code} ({err_code}/{err_subcode}) on attempt {attempt + 1}. "
                            f"Retrying in {delay}s..."
                        )
                        await asyncio.sleep(delay)
                        delay *= 2
                        continue

                    raise MetaWhatsAppAPIError(
                        f"Meta API Error ({err_code}/{err_subcode}): {err_msg}",
                        code=err_code,
                        subcode=err_subcode,
                        status_code=response.status_code,
                        raw_error=err_dict,
                    )
            except (httpx.ConnectError, httpx.TimeoutException) as exc:
                last_exception = exc
                if attempt < max_retries:
                    logger.warning(
                        f"Network error {exc} on attempt {attempt + 1}. Retrying in {delay}s..."
                    )
                    await asyncio.sleep(delay)
                    delay *= 2
                    continue
                raise MetaWhatsAppAPIError(
                    f"Meta connection failed after {max_retries} retries: {exc}"
                ) from exc

        if last_exception:
            raise MetaWhatsAppAPIError(f"Meta request failed: {last_exception}") from last_exception
        raise MetaWhatsAppAPIError("Meta request failed unexpectedly")

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        return await self.send_text(contact.value, body)

    async def send_text(self, to: str, body: str, preview_url: bool = False) -> SendResult:
        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "text",
            "text": {"preview_url": preview_url, "body": body[:4096]},
        }
        data = await self._post_with_retry(self.messages_url, payload)
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

        interactive_payload: dict[str, Any] = {
            "type": "button",
            "body": {"text": body_text[:1024]},
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
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_interactive_list(
        self,
        to: str,
        body_text: str,
        button_title: str,
        sections: list[dict[str, Any]],
        header_text: str | None = None,
        footer_text: str | None = None,
    ) -> SendResult:
        """
        Sends an interactive list menu (up to 10 rows total across sections).
        """
        formatted_sections = []
        total_rows = 0
        for sec in sections:
            rows = []
            for r in sec.get("rows", []):
                if total_rows >= 10:
                    break
                rows.append(
                    {
                        "id": str(r.get("id", ""))[:200],
                        "title": str(r.get("title", ""))[:24],
                        **(
                            {"description": str(r.get("description", ""))[:72]}
                            if r.get("description")
                            else {}
                        ),
                    }
                )
                total_rows += 1
            if rows:
                formatted_sections.append(
                    {
                        "title": str(sec.get("title", ""))[:24],
                        "rows": rows,
                    }
                )

        interactive_payload: dict[str, Any] = {
            "type": "list",
            "body": {"text": body_text[:1024]},
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
        data = await self._post_with_retry(self.messages_url, payload)
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
        interactive_payload: dict[str, Any] = {
            "type": "cta_url",
            "body": {"text": body_text[:1024]},
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
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_location_request(
        self,
        to: str,
        body_text: str,
    ) -> SendResult:
        """Sends an interactive location request message asking user to share their location."""
        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": {
                "type": "location_request_message",
                "body": {"text": body_text[:1024]},
                "action": {"name": "send_location"},
            },
        }
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_catalog_message(
        self,
        to: str,
        body_text: str,
        catalog_id: str,
        thumbnail_product_retailer_id: str | None = None,
        footer_text: str | None = None,
    ) -> SendResult:
        """Sends a WhatsApp Commerce Catalog message allowing browsing the whole store."""
        action_payload: dict[str, Any] = {"name": "catalog_message"}
        if thumbnail_product_retailer_id:
            action_payload["parameters"] = {
                "thumbnail_product_retailer_id": thumbnail_product_retailer_id
            }

        interactive_payload: dict[str, Any] = {
            "type": "catalog_message",
            "body": {"text": body_text[:1024]},
            "action": action_payload,
        }
        if footer_text:
            interactive_payload["footer"] = {"text": footer_text[:60]}

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": interactive_payload,
        }
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_product_message(
        self,
        to: str,
        catalog_id: str,
        product_retailer_id: str,
        body_text: str | None = None,
        footer_text: str | None = None,
    ) -> SendResult:
        """Sends a single product card message from a Meta catalog."""
        interactive_payload: dict[str, Any] = {
            "type": "product",
            "action": {
                "catalog_id": catalog_id,
                "product_retailer_id": product_retailer_id,
            },
        }
        if body_text:
            interactive_payload["body"] = {"text": body_text[:1024]}
        if footer_text:
            interactive_payload["footer"] = {"text": footer_text[:60]}

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": interactive_payload,
        }
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_multi_product_message(
        self,
        to: str,
        catalog_id: str,
        header_text: str,
        body_text: str,
        sections: list[dict[str, Any]],
        footer_text: str | None = None,
    ) -> SendResult:
        """Sends a multi-product message grouping products by sections."""
        interactive_payload: dict[str, Any] = {
            "type": "product_list",
            "header": {"type": "text", "text": header_text[:60]},
            "body": {"text": body_text[:1024]},
            "action": {
                "catalog_id": catalog_id,
                "sections": sections,
            },
        }
        if footer_text:
            interactive_payload["footer"] = {"text": footer_text[:60]}

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": "interactive",
            "interactive": interactive_payload,
        }
        data = await self._post_with_retry(self.messages_url, payload)
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
        is_link = media_url_or_id.startswith(("http://", "https://"))
        media_payload: dict[str, Any] = {"link" if is_link else "id": media_url_or_id}
        if caption and media_type in ("image", "document", "video"):
            media_payload["caption"] = caption[:1024]
        if filename and media_type == "document":
            media_payload["filename"] = filename

        payload = {
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.lstrip("+"),
            "type": media_type,
            media_type: media_payload,
        }
        data = await self._post_with_retry(self.messages_url, payload)
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
        loc_payload: dict[str, Any] = {
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
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def send_reaction(self, to: str, message_id: str, emoji: str) -> SendResult:
        """Reacts to a user's message with an emoji."""
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
        data = await self._post_with_retry(self.messages_url, payload)
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)

    async def mark_as_read(self, message_id: str) -> bool:
        """Marks user's incoming message as read (blue double check marks on WhatsApp)."""
        if not message_id:
            return False
        payload = {
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id,
        }
        try:
            data = await self._post_with_retry(self.messages_url, payload, max_retries=1)
            return bool(data.get("success", True))
        except Exception:  # noqa: BLE001
            return False

    async def send_template(
        self,
        to: str,
        template_name: str,
        language: str = "en_US",
        body_params: list[str] | None = None,
        header_params: list[dict[str, Any]] | None = None,
        button_params: list[dict[str, Any]] | None = None,
    ) -> SendResult:
        """
        Sends an approved template message -- allows re-engaging outside a contact's
        24h session window. Supports body parameters, header parameters, and button parameters.
        """
        components: list[dict[str, Any]] = []

        if header_params:
            components.append(
                {
                    "type": "header",
                    "parameters": header_params,
                }
            )

        if body_params:
            components.append(
                {
                    "type": "body",
                    "parameters": [{"type": "text", "text": str(p)} for p in body_params],
                }
            )

        if button_params:
            components.extend(button_params)

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
        data = await self._post_with_retry(self.messages_url, payload)
        message_id = (data.get("messages") or [{}])[0].get("id", "")
        return SendResult(provider="meta", provider_message_id=message_id, raw_response=data)
