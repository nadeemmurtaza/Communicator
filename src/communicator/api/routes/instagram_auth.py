"""
Instagram Business Login OAuth callback -- the redirect target Meta sends a
user back to after they authorize the app on instagram.com. Exchanges the
authorization code for a long-lived access token and stores it on the
Instagram inbox, the same way _get_or_create_instagram_inbox seeds demo
credentials.
"""

from __future__ import annotations

import secrets
import time
from urllib.parse import urlencode

import httpx
from fastapi import APIRouter, Depends, Header, HTTPException, Query
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.config import settings
from communicator.models.contact import ChannelType
from communicator.models.inbox import Inbox
from communicator.services.nid_client import nid_client

router = APIRouter(prefix="/auth", tags=["Instagram Business Login"])

_SCOPES = (
    "instagram_business_basic,instagram_business_manage_messages,instagram_business_manage_comments"
)

# CSRF state for the Instagram OAuth handshake -- same pattern as
# nid_client's _pending_states, kept separate since this is a different
# provider's flow.
_pending_states: dict[str, float] = {}


def _prune_expired_states(max_age_seconds: int = 600) -> None:
    now = time.time()
    expired = [s for s, created_at in _pending_states.items() if now - created_at > max_age_seconds]
    for s in expired:
        _pending_states.pop(s, None)


def _require_operator(authorization: str | None) -> None:
    """
    Connecting an Instagram account replaces the business's live
    credentials, so only a signed-in operator may start or complete this
    flow -- not an anonymous visitor who happens to hit these URLs.
    """
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(status_code=401, detail="Sign in before connecting Instagram.")
    token = authorization.split("Bearer ", 1)[1].strip()
    if not nid_client.validate_session(token):
        raise HTTPException(status_code=401, detail="Invalid or expired session.")


@router.get("/instagram/start")
async def instagram_oauth_start(authorization: str | None = Header(default=None)) -> dict[str, str]:
    """
    Returns the Instagram business-login consent-screen URL for the "Connect
    Instagram Account" button to navigate to (JSON, not a server-side
    redirect, so the frontend can attach the operator's session header --
    a plain anchor tag can't send an Authorization header). Instagram
    redirects the browser back to /auth/instagram/callback with
    ?code=&state=... on approval.
    """
    _require_operator(authorization)
    if not settings.meta_instagram_app_id:
        raise HTTPException(status_code=503, detail="META_INSTAGRAM_APP_ID not configured.")

    _prune_expired_states()
    state = secrets.token_urlsafe(24)
    _pending_states[state] = time.time()

    params = {
        "client_id": settings.meta_instagram_app_id,
        "redirect_uri": settings.meta_instagram_redirect_uri,
        "response_type": "code",
        "scope": _SCOPES,
        "state": state,
    }
    return {"authorizeUrl": f"https://www.instagram.com/oauth/authorize?{urlencode(params)}"}


@router.get("/instagram/callback")
async def instagram_oauth_callback(
    code: str | None = Query(default=None),
    state: str | None = Query(default=None),
    error: str | None = Query(default=None),
    error_description: str | None = Query(default=None),
    session: AsyncSession = Depends(get_db_session),
):
    """
    Meta redirects here with ?code=&state=... after the user authorizes the
    app on instagram.com. This is a cross-site browser navigation, not a
    fetch from our own frontend, so it carries no Authorization header --
    the CSRF/authorization check happens at /instagram/start instead: only
    a signed-in operator can obtain a valid `state`, and that state can be
    redeemed here exactly once, which is what proves this callback traces
    back to an authorized start.
    """
    if error:
        raise HTTPException(
            status_code=400, detail=error_description or f"Instagram authorization failed: {error}"
        )
    if not code or not state:
        raise HTTPException(status_code=400, detail="Missing authorization code or state.")

    _prune_expired_states()
    if state not in _pending_states:
        raise HTTPException(
            status_code=400, detail="Invalid, expired, or already-used OAuth state parameter."
        )
    _pending_states.pop(state, None)

    if not settings.meta_instagram_app_id or not settings.meta_instagram_app_secret:
        raise HTTPException(
            status_code=503,
            detail="META_INSTAGRAM_APP_ID / META_INSTAGRAM_APP_SECRET not configured.",
        )

    redirect_uri = settings.meta_instagram_redirect_uri

    async with httpx.AsyncClient(timeout=20.0) as client:
        short_lived = await client.post(
            "https://api.instagram.com/oauth/access_token",
            data={
                "client_id": settings.meta_instagram_app_id,
                "client_secret": settings.meta_instagram_app_secret,
                "grant_type": "authorization_code",
                "redirect_uri": redirect_uri,
                "code": code,
            },
        )
        short_lived.raise_for_status()
        short_data = short_lived.json()
        ig_user_id = str(short_data["user_id"])
        short_token = short_data["access_token"]

        long_lived = await client.get(
            "https://graph.instagram.com/access_token",
            params={
                "grant_type": "ig_exchange_token",
                "client_secret": settings.meta_instagram_app_secret,
                "access_token": short_token,
            },
        )
        long_lived.raise_for_status()
        long_data = long_lived.json()
        access_token = long_data["access_token"]

        profile = await client.get(
            f"https://graph.instagram.com/{ig_user_id}",
            params={"fields": "username", "access_token": access_token},
        )
        profile.raise_for_status()
        ig_username = profile.json().get("username", "")

    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.INSTAGRAM)
    inbox = (await session.execute(stmt)).scalars().first()
    if inbox is None:
        inbox = Inbox(name="Instagram Direct", channel_type=ChannelType.INSTAGRAM, phone_number="")
        session.add(inbox)

    inbox.credentials = {
        **(inbox.credentials or {}),
        "igId": ig_user_id,
        "igUsername": f"@{ig_username}" if ig_username else "",
        "accessToken": access_token,
        # Webhook payloads are signed with the app's own secret regardless of
        # which Instagram account sent them -- without this, meta_instagram_inbound
        # has nothing to check the signature against and accepts unsigned events.
        "appSecret": settings.meta_instagram_app_secret,
    }
    inbox.is_active = True
    await session.flush()

    return {
        "status": "connected",
        "igId": ig_user_id,
        "igUsername": ig_username,
    }
