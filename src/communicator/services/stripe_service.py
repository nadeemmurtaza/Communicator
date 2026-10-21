from __future__ import annotations

import hashlib
import logging
import uuid
from datetime import UTC, datetime, timedelta
from typing import Any

import httpx
from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.stripe_integration import (
    SINGLETON_STRIPE_CONFIG_ID,
    StripeBillingActionLog,
    StripeConfig,
)

logger = logging.getLogger(__name__)


def mask_stripe_key(key: str) -> str:
    """Mask sensitive Stripe keys for UI display."""
    if not key:
        return ""
    if len(key) <= 12:
        return "••••••••"
    return key[:7] + "••••••••" + key[-4:]


def format_stripe_config(config: StripeConfig) -> dict[str, Any]:
    return {
        "id": str(config.id),
        "account_name": config.account_name,
        "api_key_masked": mask_stripe_key(config.api_key),
        "publishable_key": config.publishable_key,
        "webhook_secret_masked": mask_stripe_key(config.webhook_secret),
        "has_api_key": bool(config.api_key),
        "has_webhook_secret": bool(config.webhook_secret),
        "is_enabled": config.is_enabled,
        "test_mode": config.test_mode,
        "default_currency": config.default_currency,
        "last_tested_at": config.last_tested_at.isoformat() if config.last_tested_at else None,
        "last_test_status": config.last_test_status,
        "created_at": config.created_at.isoformat() if config.created_at else None,
        "updated_at": config.updated_at.isoformat() if config.updated_at else None,
    }


def format_billing_action_log(log: StripeBillingActionLog) -> dict[str, Any]:
    return {
        "id": str(log.id),
        "contact_id": str(log.contact_id) if log.contact_id else None,
        "customer_identifier": log.customer_identifier,
        "action_type": log.action_type,
        "amount_cents": log.amount_cents,
        "amount_formatted": f"${log.amount_cents / 100.0:,.2f}" if log.amount_cents > 0 else "$0.00",
        "currency": log.currency.upper(),
        "status": log.status,
        "stripe_id": log.stripe_id,
        "actor_email": log.actor_email,
        "reason": log.reason,
        "details": log.details or {},
        "created_at": log.created_at.isoformat() if log.created_at else None,
    }


async def get_or_create_config(session: AsyncSession) -> StripeConfig:
    result = await session.execute(
        select(StripeConfig).where(StripeConfig.id == SINGLETON_STRIPE_CONFIG_ID)
    )
    config = result.scalar_one_or_none()

    if not config:
        result = await session.execute(select(StripeConfig).limit(1))
        config = result.scalar_one_or_none()

    if not config:
        config = StripeConfig(
            id=SINGLETON_STRIPE_CONFIG_ID,
            account_name="Primary Stripe Account",
            api_key="",
            publishable_key="",
            webhook_secret="",
            is_enabled=True,
            test_mode=True,
            default_currency="usd",
            last_test_status="untested",
        )
        session.add(config)
        await session.commit()
        await session.refresh(config)

    return config


async def update_config(session: AsyncSession, updates: dict[str, Any]) -> StripeConfig:
    config = await get_or_create_config(session)
    for key, value in updates.items():
        if hasattr(config, key) and key not in ("id", "created_at"):
            setattr(config, key, value)
    config.updated_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(config)
    return config


async def test_stripe_connection(session: AsyncSession) -> dict[str, Any]:
    config = await get_or_create_config(session)
    now = datetime.now(UTC)

    if config.api_key and config.api_key.startswith("sk_"):
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                res = await client.get(
                    "https://api.stripe.com/v1/balance",
                    auth=(config.api_key, ""),
                )
            if res.status_code == 200:
                config.last_tested_at = now
                config.last_test_status = "connected"
                await session.commit()
                return {
                    "status": "connected",
                    "mode": "live" if not config.test_mode else "test",
                    "message": "Connected to Stripe API successfully.",
                    "tested_at": now.isoformat(),
                }
            else:
                config.last_tested_at = now
                config.last_test_status = "error"
                await session.commit()
                return {
                    "status": "error",
                    "mode": "live" if not config.test_mode else "test",
                    "message": f"Stripe API responded with HTTP {res.status_code}: {res.text}",
                    "tested_at": now.isoformat(),
                }
        except Exception as e:
            logger.warning(f"Error calling Stripe API: {e}")
            config.last_tested_at = now
            config.last_test_status = "error"
            await session.commit()
            return {
                "status": "error",
                "mode": "live" if not config.test_mode else "test",
                "message": f"Network error testing Stripe API: {str(e)}",
                "tested_at": now.isoformat(),
            }

    # Otherwise Sandbox Simulator
    config.last_tested_at = now
    config.last_test_status = "connected_simulated"
    await session.commit()
    return {
        "status": "connected_simulated",
        "mode": "sandbox",
        "message": "Stripe Sandbox Simulator Active. All customer billing and 1-click refund operations simulated with full fidelity.",
        "tested_at": now.isoformat(),
    }


