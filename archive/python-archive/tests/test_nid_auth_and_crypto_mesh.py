"""Tests for nID Authentication and Zero-Trust Crypto Mesh Ingress in Communicator."""

import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.crypto_mesh import CryptoMeshClient


@pytest.fixture
def mesh_keys():
    return {
        "tenant_id": "test_tenant",
        "entity_id": "communicator",
        "key_id": "kid_comm_test",
        "ingress_secret_hex": "11" * 32,
        "egress_secret_hex": "22" * 32,
        "auth_secret_hex": "33" * 32,
    }


@pytest.mark.asyncio
async def test_nid_login_start_and_callback():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Start Login
        resp = await client.get("/api/v1/auth/nid/start")
        assert resp.status_code == 200
        data = resp.json()
        assert "authorize_url" in data
        assert "poll_token" in data
        assert "state" in data

        state = data["state"]

        # 2. Callback with valid code
        callback_resp = await client.get(
            "/api/v1/auth/nid/callback",
            params={"code": "test_auth_code_123", "state": state},
        )
        assert callback_resp.status_code == 200
        cb_data = callback_resp.json()
        assert "token" in cb_data
        assert "account_id" in cb_data
        assert cb_data["account_id"].startswith("acc_")

        # 3. GET /me
        me_resp = await client.get(
            "/api/v1/auth/nid/me",
            headers={"Authorization": f"Bearer {cb_data['token']}"},
        )
        assert me_resp.status_code == 200
        assert me_resp.json()["status"] == "authenticated"

        # 4. GET /me unauthorized
        unauth_resp = await client.get("/api/v1/auth/nid/me")
        assert unauth_resp.status_code == 401


@pytest.mark.asyncio
async def test_crypto_mesh_ingress_workflow(mesh_keys):
    # Setup client simulating NeuraCore re-encrypting an envelope for Communicator
    # For Communicator, ingress decrypts using its egress_key!
    sender_client = CryptoMeshClient(
        tenant_id=mesh_keys["tenant_id"],
        entity_id="neura-core",
        key_id=mesh_keys["key_id"],
        ingress_secret_hex=mesh_keys["egress_secret_hex"],  # matching communicator's egress
        egress_secret_hex=mesh_keys["ingress_secret_hex"],
        auth_secret_hex=mesh_keys["auth_secret_hex"],
    )

    from communicator.services.ecosystem_client import ecosystem_client

    # Configure ecosystem_client with matching test keys
    ecosystem_client.mesh_client = CryptoMeshClient(
        tenant_id=mesh_keys["tenant_id"],
        entity_id="communicator",
        key_id=mesh_keys["key_id"],
        ingress_secret_hex=mesh_keys["ingress_secret_hex"],
        egress_secret_hex=mesh_keys["egress_secret_hex"],
        auth_secret_hex=mesh_keys["auth_secret_hex"],
    )

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Encrypt an inbound order.placed event
        payload = {
            "name": "order.placed",
            "payload": {
                "orderId": "ORD-TEST-999",
                "customerPhone": "+923001112233",
                "amount": 25000,
            },
        }
        envelope = sender_client.encrypt_for_relay(
            recipient_id="communicator",
            payload=payload,
        )

        resp = await client.post(
            "/api/v1/crypto-mesh/ingress",
            json=envelope.model_dump(),
        )
        assert resp.status_code == 200
        res_data = resp.json()
        assert res_data["status"] == "processed"
        assert res_data["event"] == "order.placed"
        assert res_data["action"] == "order_confirmation_scheduled"

        # Test tampered envelope rejection
        tampered = envelope.model_dump()
        tampered["signature"] = "invalid_tampered_signature"
        bad_resp = await client.post("/api/v1/crypto-mesh/ingress", json=tampered)
        assert bad_resp.status_code == 400
