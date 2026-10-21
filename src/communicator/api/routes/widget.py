from __future__ import annotations

import logging
import uuid
from datetime import UTC, datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.inbox import Inbox
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/widget", tags=["Website Live Chat Widget API"])


class WidgetContactIn(BaseModel):
    website_token: str
    email: str
    name: str = ""
    phone: str = ""
    identifier_hash: str | None = None
    custom_attributes: dict[str, Any] | None = None


class WidgetMessageIn(BaseModel):
    conversation_id: uuid.UUID
    body: str


class WidgetPresenceIn(BaseModel):
    conversation_id: uuid.UUID
    status: str = "online"  # online, typing, left
    page_url: str = ""


@router.get("/config")
async def get_widget_config(
    token: str = Query(..., description="Website live chat widget token"),
    session: AsyncSession = Depends(get_db_session),
):
    """
    Public configuration endpoint for the embeddable chat widget script.
    Returns brand styling, greeting titles, pre-chat form requirements, and presence.
    """
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.WEBSITE)
    inboxes = (await session.execute(stmt)).scalars().all()

    target_inbox: Inbox | None = None
    for i in inboxes:
        creds = i.credentials or {}
        if creds.get("websiteToken") == token:
            target_inbox = i
            break

    if not target_inbox:
        if inboxes:
            target_inbox = inboxes[0]
        else:
            raise HTTPException(status_code=404, detail="Widget token not found")

    creds = target_inbox.credentials or {}

    return {
        "inboxId": str(target_inbox.id),
        "name": target_inbox.name,
        "websiteToken": creds.get("websiteToken", token),
        "websiteDomain": creds.get("websiteDomain", "bayworks.com"),
        "widgetColor": creds.get("widgetColor", "#1570EF"),
        "greetingTitle": creds.get("greetingTitle", "Hi there! 👋"),
        "greetingTagline": creds.get("greetingTagline", "We typically reply in a few minutes."),
        "widgetPosition": creds.get("widgetPosition", "right"),
        "preChatFormEnabled": creds.get("preChatFormEnabled", True),
        "preChatFields": creds.get("preChatFields", [
            {"name": "fullName", "label": "Full Name", "required": True},
            {"name": "email", "label": "Email Address", "required": True},
            {"name": "phone", "label": "Phone Number", "required": False},
        ]),
        "greetingEnabled": target_inbox.greeting_enabled,
        "greetingMessage": target_inbox.greeting_message,
        "avatarUrl": target_inbox.avatar_url,
        "isOnline": True,
        "csatEnabled": target_inbox.csat_enabled,
    }


