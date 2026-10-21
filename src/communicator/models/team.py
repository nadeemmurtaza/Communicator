import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import Column, DateTime, ForeignKey, String, Table, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.conversation import Conversation
    from communicator.models.team_member import TeamMember

# Join table between Team and TeamMember
team_memberships = Table(
    "team_memberships",
    Base.metadata,
    Column("team_id", UUID(as_uuid=True), ForeignKey("teams.id", ondelete="CASCADE"), primary_key=True),
    Column("team_member_id", UUID(as_uuid=True), ForeignKey("team_members.id", ondelete="CASCADE"), primary_key=True),
)


class Team(Base):
    """Internal support/sales team (e.g. Sales, Support, VIP Accounts)."""

    __tablename__ = "teams"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(100), nullable=False, unique=True)
    description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    auto_assign: Mapped[bool] = mapped_column(default=True, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    members: Mapped[list["TeamMember"]] = relationship(
        "TeamMember", secondary=team_memberships, lazy="selectin"
    )
    conversations: Mapped[list["Conversation"]] = relationship(
        "Conversation", back_populates="team"
    )
