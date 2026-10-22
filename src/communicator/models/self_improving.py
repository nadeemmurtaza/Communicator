import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import DateTime, Float, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base


class InsightCategory(str, enum.Enum):
    HUMAN_EXEMPLAR = "human_exemplar"
    TREND = "trend"
    POLICY_REFINEMENT = "policy_refinement"
    CONTENT_GAP = "content_gap"
    TEAM_COACHING = "team_coaching"


class InsightStatus(str, enum.Enum):
    OPEN = "open"
    APPLIED = "applied"
    DISMISSED = "dismissed"


class AgentInsight(Base):
    """
    Self-improving AI Insight or Suggestion.
    Represents patterns learned from top human reps, emerging customer trends,
    or actionable recommendations to boost Agent resolution rate and CSAT.
    """

    __tablename__ = "agent_insights"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    category: Mapped[str] = mapped_column(
        String(50), default=InsightCategory.TREND.value, nullable=False, index=True
    )
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)

    # Human rep attribution if learned from top agent
    human_rep_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("team_members.id", ondelete="SET NULL"), nullable=True
    )
    human_rep_name: Mapped[str | None] = mapped_column(String(100), nullable=True)

    # Winning response text or pattern extracted from top human conversations
    exemplar_content: Mapped[str | None] = mapped_column(Text, nullable=True)

    # Impact metrics
    impact_score: Mapped[float] = mapped_column(Float, default=8.0, nullable=False)
    estimated_resolution_lift: Mapped[float] = mapped_column(Float, default=3.5, nullable=False)

    # Suggested 1-click action: apply_policy | add_kb | retrain_agent | coach_team
    suggested_action: Mapped[str] = mapped_column(
        String(50), default="retrain_agent", nullable=False
    )
    action_payload: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    status: Mapped[str] = mapped_column(
        String(30), default=InsightStatus.OPEN.value, nullable=False, index=True
    )
    applied_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )

    retraining_events: Mapped[list["AgentRetrainingEvent"]] = relationship(
        "AgentRetrainingEvent", back_populates="insight"
    )


class AgentRetrainingEvent(Base):
    """
    Historical log of an agent retraining iteration.
    Tracks before/after resolution rate, version bump, and the specific learnings incorporated.
    """

    __tablename__ = "agent_retraining_events"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    trigger_type: Mapped[str] = mapped_column(
        String(50), default="human_exemplar_sync", nullable=False
    )

    version_from: Mapped[str] = mapped_column(String(20), default="2.1", nullable=False)
    version_to: Mapped[str] = mapped_column(String(20), default="2.2", nullable=False)

    insight_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("agent_insights.id", ondelete="SET NULL"), nullable=True
    )

    changes_applied: Mapped[list[dict[str, Any]]] = mapped_column(
        JSONB, default=list, nullable=False
    )
    status: Mapped[str] = mapped_column(String(30), default="applied", nullable=False)

    # Before and after performance impact
    baseline_resolution_rate: Mapped[float] = mapped_column(Float, default=86.4, nullable=False)
    post_training_resolution_rate: Mapped[float] = mapped_column(
        Float, default=91.2, nullable=False
    )
    resolution_lift: Mapped[float] = mapped_column(Float, default=4.8, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )

    insight: Mapped["AgentInsight | None"] = relationship(
        "AgentInsight", back_populates="retraining_events"
    )
