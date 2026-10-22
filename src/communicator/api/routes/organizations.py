"""
Organization (tenant) management -- the step after "Login with nID": an
authenticated account creates or joins an organization, which becomes their
tenant_key for every tenant-scoped table in the system. NEWAX is tenant #1,
seeded the same way any later organization is created here -- no hardcoded
tenant content in application code.
"""

from __future__ import annotations

import re
import uuid

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.tenant import Tenant

router = APIRouter(prefix="/organizations", tags=["Organizations"])


def _slugify(name: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", name.strip().lower()).strip("-")
    return slug or uuid.uuid4().hex[:8]


def _serialize(t: Tenant) -> dict:
    return {
        "id": str(t.id),
        "tenantKey": t.tenant_key,
        "name": t.name,
        "createdAt": t.created_at.isoformat(),
    }


class OrganizationIn(BaseModel):
    name: str
    account_id: str = ""


@router.get("")
async def list_organizations(
    account_id: str | None = Query(default=None), session: AsyncSession = Depends(get_db_session)
):
    stmt = select(Tenant).order_by(Tenant.created_at)
    if account_id:
        stmt = stmt.where(Tenant.owner_account_id == account_id)
    tenants = (await session.execute(stmt)).scalars().all()
    return [_serialize(t) for t in tenants]


@router.post("")
async def create_organization(payload: OrganizationIn, session: AsyncSession = Depends(get_db_session)):
    if not payload.name.strip():
        raise HTTPException(status_code=400, detail="Organization name is required.")

    base_key = _slugify(payload.name)
    tenant_key = base_key
    suffix = 1
    while (
        await session.execute(select(Tenant).where(Tenant.tenant_key == tenant_key))
    ).scalar_one_or_none() is not None:
        suffix += 1
        tenant_key = f"{base_key}-{suffix}"

    tenant = Tenant(tenant_key=tenant_key, name=payload.name.strip(), owner_account_id=payload.account_id)
    session.add(tenant)
    await session.flush()
    return _serialize(tenant)
