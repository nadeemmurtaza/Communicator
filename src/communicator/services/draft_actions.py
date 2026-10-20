from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.harvester_client import HarvesterClient
from communicator.models.audit import AuditLog
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.draft import Draft, DraftStatus
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus


async def approve_and_send(
    session: AsyncSession, draft: Draft, harvester_client: HarvesterClient, actor: str = "human"
) -> Draft:
    conversation = await session.get(Conversation, draft.conversation_id)
    contact = await session.get(Contact, conversation.contact_id)
    lead = await session.get(Lead, contact.lead_id)

    provider = await get_provider(contact.channel, session)
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
    action = "auto_sent" if actor == "ai" else "approved_and_sent"
    session.add(AuditLog(entity_type="draft", entity_id=draft.id, action=action, actor=actor))

    await harvester_client.push_outreach_outcome(
        institution_id=lead.harvester_institution_id, replied=True, positive_reply=True
    )

    await session.flush()
    return draft


async def dispatch_auto_approved_drafts(
    session: AsyncSession, harvester_client: HarvesterClient
) -> list[Draft]:
    """
    Phase 4 auto-send: the reply engine already gates which drafts reach
    AUTO_APPROVED (see services/reply_engine.py); this just carries out the
    send for anything sitting in that state, same as a human clicking
    "approve" would.
    """
    stmt = select(Draft).where(Draft.status == DraftStatus.AUTO_APPROVED)
    drafts = (await session.execute(stmt)).scalars().all()
    return [
        await approve_and_send(session, draft, harvester_client, actor="ai") for draft in drafts
    ]


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
