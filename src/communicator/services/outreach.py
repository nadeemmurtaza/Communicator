"""
Phase 1 first-touch send: a manually-templated message (no AI drafting
yet -- that's Phase 3), sent through the right channel adapter, with the
send reported back to Harvester's OutreachOutcome. See
docs/ARCHITECTURE.md sections 5 and 9.
"""

from __future__ import annotations

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.harvester_client import HarvesterClient
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead
from communicator.models.message import Message
from communicator.models.sequence import FollowUpSequence
from communicator.services.followup_scheduler import enroll_in_sequence
from communicator.services.messaging import send_outbound_message


async def send_first_touch(
    session: AsyncSession,
    lead: Lead,
    contact: Contact,
    body: str,
    harvester_client: HarvesterClient,
    subject: str | None = None,
    sequence: FollowUpSequence | None = None,
) -> Message:
    provider = get_provider(contact.channel)
    message = await send_outbound_message(session, contact, provider, body, subject=subject)

    if sequence is not None:
        conversation = await session.get(Conversation, message.conversation_id)
        await enroll_in_sequence(session, conversation, sequence)

    await harvester_client.push_outreach_outcome(
        institution_id=lead.harvester_institution_id,
        opened=False,
        clicked=False,
        replied=False,
    )

    return message
