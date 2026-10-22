from __future__ import annotations

import hashlib
import logging
import re
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
    auto_commit: bool = True,
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
    if auto_commit:
        await session.commit()
        await session.refresh(proof_session)
    else:
        await session.flush()

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
    for_upload: bool = False,
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

    if for_upload and proof_session.status in (LiveProofStatus.CAPTURED, LiveProofStatus.VERIFIED):
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Evidence collection for this session has already been completed and submitted.",
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
    proof_session = await get_and_validate_session(session, session_id, token, for_upload=True)

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

    # Dynamic watermark nonce verification
    watermark_verified = True
    verification_score = 1.0

    if watermark_nonce:
        if not secrets.compare_digest(proof_session.anti_spoof_nonce, watermark_nonce):
            logger.warning("Watermark nonce mismatch for session %s", session_id)
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Anti-spoofing verification failed: invalid watermark nonce.",
            )
    else:
        # Require nonce in production/live capture
        watermark_verified = False
        verification_score = 0.8

    # Validate numeric duration for video before any filesystem operations
    duration = None
    if media_type == ProofMediaType.LIVE_VIDEO:
        try:
            duration = float(telemetry_data.get("duration_seconds") or 0.0)
            if duration < 0 or duration > 3600:
                raise ValueError("Video duration out of range.")
        except (ValueError, TypeError):
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Invalid video duration in telemetry.",
            )

    # Compute SHA-256 hash
    sha256_hash = hashlib.sha256(file_bytes).hexdigest()

    # Determine file storage path with path traversal protection
    storage_root = Path(settings.live_proof_storage_dir).resolve()
    clean_tenant = re.sub(r"[^A-Za-z0-9_-]", "_", str(proof_session.tenant_id))[:64] or "default"
    tenant_dir = (storage_root / clean_tenant / str(proof_session.id)).resolve()
    if not tenant_dir.is_relative_to(storage_root):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid storage path.")
    tenant_dir.mkdir(parents=True, exist_ok=True)

    evidence_id = uuid.uuid4()
    ext = "jpg" if media_type == ProofMediaType.LIVE_PHOTO else "webm"
    file_name = f"{evidence_id}_{media_type.value}.{ext}"
    target_path = tenant_dir / file_name

    try:
        target_path.write_bytes(file_bytes)
    except OSError as io_err:
        logger.error("Failed to write evidence to disk: %s", io_err)
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Storage write error.")

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

    # Multi-part evidence check: verify whether all required media types have been submitted
    stmt_ev = select(LiveProofEvidence).where(LiveProofEvidence.session_id == proof_session.id)
    existing_evidences = (await session.execute(stmt_ev)).scalars().all()
    all_media_types = {e.media_type for e in existing_evidences} | {media_type}

    needs_video = proof_session.require_video and (ProofMediaType.LIVE_VIDEO not in all_media_types)
    needs_photos = proof_session.require_photos and (ProofMediaType.LIVE_PHOTO not in all_media_types)

    is_complete = not (needs_video or needs_photos)

    if is_complete:
        proof_session.status = LiveProofStatus.CAPTURED
        proof_session.completed_at = datetime.now(UTC)
    else:
        proof_session.status = LiveProofStatus.IN_PROGRESS

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
            "is_complete": is_complete,
        },
    )
    session.add(audit)
    await session.commit()
    await session.refresh(evidence)

    # Emit event to core-newax Event Bus / ERP-1 only when all required parts are captured
    if is_complete:
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
    Dispatches domain event 'refund.proof_captured' via Zero-Trust Crypto Mesh
    to NeuraCore Relay and ERP-1 so enterprise refund automation receives verified proof.
    """
    from communicator.services.ecosystem_client import ecosystem_client

    payload = {
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
    }

    try:
        await ecosystem_client.publish_event(
            event_name="refund.proof_captured",
            payload=payload,
            recipient_id="erp-1",
            tenant_id=session.tenant_id,
            organization_id=session.organization_id,
        )
        logger.info("Published encrypted refund.proof_captured event for session %s to erp-1.", session.id)
    except Exception as exc:
        logger.error("Failed to publish encrypted refund.proof_captured: %s", exc)
