"""
Validates Meta's X-Hub-Signature-256 header: HMAC-SHA256 over the raw
request body, keyed with the app secret. See Meta's webhook security docs.
Without this, anyone who finds the webhook URL could forge inbound
WhatsApp messages (fake STOP, impersonated contacts, bogus Harvester
outcomes) the same way an unverified Twilio request could.
"""

from __future__ import annotations

import hashlib
import hmac


def validate_meta_signature(app_secret: str, body: bytes, signature_header: str | None) -> bool:
    if not signature_header or not signature_header.startswith("sha256="):
        return False
    expected = hmac.new(app_secret.encode("utf-8"), body, hashlib.sha256).hexdigest()
    return hmac.compare_digest(expected, signature_header.removeprefix("sha256="))
