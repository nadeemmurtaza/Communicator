from __future__ import annotations

import uuid
from datetime import UTC, datetime, timedelta
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.message import Message, MessageDirection
from communicator.models.sla import SLAPolicy

DEFAULT_SLA_POLICIES = [
    {
        "name": "Critical VIP Escalation SLA",
        "description": "Strict 15-minute first response and 2-hour resolution target for urgent escalations & VIP tier accounts.",
        "first_response_time_threshold": 15,
        "resolution_time_threshold": 120,
        "only_during_business_hours": False,
        "priorities": ["urgent"],
        "inbox_ids": [],
    },
    {
        "name": "High Priority SLA",
        "description": "30-minute first response and 6-hour resolution target for high priority support tickets.",
        "first_response_time_threshold": 30,
        "resolution_time_threshold": 360,
        "only_during_business_hours": False,
        "priorities": ["high"],
        "inbox_ids": [],
    },
    {
        "name": "Standard Customer Support SLA",
        "description": "60-minute first response and 24-hour resolution target for general inquiries and routine support.",
        "first_response_time_threshold": 60,
        "resolution_time_threshold": 1440,
        "only_during_business_hours": True,
        "priorities": ["medium", "low"],
        "inbox_ids": [],
    },
]


async def list_policies(session: AsyncSession) -> list[SLAPolicy]:
    """Lists all SLA policies, seeding defaults if empty."""
    res = await session.execute(
        select(SLAPolicy).order_by(SLAPolicy.first_response_time_threshold.asc())
    )
    policies = list(res.scalars().all())
    if not policies:
        for item in DEFAULT_SLA_POLICIES:
            p = SLAPolicy(**item)
            session.add(p)
        await session.flush()
        res = await session.execute(
            select(SLAPolicy).order_by(SLAPolicy.first_response_time_threshold.asc())
        )
        policies = list(res.scalars().all())
    return policies


async def get_policy(session: AsyncSession, policy_id: uuid.UUID) -> SLAPolicy | None:
    return await session.get(SLAPolicy, policy_id)


async def create_policy(
    session: AsyncSession,
    name: str,
    description: str = "",
    first_response_time_threshold: int = 60,
    resolution_time_threshold: int = 1440,
    only_during_business_hours: bool = False,
    priorities: list[str] | None = None,
    inbox_ids: list[str] | None = None,
) -> SLAPolicy:
    policy = SLAPolicy(
        name=name.strip(),
        description=description.strip(),
        first_response_time_threshold=max(1, first_response_time_threshold),
        resolution_time_threshold=max(1, resolution_time_threshold),
        only_during_business_hours=only_during_business_hours,
        priorities=priorities or [],
        inbox_ids=inbox_ids or [],
        is_active=True,
    )
    session.add(policy)
    await session.flush()
    return policy


async def update_policy(
    session: AsyncSession,
    policy_id: uuid.UUID,
    name: str | None = None,
    description: str | None = None,
    first_response_time_threshold: int | None = None,
    resolution_time_threshold: int | None = None,
    only_during_business_hours: bool | None = None,
    priorities: list[str] | None = None,
    inbox_ids: list[str] | None = None,
    is_active: bool | None = None,
) -> SLAPolicy:
    policy = await session.get(SLAPolicy, policy_id)
    if not policy:
        raise ValueError(f"SLA Policy {policy_id} not found")

    if name is not None:
        policy.name = name.strip()
    if description is not None:
        policy.description = description.strip()
    if first_response_time_threshold is not None:
        policy.first_response_time_threshold = max(1, first_response_time_threshold)
    if resolution_time_threshold is not None:
        policy.resolution_time_threshold = max(1, resolution_time_threshold)
    if only_during_business_hours is not None:
        policy.only_during_business_hours = only_during_business_hours
    if priorities is not None:
        policy.priorities = priorities
    if inbox_ids is not None:
        policy.inbox_ids = inbox_ids
    if is_active is not None:
        policy.is_active = is_active

    policy.updated_at = datetime.now(UTC)
    await session.flush()
    return policy


async def delete_policy(session: AsyncSession, policy_id: uuid.UUID) -> bool:
    policy = await session.get(SLAPolicy, policy_id)
    if not policy:
        return False
    await session.delete(policy)
    await session.flush()
    return True


async def match_policy_for_conversation(
    session: AsyncSession,
    conversation: Conversation,
) -> SLAPolicy:
    """Finds best matching SLA policy for a conversation."""
    policies = await list_policies(session)
    active_policies = [p for p in policies if p.is_active]
    if not active_policies:
        # Fallback dummy policy
        return SLAPolicy(
            name="Default Fallback SLA",
            first_response_time_threshold=60,
            resolution_time_threshold=1440,
        )

    # 1. Match by exact priority
    prio = (conversation.priority or "medium").lower()
    for p in active_policies:
        if prio in [pr.lower() for pr in p.priorities]:
            return p

    # 2. Match by inbox
    if conversation.inbox_id:
        inbox_str = str(conversation.inbox_id)
        for p in active_policies:
            if inbox_str in p.inbox_ids:
                return p

    # 3. Fallback to standard/first policy
    return active_policies[-1]


