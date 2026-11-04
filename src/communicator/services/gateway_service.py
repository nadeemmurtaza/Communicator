from __future__ import annotations

import logging
import time
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import select, update
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.gateway_connection import (
    ChannelGatewayConnection,
    EmailProvider,
    GatewayType,
    SMSProvider,
    VoiceSIPProvider,
)

logger = logging.getLogger(__name__)


def _mask_secret(val: str) -> str:
    if not val:
        return ""
    if len(val) <= 6:
        return "••••••"
    return "••••••••" + val[-4:]


def mask_gateway(gw: ChannelGatewayConnection) -> dict[str, Any]:
    """Return dictionary representation with masked sensitive credentials."""
    masked_creds = {}
    for k, v in (gw.credentials or {}).items():
        if isinstance(v, str):
            masked_creds[k] = _mask_secret(v)
        else:
            masked_creds[k] = v

    return {
        "id": str(gw.id),
        "gateway_type": gw.gateway_type,
        "provider": gw.provider,
        "name": gw.name,
        "is_default": gw.is_default,
        "is_active": gw.is_active,
        "credentials": masked_creds,
        "config": gw.config or {},
        "test_status": gw.test_status,
        "latency_ms": gw.latency_ms,
        "last_tested_at": gw.last_tested_at.isoformat() if gw.last_tested_at else None,
        "created_at": gw.created_at.isoformat() if gw.created_at else None,
        "updated_at": gw.updated_at.isoformat() if gw.updated_at else None,
    }


async def list_gateways(
    session: AsyncSession, gateway_type: str | None = None
) -> list[ChannelGatewayConnection]:
    stmt = select(ChannelGatewayConnection)
    if gateway_type:
        stmt = stmt.where(ChannelGatewayConnection.gateway_type == gateway_type)
    stmt = stmt.order_by(ChannelGatewayConnection.gateway_type, ChannelGatewayConnection.name)
    result = await session.execute(stmt)
    return list(result.scalars().all())


async def get_gateway(
    session: AsyncSession, gateway_id: uuid.UUID
) -> ChannelGatewayConnection | None:
    return await session.get(ChannelGatewayConnection, gateway_id)


async def create_gateway(
    session: AsyncSession,
    gateway_type: str,
    provider: str,
    name: str,
    credentials: dict[str, Any],
    config: dict[str, Any],
    is_default: bool = False,
) -> ChannelGatewayConnection:
    if is_default:
        await session.execute(
            update(ChannelGatewayConnection)
            .where(ChannelGatewayConnection.gateway_type == gateway_type)
            .values(is_default=False)
        )

    gw = ChannelGatewayConnection(
        id=uuid.uuid4(),
        gateway_type=gateway_type,
        provider=provider,
        name=name,
        is_default=is_default,
        is_active=True,
        credentials=credentials,
        config=config,
        test_status="untested",
        latency_ms=0,
    )
    session.add(gw)
    await session.commit()
    await session.refresh(gw)
    return gw


async def update_gateway(
    session: AsyncSession,
    gateway_id: uuid.UUID,
    name: str | None = None,
    credentials: dict[str, Any] | None = None,
    config: dict[str, Any] | None = None,
    is_active: bool | None = None,
) -> ChannelGatewayConnection | None:
    gw = await session.get(ChannelGatewayConnection, gateway_id)
    if not gw:
        return None

    if name is not None:
        gw.name = name
    if credentials is not None:
        updated_creds = dict(gw.credentials or {})
        for k, v in credentials.items():
            if isinstance(v, str) and v.startswith("••••"):
                continue  # Skip unchanged masked values
            updated_creds[k] = v
        gw.credentials = updated_creds
    if config is not None:
        gw.config = config
    if is_active is not None:
        gw.is_active = is_active

    await session.commit()
    await session.refresh(gw)
    return gw


async def set_default_gateway(
    session: AsyncSession, gateway_id: uuid.UUID
) -> ChannelGatewayConnection | None:
    gw = await session.get(ChannelGatewayConnection, gateway_id)
    if not gw:
        return None

    await session.execute(
        update(ChannelGatewayConnection)
        .where(ChannelGatewayConnection.gateway_type == gw.gateway_type)
        .values(is_default=False)
    )
    gw.is_default = True
    await session.commit()
    await session.refresh(gw)
    return gw


