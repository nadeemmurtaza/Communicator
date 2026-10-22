import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.simulation import (
    SimulationVerdict,
)
from communicator.services.simulation_service import SimulationService


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_seed_and_list_policies(db_session):
    await SimulationService.seed_default_policies_and_scenarios(db_session)
    policies = await SimulationService.list_policies(db_session)

    assert len(policies) >= 3
    names = [p.name for p in policies]
    assert any("Return & Refund Policy" in n for n in names)
    assert any("Shipping Delays" in n for n in names)
    assert any("VIP Customer Protocol" in n for n in names)

    refund_p = next(p for p in policies if "Return & Refund Policy" in p.name)
    assert len(refund_p.process_steps) == 5
    assert refund_p.rules["autonomous_refund_cap"] == 50.0
    assert len(refund_p.guardrails) >= 2


@pytest.mark.asyncio
async def test_run_simulation_scenario_autonomous(db_session):
    await SimulationService.seed_default_policies_and_scenarios(db_session)
    scenarios = await SimulationService.list_scenarios(db_session)

    # Find the $42 perfume bottle scenario
    s1 = next(s for s in scenarios if "Damaged Goods" in s["name"])

    import uuid

    run = await SimulationService.run_simulation(db_session, uuid.UUID(s1["id"]))

    assert run.id is not None
    assert run.verdict == SimulationVerdict.PASSED.value
    assert run.policy_adherence_score >= 90.0
    assert run.empathy_score >= 85.0
    assert run.turn_count >= 2
    assert len(run.transcript) >= 4  # 2 customer turns, 2 agent turns

    # Check transcript roles and speakers
    speakers = [t["speaker"] for t in run.transcript]
    assert "Sarah Miller" in speakers
    assert "Captain AI Agent" in speakers

    # Verify Agent applied the <$50 autonomous refund rule
    agent_replies = [t["content"] for t in run.transcript if t["role"] == "agent"]
    assert any("$42.00" in r or "refund" in r.lower() for r in agent_replies)


@pytest.mark.asyncio
async def test_run_simulation_scenario_escalation(db_session):
    await SimulationService.seed_default_policies_and_scenarios(db_session)
    scenarios = await SimulationService.list_scenarios(db_session)

    # Find the high value watch scenario ($280)
    s2 = next(s for s in scenarios if "High-Value Return" in s["name"])

    import uuid

    run = await SimulationService.run_simulation(db_session, uuid.UUID(s2["id"]))

    assert run.id is not None
    # Because expected outcome was escalate_to_human and agent routed for approval, verdict is passed
    assert run.verdict == SimulationVerdict.PASSED.value
    assert run.policy_adherence_score >= 90.0

    agent_replies = [t["content"] for t in run.transcript if t["role"] == "agent"]
    # Agent must NOT issue immediate $280 refund autonomously; must route for supervisor/teammate approval
    assert any("approval" in r.lower() or "escalation" in r.lower() for r in agent_replies)


@pytest.mark.asyncio
async def test_batch_simulations_suite(db_session):
    await SimulationService.seed_default_policies_and_scenarios(db_session)
    batch_res = await SimulationService.run_batch_simulations(db_session)

    assert batch_res["total_scenarios_tested"] >= 4
    assert batch_res["pass_rate_percentage"] >= 90.0
    assert batch_res["average_score"] >= 90.0
    assert len(batch_res["results"]) >= 4


@pytest.mark.asyncio
async def test_interactive_sandbox_turn(db_session):
    await SimulationService.seed_default_policies_and_scenarios(db_session)

    # Test refund query
    res = await SimulationService.interactive_sandbox_turn(
        db_session,
        message="Can I get a refund for my order #ORD-1234? The dress has a broken zipper.",
    )
    assert "reply" in res
    assert "SOP-REF-201" in res["policy_matched"]
    assert len(res["guardrails_applied"]) > 0

    # Test shipping delay query
    res_ship = await SimulationService.interactive_sandbox_turn(
        db_session,
        message="Where is my tracking number? The express delivery is 2 days delayed.",
    )
    assert "credit" in res_ship["reply"].lower() or "shipping" in res_ship["reply"].lower()


@pytest.mark.asyncio
async def test_simulation_api_routes(override_database_dependency, db_session):
    async with AsyncClient(
        transport=ASGITransport(app=app),
        base_url="http://test",
    ) as client:
        # 1. List Policies
        p_resp = await client.get("/api/v1/simulations/policies")
        assert p_resp.status_code == 200
        policies = p_resp.json()
        assert len(policies) >= 3

        # 2. List Scenarios
        s_resp = await client.get("/api/v1/simulations/scenarios")
        assert s_resp.status_code == 200
        scenarios = s_resp.json()
        assert len(scenarios) >= 4

        # 3. Run single scenario via API
        scenario_id = scenarios[0]["id"]
        run_resp = await client.post(
            "/api/v1/simulations/run",
            json={"scenario_id": scenario_id},
        )
        assert run_resp.status_code == 200
        run_data = run_resp.json()
        assert run_data["verdict"] in ["passed", "escalated"]
        assert len(run_data["transcript"]) > 0

        # 4. Interactive Sandbox Chat via API
        sandbox_resp = await client.post(
            "/api/v1/simulations/sandbox/chat",
            json={"message": "I want to exchange size 8 shoes for size 9."},
        )
        assert sandbox_resp.status_code == 200
        assert "reply" in sandbox_resp.json()

        # 5. Stats
        stats_resp = await client.get("/api/v1/simulations/stats")
        assert stats_resp.status_code == 200
        stats_data = stats_resp.json()
        assert "pass_rate_percentage" in stats_data
        assert stats_data["agent_name"] == "Captain AI Agent"
