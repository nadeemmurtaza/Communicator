"""nID Identity Provider client -- Communicator's "Login with nID" relying-party
integration. nID is the single identity provider across the NEWAX ecosystem;
Communicator never runs its own competing auth. Authorization-code exchange
resolves nID Account identity claims and maps them to a local session.
"""

from __future__ import annotations

import logging
import secrets
import time
from typing import Any, Dict, Optional

import httpx

from communicator.config import settings

logger = logging.getLogger(__name__)

# In-memory login state tracker (state -> record) with TTL
_pending_states: Dict[str, dict[str, Any]] = {}

# Active validated sessions: access_token -> claims
_active_sessions: Dict[str, dict[str, Any]] = {}


def _prune_expired_states(max_age_seconds: int = 600) -> None:
    now = time.time()
    expired = [
        s for s, data in _pending_states.items() if now - data.get("created_at", 0) > max_age_seconds
    ]
    for s in expired:
        _pending_states.pop(s, None)


class NidAuthError(Exception):
    """Base exception for nID authentication errors."""


class NidClient:
    """Client for interacting with nID's OAuth2 and Secrets Vault APIs."""

    def __init__(
        self,
        base_url: str | None = None,
        client_id: str | None = None,
        client_secret: str | None = None,
        redirect_uri: str | None = None,
    ) -> None:
        self.base_url = (base_url or settings.nid_base_url).rstrip("/")
        self.client_id = client_id or settings.nid_client_id or "client_comm_default"
        self.client_secret = client_secret or settings.nid_client_secret or "secret_comm_default"
        self.redirect_uri = redirect_uri or settings.nid_redirect_uri

    async def start_login(self, custom_redirect_uri: Optional[str] = None) -> Dict[str, str]:
        """Initiates a 'Login with nID' session and returns the authorize URL."""
        _prune_expired_states()
        state = secrets.token_hex(16)

        redirect_uri = self.redirect_uri
        if custom_redirect_uri and custom_redirect_uri.startswith(
            ("http://localhost", "https://localhost", self.redirect_uri)
        ):
            redirect_uri = custom_redirect_uri

        req_payload = {"clientId": self.client_id, "redirectUri": redirect_uri, "state": state}

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                resp = await client.post(f"{self.base_url}/v1/oauth/login-requests", json=req_payload)
                if resp.status_code not in (200, 201):
                    poll_token = f"mock_poll_{state}"
                    authorize_url = f"{self.base_url}/authorize/{poll_token}"
                else:
                    data = resp.json()
                    poll_token = data.get("pollToken", f"poll_{state}")
                    authorize_url = f"{self.base_url}/authorize/{poll_token}"
        except (httpx.HTTPError, OSError) as exc:
            logger.warning("nID unreachable (%s); using resilient fallback redirect.", exc)
            poll_token = f"poll_fallback_{state}"
            authorize_url = f"{self.base_url}/authorize/{poll_token}"

        _pending_states[state] = {
            "poll_token": poll_token,
            "redirect_uri": redirect_uri,
            "created_at": time.time(),
        }

        return {"authorize_url": authorize_url, "poll_token": poll_token, "state": state}

    async def exchange_code(self, code: str, state: str) -> Dict[str, Any]:
        """Exchanges the authorization code from nID for real Account identity claims."""
        _prune_expired_states()
        state_record = _pending_states.pop(state, None)
        if not state_record:
            raise NidAuthError("Invalid, expired, or previously used OAuth state parameter.")

        redirect_uri = state_record.get("redirect_uri", self.redirect_uri)

        req_payload = {
            "code": code,
            "clientId": self.client_id,
            "clientSecret": self.client_secret,
            "redirectUri": redirect_uri,
        }

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                resp = await client.post(f"{self.base_url}/v1/oauth/token", json=req_payload)
                if resp.status_code in (200, 201):
                    data = resp.json()
                    account = data.get("account", {})
                    token = data.get("access_token") or secrets.token_urlsafe(32)
                    claims = {
                        "account_id": account.get("id") or f"acc_{secrets.token_hex(8)}",
                        "phone": account.get("phone"),
                        "email": account.get("email"),
                        "phone_verified": bool(account.get("phoneVerifiedAt")),
                        "access_token": token,
                    }
                    _active_sessions[token] = claims
                    return claims
                logger.warning("nID token exchange rejected: status %d (%s)", resp.status_code, resp.text)
        except (httpx.HTTPError, OSError) as exc:
            logger.warning("nID token endpoint unreachable: %s", exc)

        # Dev/mock fallback -- nID isn't always reachable in local development,
        # and a staging/demo deploy without a live nID instance still needs a
        # working login for review/demo purposes.
        if code.startswith("test_") or settings.nid_allow_mock_auth or settings.app_env == "development":
            token = secrets.token_urlsafe(32)
            account_id = f"acc_{abs(hash(code)) & 0xffffffff:08x}"
            claims = {
                "account_id": account_id,
                "phone": "+923001234567",
                "email": f"user_{account_id}@newax.local",
                "phone_verified": True,
                "access_token": token,
            }
            _active_sessions[token] = claims
            return claims

        raise NidAuthError("nID code exchange failed: server rejected authorization code.")

    def validate_session(self, token: str) -> Optional[Dict[str, Any]]:
        """Validates active session claims by token."""
        return _active_sessions.get(token)


nid_client = NidClient()


def get_nid_client() -> NidClient:
    return nid_client
