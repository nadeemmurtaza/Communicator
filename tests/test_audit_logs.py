import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.services import audit_service


@pytest.mark.asyncio
async def test_record_and_search_audit_events(db_session):
    test_id = str(uuid.uuid4())
    # 1. Record several audit events
    log1 = await audit_service.record_audit_event(
        db_session,
        action="conversation.assigned",
        entity_type="conversation",
        entity_id=test_id,
        actor="alex.turner",
        actor_email="alex@bayworks.com",
        actor_role="lead_agent",
        severity="info",
        compliance_category="access_control",
        description="Assigned conversation to Tier 2 support team",
        ip_address="198.51.100.12",
    )
    assert log1.id is not None

    log2 = await audit_service.record_audit_event(
        db_session,
        action="auth.login_failed",
        entity_type="auth",
        actor="unknown_user",
        actor_email="attacker@external.org",
        severity="warning",
        compliance_category="security_event",
        description="Failed login attempt with invalid 2FA token",
        ip_address="203.0.113.88",
    )
    assert log2.id is not None

    log3 = await audit_service.record_audit_event(
        db_session,
        action="customer_data.exported",
        entity_type="contact",
        actor="sarah.connor",
        actor_email="sarah@bayworks.com",
        severity="notice",
        compliance_category="export_activity",
        description="Exported 500 contact records for compliance audit",
        ip_address="198.51.100.15",
    )
    assert log3.id is not None

    # 2. Search all
    res_all = await audit_service.search_audit_logs(db_session, page=1, page_size=20)
    assert res_all["total"] >= 3
    assert len(res_all["items"]) >= 3

    # 3. Search with action filter
    res_auth = await audit_service.search_audit_logs(db_session, action="auth.login_failed")
    assert any(item["action"] == "auth.login_failed" for item in res_auth["items"])

    # 4. Search with actor filter
    res_actor = await audit_service.search_audit_logs(db_session, actor="alex.turner")
    assert any(item["actor"] == "alex.turner" for item in res_actor["items"])

    # 5. Search with severity filter
    res_sev = await audit_service.search_audit_logs(db_session, severity="warning")
    assert any(item["severity"] == "warning" for item in res_sev["items"])


@pytest.mark.asyncio
async def test_get_audit_statistics(db_session):
    await audit_service.record_audit_event(
        db_session,
        action="macro.executed",
        entity_type="macro",
        actor="agent.smith",
        severity="info",
        compliance_category="ai_governance",
    )

    stats = await audit_service.get_audit_statistics(db_session)
    assert stats["total_events"] >= 1
    assert "severity_distribution" in stats
    assert "category_distribution" in stats
    assert "top_actors" in stats
    assert stats["soc2_compliance_status"] == "MONITORED_AND_AUDITED"


@pytest.mark.asyncio
async def test_export_audit_trail_csv_and_json(db_session):
    await audit_service.record_audit_event(
        db_session,
        action="api_key.rotated",
        entity_type="system",
        actor="secops_admin",
        actor_email="secops@bayworks.com",
        severity="warning",
        compliance_category="access_control",
        description="Rotated production Twilio API auth token",
        ip_address="10.0.0.1",
    )

    # 1. Test CSV Export
    csv_text = await audit_service.export_audit_trail_csv(db_session, limit=100)
    assert "Event ID" in csv_text
    assert "api_key.rotated" in csv_text
    assert "secops_admin" in csv_text
    assert "access_control" in csv_text

    # 2. Test JSON Export
    json_data = await audit_service.export_audit_trail_json(db_session, limit=100)
    assert "export_metadata" in json_data
    assert "records" in json_data
    assert json_data["export_metadata"]["standard"] == "SOC 2 Type II / ISO 27001 Activity Trail"
    assert any(r["action"] == "api_key.rotated" for r in json_data["records"])


@pytest.mark.asyncio
async def test_audit_logs_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. POST /api/v1/audit-logs
        post_res = await client.post(
            "/api/v1/audit-logs",
            json={
                "action": "sla_policy.created",
                "entity_type": "sla_policy",
                "actor": "admin_claire",
                "actor_email": "claire@bayworks.com",
                "severity": "info",
                "compliance_category": "configuration_change",
                "description": "Created new Enterprise SLA Policy (15 min response target)",
                "details": {"sla_minutes": 15, "tier": "enterprise"},
            },
        )
        assert post_res.status_code == 200
        post_data = post_res.json()
        assert post_data["success"] is True
        assert "id" in post_data

        # 2. GET /api/v1/audit-logs
        get_res = await client.get("/api/v1/audit-logs?action=sla_policy.created")
        assert get_res.status_code == 200
        data = get_res.json()
        assert "items" in data
        assert any(item["action"] == "sla_policy.created" for item in data["items"])

        # 3. GET /api/v1/audit-logs/stats
        stats_res = await client.get("/api/v1/audit-logs/stats")
        assert stats_res.status_code == 200
        stats = stats_res.json()
        assert stats["total_events"] >= 1

        # 4. GET /api/v1/audit-logs/export (CSV)
        csv_res = await client.get("/api/v1/audit-logs/export?format=csv")
        assert csv_res.status_code == 200
        assert "text/csv" in csv_res.headers["content-type"]
        assert "Event ID" in csv_res.text
        assert "sla_policy.created" in csv_res.text

        # 5. GET /api/v1/audit-logs/export (JSON)
        json_res = await client.get("/api/v1/audit-logs/export?format=json")
        assert json_res.status_code == 200
        json_data = json_res.json()
        assert "export_metadata" in json_data
        assert "records" in json_data
