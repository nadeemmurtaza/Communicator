from __future__ import annotations

import uuid
from datetime import UTC, datetime, timedelta
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.models.contact import ChannelType, Contact
from communicator.models.contact_note import ContactNote
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.macro import Macro
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.models.team import Team
from communicator.models.team_member import TeamMember


DEFAULT_SYSTEM_MACROS = [
    {
        "name": "Resolve Ticket + Send CSAT + Tag 'Resolved'",
        "description": "Marks conversation resolved, dispatches CSAT rating survey, tags conversation as resolved, and logs an internal resolution note.",
        "actions": [
            {"type": "resolve_conversation", "resolution_type": "human_assisted"},
            {"type": "send_csat_survey"},
            {"type": "add_tags", "tags": ["resolved", "csat-dispatched"]},
            {
                "type": "add_private_note",
                "note": "Conversation successfully resolved via one-click macro. CSAT survey sent.",
            },
        ],
    },
    {
        "name": "Escalate to Tier 2 + Mark Urgent + Assign Support Team",
        "description": "Increases priority to Urgent, assigns conversation to Technical Support, and logs escalation details.",
        "actions": [
            {"type": "change_priority", "priority": "urgent"},
            {"type": "assign_team", "team_name": "Technical Support"},
            {"type": "add_tags", "tags": ["escalated", "tier-2-review"]},
            {
                "type": "add_private_note",
                "note": "Escalated to Tier 2 Engineering for priority technical investigation.",
            },
        ],
    },
    {
        "name": "Send Password Reset Link + Snooze 24h",
        "description": "Sends customer password reset guidance, tags ticket as pending verification, and snoozes for 24 hours.",
        "actions": [
            {
                "type": "send_message",
                "message": "We have sent a secure password reset link to your registered email. Please check your inbox and follow the instructions.",
            },
            {"type": "snooze_conversation", "duration_hours": 24},
            {"type": "add_tags", "tags": ["auth-reset", "pending-customer"]},
        ],
    },
    {
        "name": "VIP Refund Approved + Assign Billing",
        "description": "Routes conversation to Sales & Billing team, sets priority High, sends confirmation, and tags as refund-approved.",
        "actions": [
            {"type": "assign_team", "team_name": "Sales & Billing"},
            {"type": "change_priority", "priority": "high"},
            {
                "type": "send_message",
                "message": "Your refund request has been approved and is being processed by our billing team. The credit will reflect in 3-5 business days.",
            },
            {"type": "add_tags", "tags": ["refund-approved", "vip-support"]},
            {
                "type": "add_private_note",
                "note": "Refund authorized by agent via one-click macro.",
            },
        ],
    },
]


async def seed_default_macros(session: AsyncSession) -> None:
    """Seeds default standard macros if table is empty."""
    existing = (await session.execute(select(Macro))).scalars().first()
    if existing is None:
        for m_data in DEFAULT_SYSTEM_MACROS:
            macro = Macro(
                name=m_data["name"],
                description=m_data["description"],
                actions=m_data["actions"],
            )
            session.add(macro)
        await session.flush()


async def list_macros(session: AsyncSession) -> list[Macro]:
    await seed_default_macros(session)
    stmt = select(Macro).order_by(Macro.created_at)
    return list((await session.execute(stmt)).scalars().all())


async def get_macro(session: AsyncSession, macro_id: uuid.UUID) -> Macro | None:
    return await session.get(Macro, macro_id)


async def create_macro(
    session: AsyncSession,
    name: str,
    description: str,
    actions: list[dict[str, Any]],
) -> Macro:
    macro = Macro(
        name=name.strip(),
        description=description.strip(),
        actions=actions,
    )
    session.add(macro)
    await session.flush()
    return macro


async def update_macro(
    session: AsyncSession,
    macro_id: uuid.UUID,
    name: str | None = None,
    description: str | None = None,
    actions: list[dict[str, Any]] | None = None,
) -> Macro:
    macro = await session.get(Macro, macro_id)
    if not macro:
        raise ValueError(f"Macro {macro_id} not found")
    if name is not None:
        macro.name = name.strip()
    if description is not None:
        macro.description = description.strip()
    if actions is not None:
        macro.actions = actions
    macro.updated_at = datetime.now(UTC)
    await session.flush()
    return macro


async def delete_macro(session: AsyncSession, macro_id: uuid.UUID) -> bool:
    macro = await session.get(Macro, macro_id)
    if not macro:
        return False
    await session.delete(macro)
    await session.flush()
    return True


