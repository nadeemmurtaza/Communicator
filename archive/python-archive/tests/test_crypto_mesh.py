"""Tests for CryptoMeshClient (Zero-Trust Multi-Hop Re-Encryption Client)."""

import os

import pytest

from communicator.crypto_mesh import (
    CryptographicIntegrityError,
    CryptoMeshClient,
)


@pytest.fixture
def keys():
    return {
        "ingress": os.urandom(32).hex(),
        "egress": os.urandom(32).hex(),
        "auth": os.urandom(32).hex(),
    }


def test_crypto_mesh_encryption_decryption(keys):
    client = CryptoMeshClient(
        tenant_id="tenant-123",
        entity_id="communicator",
        key_id="kid_001",
        ingress_secret_hex=keys["ingress"],
        egress_secret_hex=keys["ingress"],  # Loopback test with same key
        auth_secret_hex=keys["auth"],
    )

    payload = {"lead_id": "lead_99", "channel": "whatsapp", "status": "sent"}
    envelope = client.encrypt_for_relay(recipient_id="communicator", payload=payload)

    assert envelope.tenantId == "tenant-123"
    assert envelope.senderId == "communicator"
    assert envelope.recipientId == "communicator"
    assert envelope.ciphertext != str(payload)
    assert envelope.signature

    # Decrypt
    decrypted = client.decrypt_from_relay(envelope)
    assert decrypted == payload


def test_crypto_mesh_tampering_detected(keys):
    client = CryptoMeshClient(
        tenant_id="tenant-123",
        entity_id="communicator",
        key_id="kid_001",
        ingress_secret_hex=keys["ingress"],
        egress_secret_hex=keys["ingress"],
        auth_secret_hex=keys["auth"],
    )

    payload = {"secure_data": "passcode_123"}
    envelope = client.encrypt_for_relay(recipient_id="communicator", payload=payload)

    # Tamper with ciphertext
    tampered = envelope.model_copy(update={"ciphertext": "dGFtcGVyZWQ="})

    with pytest.raises(CryptographicIntegrityError):
        client.decrypt_from_relay(tampered)


def test_crypto_mesh_cross_tenant_rejected(keys):
    client_a = CryptoMeshClient(
        tenant_id="tenant-A",
        entity_id="communicator",
        key_id="kid_001",
        ingress_secret_hex=keys["ingress"],
        egress_secret_hex=keys["ingress"],
        auth_secret_hex=keys["auth"],
    )
    client_b = CryptoMeshClient(
        tenant_id="tenant-B",
        entity_id="communicator",
        key_id="kid_001",
        ingress_secret_hex=keys["ingress"],
        egress_secret_hex=keys["ingress"],
        auth_secret_hex=keys["auth"],
    )

    envelope_a = client_a.encrypt_for_relay(recipient_id="communicator", payload={"secret": 1})

    with pytest.raises(CryptographicIntegrityError, match="Cross-tenant violation"):
        client_b.decrypt_from_relay(envelope_a)
