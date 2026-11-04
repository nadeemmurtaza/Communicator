from __future__ import annotations

import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession


@pytest.mark.asyncio
async def test_get_knowledge_hub_overview(test_client: AsyncClient, db_session: AsyncSession):
    # Fetch Hub overview (auto-seeds topics and sources)
    res = await test_client.get("/api/v1/knowledge-base/hub")
    assert res.status_code == 200
    data = res.json()

    assert data["totalSources"] >= 6
    assert data["overallResolutionRate"] > 80.0
    assert len(data["topics"]) >= 4

    # Verify topics grouping
    topic_names = [t["name"] for t in data["topics"]]
    assert "Billing & Refunds" in topic_names
    assert "Shipping & Delivery" in topic_names
    assert "Orders & Returns" in topic_names

    # Check touchpoint breakdown
    tp = data["touchpointBreakdown"]
    assert "agent" in tp
    assert "copilot" in tp
    assert "help_center" in tp
    assert tp["agent"]["sourcesDeployed"] >= 5
    assert tp["help_center"]["status"] == "Public Self-Serve"


@pytest.mark.asyncio
async def test_filter_hub_by_topic_and_query(test_client: AsyncClient, db_session: AsyncSession):
    # Filter by topic
    topic_res = await test_client.get("/api/v1/knowledge-base/hub?topic=Shipping%20%26%20Delivery")
    assert topic_res.status_code == 200
    t_data = topic_res.json()
    assert len(t_data["topics"]) == 1
    assert t_data["topics"][0]["name"] == "Shipping & Delivery"
    assert all("Shipping" in s["topic"] for s in t_data["topics"][0]["sources"])

    # Search query
    search_res = await test_client.get("/api/v1/knowledge-base/hub?q=refund")
    assert search_res.status_code == 200
    s_data = search_res.json()
    assert any("Billing" in t["name"] for t in s_data["topics"])


@pytest.mark.asyncio
async def test_list_and_apply_recommendations(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch recommendations (auto-seeds content, customer data, and action gaps)
    rec_res = await test_client.get("/api/v1/knowledge-base/recommendations")
    assert rec_res.status_code == 200
    recs = rec_res.json()
    assert len(recs) >= 3

    gap_types = [r["gapType"] for r in recs]
    assert "content_gap" in gap_types
    assert "customer_data_gap" in gap_types
    assert "action_gap" in gap_types

    # Find the content gap recommendation
    content_gap_rec = next(r for r in recs if r["gapType"] == "content_gap")
    rec_id = content_gap_rec["id"]
    assert content_gap_rec["status"] == "open"
    assert content_gap_rec["impactOnResolution"] > 0

    # 2. 1-Click apply recommendation to generate new article and close gap
    apply_res = await test_client.post(f"/api/v1/knowledge-base/recommendations/{rec_id}/apply")
    assert apply_res.status_code == 200
    applied_data = apply_res.json()
    assert applied_data["status"] == "applied"
    assert applied_data["recommendation"]["status"] == "applied"

    created = applied_data["createdEntry"]
    assert created["topic"] == content_gap_rec["topic"]
    assert "agent" in created["targetTouchpoints"]

    # 3. Dismiss another recommendation
    other_rec = next(r for r in recs if r["id"] != rec_id)
    dismiss_res = await test_client.post(
        f"/api/v1/knowledge-base/recommendations/{other_rec['id']}/dismiss"
    )
    assert dismiss_res.status_code == 200
    assert dismiss_res.json()["status"] == "dismissed"
