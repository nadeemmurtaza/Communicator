from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import stripe_service

router = APIRouter(prefix="/integrations/stripe", tags=["stripe-billing"])


class UpdateStripeConfigRequest(BaseModel):
    account_name: str | None = None
    api_key: str | None = None
    publishable_key: str | None = None
    webhook_secret: str | None = None
    is_enabled: bool | None = None
    test_mode: bool | None = None
    default_currency: str | None = None


class ProcessRefundRequest(BaseModel):
    charge_or_invoice_id: str = Field(..., description="Stripe charge or invoice ID to refund")
    amount_cents: int | None = Field(None, description="Optional refund amount in cents; full amount if omitted")
    reason: str = Field("requested_by_customer", description="Reason for refund (duplicate, fraudulent, requested_by_customer)")
    actor_email: str = Field("agent@newax.com", description="Email of agent issuing the refund")
    contact_id: str | None = Field(None, description="Optional associated contact UUID")
    customer_identifier: str = Field("", description="Optional customer email or identifier")


class CreatePortalSessionRequest(BaseModel):
    identifier: str = Field(..., description="Contact UUID, email address, or customer ID")
    return_url: str | None = Field(None, description="URL where customer returns after billing portal")
    actor_email: str = Field("agent@newax.com", description="Email of agent creating the link")


@router.get("/config")
async def get_stripe_config(session: AsyncSession = Depends(get_db_session)) -> dict[str, Any]:
    """Retrieve Stripe configuration (with keys safely masked)."""
    cfg = await stripe_service.get_or_create_config(session)
    return stripe_service.format_stripe_config(cfg)


@router.post("/config")
async def update_stripe_config(
    payload: UpdateStripeConfigRequest, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Update Stripe API keys, test mode, or currency."""
    updates = payload.model_dump(exclude_unset=True)
    cfg = await stripe_service.update_config(session, updates)
    return stripe_service.format_stripe_config(cfg)


@router.post("/test")
async def test_stripe_connection(session: AsyncSession = Depends(get_db_session)) -> dict[str, Any]:
    """Verify Stripe API connection credentials or status."""
    return await stripe_service.test_stripe_connection(session)


@router.get("/billing/{identifier}")
async def get_customer_billing(
    identifier: str, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """
    Retrieve customer's subscription, plan tier, MRR, default payment card,
    renewal date, and lifetime value.
    """
    return await stripe_service.get_customer_billing(session, identifier)


@router.get("/invoices/{identifier}")
async def get_customer_invoices(
    identifier: str,
    limit: int = Query(10, ge=1, le=50),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    """Retrieve recent Stripe invoices with status and PDF download links."""
    return await stripe_service.get_customer_invoices(session, identifier, limit=limit)


@router.post("/refund")
async def process_refund(
    payload: ProcessRefundRequest, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Issue a 1-click refund for an invoice or charge and create an audit log."""
    try:
        return await stripe_service.process_refund(
            session=session,
            charge_or_invoice_id=payload.charge_or_invoice_id,
            amount_cents=payload.amount_cents,
            reason=payload.reason,
            actor_email=payload.actor_email,
            contact_id=payload.contact_id,
            customer_identifier=payload.customer_identifier,
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/portal-session")
async def create_portal_session(
    payload: CreatePortalSessionRequest, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Generate a Stripe Customer Portal link to share with customer."""
    try:
        return await stripe_service.create_customer_portal_session(
            session=session,
            identifier=payload.identifier,
            return_url=payload.return_url,
            actor_email=payload.actor_email,
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/logs/{identifier}")
async def get_billing_logs(
    identifier: str,
    limit: int = Query(25, ge=1, le=100),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    """Retrieve audit trail of billing actions executed for this contact."""
    logs = await stripe_service.list_billing_action_logs(session, identifier, limit=limit)
    return [stripe_service.format_billing_action_log(log) for log in logs]
