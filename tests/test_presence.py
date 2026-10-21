import time
import pytest
from httpx import AsyncClient

from communicator.services.presence import presence_manager


@pytest.fixture(autouse=True)
def clean_presences():
    presence_manager.reset_all()
    yield
    presence_manager.reset_all()


@pytest.mark.asyncio
async def test_presence_viewing_heartbeat(test_client: AsyncClient):
    conv_id = "conv-presence-1"

    # 1. Update presence to viewing
    res = await test_client.post(
        f"/api/v1/conversations/{conv_id}/presence",
        json={
            "agent_name": "Jamie Cole",
            "action": "viewing",
            "agent_avatar": "https://example.com/avatar.jpg",
        },
    )
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert data["presence"]["agentName"] == "Jamie Cole"
    assert data["presence"]["action"] == "viewing"

    # 2. Get list of presences
    res_list = await test_client.get(f"/api/v1/conversations/{conv_id}/presence")
    assert res_list.status_code == 200
    agents = res_list.json()
    assert len(agents) == 1
    assert agents[0]["agentName"] == "Jamie Cole"
    assert agents[0]["action"] == "viewing"


@pytest.mark.asyncio
async def test_presence_typing_and_collision_warning(test_client: AsyncClient):
    conv_id = "conv-presence-2"

    # Jamie Cole begins typing
    await test_client.post(
        f"/api/v1/conversations/{conv_id}/presence",
        json={
            "agent_name": "Jamie Cole",
            "action": "typing",
        },
    )

    # Riley Brooks checks for collision
    res_col = await test_client.get(
        f"/api/v1/conversations/{conv_id}/collision-check?current_agent=Riley Brooks"
    )
    assert res_col.status_code == 200
    col = res_col.json()
    assert col["hasCollision"] is True
    assert col["severity"] == "danger"
    assert "Jamie Cole is actively composing a reply" in col["warningMessage"]
    assert len(col["typers"]) == 1
    assert col["typers"][0]["agentName"] == "Jamie Cole"

    # If Jamie Cole queries for herself, no collision with herself
    res_self = await test_client.get(
        f"/api/v1/conversations/{conv_id}/collision-check?current_agent=Jamie Cole"
    )
    assert res_self.status_code == 200
    assert res_self.json()["hasCollision"] is False


@pytest.mark.asyncio
async def test_presence_viewing_warning(test_client: AsyncClient):
    conv_id = "conv-presence-3"

    # Morgan Hayes is viewing
    await test_client.post(
        f"/api/v1/conversations/{conv_id}/presence",
        json={
            "agent_name": "Morgan Hayes",
            "action": "viewing",
        },
    )

    # Riley checks collision
    res = await test_client.get(
        f"/api/v1/conversations/{conv_id}/collision-check?current_agent=Riley Brooks"
    )
    assert res.status_code == 200
    data = res.json()
    assert data["hasCollision"] is True
    assert data["severity"] == "warning"
    assert "Morgan Hayes is viewing this conversation" in data["warningMessage"]
    assert len(data["viewers"]) == 1


@pytest.mark.asyncio
async def test_presence_leave_and_delete(test_client: AsyncClient):
    conv_id = "conv-presence-4"

    # Register presence
    await test_client.post(
        f"/api/v1/conversations/{conv_id}/presence",
        json={"agent_name": "Alex Morgan", "action": "viewing"},
    )
    assert len(presence_manager.get_presences(conv_id)) == 1

    # Send left action
    res_left = await test_client.post(
        f"/api/v1/conversations/{conv_id}/presence",
        json={"agent_name": "Alex Morgan", "action": "left"},
    )
    assert res_left.status_code == 200
    assert len(presence_manager.get_presences(conv_id)) == 0

    # Test explicit DELETE endpoint
    await test_client.post(
        f"/api/v1/conversations/{conv_id}/presence",
        json={"agent_name": "Alex Morgan", "action": "viewing"},
    )
    del_res = await test_client.delete(
        f"/api/v1/conversations/{conv_id}/presence?agent_name=Alex Morgan"
    )
    assert del_res.status_code == 200
    assert del_res.json()["removed"] is True
    assert len(presence_manager.get_presences(conv_id)) == 0


@pytest.mark.asyncio
async def test_presence_ttl_cleanup(test_client: AsyncClient):
    conv_id = "conv-presence-5"

    # Register agent
    presence_manager.update_presence(
        conversation_id=conv_id,
        agent_name="Ghost Agent",
        action="viewing",
    )
    assert len(presence_manager.get_presences(conv_id)) == 1

    # Artificially age the presence past the 30-second TTL
    presence_manager._store[conv_id]["Ghost Agent"].last_seen_at = time.time() - 35.0

    # Querying presences should now automatically prune the expired entry
    active = presence_manager.get_presences(conv_id)
    assert len(active) == 0

    # Collision check should also report no collision
    res = await test_client.get(
        f"/api/v1/conversations/{conv_id}/collision-check?current_agent=Riley Brooks"
    )
    assert res.json()["hasCollision"] is False
