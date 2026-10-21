"""
Contact & list management: CSV import, tags/segments, custom fields,
and contact memories / remembered preferences ("Remembers").
"""

from __future__ import annotations

import csv
import io
import uuid

from fastapi import APIRouter, Depends, HTTPException, UploadFile
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.api.dependencies import get_db_session
from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.lead import Lead
from communicator.models.tag import Tag
from communicator.services.memory import (
    add_manual_memory,
    delete_contact_memory,
    get_contact_memories,
)
from communicator.services.tags import add_tag_to_contact, get_or_create_tag

router = APIRouter(prefix="/contacts", tags=["Contacts"])


def _serialize(contact: Contact) -> dict:
    return {
        "id": str(contact.id),
        "value": contact.value,
        "personName": contact.person_name,
        "channel": contact.channel.value,
        "language": contact.language,
        "customFields": contact.custom_fields,
        "tags": [t.name for t in contact.tags],
    }


@router.get("")
async def list_contacts(tag: str | None = None, session: AsyncSession = Depends(get_db_session)):
    stmt = select(Contact).options(selectinload(Contact.tags))
    if tag:
        stmt = stmt.join(Contact.tags).where(Contact.tags.any(name=tag))
    contacts = (await session.execute(stmt)).scalars().unique().all()
    return [_serialize(c) for c in contacts]


class TagIn(BaseModel):
    tag: str


@router.post("/{contact_id}/tags")
async def add_tag(
    contact_id: uuid.UUID, payload: TagIn, session: AsyncSession = Depends(get_db_session)
):
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")
    await add_tag_to_contact(session, contact, payload.tag)
    await session.refresh(contact, attribute_names=["tags"])
    return _serialize(contact)


@router.delete("/{contact_id}/tags/{tag_name}")
async def remove_tag(
    contact_id: uuid.UUID, tag_name: str, session: AsyncSession = Depends(get_db_session)
):
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")
    await session.refresh(contact, attribute_names=["tags"])
    contact.tags = [t for t in contact.tags if t.name != tag_name]
    await session.flush()
    return _serialize(contact)


@router.get("/tags")
async def list_tags(session: AsyncSession = Depends(get_db_session)):
    tags = (await session.execute(select(Tag).order_by(Tag.name))).scalars().all()
    return [{"id": str(t.id), "name": t.name} for t in tags]


# --- Contact Memories & Preferences ("Remembers") ---


class MemoryIn(BaseModel):
    fact: str
    category: str = "preference"


@router.get("/{contact_id}/memories")
async def list_memories(
    contact_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")
    memories = await get_contact_memories(session, contact_id)
    return [
        {
            "id": str(m.id),
            "contactId": str(m.contact_id),
            "fact": m.fact,
            "category": m.category,
            "confidence": m.confidence,
            "notedAt": m.noted_at.isoformat(),
            "createdAt": m.created_at.isoformat(),
        }
        for m in memories
    ]


@router.post("/{contact_id}/memories")
async def create_memory(
    contact_id: uuid.UUID, payload: MemoryIn, session: AsyncSession = Depends(get_db_session)
):
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")
    mem = await add_manual_memory(session, contact_id, payload.fact, payload.category)
    return {
        "id": str(mem.id),
        "contactId": str(mem.contact_id),
        "fact": mem.fact,
        "category": mem.category,
        "confidence": mem.confidence,
        "notedAt": mem.noted_at.isoformat(),
    }


@router.delete("/{contact_id}/memories/{memory_id}")
async def delete_memory(
    contact_id: uuid.UUID, memory_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    success = await delete_contact_memory(session, contact_id, memory_id)
    if not success:
        raise HTTPException(status_code=404, detail="Memory not found")
    return {"status": "deleted"}


# --- CSV Import ---


@router.post("/import")
async def import_contacts(file: UploadFile, session: AsyncSession = Depends(get_db_session)):
    """
    CSV columns: name, phone (required, WhatsApp number incl. country code),
    email (optional), tags (optional, comma/pipe separated).
    """
    raw = (await file.read()).decode("utf-8-sig")
    reader = csv.DictReader(io.StringIO(raw))

    created = 0
    skipped = 0
    for row in reader:
        phone = (row.get("phone") or row.get("whatsapp") or "").strip()
        if not phone:
            skipped += 1
            continue
        if not phone.startswith("+"):
            phone = "+" + phone.lstrip("+")

        existing_stmt = select(Contact).where(
            Contact.channel == ChannelType.WHATSAPP, Contact.value == phone
        )
        existing = (await session.execute(existing_stmt)).scalar_one_or_none()
        if existing is not None:
            skipped += 1
            continue

        lead = Lead(
            harvester_institution_id=f"local-import:{uuid.uuid4()}",
            name=(row.get("name") or "").strip(),
            institution_type="imported",
        )
        session.add(lead)
        await session.flush()

        contact = Contact(
            lead_id=lead.id,
            channel=ChannelType.WHATSAPP,
            value=phone,
            person_name=(row.get("name") or "").strip(),
        )
        session.add(contact)
        await session.flush()

        await record_consent(session, contact.id, ConsentStatus.OPTED_IN, source="csv_import")

        tag_field = (row.get("tags") or "").strip()
        if tag_field:
            for tag_name in [
                t.strip() for t in tag_field.replace("|", ",").split(",") if t.strip()
            ]:
                await get_or_create_tag(session, tag_name)
                await add_tag_to_contact(session, contact, tag_name)

        created += 1

    await session.flush()
    return {"created": created, "skipped": skipped}