async def execute_macro(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    macro_id: uuid.UUID,
    agent_name: str = "Riley Brooks",
) -> dict[str, Any]:
    """
    Executes a multi-action macro against a conversation thread atomically.
    Supports resolving, priority changes, team/agent routing, customer messaging,
    private notes, tags, and CSAT survey dispatch.
    """
    macro = await session.get(Macro, macro_id)
    if not macro:
        raise ValueError(f"Macro {macro_id} not found")

    conversation = await session.get(
        Conversation,
        conversation_id,
        options=[selectinload(Conversation.contact), selectinload(Conversation.inbox)],
    )
    if not conversation:
        raise ValueError(f"Conversation {conversation_id} not found")

    executed_actions: list[dict[str, Any]] = []
    now = datetime.now(UTC)

    # Prepare custom attributes tag storage
    attrs = dict(conversation.custom_attributes or {})
    current_tags = list(attrs.get("tags", []))

    for action in macro.actions:
        a_type = action.get("type", "")

        if a_type == "resolve_conversation":
            conversation.state = ConversationState.CLOSED_WON
            conversation.resolution_type = action.get("resolution_type", "human_assisted")
            executed_actions.append({"action": a_type, "status": "resolved"})

        elif a_type == "change_priority":
            p = action.get("priority", "medium").lower()
            conversation.priority = p
            executed_actions.append({"action": a_type, "priority": p})

        elif a_type == "change_status":
            st = action.get("status", "closed_won")
            try:
                conversation.state = ConversationState(st)
                executed_actions.append({"action": a_type, "state": st})
            except ValueError:
                pass

        elif a_type == "assign_team":
            team_name = action.get("team_name")
            team_id = action.get("team_id")
            if team_id:
                conversation.team_id = uuid.UUID(str(team_id))
                executed_actions.append({"action": a_type, "team_id": str(team_id)})
            elif team_name:
                t_stmt = select(Team).where(Team.name.ilike(f"%{team_name}%"))
                matched_team = (await session.execute(t_stmt)).scalars().first()
                if matched_team:
                    conversation.team_id = matched_team.id
                    executed_actions.append({"action": a_type, "team": matched_team.name})
                else:
                    executed_actions.append({"action": a_type, "team": team_name, "status": "noted"})

        elif a_type == "assign_agent":
            agent_name_val = action.get("agent_name")
            agent_id = action.get("agent_id")
            if agent_id:
                conversation.assigned_to_id = uuid.UUID(str(agent_id))
                executed_actions.append({"action": a_type, "agent_id": str(agent_id)})
            elif agent_name_val:
                ag_stmt = select(TeamMember).where(TeamMember.name.ilike(f"%{agent_name_val}%"))
                matched_ag = (await session.execute(ag_stmt)).scalars().first()
                if matched_ag:
                    conversation.assigned_to_id = matched_ag.id
                    executed_actions.append({"action": a_type, "agent": matched_ag.name})

        elif a_type == "add_tags":
            tags_to_add = action.get("tags", [])
            for t in tags_to_add:
                if t not in current_tags:
                    current_tags.append(t)
            executed_actions.append({"action": a_type, "added": tags_to_add})

        elif a_type == "remove_tags":
            tags_to_remove = action.get("tags", [])
            current_tags = [t for t in current_tags if t not in tags_to_remove]
            executed_actions.append({"action": a_type, "removed": tags_to_remove})

        elif a_type == "send_message":
            msg_text = action.get("message", "").strip()
            if msg_text:
                ch = conversation.contact.channel if conversation.contact else ChannelType.WHATSAPP
                out_msg = Message(
                    conversation_id=conversation.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=ch,
                    body=msg_text,
                    provider="macro_executor",
                    provider_message_id=f"macro_msg_{uuid.uuid4().hex[:8]}",
                    status=MessageStatus.SENT,
                    is_private=False,
                )
                session.add(out_msg)
                conversation.last_outbound_at = now
                executed_actions.append({"action": a_type, "message": msg_text})

        elif a_type == "add_private_note":
            note_text = action.get("note", "").strip()
            if note_text:
                if conversation.contact:
                    note = ContactNote(
                        contact_id=conversation.contact.id,
                        author_name=agent_name,
                        body=note_text,
                    )
                    session.add(note)
                ch = conversation.contact.channel if conversation.contact else ChannelType.WHATSAPP
                priv_msg = Message(
                    conversation_id=conversation.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=ch,
                    body=note_text,
                    provider="macro_executor",
                    provider_message_id=f"macro_note_{uuid.uuid4().hex[:8]}",
                    is_private=True,
                )
                session.add(priv_msg)
                executed_actions.append({"action": a_type, "note": note_text})

        elif a_type == "send_csat_survey":
            # Record CSAT dispatch
            attrs["csat_dispatched_at"] = now.isoformat()
            executed_actions.append({"action": a_type, "dispatched": True})

        elif a_type == "snooze_conversation":
            hours = action.get("duration_hours", 24)
            snooze_until = now + timedelta(hours=hours)
            conversation.snoozed_until = snooze_until
            executed_actions.append({"action": a_type, "snoozed_until": snooze_until.isoformat()})

        elif a_type == "mutate_custom_attributes":
            extra_attrs = action.get("attributes", {})
            attrs.update(extra_attrs)
            executed_actions.append({"action": a_type, "mutated": list(extra_attrs.keys())})

    attrs["tags"] = current_tags
    attrs["last_macro_executed"] = {
        "macro_id": str(macro.id),
        "macro_name": macro.name,
        "executed_at": now.isoformat(),
        "executed_by": agent_name,
    }
    conversation.custom_attributes = attrs
    conversation.updated_at = now

    await session.flush()

    return {
        "status": "ok",
        "macroId": str(macro.id),
        "macroName": macro.name,
        "conversationId": str(conversation.id),
        "executedActionsCount": len(executed_actions),
        "executedActions": executed_actions,
        "conversationState": conversation.state.value,
        "conversationPriority": conversation.priority,
        "conversationTags": current_tags,
        "executedAt": now.isoformat(),
    }
