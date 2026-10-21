"""
Inbound message webhooks for each channel provider. Twilio requests are
verified via X-Twilio-Signature (see communicator.webhooks.twilio_signature);
Postmark doesn't sign inbound webhook requests, so its URL carries a
shared-secret query token instead. A message from a contact we don't
recognize is acknowledged with 200 and dropped -- there's nothing in our
data model to attach it to, and providers retry on non-2xx responses.
"""

from __future__ import annotations

from datetime import UTC, datetime
import hmac

from fastapi import APIRouter, Depends, Header, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.broadcast import Broadcast, BroadcastRecipient, RecipientStatus
from communicator.models.contact import ChannelType
from communicator.models.message import Message, MessageStatus
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
    Cloud API inbound events. Handles:
    - Text messages
    - Interactive quick-reply buttons (id + title)
    - Interactive list menu selections (id + title)
    - Media attachments (image, document, audio, video)
    - Location pins
    - Delivery and read status receipts
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
                msg_id = msg.get("id", "")
                # Webhook idempotency: skip already-processed messages
                if msg_id:
                    existing_msg = (
                        await session.execute(
                            select(Message.id).where(Message.provider_message_id == msg_id)
                        )
                    ).scalar_one_or_none()
                    if existing_msg is not None:
                        continue

                text = _extract_meta_message_text(msg)
                payload_id = _extract_meta_payload_id(msg)
                if not text and not payload_id:
                    continue
                if not text and payload_id:
                    text = payload_id

                try:
                    await process_inbound_message(
                        session,
                        channel=ChannelType.WHATSAPP,
                        from_value=_normalize_meta_sender(msg.get("from", "")),
                        body=text,
                        provider="meta",
                        provider_message_id=msg_id,
                        harvester_client=harvester_client,
                        interactive_payload_id=payload_id,
                    )
                except UnknownContactError:
                    pass

            for status_event in value.get("statuses", []):
                await _apply_meta_status(session, status_event)

    return {"status": "ok"}


_META_STATUS_TO_MESSAGE_STATUS = {
    "sent": MessageStatus.SENT,
    "delivered": MessageStatus.DELIVERED,
    "read": MessageStatus.DELIVERED,  # no READ value on Message; DELIVERED is the closest we track
    "failed": MessageStatus.FAILED,
}
_META_STATUS_TO_RECIPIENT_STATUS = {
    "sent": RecipientStatus.SENT,
    "delivered": RecipientStatus.DELIVERED,
    "read": RecipientStatus.READ,
    "failed": RecipientStatus.FAILED,
}


async def _apply_meta_status(session: AsyncSession, status_event: dict) -> None:
    """
    Delivery/read receipts for messages we sent -- updates both the
    Message row (1:1 conversations) and any BroadcastRecipient row
    (campaign sends), matched by Meta's provider_message_id.
    """
    provider_message_id = status_event.get("id", "")
    meta_status = status_event.get("status", "")
    if not provider_message_id or meta_status not in _META_STATUS_TO_MESSAGE_STATUS:
        return

    # Extract event timestamp if provided
    event_dt = datetime.now(UTC)
    raw_ts = status_event.get("timestamp")
    if raw_ts:
        try:
            event_dt = datetime.fromtimestamp(int(raw_ts), tz=UTC)
        except Exception:
            pass

    # Extract detailed error message if failed
    failure_reason: str | None = None
    if meta_status == "failed":
        errors = status_event.get("errors", [])
        if errors and isinstance(errors, list):
            err = errors[0]
            err_code = err.get("code")
            err_title = err.get("title", "")
            err_details = (err.get("error_data") or {}).get("details") or err.get("message", "")
            if err_code == 131026 or "not on whatsapp" in err_title.lower() or "not on whatsapp" in err_details.lower():
                failure_reason = "Number is not on WhatsApp"
            elif err_details:
                failure_reason = f"{err_title}: {err_details}"
            else:
                failure_reason = err_title or f"Meta error {err_code}"
        else:
            failure_reason = "Message delivery failed"

    message_stmt = select(Message).where(Message.provider_message_id == provider_message_id)
    message = (await session.execute(message_stmt)).scalar_one_or_none()
    if message is not None:
        message.status = _META_STATUS_TO_MESSAGE_STATUS[meta_status]

    recipient_stmt = select(BroadcastRecipient).where(
        BroadcastRecipient.provider_message_id == provider_message_id
    )
    recipient = (await session.execute(recipient_stmt)).scalar_one_or_none()
    if recipient is not None:
        new_status = _META_STATUS_TO_RECIPIENT_STATUS[meta_status]
        old_status = recipient.status
        recipient.status = new_status

        if meta_status == "sent" and not recipient.sent_at:
            recipient.sent_at = event_dt
        elif meta_status == "delivered" and not recipient.delivered_at:
            recipient.delivered_at = event_dt
        elif meta_status == "read" and not recipient.read_at:
            recipient.read_at = event_dt
        elif meta_status == "failed":
            recipient.error = failure_reason

        broadcast = await session.get(Broadcast, recipient.broadcast_id)
        if broadcast is not None and old_status != new_status:
            if meta_status == "delivered" and old_status not in (RecipientStatus.DELIVERED, RecipientStatus.READ):
                broadcast.delivered_count += 1
            elif meta_status == "read":
                broadcast.read_count += 1
                if old_status not in (RecipientStatus.DELIVERED, RecipientStatus.READ):
                    broadcast.delivered_count += 1
            elif meta_status == "failed" and old_status != RecipientStatus.FAILED:
                broadcast.failed_count += 1

    await session.flush()


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
    if msg_type == "image":
        caption = (msg.get("image") or {}).get("caption", "")
        return caption if caption else "[User sent an image]"
    if msg_type == "document":
        caption = (msg.get("document") or {}).get("caption", "")
        filename = (msg.get("document") or {}).get("filename", "document")
        return caption if caption else f"[User sent document: {filename}]"
    if msg_type == "audio":
        return "[User sent an audio message]"
    if msg_type == "video":
        caption = (msg.get("video") or {}).get("caption", "")
        return caption if caption else "[User sent a video]"
    if msg_type == "location":
        loc = msg.get("location") or {}
        name = loc.get("name") or loc.get("address") or ""
        return f"[User shared location: {name}]" if name else "[User shared location]"
    return ""


def _extract_meta_payload_id(msg: dict) -> str | None:
    msg_type = msg.get("type")
    if msg_type == "interactive":
        interactive = msg.get("interactive") or {}
        if interactive.get("type") == "button_reply":
            return (interactive.get("button_reply") or {}).get("id")
        if interactive.get("type") == "list_reply":
            return (interactive.get("list_reply") or {}).get("id")
    if msg_type == "button":
        return (msg.get("button") or {}).get("payload")
    return None


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
