"""
Single-row IntegrationSettings read/write -- the Connections page's backing
store, and the one place that resolves "DB value if the admin set one on
the Connections page, else the .env default" for WhatsApp credentials and
the business profile the reply engine grounds itself in.
"""

from __future__ import annotations

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.config import settings
from communicator.models.integration_settings import SINGLETON_ID, IntegrationSettings


async def get_settings_row(session: AsyncSession) -> IntegrationSettings:
    row = await session.get(IntegrationSettings, SINGLETON_ID)
    if row is None:
        row = IntegrationSettings(
            id=SINGLETON_ID,
            whatsapp_provider=settings.whatsapp_provider,
            meta_access_token=settings.meta_whatsapp_access_token,
            meta_phone_number_id=settings.meta_whatsapp_phone_number_id,
            meta_app_secret=settings.meta_app_secret,
            meta_webhook_verify_token=settings.meta_webhook_verify_token,
        )
        session.add(row)
        await session.flush()
    return row


class WhatsAppConfig:
    def __init__(
        self,
        provider: str,
        access_token: str,
        phone_number_id: str,
        waba_id: str,
        app_secret: str,
        webhook_verify_token: str,
    ) -> None:
        self.provider = provider
        self.access_token = access_token
        self.phone_number_id = phone_number_id
        self.waba_id = waba_id
        self.app_secret = app_secret
        self.webhook_verify_token = webhook_verify_token


async def get_whatsapp_config(session: AsyncSession) -> WhatsAppConfig:
    """DB row wins field-by-field; an empty DB field falls back to .env."""
    row = await get_settings_row(session)
    return WhatsAppConfig(
        provider=row.whatsapp_provider or settings.whatsapp_provider,
        access_token=row.meta_access_token or settings.meta_whatsapp_access_token,
        phone_number_id=row.meta_phone_number_id or settings.meta_whatsapp_phone_number_id,
        waba_id=row.meta_waba_id,
        app_secret=row.meta_app_secret or settings.meta_app_secret,
        webhook_verify_token=row.meta_webhook_verify_token or settings.meta_webhook_verify_token,
    )
