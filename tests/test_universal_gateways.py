import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.channels.email import UniversalEmailProvider
from communicator.channels.sms import UniversalSMSProvider
from communicator.channels.voice import VoiceProvider
from communicator.models.contact import ChannelType, Contact
from communicator.models.gateway_connection import (
    ChannelGatewayConnection,
    EmailProvider,
    GatewayType,
    SMSProvider,
    VoiceSIPProvider,
)
from communicator.services import gateway_service


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_seed_and_list_gateways(db_session):
    await gateway_service.seed_default_gateways_if_empty(db_session)
    sms_gateways = await gateway_service.list_gateways(db_session, gateway_type="sms")
    email_gateways = await gateway_service.list_gateways(db_session, gateway_type="email")
    voice_gateways = await gateway_service.list_gateways(db_session, gateway_type="voice_sip")

    assert len(sms_gateways) >= 3
    assert any(gw.provider == "twilio" for gw in sms_gateways)
    assert any(gw.provider == "vonage" for gw in sms_gateways)

    assert len(email_gateways) >= 3
    assert any(gw.provider == "smtp" for gw in email_gateways)
    assert any(gw.provider == "sendgrid" for gw in email_gateways)

    assert len(voice_gateways) >= 2
    assert any(gw.provider == "twilio_voice" for gw in voice_gateways)
    assert any(gw.provider == "sip_trunk" for gw in voice_gateways)


@pytest.mark.asyncio
async def test_create_and_test_custom_sms_gateway(db_session):
    gw = await gateway_service.create_gateway(
        session=db_session,
        gateway_type=GatewayType.SMS.value,
        provider=SMSProvider.CUSTOM_HTTP.value,
        name="FinTech Internal SMS Gateway",
        credentials={"bearer_token": "secret_bearer_token_12345"},
        config={
            "endpoint_url": "https://api.fintech.internal/v1/sms",
            "headers": {"X-Service": "bayworks-crm"},
            "payload_template": '{"to": "{{to}}", "text": "{{body}}"}',
        },
        is_default=False,
    )
    assert gw.name == "FinTech Internal SMS Gateway"
    assert gw.test_status == "untested"

    # Run connection test
    test_res = await gateway_service.test_gateway_connection(db_session, gw.id)
    assert test_res["success"] is True
    assert test_res["test_status"] == "connected"
    assert test_res["latency_ms"] > 0


@pytest.mark.asyncio
async def test_set_default_gateway(db_session):
    gw1 = await gateway_service.create_gateway(
        session=db_session,
        gateway_type=GatewayType.EMAIL.value,
        provider=EmailProvider.SENDGRID.value,
        name="SendGrid Secondary",
        credentials={"api_key": "SG.test_key_881928374"},
        config={"from_address": "test@bayworks.co"},
        is_default=False,
    )

    updated = await gateway_service.set_default_gateway(db_session, gw1.id)
    assert updated is not None
    assert updated.is_default is True

    active = await gateway_service.get_active_gateway(db_session, "email")
    assert active is not None
    assert active.id == gw1.id


@pytest.mark.asyncio
async def test_universal_sms_provider_dispatch():
    contact = Contact(id=uuid.uuid4(), lead_id=uuid.uuid4(), channel=ChannelType.SMS, value="+14155552671")

    # 1. Custom HTTP Provider
    custom_prov = UniversalSMSProvider(
        provider="custom_http",
        credentials={"bearer_token": "token123"},
        config={"endpoint_url": "https://sms.example.com/api/send"},
    )
    res1 = await custom_prov.send(contact, "Hello via custom gateway!")
    assert res1.provider == "custom_http"
    assert res1.raw_response["simulated"] is True

    # 2. Vonage Provider
    vonage_prov = UniversalSMSProvider(provider="vonage")
    res2 = await vonage_prov.send(contact, "Hello via Vonage!")
    assert res2.provider == "vonage"
    assert res2.raw_response["simulated"] is True

    # 3. MessageBird Provider
    mb_prov = UniversalSMSProvider(provider="messagebird")
    res3 = await mb_prov.send(contact, "Hello via MessageBird!")
    assert res3.provider == "messagebird"


