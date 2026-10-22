from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.models.conversation import Conversation, ConversationState
from communicator.models.team import Team
from communicator.models.team_member import TeamMember

DEFAULT_TEAM_MEMBERS = [
    {
        "name": "Riley Brooks",
        "email": "riley.brooks@bayworks.com",
        "availability_status": "online",
        "max_concurrent_conversations": 5,
    },
    {
        "name": "Jamie Cole",
        "email": "jamie.cole@bayworks.com",
        "availability_status": "online",
        "max_concurrent_conversations": 5,
    },
    {
        "name": "Alex Rivera",
        "email": "alex.rivera@bayworks.com",
        "availability_status": "busy",
        "max_concurrent_conversations": 3,
    },
    {
        "name": "Morgan Vance",
        "email": "morgan.vance@bayworks.com",
        "availability_status": "offline",
        "max_concurrent_conversations": 5,
    },
]

CLOSED_STATES = (ConversationState.CLOSED_WON, ConversationState.CLOSED_LOST)


async def list_agents(session: AsyncSession) -> list[dict[str, Any]]:
    """Lists all agents with real-time active load and capacity calculation."""
    res = await session.execute(select(TeamMember).order_by(TeamMember.name.asc()))
    agents = list(res.scalars().all())

    if not agents:
        for item in DEFAULT_TEAM_MEMBERS:
            session.add(TeamMember(**item))
        await session.flush()
        res = await session.execute(select(TeamMember).order_by(TeamMember.name.asc()))
        agents = list(res.scalars().all())

    results: list[dict[str, Any]] = []
    for agent in agents:
        count_res = await session.execute(
            select(func.count(Conversation.id)).where(
                Conversation.assigned_to_id == agent.id,
                Conversation.state.notin_(CLOSED_STATES),
            )
        )
        active_count = count_res.scalar() or 0
        available_slots = max(0, agent.max_concurrent_conversations - active_count)
        utilization = (
            round((active_count / agent.max_concurrent_conversations) * 100, 1)
            if agent.max_concurrent_conversations > 0
            else 100.0
        )

        results.append(
            {
                "id": str(agent.id),
                "name": agent.name,
                "email": agent.email,
                "availabilityStatus": agent.availability_status,
                "maxCapacity": agent.max_concurrent_conversations,
                "activeConversationsCount": active_count,
                "availableSlots": available_slots,
                "utilizationRatePct": utilization,
                "lastAssignedAt": agent.last_assigned_at.isoformat()
                if agent.last_assigned_at
                else None,
            }
        )

    return results


async def get_agent(session: AsyncSession, agent_id: uuid.UUID) -> TeamMember | None:
    return await session.get(TeamMember, agent_id)


async def update_agent_status(
    session: AsyncSession, agent_id: uuid.UUID, status: str
) -> dict[str, Any]:
    norm_status = status.lower().strip()
    if norm_status not in ("online", "busy", "offline"):
        raise ValueError(f"Invalid status '{status}'. Must be online, busy, or offline.")

    agent = await session.get(TeamMember, agent_id)
    if not agent:
        raise ValueError(f"Agent {agent_id} not found")

    agent.availability_status = norm_status
    await session.flush()

    return {
        "id": str(agent.id),
        "name": agent.name,
        "availabilityStatus": agent.availability_status,
        "maxCapacity": agent.max_concurrent_conversations,
    }


async def update_agent_capacity(
    session: AsyncSession, agent_id: uuid.UUID, max_capacity: int
) -> dict[str, Any]:
    if max_capacity < 1:
        raise ValueError("Capacity limit must be at least 1 concurrent conversation.")

    agent = await session.get(TeamMember, agent_id)
    if not agent:
        raise ValueError(f"Agent {agent_id} not found")

    agent.max_concurrent_conversations = max_capacity
    await session.flush()

    return {
        "id": str(agent.id),
        "name": agent.name,
        "availabilityStatus": agent.availability_status,
        "maxCapacity": agent.max_concurrent_conversations,
    }


