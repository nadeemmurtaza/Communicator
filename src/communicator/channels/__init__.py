from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels.base import ChannelProvider, SendResult
from communicator.channels.email import PostmarkEmailProvider
from communicator.channels.sms import TwilioSMSProvider
from communicator.channels.whatsapp import MetaWhatsAppProvider, TwilioWhatsAppProvider
from communicator.models.contact import ChannelType
from communicator.services.integration_settings import get_whatsapp_config

_PROVIDERS: dict[ChannelType, type[ChannelProvider]] = {
    ChannelType.EMAIL: PostmarkEmailProvider,
    ChannelType.SMS: TwilioSMSProvider,
}


async def get_provider(channel: ChannelType, session: AsyncSession) -> ChannelProvider:
    if channel == ChannelType.WHATSAPP:
        config = await get_whatsapp_config(session)
        if config.provider == "meta":
            return MetaWhatsAppProvider(
                access_token=config.access_token, phone_number_id=config.phone_number_id
            )
        if config.provider == "twilio":
            return TwilioWhatsAppProvider()
        raise ValueError(f"Unknown whatsapp_provider setting: {config.provider!r}")

    try:
        provider_cls = _PROVIDERS[channel]
    except KeyError:
        raise ValueError(f"No channel provider configured for {channel}.") from None
    return provider_cls()


__all__ = ["ChannelProvider", "SendResult", "get_provider"]
