from __future__ import annotations

import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, Float, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base

SINGLETON_TRANSLATION_SETTINGS_ID = uuid.UUID("00000000-0000-0000-0000-000000000066")


class TranslationSettings(Base):
    """
    Workspace-wide Bi-Directional Real-Time Auto-Translation configuration.
    Enables instant translation of incoming customer messages and outgoing agent replies.
    """

    __tablename__ = "translation_settings"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    is_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    auto_translate_inbound: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    auto_translate_outbound: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    agent_primary_language: Mapped[str] = mapped_column(String(10), default="en", nullable=False)
    preferred_engine: Mapped[str] = mapped_column(
        String(50), default="ai_gateway", nullable=False
    )  # ai_gateway | google | deepl | libretranslate

    # Confidence threshold to trigger automatic translation (0.0 - 1.0)
    confidence_threshold: Mapped[float] = mapped_column(Float, default=0.6, nullable=False)

    # Enabled language codes (empty list means all 50+ supported languages enabled)
    enabled_languages: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )
