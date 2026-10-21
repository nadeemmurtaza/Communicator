import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.services import issue_tracker_service


@pytest.fixture
def anyio_backend():
    return "asyncio"


async def _create_test_conversation(db_session) -> Conversation:
    lead = Lead(
        id=uuid.uuid4(),
        harvester_institution_id=f"inst_test_{uuid.uuid4().hex[:8]}",
        name="Issue Tracker Test Corp",
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        id=uuid.uuid4(),
        lead_id=lead.id,
        channel=ChannelType.EMAIL,
        value=f"user_{uuid.uuid4().hex[:6]}@example.com",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(
        id=uuid.uuid4(),
        contact_id=contact.id,
        state=ConversationState.ENGAGED,
    )
    db_session.add(conv)
    await db_session.commit()
    await db_session.refresh(conv)
    return conv


@pytest.mark.asyncio
async def test_seed_and_list_tracker_configs(db_session):
    configs = await issue_tracker_service.list_tracker_configs(db_session)
    assert len(configs) >= 4
    providers = [c.provider for c in configs]
    assert "linear" in providers
    assert "github" in providers
    assert "jira" in providers
    assert "slack" in providers

    linear_cfg = next(c for c in configs if c.provider == "linear")
    formatted = issue_tracker_service.format_tracker_config(linear_cfg)
    assert "••••" in formatted["credentials"]["api_key"]
    assert formatted["credentials"]["has_api_key"] is True


@pytest.mark.asyncio
async def test_update_tracker_config(db_session):
    updated = await issue_tracker_service.update_tracker_config(
        db_session,
        "github",
        {
            "account_name": "Newax Enterprise Org",
            "credentials": {"access_token": "ghp_updated_token_9999999999"},
            "config": {"owner": "newax-hq", "repo": "main-product"},
        },
    )
    assert updated.account_name == "Newax Enterprise Org"
    assert updated.credentials["access_token"] == "ghp_updated_token_9999999999"
    assert updated.config["owner"] == "newax-hq"


@pytest.mark.asyncio
async def test_connection_test(db_session):
    res_linear = await issue_tracker_service.test_tracker_connection(db_session, "linear")
    assert res_linear["status"] in ("connected", "connected_simulated", "error")

    res_slack = await issue_tracker_service.test_tracker_connection(db_session, "slack")
    assert res_slack["status"] in ("connected", "connected_simulated", "error")


@pytest.mark.asyncio
async def test_create_linked_issues_all_providers(db_session):
    conv = await _create_test_conversation(db_session)

    # 1. Linear
    linear_issue = await issue_tracker_service.create_linked_issue(
        session=db_session,
        provider="linear",
        title="Checkout payment gateway timeout on Safari",
        description="Customer encountered timeout during Stripe 3DS auth.",
        priority="urgent",
        issue_type="bug",
        conversation_id=conv.id,
        actor_email="agent@newax.com",
    )
    assert linear_issue.provider == "linear"
    assert linear_issue.issue_key.startswith("ENG-")
    assert "linear.app" in linear_issue.issue_url
    assert linear_issue.status in ("In Progress", "Todo", "Open")

    # 2. GitHub
    gh_issue = await issue_tracker_service.create_linked_issue(
        session=db_session,
        provider="github",
        title="Webhook retries not following exponential backoff",
        description="Customer noticed deliveries failing without retry interval.",
        priority="high",
        issue_type="bug",
        conversation_id=conv.id,
    )
    assert gh_issue.provider == "github"
    assert gh_issue.issue_key.startswith("#")
    assert "github.com" in gh_issue.issue_url

    # 3. Jira
    jira_issue = await issue_tracker_service.create_linked_issue(
        session=db_session,
        provider="jira",
        title="Audit trail export missing timestamp column",
        description="SOC 2 compliance export is missing UTC offset.",
        priority="medium",
        issue_type="task",
        conversation_id=conv.id,
    )
    assert jira_issue.provider == "jira"
    assert jira_issue.issue_key.startswith("SUP-")
    assert "atlassian.net" in jira_issue.issue_url

    # Retrieve all issues for this conversation
    linked = await issue_tracker_service.list_linked_issues(db_session, conversation_id=conv.id)
    assert len(linked) == 3


@pytest.mark.asyncio
async def test_sync_status_and_unlink(db_session):
    conv = await _create_test_conversation(db_session)
    issue = await issue_tracker_service.create_linked_issue(
        session=db_session,
        provider="linear",
        title="Test issue for lifecycle sync",
        description="Testing sync and delete.",
        conversation_id=conv.id,
    )
    orig_status = issue.status
    synced = await issue_tracker_service.sync_issue_status(db_session, issue.id)
    assert synced is not None
    assert synced.status != orig_status or synced.status == "Done"

    # Unlink
    unlinked = await issue_tracker_service.unlink_issue(db_session, issue.id)
    assert unlinked is True
    issues = await issue_tracker_service.list_linked_issues(db_session, conversation_id=conv.id)
    assert not any(i.id == issue.id for i in issues)


@pytest.mark.asyncio
async def test_slack_conversation_sync(db_session):
    conv = await _create_test_conversation(db_session)

    # Sync to Slack
    sync_res = await issue_tracker_service.sync_conversation_to_slack(
        session=db_session,
        conversation_id=conv.id,
        channel_name="#customer-escalations",
        note="Customer is reporting a production blocker.",
        author="Riley Brooks",
    )
    assert sync_res["is_synced"] is True
    assert sync_res["slack_channel_name"] == "#customer-escalations"
    assert sync_res["slack_thread_ts"] != ""
    assert "slack.com" in sync_res["thread_url"]

    # Verify status query
    status = await issue_tracker_service.get_slack_sync_status(db_session, conv.id)
    assert status is not None
    assert status["is_synced"] is True
    assert status["slack_thread_ts"] == sync_res["slack_thread_ts"]


@pytest.mark.asyncio
async def test_issue_tracker_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. List configs
        res = await client.get("/api/v1/integrations/trackers/configs")
        assert res.status_code == 200
        configs = res.json()
        assert len(configs) >= 4

        # 2. Update config
        res = await client.post(
            "/api/v1/integrations/trackers/configs/linear",
            json={"account_name": "API Configured Linear", "is_enabled": True},
        )
        assert res.status_code == 200
        assert res.json()["account_name"] == "API Configured Linear"

        # 3. Test connection
        res = await client.post("/api/v1/integrations/trackers/test/linear")
        assert res.status_code == 200
        assert "status" in res.json()

        # 4. Create linked issue with real conversation in DB
        conv = await _create_test_conversation(db_session)
        res = await client.post(
            "/api/v1/integrations/trackers/create-issue",
            json={
                "provider": "linear",
                "title": "Bug in billing invoice generation",
                "description": "Invoice missing tax breakdown for EU customer.",
                "priority": "high",
                "conversation_id": str(conv.id),
            },
        )
        assert res.status_code == 200
        created = res.json()
        assert created["provider"] == "linear"
        assert created["issue_key"].startswith("ENG-")
        issue_id = created["id"]

        # 5. List linked issues
        res = await client.get(f"/api/v1/integrations/trackers/linked?conversation_id={conv.id}")
        assert res.status_code == 200
        assert len(res.json()) == 1

        # 6. Sync issue status
        res = await client.post(f"/api/v1/integrations/trackers/linked/{issue_id}/sync")
        assert res.status_code == 200

        # 7. Unlink issue
        res = await client.delete(f"/api/v1/integrations/trackers/linked/{issue_id}")
        assert res.status_code == 200
        assert res.json()["success"] is True

        # 8. Slack thread sync
        res = await client.post(
            "/api/v1/integrations/trackers/slack/sync",
            json={
                "conversation_id": str(conv.id),
                "channel_name": "#customer-support",
                "note": "Urgent customer issue escalated.",
            },
        )
        assert res.status_code == 200
        assert res.json()["is_synced"] is True

        # 9. Get Slack status
        res = await client.get(f"/api/v1/integrations/trackers/slack/status/{conv.id}")
        assert res.status_code == 200
        assert res.json()["is_synced"] is True
