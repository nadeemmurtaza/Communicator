import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, backref, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact


class BroadcastStatus(str, enum.Enum):
    DRAFT = "draft"
    SCHEDULED = "scheduled"
    SENDING = "sending"
    COMPLETED = "completed"
    FAILED = "failed"


class RecipientStatus(str, enum.Enum):
    PENDING = "pending"
    SENT = "sent"
    DELIVERED = "delivered"
    READ = "read"
    FAILED = "failed"
    SKIPPED = "skipped"


class Broadcast(Base):
    """
    A bulk send of one approved WhatsApp template message to contacts
    filtered by audience tags (e.g. VIP customers, newsletter).
    Supports scheduled sends, dynamic variable mapping, delivery analytics,
    and direct shared inbox follow-up.
    """

    __tablename__ = "broadcasts"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", nullable=False, index=True
    )
    name: Mapped[str] = mapped_column(String(255), nullable=False)

    # Business purpose -- e.g. "business_awareness", "operational_discovery",
    # "lead_conversion", "client_communication", "re_engagement" (spec Part 2)
    campaign_type: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    objective: Mapped[str] = mapped_column(String(500), default="", nullable=False)

    # When set, the campaign draws its template content from the tenant's
    # approved CampaignTemplate library instead of the free-form fields
    # below (kept for backward compatibility / manual one-offs).
    campaign_template_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("campaign_templates.id"), nullable=True
    )

    # Sequence support (spec Part 4): a campaign can have follow-up steps
    # that only fire for recipients who haven't replied/opted out since
    # the previous step.
    parent_campaign_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("broadcasts.id"), nullable=True, index=True
    )
    step_index: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    step_delay_days: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    stop_on_reply: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    template_name: Mapped[str] = mapped_column(String(255), nullable=False)
    template_language: Mapped[str] = mapped_column(String(20), default="en_US", nullable=False)
    body_preview: Mapped[str] = mapped_column(Text, default="", nullable=False)
    template_params: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    tag_name: Mapped[str] = mapped_column(String(255), nullable=False)
    status: Mapped[BroadcastStatus] = mapped_column(
        Enum(BroadcastStatus), default=BroadcastStatus.DRAFT, nullable=False
    )
    scheduled_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    audience_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    sent_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    delivered_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    read_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    failed_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    skipped_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    recipients: Mapped[list["BroadcastRecipient"]] = relationship(
        "BroadcastRecipient", back_populates="broadcast", cascade="all, delete-orphan"
    )
    steps: Mapped[list["Broadcast"]] = relationship(
        "Broadcast", backref=backref("parent_campaign", remote_side="Broadcast.id")
    )


class BroadcastRecipient(Base):
    __tablename__ = "broadcast_recipients"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    broadcast_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("broadcasts.id", ondelete="CASCADE"), nullable=False, index=True
    )
    contact_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("contacts.id"), nullable=False)

    status: Mapped[RecipientStatus] = mapped_column(
        Enum(RecipientStatus), default=RecipientStatus.PENDING, nullable=False
    )
    provider_message_id: Mapped[str] = mapped_column(
        String(255), default="", nullable=False, index=True
    )
    error: Mapped[str] = mapped_column(Text, default="", nullable=False)

    sent_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    delivered_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    read_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    broadcast: Mapped["Broadcast"] = relationship("Broadcast", back_populates="recipients")
    contact: Mapped["Contact"] = relationship("Contact")
