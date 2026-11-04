import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

from sqlalchemy import DateTime, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact
    from communicator.models.conversation import Conversation
    from communicator.models.message import Message
    from communicator.models.team_member import TeamMember


class ApprovalType(str, enum.Enum):
    REFUND = "refund"
    DISCOUNT = "discount"
    POLICY_EXCEPTION = "policy_exception"
    INFO_REQUEST = "info_request"
    REPLY_APPROVAL = "reply_approval"


class ApprovalStatus(str, enum.Enum):
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    AMENDED = "amended"


class CopilotApprovalRequest(Base):
    """
    An AI Copilot Teammate Approval Request.
    Enables AI agents and human teammates to collaborate seamlessly from the same
    inbox and customer record. When the AI needs approval for a sensitive action
    (e.g. a refund reply) or needs a piece of information, it asks a teammate and
    then finishes the conversation itself.
    """

    __tablename__ = "copilot_approvals"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    conversation_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("conversations.id", ondelete="CASCADE"), nullable=False, index=True
    )
    contact_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("contacts.id", ondelete="SET NULL"), nullable=True, index=True
    )

    request_type: Mapped[str] = mapped_column(
        String(50), default=ApprovalType.REFUND.value, nullable=False, index=True
    )
    status: Mapped[str] = mapped_column(
        String(50), default=ApprovalStatus.PENDING.value, nullable=False, index=True
    )

    title: Mapped[str] = mapped_column(String(255), nullable=False)
    proposed_reply: Mapped[str] = mapped_column(Text, default="", nullable=False)
    amended_reply: Mapped[str | None] = mapped_column(Text, nullable=True)

    action_data: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)
    internal_note: Mapped[str] = mapped_column(Text, default="", nullable=False)

    question_to_teammate: Mapped[str | None] = mapped_column(Text, nullable=True)
    teammate_answer: Mapped[str | None] = mapped_column(Text, nullable=True)

    assigned_to_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("team_members.id", ondelete="SET NULL"), nullable=True, index=True
    )
    reviewer_name: Mapped[str | None] = mapped_column(String(120), nullable=True)
    reviewer_feedback: Mapped[str | None] = mapped_column(Text, nullable=True)

    dispatched_message_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("messages.id", ondelete="SET NULL"), nullable=True
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
    resolved_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    # Relationships
    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="approvals")
    contact: Mapped["Contact | None"] = relationship("Contact")
    assigned_to: Mapped["TeamMember | None"] = relationship("TeamMember")
    dispatched_message: Mapped["Message | None"] = relationship("Message")