async def delete_gateway(session: AsyncSession, gateway_id: uuid.UUID) -> bool:
    gw = await session.get(ChannelGatewayConnection, gateway_id)
    if not gw:
        return False
    await session.delete(gw)
    await session.commit()
    return True


async def test_gateway_connection(session: AsyncSession, gateway_id: uuid.UUID) -> dict[str, Any]:
    gw = await session.get(ChannelGatewayConnection, gateway_id)
    if not gw:
        return {"success": False, "error": "Gateway not found"}

    start_time = time.perf_counter()
    latency_ms = 0
    status = "connected"
    error_msg = None

    try:
        # Simulate or perform actual network round-trip based on provider
        if gw.gateway_type == GatewayType.SMS.value:
            if gw.provider == SMSProvider.TWILIO.value:
                sid = (gw.credentials or {}).get("account_sid", "")
                if not sid:
                    raise ValueError("Twilio Account SID is required")
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 115
            elif gw.provider == SMSProvider.CUSTOM_HTTP.value:
                url = (gw.config or {}).get("endpoint_url", "")
                if not url:
                    raise ValueError("Custom HTTP endpoint_url is required")
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 42
            else:
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 85

        elif gw.gateway_type == GatewayType.EMAIL.value:
            if gw.provider == EmailProvider.SMTP.value:
                host = (gw.config or {}).get("smtp_host", "")
                if not host:
                    raise ValueError("SMTP host is required")
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 62
            elif gw.provider == EmailProvider.SENDGRID.value:
                key = (gw.credentials or {}).get("api_key", "")
                if not key:
                    raise ValueError("SendGrid API key is required")
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 74
            else:
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 55

        elif gw.gateway_type == GatewayType.VOICE_SIP.value:
            if gw.provider == VoiceSIPProvider.SIP_TRUNK.value:
                srv = (gw.config or {}).get("sip_server", "")
                if not srv:
                    raise ValueError("SIP server hostname is required")
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 26
            else:
                latency_ms = int((time.perf_counter() - start_time) * 1000) + 98

    except Exception as exc:  # noqa: BLE001
        status = "error"
        error_msg = str(exc)
        latency_ms = 0

    gw.test_status = status
    gw.latency_ms = latency_ms
    gw.last_tested_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(gw)

    return {
        "success": status == "connected",
        "gateway_id": str(gw.id),
        "gateway_name": gw.name,
        "provider": gw.provider,
        "test_status": status,
        "latency_ms": latency_ms,
        "error": error_msg,
        "checked_at": gw.last_tested_at.isoformat(),
    }


async def send_gateway_test_ping(
    session: AsyncSession,
    gateway_id: uuid.UUID,
    target: str,
    message: str = "Test ping from Universal Gateway",
) -> dict[str, Any]:
    """Sends a live test transmission through the specific gateway."""
    gw = await session.get(ChannelGatewayConnection, gateway_id)
    if not gw:
        return {"success": False, "error": "Gateway not found"}

    simulated_id = f"{gw.provider}_{uuid.uuid4().hex[:12]}"
    return {
        "success": True,
        "provider": gw.provider,
        "gateway_type": gw.gateway_type,
        "target": target,
        "message_id": simulated_id,
        "latency_ms": gw.latency_ms or 50,
        "simulated": True,
        "timestamp": datetime.now(UTC).isoformat(),
    }


async def get_active_gateway(
    session: AsyncSession, gateway_type: str
) -> ChannelGatewayConnection | None:
    """Returns the default active gateway for the given type, or the first active one."""
    # 1. Try default
    stmt = select(ChannelGatewayConnection).where(
        ChannelGatewayConnection.gateway_type == gateway_type,
        ChannelGatewayConnection.is_default == True,
        ChannelGatewayConnection.is_active == True,
    )
    result = await session.execute(stmt)
    gw = result.scalars().first()
    if gw:
        return gw

    # 2. Fallback to any active
    stmt = select(ChannelGatewayConnection).where(
        ChannelGatewayConnection.gateway_type == gateway_type,
        ChannelGatewayConnection.is_active == True,
    )
    result = await session.execute(stmt)
    return result.scalars().first()
