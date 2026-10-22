from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, Query, Request
from fastapi.responses import HTMLResponse
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import widget_customizer_service

router = APIRouter(prefix="/widget", tags=["Live Chat Widget Customizer"])


class UpdateWidgetCustomizerRequest(BaseModel):
    inbox_name: str | None = None
    website_domain: str | None = None
    widget_color: str | None = Field(None, description="HEX brand color e.g. #1570EF")
    theme_mode: str | None = Field(None, description="light | dark | system")
    launcher_style: str | None = Field(None, description="round_bubble | pill_with_text | minimal_tab")
    launcher_icon: str | None = Field(None, description="chat_bubble | sparkle_ai | headset_support | envelope")
    launcher_text: str | None = Field(None, description="Text for pill launcher e.g. Chat with us")
    widget_position: str | None = Field(None, description="right | left")
    greeting_title: str | None = None
    greeting_tagline: str | None = None
    avatar_url: str | None = None
    border_radius: str | None = Field(None, description="rounded_lg | rounded_2xl | rounded_full")
    play_sound: bool | None = None
    show_branding: bool | None = None
    allowed_domains: str | None = Field(None, description="Comma-separated allowed origins")
    pre_chat_form_enabled: bool | None = None
    identity_validation: bool | None = None
    hmac_secret: str | None = None


@router.get("/customizer")
async def get_widget_customizer(
    request: Request, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Retrieve full widget customizer configuration, branding, and embed scripts."""
    base_url = str(request.base_url).rstrip("/")
    return await widget_customizer_service.get_customizer_config(session, base_url=base_url)


@router.post("/customizer")
async def update_widget_customizer(
    payload: UpdateWidgetCustomizerRequest,
    request: Request,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Save live chat widget customization styles, launcher, and security settings."""
    base_url = str(request.base_url).rstrip("/")
    updates = payload.model_dump(exclude_unset=True)
    return await widget_customizer_service.update_customizer_config(
        session, updates, base_url=base_url
    )


@router.post("/customizer/reset")
async def reset_widget_customizer(
    request: Request, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Reset widget styling and launcher configuration to factory defaults."""
    base_url = str(request.base_url).rstrip("/")
    return await widget_customizer_service.reset_customizer_defaults(session, base_url=base_url)


@router.get("/customizer/embed-code")
async def get_embed_code(
    request: Request,
    token: str | None = Query(None),
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Returns ready-to-copy HTML script, React/Next.js component, and GTM snippets."""
    base_url = str(request.base_url).rstrip("/")
    cfg = await widget_customizer_service.get_customizer_config(session, base_url=base_url)
    target_token = token or cfg["website_token"]
    codes = widget_customizer_service.generate_embed_codes(target_token, base_url, cfg["styling"])
    return {
        "website_token": target_token,
        "base_url": base_url,
        "embed_codes": codes,
    }


@router.get("/preview-page", response_class=HTMLResponse)
async def get_preview_page(
    request: Request,
    token: str | None = Query(None),
    session: AsyncSession = Depends(get_db_session),
):
    """Renders a self-contained HTML page embedding the customized widget for live testing."""
    base_url = str(request.base_url).rstrip("/")
    cfg = await widget_customizer_service.get_customizer_config(session, base_url=base_url)
    target_token = token or cfg["website_token"]
    html = widget_customizer_service.generate_preview_page_html(target_token, base_url, cfg["styling"])
    return HTMLResponse(content=html)
