import uuid

import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.self_improving import (
    AgentInsight,
    InsightCategory,
    InsightStatus,
)
from communicator.services.self_improving_service import SelfImprovingService


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_seed_and_list_insights(db_session):
    await SelfImprovingService.seed_default_insights_and_retraining(db_session)
    insights = await SelfImprovingService.list_insights(db_session)

    assert len(insights) >= 4
    categories = [i["category"] for i in insights]
    assert InsightCategory.HUMAN_EXEMPLAR.value in categories
    assert InsightCategory.TREND.value in categories
    assert InsightCategory.TEAM_COACHING.value in categories

    exemplar = next(i for i in insights if i["category"] == InsightCategory.HUMAN_EXEMPLAR.value)
    assert exemplar["human_rep_name"] is not None
    assert exemplar["exemplar_content"] is not None
    assert exemplar["impact_score"] >= 8.0
    assert exemplar["estimated_resolution_lift"] > 0


@pytest.mark.asyncio
async def test_apply_insight_retraining(db_session):
    await SelfImprovingService.seed_default_insights_and_retraining(db_session)
    insights = await SelfImprovingService.list_insights(db_session, status="open")
    target_insight = insights[0]

    result = await SelfImprovingService.apply_insight_retraining(
        db_session, uuid.UUID(target_insight["id"]), author_name="Operations Lead"
    )

    assert "retraining_event_id" in result
    assert result["version_to"] is not None
    assert result["post_training_resolution_rate"] > result["baseline_resolution_rate"]
    assert result["resolution_lift"] == target_insight["estimated_resolution_lift"]

    # Check updated insight status in DB
    refreshed = await db_session.get(AgentInsight, uuid.UUID(target_insight["id"]))
    assert refreshed.status == InsightStatus.APPLIED.value
    assert refreshed.applied_at is not None


@pytest.mark.asyncio
async def test_dismiss_insight(db_session):
    await SelfImprovingService.seed_default_insights_and_retraining(db_session)
    insights = await SelfImprovingService.list_insights(db_session, status="open")
    target = insights[-1]

    dismissed = await SelfImprovingService.dismiss_insight(db_session, uuid.UUID(target["id"]))
    assert dismissed is True

    refreshed = await db_session.get(AgentInsight, uuid.UUID(target["id"]))
    assert refreshed.status == InsightStatus.DISMISSED.value


@pytest.mark.asyncio
async def test_retraining_history_and_stats(db_session):
    await SelfImprovingService.seed_default_insights_and_retraining(db_session)

    history = await SelfImprovingService.list_retraining_history(db_session)
    assert len(history) >= 1
    assert history[0]["resolution_lift"] > 0

    stats = await SelfImprovingService.get_self_improving_stats(db_session)
    assert stats["total_insights"] >= 4
    assert stats["cumulative_resolution_lift"] >= 4.0
    assert stats["agent_name"] == "Captain AI Agent"


@pytest.mark.asyncio
async def test_trigger_auto_learning_scan(db_session):
    await SelfImprovingService.seed_default_insights_and_retraining(db_session)
    scan_res = await SelfImprovingService.trigger_auto_learning_scan(db_session)

    assert "conversations_scanned" in scan_res
    assert scan_res["status"] == "scan_complete"


@pytest.mark.asyncio
async def test_self_improving_api_routes(override_database_dependency, db_session):
    async with AsyncClient(
        transport=ASGITransport(app=app),
        base_url="http://test",
    ) as client:
        # 1. List Insights
        resp = await client.get("/api/v1/self-improving/insights")
        assert resp.status_code == 200
        insights = resp.json()
        assert len(insights) >= 4

        # 2. Apply Insight via API
        first_id = insights[0]["id"]
        apply_resp = await client.post(
            f"/api/v1/self-improving/insights/{first_id}/apply",
            json={"author_name": "Test Manager"},
        )
        assert apply_resp.status_code == 200
        assert "version_to" in apply_resp.json()

        # 3. Retraining History via API
        history_resp = await client.get("/api/v1/self-improving/retraining-history")
        assert history_resp.status_code == 200
        assert len(history_resp.json()) >= 1

        # 4. Trigger Scan via API
        scan_resp = await client.post("/api/v1/self-improving/scan")
        assert scan_resp.status_code == 200
        assert scan_resp.json()["status"] == "scan_complete"

        # 5. Stats via API
        stats_resp = await client.get("/api/v1/self-improving/stats")
        assert stats_resp.status_code == 200
        stats = stats_resp.json()
        assert stats["cumulative_resolution_lift"] > 0
        assert stats["agent_name"] == "Captain AI Agent"
