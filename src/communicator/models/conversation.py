import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.broadcast import Broadcast
    from communicator.models.contact import Contact
    from communicator.models.draft import Draft
    from communicator.models.message import Message
    from communicator.models.sequence import FollowUpSequence
    from communicator.models.team_member import TeamMember


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

    # Link to Campaign / Broadcast if this conversation was triggered by a campaign
    broadcast_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("broadcasts.id", ondelete="SET NULL"), nullable=True
    )

    # Human-takeover: when true, the AI reply engine stays out of this
    # conversation entirely regardless of auto_reply_enabled, until an
    # agent flips it back off. Independent of `state` -- a HOT_LEAD can
    # still be bot-handled, a NEW conversation can be manually taken over.
    human_takeover: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    assigned_to_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("team_members.id"), nullable=True
    )

    # CSAT Feedback Collection (1 to 5 stars or sentiment rating)
    csat_score: Mapped[int | None] = mapped_column(Integer, nullable=True)
    csat_comment: Mapped[str | None] = mapped_column(Text, nullable=True)

    # Resolution outcome: autonomous_no_handoff | human_assisted | unresolved
    resolution_type: Mapped[str | None] = mapped_column(String(50), nullable=True)

    # WhatsApp 24-hour Session Window tracking
    last_inbound_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )
    last_outbound_at: Mapped[datetime | None] = mapped_column(
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
    broadcast: Mapped["Broadcast | None"] = relationship("Broadcast")
    assigned_to: Mapped["TeamMember | None"] = relationship("TeamMember")
    messages: Mapped[list["Message"]] = relationship(
        "Message",
        back_populates="conversation",
        cascade="all, delete-orphan",
        order_by="Message.sent_at",
    )
    drafts: Mapped[list["Draft"]] = relationship(
        "Draft", back_populates="conversation", cascade="all, delete-orphan"
    )
