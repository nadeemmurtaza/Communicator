import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class AuditLog(Base):
    """
    Every AI decision (intent classification, draft generated, auto-send
    vs. escalate) and every human action, for compliance and debugging.
    Intentionally generic (entity_type/entity_id) rather than one table per
    entity, since the set of auditable entities will keep growing.
    """

    __tablename__ = "audit_logs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)

    entity_type: Mapped[str] = mapped_column(String(50), nullable=False)
    entity_id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), nullable=False, index=True)

    # e.g. "draft_generated", "auto_sent", "opted_out", "human_approved"
    action: Mapped[str] = mapped_column(String(100), nullable=False)
    # "system" | "ai" | a username
    actor: Mapped[str] = mapped_column(String(100), default="system", nullable=False)
    details: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
