import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class CampaignTemplate(Base):
    """
    A tenant's reusable WhatsApp campaign message template -- the body
    copy, variables, and Meta template metadata that Broadcast campaigns
    send. Mirrors DiscoveryFlowTemplate's pattern: scoped by tenant_key,
    never tenant content in code. NEWAX is the first tenant, seeded as
    data (see seed_campaign_templates.py). Each row maps 1:1 to a real
    Meta message template registered under that tenant's WABA.
    """

    __tablename__ = "campaign_templates"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    key: Mapped[str] = mapped_column(String(100), nullable=False)

    # Empty = shared/reusable across every channel the tenant has (a
    # template approved identically on WhatsApp, Instagram and Facebook,
    # for example). Set to a specific ChannelType value (e.g. "whatsapp")
    # when the message format or approval is channel-specific -- WhatsApp
    # business-initiated templates, for instance, are never reusable as-is
    # on another channel's API.
    channel: Mapped[str] = mapped_column(String(50), default="", nullable=False, index=True)

    name: Mapped[str] = mapped_column(String(255), nullable=False)
    purpose: Mapped[str] = mapped_column(String(500), default="", nullable=False)

    # Meta-side identity -- what the Broadcast dispatcher actually sends
    meta_template_name: Mapped[str] = mapped_column(String(255), nullable=False)
    meta_language: Mapped[str] = mapped_column(String(10), default="en_US", nullable=False)
    meta_category: Mapped[str] = mapped_column(String(20), default="MARKETING", nullable=False)
    meta_status: Mapped[str] = mapped_column(String(20), default="PENDING", nullable=False)

    body_preview: Mapped[str] = mapped_column(String(1024), default="", nullable=False)
    variables: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
