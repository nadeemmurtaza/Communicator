"""NEWAX Cryptographic Transit Relay - Python Client Adapter

Provides Zero-Trust Multi-Hop Re-Encryption client for Communicator:
- Encrypts outbound messages for NeuraCore using Communicator's unique ingress key.
- Decrypts inbound messages arriving from NeuraCore using Communicator's unique egress key.
- Enforces AES-256-GCM AEAD, HMAC-SHA256 canonical envelope signatures, and anti-replay protection.
"""

from __future__ import annotations

import base64
import hmac
import hashlib
import json
import os
import time
from datetime import datetime, timezone
from typing import Any, Dict, Optional
from pydantic import BaseModel, Field
from cryptography.hazmat.primitives.ciphers.aead import AESGCM


class CryptographicIntegrityError(Exception):
    """Raised when ciphertext authentication tag or signature verification fails."""
    pass


class ReplayAttackDetectedError(Exception):
    """Raised when an envelope timestamp or signature indicates a replayed transmission."""
    pass


class EncryptedTransitEnvelope(BaseModel):
    """Encrypted transit envelope matching @newax/crypto-mesh specification."""
    version: str = Field(default="1.0.0")
    tenantId: str
    senderId: str
    recipientId: str
    iv: str
    ciphertext: str
    authTag: str
    keyId: str
    timestamp: str
    signature: str


