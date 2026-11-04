from __future__ import annotations

import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, String, Text, UniqueConstraint
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class CustomAttributeDefinition(Base):
    """
    Schema definition for enterprise CRM custom attributes attached to Contacts or Conversations.
    Supports dynamic types: text, number, currency, boolean, list (select), date, and link (URL).
    """

    __tablename__ = "custom_attribute_definitions"
    __table_args__ = (
        UniqueConstraint("name", "attribute_model", name="uq_custom_attribute_name_model"),
    )

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)

    # Identifier slug e.g. "account_tier" or "arr_usd"
    name: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    display_name: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str] = mapped_column(String(500), default="", nullable=False)

    # 'contact_attribute' or 'conversation_attribute'
    attribute_model: Mapped[str] = mapped_column(
        String(50), default="contact_attribute", nullable=False, index=True
    )

    # 'text' | 'number' | 'currency' | 'boolean' | 'list' | 'date' | 'link'
    attribute_type: Mapped[str] = mapped_column(String(50), default="text", nullable=False)

    default_value: Mapped[str | None] = mapped_column(Text, nullable=True)
    options: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    is_required: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)

    regex_pattern: Mapped[str | None] = mapped_column(String(255), nullable=True)
    regex_cue: Mapped[str | None] = mapped_column(String(255), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )
