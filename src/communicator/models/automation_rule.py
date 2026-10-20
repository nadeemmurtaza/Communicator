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


class RuleActionType(str, enum.Enum):
    AUTO_REPLY = "auto_reply"
    HANDOFF = "handoff"
    ADD_TAG = "add_tag"


class AutomationRule(Base):
    """
    Keyword-triggered shortcut evaluated before the AI reply engine --
    same role as the standalone reply bot's rulesEngine. Cheaper and more
    deterministic than an LLM call for the "menu", "agent", "STOP"-adjacent
    cases a business wants guaranteed, literal behavior for.
    """

    __tablename__ = "automation_rules"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)

    trigger_keyword: Mapped[str] = mapped_column(String(255), nullable=False)
    match_type: Mapped[RuleMatchType] = mapped_column(
        Enum(RuleMatchType), default=RuleMatchType.CONTAINS, nullable=False
    )

    action_type: Mapped[RuleActionType] = mapped_column(Enum(RuleActionType), nullable=False)
    # AUTO_REPLY: the reply text. HANDOFF: optional note. ADD_TAG: tag name.
    action_value: Mapped[str] = mapped_column(Text, default="", nullable=False)

    priority: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
