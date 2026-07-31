import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base
from communicator.models.contact import ChannelType


class FollowUpSequence(Base):
    """A named, reusable cadence of SequenceSteps (e.g. day 0/3/7/14)."""
    __tablename__ = "follow_up_sequences"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=lambda: datetime.now(UTC))

    steps: Mapped[list["SequenceStep"]] = relationship(
        "SequenceStep",
        back_populates="sequence",
        cascade="all, delete-orphan",
        order_by="SequenceStep.step_number",
    )


class SequenceStep(Base):
    """
    One templated message in a FollowUpSequence. delay_days is measured
    from the previous step (or from first contact for step 1).
    """
    __tablename__ = "sequence_steps"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    sequence_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("follow_up_sequences.id"), nullable=False, index=True)

    step_number: Mapped[int] = mapped_column(Integer, nullable=False)
    delay_days: Mapped[int] = mapped_column(Integer, nullable=False)
    channel: Mapped[ChannelType] = mapped_column(Enum(ChannelType), nullable=False)
    template: Mapped[str] = mapped_column(Text, default="", nullable=False)

    sequence: Mapped["FollowUpSequence"] = relationship("FollowUpSequence", back_populates="steps")
