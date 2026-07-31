from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.service import Service

router = APIRouter(prefix="/services", tags=["Services"])


class ServiceIn(BaseModel):
    name: str
    summary: str = ""
    details: str = ""
    target_institution_types: list[str] = []
    faq: list[dict] = []
    pricing_notes: str = ""


@router.get("")
async def list_services(session: AsyncSession = Depends(get_db_session)):
    services = (await session.execute(select(Service))).scalars().all()
    return [
        {
            "id": str(s.id),
            "name": s.name,
            "summary": s.summary,
            "targetInstitutionTypes": s.target_institution_types,
        }
        for s in services
    ]


@router.post("")
async def create_service(payload: ServiceIn, session: AsyncSession = Depends(get_db_session)):
    service = Service(
        name=payload.name,
        summary=payload.summary,
        details=payload.details,
        target_institution_types=payload.target_institution_types,
        faq=payload.faq,
        pricing_notes=payload.pricing_notes,
    )
    session.add(service)
    await session.flush()
    return {"id": str(service.id)}


@router.get("/{service_id}")
async def get_service(service_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    service = await session.get(Service, service_id)
    if service is None:
        raise HTTPException(status_code=404, detail="Service not found")
    return {
        "id": str(service.id),
        "name": service.name,
        "summary": service.summary,
        "details": service.details,
        "targetInstitutionTypes": service.target_institution_types,
        "faq": service.faq,
        "pricingNotes": service.pricing_notes,
    }
