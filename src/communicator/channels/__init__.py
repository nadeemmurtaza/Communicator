from communicator.channels.base import ChannelProvider, SendResult
from communicator.channels.email import PostmarkEmailProvider
from communicator.channels.sms import TwilioSMSProvider
from communicator.channels.whatsapp import TwilioWhatsAppProvider
from communicator.models.contact import ChannelType

_PROVIDERS: dict[ChannelType, type[ChannelProvider]] = {
    ChannelType.EMAIL: PostmarkEmailProvider,
    ChannelType.SMS: TwilioSMSProvider,
    ChannelType.WHATSAPP: TwilioWhatsAppProvider,
}


def get_provider(channel: ChannelType) -> ChannelProvider:
    try:
        provider_cls = _PROVIDERS[channel]
    except KeyError:
        raise ValueError(f"No channel provider configured for {channel}.") from None
    return provider_cls()


__all__ = ["ChannelProvider", "SendResult", "get_provider"]
