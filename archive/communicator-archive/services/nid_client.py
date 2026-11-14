"""nID Identity Provider & Secrets Client for Communicator

Implements 'Login with nID' relying-party integration as specified in ECOSYSTEM.md:
- nID is the single identity provider across the NEWAX ecosystem.
- Authorization-code exchange resolves nID Account identity claims.
- Maps nID Account to local session and tenant context.
- Pre-bootstrap Vault Loader fetches dynamic credentials at runtime.
"""

from __future__ import annotations

import logging
import secrets
import time
from typing import Any
from urllib.parse import urlsplit

import httpx

from communicator.config import settings

logger = logging.getLogger(__name__)

# In-memory login state tracker (state -> record) with TTL
_pending_states: dict[str, dict[str, Any]] = {}

# Active verified sessions (token -> session_claims)
_active_sessions: dict[str, dict[str, Any]] = {}

STATE_TTL_SECONDS = 600  # 10 minutes


def _prune_expired_states() -> None:
    now = time.time()
    expired = [k for k, v in _pending_states.items() if now - v.get("created_at", 0) > STATE_TTL_SECONDS]
    for k in expired:
        _pending_states.pop(k, None)


class NidAuthError(Exception):
    """Raised when an nID OAuth exchange or validation fails."""


class NidClient:
    """Client for interacting with nID's OAuth2 and Secrets Vault APIs."""

    def __init__(
        self,
        base_url: str | None = None,
        client_id: str | None = None,
        client_secret: str | None = None,
        redirect_uri: str | None = None,
    ) -> None:
        self.base_url = (base_url or settings.nid_base_url or "https://nid.newax.co").rstrip("/")
        self.client_id = client_id or settings.nid_client_id or "client_communicator"
        self.client_secret = client_secret or settings.nid_client_secret or "secret_comm_default"
        self.redirect_uri = redirect_uri or settings.nid_redirect_uri

    def _is_allowed_redirect_uri(self, candidate: str) -> bool:
        """The anonymous /auth/nid/start endpoint lets a caller name a custom
        redirect_uri (so the frontend's own callback page, not just the
        backend's, can receive the code) -- a plain string-prefix check
        like `startswith("http://localhost")` is bypassed by
        `http://localhost.attacker.com`, which also starts with that
        string. Parse the URI and compare scheme/hostname/port exactly."""
        try:
            parsed = urlsplit(candidate)
        except ValueError:
            return False
        if parsed.hostname is None:
            return False
        if parsed.hostname == "localhost" and parsed.scheme in ("http", "https"):
            return True
        configured = urlsplit(self.redirect_uri)
        return (
            parsed.scheme == configured.scheme
            and parsed.hostname == configured.hostname
            and parsed.port == configured.port
        )

    async def start_login(self, custom_redirect_uri: str | None = None) -> dict[str, str]:
        """Initiates a 'Login with nID' session and returns the authorize URL."""
        _prune_expired_states()
        state = secrets.token_hex(16)

        redirect_uri = self.redirect_uri
        if custom_redirect_uri and self._is_allowed_redirect_uri(custom_redirect_uri):
            redirect_uri = custom_redirect_uri

        req_payload = {"clientId": self.client_id, "redirectUri": redirect_uri, "state": state}

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                resp = await client.post(
                    f"{self.base_url}/v1/oauth/login-requests",
                    json=req_payload,
                )
                if resp.status_code not in (200, 201):
                    authorize_url = f"{self.base_url}/authorize/mock_poll_{state}"
                    poll_token = f"mock_poll_{state}"
                else:
                    data = resp.json()
                    authorize_url = data.get("authorizeUrl") or f"{self.base_url}/authorize/{data.get('pollToken')}"
                    poll_token = data.get("pollToken") or f"poll_{state}"
        except (httpx.HTTPError, OSError) as exc:
            logger.warning("nID login-request endpoint unreachable (%s), falling back to direct authorize URL", exc)
            authorize_url = f"{self.base_url}/authorize/mock_poll_{state}"
            poll_token = f"mock_poll_{state}"

        _pending_states[state] = {
            "created_at": time.time(),
            "redirect_uri": redirect_uri,
            "poll_token": poll_token,
        }

        return {"authorize_url": authorize_url, "poll_token": poll_token, "state": state}

    async def exchange_code(self, code: str, state: str) -> dict[str, Any]:
        """Exchanges the authorization code from nID for real Account identity claims."""
        _prune_expired_states()
        state_record = _pending_states.pop(state, None)
        if not state_record:
            raise NidAuthError("Invalid or expired state token. Please initiate login again.")

        req_payload = {
            "grant_type": "authorization_code",
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "code": code,
            "redirect_uri": state_record["redirect_uri"],
        }

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                resp = await client.post(f"{self.base_url}/v1/oauth/token", json=req_payload)
                if resp.status_code in (200, 201):
                    data = resp.json()
                    account = data.get("account", {})
                    token = data.get("access_token") or secrets.token_urlsafe(32)
                    claims = {
                        "account_id": account.get("id", f"acc_{secrets.token_hex(6)}"),
                        "phone": account.get("phone", "+923001234567"),
                        "email": account.get("email"),
                        "phone_verified": account.get("phoneVerified", True),
                        "access_token": token,
                    }
                    _active_sessions[token] = claims
                    return claims
                # Body/exception text from an auth endpoint can carry tokens
                # or other credential material -- log the status only.
                logger.warning("nID token exchange rejected: status %d", resp.status_code)
        except (httpx.HTTPError, OSError):
            logger.warning("nID token endpoint unreachable.")

        # Dev/mock fallback -- nID isn't always reachable in local development,
        # and a staging/demo deploy without a live nID instance still needs a
        # working login for review/demo purposes. Gated strictly on the
        # explicit mock-auth flag/dev environment.
        if settings.nid_allow_mock_auth or settings.app_env == "development":
            token = secrets.token_urlsafe(32)
            account_id = f"acc_{abs(hash(code)) & 0xFFFFFFFF:08x}"
            claims = {
                "account_id": account_id,
                "phone": "+923001234567",
                "email": f"{account_id}@newax.local",
                "phone_verified": True,
                "access_token": token,
            }
            _active_sessions[token] = claims
            return claims

        raise NidAuthError("Authentication failed: could not verify code with nID identity provider.")

    def validate_session(self, token: str) -> dict[str, Any] | None:
        """Validates active session claims by token."""
        return _active_sessions.get(token)

    def reconfigure(self, new_settings=None) -> None:
        cfg = new_settings or settings
        self.base_url = (cfg.nid_base_url or self.base_url).rstrip("/")
        self.client_id = cfg.nid_client_id or self.client_id
        self.client_secret = cfg.nid_client_secret or self.client_secret
        self.redirect_uri = cfg.nid_redirect_uri or self.redirect_uri
        logger.info("NidClient reconfigured with runtime credentials.")

    async def fetch_vault_secrets(self, role_id: str, secret_id: str) -> dict[str, Any]:
        """Fetches Communicator runtime secrets from nID Secrets Vault using AppRole credentials."""
        vault_url = settings.nid_vault_addr.rstrip("/")
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                auth_resp = await client.post(
                    f"{vault_url}/v1/auth/approle/login",
                    json={"role_id": role_id, "secret_id": secret_id},
                )
                if auth_resp.status_code != 200:
                    return {}
                token = auth_resp.json().get("auth", {}).get("client_token")

                sec_resp = await client.get(
                    f"{vault_url}/v1/secret/data/communicator",
                    headers={"X-Vault-Token": token},
                )
                if sec_resp.status_code == 200:
                    return sec_resp.json().get("data", {}).get("data", {})
        except (httpx.HTTPError, ValueError) as exc:
            logger.debug("Vault unavailable: %s", exc)
        return {}


nid_client = NidClient()
