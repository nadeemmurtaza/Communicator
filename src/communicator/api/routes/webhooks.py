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

from fastapi import APIRouter, Depends, Header, HTTPException, Request

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.contact import ChannelType
from communicator.services.inbound import UnknownContactError, process_inbound_message
from communicator.webhooks.twilio_signature import validate_twilio_signature

router = APIRouter(prefix="/webhooks", tags=["Webhooks"])


def _require_twilio_signature(request: Request, form: dict[str, str], signature: str | None) -> None:
    if not settings.twilio_auth_token:
        raise HTTPException(status_code=503, detail="Twilio webhook not configured.")
    if not signature or not validate_twilio_signature(settings.twilio_auth_token, str(request.url), form, signature):
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