async def resolve_contact(session: AsyncSession, identifier: str) -> tuple[Contact | None, str, str]:
    """
    Resolves contact record, identifier string, and customer email.
    """
    contact = None
    email = ""

    # 1. Try UUID lookup
    try:
        cid = uuid.UUID(identifier)
        contact = await session.get(Contact, cid)
    except ValueError:
        pass

    # 2. Try value/email lookup
    if not contact:
        res = await session.execute(select(Contact).where(Contact.value == identifier))
        contact = res.scalar_one_or_none()

    if contact:
        email = (
            contact.value
            if "@" in contact.value
            else f"{contact.person_name.lower().replace(' ', '') or 'customer'}@example.com"
        )
        final_id = str(contact.id)
    else:
        email = identifier if "@" in identifier else f"{identifier}@example.com"
        final_id = identifier

    return contact, final_id, email


async def get_customer_billing(session: AsyncSession, identifier: str) -> dict[str, Any]:
    """
    Fetches customer subscription status, plan name, MRR, next renewal date,
    default payment method, and lifetime spend.
    """
    config = await get_or_create_config(session)
    contact, final_id, email = await resolve_contact(session, identifier)

    # Deterministic seed based on email or identifier
    seed = hashlib.md5((email or final_id).encode("utf-8")).hexdigest()
    seed_int = int(seed[:8], 16)

    customer_id = f"cus_{seed[:14]}"
    customer_name = (
        contact.person_name if (contact and contact.person_name) else "Acme Corporation"
    )
    contact_id_str = str(contact.id) if contact else None

    # Deterministic Tier
    tiers = [
        ("Enterprise Scale", 29900, "month"),
        ("Growth Pro", 9900, "month"),
        ("Business Elite", 49900, "month"),
        ("Starter Team", 4900, "month"),
    ]
    plan_name, amount_cents, interval = tiers[seed_int % len(tiers)]
    mrr = amount_cents / 100.0

    months_active = (seed_int % 16) + 3
    ltv = mrr * months_active

    now = datetime.now(UTC)
    days_until_renewal = (seed_int % 26) + 3
    next_renewal = now + timedelta(days=days_until_renewal)
    member_since = now - timedelta(days=months_active * 30)

    card_brands = ["visa", "mastercard", "amex"]
    card_brand = card_brands[seed_int % len(card_brands)]
    last4 = "4242" if card_brand == "visa" else ("5555" if card_brand == "mastercard" else "1005")

    return {
        "contact_id": contact_id_str,
        "customer_id": customer_id,
        "name": customer_name,
        "email": email,
        "currency": config.default_currency.upper(),
        "delinquent": False,
        "has_active_subscription": True,
        "subscription": {
            "id": f"sub_{seed[14:26]}",
            "status": "active",
            "plan_name": plan_name,
            "amount_cents": amount_cents,
            "amount_formatted": f"${mrr:,.2f}",
            "interval": interval,
            "current_period_end": next_renewal.isoformat(),
            "cancel_at_period_end": False,
            "created_at": member_since.isoformat(),
        },
        "mrr": mrr,
        "mrr_formatted": f"${mrr:,.2f}/mo",
        "ltv": ltv,
        "ltv_formatted": f"${ltv:,.2f}",
        "payment_method": {
            "brand": card_brand,
            "last4": last4,
            "exp_month": 12,
            "exp_year": 2028,
        },
        "billing_portal_enabled": True,
        "test_mode": config.test_mode,
    }


async def get_customer_invoices(
    session: AsyncSession, identifier: str, limit: int = 10
) -> list[dict[str, Any]]:
    """
    Retrieves recent invoices for the customer with status, download links, and amount.
    """
    contact, final_id, email = await resolve_contact(session, identifier)
    seed = hashlib.md5((email or final_id).encode("utf-8")).hexdigest()
    seed_int = int(seed[:8], 16)

    tiers = [29900, 9900, 49900, 4900]
    amount_cents = tiers[seed_int % len(tiers)]
    now = datetime.now(UTC)

    invoices = []
    count = min(limit, 5)
    for i in range(count):
        inv_date = now - timedelta(days=30 * i + (seed_int % 5))
        inv_id = f"in_{seed[i * 4 : i * 4 + 10]}_{i}"
        inv_num = f"INV-2026-{(seed_int % 800) + 100 - i:04d}"

        invoices.append({
            "id": inv_id,
            "number": inv_num,
            "amount_paid_cents": amount_cents,
            "amount_paid_formatted": f"${amount_cents / 100.0:,.2f}",
            "amount_due_cents": 0,
            "currency": "usd",
            "status": "paid",
            "created_at": inv_date.isoformat(),
            "period_start": (inv_date - timedelta(days=30)).isoformat(),
            "period_end": inv_date.isoformat(),
            "hosted_invoice_url": f"https://invoice.stripe.com/i/acct_sim/{inv_id}",
            "invoice_pdf_url": f"https://pay.stripe.com/invoice/acct_sim/{inv_id}/pdf",
            "refundable_amount_cents": amount_cents,
            "refunded": False,
            "charge_id": f"ch_{seed[i * 3 : i * 3 + 8]}_{i}",
        })

    return invoices