@router.post("/contact")
async def register_widget_contact(
    payload: WidgetContactIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Registers or updates a website visitor profile and creates an active conversation.
    """
    # Find matching Website Inbox
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.WEBSITE)
    inbox = (await session.execute(stmt)).scalars().first()

    # Look for existing contact by email/value
    val = payload.email.strip().lower()
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.WEBSITE, Contact.value == val
    )
    contact = (await session.execute(contact_stmt)).scalar_one_or_none()

    if not contact:
        # Create Lead container for visitor
        lead = Lead(
            harvester_institution_id=f"web_vis_{uuid.uuid4().hex[:12]}",
            name=payload.name or payload.email,
            institution_type="Website Visitor",
            city="Online",
        )
        session.add(lead)
        await session.flush()

        contact = Contact(
            lead_id=lead.id,
            channel=ChannelType.WEBSITE,
            value=val,
            person_name=payload.name or "Website Visitor",
            custom_fields=payload.custom_attributes or {},
        )
        session.add(contact)
        await session.flush()
        await record_consent(
            session,
            contact.id,
            ConsentStatus.OPTED_IN,
            source="website_live_chat",
            note="Visitor initiated live chat session",
        )
    else:
        if payload.name:
            contact.person_name = payload.name
        if payload.custom_attributes:
            cf = dict(contact.custom_fields or {})
            cf.update(payload.custom_attributes)
            contact.custom_fields = cf

    # Find or create active conversation
    conv_stmt = (
        select(Conversation)
        .where(Conversation.contact_id == contact.id)
        .order_by(Conversation.created_at.desc())
    )
    conv = (await session.execute(conv_stmt)).scalars().first()

    if not conv:
        conv = Conversation(
            contact_id=contact.id,
            inbox_id=inbox.id if inbox else None,
            priority="medium",
            custom_attributes={
                "source": "website_live_chat",
                "visitor_email": contact.value,
                **(payload.custom_attributes or {}),
            },
        )
        session.add(conv)
        await session.flush()

        # If inbox greeting is enabled, auto-inject initial greeting
        if inbox and inbox.greeting_enabled and inbox.greeting_message:
            greeting_msg = Message(
                conversation_id=conv.id,
                direction=MessageDirection.OUTBOUND,
                channel=ChannelType.WEBSITE,
                body=inbox.greeting_message,
                provider="web_widget",
                provider_message_id=f"greet_{uuid.uuid4().hex[:12]}",
                status=MessageStatus.DELIVERED,
                sender_type="agent",
                sender_name=inbox.name,
                sent_at=datetime.now(UTC),
            )
            session.add(greeting_msg)
            await session.flush()

    return {
        "contactId": str(contact.id),
        "conversationId": str(conv.id),
        "name": contact.person_name,
        "email": contact.value,
    }


@router.post("/messages")
async def send_widget_message(
    payload: WidgetMessageIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Submits an inbound message from a website visitor into the Shared Inbox.
    """
    conv = await session.get(Conversation, payload.conversation_id)
    if not conv:
        raise HTTPException(status_code=404, detail="Conversation not found")

    contact = await session.get(Contact, conv.contact_id)
    now = datetime.now(UTC)

    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.WEBSITE,
        body=payload.body,
        provider="web_widget",
        provider_message_id=f"in_web_{uuid.uuid4().hex[:12]}",
        status=MessageStatus.DELIVERED,
        sender_type="customer",
        sender_name=contact.person_name if contact else "Visitor",
        sent_at=now,
    )
    session.add(msg)
    conv.last_inbound_at = now
    conv.updated_at = now
    await session.flush()

    return {
        "id": str(msg.id),
        "conversationId": str(conv.id),
        "body": msg.body,
        "sentAt": msg.sent_at.isoformat(),
        "senderType": "customer",
    }


@router.get("/conversations/{conversation_id}/messages")
async def get_widget_conversation_messages(
    conversation_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
):
    """Returns messages for a website visitor session (omits internal private notes)."""
    stmt = (
        select(Message)
        .where(
            Message.conversation_id == conversation_id,
            Message.is_private == False,  # Never expose internal team private notes to visitors
        )
        .order_by(Message.sent_at.asc())
    )
    messages = (await session.execute(stmt)).scalars().all()
    return [
        {
            "id": str(m.id),
            "body": m.body,
            "direction": m.direction.value,
            "senderType": m.sender_type,
            "senderName": m.sender_name or ("Agent" if m.direction == MessageDirection.OUTBOUND else "You"),
            "sentAt": m.sent_at.isoformat(),
        }
        for m in messages
    ]


@router.post("/presence")
async def update_widget_presence(
    payload: WidgetPresenceIn,
    session: AsyncSession = Depends(get_db_session),
):
    """Heartbeat indicating visitor is actively viewing the website or typing."""
    conv = await session.get(Conversation, payload.conversation_id)
    if conv:
        attrs = dict(conv.custom_attributes or {})
        attrs["visitor_presence"] = payload.status
        if payload.page_url:
            attrs["last_page_url"] = payload.page_url
        attrs["last_presence_at"] = datetime.now(UTC).isoformat()
        conv.custom_attributes = attrs
        await session.flush()

    return {"status": "ok", "agentOnline": True}
