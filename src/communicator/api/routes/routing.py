from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import routing_service

router = APIRouter(tags=["Auto-Assignment & Agent Capacity Routing"])


class AgentStatusUpdateIn(BaseModel):
    status: str  # online | busy | offline


class AgentCapacityUpdateIn(BaseModel):
    max_capacity: int


class AutoAssignIn(BaseModel):
    strategy: str = "capacity_based"  # capacity_based | round_robin
    team_id: uuid.UUID | None = None


class BatchAutoAssignIn(BaseModel):
    strategy: str = "capacity_based"
    team_id: uuid.UUID | None = None


@router.get("/routing/agents")
async def list_routing_agents(session: AsyncSession = Depends(get_db_session)) -> list[dict[str, Any]]:
    """Lists all team agents with live availability, current active load, and capacity slots."""
    return await routing_service.list_agents(session)


@router.put("/routing/agents/{agent_id}/status")
async def update_agent_status(
    agent_id: uuid.UUID,
    payload: AgentStatusUpdateIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Updates agent availability status: online, busy, or offline."""
    try:
        return await routing_service.update_agent_status(session, agent_id, payload.status)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.put("/routing/agents/{agent_id}/capacity")
async def update_agent_capacity(
    agent_id: uuid.UUID,
    payload: AgentCapacityUpdateIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Updates maximum concurrent conversation capacity for an agent."""
    try:
        return await routing_service.update_agent_capacity(session, agent_id, payload.max_capacity)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/routing/capacity-overview")
async def get_capacity_overview(
    team_id: uuid.UUID | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Returns team-wide real-time agent load, online capacity, and slot availability."""
    return await routing_service.get_capacity_overview(session, team_id)


@router.post("/conversations/{conversation_id}/auto-assign")
async def auto_assign_conversation(
    conversation_id: str,
    payload: AutoAssignIn | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Auto-routes a conversation to an available online agent based on
    either least-loaded capacity or round-robin rotation.
    """
    strategy = payload.strategy if payload else "capacity_based"
    team_id = payload.team_id if payload else None

    try:
        c_id = uuid.UUID(str(conversation_id))
    except (ValueError, TypeError):
        # Demo / mock conversation handler
        agents = await routing_service.list_agents(session)
        online_agents = [a for a in agents if a["availabilityStatus"] == "online"]
        chosen = online_agents[0] if online_agents else (agents[0] if agents else {"name": "Riley Brooks", "id": "agent-riley"})
        return {
            "status": "assigned",
            "assigned": True,
            "conversationId": str(conversation_id),
            "agentId": str(chosen.get("id")),
            "agentName": chosen.get("name", "Riley Brooks"),
            "strategy": strategy,
            "activeCount": chosen.get("activeConversationsCount", 1) + 1,
            "maxCapacity": chosen.get("maxCapacity", 5),
            "tags": ["auto-assigned"],
        }

    try:
        return await routing_service.auto_assign_conversation(
            session=session,
            conversation_id=c_id,
            strategy=strategy,
            team_id=team_id,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/routing/auto-assign-batch")
async def batch_auto_assign(
    payload: BatchAutoAssignIn | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Batch-assigns all unassigned open conversations across the team."""
    strategy = payload.strategy if payload else "capacity_based"
    team_id = payload.team_id if payload else None
    return await routing_service.batch_auto_assign(session, strategy=strategy, team_id=team_id)
