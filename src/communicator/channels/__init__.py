from communicator.channels.base import ChannelProvider, SendResult
from communicator.channels.email import PostmarkEmailProvider
from communicator.channels.sms import TwilioSMSProvider
from communicator.channels.whatsapp import MetaWhatsAppProvider, TwilioWhatsAppProvider
from communicator.config import settings
from communicator.models.contact import ChannelType

_PROVIDERS: dict[ChannelType, type[ChannelProvider]] = {
    ChannelType.EMAIL: PostmarkEmailProvider,
    ChannelType.SMS: TwilioSMSProvider,
    ChannelType.WHATSAPP: TwilioWhatsAppProvider,
}

_WHATSAPP_PROVIDERS: dict[str, type[ChannelProvider]] = {
    "twilio": TwilioWhatsAppProvider,
    "meta": MetaWhatsAppProvider,
}


def get_provider(channel: ChannelType) -> ChannelProvider:
    if channel == ChannelType.WHATSAPP:
        try:
            provider_cls = _WHATSAPP_PROVIDERS[settings.whatsapp_provider]
        except KeyError:
            raise ValueError(
                f"Unknown whatsapp_provider setting: {settings.whatsapp_provider!r}"
            ) from None
        return provider_cls()

    try:
        provider_cls = _PROVIDERS[channel]
    except KeyError:
        raise ValueError(f"No channel provider configured for {channel}.") from None
    return provider_cls()


__all__ = ["ChannelProvider", "SendResult", "get_provider"]
