from __future__ import annotations

import json
import uuid
from datetime import UTC, datetime, timedelta
from unittest.mock import AsyncMock, patch

import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels.base import SendResult
from communicator.compliance.optout import record_consent
from communicator.models.automation_rule import (
    AutomationRule,
    RuleActionType,
    RuleMatchType,
)
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.team_member import TeamMember
from communicator.services.whatsapp_automations import (
    execute_handoff,
    execute_rule_action,
    handle_agent_commands,
    handle_csat_response,
    handle_navigation_menu,
    match_advanced_rule,
    run_inactivity_nudges,
    seed_default_whatsapp_automations,
    send_csat_survey,
    should_send_welcome_greeting,
)


@pytest.mark.asyncio
async def test_seed_default_whatsapp_automations(db_session: AsyncSession):
    count = await seed_default_whatsapp_automations(db_session)
    assert count >= 10

    # Calling again should not duplicate
    recount = await seed_default_whatsapp_automations(db_session)
    assert recount == 0


@pytest.mark.asyncio
async def test_match_advanced_rule_types(db_session: AsyncSession):
    r1 = AutomationRule(
        name="Exact Test",
        trigger_keyword="pricing",
        match_type=RuleMatchType.EXACT,
        action_type=RuleActionType.AUTO_REPLY,
        action_value="Here are our prices.",
        priority=10,
    )
    r2 = AutomationRule(
        name="Regex Test",
        trigger_keyword=r"quote\s*#?\d+",
        match_type=RuleMatchType.REGEX,
        action_type=RuleActionType.AUTO_REPLY,
        action_value="Looking up your quote.",
        priority=20,
    )
    r3 = AutomationRule(
        name="Button Click Test",
        trigger_keyword="btn_services",
        match_type=RuleMatchType.BUTTON_CLICK,
        action_type=RuleActionType.AUTO_REPLY,
        action_value="Services list.",
        priority=30,
    )
    db_session.add_all([r1, r2, r3])
    await db_session.flush()

    # Exact match
    m1 = await match_advanced_rule(db_session, "pricing")
    assert m1 is not None and m1.name == "Exact Test"

    # Regex match
    m2 = await match_advanced_rule(db_session, "can I check quote #1234 please")
    assert m2 is not None and m2.name == "Regex Test"

    # Button click match
    m3 = await match_advanced_rule(db_session, "random text", payload_id="btn_services")
    assert m3 is not None and m3.name == "Button Click Test"


@pytest.mark.asyncio
async def test_agent_commands(db_session: AsyncSession):
    lead = Lead(harvester_institution_id="inst-1", name="Test Org")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+1234567890")
    db_session.add(contact)
    await db_session.flush()
    await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")

    conv = Conversation(contact_id=contact.id, human_takeover=True, state=ConversationState.HANDED_OFF)
    db_session.add(conv)
    await db_session.flush()

    with patch("communicator.services.whatsapp_automations.get_provider") as mock_provider:
        prov = AsyncMock()
        prov.send.return_value = SendResult(provider="meta", provider_message_id="m1")
        mock_provider.return_value = prov

        # #bot command unpauses bot
        handled = await handle_agent_commands(db_session, conv, contact, "#bot")
        assert handled is True
        assert conv.human_takeover is False
        assert conv.state == ConversationState.ENGAGED


@pytest.mark.asyncio
async def test_csat_survey_flow(db_session: AsyncSession):
    lead = Lead(harvester_institution_id="inst-2", name="Org 2")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+1987654321")
    db_session.add(contact)
    await db_session.flush()
    await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    with patch("communicator.services.whatsapp_automations.get_provider") as mock_provider:
        prov = AsyncMock()
        prov.send.return_value = SendResult(provider="meta", provider_message_id="m2")
        mock_provider.return_value = prov

        # Customer sends csat:5 button click
        handled = await handle_csat_response(db_session, conv, contact, "Great", payload_id="csat:5")
        assert handled is True
        assert conv.csat_score == 5


@pytest.mark.asyncio
async def test_handoff_and_round_robin(db_session: AsyncSession):
    lead = Lead(harvester_institution_id="inst-3", name="Org 3")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(lead_id=lead.id, channel=ChannelType.WHATSAPP, value="+1555000111")
    db_session.add(contact)
    await db_session.flush()
    await record_consent(db_session, contact.id, ConsentStatus.OPTED_IN, source="test")

    team_member = TeamMember(name="Alice Agent", email="alice@example.com")
    db_session.add(team_member)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    with patch("communicator.services.whatsapp_automations.get_provider") as mock_provider:
        prov = AsyncMock()
        prov.send.return_value = SendResult(provider="meta", provider_message_id="m3")
        mock_provider.return_value = prov

        await execute_handoff(db_session, conv, contact, note="Customer VIP")
        assert conv.human_takeover is True
        assert conv.state == ConversationState.HANDED_OFF
        assert conv.assigned_to_id == team_member.id


@pytest.mark.asyncio
async def test_automation_rules_api(test_client: AsyncClient):
    # 1. Seed defaults via API
    res = await test_client.post("/api/v1/automation-rules/seed-defaults")
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert data["installedCount"] >= 10

    # 2. Test simulator endpoint
    test_res = await test_client.post(
        "/api/v1/automation-rules/test",
        json={"text": "menu"},
    )
    assert test_res.status_code == 200
    match_data = test_res.json()
    assert match_data["matched"] is True
    assert "Menu" in match_data["rule"]["name"]

    # 3. Stats endpoint
    stats_res = await test_client.get("/api/v1/automation-rules/stats")
    assert stats_res.status_code == 200
    stats = stats_res.json()
    assert stats["totalRules"] >= 10
