import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base
from communicator.models.contact import ChannelType, ChannelTypeEnum

if TYPE_CHECKING:
    from communicator.models.conversation import Conversation


class MessageDirection(str, enum.Enum):
    INBOUND = "inbound"
    OUTBOUND = "outbound"


class MessageStatus(str, enum.Enum):
    QUEUED = "queued"
    SENT = "sent"
    DELIVERED = "delivered"
    FAILED = "failed"
    RECEIVED = "received"


class Message(Base):
    """A single inbound or outbound message within a Conversation."""

    __tablename__ = "messages"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    conversation_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("conversations.id"), nullable=False, index=True
    )

    direction: Mapped[MessageDirection] = mapped_column(Enum(MessageDirection), nullable=False)
    channel: Mapped[ChannelType] = mapped_column(ChannelTypeEnum, nullable=False)
    body: Mapped[str] = mapped_column(Text, default="", nullable=False)

    # Internal Team Note: private messages are visible to agents only and never sent to the customer
    is_private: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    sender_type: Mapped[str] = mapped_column(String(20), default="customer", nullable=False)
    sender_name: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    attachments: Mapped[list] = mapped_column(JSONB, default=list, nullable=False)

    provider: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    provider_message_id: Mapped[str] = mapped_column(
        String(255), default="", nullable=False, index=True
    )
    status: Mapped[MessageStatus] = mapped_column(
        Enum(MessageStatus), default=MessageStatus.QUEUED, nullable=False
    )
    provider_metadata: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)
    intent: Mapped[str] = mapped_column(String(50), default="", nullable=False)

    sent_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="messages")
