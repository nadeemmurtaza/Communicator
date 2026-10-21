import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base
from communicator.models.contact import ChannelType, ChannelTypeEnum

if TYPE_CHECKING:
    from communicator.models.conversation import Conversation


class Inbox(Base):
    """
    A communication channel inbox (e.g. WhatsApp, Email, Live Chat, SMS).
    Holds channel credentials, business hours, greetings, and routing configuration.
    """

    __tablename__ = "inboxes"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    channel_type: Mapped[ChannelType] = mapped_column(ChannelTypeEnum, nullable=False, index=True)
    phone_number: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    avatar_url: Mapped[str] = mapped_column(String(500), default="", nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    # Channel-specific credentials (e.g. meta access_token, phone_number_id, waba_id, app_secret)
    credentials: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    # Channel-level automations
    greeting_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    greeting_message: Mapped[str] = mapped_column(Text, default="", nullable=False)
    out_of_office_enabled: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    out_of_office_message: Mapped[str] = mapped_column(Text, default="", nullable=False)
    csat_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    auto_read_receipts: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    working_hours_enabled: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    working_hours_start: Mapped[int] = mapped_column(default=9, nullable=False)
    working_hours_end: Mapped[int] = mapped_column(default=18, nullable=False)
    timezone: Mapped[str] = mapped_column(String(50), default="UTC", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )

    conversations: Mapped[list["Conversation"]] = relationship(
        "Conversation", back_populates="inbox", cascade="all"
    )
