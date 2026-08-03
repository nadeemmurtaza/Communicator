import enum
import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, Index, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class EmailComplaintType(str, enum.Enum):
    RECIPIENT_SPAM = "recipient_spam"
    FEEDBACK_LOOP = "feedback_loop"
    DMARC_FORENSIC = "dmarc_forensic"
    AUTHENTICATION_FAILURE = "authentication_failure"
    OTHER = "other"


class EmailComplaint(Base):
    """Append-only, idempotent evidence of an email complaint or DMARC failure report.

    ``evidence_metadata`` must contain only normalized, redacted evidence needed for
    investigation. Provider webhook bodies and complete message content do not belong
    in this table.
    """

    __tablename__ = "email_complaints"
    __table_args__ = (
        Index("ix_email_complaints_recipient_reported", "recipient_email", "reported_at"),
        Index("ix_email_complaints_provider_reported", "provider", "reported_at"),
    )

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    contact_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("contacts.id", ondelete="SET NULL"), nullable=True, index=True
    )
    message_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("messages.id", ondelete="SET NULL"), nullable=True, index=True
    )

    source_event_id: Mapped[str] = mapped_column(String(255), nullable=False, unique=True)
    complaint_type: Mapped[EmailComplaintType] = mapped_column(
        Enum(EmailComplaintType), nullable=False
    )
    provider: Mapped[str] = mapped_column(String(64), nullable=False)
    provider_message_id: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    recipient_email: Mapped[str] = mapped_column(String(320), nullable=False, index=True)

    dmarc_result: Mapped[str] = mapped_column(String(32), default="", nullable=False)
    spf_aligned: Mapped[bool | None] = mapped_column(Boolean, nullable=True)
    dkim_aligned: Mapped[bool | None] = mapped_column(Boolean, nullable=True)
    source_ip: Mapped[str] = mapped_column(String(64), default="", nullable=False)
    evidence_metadata: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)
    suppression_applied: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)

    reported_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    received_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