async def evaluate_conversation_sla(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    policy_id: uuid.UUID | None = None,
) -> dict[str, Any]:
    """
    Evaluates SLA deadlines, first response fulfillment, resolution breaches,
    and auto-applies escalation tags.
    """
    conversation = await session.get(Conversation, conversation_id)
    if not conversation:
        raise ValueError(f"Conversation {conversation_id} not found")

    if policy_id:
        policy = await get_policy(session, policy_id)
        if not policy:
            policy = await match_policy_for_conversation(session, conversation)
    else:
        policy = await match_policy_for_conversation(session, conversation)

    now = datetime.now(UTC)
    created_at = conversation.created_at or now

    target_fr_at = created_at + timedelta(minutes=policy.first_response_time_threshold)
    target_res_at = created_at + timedelta(minutes=policy.resolution_time_threshold)

    # 1. Check First Response
    # Look for first outbound agent message that isn't a private note
    res_msgs = await session.execute(
        select(Message)
        .where(Message.conversation_id == conversation.id)
        .order_by(Message.sent_at.asc())
    )
    all_msgs = list(res_msgs.scalars().all())
    outbound_msgs = [
        m
        for m in all_msgs
        if m.direction == MessageDirection.OUTBOUND and not getattr(m, "is_private", False)
    ]

    first_resp_msg = outbound_msgs[0] if outbound_msgs else None
    if first_resp_msg:
        first_resp_at = first_resp_msg.sent_at
        first_resp_breached = first_resp_at > target_fr_at
        first_resp_status = "breached" if first_resp_breached else "fulfilled"
        minutes_to_fr_breach = 0
    else:
        first_resp_at = None
        first_resp_breached = now > target_fr_at
        first_resp_status = "breached" if first_resp_breached else "active"
        minutes_to_fr_breach = int((target_fr_at - now).total_seconds() / 60)

    # 2. Check Resolution
    is_resolved = conversation.state in (
        ConversationState.CLOSED_WON,
        ConversationState.CLOSED_LOST,
    )
    if is_resolved:
        resolution_at = conversation.updated_at or now
        resolution_breached = resolution_at > target_res_at
        resolution_status = "breached" if resolution_breached else "fulfilled"
        minutes_to_res_breach = 0
    else:
        resolution_at = None
        resolution_breached = now > target_res_at
        resolution_status = "breached" if resolution_breached else "active"
        minutes_to_res_breach = int((target_res_at - now).total_seconds() / 60)

    is_breached = first_resp_breached or resolution_breached
    nearing_breach = (not is_breached) and (
        (0 <= minutes_to_fr_breach <= 15 and first_resp_status == "active")
        or (0 <= minutes_to_res_breach <= 60 and resolution_status == "active")
    )

    # Auto-tagging updates
    attrs = dict(conversation.custom_attributes or {})
    current_tags = list(attrs.get("tags", []))

    if is_breached:
        if "sla-breached" not in current_tags:
            current_tags.append("sla-breached")
        if "sla-warning" in current_tags:
            current_tags.remove("sla-warning")
    elif nearing_breach:
        if "sla-warning" not in current_tags:
            current_tags.append("sla-warning")
        if "sla-breached" in current_tags:
            current_tags.remove("sla-breached")
    else:
        if "sla-breached" in current_tags:
            current_tags.remove("sla-breached")
        if "sla-warning" in current_tags:
            current_tags.remove("sla-warning")

    sla_data = {
        "policyId": str(policy.id),
        "policyName": policy.name,
        "firstResponseThresholdMinutes": policy.first_response_time_threshold,
        "resolutionThresholdMinutes": policy.resolution_time_threshold,
        "firstResponseTargetAt": target_fr_at.isoformat(),
        "resolutionTargetAt": target_res_at.isoformat(),
        "firstResponseAt": first_resp_at.isoformat() if first_resp_at else None,
        "firstResponseStatus": first_resp_status,
        "firstResponseBreached": first_resp_breached,
        "resolutionAt": resolution_at.isoformat() if resolution_at else None,
        "resolutionStatus": resolution_status,
        "resolutionBreached": resolution_breached,
        "isBreached": is_breached,
        "nearingBreach": nearing_breach,
        "minutesToFirstResponseBreach": minutes_to_fr_breach,
        "minutesToResolutionBreach": minutes_to_res_breach,
        "evaluatedAt": now.isoformat(),
    }

    attrs["sla"] = sla_data
    attrs["tags"] = current_tags
    conversation.custom_attributes = attrs
    await session.flush()

    return {
        "conversationId": str(conversation.id),
        "sla": sla_data,
        "tags": current_tags,
    }


