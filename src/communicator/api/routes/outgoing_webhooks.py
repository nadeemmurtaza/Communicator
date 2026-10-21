from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.webhook import WebhookEvent
from communicator.services.webhook_service import (
    create_endpoint,
    delete_endpoint,
    format_delivery_log,
    get_endpoint,
    list_delivery_logs,
    list_endpoints,
    mask_endpoint,
    resend_delivery,
    test_endpoint_ping,
    update_endpoint,
)

router = APIRouter(prefix="/outgoing-webhooks", tags=["outgoing-webhooks"])


class CreateWebhookRequest(BaseModel):
    name: str = Field(..., description="Descriptive name for webhook subscriber")
    url: str = Field(..., description="Target HTTPS callback URL")
    events: list[str] = Field(..., description="List of event names or ['*']")
    secret: str | None = Field(None, description="Optional custom HMAC secret; generated if omitted")
    headers: dict[str, str] = Field(default_factory=dict, description="Custom HTTP headers")
    is_active: bool = True
    retry_count: int = 3
    timeout_seconds: int = 10


class UpdateWebhookRequest(BaseModel):
    name: str | None = None
    url: str | None = None
    events: list[str] | None = None
    headers: dict[str, str] | None = None
    is_active: bool | None = None
    retry_count: int | None = None
    timeout_seconds: int | None = None
    rotate_secret: bool = False


@router.get("")
async def get_all_webhook_endpoints(
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    endpoints = await list_endpoints(session)
    return [mask_endpoint(ep) for ep in endpoints]


@router.get("/events/catalog")
async def get_events_catalog() -> list[dict[str, Any]]:
    return [
        {
            "event": WebhookEvent.CONVERSATION_CREATED.value,
            "description": "Triggered when a new conversation thread is initiated by customer or agent.",
            "category": "Conversations",
        },
        {
            "event": WebhookEvent.CONVERSATION_STATUS_CHANGED.value,
            "description": "Triggered when conversation state transitions (open, pending, resolved, snoozed).",
            "category": "Conversations",
        },
        {
            "event": WebhookEvent.CONVERSATION_ASSIGNED.value,
            "description": "Triggered when conversation is assigned to an agent or team queue.",
            "category": "Conversations",
        },
        {
            "event": WebhookEvent.MESSAGE_CREATED.value,
            "description": "Triggered on every inbound message from customer or outbound agent reply.",
            "category": "Messages",
        },
        {
            "event": WebhookEvent.TICKET_CREATED.value,
            "description": "Triggered when an issue ticket or tracker outage ticket is created.",
            "category": "Ticketing",
        },
        {
            "event": WebhookEvent.TICKET_UPDATED.value,
            "description": "Triggered when ticket department, priority, or assignee is modified.",
            "category": "Ticketing",
        },
        {
            "event": WebhookEvent.TICKET_RESOLVED.value,
            "description": "Triggered when ticket is closed and marked resolved.",
            "category": "Ticketing",
        },
        {
            "event": WebhookEvent.CSAT_SUBMITTED.value,
            "description": "Triggered when customer submits satisfaction score (1-5 stars) and comment.",
            "category": "Satisfaction",
        },
        {
            "event": WebhookEvent.CONTACT_CREATED.value,
            "description": "Triggered when a new contact record is created or discovered.",
            "category": "Contacts",
        },
        {
            "event": WebhookEvent.CONTACT_UPDATED.value,
            "description": "Triggered when contact attributes, tags, or preferences change.",
            "category": "Contacts",
        },
    ]


@router.get("/deliveries")
async def get_all_deliveries(
    limit: int = Query(50, ge=1, le=200),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    logs = await list_delivery_logs(session, limit=limit)
    return [format_delivery_log(l) for l in logs]


@router.get("/{endpoint_id}")
async def get_single_endpoint(
    endpoint_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    ep = await get_endpoint(session, endpoint_id)
    if not ep:
        raise HTTPException(status_code=404, detail="Webhook endpoint not found")
    return mask_endpoint(ep)


@router.post("", status_code=201)
async def create_new_endpoint(
    payload: CreateWebhookRequest,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    ep = await create_endpoint(
        session=session,
        name=payload.name,
        url=payload.url,
        events=payload.events,
        secret=payload.secret,
        headers=payload.headers,
        is_active=payload.is_active,
        retry_count=payload.retry_count,
        timeout_seconds=payload.timeout_seconds,
    )
    return mask_endpoint(ep)


@router.put("/{endpoint_id}")
async def update_existing_endpoint(
    endpoint_id: uuid.UUID,
    payload: UpdateWebhookRequest,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    ep = await update_endpoint(
        session=session,
        endpoint_id=endpoint_id,
        name=payload.name,
        url=payload.url,
        events=payload.events,
        headers=payload.headers,
        is_active=payload.is_active,
        retry_count=payload.retry_count,
        timeout_seconds=payload.timeout_seconds,
        rotate_secret=payload.rotate_secret,
    )
    if not ep:
        raise HTTPException(status_code=404, detail="Webhook endpoint not found")
    return mask_endpoint(ep)


@router.delete("/{endpoint_id}")
async def delete_existing_endpoint(
    endpoint_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    success = await delete_endpoint(session, endpoint_id)
    if not success:
        raise HTTPException(status_code=404, detail="Webhook endpoint not found")
    return {"success": True, "deleted_id": str(endpoint_id)}


@router.post("/{endpoint_id}/test")
async def run_endpoint_test(
    endpoint_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    res = await test_endpoint_ping(session, endpoint_id)
    if not res.get("success") and res.get("error") == "Endpoint not found":
        raise HTTPException(status_code=404, detail="Webhook endpoint not found")
    return res


@router.get("/{endpoint_id}/deliveries")
async def get_endpoint_deliveries(
    endpoint_id: uuid.UUID,
    limit: int = Query(50, ge=1, le=100),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    logs = await list_delivery_logs(session, endpoint_id=endpoint_id, limit=limit)
    return [format_delivery_log(l) for l in logs]


@router.post("/deliveries/{delivery_id}/resend")
async def resend_delivery_attempt(
    delivery_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    res = await resend_delivery(session, delivery_id)
    if not res.get("success") and res.get("error") == "Delivery log not found":
        raise HTTPException(status_code=404, detail="Delivery log not found")
    return res
