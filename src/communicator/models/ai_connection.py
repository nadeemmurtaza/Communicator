import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, Float, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class AIProvider(str, enum.Enum):
    ANTHROPIC = "anthropic"
    OPENAI = "openai"
    GOOGLE = "google"
    DEEPSEEK = "deepseek"
    GROQ = "groq"
    OLLAMA = "ollama"
    CUSTOM = "custom"


class AIJobType(str, enum.Enum):
    SELF_IMPROVING = "self_improving"
    AUTONOMOUS_AGENT = "autonomous_agent"
    COPILOT_DRAFTS = "copilot_drafts"
    SIMULATIONS = "simulations"
    QUALITY_TRIAGE = "quality_triage"
    GLOBAL = "global"


class AIConnection(Base):
    """
    Connected AI Model provider configuration.
    Allows connecting any model (Claude, GPT-4o, Gemini, DeepSeek, Groq, Ollama, custom OpenAI-compatible endpoint)
    and routing specific models to distinct AI jobs.
    """

    __tablename__ = "ai_connections"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    provider: Mapped[str] = mapped_column(
        String(50), default=AIProvider.ANTHROPIC.value, nullable=False, index=True
    )
    model_id: Mapped[str] = mapped_column(
        String(100), nullable=False
    )  # e.g. claude-3-5-sonnet, gpt-4o, deepseek-r1

    # Credentials & Endpoint
    api_key: Mapped[str] = mapped_column(Text, default="", nullable=False)
    base_url: Mapped[str | None] = mapped_column(
        String(255), nullable=True
    )  # e.g. http://localhost:11434/v1 for Ollama

    # Generation parameters
    temperature: Mapped[float] = mapped_column(Float, default=0.7, nullable=False)
    max_tokens: Mapped[int] = mapped_column(Integer, default=1024, nullable=False)

    # Active state & Job routing
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    assigned_jobs: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    # Health & Verification
    last_tested_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    test_status: Mapped[str] = mapped_column(
        String(30), default="untested", nullable=False
    )  # connected | failed | untested
    latency_ms: Mapped[int | None] = mapped_column(Integer, nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )


class AICustomPolicy(Base):
    """
    Custom policies, rules, and guidelines for specific AI jobs
    (e.g. Self-Improving learning rules, Autonomous Agent guidelines, Copilot rules).
    """

    __tablename__ = "ai_custom_policies"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    job_type: Mapped[str] = mapped_column(
        String(50), default=AIJobType.SELF_IMPROVING.value, unique=True, nullable=False, index=True
    )

    # Tone & persona
    tone: Mapped[str] = mapped_column(String(50), default="empathetic", nullable=False)
    guidelines: Mapped[str] = mapped_column(Text, default="", nullable=False)

    # Dynamic rules: [{"id": "r1", "rule": "...", "enabled": true, "category": "..."}]
    rules: Mapped[list[dict[str, Any]]] = mapped_column(JSONB, default=list, nullable=False)

    # Guardrails: ["Never commit to refunds > $150 without approval", "Never reveal competitor names"]
    guardrails: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    # Self-improving specific controls
    auto_approval_mode: Mapped[str] = mapped_column(
        String(50), default="require_manager_review", nullable=False
    )
    min_csat_learning_threshold: Mapped[float] = mapped_column(Float, default=4.0, nullable=False)
    max_autonomous_refund: Mapped[float] = mapped_column(Float, default=50.0, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )
