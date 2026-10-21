from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services.gateway_service import (
    create_gateway,
    delete_gateway,
    get_gateway,
    list_gateways,
    mask_gateway,
    send_gateway_test_ping,
    set_default_gateway,
    test_gateway_connection,
    update_gateway,
)

router = APIRouter(prefix="/gateways", tags=["gateways"])


class CreateGatewayRequest(BaseModel):
    gateway_type: str = Field(..., description="sms | email | voice_sip")
    provider: str = Field(..., description="Provider code e.g. twilio, vonage, sendgrid, telnyx, custom_http, sip_trunk")
    name: str = Field(..., description="Human-readable name")
    credentials: dict[str, Any] = Field(default_factory=dict)
    config: dict[str, Any] = Field(default_factory=dict)
    is_default: bool = False


class UpdateGatewayRequest(BaseModel):
    name: str | None = None
    credentials: dict[str, Any] | None = None
    config: dict[str, Any] | None = None
    is_active: bool | None = None


class TestPingRequest(BaseModel):
    target: str = Field(..., description="Destination phone number, email address, or SIP URI")
    message: str = Field("Test message from Universal Gateway", description="Message content")


@router.get("")
async def get_all_gateways(
    gateway_type: str | None = Query(None, description="Filter by sms, email, or voice_sip"),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    gateways = await list_gateways(session, gateway_type=gateway_type)
    return [mask_gateway(gw) for gw in gateways]


@router.get("/providers/supported")
async def get_supported_providers() -> dict[str, Any]:
    return {
        "sms": [
            {"provider": "twilio", "name": "Twilio SMS", "fields": ["account_sid", "auth_token", "from_number"]},
            {"provider": "vonage", "name": "Vonage (Nexmo) SMS", "fields": ["api_key", "api_secret", "from"]},
            {"provider": "messagebird", "name": "MessageBird SMS", "fields": ["access_key", "originator"]},
            {"provider": "plivo", "name": "Plivo SMS", "fields": ["auth_id", "auth_token", "from_number"]},
            {"provider": "sinch", "name": "Sinch SMS", "fields": ["service_plan_id", "api_token", "from_number"]},
            {"provider": "aws_sns", "name": "AWS SNS SMS", "fields": ["aws_access_key", "aws_secret_key", "region"]},
            {"provider": "infobip", "name": "Infobip SMS", "fields": ["api_key", "base_url", "from_number"]},
            {"provider": "custom_http", "name": "Custom HTTP REST SMS Gateway", "fields": ["endpoint_url", "bearer_token", "payload_template"]},
        ],
        "email": [
            {"provider": "smtp", "name": "SMTP Server (TLS/SSL)", "fields": ["smtp_host", "smtp_port", "smtp_username", "smtp_password", "from_address"]},
            {"provider": "sendgrid", "name": "SendGrid API", "fields": ["api_key", "from_address", "from_name"]},
            {"provider": "aws_ses", "name": "AWS SES (Simple Email Service)", "fields": ["aws_access_key", "aws_secret_key", "region", "from_address"]},
            {"provider": "mailgun", "name": "Mailgun API", "fields": ["api_key", "domain", "from_address"]},
            {"provider": "postmark", "name": "Postmark API", "fields": ["api_key", "from_address"]},
            {"provider": "resend", "name": "Resend API", "fields": ["api_key", "from_address", "from_name"]},
            {"provider": "custom_webhook", "name": "Custom Outgoing Email Webhook", "fields": ["webhook_url", "auth_header", "from_address"]},
        ],
        "voice_sip": [
            {"provider": "twilio_voice", "name": "Twilio Voice & WebRTC", "fields": ["account_sid", "auth_token", "caller_id", "twiml_app_sid"]},
            {"provider": "telnyx", "name": "Telnyx SIP & Voice", "fields": ["api_key", "connection_id", "sip_domain", "caller_id"]},
            {"provider": "plivo_voice", "name": "Plivo Voice", "fields": ["auth_id", "auth_token", "caller_id"]},
            {"provider": "vonage_voice", "name": "Vonage Voice API", "fields": ["api_key", "api_secret", "application_id", "caller_id"]},
            {"provider": "sip_trunk", "name": "Generic SIP Trunk (Asterisk / FreeSWITCH)", "fields": ["sip_server", "sip_port", "sip_username", "sip_password", "transport", "caller_id"]},
            {"provider": "custom_webrtc", "name": "Custom WebRTC Gateway", "fields": ["signaling_url", "stun_turn_server", "auth_token"]},
        ],
    }


@router.get("/{gateway_id}")
async def get_single_gateway(
    gateway_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    gw = await get_gateway(session, gateway_id)
    if not gw:
        raise HTTPException(status_code=404, detail="Gateway not found")
    return mask_gateway(gw)


@router.post("", status_code=201)
async def create_new_gateway(
    payload: CreateGatewayRequest,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    gw = await create_gateway(
        session=session,
        gateway_type=payload.gateway_type,
        provider=payload.provider,
        name=payload.name,
        credentials=payload.credentials,
        config=payload.config,
        is_default=payload.is_default,
    )
    return mask_gateway(gw)


@router.put("/{gateway_id}")
async def update_existing_gateway(
    gateway_id: uuid.UUID,
    payload: UpdateGatewayRequest,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    gw = await update_gateway(
        session=session,
        gateway_id=gateway_id,
        name=payload.name,
        credentials=payload.credentials,
        config=payload.config,
        is_active=payload.is_active,
    )
    if not gw:
        raise HTTPException(status_code=404, detail="Gateway not found")
    return mask_gateway(gw)


@router.delete("/{gateway_id}")
async def delete_existing_gateway(
    gateway_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    success = await delete_gateway(session, gateway_id)
    if not success:
        raise HTTPException(status_code=404, detail="Gateway not found")
    return {"success": True, "deleted_id": str(gateway_id)}


@router.post("/{gateway_id}/test")
async def run_test_connection(
    gateway_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    return await test_gateway_connection(session, gateway_id)


@router.post("/{gateway_id}/set-default")
async def make_gateway_default(
    gateway_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    gw = await set_default_gateway(session, gateway_id)
    if not gw:
        raise HTTPException(status_code=404, detail="Gateway not found")
    return mask_gateway(gw)


@router.post("/{gateway_id}/ping")
async def send_ping(
    gateway_id: uuid.UUID,
    payload: TestPingRequest,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    return await send_gateway_test_ping(session, gateway_id, target=payload.target, message=payload.message)
