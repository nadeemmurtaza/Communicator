import pytest
from sqlalchemy import select

from communicator.models.consent import ConsentRecord, ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.lead import Lead
from communicator.services.lead_sync import upsert_lead_from_institution

INSTITUTION_PAYLOAD = {
    "id": "11111111-1111-1111-1111-111111111111",
    "canonical_name": "Sunrise Medical Clinic",
    "institution_type": "clinic",
    "domain": "sunrisemed.example",
    "website": "https://sunrisemed.example",
    "status": "operational",
    "confidence_score": 82.0,
    "branches": [
        {
            "id": "22222222-2222-2222-2222-222222222222",
            "branch_name": "Main",
            "address": "Islamabad, Pakistan",
            "provider_ids": {},
        }
    ],
    "contact_points": [
        {
            "id": "33333333-3333-3333-3333-333333333333",
            "contact_type": "email",
            "contact_value": "admin@sunrisemed.example",
            "person_name": "Dr. Ali",
            "role": "Director",
        },
        {
            "id": "44444444-4444-4444-4444-444444444444",
            "contact_type": "phone",
            "contact_value": "+923001234567",
            "person_name": None,
            "role": None,
        },
        {
            "id": "55555555-5555-5555-5555-555555555555",
            "contact_type": "telegram",
            "contact_value": "@sunrisemed",
            "person_name": None,
            "role": None,
        },
    ],
}


@pytest.mark.asyncio
async def test_upsert_lead_creates_lead_and_contacts(db_session):
    lead = await upsert_lead_from_institution(db_session, INSTITUTION_PAYLOAD)
    await db_session.flush()

    assert lead.name == "Sunrise Medical Clinic"
    assert lead.city == "Islamabad, Pakistan"

    contacts = (
        (await db_session.execute(select(Contact).where(Contact.lead_id == lead.id)))
        .scalars()
        .all()
    )
    channels = {c.channel for c in contacts}
    # telegram is skipped: no channel provider maps to it yet
    assert channels == {ChannelType.EMAIL, ChannelType.SMS}

    for contact in contacts:
        consent = (
            await db_session.execute(
                select(ConsentRecord).where(ConsentRecord.contact_id == contact.id)
            )
        ).scalar_one()
        assert consent.status == ConsentStatus.OPTED_IN
        assert consent.source == "harvester_sync"


@pytest.mark.asyncio
async def test_upsert_lead_is_idempotent(db_session):
    await upsert_lead_from_institution(db_session, INSTITUTION_PAYLOAD)
    await db_session.flush()
    lead_again = await upsert_lead_from_institution(db_session, INSTITUTION_PAYLOAD)
    await db_session.flush()

    leads = (
        (
            await db_session.execute(
                select(Lead).where(Lead.harvester_institution_id == INSTITUTION_PAYLOAD["id"])
            )
        )
        .scalars()
        .all()
    )
    assert len(leads) == 1

    contacts = (
        (await db_session.execute(select(Contact).where(Contact.lead_id == lead_again.id)))
        .scalars()
        .all()
    )
    assert len(contacts) == 2  # not duplicated on re-sync
