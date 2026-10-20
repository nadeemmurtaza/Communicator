"""
Inbound message webhooks for each channel provider. Twilio requests are
verified via X-Twilio-Signature (see communicator.webhooks.twilio_signature);
Postmark doesn't sign inbound webhook requests, so its URL carries a
shared-secret query token instead. A message from a contact we don't
recognize is acknowledged with 200 and dropped -- there's nothing in our
data model to attach it to, and providers retry on non-2xx responses.
"""

from __future__ import annotations

import hmac

from fastapi import APIRouter, Depends, Header, HTTPException, Query, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.contact import ChannelType
from communicator.services.inbound import UnknownContactError, process_inbound_message
from communicator.services.integration_settings import get_whatsapp_config
from communicator.webhooks.meta_signature import validate_meta_signature
from communicator.webhooks.twilio_signature import validate_twilio_signature

router = APIRouter(prefix="/webhooks", tags=["Webhooks"])


def _require_twilio_signature(
    request: Request, form: dict[str, str], signature: str | None
) -> None:
    if not settings.twilio_auth_token:
        raise HTTPException(status_code=503, detail="Twilio webhook not configured.")
    if not signature or not validate_twilio_signature(
        settings.twilio_auth_token, str(request.url), form, signature
    ):
        raise HTTPException(status_code=403, detail="Invalid Twilio signature.")


@router.post("/twilio/sms")
async def twilio_sms_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_twilio_signature: str | None = Header(default=None),
) -> dict[str, str]:
    form = dict(await request.form())
    _require_twilio_signature(request, form, x_twilio_signature)

    try:
        await process_inbound_message(
            session,
            channel=ChannelType.SMS,
            from_value=form.get("From", ""),
            body=form.get("Body", ""),
            provider="twilio",
            provider_message_id=form.get("MessageSid", ""),
            harvester_client=harvester_client,
        )
    except UnknownContactError:
        pass

    return {"status": "ok"}


@router.post("/twilio/whatsapp")
async def twilio_whatsapp_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_twilio_signature: str | None = Header(default=None),
) -> dict[str, str]:
    form = dict(await request.form())
    _require_twilio_signature(request, form, x_twilio_signature)

    # WhatsApp numbers arrive as "whatsapp:+92..."; Contacts are stored as
    # plain numbers to match Harvester's contact_points values.
    from_value = form.get("From", "").removeprefix("whatsapp:")

    try:
        await process_inbound_message(
            session,
            channel=ChannelType.WHATSAPP,
            from_value=from_value,
            body=form.get("Body", ""),
            provider="twilio",
            provider_message_id=form.get("MessageSid", ""),
            harvester_client=harvester_client,
        )
    except UnknownContactError:
        pass

    return {"status": "ok"}


@router.get("/meta/whatsapp")
async def meta_whatsapp_verify(
    hub_mode: str | None = Query(default=None, alias="hub.mode"),
    hub_verify_token: str | None = Query(default=None, alias="hub.verify_token"),
    hub_challenge: str | None = Query(default=None, alias="hub.challenge"),
    session: AsyncSession = Depends(get_db_session),
) -> Response:
    """Meta's webhook handshake -- it GETs this URL once when you subscribe it."""
    config = await get_whatsapp_config(session)
    if hub_mode == "subscribe" and hub_verify_token == config.webhook_verify_token:
        return Response(content=hub_challenge or "", media_type="text/plain")
    raise HTTPException(status_code=403, detail="Verification token mismatch")


async def _require_meta_signature(
    session: AsyncSession, raw_body: bytes, signature: str | None
) -> None:
    config = await get_whatsapp_config(session)
    if not config.app_secret:
        raise HTTPException(status_code=503, detail="Meta webhook not configured.")
    if not validate_meta_signature(config.app_secret, raw_body, signature):
        raise HTTPException(status_code=403, detail="Invalid Meta webhook signature.")


def _normalize_meta_sender(sender_id: str) -> str:
    # Contacts are stored E.164-style ("+92..."); Meta's `from` is digits-only.
    return "+" + sender_id.lstrip("+") if sender_id else sender_id


@router.post("/meta/whatsapp")
async def meta_whatsapp_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_hub_signature_256: str | None = Header(default=None),
) -> dict[str, str]:
    """
    Cloud API inbound events. Ported from the standalone reply bot's
    `parseWhatsAppWebhook` -- same entry/changes/value.messages shape, only
    text and button/interactive replies are handled.
    """
    raw_body = await request.body()
    await _require_meta_signature(session, raw_body, x_hub_signature_256)

    body = await request.json()
    if body.get("object") != "whatsapp_business_account":
        return {"status": "ignored"}

    for entry in body.get("entry", []):
        for change in entry.get("changes", []):
            value = change.get("value") or {}
            for msg in value.get("messages", []):
                text = _extract_meta_message_text(msg)
                if not text:
                    continue
                try:
                    await process_inbound_message(
                        session,
                        channel=ChannelType.WHATSAPP,
                        from_value=_normalize_meta_sender(msg.get("from", "")),
                        body=text,
                        provider="meta",
                        provider_message_id=msg.get("id", ""),
                        harvester_client=harvester_client,
                    )
                except UnknownContactError:
                    pass

    return {"status": "ok"}


def _extract_meta_message_text(msg: dict) -> str:
    msg_type = msg.get("type")
    if msg_type == "text":
        return (msg.get("text") or {}).get("body", "")
    if msg_type == "interactive":
        interactive = msg.get("interactive") or {}
        if interactive.get("type") == "button_reply":
            return (interactive.get("button_reply") or {}).get("title", "")
        if interactive.get("type") == "list_reply":
            return (interactive.get("list_reply") or {}).get("title", "")
    if msg_type == "button":
        return (msg.get("button") or {}).get("text", "")
    return ""


@router.post("/postmark/inbound")
async def postmark_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    token: str | None = None,
) -> dict[str, str]:
    if not settings.postmark_inbound_webhook_secret:
        raise HTTPException(status_code=503, detail="Postmark inbound webhook not configured.")
    if not token or not hmac.compare_digest(token, settings.postmark_inbound_webhook_secret):
        raise HTTPException(status_code=403, detail="Invalid or missing webhook token.")

    payload = await request.json()
    from_value = (payload.get("FromFull") or {}).get("Email") or payload.get("From", "")

    try:
        await process_inbound_message(
            session,
            channel=ChannelType.EMAIL,
            from_value=from_value,
            body=payload.get("TextBody", ""),
            provider="postmark",
            provider_message_id=payload.get("MessageID", ""),
            harvester_client=harvester_client,
        )
    except UnknownContactError:
        pass

    return {"status": "ok"}
