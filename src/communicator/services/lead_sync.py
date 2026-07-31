"""
Upserts Harvester leads/contacts into Communicator's own tables, and grants
initial consent -- see docs/ARCHITECTURE.md section 4 and 5.

Maps the exact payload shape of Harvester's
``GET /institutions/{id}`` (``InstitutionDetailResponse`` in
lead_harvester/schemas/institutions.py): id, canonical_name,
institution_type, branches[].address, contact_points[].{contact_type,
contact_value, person_name, role}.
"""
from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.lead import Lead

# Harvester's contact_type is a free-form string (see
# lead_harvester/models/canonical.py: "phone" | "email" | "whatsapp" |
# "telegram" | ...). Types with no channel mapping (e.g. "telegram") are
# skipped until Communicator has a provider for them.
_HARVESTER_CONTACT_TYPE_TO_CHANNEL = {
    "email": ChannelType.EMAIL,
    "phone": ChannelType.SMS,
    "whatsapp": ChannelType.WHATSAPP,
}


async def upsert_lead_from_institution(session: AsyncSession, institution: dict) -> Lead:
    """
    Upsert a Lead + its Contacts from one Harvester institution dossier.
    Every *new* Contact is granted OPTED_IN consent (source
    "harvester_sync") on creation: Harvester's own qualification and
    draft-approval flow is what gates whether we're allowed to reach out
    at all, so a lead reaching this sync step has already cleared that bar.
    """
    harvester_id = str(institution["id"])

    stmt = select(Lead).where(Lead.harvester_institution_id == harvester_id)
    lead = (await session.execute(stmt)).scalar_one_or_none()

    if lead is None:
        lead = Lead(harvester_institution_id=harvester_id)
        session.add(lead)

    lead.name = institution.get("canonical_name", "")
    lead.institution_type = institution.get("institution_type", "")

    branches = institution.get("branches") or []
    lead.city = branches[0].get("address", "") if branches else ""

    # Harvester's institution dossier doesn't currently surface
    # QualificationAssessment.rationale -- left blank until that's exposed
    # (see harvester/docs/communicator-integration.md).
    lead.qualification_rationale = institution.get("qualification_rationale", "")

    await session.flush()  # assign lead.id for new Contacts' FK

    for point in institution.get("contact_points") or []:
        channel = _HARVESTER_CONTACT_TYPE_TO_CHANNEL.get(point.get("contact_type", ""))
        contact_value = point.get("contact_value")
        if channel is None or not contact_value:
            continue

        contact_stmt = select(Contact).where(
            Contact.lead_id == lead.id,
            Contact.channel == channel,
            Contact.value == contact_value,
        )
        contact = (await session.execute(contact_stmt)).scalar_one_or_none()
        is_new_contact = contact is None

        if is_new_contact:
            contact = Contact(lead_id=lead.id, channel=channel, value=contact_value)
            session.add(contact)

        contact.harvester_contact_point_id = str(point.get("id", ""))
        contact.person_name = point.get("person_name") or ""
        contact.role = point.get("role") or ""

        if is_new_contact:
            await session.flush()  # assign contact.id for the ConsentRecord FK
            await record_consent(session, contact.id, ConsentStatus.OPTED_IN, source="harvester_sync")

    return lead
