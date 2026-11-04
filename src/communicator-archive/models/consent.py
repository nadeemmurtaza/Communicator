import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, Enum, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact


class ConsentStatus(str, enum.Enum):
    UNKNOWN = "unknown"
    OPTED_IN = "opted_in"
    OPTED_OUT = "opted_out"


class ConsentRecord(Base):
    """
    Append-only log of consent changes for a Contact. The compliance gate
    (see communicator.compliance.optout) reads the latest row per contact;
    every inbound STOP-class message writes a new OPTED_OUT row before
    anything else in the reply pipeline runs.
    """

    __tablename__ = "consent_records"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    contact_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("contacts.id"), nullable=False, index=True
    )

    status: Mapped[ConsentStatus] = mapped_column(Enum(ConsentStatus), nullable=False)
    # e.g. "harvester_sync", "inbound_stop_keyword", "inbound_start_keyword", "manual"
    source: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    note: Mapped[str] = mapped_column(Text, default="", nullable=False)

    recorded_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    contact: Mapped["Contact"] = relationship("Contact", back_populates="consent_records")
