from __future__ import annotations

import csv
import io
import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query, Response
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session, get_tenant_key
from communicator.models.segment import ContactSegment
from communicator.services.segment_service import (
    count_segment_contacts,
    get_segment_contacts,
    seed_default_segments,
)

router = APIRouter(prefix="/segments", tags=["Contact Segments"])


class SegmentIn(BaseModel):
    name: str
    description: str = ""
    filters: dict[str, Any] = {}


class SegmentUpdateIn(BaseModel):
    name: str | None = None
    description: str | None = None
    filters: dict[str, Any] | None = None


async def _serialize_segment(session: AsyncSession, segment: ContactSegment) -> dict[str, Any]:
    member_count = await count_segment_contacts(session, segment)
    return {
        "id": str(segment.id),
        "name": segment.name,
        "description": segment.description,
        "filters": segment.filters or {},
        "memberCount": member_count,
        "createdAt": segment.created_at.isoformat(),
        "updatedAt": segment.updated_at.isoformat(),
    }


@router.get("")
async def list_segments(
    session: AsyncSession = Depends(get_db_session), tenant_key: str = Depends(get_tenant_key)
):
    """Lists all saved contact segments with real-time member counts, scoped to the caller's tenant."""
    await seed_default_segments(session, tenant_key=tenant_key)
    stmt = (
        select(ContactSegment)
        .where(ContactSegment.tenant_key == tenant_key)
        .order_by(ContactSegment.created_at.asc())
    )
    segments = (await session.execute(stmt)).scalars().all()
    return [await _serialize_segment(session, s) for s in segments]


@router.post("", status_code=201)
async def create_segment(
    payload: SegmentIn,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """Creates a new dynamic saved contact segment for the caller's tenant."""
    name = payload.name.strip()
    if not name:
        raise HTTPException(status_code=400, detail="Segment name cannot be empty.")

    existing_stmt = select(ContactSegment).where(
        ContactSegment.tenant_key == tenant_key, ContactSegment.name == name
    )
    existing = (await session.execute(existing_stmt)).scalar_one_or_none()
    if existing is not None:
        raise HTTPException(status_code=400, detail=f"A segment with name '{name}' already exists.")

    segment = ContactSegment(
        tenant_key=tenant_key,
        name=name,
        description=payload.description.strip(),
        filters=payload.filters or {},
    )
    session.add(segment)
    await session.flush()
    return await _serialize_segment(session, segment)


@router.get("/{segment_id}")
async def get_segment(
    segment_id: uuid.UUID,
    limit: int = Query(default=50, ge=1, le=500),
    offset: int = Query(default=0, ge=0),
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """Retrieves segment details and matching contact members."""
    segment = await session.get(ContactSegment, segment_id)
    if segment is None or segment.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Segment not found.")

    contacts = await get_segment_contacts(session, segment, limit=limit, offset=offset)
    serialized = await _serialize_segment(session, segment)
    serialized["contacts"] = [
        {
            "id": str(c.id),
            "personName": c.person_name,
            "channel": c.channel.value,
            "value": c.value,
            "tags": [t.name for t in c.tags],
            "createdAt": c.created_at.isoformat(),
        }
        for c in contacts
    ]
    return serialized


@router.put("/{segment_id}")
async def update_segment(
    segment_id: uuid.UUID,
    payload: SegmentUpdateIn,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """Updates segment properties and targeting rules."""
    segment = await session.get(ContactSegment, segment_id)
    if segment is None or segment.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Segment not found.")

    if payload.name is not None:
        new_name = payload.name.strip()
        if not new_name:
            raise HTTPException(status_code=400, detail="Segment name cannot be empty.")
        if new_name != segment.name:
            existing_stmt = select(ContactSegment).where(
                ContactSegment.tenant_key == tenant_key, ContactSegment.name == new_name
            )
            existing = (await session.execute(existing_stmt)).scalar_one_or_none()
            if existing is not None:
                raise HTTPException(
                    status_code=400, detail=f"A segment with name '{new_name}' already exists."
                )
            segment.name = new_name

    if payload.description is not None:
        segment.description = payload.description.strip()

    if payload.filters is not None:
        segment.filters = payload.filters

    await session.flush()
    return await _serialize_segment(session, segment)


@router.delete("/{segment_id}")
async def delete_segment(
    segment_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """Deletes a saved contact segment."""
    segment = await session.get(ContactSegment, segment_id)
    if segment is None or segment.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Segment not found.")

    await session.delete(segment)
    await session.flush()
    return {"status": "ok", "deleted": True, "id": str(segment_id)}


@router.get("/{segment_id}/export")
async def export_segment_contacts(
    segment_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
    tenant_key: str = Depends(get_tenant_key),
):
    """Exports all contacts matching the segment as downloadable CSV."""
    segment = await session.get(ContactSegment, segment_id)
    if segment is None or segment.tenant_key != tenant_key:
        raise HTTPException(status_code=404, detail="Segment not found.")

    contacts = await get_segment_contacts(session, segment, limit=5000)

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

    filename = f"segment_{segment.name.lower().replace(' ', '_')}_{segment.id.hex[:6]}.csv"
    return Response(
        content=output.getvalue(),
        media_type="text/csv",
        headers={"Content-Disposition": f'attachment; filename="{filename}"'},
    )
