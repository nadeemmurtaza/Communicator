import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, Enum, Float, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.conversation import Conversation


class DraftStatus(str, enum.Enum):
    PENDING_REVIEW = "pending_review"
    APPROVED = "approved"
    AUTO_APPROVED = "auto_approved"
    REJECTED = "rejected"
    SENT = "sent"


class Draft(Base):
    """
    An AI-generated candidate reply awaiting send, modeled directly on
    Harvester's OutreachDraft human-approval gate.
    """

    __tablename__ = "drafts"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    conversation_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("conversations.id"), nullable=False, index=True
    )

    body: Mapped[str] = mapped_column(Text, default="", nullable=False)
    # Model's self-reported confidence; below the configured threshold (or
    # for any hot-lead/complaint/off-catalog question) the draft stays
    # PENDING_REVIEW regardless of this value.
    confidence: Mapped[float] = mapped_column(Float, default=0.0, nullable=False)
    # Which Service/FAQ entries the draft drew on, for reviewer sanity-checking.
    citations: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    # Twilio Content API SID to send this draft as, if it's a WhatsApp
    # message that may need to go out as a pre-approved template rather
    # than freeform text -- see communicator.services.followup_scheduler.
    content_sid: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    status: Mapped[DraftStatus] = mapped_column(
        Enum(DraftStatus), default=DraftStatus.PENDING_REVIEW, nullable=False
    )
    reviewer: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    reviewer_feedback: Mapped[str] = mapped_column(Text, default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )

    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="drafts")
