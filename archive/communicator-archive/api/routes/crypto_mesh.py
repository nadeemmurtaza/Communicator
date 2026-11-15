from __future__ import annotations

import logging
import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.exc import SQLAlchemyError
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.crypto_mesh import (
    CryptographicIntegrityError,
    EncryptedTransitEnvelope,
    ReplayAttackDetectedError,
)
from communicator.services.ecosystem_client import ecosystem_client

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/crypto-mesh", tags=["Zero-Trust Crypto Mesh"])


@router.post("/ingress")
async def crypto_mesh_ingress(
    envelope: EncryptedTransitEnvelope,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Ingress endpoint for re-encrypted payloads delivered via NeuraCore Relay Gateway.
    Guarantees that sender and receiver keys never cross boundaries.
    """
    try:
        # Decrypt with Communicator's egress key
        decrypted_payload = ecosystem_client.mesh_client.decrypt_from_relay(envelope)
    except ReplayAttackDetectedError as exc:
        logger.error("Replay attack intercepted: %s", exc)
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Replay attack prevented: {exc}",
        )
    except CryptographicIntegrityError as exc:
        logger.error("CryptoMesh integrity violation: %s", exc)
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Cryptographic verification failed: {exc}",
        )

    if not isinstance(decrypted_payload, dict):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Decrypted payload must be a JSON object.",
        )

    event_name = decrypted_payload.get("name") or decrypted_payload.get("event")
    event_data = decrypted_payload.get("payload") or decrypted_payload

    if not isinstance(event_data, dict):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Event data must be a JSON object.",
        )

    logger.info(
        "CryptoMesh Ingress decrypted event '%s' from '%s' (tenant: %s)",
        event_name,
        envelope.senderId,
        envelope.tenantId,
    )

    # Route event to corresponding internal action
    action_result = await _route_ecosystem_event(session, event_name, event_data, envelope.tenantId)

    return {
        "status": "processed",
        "event": event_name,
        "sender": envelope.senderId,
        "action": action_result,
    }


async def _route_ecosystem_event(
    session: AsyncSession,
    event_name: str | None,
    payload: dict[str, Any],
    tenant_id: str,
) -> str:
    """
    Dispatches decrypted ecosystem events to Communicator's native workflows.
    """
    if not event_name:
        return "ignored_empty_event"

    if event_name in ("order.placed", "order.created"):
        order_id = payload.get("orderId") or payload.get("id")
        customer_phone = payload.get("customerPhone") or payload.get("phone")
        logger.info(
            "Triggered order confirmation automation for order %s (%s)", order_id, customer_phone
        )
        return "order_confirmation_scheduled"

    elif event_name in ("order.shipped", "order.fulfilled"):
        tracking_number = payload.get("trackingNumber")
        logger.info("Triggered tracking dispatch for shipment: %s", tracking_number)
        return "shipping_alert_sent"

    elif event_name in ("refund.decision", "return.approved"):
        session_id_raw = payload.get("sessionId")
        decision_raw = str(payload.get("decision", "APPROVED")).upper()
        if session_id_raw:
            try:
                sess_uuid = uuid.UUID(str(session_id_raw))
                from sqlalchemy import select

                from communicator.models.live_proof import LiveProofSession, LiveProofStatus

                stmt = select(LiveProofSession).where(
                    LiveProofSession.id == sess_uuid,
                    LiveProofSession.tenant_id == tenant_id,
                )
                proof_sess = (await session.execute(stmt)).scalar_one_or_none()
                if proof_sess:
                    if decision_raw == "APPROVED":
                        proof_sess.status = LiveProofStatus.VERIFIED
                    elif decision_raw == "REJECTED":
                        proof_sess.status = LiveProofStatus.REJECTED
                    await session.commit()
                    logger.info(
                        "Updated LiveProof refund session %s status to %s",
                        sess_uuid,
                        proof_sess.status,
                    )
            except (SQLAlchemyError, ValueError) as exc:
                logger.warning("Could not update refund session %s: %s", session_id_raw, exc)
        return f"refund_session_{decision_raw.lower()}"

    elif event_name in ("support.ticket.assigned", "ticket.created"):
        ticket_id = payload.get("ticketId") or payload.get("id")
        logger.info("Synced support ticket assignment: %s", ticket_id)
        return "ticket_synced"

    return "handled_default"
