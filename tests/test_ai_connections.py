import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.ai_connection import (
    AICustomPolicy,
    AIConnection,
    AIJobType,
    AIProvider,
)
from communicator.services.ai_connection_service import AIConnectionService


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_seed_and_list_ai_connections(db_session):
    await AIConnectionService.seed_default_connections_and_policies(db_session)
    connections = await AIConnectionService.list_connections(db_session)

    assert len(connections) >= 5
    providers = [c["provider"] for c in connections]
    assert AIProvider.ANTHROPIC.value in providers
    assert AIProvider.OPENAI.value in providers
    assert AIProvider.DEEPSEEK.value in providers
    assert AIProvider.OLLAMA.value in providers

    claude = next(c for c in connections if c["provider"] == AIProvider.ANTHROPIC.value)
    assert claude["has_api_key"] is True
    assert claude["api_key_masked"].startswith("••••")
    assert AIJobType.AUTONOMOUS_AGENT.value in claude["assigned_jobs"]


@pytest.mark.asyncio
async def test_create_and_test_custom_connection(db_session):
    await AIConnectionService.seed_default_connections_and_policies(db_session)

    # Add custom Groq connection
    conn = await AIConnectionService.create_or_update_connection(
        db=db_session,
        name="Groq Llama 3.3 70B Fast Inference",
        provider="groq",
        model_id="llama-3.3-70b-versatile",
        api_key="gsk_custom_groq_key_12345",
        temperature=0.5,
    )
    assert conn.id is not None
    assert conn.provider == "groq"

    # Test the connection
    test_res = await AIConnectionService.test_connection(db_session, conn.id)
    assert test_res["status"] == "connected"
    assert test_res["latency_ms"] > 0


@pytest.mark.asyncio
async def test_assign_model_to_ai_job(db_session):
    await AIConnectionService.seed_default_connections_and_policies(db_session)
    connections = await AIConnectionService.list_connections(db_session)

    # Route self_improving to DeepSeek
    deepseek = next(c for c in connections if c["provider"] == AIProvider.DEEPSEEK.value)
    res = await AIConnectionService.assign_model_to_job(
        db_session,
        job_type=AIJobType.SELF_IMPROVING.value,
        connection_id=uuid.UUID(deepseek["id"]),
    )
    assert res["status"] == "routed"
    assert res["assigned_to"] == deepseek["name"]

    # Verify mapping
    mapping = await AIConnectionService.get_job_model_mapping(db_session)
    assert mapping[AIJobType.SELF_IMPROVING.value]["provider"] == AIProvider.DEEPSEEK.value


@pytest.mark.asyncio
async def test_custom_policies_and_effective_prompt(db_session):
    await AIConnectionService.seed_default_connections_and_policies(db_session)

    # Update Self-Improving AI Policy
    updated = await AIConnectionService.update_custom_policy(
        db=db_session,
        job_type=AIJobType.SELF_IMPROVING.value,
        tone="rigorous, data-driven",
        guidelines="Focus heavily on reducing customer escalations. Extract winning de-escalation scripts.",
        rules=[
            {"id": "r1", "rule": "Never adopt unverified refunds over $50.", "enabled": True},
            {"id": "r2", "rule": "Prioritize replies that mention tracking links.", "enabled": True},
        ],
        guardrails=["Do not alter active refund thresholds without manager review."],
        min_csat_learning_threshold=4.5,
    )

    assert updated["tone"] == "rigorous, data-driven"
    assert updated["min_csat_learning_threshold"] == 4.5
    assert len(updated["rules"]) == 2

    # Check dynamically compiled prompt
    prompt = await AIConnectionService.get_effective_prompt(db_session, AIJobType.SELF_IMPROVING.value)
    assert "rigorous, data-driven" in prompt
    assert "Never adopt unverified refunds over $50." in prompt
    assert "[DO NOT] Do not alter active refund thresholds" in prompt


@pytest.mark.asyncio
async def test_ai_connections_api_routes(override_database_dependency, db_session):
    async with AsyncClient(
        transport=ASGITransport(app=app),
        base_url="http://test",
    ) as client:
        # 1. List connections
        resp = await client.get("/api/v1/ai/connections")
        assert resp.status_code == 200
        conns = resp.json()
        assert len(conns) >= 5

        # 2. Test connection via API
        test_resp = await client.post(f"/api/v1/ai/connections/{conns[0]['id']}/test")
        assert test_resp.status_code == 200
        assert test_resp.json()["status"] == "connected"

        # 3. Get job mapping
        map_resp = await client.get("/api/v1/ai/jobs/mapping")
        assert map_resp.status_code == 200
        assert "self_improving" in map_resp.json()
        assert "autonomous_agent" in map_resp.json()

        # 4. Get and update custom policy
        pol_resp = await client.get("/api/v1/ai/policies/autonomous_agent")
        assert pol_resp.status_code == 200
        assert pol_resp.json()["job_type"] == "autonomous_agent"

        update_resp = await client.put(
            "/api/v1/ai/policies/autonomous_agent",
            json={
                "tone": "hyper-empathetic",
                "guidelines": "Always lead with warm reassurance.",
            },
        )
        assert update_resp.status_code == 200
        assert update_resp.json()["tone"] == "hyper-empathetic"

        # 5. Get compiled prompt
        prompt_resp = await client.get("/api/v1/ai/prompts/autonomous_agent")
        assert prompt_resp.status_code == 200
        assert "hyper-empathetic" in prompt_resp.json()["prompt"]
