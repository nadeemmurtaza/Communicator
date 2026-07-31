import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact


class Lead(Base):
    """
    Local read cache of a Harvester CanonicalInstitution, refreshed by the
    Lead Sync Worker from Harvester's API. Never treated as the source of
    truth for lead data -- Harvester owns that.
    """
    __tablename__ = "leads"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    harvester_institution_id: Mapped[str] = mapped_column(String(64), nullable=False, unique=True, index=True)

    name: Mapped[str] = mapped_column(String(255), nullable=False)
    institution_type: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    city: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    qualification_rationale: Mapped[str] = mapped_column(Text, default="", nullable=False)

    synced_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=lambda: datetime.now(UTC))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=lambda: datetime.now(UTC))

    contacts: Mapped[list["Contact"]] = relationship(
        "Contact", back_populates="lead", cascade="all, delete-orphan"
    )
