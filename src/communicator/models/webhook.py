import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base


class WebhookEvent(str, enum.Enum):
    CONVERSATION_CREATED = "conversation.created"
    CONVERSATION_STATUS_CHANGED = "conversation.status_changed"
    CONVERSATION_ASSIGNED = "conversation.assigned"
    MESSAGE_CREATED = "message.created"
    TICKET_CREATED = "ticket.created"
    TICKET_UPDATED = "ticket.updated"
    TICKET_RESOLVED = "ticket.resolved"
    CSAT_SUBMITTED = "csat.submitted"
    CONTACT_CREATED = "contact.created"
    CONTACT_UPDATED = "contact.updated"


class WebhookEndpoint(Base):
    """
    Outgoing webhook subscriber endpoint.
    Notifies external services in real time when events occur in the system.
    """

    __tablename__ = "webhook_endpoints"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    url: Mapped[str] = mapped_column(String(1024), nullable=False)
    secret: Mapped[str] = mapped_column(String(255), nullable=False)  # HMAC-SHA256 signing secret

    # List of subscribed events e.g. ["conversation.created", "message.created", "ticket.created"]
    events: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)

    # Optional custom HTTP headers sent with the webhook e.g. {"Authorization": "Bearer ..."}
    headers: Mapped[dict[str, str]] = mapped_column(JSONB, default=dict, nullable=False)

    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    retry_count: Mapped[int] = mapped_column(Integer, default=3, nullable=False)
    timeout_seconds: Mapped[int] = mapped_column(Integer, default=10, nullable=False)

    # Operational telemetry
    total_delivered: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    total_failed: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    last_status_code: Mapped[int | None] = mapped_column(Integer, nullable=True)
    last_delivered_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )

    deliveries: Mapped[list["WebhookDeliveryLog"]] = relationship(
        "WebhookDeliveryLog", back_populates="endpoint", cascade="all, delete-orphan"
    )


class WebhookDeliveryLog(Base):
    """
    Historical log of every webhook delivery attempt with payload, HTTP status code, latency, and response.
    """

    __tablename__ = "webhook_delivery_logs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    endpoint_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("webhook_endpoints.id", ondelete="CASCADE"), nullable=False, index=True
    )
    event: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    payload: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    status_code: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    response_headers: Mapped[dict[str, str]] = mapped_column(JSONB, default=dict, nullable=False)
    response_body: Mapped[str] = mapped_column(Text, default="", nullable=False)
    duration_ms: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    attempt: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    success: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    error_message: Mapped[str | None] = mapped_column(Text, nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )

    endpoint: Mapped["WebhookEndpoint"] = relationship("WebhookEndpoint", back_populates="deliveries")
