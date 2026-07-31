from __future__ import annotations

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.harvester_client import HarvesterClient
from communicator.models.audit import AuditLog
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.draft import Draft, DraftStatus
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus


async def approve_and_send(session: AsyncSession, draft: Draft, harvester_client: HarvesterClient) -> Draft:
    conversation = await session.get(Conversation, draft.conversation_id)
    contact = await session.get(Contact, conversation.contact_id)
    lead = await session.get(Lead, contact.lead_id)

    provider = get_provider(contact.channel)
    result = await provider.send(contact, draft.body)

    message = Message(
        conversation_id=conversation.id,
        direction=MessageDirection.OUTBOUND,
        channel=contact.channel,
        body=draft.body,
        provider=result.provider,
        provider_message_id=result.provider_message_id,
        provider_metadata=result.raw_response,
        status=MessageStatus.SENT,
    )
    session.add(message)

    draft.status = DraftStatus.SENT
    session.add(AuditLog(entity_type="draft", entity_id=draft.id, action="approved_and_sent", actor="human"))

    await harvester_client.push_outreach_outcome(
        institution_id=lead.harvester_institution_id, replied=True, positive_reply=True
    )

    await session.flush()
    return draft


async def reject_draft(session: AsyncSession, draft: Draft, feedback: str) -> Draft:
    draft.status = DraftStatus.REJECTED
    draft.reviewer_feedback = feedback
    session.add(
        AuditLog(
            entity_type="draft",
            entity_id=draft.id,
            action="rejected",
            actor="human",
            details={"feedback": feedback},
        )
    )
    await session.flush()
    return draft
