import enum
import uuid
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, Enum, ForeignKey, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base
from communicator.models.tag import contact_tags

if TYPE_CHECKING:
    from communicator.models.consent import ConsentRecord
    from communicator.models.contact_memory import ContactMemory
    from communicator.models.contact_note import ContactNote
    from communicator.models.conversation import Conversation
    from communicator.models.customer import Customer
    from communicator.models.lead import Lead
    from communicator.models.tag import Tag
    from communicator.models.ticket import Ticket


class ChannelType(str, enum.Enum):
    EMAIL = "email"
    SMS = "sms"
    WHATSAPP = "whatsapp"
    WEBSITE = "website"
    FACEBOOK = "facebook"
    INSTAGRAM = "instagram"
    TELEGRAM = "telegram"
    LINE = "line"
    VOICE = "voice"
    TIKTOK = "tiktok"


# Shared column type for every ChannelType column (contacts, messages,
# sequence_steps). The underlying Postgres enum ("channeltype", see
# migration 2c04e50c8093) stores lowercase labels matching `.value`, not
# SQLAlchemy's default of binding by member `.name` -- values_callable
# makes every comparison/insert actually match those labels.
ChannelTypeEnum = Enum(
    ChannelType, name="channeltype", values_callable=lambda obj: [e.value for e in obj]
)


class Contact(Base):
    """
    A single reachable channel for a person at a Lead -- e.g. one email
    address or one phone number. A Lead may have several.
    """

    __tablename__ = "contacts"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(String(100), default="newax", nullable=False, index=True)
    # The person this channel identity belongs to -- shared across every
    # channel they reach out on within this tenant. See models/customer.py.
    customer_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("customers.id"), nullable=True, index=True
    )
    lead_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("leads.id"), nullable=False, index=True)
    harvester_contact_point_id: Mapped[str] = mapped_column(
        String(64), default="", nullable=False, index=True
    )

    channel: Mapped[ChannelType] = mapped_column(ChannelTypeEnum, nullable=False)
    value: Mapped[str] = mapped_column(String(255), nullable=False, index=True)
    person_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    role: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    # BCP-47-ish short code (e.g. "en", "es"). Updated from whatever
    # language the reply engine detects in the lead's own messages; used to
    # reply in kind and to translate follow-up sequence templates.
    language: Mapped[str] = mapped_column(String(10), default="en", nullable=False)
    custom_fields: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )

    lead: Mapped["Lead"] = relationship("Lead", back_populates="contacts")
    customer: Mapped["Customer | None"] = relationship("Customer", back_populates="contacts")
    consent_records: Mapped[list["ConsentRecord"]] = relationship(
        "ConsentRecord", back_populates="contact", cascade="all, delete-orphan"
    )
    conversations: Mapped[list["Conversation"]] = relationship(
        "Conversation", back_populates="contact", cascade="all, delete-orphan"
    )
    tags: Mapped[list["Tag"]] = relationship("Tag", secondary=contact_tags)
    memories: Mapped[list["ContactMemory"]] = relationship(
        "ContactMemory",
        back_populates="contact",
        cascade="all, delete-orphan",
        order_by="ContactMemory.noted_at.desc()",
    )
    notes: Mapped[list["ContactNote"]] = relationship(
        "ContactNote",
        back_populates="contact",
        cascade="all, delete-orphan",
        order_by="ContactNote.created_at.desc()",
    )
    tickets: Mapped[list["Ticket"]] = relationship(
        "Ticket", back_populates="contact", cascade="all, delete-orphan"
    )
