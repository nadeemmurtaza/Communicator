import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, Float, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class KnowledgeBaseEntry(Base):
    """
    Centralized Knowledge Hub entry.
    Powers Fin AI Agent, Copilot in the shared inbox, and self-serve customer Help Center.
    Organized into structured topics with cross-touchpoint performance tracking.
    """

    __tablename__ = "knowledge_base_entries"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", nullable=False, index=True
    )
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    content: Mapped[str] = mapped_column(Text, default="", nullable=False)
    source_type: Mapped[str] = mapped_column(String(50), default="note", nullable=False)
    source_url: Mapped[str | None] = mapped_column(String(1024), nullable=True)
    file_name: Mapped[str | None] = mapped_column(String(255), nullable=True)

    # Topic organization
    topic: Mapped[str] = mapped_column(String(100), default="General", nullable=False, index=True)

    # Touchpoints where this article is deployed: ["agent", "copilot", "help_center"]
    target_touchpoints: Mapped[list[str]] = mapped_column(
        JSONB, default=lambda: ["agent", "copilot", "help_center"], nullable=False
    )
    status: Mapped[str] = mapped_column(String(30), default="published", nullable=False)

    # Performance metrics
    resolution_rate: Mapped[float] = mapped_column(Float, default=85.0, nullable=False)
    usage_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    helpful_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    unhelpful_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )


class KnowledgeGap(Base):
    """
    Auto-detected knowledge gaps from customer conversations and queries.
    Identifies recurring unanswered questions and proposes suggested FAQs (✨ Suggested FAQ).
    """

    __tablename__ = "knowledge_gaps"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    question: Mapped[str] = mapped_column(String(500), nullable=False)
    suggested_answer: Mapped[str] = mapped_column(Text, default="", nullable=False)
    source_query: Mapped[str] = mapped_column(Text, default="", nullable=False)
    frequency: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    status: Mapped[str] = mapped_column(
        String(30), default="suggested", nullable=False
    )  # suggested, approved, dismissed

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )


class KnowledgeRecommendation(Base):
    """
    AI Content Optimization Recommendations:
    Highlights content gaps, customer data gaps, and action gaps to systematically
    improve Agent Fin's automated resolution rate.
    """

    __tablename__ = "knowledge_recommendations"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    gap_type: Mapped[str] = mapped_column(
        String(50), default="content_gap", nullable=False, index=True
    )  # content_gap, customer_data_gap, action_gap
    topic: Mapped[str] = mapped_column(String(100), default="General", nullable=False, index=True)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    impact_on_resolution: Mapped[float] = mapped_column(Float, default=3.5, nullable=False)
    suggested_action: Mapped[str] = mapped_column(Text, default="", nullable=False)
    suggested_content: Mapped[str] = mapped_column(Text, default="", nullable=False)
    status: Mapped[str] = mapped_column(
        String(30), default="open", nullable=False, index=True
    )  # open, applied, dismissed

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    resolved_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
