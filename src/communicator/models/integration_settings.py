import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base

SINGLETON_ID = uuid.UUID("00000000-0000-0000-0000-000000000001")


class IntegrationSettings(Base):
    """
    Single-row table holding what the Connections page manages: the WhatsApp
    (Meta Cloud API) credentials, the business profile and reply guidelines
    the AI reply engine grounds itself in, and the auto-reply kill switch.

    One fixed row (id == SINGLETON_ID) rather than a key-value table --
    there's exactly one business and one WhatsApp connection per deployment,
    so a real table with real columns beats a generic settings blob.
    """

    __tablename__ = "integration_settings"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)

    # WhatsApp / Meta Cloud API connection
    whatsapp_provider: Mapped[str] = mapped_column(String(20), default="meta", nullable=False)
    meta_access_token: Mapped[str] = mapped_column(Text, default="", nullable=False)
    meta_phone_number_id: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    meta_waba_id: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    meta_app_secret: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    meta_webhook_verify_token: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    # Last time /verify-whatsapp succeeded, and what it found -- shown as
    # connection status on the Connections page without re-calling Meta.
    whatsapp_verified_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )
    whatsapp_verified_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    whatsapp_display_number: Mapped[str] = mapped_column(String(50), default="", nullable=False)

    # Business profile + agent guidelines -- grounds the reply engine's
    # system prompt instead of a hardcoded business name.
    business_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    business_description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    agent_guidelines: Mapped[str] = mapped_column(Text, default="", nullable=False)
    auto_reply_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
