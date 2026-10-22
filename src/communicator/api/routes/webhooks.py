"""
Inbound message webhooks for each channel provider. Twilio requests are
verified via X-Twilio-Signature (see communicator.webhooks.twilio_signature);
Postmark doesn't sign inbound webhook requests, so its URL carries a
shared-secret query token instead. A message from a contact we don't
recognize is acknowledged with 200 and dropped -- there's nothing in our
data model to attach it to, and providers retry on non-2xx responses.
"""

from __future__ import annotations

import base64
import hashlib
import hmac
import re
from datetime import UTC, datetime
from typing import Any

from fastapi import APIRouter, Depends, Header, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_harvester_client
from communicator.compliance.optout import record_consent
from communicator.config import settings
from communicator.harvester_client import HarvesterClient
from communicator.models.broadcast import Broadcast, BroadcastRecipient, RecipientStatus
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.inbox import Inbox
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
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
        except Exception:  # noqa: BLE001, S110
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
            if (
                err_code == 131026
                or "not on whatsapp" in err_title.lower()
                or "not on whatsapp" in err_details.lower()
            ):
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
            if meta_status == "delivered" and old_status not in (
                RecipientStatus.DELIVERED,
                RecipientStatus.READ,
            ):
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




@router.post("/ecosystem/events")
async def ecosystem_event_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    x_newax_source: str | None = Header(default=None),
    x_tenant_id: str | None = Header(default=None),
) -> dict[str, str]:
    """
    Receives cross-repo domain events from core-newax Event Bus / erp-1
    (ECOSYSTEM.md, 255-ecosystem-alignment-roadmap.md).
    Enables Communicator to handle customer engagement, order notifications,
    and support triggers without direct HTTP coupling to other repos.
    """
    try:
        payload = await request.json()
    except Exception:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Malformed JSON payload.")

    if not isinstance(payload, dict):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Payload must be a JSON object.")

    event_name = payload.get("name")
    if not event_name:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Missing required 'name' in event payload.")

    source = x_newax_source or payload.get("source") or "ecosystem"
    tenant_id = x_tenant_id or payload.get("tenantId") or "default"
    event_data = payload.get("payload") if isinstance(payload.get("payload"), dict) else payload

    # Dispatch to Communicator's native workflows
    from communicator.api.routes.crypto_mesh import _route_ecosystem_event
    action = await _route_ecosystem_event(session, event_name, event_data, str(tenant_id))

    return {
        "status": "processed",
        "event": str(event_name),
        "source": str(source),
        "tenant_id": str(tenant_id),
        "action": action,
    }


async def _get_or_create_social_contact(
    session: AsyncSession, channel: ChannelType, sender_id: str, sender_name: str | None = None
) -> Contact:
    stmt = select(Contact).where(Contact.channel == channel, Contact.value == sender_id)
    contact = (await session.execute(stmt)).scalar_one_or_none()
    if contact is None:
        title = channel.value.title()
        lead = Lead(
            harvester_institution_id=f"{channel.value}_{sender_id[:16]}",
            name=sender_name
            or f"{title} Contact {sender_id[-4:] if len(sender_id) > 4 else sender_id}",
            institution_type=f"{title} Contact",
            city="Social Media",
        )
        session.add(lead)
        await session.flush()
        contact = Contact(
            lead_id=lead.id,
            channel=channel,
            value=sender_id,
            person_name=sender_name or f"{title} User",
        )
        session.add(contact)
        await session.flush()
        await record_consent(
            session, contact.id, ConsentStatus.OPTED_IN, source=f"{channel.value}_webhook_inbound"
        )
    return contact