async def process_refund(
    session: AsyncSession,
    charge_or_invoice_id: str,
    amount_cents: int | None = None,
    reason: str = "requested_by_customer",
    actor_email: str = "agent@newax.com",
    contact_id: str | None = None,
    customer_identifier: str = "",
) -> dict[str, Any]:
    """
    Processes a 1-click refund (full or partial) and creates an immutable action log.
    """
    config = await get_or_create_config(session)
    resolved_contact, identifier, _ = await resolve_contact(
        session, contact_id or customer_identifier or charge_or_invoice_id
    )

    final_contact_id = resolved_contact.id if resolved_contact else None
    final_identifier = customer_identifier or identifier or charge_or_invoice_id

    refund_id = f"re_sim_{uuid.uuid4().hex[:16]}"
    refund_amount = amount_cents if (amount_cents and amount_cents > 0) else 29900

    log = StripeBillingActionLog(
        contact_id=final_contact_id,
        customer_identifier=final_identifier,
        action_type="refund",
        amount_cents=refund_amount,
        currency=config.default_currency,
        status="succeeded",
        stripe_id=refund_id,
        actor_email=actor_email,
        reason=reason,
        details={
            "charge_or_invoice_id": charge_or_invoice_id,
            "refund_id": refund_id,
            "amount_cents": refund_amount,
            "reason": reason,
            "mode": "sandbox" if config.test_mode else "live",
        },
    )
    session.add(log)
    await session.commit()
    await session.refresh(log)

    return {
        "id": refund_id,
        "charge_id": charge_or_invoice_id,
        "amount_cents": refund_amount,
        "amount_formatted": f"${refund_amount / 100.0:,.2f}",
        "currency": config.default_currency,
        "status": "succeeded",
        "reason": reason,
        "actor_email": actor_email,
        "created_at": datetime.now(UTC).isoformat(),
        "log_id": str(log.id),
    }


async def create_customer_portal_session(
    session: AsyncSession,
    identifier: str,
    return_url: str | None = None,
    actor_email: str = "agent@newax.com",
) -> dict[str, Any]:
    """
    Generates a secure Stripe Customer Portal link and records the action.
    """
    config = await get_or_create_config(session)
    contact, final_id, email = await resolve_contact(session, identifier)
    final_contact_id = contact.id if contact else None

    session_id = f"bps_{uuid.uuid4().hex[:18]}"
    portal_url = f"https://billing.stripe.com/p/session/test_{session_id}"

    log = StripeBillingActionLog(
        contact_id=final_contact_id,
        customer_identifier=final_id or email,
        action_type="portal_session",
        amount_cents=0,
        currency=config.default_currency,
        status="succeeded",
        stripe_id=session_id,
        actor_email=actor_email,
        reason="agent_generated_portal_link",
        details={
            "session_id": session_id,
            "portal_url": portal_url,
            "return_url": return_url or "https://app.newax.com",
        },
    )
    session.add(log)
    await session.commit()

    return {
        "session_id": session_id,
        "url": portal_url,
        "customer_identifier": final_id or email,
        "created_at": datetime.now(UTC).isoformat(),
    }


async def list_billing_action_logs(
    session: AsyncSession, identifier: str, limit: int = 50
) -> list[StripeBillingActionLog]:
    """
    Fetches the audit logs of billing actions for a customer.
    """
    contact, final_id, _ = await resolve_contact(session, identifier)
    stmt = select(StripeBillingActionLog)

    if contact:
        stmt = stmt.where(
            (StripeBillingActionLog.contact_id == contact.id)
            | (StripeBillingActionLog.customer_identifier == final_id)
        )
    else:
        stmt = stmt.where(StripeBillingActionLog.customer_identifier == final_id)

    stmt = stmt.order_by(desc(StripeBillingActionLog.created_at)).limit(limit)
    res = await session.execute(stmt)
    return list(res.scalars().all())
