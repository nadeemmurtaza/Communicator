import uuid

import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.webhook import WebhookEvent
from communicator.services import webhook_service


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_seed_and_list_endpoints(db_session):
    await webhook_service.seed_default_webhooks_if_empty(db_session)
    endpoints = await webhook_service.list_endpoints(db_session)

    assert len(endpoints) >= 2
    assert any("CRM" in ep.name for ep in endpoints)
    assert any("Slack" in ep.name for ep in endpoints)

    masked = [webhook_service.mask_endpoint(ep) for ep in endpoints]
    for m in masked:
        assert "••••" in m["secret_masked"]
        assert m["has_secret"] is True


@pytest.mark.asyncio
async def test_compute_signature():
    secret = "test_webhook_secret_key_123"
    payload_bytes = b'{"event":"test","message":"hello"}'
    sig = webhook_service.compute_signature(secret, payload_bytes)

    assert sig.startswith("sha256=")
    assert len(sig) == 7 + 64  # 'sha256=' + 64 hex characters


@pytest.mark.asyncio
async def test_create_and_update_endpoint(db_session):
    ep = await webhook_service.create_endpoint(
        session=db_session,
        name="Data Platform ETL Hook",
        url="https://api.internal.bayworks.co/etl/webhooks",
        events=[WebhookEvent.CONVERSATION_CREATED.value, WebhookEvent.MESSAGE_CREATED.value],
        headers={"X-Custom-Auth": "SecretToken99"},
        is_active=True,
    )
    assert ep.name == "Data Platform ETL Hook"
    assert ep.secret.startswith("whsec_")
    assert WebhookEvent.CONVERSATION_CREATED.value in ep.events
    original_secret = ep.secret

    # Update endpoint
    updated = await webhook_service.update_endpoint(
        session=db_session,
        endpoint_id=ep.id,
        name="Data Platform ETL Hook v2",
        events=[WebhookEvent.TICKET_CREATED.value],
        rotate_secret=True,
    )
    assert updated.name == "Data Platform ETL Hook v2"
    assert WebhookEvent.TICKET_CREATED.value in updated.events
    assert updated.secret != original_secret


@pytest.mark.asyncio
async def test_dispatch_event_and_delivery_logging(db_session):
    await webhook_service.seed_default_webhooks_if_empty(db_session)

    # Dispatch ticket.created event
    event_data = {
        "ticket_id": str(uuid.uuid4()),
        "ticket_number": "TICK-9081",
        "title": "Payment gateway timeout",
        "priority": "urgent",
        "status": "open",
    }

    logs = await webhook_service.dispatch_event(
        session=db_session,
        event=WebhookEvent.TICKET_CREATED.value,
        data=event_data,
    )

    assert len(logs) >= 1
    log = logs[0]
    assert log.event == WebhookEvent.TICKET_CREATED.value
    assert log.status_code == 200
    assert log.success is True
    assert log.duration_ms > 0
    assert log.payload["data"]["ticket_number"] == "TICK-9081"


@pytest.mark.asyncio
async def test_endpoint_ping_and_resend(db_session):
    ep = await webhook_service.create_endpoint(
        session=db_session,
        name="Diagnostic Listener",
        url="https://listener.internal.bayworks.co/events",
        events=["*"],
    )

    # 1. Test ping
    ping_res = await webhook_service.test_endpoint_ping(db_session, ep.id)
    assert ping_res["success"] is True
    assert ping_res["status_code"] == 200
    delivery_id = uuid.UUID(ping_res["delivery_id"])

    # 2. Resend delivery
    resend_res = await webhook_service.resend_delivery(db_session, delivery_id)
    assert resend_res["success"] is True
    assert resend_res["attempt"] == 2


@pytest.mark.asyncio
async def test_outgoing_webhook_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Get events catalog
        res_cat = await client.get("/api/v1/outgoing-webhooks/events/catalog")
        assert res_cat.status_code == 200
        catalog = res_cat.json()
        assert len(catalog) >= 8
        assert any(c["event"] == "conversation.created" for c in catalog)

        # 2. List endpoints
        res_list = await client.get("/api/v1/outgoing-webhooks")
        assert res_list.status_code == 200
        endpoints = res_list.json()
        assert len(endpoints) >= 2

        # 3. Create endpoint via API
        create_payload = {
            "name": "Billing & Invoices Webhook",
            "url": "https://api.internal.bayworks.co/billing/hooks",
            "events": ["conversation.status_changed", "ticket.resolved"],
            "headers": {"X-App-Env": "prod"},
            "is_active": True,
        }
        res_create = await client.post("/api/v1/outgoing-webhooks", json=create_payload)
        assert res_create.status_code == 201
        created = res_create.json()
        assert created["name"] == "Billing & Invoices Webhook"
        assert created["has_secret"] is True
        ep_id = created["id"]

        # 4. Trigger live test ping via API
        res_ping = await client.post(f"/api/v1/outgoing-webhooks/{ep_id}/test")
        assert res_ping.status_code == 200
        ping_data = res_ping.json()
        assert ping_data["success"] is True
        assert ping_data["status_code"] == 200

        # 5. List deliveries for endpoint
        res_deliv = await client.get(f"/api/v1/outgoing-webhooks/{ep_id}/deliveries")
        assert res_deliv.status_code == 200
        deliveries = res_deliv.json()
        assert len(deliveries) >= 1
        delivery_id = deliveries[0]["id"]

        # 6. Resend delivery via API
        res_resend = await client.post(f"/api/v1/outgoing-webhooks/deliveries/{delivery_id}/resend")
        assert res_resend.status_code == 200
        assert res_resend.json()["success"] is True

        # 7. Delete endpoint via API
        res_del = await client.delete(f"/api/v1/outgoing-webhooks/{ep_id}")
        assert res_del.status_code == 200
        assert res_del.json()["success"] is True
