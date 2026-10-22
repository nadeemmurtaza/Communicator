"""
Backs the Connections page: WhatsApp (Meta Cloud API) credentials, a
"verify connection" check against Meta's Graph API, template message
list/create, and the business profile + guidelines the reply engine
grounds its system prompt in.

Secrets are never echoed back in full -- GET responses mask everything
except whether a value is set and its last 4 characters, so the page can
show "configured, ending in ...1234" without round-tripping the token.
"""

from __future__ import annotations

from datetime import UTC, datetime

import httpx
from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services.integration_settings import get_settings_row

router = APIRouter(prefix="/settings", tags=["Settings"])

GRAPH_API_VERSION = "v21.0"
GRAPH_BASE = f"https://graph.facebook.com/{GRAPH_API_VERSION}"


def _mask(value: str) -> str | None:
    if not value:
        return None
    return f"...{value[-4:]}" if len(value) > 4 else "...set"


@router.get("")
async def get_settings(session: AsyncSession = Depends(get_db_session)):
    row = await get_settings_row(session)
    return {
        "whatsapp": {
            "provider": row.whatsapp_provider,
            "phoneNumberId": row.meta_phone_number_id,
            "wabaId": row.meta_waba_id,
            "accessTokenMasked": _mask(row.meta_access_token),
            "appSecretMasked": _mask(row.meta_app_secret),
            "webhookVerifyTokenMasked": _mask(row.meta_webhook_verify_token),
            "webhookCallbackPath": "/api/v1/webhooks/meta/whatsapp",
            "verifiedAt": row.whatsapp_verified_at.isoformat()
            if row.whatsapp_verified_at
            else None,
            "verifiedName": row.whatsapp_verified_name,
            "displayNumber": row.whatsapp_display_number,
        },
        "business": {
            "name": row.business_name,
            "description": row.business_description,
            "agentGuidelines": row.agent_guidelines,
            "autoReplyEnabled": row.auto_reply_enabled,
            "businessHoursEnabled": row.business_hours_enabled,
            "businessHoursStart": row.business_hours_start,
            "businessHoursEnd": row.business_hours_end,
            "businessTimezone": row.business_timezone,
            "awayMessage": row.away_message,
        },
    }


class WhatsAppSettingsIn(BaseModel):
    provider: str | None = None
    access_token: str | None = None
    phone_number_id: str | None = None
    waba_id: str | None = None
    app_secret: str | None = None
    webhook_verify_token: str | None = None


class BusinessSettingsIn(BaseModel):
    name: str | None = None
    description: str | None = None
    agent_guidelines: str | None = None
    auto_reply_enabled: bool | None = None
    business_hours_enabled: bool | None = None
    business_hours_start: int | None = None
    business_hours_end: int | None = None
    business_timezone: str | None = None
    away_message: str | None = None


class SettingsIn(BaseModel):
    whatsapp: WhatsAppSettingsIn | None = None
    business: BusinessSettingsIn | None = None


@router.put("")
async def update_settings(payload: SettingsIn, session: AsyncSession = Depends(get_db_session)):
    row = await get_settings_row(session)

    if payload.whatsapp:
        w = payload.whatsapp
        if w.provider is not None:
            row.whatsapp_provider = w.provider
        if w.access_token is not None:
            row.meta_access_token = w.access_token
        if w.phone_number_id is not None:
            row.meta_phone_number_id = w.phone_number_id
        if w.waba_id is not None:
            row.meta_waba_id = w.waba_id
        if w.app_secret is not None:
            row.meta_app_secret = w.app_secret
        if w.webhook_verify_token is not None:
            row.meta_webhook_verify_token = w.webhook_verify_token

    if payload.business:
        b = payload.business
        if b.name is not None:
            row.business_name = b.name
        if b.description is not None:
            row.business_description = b.description
        if b.agent_guidelines is not None:
            row.agent_guidelines = b.agent_guidelines
        if b.auto_reply_enabled is not None:
            row.auto_reply_enabled = b.auto_reply_enabled
        if b.business_hours_enabled is not None:
            row.business_hours_enabled = b.business_hours_enabled
        if b.business_hours_start is not None:
            row.business_hours_start = b.business_hours_start
        if b.business_hours_end is not None:
            row.business_hours_end = b.business_hours_end
        if b.business_timezone is not None:
            row.business_timezone = b.business_timezone
        if b.away_message is not None:
            row.away_message = b.away_message

    await session.flush()
    return {"status": "saved"}


