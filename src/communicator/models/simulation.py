import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, Float, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base


class PolicyStatus(str, enum.Enum):
    ACTIVE = "active"
    DRAFT = "draft"
    TESTING = "testing"
    ARCHIVED = "archived"


class SimulationVerdict(str, enum.Enum):
    PASSED = "passed"
    FAILED = "failed"
    ESCALATED = "escalated"
    WARNING = "warning"


class AgentPolicy(Base):
    """
    Business process, SOP, or policy document used to train and guide our own autonomous Agent.
    Contains rules, step-by-step procedures, and guardrails for handling complex customer queries.
    """

    __tablename__ = "agent_policies"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    category: Mapped[str] = mapped_column(String(50), default="general", nullable=False, index=True)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    version: Mapped[str] = mapped_column(String(20), default="1.0", nullable=False)

    status: Mapped[str] = mapped_column(String(30), default="active", nullable=False, index=True)
    
    # Process steps: ordered list of steps our Agent must follow (e.g. Step 1: Collect order number)
    process_steps: Mapped[list[dict[str, Any]]] = mapped_column(JSONB, default=list, nullable=False)
    
    # Rules & constraints (e.g. max_refund_amount: 50.0, require_receipt: false)
    rules: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)
    
    # Negative constraints and guardrails (e.g. "Never promise same-day delivery without checking inventory")
    guardrails: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )

    scenarios: Mapped[list["SimulationScenario"]] = relationship(
        "SimulationScenario", back_populates="policy"
    )


class SimulationScenario(Base):
    """
    Pre-configured customer conversation scenario used to test our autonomous Agent before going live.
    Simulates personas with realistic intents, sentiment, and multi-step inquiry flows.
    """

    __tablename__ = "simulation_scenarios"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    category: Mapped[str] = mapped_column(String(50), default="general", nullable=False, index=True)
    
    policy_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("agent_policies.id", ondelete="SET NULL"), nullable=True, index=True
    )

    # Customer persona details: name, tone, anger_level, patience, customer_tier
    customer_persona: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)
    
    # Expected outcome: resolve_autonomous | escalate_to_human | request_more_info | deny_politely
    expected_outcome: Mapped[str] = mapped_column(String(50), default="resolve_autonomous", nullable=False)

    # Multi-turn customer message script or conversational triggers
    customer_messages: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )

    policy: Mapped["AgentPolicy | None"] = relationship("AgentPolicy", back_populates="scenarios")
    runs: Mapped[list["SimulationRun"]] = relationship(
        "SimulationRun", back_populates="scenario", cascade="all, delete-orphan"
    )


class SimulationRun(Base):
    """
    The result of an executed simulation test run.
    Contains turn-by-turn conversation logs, policy adherence score, and quality evaluation.
    """

    __tablename__ = "simulation_runs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    scenario_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("simulation_scenarios.id", ondelete="CASCADE"), nullable=False, index=True
    )
    policy_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("agent_policies.id", ondelete="SET NULL"), nullable=True
    )

    status: Mapped[str] = mapped_column(String(30), default="completed", nullable=False)
    verdict: Mapped[str] = mapped_column(String(30), default="passed", nullable=False, index=True)

    # Scores (0.0 to 100.0)
    policy_adherence_score: Mapped[float] = mapped_column(Float, default=95.0, nullable=False)
    empathy_score: Mapped[float] = mapped_column(Float, default=90.0, nullable=False)
    accuracy_score: Mapped[float] = mapped_column(Float, default=92.0, nullable=False)
    overall_score: Mapped[float] = mapped_column(Float, default=92.5, nullable=False)

    turn_count: Mapped[int] = mapped_column(Integer, default=2, nullable=False)
    
    # Turn-by-turn transcript: [{"role": "customer"|"agent", "content": "...", "policy_checked": "...", "step": 1}]
    transcript: Mapped[list[dict[str, Any]]] = mapped_column(JSONB, default=list, nullable=False)
    
    evaluation_summary: Mapped[str] = mapped_column(Text, default="", nullable=False)
    recommendations: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )

    scenario: Mapped["SimulationScenario"] = relationship("SimulationScenario", back_populates="runs")
    policy: Mapped["AgentPolicy | None"] = relationship("AgentPolicy")
