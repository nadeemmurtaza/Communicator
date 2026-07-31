from collections.abc import AsyncIterator

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.database import get_session


async def get_db_session() -> AsyncIterator[AsyncSession]:
    async with get_session() as session:
        yield session
