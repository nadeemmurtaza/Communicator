import pytest
import pytest_asyncio
from httpx import ASGITransport, AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.pool import NullPool

import communicator.models  # noqa: F401
from communicator.api.app import app
from communicator.api.dependencies import get_db_session
from communicator.config import settings, validate_test_database_url
from communicator.database import Base

validate_test_database_url(settings.test_database_url, settings.database_url)

engine = create_async_engine(settings.test_database_url, echo=False, poolclass=NullPool)


@pytest_asyncio.fixture(scope="session")
async def async_engine():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    yield engine
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)


@pytest_asyncio.fixture
async def db_session(async_engine):
    async with async_engine.connect() as connection:
        outer_transaction = await connection.begin()
        session = AsyncSession(
            bind=connection,
            expire_on_commit=False,
            join_transaction_mode="create_savepoint",
        )
        try:
            yield session
        finally:
            await session.close()
            await outer_transaction.rollback()


@pytest.fixture
def override_database_dependency(db_session):
    async def get_test_session():
        yield db_session

    app.dependency_overrides[get_db_session] = get_test_session
    yield
    app.dependency_overrides.clear()


@pytest_asyncio.fixture
async def test_client(override_database_dependency):
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
        yield client
