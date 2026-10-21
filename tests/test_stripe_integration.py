import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.contact import ChannelType, Contact
from communicator.models.lead import Lead
from communicator.services import stripe_service


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_get_and_update_stripe_config(db_session):
    cfg = await stripe_service.get_or_create_config(db_session)
    assert cfg is not None
    assert cfg.test_mode is True

    # Update config
    updated = await stripe_service.update_config(
        db_session,
        {
            "account_name": "Acme SaaS Billing",
            "api_key": "sk_test_mock1234567890abcdef",
            "publishable_key": "pk_test_mock1234567890abcdef",
            "webhook_secret": "whsec_mock1234567890abcdef",
            "default_currency": "usd",
        },
    )
    assert updated.account_name == "Acme SaaS Billing"
    assert updated.api_key == "sk_test_mock1234567890abcdef"

    formatted = stripe_service.format_stripe_config(updated)
    assert "••••" in formatted["api_key_masked"]
    assert formatted["has_api_key"] is True
    assert formatted["account_name"] == "Acme SaaS Billing"


@pytest.mark.asyncio
async def test_test_stripe_connection_sandbox(db_session):
    # Without live sk_ key or with simulated key
    result = await stripe_service.test_stripe_connection(db_session)
    assert result["status"] in ("connected_simulated", "connected", "error")
    assert "tested_at" in result


@pytest.mark.asyncio
async def test_get_customer_billing_deterministic(db_session):
    # Create test lead and contact
    lead = Lead(
        id=uuid.uuid4(),
        harvester_institution_id=f"inst_test_{uuid.uuid4().hex[:8]}",
        name="Global Dynamics Inc",
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        id=uuid.uuid4(),
        lead_id=lead.id,
        channel=ChannelType.EMAIL,
        value="sarah.connor@cyberdyne.org",
        person_name="Sarah Connor",
    )
    db_session.add(contact)
    await db_session.commit()

    # Query by contact ID
    billing = await stripe_service.get_customer_billing(db_session, str(contact.id))
    assert billing["contact_id"] == str(contact.id)
    assert billing["name"] == "Sarah Connor"
    assert billing["email"] == "sarah.connor@cyberdyne.org"
    assert billing["has_active_subscription"] is True
    assert billing["subscription"]["status"] == "active"
    assert billing["mrr"] > 0
    assert billing["ltv"] >= billing["mrr"]
    assert billing["payment_method"]["brand"] in ("visa", "mastercard", "amex")

    # Query by raw email
    billing_email = await stripe_service.get_customer_billing(db_session, "sarah.connor@cyberdyne.org")
    assert billing_email["customer_id"] == billing["customer_id"]
    assert billing_email["mrr"] == billing["mrr"]


@pytest.mark.asyncio
async def test_get_customer_invoices(db_session):
    invoices = await stripe_service.get_customer_invoices(
        db_session, "billing-user@enterprise.com", limit=3
    )
    assert len(invoices) == 3
    for inv in invoices:
        assert inv["id"].startswith("in_")
        assert inv["number"].startswith("INV-")
        assert inv["status"] == "paid"
        assert inv["amount_paid_cents"] > 0
        assert "hosted_invoice_url" in inv
        assert "invoice_pdf_url" in inv


@pytest.mark.asyncio
async def test_process_refund_and_action_log(db_session):
    contact_id = str(uuid.uuid4())
    refund = await stripe_service.process_refund(
        session=db_session,
        charge_or_invoice_id="in_test_12345",
        amount_cents=9900,
        reason="customer_cancellation_within_guarantee",
        actor_email="lead_support@newax.com",
        contact_id=contact_id,
        customer_identifier="cust_9921",
    )
    assert refund["status"] == "succeeded"
    assert refund["amount_cents"] == 9900
    assert refund["id"].startswith("re_sim_")
    assert refund["actor_email"] == "lead_support@newax.com"

    # Verify audit log retrieval
    logs = await stripe_service.list_billing_action_logs(db_session, "cust_9921")
    assert len(logs) >= 1
    assert logs[0].action_type == "refund"
    assert logs[0].amount_cents == 9900


@pytest.mark.asyncio
async def test_create_customer_portal_session(db_session):
    portal = await stripe_service.create_customer_portal_session(
        session=db_session,
        identifier="portal-user@enterprise.com",
        return_url="https://app.newax.com/dashboard",
        actor_email="agent@newax.com",
    )
    assert portal["session_id"].startswith("bps_")
    assert "billing.stripe.com" in portal["url"]

    # Verify action log
    logs = await stripe_service.list_billing_action_logs(db_session, "portal-user@enterprise.com")
    assert any(log.action_type == "portal_session" for log in logs)


@pytest.mark.asyncio
async def test_stripe_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Get config
        res = await client.get("/api/v1/integrations/stripe/config")
        assert res.status_code == 200
        data = res.json()
        assert "account_name" in data
        assert "is_enabled" in data

        # 2. Update config
        res = await client.post(
            "/api/v1/integrations/stripe/config",
            json={"account_name": "API Configured Stripe", "default_currency": "usd"},
        )
        assert res.status_code == 200
        assert res.json()["account_name"] == "API Configured Stripe"

        # 3. Test connection
        res = await client.post("/api/v1/integrations/stripe/test")
        assert res.status_code == 200
        assert "status" in res.json()

        # 4. Get customer billing
        res = await client.get("/api/v1/integrations/stripe/billing/alex.turner@arctic.io")
        assert res.status_code == 200
        billing = res.json()
        assert billing["email"] == "alex.turner@arctic.io"
        assert billing["subscription"]["status"] == "active"

        # 5. Get invoices
        res = await client.get("/api/v1/integrations/stripe/invoices/alex.turner@arctic.io?limit=2")
        assert res.status_code == 200
        invoices = res.json()
        assert len(invoices) == 2

        # 6. Process refund
        res = await client.post(
            "/api/v1/integrations/stripe/refund",
            json={
                "charge_or_invoice_id": invoices[0]["id"],
                "amount_cents": 5000,
                "reason": "goodwill_discount",
                "customer_identifier": "alex.turner@arctic.io",
            },
        )
        assert res.status_code == 200
        refund_data = res.json()
        assert refund_data["status"] == "succeeded"
        assert refund_data["amount_cents"] == 5000

        # 7. Create portal session
        res = await client.post(
            "/api/v1/integrations/stripe/portal-session",
            json={
                "identifier": "alex.turner@arctic.io",
                "return_url": "https://app.newax.com",
            },
        )
        assert res.status_code == 200
        assert "billing.stripe.com" in res.json()["url"]

        # 8. Check logs
        res = await client.get("/api/v1/integrations/stripe/logs/alex.turner@arctic.io")
        assert res.status_code == 200
        logs = res.json()
        assert len(logs) >= 2
