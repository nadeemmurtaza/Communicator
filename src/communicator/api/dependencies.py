from collections.abc import AsyncIterator

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.database import get_session
from communicator.harvester_client import HarvesterClient


async def get_db_session() -> AsyncIterator[AsyncSession]:
    async with get_session() as session:
        yield session


def get_harvester_client() -> HarvesterClient:
    return HarvesterClient()
