import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

from sqlalchemy import Boolean, DateTime, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact
    from communicator.models.conversation import Conversation
    from communicator.models.team_member import TeamMember


class TicketType(str, enum.Enum):
    CUSTOMER_ISSUE = "customer_issue"
    BACK_OFFICE = "back_office"
    TRACKER = "tracker"


class TicketStatus(str, enum.Enum):
    OPEN = "open"
    IN_PROGRESS = "in_progress"
    WAITING_ON_BACK_OFFICE = "waiting_on_back_office"
    WAITING_ON_CUSTOMER = "waiting_on_customer"
    RESOLVED = "resolved"
    CLOSED = "closed"


class TicketPriority(str, enum.Enum):
    URGENT = "urgent"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class TicketDepartment(str, enum.Enum):
    SUPPORT = "support"
    FINANCE = "finance"
    LOGISTICS = "logistics"
    ENGINEERING = "engineering"
    WAREHOUSE = "warehouse"
    LEGAL = "legal"


class ActivityType(str, enum.Enum):
    COMMENT = "comment"
    BACK_OFFICE_NOTE = "back_office_note"
    STATUS_CHANGE = "status_change"
    ASSIGNMENT = "assignment"
    BROADCAST = "broadcast"
    LINKED_TRACKER = "linked_tracker"
    RESOLUTION = "resolution"


class Ticket(Base):
    """
    A unified ticketing system supporting:
    1. Customer Issues (originating from conversations/contacts)
    2. Back-Office Collaboration (Finance, Logistics, Engineering, Warehouse, Legal tasks)
    3. Tracker Tickets (Widespread incidents aggregating child issues with broadcast resolution)
    """

    __tablename__ = "tickets"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    ticket_number: Mapped[str] = mapped_column(String(32), unique=True, index=True, nullable=False)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str] = mapped_column(Text, nullable=False)

    ticket_type: Mapped[str] = mapped_column(
        String(50), default="customer_issue", nullable=False, index=True
    )
    status: Mapped[str] = mapped_column(String(30), default="open", nullable=False, index=True)
    priority: Mapped[str] = mapped_column(String(20), default="medium", nullable=False)
    department: Mapped[str] = mapped_column(
        String(50), default="support", nullable=False, index=True
    )

    # Optional conversation link
    conversation_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("conversations.id", ondelete="SET NULL"), nullable=True, index=True
    )
    # Optional contact link
    contact_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("contacts.id", ondelete="SET NULL"), nullable=True, index=True
    )
    # Parent Tracker ticket if this ticket is a child of a widespread problem
    tracker_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("tickets.id", ondelete="SET NULL"), nullable=True, index=True
    )
    # Assigned team member
    assigned_to_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("team_members.id", ondelete="SET NULL"), nullable=True, index=True
    )

    # Resolution outcome & broadcast tracking
    resolution_summary: Mapped[str | None] = mapped_column(Text, nullable=True)
    customer_broadcast_sent: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    resolved_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )

    # Relationships
    conversation: Mapped["Conversation | None"] = relationship(
        "Conversation", foreign_keys=[conversation_id], back_populates="tickets"
    )
    contact: Mapped["Contact | None"] = relationship(
        "Contact", foreign_keys=[contact_id], back_populates="tickets"
    )
    assigned_to: Mapped["TeamMember | None"] = relationship(
        "TeamMember", foreign_keys=[assigned_to_id]
    )

    # Self-referential parent/child for Tracker Tickets
    tracker: Mapped["Ticket | None"] = relationship(
        "Ticket", remote_side=[id], foreign_keys=[tracker_id], back_populates="child_tickets"
    )
    child_tickets: Mapped[list["Ticket"]] = relationship(
        "Ticket", foreign_keys=[tracker_id], back_populates="tracker"
    )

    # Internal activities & back-office collaboration thread
    activities: Mapped[list["TicketActivity"]] = relationship(
        "TicketActivity",
        back_populates="ticket",
        cascade="all, delete-orphan",
        order_by="TicketActivity.created_at",
    )


class TicketActivity(Base):
    """Internal back-office note, status update, assignment, or broadcast activity."""

    __tablename__ = "ticket_activities"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    ticket_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("tickets.id", ondelete="CASCADE"), nullable=False, index=True
    )
    author_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("team_members.id", ondelete="SET NULL"), nullable=True
    )
    author_name: Mapped[str] = mapped_column(String(100), default="Agent", nullable=False)
    activity_type: Mapped[str] = mapped_column(String(50), default="comment", nullable=False)
    message: Mapped[str] = mapped_column(Text, nullable=False)
    activity_metadata: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )

    ticket: Mapped["Ticket"] = relationship("Ticket", back_populates="activities")
