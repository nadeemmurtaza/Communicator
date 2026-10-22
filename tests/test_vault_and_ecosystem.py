"""Tests for nID Vault Loader and Ecosystem Cross-Repo Dispatches in Communicator."""

import pytest
from communicator.vault_loader import load_secrets_from_nid_vault
from communicator.services.email_dispatch import dispatch_email_via_newcom
from communicator.services.anax_telemetry import emit_anax_customer_telemetry
from communicator.services.ecosystem_client import ecosystem_client


@pytest.mark.asyncio
async def test_vault_loader_graceful_fallback():
    # When no vault creds are present, should return empty dict without crashing
    secrets = await load_secrets_from_nid_vault()
    assert isinstance(secrets, dict)


@pytest.mark.asyncio
async def test_email_dispatch_via_newcom():
    res = await dispatch_email_via_newcom(
        to_email="customer@example.com",
        subject="Your Order Confirmed!",
        html_body="<p>Order ORD-123 is confirmed.</p>",
        order_id="ORD-123",
        tenant_id="tenant_pk",
    )
    assert res["status"] in ("dispatched", "relay_acknowledged", "queued_offline")
    assert "eventId" in res


@pytest.mark.asyncio
async def test_anax_customer_telemetry():
    res = await emit_anax_customer_telemetry(
        tenant_id="tenant_pk",
        buying_signal_count=5,
        complaint_count=1,
        refund_request_count=2,
        fraud_anomaly_rate=0.08,
        live_proof_active=True,
    )
    assert res["status"] in ("dispatched", "relay_acknowledged", "queued_offline")
    assert "eventId" in res


@pytest.mark.asyncio
async def test_ecosystem_client_rpc():
    res = await ecosystem_client.send_rpc(
        target_module="erp-1",
        action="check_inventory",
        payload={"sku": "LAWN-V3-XL"},
        tenant_id="tenant_pk",
    )
    assert res["status"] in ("dispatched", "relay_acknowledged", "queued_offline")
    assert "eventId" in res
