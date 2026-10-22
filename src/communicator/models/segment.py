from __future__ import annotations

import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, String, Text, UniqueConstraint
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class ContactSegment(Base):
    """
    Saved dynamic contact segment defining multi-channel audience targeting rules.
    Filters contacts dynamically based on channels, tags, search queries, date ranges, and custom attributes.
    Unique per tenant, not globally.
    """

    __tablename__ = "contact_segments"
    __table_args__ = (UniqueConstraint("tenant_key", "name", name="uq_segment_tenant_name"),)

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", nullable=False, index=True
    )
    name: Mapped[str] = mapped_column(String(120), nullable=False)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)

    # Filter criteria dictionary:
    # {
    #   "channels": ["whatsapp", "email", "sms", "tiktok"], # or empty list for any
    #   "tags": ["vip", "leads"],
    #   "tag_match_mode": "any" | "all",
    #   "search": "term",
    #   "created_after": "ISO string",
    #   "created_before": "ISO string"
    # }
    filters: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
