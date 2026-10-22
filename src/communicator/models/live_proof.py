import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, Enum, Float, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from communicator.database import Base


class LiveProofStatus(str, enum.Enum):
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    CAPTURED = "captured"
    VERIFIED = "verified"
    REJECTED = "rejected"
    EXPIRED = "expired"


class ProofMediaType(str, enum.Enum):
    LIVE_PHOTO = "live_photo"
    LIVE_VIDEO = "live_video"


class LiveProofSession(Base):
    """
    A secure live-capture verification session initiated for return/refund fraud prevention.
    Strictly requires in-browser live camera capture (photos and videos).
    Gallery/file upload pickers are completely prohibited.
    Multi-tenant enterprise SaaS scoped.
    """

    __tablename__ = "live_proof_sessions"

    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid.uuid4
    )
    tenant_id: Mapped[str] = mapped_column(String(64), nullable=False, index=True)
    organization_id: Mapped[str | None] = mapped_column(String(64), nullable=True, index=True)

    conversation_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("conversations.id"), nullable=True, index=True
    )
    contact_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("contacts.id"), nullable=True, index=True
    )

    order_id: Mapped[str | None] = mapped_column(String(128), nullable=True, index=True)
    customer_identifier: Mapped[str] = mapped_column(String(255), nullable=False, default="")
    claim_type: Mapped[str] = mapped_column(String(64), nullable=False, default="refund")

    session_token: Mapped[str] = mapped_column(
        String(128), nullable=False, unique=True, index=True
    )
    status: Mapped[LiveProofStatus] = mapped_column(
        Enum(LiveProofStatus), default=LiveProofStatus.PENDING, nullable=False, index=True
    )

    anti_spoof_nonce: Mapped[str] = mapped_column(String(64), nullable=False)
    max_video_seconds: Mapped[int] = mapped_column(Integer, default=45, nullable=False)
    require_video: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    require_photos: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    rejection_reason: Mapped[str | None] = mapped_column(Text, nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )

    evidences: Mapped[list["LiveProofEvidence"]] = relationship(
        "LiveProofEvidence",
        back_populates="session",
        cascade="all, delete-orphan",
        order_by="LiveProofEvidence.captured_at",
    )


class LiveProofEvidence(Base):
    """
    An immutable piece of live-captured evidence (live photo or live video stream).
    Stored with SHA-256 cryptographic digest, device telemetry, and anti-spoof watermark proofs.
    """

    __tablename__ = "live_proof_evidences"

    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid.uuid4
    )
    session_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("live_proof_sessions.id"), nullable=False, index=True
    )
    tenant_id: Mapped[str] = mapped_column(String(64), nullable=False, index=True)

    media_type: Mapped[ProofMediaType] = mapped_column(Enum(ProofMediaType), nullable=False)
    file_path: Mapped[str] = mapped_column(String(512), nullable=False)
    file_size_bytes: Mapped[int] = mapped_column(Integer, nullable=False)
    mime_type: Mapped[str] = mapped_column(String(64), nullable=False)
    sha256_hash: Mapped[str] = mapped_column(String(64), nullable=False, index=True)

    duration_seconds: Mapped[float | None] = mapped_column(Float, nullable=True)
    resolution: Mapped[str | None] = mapped_column(String(32), nullable=True)

    # Telemetry captured from live WebRTC media stream constraints & device sensors
    # e.g., framerate, facingMode, canvas grab source, client timestamp
    device_fingerprint: Mapped[dict[str, Any] | None] = mapped_column(
        JSONB, nullable=True, default=dict
    )

    watermark_verified: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    verification_score: Mapped[float] = mapped_column(Float, default=1.0, nullable=False)

    captured_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )

    session: Mapped["LiveProofSession"] = relationship("LiveProofSession", back_populates="evidences")