async def get_sla_metrics(session: AsyncSession) -> dict[str, Any]:
    """Aggregates SLA performance, fulfillment rates, and breaches across conversations."""
    policies = await list_policies(session)

    # Query evaluated conversations
    res = await session.execute(select(Conversation))
    convs = list(res.scalars().all())

    total_tracked = 0
    fr_fulfilled = 0
    fr_breached = 0
    res_fulfilled = 0
    res_breached = 0
    active_breaches = 0
    nearing_breaches = 0

    policy_stats: dict[str, dict[str, Any]] = {
        str(p.id): {
            "policyId": str(p.id),
            "policyName": p.name,
            "trackedCount": 0,
            "breachedCount": 0,
            "fulfilledCount": 0,
        }
        for p in policies
    }

    for c in convs:
        sla = (c.custom_attributes or {}).get("sla")
        if not sla:
            continue

        total_tracked += 1
        p_id = sla.get("policyId")

        if sla.get("firstResponseStatus") == "fulfilled":
            fr_fulfilled += 1
        elif sla.get("firstResponseStatus") == "breached":
            fr_breached += 1

        if sla.get("resolutionStatus") == "fulfilled":
            res_fulfilled += 1
        elif sla.get("resolutionStatus") == "breached":
            res_breached += 1

        if sla.get("isBreached"):
            active_breaches += 1
            if p_id in policy_stats:
                policy_stats[p_id]["breachedCount"] += 1
        else:
            if p_id in policy_stats:
                policy_stats[p_id]["fulfilledCount"] += 1

        if sla.get("nearingBreach"):
            nearing_breaches += 1

        if p_id in policy_stats:
            policy_stats[p_id]["trackedCount"] += 1

    total_fr_evaluated = fr_fulfilled + fr_breached
    total_res_evaluated = res_fulfilled + res_breached

    fr_rate = (
        round((fr_fulfilled / total_fr_evaluated * 100), 1) if total_fr_evaluated > 0 else 100.0
    )
    res_rate = (
        round((res_fulfilled / total_res_evaluated * 100), 1) if total_res_evaluated > 0 else 100.0
    )

    return {
        "totalTracked": total_tracked,
        "firstResponseFulfillmentRate": fr_rate,
        "firstResponseFulfilledCount": fr_fulfilled,
        "firstResponseBreachedCount": fr_breached,
        "resolutionFulfillmentRate": res_rate,
        "resolutionFulfilledCount": res_fulfilled,
        "resolutionBreachedCount": res_breached,
        "activeBreachesCount": active_breaches,
        "nearingBreachesCount": nearing_breaches,
        "policyBreakdown": list(policy_stats.values()),
    }


async def list_breached_conversations(session: AsyncSession) -> list[dict[str, Any]]:
    """Returns all active conversations currently in breach or nearing breach."""
    res = await session.execute(select(Conversation).order_by(Conversation.updated_at.desc()))
    convs = list(res.scalars().all())

    results: list[dict[str, Any]] = []
    for c in convs:
        sla = (c.custom_attributes or {}).get("sla")
        if not sla:
            continue

        if sla.get("isBreached") or sla.get("nearingBreach"):
            contact = await session.get(Contact, c.contact_id) if c.contact_id else None
            contact_name = contact.person_name if contact else "Anonymous Visitor"
            contact_val = contact.value if contact else "Unknown"
            channel_val = contact.channel.value if (contact and contact.channel) else "chat"
            results.append(
                {
                    "conversationId": str(c.id),
                    "contactName": contact_name,
                    "contactIdentifier": contact_val,
                    "priority": c.priority,
                    "channel": channel_val,
                    "state": c.state.value,
                    "policyName": sla.get("policyName", "SLA"),
                    "firstResponseStatus": sla.get("firstResponseStatus"),
                    "firstResponseTargetAt": sla.get("firstResponseTargetAt"),
                    "resolutionStatus": sla.get("resolutionStatus"),
                    "resolutionTargetAt": sla.get("resolutionTargetAt"),
                    "isBreached": sla.get("isBreached", False),
                    "nearingBreach": sla.get("nearingBreach", False),
                    "minutesToFirstResponseBreach": sla.get("minutesToFirstResponseBreach", 0),
                    "minutesToResolutionBreach": sla.get("minutesToResolutionBreach", 0),
                    "updatedAt": c.updated_at.isoformat() if c.updated_at else None,
                }
            )

    return results
