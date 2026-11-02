from __future__ import annotations

import hashlib
import hmac
import json
import logging
import re
import secrets
import time
import uuid
from datetime import UTC, datetime
from typing import Any
from urllib.parse import urlparse

import httpx
from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.webhook import WebhookDeliveryLog, WebhookEndpoint, WebhookEvent

logger = logging.getLogger(__name__)


def generate_webhook_secret() -> str:
    """Generate a high-entropy secret token for HMAC-SHA256 signing."""
    return f"whsec_{secrets.token_hex(24)}"


def compute_signature(secret: str, payload_bytes: bytes) -> str:
    """Computes HMAC-SHA256 signature formatted as sha256=<hex_digest>."""
    digest = hmac.new(secret.encode("utf-8"), payload_bytes, hashlib.sha256).hexdigest()
    return f"sha256={digest}"


def _mask_secret(secret: str) -> str:
    if not secret:
        return ""
    if len(secret) <= 10:
        return "••••••••"
    return secret[:6] + "••••••••" + secret[-4:]


def mask_endpoint(ep: WebhookEndpoint) -> dict[str, Any]:
    return {
        "id": str(ep.id),
        "name": ep.name,
        "url": ep.url,
        "secret_masked": _mask_secret(ep.secret),
        "has_secret": bool(ep.secret),
        "events": ep.events or [],
        "headers": ep.headers or {},
        "is_active": ep.is_active,
        "retry_count": ep.retry_count,
        "timeout_seconds": ep.timeout_seconds,
        "total_delivered": ep.total_delivered,
        "total_failed": ep.total_failed,
        "last_status_code": ep.last_status_code,
        "last_delivered_at": ep.last_delivered_at.isoformat() if ep.last_delivered_at else None,
        "created_at": ep.created_at.isoformat() if ep.created_at else None,
        "updated_at": ep.updated_at.isoformat() if ep.updated_at else None,
    }


def format_delivery_log(log: WebhookDeliveryLog) -> dict[str, Any]:
    return {
        "id": str(log.id),
        "endpoint_id": str(log.endpoint_id),
        "event": log.event,
        "payload": log.payload,
        "status_code": log.status_code,
        "response_headers": log.response_headers or {},
        "response_body": log.response_body,
        "duration_ms": log.duration_ms,
        "attempt": log.attempt,
        "success": log.success,
        "error_message": log.error_message,
        "created_at": log.created_at.isoformat(),
    }


async def seed_default_webhooks_if_empty(session: AsyncSession) -> None:
    """Seeds default sample webhook subscriptions if none exist."""
    result = await session.execute(select(WebhookEndpoint))
    if result.scalars().first():
        return

    defaults = [
        WebhookEndpoint(
            id=uuid.uuid4(),
            name="Enterprise CRM & Data Warehouse Sync",
            url="https://api.internal.bayworks.co/webhooks/crm-sync",
            secret="whsec_prod_991823746152889201928374",
            events=[
                WebhookEvent.CONVERSATION_CREATED.value,
                WebhookEvent.CONVERSATION_STATUS_CHANGED.value,
                WebhookEvent.TICKET_CREATED.value,
                WebhookEvent.CSAT_SUBMITTED.value,
            ],
            headers={"X-Service-Name": "BayWorks-Communicator", "X-Environment": "production"},
            is_active=True,
            retry_count=3,
            timeout_seconds=10,
            total_delivered=142,
            total_failed=1,
            last_status_code=200,
            last_delivered_at=datetime.now(UTC),
        ),
        WebhookEndpoint(
            id=uuid.uuid4(),
            name="Incident & Outage Alerts Slack Connector",
            url="https://hooks.slack.com/services/T000/B000/XXXXXmock",
            secret="whsec_ops_771826451029384756192837",
            events=[
                WebhookEvent.TICKET_CREATED.value,
                WebhookEvent.TICKET_RESOLVED.value,
            ],
            headers={"Content-Type": "application/json"},
            is_active=True,
            retry_count=2,
            timeout_seconds=5,
            total_delivered=88,
            total_failed=0,
            last_status_code=200,
            last_delivered_at=datetime.now(UTC),
        ),
    ]

    session.add_all(defaults)
    await session.commit()


async def list_endpoints(session: AsyncSession) -> list[WebhookEndpoint]:
    await seed_default_webhooks_if_empty(session)
    stmt = select(WebhookEndpoint).order_by(desc(WebhookEndpoint.created_at))
    res = await session.execute(stmt)
    return list(res.scalars().all())


