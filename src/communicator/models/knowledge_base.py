import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class KnowledgeBaseEntry(Base):
    """
    Free-form business knowledge (FAQ, policy, product doc, pasted URL
    content) the reply engine is grounded in alongside the structured
    Service catalog -- for the stuff that doesn't fit a Service record
    (returns policy, hours, "who are you", etc).
    """

    __tablename__ = "knowledge_base_entries"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    content: Mapped[str] = mapped_column(Text, default="", nullable=False)
    source_type: Mapped[str] = mapped_column(String(20), default="note", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
