from communicator.channels.base import ChannelProvider, SendResult
from communicator.channels.email import PostmarkEmailProvider
from communicator.channels.sms import TwilioSMSProvider
from communicator.models.contact import ChannelType

# WhatsApp is intentionally absent -- Phase 2 per docs/ARCHITECTURE.md,
# since it needs the 24h-session/template-message handling that doesn't
# exist yet. get_provider() fails loudly for it rather than silently
# no-op'ing.
_PROVIDERS: dict[ChannelType, type[ChannelProvider]] = {
    ChannelType.EMAIL: PostmarkEmailProvider,
    ChannelType.SMS: TwilioSMSProvider,
}


def get_provider(channel: ChannelType) -> ChannelProvider:
    try:
        provider_cls = _PROVIDERS[channel]
    except KeyError:
        raise ValueError(f"No channel provider configured for {channel}.") from None
    return provider_cls()


__all__ = ["ChannelProvider", "SendResult", "get_provider"]
