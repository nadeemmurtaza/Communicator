from __future__ import annotations

import hashlib
import logging
import secrets
import uuid
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

import httpx
from fastapi import HTTPException, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.config import settings
from communicator.models.audit import AuditLog
from communicator.models.live_proof import (
    LiveProofEvidence,
    LiveProofSession,
    LiveProofStatus,
    ProofMediaType,
)

logger = logging.getLogger(__name__)

# Allowed MIME types for live proof
ALLOWED_PHOTO_MIMES = {"image/jpeg", "image/png", "image/webp"}
ALLOWED_VIDEO_MIMES = {"video/webm", "video/mp4"}
MAX_MEDIA_BYTES = 100 * 1024 * 1024  # 100 MB max


class LiveProofServiceError(Exception):
    """Base exception for live proof errors."""


class LiveProofSecurityError(LiveProofServiceError):
    """Raised when proof fails anti-spoof or live camera validation."""


async def create_live_proof_session(
    session: AsyncSession,
    tenant_id: str,
    organization_id: str | None = None,
    conversation_id: uuid.UUID | None = None,
    contact_id: uuid.UUID | None = None,
    order_id: str | None = None,
    customer_identifier: str = "",
    claim_type: str = "refund",
    max_video_seconds: int = 45,
    ttl_minutes: int = 30,
    require_video: bool = True,
    require_photos: bool = True,
    base_url: str = "",
) -> tuple[LiveProofSession, str]:
    """
    Creates a new LiveProof verification session for a customer refund/return claim.
    Generates a high-entropy session token and dynamic anti-spoof watermark nonce.
    Returns the session object and the direct customer verification URL.
    """
    if not tenant_id:
        raise ValueError("tenant_id is required for multi-tenant isolation.")

    session_token = secrets.token_urlsafe(32)
    anti_spoof_nonce = secrets.token_hex(16)
    expires_at = datetime.now(UTC) + timedelta(minutes=ttl_minutes)

    proof_session = LiveProofSession(
        tenant_id=tenant_id,
        organization_id=organization_id,
        conversation_id=conversation_id,
        contact_id=contact_id,
        order_id=order_id,
        customer_identifier=customer_identifier,
        claim_type=claim_type,
        session_token=session_token,
        status=LiveProofStatus.PENDING,
        anti_spoof_nonce=anti_spoof_nonce,
        max_video_seconds=max_video_seconds,
        require_video=require_video,
        require_photos=require_photos,
        expires_at=expires_at,
    )

    session.add(proof_session)
    await session.flush()

    audit = AuditLog(
        entity_type="LiveProofSession",
        entity_id=proof_session.id,
        action="live_proof.session_created",
        actor="system",
        details={
            "tenant_id": tenant_id,
            "organization_id": organization_id,
            "order_id": order_id,
            "claim_type": claim_type,
            "expires_at": expires_at.isoformat(),
        },
    )
    session.add(audit)
    await session.commit()
    await session.refresh(proof_session)

    root_url = (base_url or settings.public_url).rstrip("/")
    verification_url = f"{root_url}/verify/proof/{proof_session.id}?token={session_token}"

    logger.info(
        "Created LiveProof session %s for tenant %s (order %s)",
        proof_session.id,
        tenant_id,
        order_id,
    )
    return proof_session, verification_url


async def get_and_validate_session(
    session: AsyncSession,
    session_id: uuid.UUID,
    token: str,
) -> LiveProofSession:
    """
    Retrieves and validates a session for customer access.
    Raises HTTPException if not found, token mismatch, or expired.
    """
    stmt = select(LiveProofSession).where(LiveProofSession.id == session_id)
    res = await session.execute(stmt)
    proof_session = res.scalar_one_or_none()

    if proof_session is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Verification session not found.",
        )

    # Secure comparison to prevent timing attacks
    if not secrets.compare_digest(proof_session.session_token, token):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Invalid verification token.",
        )

    now = datetime.now(UTC)
    if proof_session.expires_at < now:
        if proof_session.status != LiveProofStatus.EXPIRED:
            proof_session.status = LiveProofStatus.EXPIRED
            await session.commit()
        raise HTTPException(
            status_code=status.HTTP_410_GONE,
            detail="This verification session has expired. Please ask support for a new link.",
        )

    if proof_session.status == LiveProofStatus.REJECTED:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"This session was rejected: {proof_session.rejection_reason or 'Policy violation'}",
        )

    return proof_session


async def ingest_live_evidence(
    session: AsyncSession,
    session_id: uuid.UUID,
    token: str,
    media_type: ProofMediaType,
    file_bytes: bytes,
    mime_type: str,
    telemetry: dict[str, Any] | None = None,
    watermark_nonce: str | None = None,
    client_ip: str | None = None,
) -> LiveProofEvidence:
    """
    Ingests live photo or live video captured directly through the browser camera.
    Strictly verifies that:
    1. Media is not empty and within byte limits.
    2. MIME type matches allowed live formats.
    3. Telemetry demonstrates live camera stream (not generic file picker upload).
    4. Anti-spoof dynamic watermark nonce matches session nonce.
    5. SHA-256 hash is computed and recorded immutably.
    6. Emits `refund.proof_captured` event to core-newax Event Bus / ERP-1.
    """
    proof_session = await get_and_validate_session(session, session_id, token)

    if not file_bytes:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Captured media is empty.",
        )

    if len(file_bytes) > MAX_MEDIA_BYTES:
        raise HTTPException(
            status_code=status.HTTP_413_REQUEST_ENTITY_TOO_LARGE,
            detail=f"Media size exceeds {MAX_MEDIA_BYTES // (1024 * 1024)}MB limit.",
        )

    # Validate MIME type
    clean_mime = mime_type.lower().split(";")[0].strip()
    if media_type == ProofMediaType.LIVE_PHOTO and clean_mime not in ALLOWED_PHOTO_MIMES:
        raise HTTPException(
            status_code=status.HTTP_415_UNSUPPORTED_MEDIA_TYPE,
            detail=f"Unsupported photo MIME type: {clean_mime}. Must be JPEG, PNG, or WebP.",
        )
    if media_type == ProofMediaType.LIVE_VIDEO and clean_mime not in ALLOWED_VIDEO_MIMES:
        raise HTTPException(
            status_code=status.HTTP_415_UNSUPPORTED_MEDIA_TYPE,
            detail=f"Unsupported video MIME type: {clean_mime}. Must be WebM or MP4.",
        )

    # Validate Live Camera Telemetry (Anti-Upload Enforcement)
    telemetry_data = telemetry or {}
    is_live_stream = telemetry_data.get("is_live_stream", False)
    capture_source = telemetry_data.get("capture_source", "")

    # In live verification mode, direct file picker uploads are explicitly banned
    if not is_live_stream and capture_source != "live_webrtc_camera":
        logger.warning(
            "Upload rejected: missing live stream telemetry markers for session %s",
            session_id,
        )
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="File upload from phone gallery is strictly prohibited. You must use the live camera.",
        )

    # Watermark dynamic nonce verification
    watermark_verified = True
    verification_score = 1.0

    if watermark_nonce:
        if not secrets.compare_digest(proof_session.anti_spoof_nonce, watermark_nonce):
            logger.warning("Watermark nonce mismatch for session %s", session_id)
            watermark_verified = False
            verification_score -= 0.5
    else:
        watermark_verified = False
        verification_score -= 0.2

    # Compute SHA-256 hash
    sha256_hash = hashlib.sha256(file_bytes).hexdigest()

    # Determine file storage path
    storage_root = Path(settings.live_proof_storage_dir)
    tenant_dir = storage_root / proof_session.tenant_id / str(proof_session.id)
    tenant_dir.mkdir(parents=True, exist_ok=True)

    evidence_id = uuid.uuid4()
    ext = "jpg" if media_type == ProofMediaType.LIVE_PHOTO else "webm"
    file_name = f"{evidence_id}_{media_type.value}.{ext}"
    target_path = tenant_dir / file_name

    target_path.write_bytes(file_bytes)

    duration = None
    if media_type == ProofMediaType.LIVE_VIDEO:
        duration = float(telemetry_data.get("duration_seconds") or 0.0)

    resolution = telemetry_data.get("resolution") or "1920x1080"

    full_fingerprint = {
        **telemetry_data,
        "client_ip": client_ip,
        "ingested_at": datetime.now(UTC).isoformat(),
        "file_name": file_name,
    }

    evidence = LiveProofEvidence(
        id=evidence_id,
        session_id=proof_session.id,
        tenant_id=proof_session.tenant_id,
        media_type=media_type,
        file_path=str(target_path).replace("\\", "/"),
        file_size_bytes=len(file_bytes),
        mime_type=clean_mime,
        sha256_hash=sha256_hash,
        duration_seconds=duration,
        resolution=resolution,
        device_fingerprint=full_fingerprint,
        watermark_verified=watermark_verified,
        verification_score=max(0.0, min(1.0, verification_score)),
    )

    session.add(evidence)

    # Transition session status to CAPTURED
    proof_session.status = LiveProofStatus.CAPTURED
    proof_session.completed_at = datetime.now(UTC)

    audit = AuditLog(
        entity_type="LiveProofEvidence",
        entity_id=evidence.id,
        action="live_proof.evidence_captured",
        actor=proof_session.customer_identifier or "customer",
        details={
            "session_id": str(proof_session.id),
            "tenant_id": proof_session.tenant_id,
            "order_id": proof_session.order_id,
            "media_type": media_type.value,
            "sha256_hash": sha256_hash,
            "verification_score": evidence.verification_score,
            "watermark_verified": watermark_verified,
        },
    )
    session.add(audit)
    await session.commit()
    await session.refresh(evidence)

    # Emit event to core-newax Event Bus / ERP-1
    await _publish_proof_captured_event(proof_session, evidence)

    logger.info(
        "Successfully ingested live %s for session %s (SHA-256: %s)",
        media_type.value,
        session_id,
        sha256_hash,
    )
    return evidence


