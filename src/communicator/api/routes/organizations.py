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

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_current_account_id, get_db_session
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


@router.get("")
async def list_organizations(
    account_id: str = Depends(get_current_account_id),
    session: AsyncSession = Depends(get_db_session),
):
    # Scoped to the *authenticated* account, not a caller-supplied
    # account_id -- an anonymous or arbitrary account_id query param
    # previously let anyone enumerate every tenant's name and key.
    stmt = select(Tenant).where(Tenant.owner_account_id == account_id).order_by(Tenant.created_at)
    tenants = (await session.execute(stmt)).scalars().all()
    return [_serialize(t) for t in tenants]


@router.post("")
async def create_organization(
    payload: OrganizationIn,
    account_id: str = Depends(get_current_account_id),
    session: AsyncSession = Depends(get_db_session),
):
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

    # owner_account_id always comes from the validated session, never the
    # request body -- a client-supplied owner_account_id let anyone create
    # an organization attributed to an arbitrary account.
    tenant = Tenant(tenant_key=tenant_key, name=payload.name.strip(), owner_account_id=account_id)
    session.add(tenant)
    await session.flush()
    return _serialize(tenant)
