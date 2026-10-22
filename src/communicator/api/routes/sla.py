from __future__ import annotations

import uuid
from datetime import UTC, datetime, timedelta
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.sla import SLAPolicy
from communicator.services import sla_service

router = APIRouter(tags=["Dynamic SLA Management & Breach Detection"])


class SLAPolicyCreateIn(BaseModel):
    name: str
    description: str = ""
    first_response_time_threshold: int = 60
    resolution_time_threshold: int = 1440
    only_during_business_hours: bool = False
    priorities: list[str] = []
    inbox_ids: list[str] = []


class SLAPolicyUpdateIn(BaseModel):
    name: str | None = None
    description: str | None = None
    first_response_time_threshold: int | None = None
    resolution_time_threshold: int | None = None
    only_during_business_hours: bool | None = None
    priorities: list[str] | None = None
    inbox_ids: list[str] | None = None
    is_active: bool | None = None


class SLAEvaluateIn(BaseModel):
    policy_id: uuid.UUID | None = None


def _serialize_policy(p: SLAPolicy) -> dict[str, Any]:
    return {
        "id": str(p.id),
        "name": p.name,
        "description": p.description,
        "firstResponseTimeThreshold": p.first_response_time_threshold,
        "resolutionTimeThreshold": p.resolution_time_threshold,
        "onlyDuringBusinessHours": p.only_during_business_hours,
        "priorities": p.priorities,
        "inboxIds": p.inbox_ids,
        "isActive": p.is_active,
        "createdAt": p.created_at.isoformat(),
        "updatedAt": p.updated_at.isoformat(),
    }


@router.get("/sla/policies")
async def list_sla_policies(
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    """Lists all configured SLA policies, auto-seeding defaults if none exist."""
    policies = await sla_service.list_policies(session)
    return [_serialize_policy(p) for p in policies]


@router.post("/sla/policies")
async def create_sla_policy(
    payload: SLAPolicyCreateIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Creates a new SLA policy."""
    if not payload.name.strip():
        raise HTTPException(status_code=400, detail="Policy name cannot be empty")
    policy = await sla_service.create_policy(
        session=session,
        name=payload.name,
        description=payload.description,
        first_response_time_threshold=payload.first_response_time_threshold,
        resolution_time_threshold=payload.resolution_time_threshold,
        only_during_business_hours=payload.only_during_business_hours,
        priorities=payload.priorities,
        inbox_ids=payload.inbox_ids,
    )
    return _serialize_policy(policy)


@router.get("/sla/policies/{policy_id}")
async def get_sla_policy(
    policy_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Retrieves an SLA policy by id."""
    policy = await sla_service.get_policy(session, policy_id)
    if not policy:
        raise HTTPException(status_code=404, detail="SLA Policy not found")
    return _serialize_policy(policy)


@router.put("/sla/policies/{policy_id}")
async def update_sla_policy(
    policy_id: uuid.UUID,
    payload: SLAPolicyUpdateIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Updates an SLA policy."""
    try:
        policy = await sla_service.update_policy(
            session=session,
            policy_id=policy_id,
            name=payload.name,
            description=payload.description,
            first_response_time_threshold=payload.first_response_time_threshold,
            resolution_time_threshold=payload.resolution_time_threshold,
            only_during_business_hours=payload.only_during_business_hours,
            priorities=payload.priorities,
            inbox_ids=payload.inbox_ids,
            is_active=payload.is_active,
        )
        return _serialize_policy(policy)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.delete("/sla/policies/{policy_id}")
async def delete_sla_policy(
    policy_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Deletes an SLA policy."""
    deleted = await sla_service.delete_policy(session, policy_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="SLA Policy not found")
    return {"status": "ok", "deleted": True}


@router.get("/sla/metrics")
async def get_sla_metrics(session: AsyncSession = Depends(get_db_session)) -> dict[str, Any]:
    """Returns aggregated SLA fulfillment metrics, rates, and active breach stats."""
    return await sla_service.get_sla_metrics(session)


@router.get("/sla/breaches")
async def list_breached_conversations(
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    """Returns all conversations currently in SLA breach or nearing breach."""
    return await sla_service.list_breached_conversations(session)


@router.post("/conversations/{conversation_id}/sla/evaluate")
async def evaluate_conversation_sla(
    conversation_id: str,
    payload: SLAEvaluateIn | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Evaluates SLA deadlines for a specific conversation against active SLA policies.
    """
    try:
        c_id = uuid.UUID(str(conversation_id))
    except (ValueError, TypeError):
        # Demo / mock conversation handler
        now = datetime.now(UTC)
        return {
            "conversationId": str(conversation_id),
            "sla": {
                "policyId": "mock-policy",
                "policyName": "Standard Customer Support SLA",
                "firstResponseThresholdMinutes": 60,
                "resolutionThresholdMinutes": 1440,
                "firstResponseTargetAt": (now + timedelta(minutes=45)).isoformat(),
                "resolutionTargetAt": (now + timedelta(hours=18)).isoformat(),
                "firstResponseAt": None,
                "firstResponseStatus": "active",
                "firstResponseBreached": False,
                "resolutionAt": None,
                "resolutionStatus": "active",
                "resolutionBreached": False,
                "isBreached": False,
                "nearingBreach": False,
                "minutesToFirstResponseBreach": 45,
                "minutesToResolutionBreach": 1080,
                "evaluatedAt": now.isoformat(),
            },
            "tags": ["sla-active"],
        }

    try:
        policy_id = payload.policy_id if payload else None
        return await sla_service.evaluate_conversation_sla(
            session=session,
            conversation_id=c_id,
            policy_id=policy_id,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
