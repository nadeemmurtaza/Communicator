import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, Integer, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class TeamMember(Base):
    """A human agent conversations can be assigned to (multi-agent inbox)."""

    __tablename__ = "team_members"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    email: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    # RBAC Role & Permissions
    role: Mapped[str] = mapped_column(String(50), default="agent", nullable=False)
    custom_permissions: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    # Auto-Assignment & Capacity Routing
    availability_status: Mapped[str] = mapped_column(String(20), default="online", nullable=False)  # online | busy | offline
    max_concurrent_conversations: Mapped[int] = mapped_column(Integer, default=5, nullable=False)
    last_assigned_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
