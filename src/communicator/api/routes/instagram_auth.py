"""
Instagram Business Login OAuth callback -- the redirect target Meta sends a
user back to after they authorize the app on instagram.com. Exchanges the
authorization code for a long-lived access token and stores it on the
Instagram inbox, the same way _get_or_create_instagram_inbox seeds demo
credentials.
"""

from __future__ import annotations

from urllib.parse import urlencode

import httpx
from fastapi import APIRouter, Depends, HTTPException, Query
from fastapi.responses import RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.config import settings
from communicator.models.contact import ChannelType
from communicator.models.inbox import Inbox

router = APIRouter(prefix="/auth", tags=["Instagram Business Login"])

_SCOPES = "instagram_business_basic,instagram_business_manage_messages,instagram_business_manage_comments"


@router.get("/instagram/start")
async def instagram_oauth_start() -> RedirectResponse:
    """
    The "Connect Instagram Account" button's target -- sends the person to
    Instagram's own business-login consent screen. Instagram redirects them
    back to /auth/instagram/callback with a ?code=... on approval.
    """
    if not settings.meta_instagram_app_id:
        raise HTTPException(status_code=503, detail="META_INSTAGRAM_APP_ID not configured.")

    params = {
        "client_id": settings.meta_instagram_app_id,
        "redirect_uri": settings.meta_instagram_redirect_uri,
        "response_type": "code",
        "scope": _SCOPES,
    }
    return RedirectResponse(url=f"https://www.instagram.com/oauth/authorize?{urlencode(params)}")


@router.get("/instagram/callback")
async def instagram_oauth_callback(
    code: str | None = Query(default=None),
    error: str | None = Query(default=None),
    error_description: str | None = Query(default=None),
    session: AsyncSession = Depends(get_db_session),
):
    """
    Meta redirects here with ?code=... after the user authorizes the app on
    instagram.com. Exchanges the code for a short-lived token, then
    immediately for a long-lived one (60 days, refreshable), and saves the
    connected account onto the Instagram inbox.
    """
    if error:
        raise HTTPException(
            status_code=400, detail=error_description or f"Instagram authorization failed: {error}"
        )
    if not code:
        raise HTTPException(status_code=400, detail="Missing authorization code.")
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
    }
    inbox.is_active = True
    await session.flush()

    return {
        "status": "connected",
        "igId": ig_user_id,
        "igUsername": ig_username,
    }