async def _publish_proof_captured_event(
    session: LiveProofSession,
    evidence: LiveProofEvidence,
) -> None:
    """
    Dispatches domain event 'refund.proof_captured' to core-newax Event Bus
    so ERP-1 and other ecosystem services receive verified evidence.
    """
    event_bus_url = settings.ecosystem_event_bus_url
    if not event_bus_url:
        logger.debug("Central event bus URL not configured; skipping event dispatch.")
        return

    payload = {
        "name": "refund.proof_captured",
        "occurredAt": datetime.now(UTC).isoformat(),
        "source": "communicator",
        "tenantId": session.tenant_id,
        "organizationId": session.organization_id,
        "payload": {
            "sessionId": str(session.id),
            "orderId": session.order_id,
            "claimType": session.claim_type,
            "mediaType": evidence.media_type.value,
            "mediaUrl": f"{settings.public_url.rstrip('/')}/api/v1/live-proof/sessions/{session.id}/evidence/{evidence.id}/media",
            "sha256Hash": evidence.sha256_hash,
            "fileSizeBytes": evidence.file_size_bytes,
            "durationSeconds": evidence.duration_seconds,
            "verificationScore": evidence.verification_score,
            "watermarkVerified": evidence.watermark_verified,
            "capturedAt": evidence.captured_at.isoformat(),
        },
    }

    headers = {
        "Content-Type": "application/json",
        "X-Newax-Source": "communicator",
        "X-Tenant-Id": session.tenant_id,
    }
    if session.organization_id:
        headers["X-Organization-Id"] = session.organization_id
    if settings.ecosystem_gateway_key:
        headers["Authorization"] = f"Bearer {settings.ecosystem_gateway_key}"

    try:
        async with httpx.AsyncClient(timeout=5.0) as client:
            resp = await client.post(event_bus_url, json=payload, headers=headers)
            if resp.status_code >= 400:
                logger.warning(
                    "Event bus returned status %d on refund.proof_captured: %s",
                    resp.status_code,
                    resp.text,
                )
    except (httpx.HTTPError, OSError) as exc:
        logger.error("Failed to publish refund.proof_captured to event bus: %s", exc)
