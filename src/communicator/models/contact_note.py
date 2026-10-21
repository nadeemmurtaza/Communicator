import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact


class ContactNote(Base):
    """
    Internal notes added to a customer profile by support/sales agents.
    """

    __tablename__ = "contact_notes"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    contact_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("contacts.id", ondelete="CASCADE"), nullable=False, index=True
    )
    author_name: Mapped[str] = mapped_column(String(100), default="Agent", nullable=False)
    body: Mapped[str] = mapped_column(Text, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    contact: Mapped["Contact"] = relationship("Contact", back_populates="notes")
