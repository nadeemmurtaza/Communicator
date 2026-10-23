import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, ForeignKey, String
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact
    from communicator.models.lead import Lead


class Customer(Base):
    """
    One real person within a tenant, independent of which channel they
    reach out on. A Contact is a single channel identity (one WhatsApp
    number, one Instagram handle); a Customer groups every Contact that
    is actually the same person, so the relationship profile -- history,
    memories, inquiries -- is shared across channels instead of siloed
    per-channel. Scoped by tenant_key: each tenant manages its own
    customers, never shared across tenants.

    Contacts are never auto-merged into one Customer on name/number
    similarity -- each new channel identity gets its own Customer unless
    explicitly linked (see POST /contacts/{id}/link-customer).
    """

    __tablename__ = "customers"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", nullable=False, index=True
    )
    lead_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("leads.id"), nullable=True)
    display_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    lead: Mapped["Lead | None"] = relationship("Lead")
    contacts: Mapped[list["Contact"]] = relationship("Contact", back_populates="customer")
