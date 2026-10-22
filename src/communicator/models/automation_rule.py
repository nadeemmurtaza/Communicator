from __future__ import annotations

import enum
import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, Enum, Integer, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class RuleMatchType(str, enum.Enum):
    CONTAINS = "contains"
    EXACT = "exact"
    STARTS_WITH = "starts_with"
    REGEX = "regex"
    BUTTON_CLICK = "button_click"
    LIST_SELECT = "list_select"
    INTENT = "intent"


class RuleActionType(str, enum.Enum):
    AUTO_REPLY = "auto_reply"
    INTERACTIVE_BUTTONS = "interactive_buttons"
    INTERACTIVE_LIST = "interactive_list"
    SEND_MEDIA = "send_media"
    SEND_CTA_URL = "send_cta_url"
    SEND_TEMPLATE = "send_template"
    HANDOFF = "handoff"
    ADD_TAG = "add_tag"
    REMOVE_TAG = "remove_tag"
    ASSIGN_TEAM = "assign_team"
    ENROLL_SEQUENCE = "enroll_sequence"
    SEND_CSAT = "send_csat"
    TRIGGER_WEBHOOK = "trigger_webhook"


class AutomationRule(Base):
    """
    Evaluated on inbound WhatsApp messages and interactive selections:
    Keywords, regex, button clicks, list row picks, and classified intents.
    Dispatches rich WhatsApp interactive replies, media, live agent handoff,
    CSAT surveys, tags, sequences, or external webhooks.
    """

    __tablename__ = "automation_rules"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(String(100), default="newax", nullable=False, index=True)
    # Empty = runs on any channel this tenant has; set to a specific
    # ChannelType value (e.g. "whatsapp") to scope it to one channel.
    channel: Mapped[str] = mapped_column(String(50), default="", nullable=False, index=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    trigger_keyword: Mapped[str] = mapped_column(String(255), nullable=False)
    match_type: Mapped[RuleMatchType] = mapped_column(
        Enum(RuleMatchType), default=RuleMatchType.CONTAINS, nullable=False
    )

    action_type: Mapped[RuleActionType] = mapped_column(Enum(RuleActionType), nullable=False)
    # Payload for action:
    # - AUTO_REPLY: reply text
    # - INTERACTIVE_BUTTONS: JSON {"body": "...", "buttons": [{"id": "...", "title": "..."}]}
    # - INTERACTIVE_LIST: JSON {"body": "...", "button": "...", "sections": [...]}
    # - SEND_MEDIA: JSON {"media_type": "image|document", "url": "...", "caption": "..."}
    # - SEND_CTA_URL: JSON {"body": "...", "display_text": "...", "url": "..."}
    # - SEND_TEMPLATE: JSON {"name": "...", "language": "en_US", "params": [...]}
    # - HANDOFF: optional note to agent
    # - ADD_TAG / REMOVE_TAG: tag name
    # - ASSIGN_TEAM: team member ID or "round_robin"
    # - ENROLL_SEQUENCE: sequence ID
    # - SEND_CSAT: optional custom survey question
    # - TRIGGER_WEBHOOK: target URL
    action_value: Mapped[str] = mapped_column(Text, default="", nullable=False)

    priority: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    execution_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    last_triggered_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
