"""nID Authentication API Routes for Communicator

Provides endpoints for 'Login with nID' relying-party flow:
- GET /api/v1/auth/nid/start: Initiates login request and returns authorize URL
- GET /api/v1/auth/nid/callback: Callback receiving auth code, exchanging for tokens and creating session
- GET /api/v1/auth/me: Inspects current user session
"""

from __future__ import annotations

import logging
from typing import Any

from fastapi import APIRouter, Header, HTTPException, Query, status
from pydantic import BaseModel

from communicator.services.nid_client import NidAuthError, nid_client

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/auth/nid", tags=["Authentication (nID)"])


class LoginStartResponse(BaseModel):
    authorize_url: str
    poll_token: str
    state: str


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
    """
    Begins 'Login with nID' attempt.
    Calls nID's OAuth login-request endpoint and returns the authorization URL for the user.
    """
    res = await nid_client.start_login(custom_redirect_uri=redirect_uri)
    return LoginStartResponse(**res)


@router.get("/callback", response_model=LoginCallbackResponse)
async def login_with_nid_callback(
    code: str = Query(..., description="One-time authorization code from nID"),
    state: str = Query(..., description="Anti-CSRF state token matching start request"),
) -> LoginCallbackResponse:
    """
    Callback URL where nID redirects after user approves authentication.
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
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=str(exc),
        )

    return LoginCallbackResponse(
        token=claims["access_token"],
        account_id=claims["account_id"],
        phone=claims.get("phone"),
        email=claims.get("email"),
        phone_verified=claims.get("phone_verified", False),
    )


@router.get("/me")
async def get_current_user_profile(
    authorization: str | None = Header(default=None),
) -> dict[str, Any]:
    """
    Returns identity claims of the currently authenticated nID account.
    """
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing or invalid Bearer authorization header.",
        )

    token = authorization.split("Bearer ", 1)[1].strip()
    session_claims = nid_client.validate_session(token)
    if not session_claims:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid or expired session token.",
        )

    return {
        "status": "authenticated",
        "account_id": session_claims.get("account_id"),
        "email": session_claims.get("email"),
        "phone": session_claims.get("phone"),
        "phone_verified": session_claims.get("phone_verified", False),
        "session_token": token[:8] + "...",
        "provider": "nID",
    }
