"""
Contact & list management: CSV import, tags/segments, custom fields,
and contact memories / remembered preferences ("Remembers").
"""

from __future__ import annotations

import csv
import io
import uuid

from fastapi import APIRouter, Depends, Form, HTTPException, Response, UploadFile
from pydantic import BaseModel
from sqlalchemy import or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.api.dependencies import get_db_session, get_tenant_key
from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.customer import Customer
from communicator.models.lead import Lead
from communicator.models.segment import ContactSegment
from communicator.models.tag import Tag
from communicator.services.memory import (
    add_manual_memory,
    delete_contact_memory,
    get_contact_memories,
)
from communicator.services.relationship_profile import get_relationship_summary
from communicator.services.segment_service import build_segment_filter_stmt
from communicator.services.tags import add_tag_to_contact

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
        "createdAt": contact.created_at.isoformat(),
    }


@router.get("")
async def list_contacts(
    tag: str | None = None,
    channel: str | None = None,
    search: str | None = None,
    segment_id: uuid.UUID | None = None,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """
    Lists contacts with optional filtering by tag, channel, search term, or saved segment.
    Always scoped to the caller's own tenant -- switching organizations in
    the UI must change what this returns.
    """
    if segment_id:
        segment = await session.get(ContactSegment, segment_id)
        if segment is None or segment.tenant_key != tenant_key:
            raise HTTPException(status_code=404, detail="Segment not found")
        stmt = build_segment_filter_stmt(segment.filters or {}, tenant_key=tenant_key)
    else:
        stmt = (
            select(Contact)
            .where(Contact.tenant_key == tenant_key)
            .options(selectinload(Contact.tags))
        )

        if tag:
            stmt = stmt.where(Contact.tags.any(Tag.name == tag))

        if channel:
            try:
                ch_enum = ChannelType(channel.lower())
                stmt = stmt.where(Contact.channel == ch_enum)
            except ValueError:
                pass

        if search and search.strip():
            term = f"%{search.strip()}%"
            stmt = stmt.where(or_(Contact.person_name.ilike(term), Contact.value.ilike(term)))

    contacts = (
        (await session.execute(stmt.order_by(Contact.created_at.desc()))).scalars().unique().all()
    )
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


class LinkCustomerIn(BaseModel):
    customer_id: uuid.UUID


@router.post("/{contact_id}/link-customer")
async def link_contact_to_customer(
    contact_id: uuid.UUID, payload: LinkCustomerIn, session: AsyncSession = Depends(get_db_session)
):
    """
    Explicitly links a channel identity (e.g. this contact's Instagram
    handle) to an existing Customer (e.g. their WhatsApp profile), so the
    relationship is read as one person across channels. Never automatic --
    an agent confirms the match first.
    """
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")

    customer = await session.get(Customer, payload.customer_id)
    if customer is None:
        raise HTTPException(status_code=404, detail="Customer not found")
    if customer.tenant_key != contact.tenant_key:
        raise HTTPException(
            status_code=400, detail="Contact and customer belong to different tenants"
        )

    contact.customer_id = customer.id
    await session.flush()
    return {"status": "linked", "contactId": str(contact.id), "customerId": str(customer.id)}


@router.get("/{contact_id}/relationship-profile")
async def get_contact_relationship_profile(
    contact_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
):
    """
    The customer intelligence profile an agent (human or AI) should load
    before responding -- relationship stage, most recent documented need,
    prior outcomes, and remembered facts, each tagged with its source.
    Not the raw message log; see reply_engine.py for how this feeds the AI.
    """
    contact = await session.get(Contact, contact_id)
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")
    summary = await get_relationship_summary(session, contact_id)
    return {
        "contact": {
            "id": str(contact.id),
            "name": contact.person_name,
            "value": contact.value,
            "channel": contact.channel.value,
            "language": contact.language,
            "firstSeenAt": contact.created_at.isoformat(),
        },
        **summary,
    }


@router.get("/tags")
async def list_tags(session: AsyncSession = Depends(get_db_session)):
    tags = (await session.execute(select(Tag).order_by(Tag.name))).scalars().all()
    return [{"id": str(t.id), "name": t.name} for t in tags]


# --- Contact Memories & Preferences ("Remembers") ---


class MemoryIn(BaseModel):
    fact: str
    category: str = "preference"


@router.get("/{contact_id}/memories")
async def list_memories(contact_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
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


# --- Multi-Channel CSV Import & Export ---


def _resolve_contact_channel_and_value(
    row: dict[str, str], default_channel: str | None = None
) -> tuple[ChannelType | None, str]:
    # 1. Determine raw value
    val = (
        row.get("value")
        or row.get("phone")
        or row.get("whatsapp")
        or row.get("email")
        or row.get("number")
        or row.get("mobile")
        or row.get("handle")
        or row.get("open_id")
        or row.get("openid")
        or row.get("chat_id")
        or ""
    ).strip()

    if not val:
        return None, ""

    # 2. Determine channel type
    raw_ch = (
        (row.get("channel") or row.get("channel_type") or row.get("type") or default_channel or "")
        .strip()
        .lower()
    )

    ch_enum: ChannelType | None = None
    if raw_ch:
        try:
            ch_enum = ChannelType(raw_ch)
        except ValueError:
            pass

    # Auto-detection heuristics if channel was not explicitly recognized
    if ch_enum is None:
        if "@" in val and not val.startswith("@"):
            ch_enum = ChannelType.EMAIL
        elif "tiktok" in raw_ch or val.startswith("tt_"):
            ch_enum = ChannelType.TIKTOK
        elif "telegram" in raw_ch or val.startswith("tg_"):
            ch_enum = ChannelType.TELEGRAM
        elif "line" in raw_ch or val.startswith("U"):
            ch_enum = ChannelType.LINE
        elif "sms" in raw_ch:
            ch_enum = ChannelType.SMS
        elif "facebook" in raw_ch:
            ch_enum = ChannelType.FACEBOOK
        elif "instagram" in raw_ch:
            ch_enum = ChannelType.INSTAGRAM
        else:
            ch_enum = ChannelType.WHATSAPP

    # Normalization for phone-based channels
    if (
        ch_enum in (ChannelType.WHATSAPP, ChannelType.SMS, ChannelType.VOICE)
        and not val.startswith("+")
        and any(c.isdigit() for c in val)
    ):
        val = "+" + val.lstrip("+")

    return ch_enum, val


@router.post("/import")
async def import_contacts(
    file: UploadFile,
    default_channel: str | None = Form(default=None),
    default_tags: str | None = Form(default=None),
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """
    Multi-Channel CSV Contact Import.
    Supports WhatsApp, Email, SMS, TikTok, Telegram, LINE, Facebook, Instagram.
    Creates new contacts or updates existing tags and attributes, scoped to
    the caller's own tenant.
    """
    raw = (await file.read()).decode("utf-8-sig")
    reader = csv.DictReader(io.StringIO(raw))

    created = 0
    updated = 0
    skipped = 0
    total = 0
    channel_counts: dict[str, int] = {}

    extra_tags: list[str] = []
    if default_tags:
        extra_tags = [t.strip() for t in default_tags.replace("|", ",").split(",") if t.strip()]

    for row in reader:
        total += 1
        ch_enum, value = _resolve_contact_channel_and_value(row, default_channel)
        if not ch_enum or not value:
            skipped += 1
            continue

        name = (
            row.get("name")
            or row.get("person_name")
            or row.get("full_name")
            or row.get("display_name")
            or ""
        ).strip()
        language = (row.get("language") or "en").strip()

        # Check existing contact
        existing_stmt = (
            select(Contact)
            .options(selectinload(Contact.tags))
            .where(
                Contact.tenant_key == tenant_key, Contact.channel == ch_enum, Contact.value == value
            )
        )
        existing = (await session.execute(existing_stmt)).scalar_one_or_none()

        row_tags = []
        tag_field = (row.get("tags") or "").strip()
        if tag_field:
            row_tags = [t.strip() for t in tag_field.replace("|", ",").split(",") if t.strip()]
        all_tags = list(set(row_tags + extra_tags))

        ch_key = ch_enum.value
        channel_counts[ch_key] = channel_counts.get(ch_key, 0) + 1

        if existing is not None:
            # Update name if previously empty
            if name and not existing.person_name:
                existing.person_name = name
            # Append new tags
            existing_tag_names = {t.name for t in existing.tags}
            for t_name in all_tags:
                if t_name not in existing_tag_names:
                    await add_tag_to_contact(session, existing, t_name)
            updated += 1
        else:
            lead = Lead(
                harvester_institution_id=f"csv-import:{uuid.uuid4()}",
                name=name or f"{ch_enum.value.title()} Contact",
                institution_type="csv_import",
            )
            session.add(lead)
            await session.flush()

            contact = Contact(
                tenant_key=tenant_key,
                lead_id=lead.id,
                channel=ch_enum,
                value=value,
                person_name=name,
                language=language,
            )
            session.add(contact)
            await session.flush()

            await record_consent(session, contact.id, ConsentStatus.OPTED_IN, source="csv_import")

            for t_name in all_tags:
                await add_tag_to_contact(session, contact, t_name)

            created += 1

    await session.flush()
    return {
        "status": "ok",
        "created": created,
        "updated": updated,
        "skipped": skipped,
        "total": total,
        "channels": channel_counts,
    }


@router.post("/import/preview")
async def preview_import(
    file: UploadFile,
    default_channel: str | None = Form(default=None),
):
    """
    Parses uploaded CSV and returns detected headers, sample mapped contacts,
    and total row estimates before committing to database.
    """
    raw = (await file.read()).decode("utf-8-sig")
    reader = list(csv.DictReader(io.StringIO(raw)))

    total_rows = len(reader)
    detected_columns = list(reader[0].keys()) if reader else []

    sample_rows = []
    detected_channels = set()

    for row in reader[:10]:
        ch_enum, value = _resolve_contact_channel_and_value(row, default_channel)
        if ch_enum:
            detected_channels.add(ch_enum.value)
        name = (row.get("name") or row.get("person_name") or row.get("full_name") or "").strip()
        tags_raw = (row.get("tags") or "").strip()
        tags = [t.strip() for t in tags_raw.replace("|", ",").split(",") if t.strip()]

        sample_rows.append(
            {
                "name": name,
                "channel": ch_enum.value if ch_enum else "unknown",
                "value": value,
                "tags": tags,
                "valid": bool(ch_enum and value),
            }
        )

    return {
        "totalRows": total_rows,
        "detectedColumns": detected_columns,
        "detectedChannels": list(detected_channels),
        "sampleRows": sample_rows,
    }


@router.get("/export")
async def export_contacts(
    channel: str | None = None,
    tag: str | None = None,
    search: str | None = None,
    segment_id: uuid.UUID | None = None,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """
    Exports filtered contacts into a downloadable CSV file. Scoped to the
    caller's own tenant.
    """
    if segment_id:
        segment = await session.get(ContactSegment, segment_id)
        if segment is None or segment.tenant_key != tenant_key:
            raise HTTPException(status_code=404, detail="Segment not found")
        stmt = build_segment_filter_stmt(segment.filters or {}, tenant_key=tenant_key)
    else:
        stmt = (
            select(Contact)
            .where(Contact.tenant_key == tenant_key)
            .options(selectinload(Contact.tags))
        )
        if tag:
            stmt = stmt.where(Contact.tags.any(Tag.name == tag))
        if channel:
            try:
                ch_enum = ChannelType(channel.lower())
                stmt = stmt.where(Contact.channel == ch_enum)
            except ValueError:
                pass
        if search and search.strip():
            term = f"%{search.strip()}%"
            stmt = stmt.where(or_(Contact.person_name.ilike(term), Contact.value.ilike(term)))

    contacts = (
        (await session.execute(stmt.order_by(Contact.created_at.desc()))).scalars().unique().all()
    )

    output = io.StringIO()
    writer = csv.writer(output)
    writer.writerow(["ID", "Name", "Channel", "Value", "Language", "Tags", "Created At"])

    for c in contacts:
        writer.writerow(
            [
                str(c.id),
                c.person_name,
                c.channel.value,
                c.value,
                c.language,
                ",".join(t.name for t in c.tags),
                c.created_at.isoformat(),
            ]
        )

    return Response(
        content=output.getvalue(),
        media_type="text/csv",
        headers={"Content-Disposition": 'attachment; filename="contacts_export.csv"'},
    )
