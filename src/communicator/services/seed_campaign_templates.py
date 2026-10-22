"""
Seeds the first tenant's (NEWAX's) campaign templates as data. Each row
mirrors a real Meta WhatsApp message template registered under that
tenant's WABA. Another tenant onboards the same way: submit its own
templates to Meta, insert its own CampaignTemplate rows under its own
tenant_key -- no code change.
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.campaign_template import CampaignTemplate

NEWAX_TEMPLATES = [
    {
        "tenant_key": "newax",
        "key": "business_followup",
        "channel": "whatsapp",
        "name": "Business Follow-up",
        "purpose": "Cold outreach to a tagged audience introducing NEWAX's capabilities.",
        "meta_template_name": "newax_business_followup",
        "meta_language": "en_US",
        "meta_category": "MARKETING",
        "meta_status": "PENDING",
        "body_preview": (
            "Hi {{1}}, this is NEWAX. We help businesses connect their systems and cut "
            "manual work with Enterprise Software, Intelligent Automation, and Connected "
            "Infrastructure. Want to talk through what you are working on?"
        ),
        "variables": ["contact_name"],
    },
    {
        "tenant_key": "newax",
        "key": "meeting_reminder",
        "channel": "whatsapp",
        "name": "Meeting Reminder",
        "purpose": "Reminds a contact of an upcoming discovery call.",
        "meta_template_name": "newax_meeting_reminder",
        "meta_language": "en_US",
        "meta_category": "UTILITY",
        "meta_status": "PENDING",
        "body_preview": (
            "Hi {{1}}, quick reminder about your NEWAX discovery call on {{2}} at {{3}}. "
            "Reply here if you need to reschedule."
        ),
        "variables": ["contact_name", "meeting_date", "meeting_time"],
    },
    {
        "tenant_key": "newax",
        "key": "inquiry_received",
        "channel": "whatsapp",
        "name": "Inquiry Received",
        "purpose": "Confirms a business inquiry was logged and will be followed up on.",
        "meta_template_name": "newax_inquiry_received",
        "meta_language": "en_US",
        "meta_category": "UTILITY",
        "meta_status": "PENDING",
        "body_preview": (
            "Hi {{1}}, thanks for reaching out to NEWAX. We have logged your request and "
            "a team member will follow up shortly."
        ),
        "variables": ["contact_name"],
    },
    {
        "tenant_key": "newax",
        "key": "proposal_followup",
        "channel": "whatsapp",
        "name": "Proposal Follow-up",
        "purpose": "Nudges a lead who received a proposal and hasn't replied.",
        "meta_template_name": "newax_proposal_followup",
        "meta_language": "en_US",
        "meta_category": "MARKETING",
        "meta_status": "PENDING",
        "body_preview": (
            "Hi {{1}}, following up on the proposal we shared for {{2}}. Happy to answer "
            "any questions or walk through it together — just reply here."
        ),
        "variables": ["contact_name", "project_name"],
    },
]


async def seed_newax_campaign_templates(session: AsyncSession) -> int:
    count = 0
    for data in NEWAX_TEMPLATES:
        existing = (
            await session.execute(
                select(CampaignTemplate).where(
                    CampaignTemplate.tenant_key == data["tenant_key"],
                    CampaignTemplate.key == data["key"],
                )
            )
        ).scalar_one_or_none()
        if existing is not None:
            for field, value in data.items():
                setattr(existing, field, value)
        else:
            session.add(CampaignTemplate(**data))
        count += 1
    await session.flush()
    return count
