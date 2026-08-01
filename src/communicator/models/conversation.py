import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, Enum, ForeignKey, Integer, String
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact
    from communicator.models.draft import Draft
    from communicator.models.message import Message
    from communicator.models.sequence import FollowUpSequence


class ConversationState(str, enum.Enum):
    NEW = "new"
    AWAITING_REPLY = "awaiting_reply"
    ENGAGED = "engaged"
    HOT_LEAD = "hot_lead"
    HANDED_OFF = "handed_off"
    OPTED_OUT = "opted_out"
    CLOSED_WON = "closed_won"
    CLOSED_LOST = "closed_lost"


class Conversation(Base):
    """One message thread with a single Contact, on a single channel."""

    __tablename__ = "conversations"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    contact_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("contacts.id"), nullable=False, index=True
    )

    state: Mapped[ConversationState] = mapped_column(
        Enum(ConversationState), default=ConversationState.NEW, nullable=False
    )

    sequence_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("follow_up_sequences.id"), nullable=True
    )
    sequence_step: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    # Which SequenceStep.variant this Conversation is locked into for the
    # lifetime of its enrollment -- assigned once at enroll_conversation()
    # time, never re-rolled, so A/B results aren't contaminated mid-sequence.
    sequence_variant: Mapped[str | None] = mapped_column(String(50), nullable=True)
    next_followup_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )

    contact: Mapped["Contact"] = relationship("Contact", back_populates="conversations")
    sequence: Mapped["FollowUpSequence | None"] = relationship("FollowUpSequence")
    messages: Mapped[list["Message"]] = relationship(
        "Message",
        back_populates="conversation",
        cascade="all, delete-orphan",
        order_by="Message.sent_at",
    )
    drafts: Mapped[list["Draft"]] = relationship(
        "Draft", back_populates="conversation", cascade="all, delete-orphan"
    )