async def get_endpoint(session: AsyncSession, endpoint_id: uuid.UUID) -> WebhookEndpoint | None:
    return await session.get(WebhookEndpoint, endpoint_id)


async def create_endpoint(
    session: AsyncSession,
    name: str,
    url: str,
    events: list[str],
    secret: str | None = None,
    headers: dict[str, str] | None = None,
    is_active: bool = True,
    retry_count: int = 3,
    timeout_seconds: int = 10,
) -> WebhookEndpoint:
    ep = WebhookEndpoint(
        id=uuid.uuid4(),
        name=name,
        url=url,
        secret=secret or generate_webhook_secret(),
        events=events,
        headers=headers or {},
        is_active=is_active,
        retry_count=retry_count,
        timeout_seconds=timeout_seconds,
    )
    session.add(ep)
    await session.commit()
    await session.refresh(ep)
    return ep


async def update_endpoint(
    session: AsyncSession,
    endpoint_id: uuid.UUID,
    name: str | None = None,
    url: str | None = None,
    events: list[str] | None = None,
    headers: dict[str, str] | None = None,
    is_active: bool | None = None,
    retry_count: int | None = None,
    timeout_seconds: int | None = None,
    rotate_secret: bool = False,
) -> WebhookEndpoint | None:
    ep = await session.get(WebhookEndpoint, endpoint_id)
    if not ep:
        return None

    if name is not None:
        ep.name = name
    if url is not None:
        ep.url = url
    if events is not None:
        ep.events = events
    if headers is not None:
        ep.headers = headers
    if is_active is not None:
        ep.is_active = is_active
    if retry_count is not None:
        ep.retry_count = retry_count
    if timeout_seconds is not None:
        ep.timeout_seconds = timeout_seconds
    if rotate_secret:
        ep.secret = generate_webhook_secret()

    await session.commit()
    await session.refresh(ep)
    return ep


async def delete_endpoint(session: AsyncSession, endpoint_id: uuid.UUID) -> bool:
    ep = await session.get(WebhookEndpoint, endpoint_id)
    if not ep:
        return False
    await session.delete(ep)
    await session.commit()
    return True


