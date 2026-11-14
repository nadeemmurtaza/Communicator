import base64

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import select

from communicator.api.app import app
from communicator.models.audit import AuditLog
from communicator.models.team_member import TeamMember
from communicator.services import sso_service


@pytest.mark.asyncio
async def test_default_sso_config_initialization_and_update(db_session):
    # 1. Initialize default SSO config
    config = await sso_service.get_or_create_sso_config(db_session)
    assert config is not None
    assert config.is_enabled is False
    assert config.provider_type == "saml"
    assert config.default_role == "agent"

    # 2. Update SSO config with domain enforcement
    updated = await sso_service.update_sso_config(
        session=db_session,
        updates={
            "is_enabled": True,
            "provider_type": "okta",
            "idp_entity_id": "http://www.okta.com/exk12345",
            "idp_sso_url": "https://bayworks.okta.com/app/bayworks/sso/saml",
            "idp_x509_cert": "-----BEGIN CERTIFICATE-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...\n-----END CERTIFICATE-----",
            "enforced_domains": ["@Bayworks.COM ", "acme.io"],
            "default_role": "admin",
        },
        actor="admin_alice",
        actor_email="alice@bayworks.com",
    )
    assert updated.is_enabled is True
    assert updated.provider_type == "okta"
    assert updated.enforced_domains == ["acme.io", "bayworks.com"]
    assert updated.default_role == "admin"

    # 3. Check SOC 2 compliance audit trail
    audit_events = (
        await db_session.scalars(
            select(AuditLog)
            .where(AuditLog.action == "sso.configured")
            .order_by(AuditLog.created_at.desc())
        )
    ).all()
    assert len(audit_events) >= 1
    assert audit_events[0].details.get("compliance_category") == "security_policy_change"


@pytest.mark.asyncio
async def test_sp_metadata_xml_and_authn_request(db_session):
    config = await sso_service.get_or_create_sso_config(db_session)
    await sso_service.update_sso_config(
        db_session,
        {
            "idp_sso_url": "https://login.microsoftonline.com/tenant-123/saml2",
            "sp_entity_id": "urn:bayworks:communicator",
        },
    )

    # 1. Metadata XML Generation
    metadata_xml = sso_service.generate_sp_metadata_xml(config, base_url="https://app.bayworks.com")
    assert "md:EntityDescriptor" in metadata_xml
    assert 'entityID="urn:bayworks:communicator"' in metadata_xml
    assert "AssertionConsumerService" in metadata_xml
    assert 'Location="https://app.bayworks.com/api/v1/sso/saml/acs"' in metadata_xml

    # 2. AuthnRequest Generation
    authn = sso_service.generate_authn_request(
        config, base_url="https://app.bayworks.com", relay_state="/inbox"
    )
    assert authn["request_id"].startswith("ID_")
    assert "SAMLRequest=" in authn["redirect_url"]
    assert "RelayState=%2Finbox" in authn["redirect_url"]


@pytest.mark.asyncio
async def test_domain_sso_enforcement(db_session):
    await sso_service.update_sso_config(
        db_session,
        {
            "is_enabled": True,
            "provider_type": "azure_ad",
            "idp_sso_url": "https://login.microsoftonline.com/test",
            "enforced_domains": ["bayworks.com"],
        },
    )

    # Enforced domain
    check_enforced = await sso_service.check_domain_sso_enforced(
        db_session, "sarah.connor@bayworks.com", base_url="https://app.bayworks.com"
    )
    assert check_enforced["sso_required"] is True
    assert check_enforced["provider_type"] == "azure_ad"
    assert check_enforced["redirect_url"] is not None
    assert "SAMLRequest=" in check_enforced["redirect_url"]

    # Non-enforced domain
    check_external = await sso_service.check_domain_sso_enforced(
        db_session, "external@gmail.com", base_url="https://app.bayworks.com"
    )
    assert check_external["sso_required"] is False
    assert check_external["redirect_url"] is None


