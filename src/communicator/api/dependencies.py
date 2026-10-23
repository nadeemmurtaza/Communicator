from collections.abc import AsyncIterator

from fastapi import Header, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.database import get_session
from communicator.harvester_client import HarvesterClient
from communicator.services.nid_client import nid_client


async def get_db_session() -> AsyncIterator[AsyncSession]:
    async with get_session() as session:
        yield session


def get_harvester_client() -> HarvesterClient:
    return HarvesterClient()


def get_current_account_id(authorization: str | None = Header(default=None)) -> str:
    """Resolves the real nID account behind a request -- same Bearer-token
    validation as GET /auth/me (src/communicator/api/routes/nid_auth.py),
    pulled out so any endpoint that needs to know *who* is calling (not
    just which tenant they claim) can depend on it instead of trusting a
    client-supplied field."""
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing or invalid Bearer authorization header.",
        )
    token = authorization.split("Bearer ", 1)[1].strip()
    session_claims = nid_client.validate_session(token)
    if not session_claims:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid or expired session token."
        )
    account_id = session_claims.get("account_id")
    if not account_id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, detail="Session has no account identity."
        )
    return account_id


def get_tenant_key(x_tenant_key: str | None = Header(default=None)) -> str:
    """
    The tenant whose data this request should see -- sent by the frontend's
    `api.ts` from the organization the person has selected (AuthContext).
    Falls back to "newax" for callers that don't set it yet (admin tools,
    webhooks, older endpoints mid-migration to this header).
    """
    return x_tenant_key or "newax"