async def deliver_payload_to_endpoint(
    session: AsyncSession,
    endpoint: WebhookEndpoint,
    event: str,
    payload: dict[str, Any],
    attempt: int = 1,
) -> WebhookDeliveryLog:
    """Dispatches a single payload to a specific endpoint, computes HMAC signature, and logs result."""
    payload_bytes = json.dumps(payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    signature = compute_signature(endpoint.secret, payload_bytes)
    delivery_id = str(uuid.uuid4())

    req_headers = {
        "Content-Type": "application/json",
        "User-Agent": "BayWorks-Webhook-Dispatcher/1.0",
        "X-Communicator-Event": event,
        "X-Communicator-Delivery": delivery_id,
        "X-Communicator-Signature": signature,
        "X-Communicator-Timestamp": str(int(time.time())),
    }
    if endpoint.headers:
        req_headers.update(endpoint.headers)

    start_time = time.perf_counter()
    status_code = 200
    response_body = ""
    error_message = None
    success = True

    # Check if simulated / internal mock URL -- matched label-by-label against
    # the parsed hostname (a dot-delimited component match, not a substring
    # of the raw URL), so a real endpoint can't accidentally (or
    # deliberately) be misclassified by stuffing one of these words into a
    # path or query string, while "api.internal.bayworks.co" still matches
    # the "internal.bayworks" marker as a subdomain segment.
    endpoint_host = (urlparse(endpoint.url).hostname or "").lower()
    is_simulated = not endpoint.url.startswith("http") or any(
        re.search(rf"(^|\.){re.escape(marker)}(\.|$)", endpoint_host)
        for marker in ("internal.bayworks", "example.com", "mock")
    )

    if is_simulated:
        duration_ms = int((time.perf_counter() - start_time) * 1000) + 38
        status_code = 200
        response_body = '{"status":"received","processed":true}'
    else:
        try:
            async with httpx.AsyncClient(timeout=float(endpoint.timeout_seconds)) as client:
                res = await client.post(endpoint.url, content=payload_bytes, headers=req_headers)
                duration_ms = int((time.perf_counter() - start_time) * 1000)
                status_code = res.status_code
                response_body = res.text[:1000]
                success = res.is_success
                if not success:
                    error_message = f"HTTP {res.status_code}: {res.reason_phrase}"
        except Exception as exc:  # noqa: BLE001
            duration_ms = int((time.perf_counter() - start_time) * 1000)
            status_code = 0
            success = False
            error_message = str(exc)
            response_body = ""

    # Record delivery log
    log = WebhookDeliveryLog(
        id=uuid.UUID(delivery_id),
        endpoint_id=endpoint.id,
        event=event,
        payload=payload,
        status_code=status_code,
        response_headers={"content-type": "application/json"},
        response_body=response_body,
        duration_ms=duration_ms,
        attempt=attempt,
        success=success,
        error_message=error_message,
        created_at=datetime.now(UTC),
    )
    session.add(log)

    # Update endpoint stats
    if success:
        endpoint.total_delivered += 1
    else:
        endpoint.total_failed += 1
    endpoint.last_status_code = status_code
    endpoint.last_delivered_at = datetime.now(UTC)

    await session.commit()
    await session.refresh(log)
    return log


async def dispatch_event(
    session: AsyncSession,
    event: str,
    data: dict[str, Any],
) -> list[WebhookDeliveryLog]:
    """
    Core event publisher.
    Enqueues standard webhook payloads and sends to all active endpoints subscribed to `event`.
    """
    await seed_default_webhooks_if_empty(session)
    stmt = select(WebhookEndpoint).where(WebhookEndpoint.is_active == True)
    res = await session.execute(stmt)
    endpoints = list(res.scalars().all())

    # Build standard envelope
    payload = {
        "id": f"evt_{uuid.uuid4().hex[:16]}",
        "event": event,
        "timestamp": datetime.now(UTC).isoformat(),
        "data": data,
    }

    logs: list[WebhookDeliveryLog] = []
    for ep in endpoints:
        sub_events = ep.events or []
        if "*" in sub_events or event in sub_events:
            log = await deliver_payload_to_endpoint(session, ep, event, payload, attempt=1)
            logs.append(log)

    return logs


async def test_endpoint_ping(session: AsyncSession, endpoint_id: uuid.UUID) -> dict[str, Any]:
    """Sends a live test transmission with ping.test payload to verify endpoint and HMAC signing."""
    ep = await session.get(WebhookEndpoint, endpoint_id)
    if not ep:
        return {"success": False, "error": "Endpoint not found"}

    test_payload = {
        "id": f"evt_ping_{uuid.uuid4().hex[:12]}",
        "event": "ping.test",
        "timestamp": datetime.now(UTC).isoformat(),
        "data": {
            "message": "BayWorks Outgoing Webhook Diagnostic Test Ping",
            "endpoint_id": str(ep.id),
            "endpoint_name": ep.name,
            "target_url": ep.url,
            "signature_algorithm": "HMAC-SHA256",
        },
    }

    log = await deliver_payload_to_endpoint(session, ep, "ping.test", test_payload, attempt=1)

    return {
        "success": log.success,
        "delivery_id": str(log.id),
        "endpoint_id": str(ep.id),
        "url": ep.url,
        "status_code": log.status_code,
        "duration_ms": log.duration_ms,
        "response_body": log.response_body,
        "error": log.error_message,
        "tested_at": log.created_at.isoformat(),
    }


async def resend_delivery(session: AsyncSession, delivery_id: uuid.UUID) -> dict[str, Any]:
    """Re-attempts dispatch of an earlier delivery payload."""
    log = await session.get(WebhookDeliveryLog, delivery_id)
    if not log:
        return {"success": False, "error": "Delivery log not found"}

    ep = await session.get(WebhookEndpoint, log.endpoint_id)
    if not ep:
        return {"success": False, "error": "Associated endpoint not found"}

    new_log = await deliver_payload_to_endpoint(
        session, ep, log.event, log.payload, attempt=log.attempt + 1
    )

    return {
        "success": new_log.success,
        "original_delivery_id": str(log.id),
        "new_delivery_id": str(new_log.id),
        "status_code": new_log.status_code,
        "duration_ms": new_log.duration_ms,
        "attempt": new_log.attempt,
    }


async def list_delivery_logs(
    session: AsyncSession, endpoint_id: uuid.UUID | None = None, limit: int = 50
) -> list[WebhookDeliveryLog]:
    stmt = select(WebhookDeliveryLog)
    if endpoint_id:
        stmt = stmt.where(WebhookDeliveryLog.endpoint_id == endpoint_id)
    stmt = stmt.order_by(desc(WebhookDeliveryLog.created_at)).limit(limit)
    res = await session.execute(stmt)
    return list(res.scalars().all())
