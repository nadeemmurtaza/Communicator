"""Ecosystem Outbound Email Dispatch Service for Communicator

Complies with ECOSYSTEM.md:
"newcom/esp (nmail) owns all outbound email: sending, deliverability, DKIM, journeys, A/B testing...
Communicator: broadcasts (campaign strategy/segmentation)".

Whenever Communicator needs to send an email (broadcast, transactional, or customer reply),
it dispatches the job through the Zero-Trust Crypto Mesh to newcom/esp.
"""

from __future__ import annotations

import logging
from typing import Any

from communicator.services.ecosystem_client import ecosystem_client

logger = logging.getLogger(__name__)


async def dispatch_email_via_newcom(
    to_email: str,
    subject: str,
    html_body: str = "",
    text_body: str = "",
    action: str = "send_transactional_email",
    template_id: str | None = None,
    template_data: dict[str, Any] | None = None,
    order_id: str | None = None,
    customer_id: str | None = None,
    tenant_id: str | None = None,
    organization_id: str | None = None,
) -> dict[str, Any]:
    """
    Encrypts and dispatches outbound email instruction to newcom/esp via NeuraCore Relay.
    """
    payload = {
        "action": action,
        "to": to_email,
        "subject": subject,
        "htmlBody": html_body,
        "textBody": text_body or html_body,
        "templateId": template_id,
        "templateData": template_data or {},
        "orderId": order_id,
        "customerId": customer_id,
    }

    logger.info("Dispatching email '%s' for '%s' to newcom/esp via CryptoMesh", subject, to_email)

    return await ecosystem_client.publish_event(
        event_name="campaign.email.dispatch",
        payload=payload,
        recipient_id="newcom-esp",
        tenant_id=tenant_id,
        organization_id=organization_id,
    )
