from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class SLAPolicy(Base):
    """
    Service Level Agreement (SLA) policy defining first response and
    resolution target thresholds based on priority and channels.
    """

    __tablename__ = "sla_policies"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(120), nullable=False, unique=True)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)

    # Thresholds in minutes
    first_response_time_threshold: Mapped[int] = mapped_column(Integer, default=60, nullable=False)
    resolution_time_threshold: Mapped[int] = mapped_column(Integer, default=1440, nullable=False)

    # Business hours flag: if true, pauses timer outside 9am-6pm M-F
    only_during_business_hours: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)

    # Matching criteria: priorities = ["urgent", "high"] or [] for all
    priorities: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    inbox_ids: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
