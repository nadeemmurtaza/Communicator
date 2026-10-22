import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.services import prechat_service


@pytest.mark.asyncio
async def test_get_and_update_prechat_config(db_session):
    # 1. Retrieve defaults
    cfg = await prechat_service.get_prechat_config(db_session)
    assert cfg["enabled"] is True
    assert len(cfg["fields"]) >= 3
    assert any(f["name"] == "fullName" for f in cfg["fields"])
    assert any(f["name"] == "email" for f in cfg["fields"])
    assert len(cfg["routing_rules"]) >= 1

    # 2. Update configuration
    updates = {
        "title": "Tell Us How We Can Help",
        "description": "Customized intake prompt for new visitors",
        "submit_button_text": "Connect with an Expert",
        "fields": [
            {
                "id": "f_1",
                "name": "fullName",
                "label": "Full Name",
                "type": "text",
                "required": True,
                "order": 1,
            },
            {
                "id": "f_2",
                "name": "email",
                "label": "Work Email",
                "type": "email",
                "required": True,
                "order": 2,
            },
            {
                "id": "f_3",
                "name": "teamSize",
                "label": "Team Size",
                "type": "select",
                "required": True,
                "options": ["1-10", "11-50", "50+"],
                "order": 3,
            },
        ],
        "routing_rules": [
            {
                "id": "rule_large_team",
                "field": "teamSize",
                "operator": "equals",
                "value": "50+",
                "assign_priority": "urgent",
                "assign_tag": "enterprise_deal",
                "team_target": "Enterprise Sales",
            }
        ],
    }

    updated = await prechat_service.update_prechat_config(db_session, updates)
    assert updated["title"] == "Tell Us How We Can Help"
    assert updated["submit_button_text"] == "Connect with an Expert"
    assert len(updated["fields"]) == 3
    assert len(updated["routing_rules"]) == 1
    assert updated["routing_rules"][0]["team_target"] == "Enterprise Sales"


@pytest.mark.asyncio
async def test_validation_and_routing_logic(db_session):
    fields = [
        {"name": "fullName", "label": "Full Name", "type": "text", "required": True},
        {"name": "email", "label": "Email", "type": "email", "required": True},
        {"name": "plan", "label": "Plan", "type": "select", "options": ["Basic", "Pro"], "required": False},
    ]

    # Test missing required field
    errs = prechat_service.validate_submission_answers(fields, {"fullName": "John", "email": "invalid-email"})
    assert any("email" in e.lower() for e in errs)

    # Test invalid select option
    errs2 = prechat_service.validate_submission_answers(
        fields, {"fullName": "John", "email": "john@example.com", "plan": "EnterpriseUnknown"}
    )
    assert any("allowed choices" in e.lower() for e in errs2)

    # Test valid submission
    errs3 = prechat_service.validate_submission_answers(
        fields, {"fullName": "John Doe", "email": "john@example.com", "plan": "Pro"}
    )
    assert len(errs3) == 0

    # Test routing evaluation
    rules = [
        {
            "id": "r1",
            "field": "plan",
            "operator": "equals",
            "value": "Pro",
            "assign_priority": "high",
            "assign_tag": "pro_tier",
            "team_target": "Growth Team",
        }
    ]
    routing = prechat_service.evaluate_routing_rules(rules, {"plan": "Pro"})
    assert routing["priority"] == "high"
    assert "pro_tier" in routing["tags"]
    assert routing["team_target"] == "Growth Team"


@pytest.mark.asyncio
async def test_submit_prechat_intake_creates_contact_and_conversation(db_session):
    cfg = await prechat_service.get_prechat_config(db_session)
    token = cfg["website_token"]

    test_email = f"visitor_{uuid.uuid4().hex[:6]}@bayworks-test.com"
    answers = {
        "fullName": "Marcus Brody",
        "email": test_email,
        "inquiryType": "Billing & Invoices",
        "company": "Brody Museum of Antiquities",
        "message": "We have an invoice discrepancy on invoice #90210.",
    }

    result = await prechat_service.submit_prechat_intake(
        db_session,
        website_token=token,
        answers=answers,
        client_ip="192.168.1.50",
    )

    assert result["success"] is True
    assert result["contact_id"] is not None
    assert result["conversation_id"] is not None
    assert result["visitor_name"] == "Marcus Brody"
    assert result["visitor_email"] == test_email
    assert result["initial_message_id"] is not None
    assert result["routing_applied"]["priority"] == "high"
    assert "billing" in result["routing_applied"]["tags"]


@pytest.mark.asyncio
async def test_prechat_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Admin get config
        r_get = await client.get("/api/v1/inboxes/channels/live-chat/prechat")
        assert r_get.status_code == 200
        data = r_get.json()
        assert "fields" in data
        token = data.get("website_token", "test_token")

        # 2. Admin test intake simulation
        r_test = await client.post(
            "/api/v1/inboxes/channels/live-chat/prechat/test",
            json={
                "answers": {
                    "fullName": "Dr. Henry Jones",
                    "email": "indy@archaeology.edu",
                    "inquiryType": "Technical Support",
                }
            },
        )
        assert r_test.status_code == 200
        test_res = r_test.json()
        assert test_res["valid"] is True
        assert "routing_result" in test_res

        # 3. Public widget prechat schema
        r_pub = await client.get(f"/api/v1/widget/prechat?token={token}")
        assert r_pub.status_code == 200
        pub_data = r_pub.json()
        assert "fields" in pub_data

        # 4. Public widget prechat submit (validation failure)
        r_sub_fail = await client.post(
            "/api/v1/widget/prechat/submit",
            json={
                "website_token": token,
                "answers": {
                    "fullName": "",
                    "email": "not-an-email",
                },
            },
        )
        assert r_sub_fail.status_code == 422

        # 5. Public widget prechat submit (success)
        test_email = f"indy_{uuid.uuid4().hex[:6]}@archaeology.edu"
        r_sub_ok = await client.post(
            "/api/v1/widget/prechat/submit",
            json={
                "website_token": token,
                "answers": {
                    "fullName": "Henry Walton Jones Jr.",
                    "email": test_email,
                    "inquiryType": "Sales & Pricing",
                    "message": "Interested in enterprise expedition communications.",
                },
            },
        )
        assert r_sub_ok.status_code == 200
        sub_data = r_sub_ok.json()
        assert sub_data["success"] is True
        assert sub_data["conversation_id"] is not None
        assert sub_data["visitor_email"] == test_email