@router.post("/whatsapp/verify")
async def verify_whatsapp(session: AsyncSession = Depends(get_db_session)):
    """
    Confirms the stored access token + phone number id actually work, by
    asking Meta for that phone number's own record. Mirrors what the Meta
    console's own "test number" step shows (verified_name, display number).
    """
    row = await get_settings_row(session)
    if not (row.meta_access_token and row.meta_phone_number_id):
        raise HTTPException(
            status_code=400, detail="Access token and phone number ID are required."
        )

    url = f"{GRAPH_BASE}/{row.meta_phone_number_id}"
    async with httpx.AsyncClient(timeout=15.0) as client:
        response = await client.get(
            url,
            params={"fields": "verified_name,display_phone_number,quality_rating"},
            headers={"Authorization": f"Bearer {row.meta_access_token}"},
        )

    if response.status_code != 200:
        row.whatsapp_verified_at = None
        await session.flush()
        detail = response.json().get("error", {}).get("message", response.text)
        raise HTTPException(status_code=502, detail=f"Meta rejected the connection: {detail}")

    data = response.json()
    row.whatsapp_verified_at = datetime.now(UTC)
    row.whatsapp_verified_name = data.get("verified_name", "")
    row.whatsapp_display_number = data.get("display_phone_number", "")
    await session.flush()

    return {
        "status": "connected",
        "verifiedName": row.whatsapp_verified_name,
        "displayNumber": row.whatsapp_display_number,
        "qualityRating": data.get("quality_rating"),
    }


@router.get("/whatsapp/templates")
async def list_templates(session: AsyncSession = Depends(get_db_session)):
    row = await get_settings_row(session)
    if not (row.meta_access_token and row.meta_waba_id):
        raise HTTPException(
            status_code=400, detail="Access token and WhatsApp Business Account ID are required."
        )

    url = f"{GRAPH_BASE}/{row.meta_waba_id}/message_templates"
    async with httpx.AsyncClient(timeout=15.0) as client:
        response = await client.get(
            url,
            params={"fields": "id,name,status,category,language,components,quality_score"},
            headers={"Authorization": f"Bearer {row.meta_access_token}"},
        )

    if response.status_code != 200:
        detail = response.json().get("error", {}).get("message", response.text)
        raise HTTPException(status_code=502, detail=f"Meta rejected the request: {detail}")

    data = response.json().get("data", [])
    return [
        {
            "id": t.get("id"),
            "name": t.get("name"),
            "status": t.get("status"),
            "category": t.get("category"),
            "language": t.get("language"),
            "bodyText": next(
                (c.get("text", "") for c in t.get("components", []) if c.get("type") == "BODY"), ""
            ),
        }
        for t in data
    ]


@router.delete("/whatsapp/templates/{name}")
async def delete_template(name: str, session: AsyncSession = Depends(get_db_session)):
    row = await get_settings_row(session)
    if not (row.meta_access_token and row.meta_waba_id):
        raise HTTPException(
            status_code=400, detail="Access token and WhatsApp Business Account ID are required."
        )

    url = f"{GRAPH_BASE}/{row.meta_waba_id}/message_templates"
    async with httpx.AsyncClient(timeout=15.0) as client:
        response = await client.delete(
            url, params={"name": name}, headers={"Authorization": f"Bearer {row.meta_access_token}"}
        )

    if response.status_code != 200:
        detail = response.json().get("error", {}).get("message", response.text)
        raise HTTPException(status_code=502, detail=f"Meta rejected the deletion: {detail}")

    return {"status": "deleted"}


class TemplateIn(BaseModel):
    name: str
    category: str = "MARKETING"
    language: str = "en_US"
    body_text: str


@router.post("/whatsapp/templates")
async def create_template(payload: TemplateIn, session: AsyncSession = Depends(get_db_session)):
    row = await get_settings_row(session)
    if not (row.meta_access_token and row.meta_waba_id):
        raise HTTPException(
            status_code=400, detail="Access token and WhatsApp Business Account ID are required."
        )

    url = f"{GRAPH_BASE}/{row.meta_waba_id}/message_templates"
    body = {
        "name": payload.name,
        "category": payload.category,
        "language": payload.language,
        "components": [{"type": "BODY", "text": payload.body_text}],
    }
    async with httpx.AsyncClient(timeout=15.0) as client:
        response = await client.post(
            url, json=body, headers={"Authorization": f"Bearer {row.meta_access_token}"}
        )

    if response.status_code not in (200, 201):
        detail = response.json().get("error", {}).get("message", response.text)
        raise HTTPException(status_code=502, detail=f"Meta rejected the template: {detail}")

    return response.json()


__all__ = ["router"]
