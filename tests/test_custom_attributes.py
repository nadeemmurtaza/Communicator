import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead
from communicator.services import custom_attribute_service


@pytest.mark.asyncio
async def test_default_custom_attributes_initialization(db_session):
    attrs = await custom_attribute_service.list_attribute_definitions(db_session)
    assert len(attrs) >= 8

    contact_attrs = [a for a in attrs if a.attribute_model == "contact_attribute"]
    conv_attrs = [a for a in attrs if a.attribute_model == "conversation_attribute"]

    assert len(contact_attrs) >= 4
    assert len(conv_attrs) >= 3

    slugs = [a.name for a in attrs]
    assert "account_tier" in slugs
    assert "company_size" in slugs
    assert "issue_category" in slugs
    assert "escalation_level" in slugs


@pytest.mark.asyncio
async def test_create_and_update_attribute_definition(db_session):
    # 1. Create attribute
    created = await custom_attribute_service.create_attribute_definition(
        session=db_session,
        data={
            "name": "nps_score",
            "display_name": "Net Promoter Score",
            "description": "Customer satisfaction score from 0 to 10",
            "attribute_model": "contact_attribute",
            "attribute_type": "number",
            "default_value": "10",
        },
        actor="admin_dave",
    )
    assert created.id is not None
    assert created.name == "nps_score"
    assert created.attribute_type == "number"

    # 2. Update attribute
    updated = await custom_attribute_service.update_attribute_definition(
        session=db_session,
        id_or_name="nps_score",
        updates={"display_name": "NPS Customer Score", "is_required": True},
        actor="admin_dave",
    )
    assert updated.display_name == "NPS Customer Score"
    assert updated.is_required is True

    # 3. Duplicate prevention
    with pytest.raises(ValueError, match="already exists"):
        await custom_attribute_service.create_attribute_definition(
            session=db_session,
            data={
                "name": "nps_score",
                "display_name": "Duplicate NPS",
                "attribute_model": "contact_attribute",
                "attribute_type": "number",
            },
        )


@pytest.mark.asyncio
async def test_attribute_value_validation(db_session):
    # Setup test attributes of various types
    tier_def = await custom_attribute_service.get_attribute_definition(
        db_session, "account_tier", "contact_attribute"
    )
    assert tier_def is not None

    # Valid list option
    ok, val, err = custom_attribute_service.validate_attribute_value(tier_def, "Enterprise")
    assert ok is True
    assert val == "Enterprise"
    assert err is None

    # Invalid list option
    ok, val, err = custom_attribute_service.validate_attribute_value(tier_def, "SuperDuper")
    assert ok is False
    assert "not in allowed options" in err

    # Number / Currency validation
    rev_def = await custom_attribute_service.get_attribute_definition(
        db_session, "annual_revenue", "contact_attribute"
    )
    ok, val, err = custom_attribute_service.validate_attribute_value(rev_def, "$50,000.00")
    assert ok is True
    assert val == 50000.0

    # Boolean validation
    vip_def = await custom_attribute_service.get_attribute_definition(
        db_session, "is_vip", "contact_attribute"
    )
    ok, val, err = custom_attribute_service.validate_attribute_value(vip_def, "true")
    assert ok is True
    assert val is True

    # Date validation
    renewal_def = await custom_attribute_service.get_attribute_definition(
        db_session, "renewal_date", "contact_attribute"
    )
    ok, val, err = custom_attribute_service.validate_attribute_value(renewal_def, "2026-12-31")
    assert ok is True
    assert val == "2026-12-31"

    ok, val, err = custom_attribute_service.validate_attribute_value(renewal_def, "invalid-date")
    assert ok is False


@pytest.mark.asyncio
async def test_set_contact_and_conversation_attributes(db_session):
    lead = Lead(name="Ellen Ripley", harvester_institution_id="inst_ripley_99")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.EMAIL,
        value="ripley@weyland-yutani.corp",
        person_name="Ellen Ripley",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.commit()

    # Set contact custom attributes
    c_fields = await custom_attribute_service.set_contact_custom_attributes(
        session=db_session,
        contact_id=contact.id,
        attributes={
            "account_tier": "Enterprise",
            "is_vip": True,
            "annual_revenue": 120000,
        },
    )
    assert c_fields["account_tier"] == "Enterprise"
    assert c_fields["is_vip"] is True
    assert c_fields["annual_revenue"] == 120000

    # Set conversation custom attributes
    conv_fields = await custom_attribute_service.set_conversation_custom_attributes(
        session=db_session,
        conversation_id=conv.id,
        attributes={
            "issue_category": "Technical Support",
            "escalation_level": "Tier 2",
            "jira_key": "XENO-426",
        },
    )
    assert conv_fields["issue_category"] == "Technical Support"
    assert conv_fields["escalation_level"] == "Tier 2"
    assert conv_fields["jira_key"] == "XENO-426"


@pytest.mark.asyncio
async def test_custom_attributes_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. GET /api/v1/custom-attributes
        r_list = await client.get("/api/v1/custom-attributes")
        assert r_list.status_code == 200
        attrs = r_list.json()
        assert len(attrs) >= 8

        # Filter by model
        r_conv_only = await client.get(
            "/api/v1/custom-attributes?attribute_model=conversation_attribute"
        )
        assert r_conv_only.status_code == 200
        assert all(a["attribute_model"] == "conversation_attribute" for a in r_conv_only.json())

        # 2. POST /api/v1/custom-attributes
        r_create = await client.post(
            "/api/v1/custom-attributes",
            json={
                "name": "churn_risk",
                "display_name": "Churn Risk Assessment",
                "attribute_model": "contact_attribute",
                "attribute_type": "list",
                "options": ["Low", "Moderate", "Critical"],
                "default_value": "Low",
            },
        )
        assert r_create.status_code == 200
        created = r_create.json()
        assert created["name"] == "churn_risk"

        # 3. PUT /api/v1/custom-attributes/{id}
        r_put = await client.put(
            f"/api/v1/custom-attributes/{created['id']}",
            json={"display_name": "Account Churn Risk Level"},
        )
        assert r_put.status_code == 200
        assert r_put.json()["display_name"] == "Account Churn Risk Level"

        # 4. DELETE /api/v1/custom-attributes/{id}
        r_del = await client.delete(f"/api/v1/custom-attributes/{created['id']}")
        assert r_del.status_code == 200
        assert r_del.json()["success"] is True
