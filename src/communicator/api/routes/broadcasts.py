"""
Bulk template-message sends to every Contact carrying a given Tag --
Brevo/Wati-style campaigns. Dispatch is a POST endpoint meant to be hit
manually or by an external cron, the same pattern as
communicator.api.routes.scheduler; see that module's docstring.
"""

from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.channels.whatsapp import MetaWhatsAppProvider
from communicator.compliance.optout import is_sendable
from communicator.models.broadcast import (
    Broadcast,
    BroadcastRecipient,
    BroadcastStatus,
    RecipientStatus,
)
from communicator.models.contact import Contact
from communicator.services.integration_settings import get_whatsapp_config

router = APIRouter(prefix="/broadcasts", tags=["Broadcasts"])


def _serialize(b: Broadcast) -> dict:
    return {
        "id": str(b.id),
        "name": b.name,
        "templateName": b.template_name,
        "templateLanguage": b.template_language,
        "tagName": b.tag_name,
        "status": b.status.value,
        "scheduledAt": b.scheduled_at.isoformat() if b.scheduled_at else None,
        "sentCount": b.sent_count,
        "deliveredCount": b.delivered_count,
        "readCount": b.read_count,
        "failedCount": b.failed_count,
        "createdAt": b.created_at.isoformat(),
    }


class BroadcastIn(BaseModel):
    name: str
    template_name: str
    template_language: str = "en_US"
    body_preview: str = ""
    tag_name: str


@router.get("")
async def list_broadcasts(session: AsyncSession = Depends(get_db_session)):
    broadcasts = (
        (await session.execute(select(Broadcast).order_by(Broadcast.created_at.desc())))
        .scalars()
        .all()
    )
    return [_serialize(b) for b in broadcasts]


@router.post("")
async def create_broadcast(payload: BroadcastIn, session: AsyncSession = Depends(get_db_session)):
    broadcast = Broadcast(**payload.model_dump())
    session.add(broadcast)
    await session.flush()
    return _serialize(broadcast)


@router.get("/{broadcast_id}/recipients")
async def list_recipients(broadcast_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    stmt = select(BroadcastRecipient).where(BroadcastRecipient.broadcast_id == broadcast_id)
    recipients = (await session.execute(stmt)).scalars().all()
    return [
        {
            "id": str(r.id),
            "contactId": str(r.contact_id),
            "status": r.status.value,
            "error": r.error,
        }
        for r in recipients
    ]


@router.post("/{broadcast_id}/dispatch")
async def dispatch_broadcast(
    broadcast_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    broadcast = await session.get(Broadcast, broadcast_id)
    if broadcast is None:
        raise HTTPException(status_code=404, detail="Broadcast not found")
    if broadcast.status == BroadcastStatus.SENDING:
        raise HTTPException(status_code=409, detail="Broadcast already sending.")

    config = await get_whatsapp_config(session)
    if not (config.access_token and config.phone_number_id):
        raise HTTPException(status_code=400, detail="WhatsApp isn't connected yet.")
    provider = MetaWhatsAppProvider(
        access_token=config.access_token, phone_number_id=config.phone_number_id
    )

    stmt = select(Contact).where(Contact.tags.any(name=broadcast.tag_name))
    contacts = (await session.execute(stmt)).scalars().all()

    broadcast.status = BroadcastStatus.SENDING
    await session.flush()

    for contact in contacts:
        recipient = BroadcastRecipient(broadcast_id=broadcast.id, contact_id=contact.id)
        session.add(recipient)

        if not await is_sendable(session, contact.id):
            recipient.status = RecipientStatus.FAILED
            recipient.error = "Contact is not opted in."
            broadcast.failed_count += 1
            continue

        try:
            result = await provider.send_template(
                to=contact.value,
                template_name=broadcast.template_name,
                language=broadcast.template_language,
            )
        except Exception as exc:  # noqa: BLE001 -- one recipient's failure must not stop the batch
            recipient.status = RecipientStatus.FAILED
            recipient.error = str(exc)
            broadcast.failed_count += 1
            continue

        recipient.status = RecipientStatus.SENT
        recipient.provider_message_id = result.provider_message_id
        broadcast.sent_count += 1

    broadcast.status = BroadcastStatus.COMPLETED
    await session.flush()

    return _serialize(broadcast)
