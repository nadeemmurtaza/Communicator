import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, Integer, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class KnowledgeBaseEntry(Base):
    """
    Free-form business knowledge (FAQ, policy, product doc, pasted URL
    content, crawled helpcenter, uploaded PDF) the reply engine is grounded in
    alongside the structured Service catalog.
    """

    __tablename__ = "knowledge_base_entries"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    content: Mapped[str] = mapped_column(Text, default="", nullable=False)
    source_type: Mapped[str] = mapped_column(String(50), default="note", nullable=False)
    source_url: Mapped[str | None] = mapped_column(String(1024), nullable=True)
    file_name: Mapped[str | None] = mapped_column(String(255), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
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
    status: Mapped[str] = mapped_column(String(30), default="suggested", nullable=False)  # suggested, approved, dismissed

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
