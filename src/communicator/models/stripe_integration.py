import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base

SINGLETON_STRIPE_CONFIG_ID = uuid.UUID("00000000-0000-0000-0000-000000000002")


class StripeConfig(Base):
    """
    Stripe configuration and credentials for the workspace.
    Supports test mode and live mode Stripe API keys.
    """

    __tablename__ = "stripe_configs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    account_name: Mapped[str] = mapped_column(String(255), default="Primary Stripe Account", nullable=False)
    api_key: Mapped[str] = mapped_column(Text, default="", nullable=False)  # sk_test_... or sk_live_...
    publishable_key: Mapped[str] = mapped_column(String(255), default="", nullable=False)  # pk_test_... or pk_live_...
    webhook_secret: Mapped[str] = mapped_column(String(255), default="", nullable=False)  # whsec_...
    
    is_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    test_mode: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    default_currency: Mapped[str] = mapped_column(String(10), default="usd", nullable=False)

    last_tested_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    last_test_status: Mapped[str] = mapped_column(String(50), default="untested", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )


class StripeBillingActionLog(Base):
    """
    Audit log of customer billing operations executed by agents or automated workflows
    (refunds, customer portal session creation, manual subscription actions).
    """

    __tablename__ = "stripe_billing_action_logs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    contact_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("contacts.id", ondelete="SET NULL"),
        nullable=True,
        index=True,
    )
    customer_identifier: Mapped[str] = mapped_column(String(255), default="", nullable=False, index=True)
    action_type: Mapped[str] = mapped_column(String(50), nullable=False)  # "refund", "portal_session", "charge", etc.
    amount_cents: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    currency: Mapped[str] = mapped_column(String(10), default="usd", nullable=False)
    status: Mapped[str] = mapped_column(String(50), default="succeeded", nullable=False)
    stripe_id: Mapped[str] = mapped_column(String(255), default="", nullable=False)  # charge_id, refund_id, session_url
    actor_email: Mapped[str] = mapped_column(String(255), default="agent@newax.com", nullable=False)
    reason: Mapped[str] = mapped_column(Text, default="", nullable=False)
    details: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
