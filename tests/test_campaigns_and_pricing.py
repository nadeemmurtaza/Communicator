import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.lead import Lead
from communicator.services.tags import add_tag_to_contact


@pytest.mark.asyncio
async def test_campaign_audience_preview_and_creation(
    test_client: AsyncClient, db_session: AsyncSession
):
    # Setup leads and tagged contacts
    lead = Lead(harvester_institution_id="test:camp-1", name="Acme Inc", institution_type="tech")
    db_session.add(lead)
    await db_session.flush()

    c1 = Contact(
        lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+14155551111", person_name="Maya"
    )
    c2 = Contact(
        lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+14155552222", person_name="Daniel"
    )
    db_session.add_all([c1, c2])
    await db_session.flush()

    await add_tag_to_contact(db_session, c1, "vip-customers")
    await add_tag_to_contact(db_session, c2, "vip-customers")
    await add_tag_to_contact(db_session, c2, "newsletter")  # c2 has both tags

    # Test audience preview with deduplication
    resp_prev = await test_client.get(
        "/api/v1/broadcasts/audience-preview?tags=vip-customers,newsletter"
    )
    assert resp_prev.status_code == 200
    prev_data = resp_prev.json()
    assert prev_data["totalSelected"] == 2  # Deduplicated: exactly 2 distinct contacts
    assert prev_data["dailyLimit"] == 1000

    # Create campaign
    create_payload = {
        "name": "Spring sale early access",
        "template_name": "spring_sale_offer",
        "template_language": "en_US",
        "body_preview": "Hi Maya, our spring sale starts Friday. As a VIP customer you get early access and 20% off until Sunday.",
        "template_params": {"discount": "20%"},
        "tag_name": "vip-customers",
        "scheduled_at": "2026-03-12T10:00:00Z",
    }
    resp_create = await test_client.post("/api/v1/broadcasts", json=create_payload)
    assert resp_create.status_code == 200
    camp = resp_create.json()
    assert camp["name"] == "Spring sale early access"
    assert camp["status"] == "scheduled"
    assert camp["audienceCount"] == 2

    # Test rescheduling
    camp_id = camp["id"]
    resp_reschedule = await test_client.post(
        f"/api/v1/broadcasts/{camp_id}/reschedule",
        json={"scheduled_at": "2026-03-15T14:30:00Z"},
    )
    assert resp_reschedule.status_code == 200
    assert "2026-03-15" in resp_reschedule.json()["scheduledAt"]


@pytest.mark.asyncio
async def test_campaign_analytics_and_dispatch(test_client: AsyncClient, db_session: AsyncSession):
    # Seed default campaigns
    resp_seed = await test_client.post("/api/v1/broadcasts/seed-defaults")
    assert resp_seed.status_code == 200

    # List campaigns
    resp_list = await test_client.get("/api/v1/broadcasts")
    assert resp_list.status_code == 200
    campaigns = resp_list.json()
    assert len(campaigns) >= 3

    spring_campaign = next(c for c in campaigns if c["name"] == "Spring sale early access")

    # Fetch deep analytics
    resp_analytics = await test_client.get(f"/api/v1/broadcasts/{spring_campaign['id']}/analytics")
    assert resp_analytics.status_code == 200
    analytics = resp_analytics.json()

    metrics = analytics["metrics"]
    assert metrics["audience"] == 248
    assert metrics["submitted"] == 240
    assert metrics["delivered"] == 218
    assert metrics["read"] == 164
    assert metrics["failed"] == 12
    assert metrics["skipped"] == 8

    # Check percentages matching Chatwoot example
    assert metrics["submittedPct"] == 97  # 97% of audience
    assert metrics["deliveredPct"] == 88  # 88% of audience
    assert metrics["readPct"] == 66  # 66% of audience
    assert metrics["failedPct"] == 5  # 5% of audience
    assert metrics["skippedPct"] == 3  # 3% of audience


@pytest.mark.asyncio
async def test_whatsapp_pricing_calculator(test_client: AsyncClient):
    # Marketing rate in US
    resp_us = await test_client.get(
        "/api/v1/broadcasts/pricing-calculator?region=US&category=marketing&volume=1000"
    )
    assert resp_us.status_code == 200
    data_us = resp_us.json()
    assert data_us["region"] == "US"
    assert data_us["ratePerConversation"] == 0.0250
    assert data_us["estimatedCostUsd"] == 25.0

    # Utility rate in India
    resp_in = await test_client.get(
        "/api/v1/broadcasts/pricing-calculator?region=IN&category=utility&volume=5000"
    )
    assert resp_in.status_code == 200
    data_in = resp_in.json()
    assert data_in["region"] == "IN"
    assert data_in["ratePerConversation"] == 0.0035
    assert data_in["estimatedCostUsd"] == 17.5