@router.get("/meta/facebook")
async def meta_facebook_verify(
    hub_mode: str | None = Query(default=None, alias="hub.mode"),
    hub_verify_token: str | None = Query(default=None, alias="hub.verify_token"),
    hub_challenge: str | None = Query(default=None, alias="hub.challenge"),
    session: AsyncSession = Depends(get_db_session),
) -> Response:
    """Meta Webhook Verification Handshake for Facebook Messenger."""
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.FACEBOOK)
    inbox = (await session.execute(stmt)).scalars().first()
    expected_token = None
    if inbox and inbox.credentials:
        expected_token = inbox.credentials.get("webhookVerifyToken")
    if not expected_token:
        expected_token = "meta_facebook_verify_token"

    if hub_mode == "subscribe" and hub_verify_token == expected_token:
        return Response(content=hub_challenge or "", media_type="text/plain")
    raise HTTPException(status_code=403, detail="Verification token mismatch")


@router.post("/meta/facebook")
async def meta_facebook_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_hub_signature_256: str | None = Header(default=None),
) -> dict[str, str]:
    """Inbound webhook event processing for Facebook Messenger."""
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.FACEBOOK)
    inbox = (await session.execute(stmt)).scalars().first()
    app_secret = inbox.credentials.get("appSecret") if (inbox and inbox.credentials) else None

    raw_body = await request.body()
    if (
        app_secret
        and not app_secret.startswith("•")
        and not app_secret.startswith("mock")
        and x_hub_signature_256
        and not validate_meta_signature(app_secret, raw_body, x_hub_signature_256)
    ):
        raise HTTPException(status_code=403, detail="Invalid Meta Facebook signature.")

    body = await request.json()
    if body.get("object") not in ("page", "facebook"):
        return {"status": "ignored"}

    for entry in body.get("entry", []):
        for messaging_event in entry.get("messaging", []):
            sender = messaging_event.get("sender", {})
            sender_id = str(sender.get("id", ""))
            if not sender_id:
                continue

            msg = messaging_event.get("message")
            if msg:
                mid = msg.get("mid", "")
                text = msg.get("text", "")
                quick_reply = msg.get("quick_reply", {})
                payload_id = quick_reply.get("payload") if quick_reply else None
                if not text and payload_id:
                    text = payload_id
                elif not text and msg.get("attachments"):
                    text = "[Attachment received]"

                if not text:
                    continue

                await _get_or_create_social_contact(session, ChannelType.FACEBOOK, sender_id)
                try:
                    await process_inbound_message(
                        session,
                        channel=ChannelType.FACEBOOK,
                        from_value=sender_id,
                        body=text,
                        provider="facebook",
                        provider_message_id=mid,
                        harvester_client=harvester_client,
                        interactive_payload_id=payload_id,
                    )
                except UnknownContactError:
                    pass

    return {"status": "ok"}


@router.get("/meta/instagram")
async def meta_instagram_verify(
    hub_mode: str | None = Query(default=None, alias="hub.mode"),
    hub_verify_token: str | None = Query(default=None, alias="hub.verify_token"),
    hub_challenge: str | None = Query(default=None, alias="hub.challenge"),
    session: AsyncSession = Depends(get_db_session),
) -> Response:
    """Meta Webhook Verification Handshake for Instagram Direct Messages."""
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.INSTAGRAM)
    inbox = (await session.execute(stmt)).scalars().first()
    expected_token = None
    if inbox and inbox.credentials:
        expected_token = inbox.credentials.get("webhookVerifyToken")
    if not expected_token:
        expected_token = "meta_instagram_verify_token"

    if hub_mode == "subscribe" and hub_verify_token == expected_token:
        return Response(content=hub_challenge or "", media_type="text/plain")
    raise HTTPException(status_code=403, detail="Verification token mismatch")


@router.post("/meta/instagram")
async def meta_instagram_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_hub_signature_256: str | None = Header(default=None),
) -> dict[str, str]:
    """Inbound webhook event processing for Instagram Direct Messages & Story mentions."""
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.INSTAGRAM)
    inbox = (await session.execute(stmt)).scalars().first()
    app_secret = inbox.credentials.get("appSecret") if (inbox and inbox.credentials) else None

    raw_body = await request.body()
    if (
        app_secret
        and not app_secret.startswith("•")
        and not app_secret.startswith("mock")
        and x_hub_signature_256
        and not validate_meta_signature(app_secret, raw_body, x_hub_signature_256)
    ):
        raise HTTPException(status_code=403, detail="Invalid Meta Instagram signature.")

    body = await request.json()
    if body.get("object") not in ("instagram", "page"):
        return {"status": "ignored"}

    for entry in body.get("entry", []):
        for messaging_event in entry.get("messaging", []):
            sender = messaging_event.get("sender", {})
            sender_id = str(sender.get("id", ""))
            if not sender_id:
                continue

            msg = messaging_event.get("message")
            if msg:
                mid = msg.get("mid", "")
                text = msg.get("text", "")
                quick_reply = msg.get("quick_reply", {})
                payload_id = quick_reply.get("payload") if quick_reply else None
                if not text and payload_id:
                    text = payload_id
                elif not text and msg.get("attachments"):
                    text = "[Attachment or Story mention received]"

                if not text:
                    continue

                await _get_or_create_social_contact(session, ChannelType.INSTAGRAM, sender_id)
                try:
                    await process_inbound_message(
                        session,
                        channel=ChannelType.INSTAGRAM,
                        from_value=sender_id,
                        body=text,
                        provider="instagram",
                        provider_message_id=mid,
                        harvester_client=harvester_client,
                        interactive_payload_id=payload_id,
                    )
                except UnknownContactError:
                    pass

    return {"status": "ok"}


@router.post("/telegram")
async def telegram_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_telegram_bot_api_secret_token: str | None = Header(default=None),
) -> dict[str, str]:
    """Inbound webhook event processing for Telegram Bot API."""
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.TELEGRAM)
    inbox = (await session.execute(stmt)).scalars().first()
    expected_secret = (
        inbox.credentials.get("secretToken") if (inbox and inbox.credentials) else None
    )

    if (
        expected_secret
        and x_telegram_bot_api_secret_token
        and not hmac.compare_digest(expected_secret, x_telegram_bot_api_secret_token)
    ):
        raise HTTPException(status_code=403, detail="Invalid Telegram secret token.")

    payload = await request.json()

    # 1. Standard text message
    msg = payload.get("message")
    if msg:
        chat = msg.get("chat", {})
        sender_id = str(chat.get("id", ""))
        from_user = msg.get("from", {})
        sender_name = (
            from_user.get("first_name")
            or from_user.get("username")
            or f"Telegram User {sender_id[-4:] if len(sender_id) > 4 else sender_id}"
        )
        mid = str(msg.get("message_id", ""))
        text = msg.get("text", "")

        if sender_id and text:
            await _get_or_create_social_contact(
                session, ChannelType.TELEGRAM, sender_id, sender_name
            )
            try:
                await process_inbound_message(
                    session,
                    channel=ChannelType.TELEGRAM,
                    from_value=sender_id,
                    body=text,
                    provider="telegram",
                    provider_message_id=f"tg_{mid}",
                    harvester_client=harvester_client,
                )
            except UnknownContactError:
                pass

    # 2. Callback query (inline button clicks)
    cb = payload.get("callback_query")
    if cb:
        cb_msg = cb.get("message", {})
        chat = cb_msg.get("chat", {})
        sender_id = str(chat.get("id", ""))
        from_user = cb.get("from", {})
        sender_name = (
            from_user.get("first_name")
            or from_user.get("username")
            or f"Telegram User {sender_id[-4:] if len(sender_id) > 4 else sender_id}"
        )
        data = cb.get("data", "")
        cb_id = str(cb.get("id", ""))

        if sender_id and data:
            await _get_or_create_social_contact(
                session, ChannelType.TELEGRAM, sender_id, sender_name
            )
            try:
                await process_inbound_message(
                    session,
                    channel=ChannelType.TELEGRAM,
                    from_value=sender_id,
                    body=data,
                    provider="telegram",
                    provider_message_id=f"tg_cb_{cb_id}",
                    harvester_client=harvester_client,
                    interactive_payload_id=data,
                )
            except UnknownContactError:
                pass

    return {"status": "ok"}