@pytest.mark.asyncio
async def test_universal_email_provider_dispatch():
    contact = Contact(id=uuid.uuid4(), lead_id=uuid.uuid4(), channel=ChannelType.EMAIL, value="client@example.com")

    # 1. SendGrid
    sg_prov = UniversalEmailProvider(provider="sendgrid")
    res_sg = await sg_prov.send(contact, "Email body", subject="Testing SendGrid")
    assert res_sg.provider == "sendgrid"

    # 2. AWS SES
    ses_prov = UniversalEmailProvider(provider="aws_ses")
    res_ses = await ses_prov.send(contact, "Email body", subject="Testing SES")
    assert res_ses.provider == "aws_ses"

    # 3. Resend
    resend_prov = UniversalEmailProvider(provider="resend")
    res_re = await resend_prov.send(contact, "Email body", subject="Testing Resend")
    assert res_re.provider == "resend"


@pytest.mark.asyncio
async def test_universal_voice_sip_provider_dispatch():
    contact = Contact(id=uuid.uuid4(), lead_id=uuid.uuid4(), channel=ChannelType.VOICE, value="+14155559090")

    # 1. Telnyx
    telnyx_prov = VoiceProvider(provider="telnyx")
    res_telnyx = await telnyx_prov.send(contact, "Voice alert message")
    assert res_telnyx.provider == "telnyx"
    assert "call_control_id" in res_telnyx.raw_response

    # 2. Generic SIP Trunk
    sip_prov = VoiceProvider(provider="sip_trunk", config={"sip_server": "pbx.mycompany.com", "sip_port": 5060})
    res_sip = await sip_prov.send(contact, "SIP trunk announcement")
    assert res_sip.provider == "sip_trunk"
    assert "sip_uri" in res_sip.raw_response
    assert "pbx.mycompany.com" in res_sip.raw_response["sip_uri"]


@pytest.mark.asyncio
async def test_gateway_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. List gateways
        resp = await client.get("/api/v1/gateways")
        assert resp.status_code == 200
        gateways = resp.json()
        assert len(gateways) >= 5

        # Verify credential masking
        for gw in gateways:
            for k, v in gw.get("credentials", {}).items():
                if isinstance(v, str) and len(v) > 0:
                    assert "••••" in v

        # 2. Supported providers schema
        resp_supp = await client.get("/api/v1/gateways/providers/supported")
        assert resp_supp.status_code == 200
        data_supp = resp_supp.json()
        assert "sms" in data_supp
        assert "email" in data_supp
        assert "voice_sip" in data_supp

        # 3. Create a new gateway via API
        create_payload = {
            "gateway_type": "sms",
            "provider": "plivo",
            "name": "Plivo Backup SMS",
            "credentials": {"auth_id": "PLIVO_ID_999", "auth_token": "plivo_secret_pass"},
            "config": {"from_number": "+14155550188"},
            "is_default": False,
        }
        resp_create = await client.post("/api/v1/gateways", json=create_payload)
        assert resp_create.status_code == 201
        created_gw = resp_create.json()
        assert created_gw["name"] == "Plivo Backup SMS"
        assert created_gw["provider"] == "plivo"

        gw_id = created_gw["id"]

        # 4. Test connection via API
        resp_test = await client.post(f"/api/v1/gateways/{gw_id}/test")
        assert resp_test.status_code == 200
        test_info = resp_test.json()
        assert test_info["success"] is True
        assert test_info["test_status"] == "connected"

        # 5. Set default via API
        resp_def = await client.post(f"/api/v1/gateways/{gw_id}/set-default")
        assert resp_def.status_code == 200
        assert resp_def.json()["is_default"] is True

        # 6. Test ping via API
        resp_ping = await client.post(f"/api/v1/gateways/{gw_id}/ping", json={"target": "+14155550199", "message": "Hi test"})
        assert resp_ping.status_code == 200
        assert resp_ping.json()["success"] is True

        # 7. Delete via API
        resp_del = await client.delete(f"/api/v1/gateways/{gw_id}")
        assert resp_del.status_code == 200
        assert resp_del.json()["success"] is True
