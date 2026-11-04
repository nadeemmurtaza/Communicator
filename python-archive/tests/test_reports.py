import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession


@pytest.mark.asyncio
async def test_get_live_report(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/reports/live")
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert "liveMetrics" in data
    assert "channelBreakdown" in data
    assert "agentsOnline" in data
    assert "openConversations" in data["liveMetrics"]
    assert "unassignedConversations" in data["liveMetrics"]
    assert "pendingReplies" in data["liveMetrics"]
    assert "resolvedToday" in data["liveMetrics"]
    assert "avgWaitTimeFormatted" in data["liveMetrics"]
    assert isinstance(data["agentsOnline"], list)
    assert len(data["agentsOnline"]) >= 1


@pytest.mark.asyncio
async def test_get_conversation_reports(test_client: AsyncClient, db_session: AsyncSession):
    # Test default 7d
    res = await test_client.get("/api/v1/reports/conversations?period=7d")
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert data["period"] == "7d"
    assert "summary" in data
    assert data["summary"]["totalConversations"] > 0
    assert data["summary"]["resolutionRatePercent"] > 0
    assert len(data["timeSeries"]) == 7
    assert "whatsapp" in data["channelDistribution"]

    # Test 30d
    res_30 = await test_client.get("/api/v1/reports/conversations?period=30d")
    assert res_30.status_code == 200
    data_30 = res_30.json()
    assert data_30["period"] == "30d"
    assert len(data_30["timeSeries"]) == 15

    # Test 90d
    res_90 = await test_client.get("/api/v1/reports/conversations?period=90d")
    assert res_90.status_code == 200
    assert res_90.json()["period"] == "90d"


@pytest.mark.asyncio
async def test_get_agent_reports(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/reports/agents?period=7d")
    assert res.status_code == 200
    agents = res.json()
    assert isinstance(agents, list)
    assert len(agents) >= 3
    for agent in agents:
        assert "name" in agent
        assert "conversationsAssigned" in agent
        assert "conversationsResolved" in agent
        assert "avgFirstResponseMinutes" in agent
        assert "csatRating" in agent


@pytest.mark.asyncio
async def test_get_team_reports(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/reports/teams?period=7d")
    assert res.status_code == 200
    teams = res.json()
    assert isinstance(teams, list)
    assert len(teams) >= 2
    for team in teams:
        assert "name" in team
        assert "conversationsCount" in team
        assert "resolutionRate" in team
        assert "slaAdherencePercent" in team


@pytest.mark.asyncio
async def test_get_csat_report(test_client: AsyncClient, db_session: AsyncSession):
    res = await test_client.get("/api/v1/reports/csat?period=7d")
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert "metrics" in data
    assert data["metrics"]["satisfactionPercentage"] >= 80
    assert data["metrics"]["averageScore"] > 4.0
    assert "distribution" in data["metrics"]
    assert "5" in data["metrics"]["distribution"]
    assert "recentFeedback" in data
    assert len(data["recentFeedback"]) >= 1
    sample = data["recentFeedback"][0]
    assert "contactName" in sample
    assert "comment" in sample
    assert "score" in sample