@pytest.mark.asyncio
async def test_saml_response_processing_and_jit_provisioning(db_session):
    await sso_service.update_sso_config(
        db_session,
        {
            "is_enabled": True,
            "jit_provisioning_enabled": True,
            "default_role": "agent",
        },
    )

    sample_saml = """<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="_resp999" Version="2.0">
  <saml:Issuer>https://idp.bayworks.com</saml:Issuer>
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
  <saml:Assertion ID="_as999" Version="2.0">
    <saml:Subject>
      <saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">alex.murphy@bayworks.com</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="displayName"><saml:AttributeValue>Alex Murphy</saml:AttributeValue></saml:Attribute>
      <saml:Attribute Name="role"><saml:AttributeValue>super_admin</saml:AttributeValue></saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"""

    b64_saml = base64.b64encode(sample_saml.encode("utf-8")).decode("ascii")

    # 1. First login -> JIT provisioning
    result = await sso_service.process_saml_assertion(
        session=db_session,
        saml_response_raw=b64_saml,
        relay_state="/dashboard",
    )
    assert result["success"] is True
    assert result["provisioned"] is True
    assert result["user"]["email"] == "alex.murphy@bayworks.com"
    assert result["user"]["name"] == "Alex Murphy"
    assert result["user"]["role"] == "super_admin"

    # Verify team member exists in DB
    created_member = await db_session.scalar(
        select(TeamMember).where(TeamMember.email == "alex.murphy@bayworks.com")
    )
    assert created_member is not None
    assert created_member.name == "Alex Murphy"

    # 2. Second login -> Account exists, should authenticate without re-provisioning
    result_second = await sso_service.process_saml_assertion(
        session=db_session,
        saml_response_raw=b64_saml,
        relay_state="/dashboard",
    )
    assert result_second["success"] is True
    assert result_second["provisioned"] is False

    # 3. Verify audit log has entries for provisioning and login
    logs = (
        await db_session.scalars(
            select(AuditLog)
            .where(AuditLog.action.in_(["sso.user_provisioned", "sso.login_success"]))
            .order_by(AuditLog.created_at.desc())
        )
    ).all()
    actions = [log.action for log in logs]
    assert "sso.user_provisioned" in actions
    assert "sso.login_success" in actions


@pytest.mark.asyncio
async def test_sso_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. GET /api/v1/sso/config
        r_get = await client.get("/api/v1/sso/config")
        assert r_get.status_code == 200
        cfg = r_get.json()["sso_config"]
        assert "is_enabled" in cfg

        # 2. PUT /api/v1/sso/config
        r_put = await client.put(
            "/api/v1/sso/config",
            json={
                "is_enabled": True,
                "provider_type": "google_workspace",
                "idp_entity_id": "https://accounts.google.com/o/saml2?idpid=C012345",
                "idp_sso_url": "https://accounts.google.com/o/saml2/initsso?idpid=C012345&spid=6789",
                "enforced_domains": ["google-enterprise.com"],
            },
        )
        assert r_put.status_code == 200
        updated_cfg = r_put.json()["sso_config"]
        assert updated_cfg["provider_type"] == "google_workspace"
        assert any(d == "google-enterprise.com" for d in updated_cfg["enforced_domains"])

        # 3. GET /api/v1/sso/metadata.xml
        r_meta = await client.get("/api/v1/sso/metadata.xml")
        assert r_meta.status_code == 200
        assert "application/xml" in r_meta.headers["content-type"]
        assert "EntityDescriptor" in r_meta.text

        # 4. POST /api/v1/sso/check
        r_check = await client.post(
            "/api/v1/sso/check",
            json={"email": "dan@google-enterprise.com"},
        )
        assert r_check.status_code == 200
        check_res = r_check.json()
        assert check_res["sso_required"] is True
        assert check_res["provider_type"] == "google_workspace"

        # 5. POST /api/v1/sso/test
        r_test = await client.post("/api/v1/sso/test", json={})
        assert r_test.status_code == 200
        test_res = r_test.json()
        assert "status" in test_res
        assert len(test_res["checks"]) >= 5

        # 6. POST /api/v1/sso/saml/acs (JSON endpoint)
        raw_xml = """<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="_api999" Version="2.0">
  <saml:Issuer>https://accounts.google.com</saml:Issuer>
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
  <saml:Assertion ID="_asApi999" Version="2.0">
    <saml:Subject>
      <saml:NameID>api.tester@google-enterprise.com</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="name"><saml:AttributeValue>API Tester</saml:AttributeValue></saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"""

        r_acs = await client.post(
            "/api/v1/sso/saml/acs",
            json={"saml_response": raw_xml, "relay_state": "/team"},
        )
        assert r_acs.status_code == 200
        acs_res = r_acs.json()
        assert acs_res["success"] is True
        assert acs_res["user"]["email"] == "api.tester@google-enterprise.com"
        assert acs_res["relay_state"] == "/team"
