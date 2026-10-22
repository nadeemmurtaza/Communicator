"""nID Authentication routes -- "Login with nID" relying-party flow.

- GET /auth/nid/start: registers a login attempt, returns the authorize URL
- GET /auth/nid/callback: nID's redirect target; exchanges code for identity
- GET /auth/me: inspects the current session token
"""

from __future__ import annotations

import logging

from fastapi import APIRouter, Header, HTTPException, Query, status
from pydantic import BaseModel

from communicator.config import settings
from communicator.services.nid_client import NidAuthError, nid_client

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/auth/nid", tags=["Authentication (nID)"])


class LoginStartResponse(BaseModel):
    authorize_url: str
    poll_token: str
    state: str
    # True when no live nID instance is configured -- the frontend should
    # complete the exchange directly instead of redirecting to authorize_url.
    dev_mode: bool = False


class LoginCallbackResponse(BaseModel):
    token: str
    account_id: str
    phone: str | None = None
    email: str | None = None
    phone_verified: bool = False


@router.get("/start", response_model=LoginStartResponse)
async def login_with_nid_start(
    redirect_uri: str | None = Query(default=None, description="Optional custom redirect URI"),
) -> LoginStartResponse:
    res = await nid_client.start_login(custom_redirect_uri=redirect_uri)
    dev_mode = settings.app_env == "development" or settings.nid_allow_mock_auth
    return LoginStartResponse(**res, dev_mode=dev_mode)


@router.get("/callback", response_model=LoginCallbackResponse)
async def login_with_nid_callback(
    code: str = Query(..., description="One-time authorization code from nID"),
    state: str = Query(..., description="Anti-CSRF state token matching the start request"),
) -> LoginCallbackResponse:
    try:
        claims = await nid_client.exchange_code(code=code, state=state)
    except NidAuthError as exc:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=str(exc)) from exc

    return LoginCallbackResponse(
        token=claims["access_token"],
        account_id=claims["account_id"],
        phone=claims.get("phone"),
        email=claims.get("email"),
        phone_verified=claims.get("phone_verified", False),
    )


@router.get("/me")
async def get_current_user_profile(authorization: str | None = Header(default=None)) -> dict:
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, detail="Missing or invalid Bearer authorization header."
        )

    token = authorization.split("Bearer ", 1)[1].strip()
    session_claims = nid_client.validate_session(token)
    if not session_claims:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid or expired session token.")

    return {
        "status": "authenticated",
        "accountId": session_claims.get("account_id"),
        "email": session_claims.get("email"),
        "phone": session_claims.get("phone"),
        "phoneVerified": session_claims.get("phone_verified", False),
        "provider": "nID",
    }
