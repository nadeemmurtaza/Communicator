"""
Admin endpoints for the multi-tenant discovery flow: tenant templates
and the structured Inquiry records the flow produces.
"""

from __future__ import annotations

import uuid

import httpx
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.campaign_template import CampaignTemplate
from communicator.models.discovery_flow_template import DiscoveryFlowTemplate
from communicator.models.inquiry import Inquiry
from communicator.services.integration_settings import get_whatsapp_config
from communicator.services.seed_campaign_templates import seed_newax_campaign_templates
from communicator.services.seed_discovery_templates import seed_newax_discovery_template

router = APIRouter(prefix="/discovery", tags=["Discovery Flow"])


def _serialize_inquiry(i: Inquiry) -> dict:
    return {
        "id": str(i.id),
        "tenantKey": i.tenant_key,
        "organizationName": i.organization_name,
        "industry": i.industry,
        "contactRole": i.contact_role,
        "orgSizeRange": i.org_size_range,
        "problemDescription": i.problem_description,
        "problemCategories": i.problem_categories,
        "reportedImpact": i.reported_impact,
        "desiredOutcome": i.desired_outcome,
        "capabilityMapping": i.capability_mapping,
        "timeline": i.timeline,
        "budgetRange": i.budget_range,
        "qualificationScore": i.qualification_score,
        "qualificationFactors": i.qualification_factors,
        "status": i.status.value,
        "nextAction": i.next_action,
        "createdAt": i.created_at.isoformat(),
    }


@router.post("/tenants/newax/seed")
async def seed_newax_tenant(session: AsyncSession = Depends(get_db_session)):
    """Installs/refreshes NEWAX's discovery flow template (tenant #1)."""
    template = await seed_newax_discovery_template(session)
    return {"status": "ok", "tenantKey": template.tenant_key, "stepCount": len(template.steps)}


@router.get("/templates")
async def list_templates(session: AsyncSession = Depends(get_db_session)):
    templates = (await session.execute(select(DiscoveryFlowTemplate))).scalars().all()
    return [
        {
            "id": str(t.id),
            "tenantKey": t.tenant_key,
            "name": t.name,
            "isActive": t.is_active,
            "stepCount": len(t.steps),
        }
        for t in templates
    ]


@router.get("/inquiries")
async def list_inquiries(
    tenant_key: str | None = None, session: AsyncSession = Depends(get_db_session)
):
    stmt = select(Inquiry).order_by(Inquiry.created_at.desc())
    if tenant_key:
        stmt = stmt.where(Inquiry.tenant_key == tenant_key)
    inquiries = (await session.execute(stmt)).scalars().all()
    return [_serialize_inquiry(i) for i in inquiries]


@router.get("/inquiries/{inquiry_id}")
async def get_inquiry(inquiry_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    inquiry = await session.get(Inquiry, inquiry_id)
    if inquiry is None:
        raise HTTPException(status_code=404, detail="Inquiry not found")
    return _serialize_inquiry(inquiry)


def _serialize_campaign_template(t: CampaignTemplate) -> dict:
    return {
        "id": str(t.id),
        "tenantKey": t.tenant_key,
        "key": t.key,
        "channel": t.channel or "shared",
        "name": t.name,
        "purpose": t.purpose,
        "metaTemplateName": t.meta_template_name,
        "metaLanguage": t.meta_language,
        "metaCategory": t.meta_category,
        "metaStatus": t.meta_status,
        "bodyPreview": t.body_preview,
        "variables": t.variables,
        "isActive": t.is_active,
    }


@router.post("/tenants/newax/seed-campaign-templates")
async def seed_newax_campaigns(session: AsyncSession = Depends(get_db_session)):
    """Installs/refreshes NEWAX's campaign message templates (tenant #1)."""
    count = await seed_newax_campaign_templates(session)
    return {"status": "ok", "tenantKey": "newax", "templateCount": count}


@router.get("/campaign-templates")
async def list_campaign_templates(
    tenant_key: str | None = None,
    channel: str | None = None,
    session: AsyncSession = Depends(get_db_session),
):
    """
    `channel` filters to templates usable on that channel: an exact match,
    plus every shared template (channel == "") since those are reusable
    across any channel the tenant has.
    """
    stmt = select(CampaignTemplate).order_by(CampaignTemplate.created_at)
    if tenant_key:
        stmt = stmt.where(CampaignTemplate.tenant_key == tenant_key)
    if channel:
        stmt = stmt.where(CampaignTemplate.channel.in_([channel, ""]))
    templates = (await session.execute(stmt)).scalars().all()
    return [_serialize_campaign_template(t) for t in templates]


@router.post("/campaign-templates/sync-status")
async def sync_campaign_template_status(
    tenant_key: str = "newax", session: AsyncSession = Depends(get_db_session)
):
    """
    Pulls each template's real approval status from Meta's Graph API and
    updates the local rows -- replaces the previous hardcoded stub.
    """
    config = await get_whatsapp_config(session, tenant_key=tenant_key)
    if not config.access_token or not config.waba_id:
        raise HTTPException(status_code=503, detail="WhatsApp credentials not configured.")

    async with httpx.AsyncClient(timeout=20.0) as client:
        res = await client.get(
            f"https://graph.facebook.com/v21.0/{config.waba_id}/message_templates",
            params={"fields": "name,status,category,language"},
            headers={"Authorization": f"Bearer {config.access_token}"},
        )
        res.raise_for_status()
        remote = {t["name"]: t for t in res.json().get("data", [])}

    stmt = select(CampaignTemplate).where(CampaignTemplate.tenant_key == tenant_key)
    templates = (await session.execute(stmt)).scalars().all()
    updated = 0
    for t in templates:
        match = remote.get(t.meta_template_name)
        if match and match["status"] != t.meta_status:
            t.meta_status = match["status"]
            updated += 1
    await session.flush()
    return {
        "status": "ok",
        "tenantKey": tenant_key,
        "checked": len(templates),
        "updated": updated,
        "remoteTemplateCount": len(remote),
    }
