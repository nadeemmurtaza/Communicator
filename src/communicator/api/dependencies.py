from collections.abc import AsyncIterator

from fastapi import Header
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.database import get_session
from communicator.harvester_client import HarvesterClient


async def get_db_session() -> AsyncIterator[AsyncSession]:
    async with get_session() as session:
        yield session


def get_harvester_client() -> HarvesterClient:
    return HarvesterClient()


def get_tenant_key(x_tenant_key: str | None = Header(default=None)) -> str:
    """
    The tenant whose data this request should see -- sent by the frontend's
    `api.ts` from the organization the person has selected (AuthContext).
    Falls back to "newax" for callers that don't set it yet (admin tools,
    webhooks, older endpoints mid-migration to this header).
    """
    return x_tenant_key or "newax"
