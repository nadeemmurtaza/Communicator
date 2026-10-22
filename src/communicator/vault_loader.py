"""nID Secrets Vault Pre-Bootstrap Loader for Communicator

Implements pre-bootstrap secret resolution matching WO-255.3 and ECOSYSTEM.md:
- Authenticates against nID Vault using AppRole or Vault Token.
- Fetches database credentials, third-party API tokens, and encryption keys prior to service initialization.
- Injects resolved secrets securely into Communicator's settings object.
"""

from __future__ import annotations

import logging
import os
from typing import Any, Dict
import httpx

from communicator.config import settings

logger = logging.getLogger(__name__)


class VaultLoaderError(Exception):
    """Raised when critical secrets fail to load from nID Vault."""
    pass


async def load_secrets_from_nid_vault() -> Dict[str, Any]:
    """
    Connects to nID Secrets Vault, authenticates via AppRole (or NID_VAULT_TOKEN),
    fetches Communicator's configuration secrets, and applies them to settings.
    """
    vault_addr = (os.getenv("NID_VAULT_ADDR") or settings.nid_vault_addr).rstrip("/")
    vault_token = os.getenv("NID_VAULT_TOKEN")
    role_id = os.getenv("NID_VAULT_ROLE_ID") or settings.nid_vault_role_id
    secret_id = os.getenv("NID_VAULT_SECRET_ID") or settings.nid_vault_secret_id

    # If no vault credentials are provided, skip silently (standard development/offline fallback)
    if not vault_token and (not role_id or not secret_id):
        logger.debug("nID Vault credentials not supplied; proceeding with environment configuration.")
        return {}

    async with httpx.AsyncClient(timeout=5.0) as client:
        # Step 1: Resolve token via AppRole if needed
        if not vault_token and role_id and secret_id:
            try:
                auth_resp = await client.post(
                    f"{vault_addr}/v1/auth/approle/login",
                    json={"role_id": role_id, "secret_id": secret_id},
                )
                if auth_resp.status_code == 200:
                    vault_token = auth_resp.json().get("auth", {}).get("client_token")
                else:
                    logger.warning("nID Vault AppRole login rejected with status %d", auth_resp.status_code)
                    return {}
            except Exception as exc:
                logger.warning("nID Vault unreachable: %s", exc)
                return {}

        # Step 2: Fetch secrets from /v1/secret/data/communicator
        try:
            sec_resp = await client.get(
                f"{vault_addr}/v1/secret/data/communicator",
                headers={"X-Vault-Token": vault_token},
            )
            if sec_resp.status_code == 200:
                data = sec_resp.json().get("data", {}).get("data", {})
                _apply_vault_secrets(data)
                logger.info("Successfully loaded %d secrets from nID Vault.", len(data))
                return data
            else:
                logger.warning("nID Vault secrets path returned status %d", sec_resp.status_code)
                return {}
        except Exception as exc:
            logger.warning("Failed to fetch secrets from nID Vault: %s", exc)
            return {}


def _apply_vault_secrets(secrets_dict: Dict[str, Any]) -> None:
    """Applies resolved vault secrets to the settings singleton and environment."""
    for key, val in secrets_dict.items():
        lower_key = key.lower()
        if hasattr(settings, lower_key):
            setattr(settings, lower_key, val)
        os.environ[key.upper()] = str(val)
