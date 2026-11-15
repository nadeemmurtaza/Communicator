"""
Reimplements Twilio's request-validation algorithm without depending on
the twilio SDK: HMAC-SHA1 over the full request URL followed by each POST
param's key+value (sorted by key), base64-encoded, compared to the
X-Twilio-Signature header. See Twilio's "Validating requests" docs.

This exists so inbound SMS/WhatsApp webhooks can't be spoofed -- without
it, anyone who guesses the endpoint URL could forge an inbound message
(e.g. a fake STOP to silently opt a lead out, or once Phase 3's reply
engine exists, injected content aimed at it).
"""

from __future__ import annotations

import base64
import hashlib
import hmac


def compute_twilio_signature(auth_token: str, url: str, params: dict[str, str]) -> str:
    data = url
    for key in sorted(params.keys()):
        data += key + params[key]

    digest = hmac.new(auth_token.encode("utf-8"), data.encode("utf-8"), hashlib.sha1).digest()
    return base64.b64encode(digest).decode("utf-8")


def validate_twilio_signature(
    auth_token: str, url: str, params: dict[str, str], signature: str
) -> bool:
    expected = compute_twilio_signature(auth_token, url, params)
    return hmac.compare_digest(expected, signature)
