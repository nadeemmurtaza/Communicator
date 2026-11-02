"""nID Authentication routes -- "Login with nID" relying-party flow.

- GET /auth/nid/start: registers a login attempt, returns the authorize URL
- GET /auth/nid/callback: nID's redirect target; exchanges code for identity
- GET /auth/me: inspects the current session token
"""

from __future__ import annotations

import logging
from typing import Any

from fastapi import APIRouter, Header, HTTPException, Query, status
from pydantic import BaseModel

from communicator.config import settings
from communicator.services.nid_client import NidAuthError, nid_client

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/auth", tags=["auth"])


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
    email: str | None = None
    phone: str | None = None


@router.get("/nid/start", response_model=LoginStartResponse)
async def login_with_nid_start(
    redirect_uri: str | None = Query(default=None, description="Optional custom redirect URI"),
) -> LoginStartResponse:
    """Begins 'Login with nID' attempt.
    Calls nID's OAuth login-request endpoint and returns the authorization URL for the user.
    """
    res = await nid_client.start_login(custom_redirect_uri=redirect_uri)
    dev_mode = settings.app_env == "development" or settings.nid_allow_mock_auth
    return LoginStartResponse(**res, dev_mode=dev_mode)


@router.get("/callback", response_model=LoginCallbackResponse)
async def login_with_nid_callback(
    code: str = Query(..., description="One-time authorization code from nID"),
    state: str = Query(..., description="Anti-CSRF state token matching the start request"),
) -> LoginCallbackResponse:
    """Callback URL where nID redirects after user approves authentication.
    Exchanges one-time code server-to-server for verified account identity.
    """
    if not code or not state:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Missing code or state parameter.",
        )

    try:
        claims = await nid_client.exchange_code(code=code, state=state)
    except NidAuthError as exc:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=str(exc)) from exc

    return LoginCallbackResponse(
        token=claims["access_token"],
        account_id=claims["account_id"],
        email=claims.get("email"),
        phone=claims.get("phone"),
    )


@router.get("/me")
async def get_current_user_profile(
    authorization: str | None = Header(default=None),
) -> dict[str, Any]:
    """Returns identity claims of the currently authenticated nID account."""
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

    return {
        "status": "authenticated",
        "account_id": session_claims.get("account_id"),
        "accountId": session_claims.get("account_id"),
        "email": session_claims.get("email"),
        "phone": session_claims.get("phone"),
        "phone_verified": session_claims.get("phone_verified", False),
        "phoneVerified": session_claims.get("phone_verified", False),
        "session_token": token[:8] + "...",
        "provider": "nID",
    }