def _validate_line_signature(
    channel_secret: str, body: bytes, signature_header: str | None
) -> bool:
    if not signature_header:
        return False
    hash_val = hmac.new(channel_secret.encode("utf-8"), body, hashlib.sha256).digest()
    expected = base64.b64encode(hash_val).decode("utf-8")
    return hmac.compare_digest(expected, signature_header)


@router.post("/line")
async def line_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_line_signature: str | None = Header(default=None),
) -> dict[str, str]:
    """Inbound webhook event processing for LINE Messaging API."""
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.LINE)
    inbox = (await session.execute(stmt)).scalars().first()
    channel_secret = (
        inbox.credentials.get("channelSecret") if (inbox and inbox.credentials) else None
    )

    raw_body = await request.body()
    if (
        channel_secret
        and not channel_secret.startswith("•")
        and not channel_secret.startswith("mock")
        and x_line_signature
    ) and not _validate_line_signature(channel_secret, raw_body, x_line_signature):
        raise HTTPException(status_code=403, detail="Invalid LINE webhook signature.")

    payload = await request.json()
    events = payload.get("events", [])

    for event in events:
        source = event.get("source", {})
        user_id = str(source.get("userId", ""))
        if not user_id:
            continue

        event_type = event.get("type")
        text = ""
        mid = ""
        payload_id = None

        if event_type == "message":
            msg = event.get("message", {})
            mid = str(msg.get("id", ""))
            text = msg.get("text", "")
            if not text and msg.get("type") != "text":
                text = f"[{msg.get('type', 'attachment').capitalize()} received]"
        elif event_type == "postback":
            postback = event.get("postback", {})
            text = postback.get("data", "")
            payload_id = text
            mid = f"line_pb_{event.get('timestamp', '')}"

        if not text:
            continue

        await _get_or_create_social_contact(session, ChannelType.LINE, user_id)
        try:
            await process_inbound_message(
                session,
                channel=ChannelType.LINE,
                from_value=user_id,
                body=text,
                provider="line",
                provider_message_id=mid or f"line_msg_{user_id[:8]}",
                harvester_client=harvester_client,
                interactive_payload_id=payload_id,
            )
        except UnknownContactError:
            pass

    return {"status": "ok"}


