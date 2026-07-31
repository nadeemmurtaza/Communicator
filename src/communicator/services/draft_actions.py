from __future__ import annotations

from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.harvester_client import HarvesterClient
from communicator.models.audit import AuditLog
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.draft import Draft, DraftStatus
from communicator.models.lead import Lead
from communicator.services.messaging import send_outbound_message


async def approve_and_send(
    session: AsyncSession, draft: Draft, harvester_client: HarvesterClient, actor: str = "human"
) -> Draft:
    conversation = await session.get(Conversation, draft.conversation_id)
    contact = await session.get(Contact, conversation.contact_id)
    lead = await session.get(Lead, contact.lead_id)

    provider = get_provider(contact.channel)
    await send_outbound_message(
        session, contact, provider, draft.body, content_sid=draft.content_sid or None
    )

    draft.status = DraftStatus.SENT
    action = "approved_and_sent" if actor == "human" else "auto_sent"
    session.add(AuditLog(entity_type="draft", entity_id=draft.id, action=action, actor=actor))

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
