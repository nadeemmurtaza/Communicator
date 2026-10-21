from __future__ import annotations

import uuid
from typing import Any, Literal

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel

from communicator.services.presence import PresenceAction, presence_manager

router = APIRouter(prefix="/conversations", tags=["Agent Presence & Collision Detection"])


class PresenceIn(BaseModel):
    agent_name: str
    action: PresenceAction = "viewing"  # viewing | typing | left
    agent_id: str | None = None
    agent_avatar: str | None = None


@router.post("/{conversation_id}/presence")
async def update_presence(
    conversation_id: str,
    payload: PresenceIn,
) -> dict[str, Any]:
    """
    Heartbeats or updates agent activity in a conversation thread:
    - viewing: Agent opened or is reading the conversation.
    - typing: Agent is actively composing a response in the composer.
    - left: Agent closed the conversation or navigated away.
    """
    if not payload.agent_name.strip():
        raise HTTPException(status_code=400, detail="agent_name cannot be empty")

    presence = presence_manager.update_presence(
        conversation_id=conversation_id,
        agent_name=payload.agent_name.strip(),
        action=payload.action,
        agent_id=payload.agent_id,
        agent_avatar=payload.agent_avatar,
    )
    return {
        "status": "ok",
        "presence": {
            "agentId": presence.agent_id,
            "agentName": presence.agent_name,
            "agentAvatar": presence.agent_avatar,
            "action": presence.action,
            "lastSeenAt": presence.last_seen_at,
            "conversationId": presence.conversation_id,
        },
    }


@router.get("/{conversation_id}/presence")
async def get_presence(
    conversation_id: str,
) -> list[dict[str, Any]]:
    """
    Retrieves all currently active agents inside this conversation thread.
    Automatically excludes entries older than the 30-second TTL.
    """
    presences = presence_manager.get_presences(conversation_id)
    return [
        {
            "agentId": p["agent_id"],
            "agentName": p["agent_name"],
            "agentAvatar": p["agent_avatar"],
            "action": p["action"],
            "lastSeenAt": p["last_seen_at"],
            "conversationId": p["conversation_id"],
        }
        for p in presences
    ]


@router.get("/{conversation_id}/collision-check")
async def check_collision(
    conversation_id: str,
    current_agent: str = Query("Riley Brooks", description="Name of the requesting agent"),
) -> dict[str, Any]:
    """
    Real-time Collision Detection:
    Checks if another teammate is concurrently viewing or composing a reply in the same thread.
    Returns collision status, active typers, active viewers, and recommended coordinate warnings.
    """
    return presence_manager.check_collision(
        conversation_id=conversation_id,
        current_agent_name=current_agent,
    )


@router.delete("/{conversation_id}/presence")
async def clear_presence(
    conversation_id: str,
    agent_name: str = Query(..., description="Agent name to remove from thread presence"),
) -> dict[str, Any]:
    """
    Explicitly removes an agent's presence from a conversation.
    """
    removed = presence_manager.clear_agent(conversation_id, agent_name)
    return {"status": "ok", "removed": removed}