@router.post("/voice/inbound")
async def voice_inbound_webhook(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_twilio_signature: str | None = Header(default=None),
):
    """
    Inbound Voice Call webhook for Twilio Voice and WebRTC.
    Handles incoming phone calls, generates TwiML responses for IVR / direct agent queues,
    and auto-provisions Contact, Conversation, and Call records.
    """
    content_type = request.headers.get("content-type", "")
    if "application/json" in content_type:
        data = await request.json()
    else:
        form = await request.form()
        data = dict(form)
        if (
            x_twilio_signature
            and settings.twilio_auth_token
            and not settings.twilio_auth_token.startswith("mock")
            and not validate_twilio_signature(
                settings.twilio_auth_token, str(request.url), data, x_twilio_signature
            )
        ):
            raise HTTPException(status_code=403, detail="Invalid Twilio signature.")

    call_sid = str(
        data.get("CallSid")
        or data.get("call_sid")
        or f"CA{datetime.now().strftime('%Y%m%d%H%M%S')}"
    )
    from_number = str(data.get("From") or data.get("from") or "+14155550100")
    to_number = str(data.get("To") or data.get("to") or "+14155550199")
    digits = data.get("Digits") or data.get("digits")

    # Fetch voice inbox settings
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.VOICE)
    inbox = (await session.execute(stmt)).scalars().first()
    creds = inbox.credentials if (inbox and inbox.credentials) else {}
    ivr_enabled = creds.get("ivrEnabled", True)
    ivr_prompt = creds.get(
        "ivrPrompt",
        "Thank you for calling Bay Works. Press 1 for Sales, 2 for Technical Support, or 3 to leave a Voicemail.",
    )

    # Locate or create contact & lead
    contact = await _get_or_create_social_contact(
        session, ChannelType.VOICE, from_number, f"Caller {from_number[-4:]}"
    )

    # Locate or create active conversation
    conv_stmt = (
        select(Conversation)
        .where(
            Conversation.contact_id == contact.id,
            Conversation.inbox_id == (inbox.id if inbox else None),
        )
        .order_by(Conversation.created_at.desc())
    )
    conv = (await session.execute(conv_stmt)).scalars().first()

    if conv is None or conv.state in (ConversationState.CLOSED_WON, ConversationState.CLOSED_LOST):
        conv = Conversation(
            contact_id=contact.id,
            inbox_id=inbox.id if inbox else None,
            state=ConversationState.ENGAGED,
        )
        session.add(conv)
        await session.flush()

    # Route based on IVR digits
    if str(digits) == "1":
        action_note = "Selected option [1: Sales] · Routing to Sales queue"
        twiml = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            "<Response>\n"
            '  <Say voice="Polly.Joanna">Connecting you to our Sales team.</Say>\n'
            '  <Dial timeout="30" record="record-from-answer-dual" action="/api/v1/webhooks/voice/status">\n'
            "    <Client>sales-queue</Client>\n"
            "  </Dial>\n"
            "</Response>"
        )
    elif str(digits) == "2":
        action_note = "Selected option [2: Support] · Routing to Technical Support"
        twiml = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            "<Response>\n"
            '  <Say voice="Polly.Joanna">Connecting you to Technical Support.</Say>\n'
            '  <Dial timeout="30" record="record-from-answer-dual" action="/api/v1/webhooks/voice/status">\n'
            "    <Client>support-queue</Client>\n"
            "  </Dial>\n"
            "</Response>"
        )
    elif str(digits) == "3":
        action_note = "Selected option [3: Voicemail] · Recording voicemail"
        twiml = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            "<Response>\n"
            '  <Say voice="Polly.Joanna">Please leave your message after the tone. Press pound when finished.</Say>\n'
            '  <Record maxLength="120" action="/api/v1/webhooks/voice/recording" transcribe="true" transcribeCallback="/api/v1/webhooks/voice/transcription"/>\n'
            "</Response>"
        )
    elif ivr_enabled and not digits:
        action_note = "Ringing · IVR Menu presented"
        twiml = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            "<Response>\n"
            '  <Gather action="/api/v1/webhooks/voice/inbound" numDigits="1" timeout="10">\n'
            f'    <Say voice="Polly.Joanna">{ivr_prompt}</Say>\n'
            "  </Gather>\n"
            '  <Say voice="Polly.Joanna">We did not receive any input. Connecting you to an agent.</Say>\n'
            '  <Dial timeout="20" record="record-from-answer-dual" action="/api/v1/webhooks/voice/status">\n'
            "    <Client>agent-inbox</Client>\n"
            "  </Dial>\n"
            "</Response>"
        )
    else:
        action_note = "Direct Dial · Connecting to agent inbox"
        twiml = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            "<Response>\n"
            '  <Say voice="Polly.Joanna">Connecting your call to our support team.</Say>\n'
            '  <Dial timeout="30" record="record-from-answer-dual" action="/api/v1/webhooks/voice/status">\n'
            "    <Client>agent-inbox</Client>\n"
            "  </Dial>\n"
            "</Response>"
        )

    # Record message in conversation thread
    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.VOICE,
        body=f"📞 Inbound Call from {from_number} ({action_note})",
        provider="twilio_voice",
        provider_message_id=call_sid,
        status=MessageStatus.RECEIVED,
        provider_metadata={
            "call_sid": call_sid,
            "from": from_number,
            "to": to_number,
            "digits": digits,
            "action_note": action_note,
            "call_status": "ringing",
            "direction": "inbound",
        },
    )
    session.add(msg)
    await session.flush()

    return Response(content=twiml, media_type="application/xml")


