import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class Service(Base):
    """
    The catalog of services the AI reply engine is allowed to talk about --
    the grounding data behind "explain about services and answer queries."
    Anything not in here (pricing beyond pricing_notes, legal/contractual
    terms) must be escalated to a human, not guessed.
    """
    __tablename__ = "services"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)

    name: Mapped[str] = mapped_column(String(255), nullable=False)
    summary: Mapped[str] = mapped_column(Text, default="", nullable=False)
    details: Mapped[str] = mapped_column(Text, default="", nullable=False)

    target_institution_types: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    faq: Mapped[list[dict]] = mapped_column(JSONB, default=list, nullable=False)
    pricing_notes: Mapped[str] = mapped_column(Text, default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=lambda: datetime.now(UTC))
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