async def auto_assign_conversation(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    strategy: str = "capacity_based",  # "capacity_based" | "round_robin"
    team_id: uuid.UUID | None = None,
) -> dict[str, Any]:
    """
    Auto-assigns a conversation to an available online agent based on
    either least-loaded capacity or round-robin rotation.
    """
    conversation = await session.get(Conversation, conversation_id)
    if not conversation:
        raise ValueError(f"Conversation {conversation_id} not found")

    target_team_id = team_id or conversation.team_id

    # 1. Fetch eligible agents
    if target_team_id:
        team = await session.get(Team, target_team_id, options=[selectinload(Team.members)])
        candidates = team.members if team else []
        if not candidates:
            # Fallback to all agents if team has no specific members assigned
            all_agents_res = await session.execute(select(TeamMember))
            candidates = list(all_agents_res.scalars().all())
    else:
        all_agents_res = await session.execute(select(TeamMember))
        candidates = list(all_agents_res.scalars().all())

    # Ensure system has agents seeded
    if not candidates:
        await list_agents(session)
        all_agents_res = await session.execute(select(TeamMember))
        candidates = list(all_agents_res.scalars().all())

    # Filter for online agents only
    online_candidates = [a for a in candidates if a.availability_status == "online"]

    # Compute load for each candidate
    eligible: list[tuple[TeamMember, int, float]] = []
    for agent in online_candidates:
        c_res = await session.execute(
            select(func.count(Conversation.id)).where(
                Conversation.assigned_to_id == agent.id,
                Conversation.state.notin_(CLOSED_STATES),
            )
        )
        active_count = c_res.scalar() or 0
        if active_count < agent.max_concurrent_conversations:
            ratio = active_count / agent.max_concurrent_conversations
            eligible.append((agent, active_count, ratio))

    # 2. Check for overflow
    if not eligible:
        attrs = dict(conversation.custom_attributes or {})
        current_tags = list(attrs.get("tags", []))
        if "team-overflow" not in current_tags:
            current_tags.append("team-overflow")
        attrs["tags"] = current_tags
        attrs["routing_error"] = {
            "attempted_at": datetime.now(UTC).isoformat(),
            "reason": "All eligible agents are currently at capacity or offline.",
            "online_agents_count": len(online_candidates),
        }
        conversation.custom_attributes = attrs
        await session.flush()
        return {
            "status": "overflow",
            "assigned": False,
            "conversationId": str(conversation.id),
            "reason": "All online agents at full capacity or offline",
            "tags": current_tags,
        }

    # 3. Sort eligible agents according to strategy
    if strategy == "round_robin":
        # Least recently assigned agent first
        # None (never assigned) comes first
        eligible.sort(
            key=lambda item: (
                0 if item[0].last_assigned_at is None else 1,
                item[0].last_assigned_at or datetime.min.replace(tzinfo=UTC),
            )
        )
    else:
        # Default capacity_based: lowest utilization ratio first, then lowest absolute count
        eligible.sort(key=lambda item: (item[2], item[1]))

    chosen_agent, current_load, _ = eligible[0]
    now = datetime.now(UTC)

    # 4. Assign
    conversation.assigned_to_id = chosen_agent.id
    chosen_agent.last_assigned_at = now

    attrs = dict(conversation.custom_attributes or {})
    current_tags = list(attrs.get("tags", []))
    if "team-overflow" in current_tags:
        current_tags.remove("team-overflow")
    attrs["tags"] = current_tags

    attrs["routing"] = {
        "strategy": strategy,
        "assigned_at": now.isoformat(),
        "assigned_agent_id": str(chosen_agent.id),
        "assigned_agent_name": chosen_agent.name,
        "agent_load_at_assignment": f"{current_load + 1}/{chosen_agent.max_concurrent_conversations}",
    }
    conversation.custom_attributes = attrs
    conversation.updated_at = now
    await session.flush()

    return {
        "status": "assigned",
        "assigned": True,
        "conversationId": str(conversation.id),
        "agentId": str(chosen_agent.id),
        "agentName": chosen_agent.name,
        "strategy": strategy,
        "activeCount": current_load + 1,
        "maxCapacity": chosen_agent.max_concurrent_conversations,
        "assignedAt": now.isoformat(),
        "tags": current_tags,
    }


async def batch_auto_assign(
    session: AsyncSession,
    strategy: str = "capacity_based",
    team_id: uuid.UUID | None = None,
) -> dict[str, Any]:
    """Auto-assigns all unassigned open conversations."""
    stmt = select(Conversation).where(
        Conversation.assigned_to_id.is_(None),
        Conversation.state.notin_(CLOSED_STATES),
    )
    if team_id:
        stmt = stmt.where(Conversation.team_id == team_id)

    res = await session.execute(stmt)
    unassigned_convs = list(res.scalars().all())

    assigned_count = 0
    overflow_count = 0
    results: list[dict[str, Any]] = []

    for conv in unassigned_convs:
        outcome = await auto_assign_conversation(
            session=session,
            conversation_id=conv.id,
            strategy=strategy,
            team_id=team_id,
        )
        if outcome.get("assigned"):
            assigned_count += 1
        else:
            overflow_count += 1
        results.append(outcome)

    return {
        "totalProcessed": len(unassigned_convs),
        "assignedCount": assigned_count,
        "overflowCount": overflow_count,
        "results": results,
    }


async def get_capacity_overview(
    session: AsyncSession, team_id: uuid.UUID | None = None
) -> dict[str, Any]:
    """Calculates team-wide active capacity, slots, and availability ratios."""
    agent_stats = await list_agents(session)

    total_agents = len(agent_stats)
    online_agents = sum(1 for a in agent_stats if a["availabilityStatus"] == "online")
    busy_agents = sum(1 for a in agent_stats if a["availabilityStatus"] == "busy")
    offline_agents = sum(1 for a in agent_stats if a["availabilityStatus"] == "offline")

    total_capacity = sum(a["maxCapacity"] for a in agent_stats)
    online_capacity = sum(
        a["maxCapacity"] for a in agent_stats if a["availabilityStatus"] == "online"
    )
    total_active = sum(a["activeConversationsCount"] for a in agent_stats)
    available_slots = sum(
        a["availableSlots"] for a in agent_stats if a["availabilityStatus"] == "online"
    )

    utilization_pct = round((total_active / total_capacity * 100), 1) if total_capacity > 0 else 0.0

    return {
        "totalAgents": total_agents,
        "onlineAgents": online_agents,
        "busyAgents": busy_agents,
        "offlineAgents": offline_agents,
        "totalMaxCapacity": total_capacity,
        "onlineCapacity": online_capacity,
        "currentActiveConversations": total_active,
        "availableOnlineSlots": available_slots,
        "utilizationRatePct": utilization_pct,
        "agents": agent_stats,
    }