@router.post("/voice/status")
async def voice_status_webhook(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Call status callback: receives call progress updates (ringing, in-progress, completed, busy, no-answer).
    Updates duration, call metadata, and conversation records.
    """
    content_type = request.headers.get("content-type", "")
    if "application/json" in content_type:
        data = await request.json()
    else:
        form = await request.form()
        data = dict(form)

    call_sid = str(data.get("CallSid") or data.get("call_sid") or "")
    call_status = str(data.get("CallStatus") or data.get("call_status") or "completed")
    duration = int(data.get("CallDuration") or data.get("duration") or 0)

    if call_sid:
        stmt = select(Message).where(Message.provider_message_id == call_sid)
        msg = (await session.execute(stmt)).scalars().first()
        if msg:
            meta = dict(msg.provider_metadata or {})
            meta["call_status"] = call_status
            meta["duration"] = duration
            meta["ended_at"] = datetime.now(UTC).isoformat()
            msg.provider_metadata = meta
            if call_status == "completed":
                msg.body += f" · Call completed ({duration}s)"
            await session.flush()

    return {"status": "ok", "call_sid": call_sid, "call_status": call_status, "duration": duration}


@router.post("/voice/recording")
async def voice_recording_webhook(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Call recording callback: receives audio recording URL and duration.
    Attaches recording to the conversation message.
    """
    content_type = request.headers.get("content-type", "")
    if "application/json" in content_type:
        data = await request.json()
    else:
        form = await request.form()
        data = dict(form)

    call_sid = str(data.get("CallSid") or data.get("call_sid") or "")
    recording_url = str(data.get("RecordingUrl") or data.get("recording_url") or "")
    recording_duration = int(data.get("RecordingDuration") or data.get("recording_duration") or 0)
    recording_sid = str(data.get("RecordingSid") or data.get("recording_sid") or "")

    if call_sid:
        stmt = select(Message).where(Message.provider_message_id == call_sid)
        msg = (await session.execute(stmt)).scalars().first()
        if msg:
            meta = dict(msg.provider_metadata or {})
            meta["recording_url"] = recording_url
            meta["recording_duration"] = recording_duration
            meta["recording_sid"] = recording_sid
            msg.provider_metadata = meta
            msg.body += f" · 🎙️ Recording ({recording_duration}s)"
            await session.flush()

    return {
        "status": "ok",
        "call_sid": call_sid,
        "recording_url": recording_url,
        "recording_duration": recording_duration,
    }


@router.post("/voice/transcription")
async def voice_transcription_webhook(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Call transcription callback: receives speech-to-text transcript.
    Attaches transcript to conversation message.
    """
    content_type = request.headers.get("content-type", "")
    if "application/json" in content_type:
        data = await request.json()
    else:
        form = await request.form()
        data = dict(form)

    call_sid = str(data.get("CallSid") or data.get("call_sid") or "")
    transcription_text = str(data.get("TranscriptionText") or data.get("transcription_text") or "")
    transcription_status = str(
        data.get("TranscriptionStatus") or data.get("transcription_status") or "completed"
    )

    if call_sid and transcription_text:
        stmt = select(Message).where(Message.provider_message_id == call_sid)
        msg = (await session.execute(stmt)).scalars().first()
        if msg:
            meta = dict(msg.provider_metadata or {})
            meta["transcription_text"] = transcription_text
            meta["transcription_status"] = transcription_status
            msg.provider_metadata = meta
            msg.body += f'\n📝 Transcript: "{transcription_text}"'
            await session.flush()

    return {"status": "ok", "call_sid": call_sid, "transcription_status": transcription_status}


def _clean_email_body(body: str) -> str:
    """Strips common email quotation headers and signatures."""
    if not body:
        return ""
    patterns = [
        r"(?i)\n\s*On\s+.+?\s+wrote:\s*\n.*",
        r"(?i)\n\s*---\s*Original Message\s*---\s*\n.*",
        r"(?i)\n\s*From:\s+.+?\nSent:\s+.+?\n.*",
        r"(?i)\n\s*_{10,}\s*\n.*",
    ]
    cleaned = body
    for pat in patterns:
        cleaned = re.sub(pat, "", cleaned, flags=re.DOTALL)
    return cleaned.strip()


@router.post("/email/inbound")
async def email_inbound_webhook(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
):
    """
    Inbound Email Webhook.
    Receives incoming customer emails (from Postmark, SendGrid Inbound Parse, Cloudflare,
    or custom mailer daemon), parses sender details, strips quoted replies,
    auto-provisions Contact & Conversation thread, and records incoming message.
    """
    content_type = request.headers.get("content-type", "")
    if "application/json" in content_type:
        payload = await request.json()
    else:
        form = await request.form()
        payload = dict(form)

    # 1. Parse sender email and name
    raw_from = str(payload.get("From") or payload.get("from") or payload.get("sender") or "")
    from_name = str(payload.get("FromName") or payload.get("from_name") or "")

    if "<" in raw_from and ">" in raw_from:
        parts = raw_from.split("<", 1)
        if not from_name:
            from_name = parts[0].strip(" \"'")
        sender_email = parts[1].split(">", 1)[0].strip().lower()
    else:
        sender_email = raw_from.strip().lower()

    if not sender_email or "@" not in sender_email:
        sender_email = "inbound-sender@example.com"
    if not from_name:
        from_name = sender_email.split("@")[0].replace(".", " ").title()

    subject = str(payload.get("Subject") or payload.get("subject") or "Support Request")
    text_body = str(payload.get("TextBody") or payload.get("text") or payload.get("body") or "")
    html_body = str(payload.get("HtmlBody") or payload.get("html") or "")
    message_id = str(
        payload.get("MessageID")
        or payload.get("message_id")
        or payload.get("headers", {}).get(
            "Message-ID", f"<{datetime.now().strftime('%Y%m%d%H%M%S')}@customer-email>"
        )
    )
    in_reply_to = payload.get("InReplyTo") or payload.get("in_reply_to")
    references = payload.get("References") or payload.get("references")
    attachments = payload.get("Attachments") or payload.get("attachments") or []

    clean_body = _clean_email_body(text_body) or "[Empty email body]"

    # 2. Lookup email inbox
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.EMAIL)
    inbox = (await session.execute(stmt)).scalars().first()

    # 3. Locate or create contact & lead
    contact = await _get_or_create_social_contact(
        session, ChannelType.EMAIL, sender_email, from_name
    )

    # 4. Resolve conversation thread
    conv_stmt = (
        select(Conversation)
        .where(
            Conversation.contact_id == contact.id,
            Conversation.inbox_id == (inbox.id if inbox else None),
        )
        .order_by(Conversation.created_at.desc())
    )
    conv = (await session.execute(conv_stmt)).scalars().first()

    if conv is None or conv.state in (ConversationState.CLOSED_WON, ConversationState.CLOSED_LOST):
        conv = Conversation(
            contact_id=contact.id,
            inbox_id=inbox.id if inbox else None,
            state=ConversationState.ENGAGED,
            custom_attributes={"subject": subject},
        )
        session.add(conv)
        await session.flush()

    # 5. Record message in thread
    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.EMAIL,
        body=f"Subject: {subject}\n\n{clean_body}",
        sender_name=from_name,
        sender_type="customer",
        provider="email",
        provider_message_id=message_id,
        status=MessageStatus.RECEIVED,
        attachments=attachments if isinstance(attachments, list) else [],
        provider_metadata={
            "message_id": message_id,
            "subject": subject,
            "from": sender_email,
            "from_name": from_name,
            "to": str(payload.get("To") or payload.get("to") or "support@bayworks.co"),
            "in_reply_to": in_reply_to,
            "references": references,
            "has_html": bool(html_body),
        },
    )
    session.add(msg)
    await session.flush()

    return {
        "status": "ok",
        "messageId": message_id,
        "conversationId": str(conv.id),
        "from": sender_email,
        "subject": subject,
    }


@router.get("/tiktok")
async def tiktok_webhook_verify(
    challenge: str | None = Query(default=None),
    hub_challenge: str | None = Query(default=None, alias="hub.challenge"),
    echo: str | None = Query(default=None),
) -> Response:
    """
    TikTok Webhook Verification Handshake.
    Returns the challenge string back to TikTok for subscription verification.
    """
    token = challenge or hub_challenge or echo or "tiktok_challenge_ok"
    return Response(content=token, media_type="text/plain")


@router.post("/tiktok")
async def tiktok_webhook_inbound(
    request: Request,
    session: AsyncSession = Depends(get_db_session),
    harvester_client: HarvesterClient = Depends(get_harvester_client),
    x_tiktok_signature: str | None = Header(default=None, alias="X-TikTok-Signature"),
) -> dict[str, Any]:
    """
    Inbound webhook processing for TikTok Business DMs and Video Comment interactions.
    Auto-creates Contact and Conversation threads and records incoming messages.
    """
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.TIKTOK)
    inbox = (await session.execute(stmt)).scalars().first()
    app_secret = inbox.credentials.get("appSecret") if (inbox and inbox.credentials) else None

    raw_body = await request.body()
    if (
        app_secret
        and not app_secret.startswith("•")
        and not app_secret.startswith("mock")
        and x_tiktok_signature
    ):
        from communicator.channels.tiktok import TikTokProvider

        if not TikTokProvider.verify_signature(raw_body, x_tiktok_signature, app_secret):
            raise HTTPException(status_code=403, detail="Invalid TikTok webhook signature.")

    payload = await request.json()
    event_type = str(
        payload.get("event") or payload.get("event_type") or "im.direct_message.receive"
    )
    content = payload.get("content") or payload.get("data") or {}
    user = payload.get("user") or {}

    sender_open_id = str(
        content.get("open_id")
        or content.get("from_user_id")
        or user.get("open_id")
        or payload.get("open_id")
        or payload.get("sender_id")
        or ""
    )
    if not sender_open_id:
        return {"status": "ignored", "reason": "No sender open_id provided"}

    sender_name = (
        user.get("nickname")
        or user.get("display_name")
        or content.get("sender_name")
        or f"TikTok User {sender_open_id[-4:] if len(sender_open_id) > 4 else sender_open_id}"
    )

    text = (
        content.get("text")
        or content.get("body")
        or payload.get("text")
        or payload.get("message")
        or ""
    )

    message_id = str(
        content.get("message_id")
        or content.get("comment_id")
        or payload.get("message_id")
        or f"tt_in_{datetime.now(UTC).strftime('%Y%m%d%H%M%S')}_{sender_open_id[-4:]}"
    )

    is_comment = "comment" in event_type.lower()
    if is_comment:
        video_id = content.get("video_id") or payload.get("video_id") or "v_unknown"
        formatted_text = (
            f"[TikTok Video Comment #{video_id}]: {text}"
            if text
            else f"[TikTok Video Comment #{video_id}]"
        )
    else:
        formatted_text = text or "[TikTok Direct Message]"

    # Resolve or create contact & lead
    await _get_or_create_social_contact(session, ChannelType.TIKTOK, sender_open_id, sender_name)

    # Process inbound message
    try:
        await process_inbound_message(
            session,
            channel=ChannelType.TIKTOK,
            from_value=sender_open_id,
            body=formatted_text,
            provider="tiktok",
            provider_message_id=message_id,
            harvester_client=harvester_client,
        )
    except UnknownContactError:
        pass

    return {
        "status": "ok",
        "event": event_type,
        "messageId": message_id,
        "openId": sender_open_id,
        "isComment": is_comment,
    }
