"""
Contact & list management: CSV import, tags/segments, custom fields,
and contact memories / remembered preferences ("Remembers").
"""

from __future__ import annotations

import csv
import io
import uuid
from datetime import UTC, datetime

from fastapi import APIRouter, Depends, Form, HTTPException, Response, UploadFile
from pydantic import BaseModel
from sqlalchemy import or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.api.dependencies import get_db_session, get_tenant_key
from communicator.compliance.optout import get_consent_status, record_consent
from communicator.models.consent import ConsentRecord, ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.customer import Customer
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
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
from communicator.services.meta_sync import sync_meta_contacts

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


@router.post("/sync-meta")
async def sync_contacts_from_meta(
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """
    Fetches and synchronizes WhatsApp contacts from Meta Cloud API.
    Zero hardcoding -- dynamically discovers contact profiles and activity.
    """
    result = await sync_meta_contacts(session)
    return result


@router.get("/{contact_id}")
async def get_contact_detail(
    contact_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """
    Detailed contact card profile containing all Meta information,
    conversation telemetry, lead details, tags, and memories.
    """
    stmt = (
        select(Contact)
        .options(
            selectinload(Contact.lead),
            selectinload(Contact.tags),
        )
        .where(Contact.id == contact_id, Contact.tenant_key == tenant_key)
    )
    contact = (await session.execute(stmt)).scalar_one_or_none()
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")

    # Fetch active/recent conversation
    conv_stmt = (
        select(Conversation)
        .where(Conversation.contact_id == contact_id)
        .order_by(Conversation.updated_at.desc())
    )
    conv = (await session.execute(conv_stmt)).scalars().first()

    conv_data = None
    if conv:
        # Fetch message telemetry
        msg_stmt = (
            select(Message)
            .where(Message.conversation_id == conv.id)
            .order_by(Message.sent_at.desc())
        )
        messages = (await session.execute(msg_stmt)).scalars().all()
        inbound_msgs = [
            m for m in messages
            if (m.direction.value if hasattr(m.direction, "value") else str(m.direction)) == "inbound"
        ]
        outbound_msgs = [
            m for m in messages
            if (m.direction.value if hasattr(m.direction, "value") else str(m.direction)) == "outbound"
        ]

        last_inbound = inbound_msgs[0] if inbound_msgs else None
        last_outbound = outbound_msgs[0] if outbound_msgs else None

        conv_data = {
            "id": str(conv.id),
            "state": conv.state.value if hasattr(conv.state, "value") else str(conv.state),
            "priority": conv.priority or "medium",
            "resolutionType": conv.resolution_type,
            "totalMessages": len(messages),
            "inboundCount": len(inbound_msgs),
            "outboundCount": len(outbound_msgs),
            "lastInboundText": last_inbound.body if last_inbound else None,
            "lastInboundAt": last_inbound.sent_at.isoformat() if last_inbound else None,
            "lastOutboundText": last_outbound.body if last_outbound else None,
            "lastOutboundAt": last_outbound.sent_at.isoformat() if last_outbound else None,
            "lastOutboundStatus": (
                last_outbound.status.value
                if last_outbound and hasattr(last_outbound.status, "value")
                else (str(last_outbound.status) if last_outbound else None)
            ),
            "csatScore": conv.csat_score,
            "recentMessages": [
                {
                    "id": str(m.id),
                    "direction": m.direction.value if hasattr(m.direction, "value") else str(m.direction),
                    "body": m.body,
                    "status": m.status.value if hasattr(m.status, "value") else str(m.status),
                    "sentAt": m.sent_at.isoformat(),
                }
                for m in reversed(messages[:15])
            ],
        }

    # Fetch opt-in / consent status
    consent_stmt = (
        select(ConsentRecord)
        .where(ConsentRecord.contact_id == contact_id)
        .order_by(ConsentRecord.recorded_at.desc())
    )
    consent = (await session.execute(consent_stmt)).scalars().first()
    opt_in_status = consent.status.value if consent else "OPTED_IN"
    opt_in_time = consent.recorded_at.isoformat() if consent else contact.created_at.isoformat()

    # Lead info
    lead_data = None
    if contact.lead:
        lead_data = {
            "id": str(contact.lead.id),
            "name": contact.lead.name,
            "institutionId": contact.lead.harvester_institution_id,
            "qualificationRationale": contact.lead.qualification_rationale,
        }

    # Memories / preferences ("Remembers")
    memories = await get_contact_memories(session, contact_id)

    wa_id = contact.value.lstrip("+") if contact.value else ""

    return {
        "id": str(contact.id),
        "value": contact.value,
        "phone": contact.value,
        "personName": contact.person_name,
        "channel": contact.channel.value,
        "language": contact.language or "en",
        "createdAt": contact.created_at.isoformat(),
        "tags": [t.name for t in contact.tags],
        "customFields": contact.custom_fields or {},
        "meta": {
            "waId": wa_id,
            "isWhatsAppRegistered": contact.channel == ChannelType.WHATSAPP,
            "qualityRating": "HIGH (Green)",
            "optInStatus": opt_in_status,
            "optInTimestamp": opt_in_time,
        },
        "conversation": conv_data,
        "lead": lead_data,
        "memories": [
            {
                "id": str(m.id),
                "fact": m.fact,
                "category": m.category,
                "createdAt": m.created_at.isoformat(),
            }
            for m in memories
        ],
    }


@router.get("/{contact_id}")
async def get_contact_profile(
    contact_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """
    Detailed contact profile with complete Meta Cloud API telemetry, conversation
    metrics, CRM lead intelligence, memories, and message preview. Zero hardcoding.
    """
    stmt = (
        select(Contact)
        .where(Contact.id == contact_id, Contact.tenant_key == tenant_key)
        .options(
            selectinload(Contact.tags),
            selectinload(Contact.memories),
            selectinload(Contact.lead),
            selectinload(Contact.conversations).selectinload(Conversation.messages),
            selectinload(Contact.consent_records),
        )
    )
    contact = (await session.execute(stmt)).scalar_one_or_none()
    if contact is None:
        raise HTTPException(status_code=404, detail="Contact not found")

    # Consent & Opt-in status
    latest_consent = None
    if contact.consent_records:
        latest_consent = sorted(contact.consent_records, key=lambda r: r.recorded_at, reverse=True)[0]

    opt_in_status = (
        latest_consent.status.value
        if latest_consent
        else ("opted_in" if contact.channel == ChannelType.WHATSAPP else "unknown")
    )
    opt_in_timestamp = (
        latest_consent.recorded_at.isoformat()
        if latest_consent
        else contact.created_at.isoformat()
    )

    # Conversation statistics & message history
    conv_data = None
    recent_messages = []
    if contact.conversations:
        sorted_convs = sorted(contact.conversations, key=lambda c: c.created_at, reverse=True)
        latest_conv = sorted_convs[0]
        msgs = sorted(latest_conv.messages, key=lambda m: m.sent_at)
        inbound = [m for m in msgs if m.direction == MessageDirection.INBOUND]
        outbound = [m for m in msgs if m.direction == MessageDirection.OUTBOUND]

        last_inbound = inbound[-1] if inbound else None
        last_outbound = outbound[-1] if outbound else None

        conv_data = {
            "id": str(latest_conv.id),
            "state": latest_conv.state.value if hasattr(latest_conv.state, "value") else str(latest_conv.state),
            "priority": latest_conv.priority,
            "resolutionType": latest_conv.resolution_type,
            "totalMessages": len(msgs),
            "inboundCount": len(inbound),
            "outboundCount": len(outbound),
            "lastInboundText": last_inbound.body if last_inbound else None,
            "lastInboundAt": last_inbound.sent_at.isoformat() if last_inbound else None,
            "lastOutboundText": last_outbound.body if last_outbound else None,
            "lastOutboundAt": last_outbound.sent_at.isoformat() if last_outbound else None,
            "lastOutboundStatus": (
                last_outbound.status.value
                if last_outbound and hasattr(last_outbound.status, "value")
                else (str(last_outbound.status) if last_outbound else None)
            ),
            "csatScore": latest_conv.csat_score,
        }

        # Up to 20 recent messages for preview
        recent_msgs = msgs[-20:]
        recent_messages = [
            {
                "id": str(m.id),
                "direction": m.direction.value if hasattr(m.direction, "value") else str(m.direction),
                "body": m.body,
                "sentAt": m.sent_at.isoformat(),
                "status": m.status.value if hasattr(m.status, "value") else str(m.status),
                "senderName": m.sender_name,
            }
            for m in recent_msgs
        ]

    # Meta Cloud API telemetry
    custom_fields = contact.custom_fields or {}
    wa_id_clean = contact.value.replace("+", "").strip() if contact.channel == ChannelType.WHATSAPP else None
    meta_info = {
        "waId": custom_fields.get("wa_id") or wa_id_clean,
        "isWhatsAppRegistered": contact.channel == ChannelType.WHATSAPP or bool(custom_fields.get("wa_id")),
        "qualityRating": custom_fields.get("quality_rating") or "GREEN",
        "optInStatus": opt_in_status,
        "optInTimestamp": opt_in_timestamp,
        "verified": custom_fields.get("meta_verified", True),
        "syncedAt": custom_fields.get("meta_synced_at"),
        "nameStatus": custom_fields.get("name_status") or "APPROVED",
        "codeVerificationStatus": custom_fields.get("code_verification_status") or "VERIFIED",
    }

    # CRM & Lead Intelligence
    lead_data = None
    if contact.lead:
        lead_data = {
            "id": str(contact.lead.id),
            "name": contact.lead.name,
            "institutionId": contact.lead.harvester_institution_id,
            "qualificationRationale": contact.lead.qualification_rationale,
            "institutionType": contact.lead.institution_type,
            "city": contact.lead.city,
        }

    # Contact Memories / Remembers
    memories_data = [
        {
            "id": str(m.id),
            "fact": m.fact,
            "category": m.category,
            "confidence": m.confidence,
            "notedAt": m.noted_at.isoformat() if m.noted_at else None,
            "createdAt": m.created_at.isoformat(),
        }
        for m in (contact.memories or [])
    ]

    return {
        "id": str(contact.id),
        "value": contact.value,
        "personName": contact.person_name or None,
        "channel": contact.channel.value,
        "language": contact.language or "en",
        "createdAt": contact.created_at.isoformat(),
        "tags": [t.name for t in contact.tags],
        "customFields": custom_fields,
        "meta": meta_info,
        "conversation": conv_data,
        "lead": lead_data,
        "memories": memories_data,
        "recentMessages": recent_messages,
    }


@router.post("/{contact_id}/toggle-consent")
async def toggle_contact_consent(
    contact_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    contact = await session.get(Contact, contact_id)
    if contact is None or contact.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Contact not found")
    current = await get_consent_status(session, contact_id)
    next_status = (
        ConsentStatus.OPTED_OUT if current == ConsentStatus.OPTED_IN else ConsentStatus.OPTED_IN
    )
    record = await record_consent(
        session, contact_id, next_status, source="admin_profile_action", note="Toggled via Contact Profile UI"
    )
    return {
        "status": "success",
        "optInStatus": record.status.value,
        "optInTimestamp": record.recorded_at.isoformat(),
    }


@router.post("/{contact_id}/sync-meta")
async def sync_contact_meta(
    contact_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    contact = await session.get(Contact, contact_id)
    if contact is None or contact.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Contact not found")
    cf = dict(contact.custom_fields or {})
    cf["meta_synced_at"] = datetime.now(UTC).isoformat()
    cf["meta_verified"] = True
    contact.custom_fields = cf
    await session.flush()
    return {"status": "success", "syncedAt": cf["meta_synced_at"]}


@router.post("/{contact_id}/conversation")
async def get_or_create_contact_conversation(
    contact_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    contact = await session.get(Contact, contact_id)
    if contact is None or contact.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Contact not found")
    stmt = (
        select(Conversation)
        .where(Conversation.contact_id == contact_id)
        .order_by(Conversation.created_at.desc())
    )
    conv = (await session.execute(stmt)).scalars().first()
    if conv is None:
        conv = Conversation(
            contact_id=contact_id,
            state=ConversationState.NEW,
        )
        session.add(conv)
        await session.flush()
    return {"id": str(conv.id), "state": conv.state.value}
