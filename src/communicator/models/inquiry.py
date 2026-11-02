import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, Enum, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base

if TYPE_CHECKING:
    from communicator.models.contact import Contact
    from communicator.models.conversation import Conversation
    from communicator.models.lead import Lead


class InquiryStatus(str, enum.Enum):
    DISCOVERY_IN_PROGRESS = "discovery_in_progress"
    DISCOVERY_COMPLETE = "discovery_complete"
    AWAITING_REVIEW = "awaiting_review"
    QUALIFIED = "qualified"
    MEETING_PENDING = "meeting_pending"
    MEETING_BOOKED = "meeting_booked"
    PROPOSAL_PENDING = "proposal_pending"
    WON = "won"
    LOST = "lost"
    NURTURE = "nurture"


class Inquiry(Base):
    """
    A structured business inquiry captured through the WhatsApp discovery
    flow -- NEWAX's "what does this visitor actually need" record, kept
    separate from the raw message transcript (Conversation/Message) and
    from support tickets (Ticket), per the NEWAX WhatsApp Automation spec.
    """

    __tablename__ = "inquiries"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", nullable=False, index=True
    )
    conversation_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("conversations.id"), nullable=False, unique=True, index=True
    )
    contact_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("contacts.id"), nullable=False)
    lead_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("leads.id"), nullable=True)

    # -- Organization & respondent --
    organization_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    organization_status: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    industry: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    contact_role: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    org_size_range: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    locations_count: Mapped[str] = mapped_column(String(50), default="", nullable=False)

    # -- The business problem --
    problem_description: Mapped[str] = mapped_column(Text, default="", nullable=False)
    problem_categories: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    reported_impact: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    frequency: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    impact_level: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    current_workaround: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    limitations: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    desired_outcome: Mapped[str] = mapped_column(String(100), default="", nullable=False)

    # -- Systems & integration --
    existing_systems: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    integration_required: Mapped[str] = mapped_column(String(50), default="", nullable=False)

    # -- Capability mapping --
    capability_mapping: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    # -- Readiness --
    timeline: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    decision_process: Mapped[str] = mapped_column(String(100), default="", nullable=False)
    budget_range: Mapped[str] = mapped_column(String(50), default="", nullable=False)

    # -- Qualification --
    qualification_score: Mapped[int | None] = mapped_column(Integer, nullable=True)
    qualification_factors: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    # -- Routing --
    status: Mapped[InquiryStatus] = mapped_column(
        Enum(InquiryStatus), default=InquiryStatus.DISCOVERY_IN_PROGRESS, nullable=False
    )
    assigned_owner: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    next_action: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )

    conversation: Mapped["Conversation"] = relationship("Conversation")
    contact: Mapped["Contact"] = relationship("Contact")
    lead: Mapped["Lead | None"] = relationship("Lead")