class CryptoMeshClient:
    """Client codec for communicating securely across NeuraCore's cryptographic relay bridge."""

    def __init__(
        self,
        tenant_id: str,
        entity_id: str,
        key_id: str,
        ingress_secret_hex: str,
        egress_secret_hex: str,
        auth_secret_hex: str,
        max_clock_skew_seconds: int = 300,
    ) -> None:
        self.tenant_id = tenant_id
        self.entity_id = entity_id
        self.key_id = key_id
        self.ingress_key = bytes.fromhex(ingress_secret_hex)
        self.egress_key = bytes.fromhex(egress_secret_hex)
        self.auth_secret = bytes.fromhex(auth_secret_hex)
        self.max_clock_skew_seconds = max_clock_skew_seconds

        if len(self.ingress_key) != 32 or len(self.egress_key) != 32:
            raise ValueError("AES-256 keys must be exactly 32 bytes (64 hex characters).")

    @classmethod
    def from_env(cls, prefix: str = "NEURA_CRYPTO_") -> "CryptoMeshClient":
        """Instantiates client from environment variables."""
        tenant_id = os.getenv(f"{prefix}TENANT_ID", "default")
        entity_id = os.getenv(f"{prefix}ENTITY_ID", "communicator")
        key_id = os.getenv(f"{prefix}KEY_ID", "kid_default")
        ingress = os.getenv(f"{prefix}INGRESS_SECRET_HEX", "00" * 32)
        egress = os.getenv(f"{prefix}EGRESS_SECRET_HEX", "00" * 32)
        auth = os.getenv(f"{prefix}AUTH_SECRET_HEX", "00" * 32)
        return cls(tenant_id, entity_id, key_id, ingress, egress, auth)

    @staticmethod
    def _build_aad(
        version: str,
        tenant_id: str,
        sender_id: str,
        recipient_id: str,
        key_id: str,
        timestamp: str,
    ) -> bytes:
        return f"{version}:{tenant_id}:{sender_id}:{recipient_id}:{key_id}:{timestamp}".encode("utf-8")

    def _compute_signature(
        self,
        version: str,
        tenant_id: str,
        sender_id: str,
        recipient_id: str,
        key_id: str,
        timestamp: str,
        iv_b64: str,
        auth_tag_b64: str,
        ciphertext_b64: str,
    ) -> str:
        canonical_string = "|".join([
            version,
            tenant_id,
            sender_id,
            recipient_id,
            key_id,
            timestamp,
            iv_b64,
            auth_tag_b64,
            ciphertext_b64,
        ])
        sig = hmac.new(self.auth_secret, canonical_string.encode("utf-8"), hashlib.sha512).digest()
        return base64.b64encode(sig).decode("utf-8")

    def encrypt_for_relay(self, recipient_id: str, payload: Any) -> EncryptedTransitEnvelope:
        """Encrypts a payload with this entity's ingress key, ready for NeuraCore transit."""
        version = "1.0.0"
        timestamp = datetime.now(timezone.utc).isoformat()
        iv = os.urandom(12)

        serialized = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        aad = self._build_aad(
            version,
            self.tenant_id,
            self.entity_id,
            recipient_id,
            self.key_id,
            timestamp,
        )

        aesgcm = AESGCM(self.ingress_key)
        encrypted = aesgcm.encrypt(iv, serialized, aad)

        ciphertext = encrypted[:-16]
        auth_tag = encrypted[-16:]

        iv_b64 = base64.b64encode(iv).decode("utf-8")
        ciphertext_b64 = base64.b64encode(ciphertext).decode("utf-8")
        auth_tag_b64 = base64.b64encode(auth_tag).decode("utf-8")

        signature = self._compute_signature(
            version,
            self.tenant_id,
            self.entity_id,
            recipient_id,
            self.key_id,
            timestamp,
            iv_b64,
            auth_tag_b64,
            ciphertext_b64,
        )

        return EncryptedTransitEnvelope(
            version=version,
            tenantId=self.tenant_id,
            senderId=self.entity_id,
            recipientId=recipient_id,
            iv=iv_b64,
            ciphertext=ciphertext_b64,
            authTag=auth_tag_b64,
            keyId=self.key_id,
            timestamp=timestamp,
            signature=signature,
        )

    def decrypt_from_relay(self, envelope: EncryptedTransitEnvelope | Dict[str, Any]) -> Any:
        """Decrypts an inbound transit envelope delivered from NeuraCore using egress key."""
        env = envelope if isinstance(envelope, EncryptedTransitEnvelope) else EncryptedTransitEnvelope(**envelope)

        if env.tenantId != self.tenant_id:
            raise CryptographicIntegrityError(
                f"Cross-tenant violation: envelope belongs to '{env.tenantId}', expected '{self.tenant_id}'."
            )

        if env.recipientId != self.entity_id:
            raise CryptographicIntegrityError(
                f"Mismatched recipient: envelope addressed to '{env.recipientId}', expected '{self.entity_id}'."
            )

        # Anti-replay timestamp check
        try:
            env_time = datetime.fromisoformat(env.timestamp.replace("Z", "+00:00"))
            delta_seconds = abs((datetime.now(timezone.utc) - env_time).total_seconds())
            if delta_seconds > self.max_clock_skew_seconds:
                raise ReplayAttackDetectedError(
                    f"Replay attack prevented: timestamp delta ({delta_seconds:.1f}s) exceeds max allowed ({self.max_clock_skew_seconds}s)."
                )
        except Exception as e:
            if isinstance(e, ReplayAttackDetectedError):
                raise
            raise CryptographicIntegrityError(f"Malformed timestamp: {e}") from e

        # Signature verification
        expected_sig = self._compute_signature(
            env.version,
            env.tenantId,
            env.senderId,
            env.recipientId,
            env.keyId,
            env.timestamp,
            env.iv,
            env.authTag,
            env.ciphertext,
        )
        if not hmac.compare_digest(env.signature, expected_sig):
            raise CryptographicIntegrityError(
                "Envelope signature mismatch: message has been tampered with in transit."
            )

        # AES-256-GCM Decryption
        try:
            iv = base64.b64decode(env.iv)
            ciphertext = base64.b64decode(env.ciphertext)
            auth_tag = base64.b64decode(env.authTag)
            encrypted = ciphertext + auth_tag

            aad = self._build_aad(
                env.version,
                env.tenantId,
                env.senderId,
                env.recipientId,
                env.keyId,
                env.timestamp,
            )

            aesgcm = AESGCM(self.egress_key)
            decrypted = aesgcm.decrypt(iv, encrypted, aad)
            return json.loads(decrypted.decode("utf-8"))
        except Exception as e:
            raise CryptographicIntegrityError(f"Decryption failed: {e}") from e
