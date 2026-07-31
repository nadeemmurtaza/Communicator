from __future__ import annotations

import abc
from dataclasses import dataclass, field

from communicator.models.contact import Contact


@dataclass
class SendResult:
    provider: str
    provider_message_id: str
    raw_response: dict = field(default_factory=dict)


class ChannelProvider(abc.ABC):
    """
    One implementation per outbound channel (email, SMS, ...). Nothing
    outside communicator.services.messaging should call a provider's
    ``send`` directly -- that module is what enforces the compliance gate
    and records the Message row.
    """

    @abc.abstractmethod
    async def send(
        self,
        contact: Contact,
        body: str,
        subject: str | None = None,
        content_sid: str | None = None,
    ) -> SendResult:
        """
        ``content_sid`` is a pre-approved template reference (e.g. a Twilio
        Content API SID). Only WhatsApp acts on it -- see
        communicator.channels.whatsapp -- other providers accept and ignore
        it so callers don't need to branch by channel.
        """
        ...
