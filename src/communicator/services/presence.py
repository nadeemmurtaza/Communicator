from __future__ import annotations

import time
import uuid
from dataclasses import dataclass
from typing import Any, Literal

PresenceAction = Literal["viewing", "typing", "left"]

PRESENCE_TTL_SECONDS = 30.0  # Agent presence expires after 30 seconds of inactivity


@dataclass
class AgentPresence:
    agent_id: str
    agent_name: str
    agent_avatar: str
    action: PresenceAction
    last_seen_at: float
    conversation_id: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "agentId": self.agent_id,
            "agent_id": self.agent_id,
            "agentName": self.agent_name,
            "agent_name": self.agent_name,
            "agentAvatar": self.agent_avatar,
            "agent_avatar": self.agent_avatar,
            "action": self.action,
            "lastSeenAt": self.last_seen_at,
            "last_seen_at": self.last_seen_at,
            "conversationId": self.conversation_id,
            "conversation_id": self.conversation_id,
        }


class PresenceManager:
    """
    In-memory real-time agent presence and collision detection store.
    Tracks active agents viewing and typing across conversations with automatic TTL expiration.
    """

    def __init__(self, ttl: float = PRESENCE_TTL_SECONDS):
        self._ttl = ttl
        # Map: conversation_id -> dict of agent_name -> AgentPresence
        self._store: dict[str, dict[str, AgentPresence]] = {}

    def _cleanup_expired(self, conv_id: str) -> None:
        if conv_id not in self._store:
            return
        now = time.time()
        active = {}
        for name, p in self._store[conv_id].items():
            if now - p.last_seen_at <= self._ttl and p.action != "left":
                active[name] = p
        if active:
            self._store[conv_id] = active
        else:
            self._store.pop(conv_id, None)

    def update_presence(
        self,
        conversation_id: str,
        agent_name: str,
        action: PresenceAction,
        agent_id: str | None = None,
        agent_avatar: str | None = None,
    ) -> AgentPresence:
        now = time.time()
        conv_str = str(conversation_id)

        if conv_str not in self._store:
            self._store[conv_str] = {}

        if action == "left":
            self._store[conv_str].pop(agent_name, None)
            return AgentPresence(
                agent_id=agent_id or f"ag-{uuid.uuid4().hex[:6]}",
                agent_name=agent_name,
                agent_avatar=agent_avatar or "",
                action="left",
                last_seen_at=now,
                conversation_id=conv_str,
            )

        presence = AgentPresence(
            agent_id=agent_id or f"ag-{abs(hash(agent_name)) % 10000}",
            agent_name=agent_name,
            agent_avatar=agent_avatar
            or "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80",
            action=action,
            last_seen_at=now,
            conversation_id=conv_str,
        )
        self._store[conv_str][agent_name] = presence
        self._cleanup_expired(conv_str)
        return presence

    def get_presences(self, conversation_id: str) -> list[dict[str, Any]]:
        conv_str = str(conversation_id)
        self._cleanup_expired(conv_str)
        if conv_str not in self._store:
            return []
        return [p.to_dict() for p in self._store[conv_str].values()]

    def check_collision(
        self,
        conversation_id: str,
        current_agent_name: str = "Riley Brooks",
    ) -> dict[str, Any]:
        conv_str = str(conversation_id)
        self._cleanup_expired(conv_str)
        all_presences = self._store.get(conv_str, {})

        other_agents = [
            p
            for name, p in all_presences.items()
            if name.lower().strip() != current_agent_name.lower().strip()
        ]

        typers = [p for p in other_agents if p.action == "typing"]
        viewers = [p for p in other_agents if p.action == "viewing"]

        has_collision = len(other_agents) > 0
        warning_message: str | None = None
        severity: str = "none"

        if typers:
            names = ", ".join(t.agent_name for t in typers)
            warning_message = f"✍️ {names} is actively composing a reply! Coordinate with teammates to prevent duplicate responses."
            severity = "danger"
        elif viewers:
            names = ", ".join(v.agent_name for v in viewers)
            warning_message = (
                f"👀 {names} is viewing this conversation. Overlapping replies will be warned."
            )
            severity = "warning"

        return {
            "conversationId": conv_str,
            "hasCollision": has_collision,
            "severity": severity,
            "warningMessage": warning_message,
            "otherAgentsCount": len(other_agents),
            "typers": [p.to_dict() for p in typers],
            "viewers": [p.to_dict() for p in viewers],
            "allPresences": [p.to_dict() for p in all_presences.values()],
        }

    def clear_agent(self, conversation_id: str, agent_name: str) -> bool:
        conv_str = str(conversation_id)
        if conv_str in self._store and agent_name in self._store[conv_str]:
            del self._store[conv_str][agent_name]
            return True
        return False

    def reset_all(self) -> None:
        self._store.clear()


# Global singleton instance for app runtime
presence_manager = PresenceManager()
