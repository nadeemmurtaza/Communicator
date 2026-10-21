from __future__ import annotations

import io
import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.segment import ContactSegment


@pytest.mark.asyncio
async def test_multi_channel_csv_import(test_client: AsyncClient, db_session: AsyncSession):
    csv_content = (
        "name,channel,value,tags\n"
        "Alice Wang,whatsapp,+14155550101,vip|enterprise\n"
        "Bob Dylan,email,bob@company.org,leads\n"
        "Charlie Brown,sms,+14155550102,prospect\n"
        "Diana Prince,tiktok,tt_open_id_555,influencer\n"
        "Evan Wright,telegram,tg_chat_1122,community\n"
        "Alice Wang,whatsapp,+14155550101,renewal\n"  # Duplicate -> should update tags
        ",,,\n"  # Empty -> should skip
    )

    files = {"file": ("contacts.csv", io.BytesIO(csv_content.encode("utf-8")), "text/csv")}
    res = await test_client.post(
        "/api/v1/contacts/import",
        files=files,
        data={"default_tags": "imported_2026"},
    )
    assert res.status_code == 200
    data = res.json()
    assert data["status"] == "ok"
    assert data["created"] == 5
    assert data["updated"] == 1
    assert data["skipped"] == 1
    assert data["total"] == 7
    assert data["channels"]["whatsapp"] == 2
    assert data["channels"]["email"] == 1
    assert data["channels"]["sms"] == 1
    assert data["channels"]["tiktok"] == 1
    assert data["channels"]["telegram"] == 1

    # Verify contacts in database
    from sqlalchemy.orm import selectinload

    alice_stmt = (
        select(Contact)
        .options(selectinload(Contact.tags))
        .where(Contact.channel == ChannelType.WHATSAPP, Contact.value == "+14155550101")
    )
    alice = (await db_session.execute(alice_stmt)).scalar_one_or_none()
    assert alice is not None
    assert alice.person_name == "Alice Wang"
    alice_tags = {t.name for t in alice.tags}
    assert "vip" in alice_tags
    assert "enterprise" in alice_tags
    assert "renewal" in alice_tags
    assert "imported_2026" in alice_tags

    bob_stmt = select(Contact).where(
        Contact.channel == ChannelType.EMAIL, Contact.value == "bob@company.org"
    )
    bob = (await db_session.execute(bob_stmt)).scalar_one_or_none()
    assert bob is not None
    assert bob.channel == ChannelType.EMAIL

    diana_stmt = select(Contact).where(
        Contact.channel == ChannelType.TIKTOK, Contact.value == "tt_open_id_555"
    )
    diana = (await db_session.execute(diana_stmt)).scalar_one_or_none()
    assert diana is not None
    assert diana.channel == ChannelType.TIKTOK


@pytest.mark.asyncio
async def test_csv_import_preview(test_client: AsyncClient):
    csv_content = (
        "name,email,phone,channel,tags\n"
        "Frank Castle,frank@marvel.com,,email,hero|vigilante\n"
        "Peter Parker,,+14155550999,whatsapp,avenger\n"
    )
    files = {"file": ("preview_test.csv", io.BytesIO(csv_content.encode("utf-8")), "text/csv")}
    res = await test_client.post("/api/v1/contacts/import/preview", files=files)
    assert res.status_code == 200
    data = res.json()
    assert data["totalRows"] == 2
    assert "name" in data["detectedColumns"]
    assert "email" in data["detectedChannels"]
    assert "whatsapp" in data["detectedChannels"]
    assert len(data["sampleRows"]) == 2
    assert data["sampleRows"][0]["name"] == "Frank Castle"
    assert data["sampleRows"][0]["value"] == "frank@marvel.com"
    assert data["sampleRows"][0]["valid"] is True


@pytest.mark.asyncio
async def test_contacts_export_csv(test_client: AsyncClient, db_session: AsyncSession):
    # Ensure at least one contact exists
    contact = Contact(
        channel=ChannelType.EMAIL,
        value="export_test@example.com",
        person_name="Export Tester",
    )
    from communicator.models.lead import Lead
    lead = Lead(harvester_institution_id="export_lead_1", name="Export Tester", institution_type="test")
    db_session.add(lead)
    await db_session.flush()
    contact.lead_id = lead.id
    db_session.add(contact)
    await db_session.flush()

    res = await test_client.get("/api/v1/contacts/export?channel=email")
    assert res.status_code == 200
    assert "text/csv" in res.headers["content-type"]
    text = res.text
    assert "ID,Name,Channel,Value,Language,Tags,Created At" in text
    assert "export_test@example.com" in text


@pytest.mark.asyncio
async def test_saved_segments_crud_and_filtering(test_client: AsyncClient, db_session: AsyncSession):
    # 1. Fetch default segments
    list_res = await test_client.get("/api/v1/segments")
    assert list_res.status_code == 200
    segments = list_res.json()
    assert len(segments) >= 4
    assert any(s["name"] == "All Omnichannel Contacts" for s in segments)
    assert any(s["name"] == "WhatsApp VIP Audience" for s in segments)

    # 2. Create custom segment
    create_res = await test_client.post(
        "/api/v1/segments",
        json={
            "name": "High Value Enterprise Segment",
            "description": "High value accounts with enterprise tag",
            "filters": {
                "channels": ["whatsapp", "email"],
                "tags": ["enterprise"],
                "tag_match_mode": "any",
            },
        },
    )
    assert create_res.status_code == 201
    seg = create_res.json()
    seg_id = seg["id"]
    assert seg["name"] == "High Value Enterprise Segment"
    assert "memberCount" in seg

    # 3. Get segment details and members
    detail_res = await test_client.get(f"/api/v1/segments/{seg_id}")
    assert detail_res.status_code == 200
    detail = detail_res.json()
    assert detail["name"] == "High Value Enterprise Segment"
    assert "contacts" in detail

    # 4. Update segment
    update_res = await test_client.put(
        f"/api/v1/segments/{seg_id}",
        json={
            "name": "Updated Enterprise Segment",
            "description": "Updated description for enterprise accounts",
        },
    )
    assert update_res.status_code == 200
    assert update_res.json()["name"] == "Updated Enterprise Segment"

    # 5. Export segment CSV
    export_res = await test_client.get(f"/api/v1/segments/{seg_id}/export")
    assert export_res.status_code == 200
    assert "text/csv" in export_res.headers["content-type"]

    # 6. Delete segment
    del_res = await test_client.delete(f"/api/v1/segments/{seg_id}")
    assert del_res.status_code == 200
    assert del_res.json()["deleted"] is True

    # Confirm 404 after deletion
    get_after_del = await test_client.get(f"/api/v1/segments/{seg_id}")
    assert get_after_del.status_code == 404


@pytest.mark.asyncio
async def test_contacts_list_by_channel_and_search(test_client: AsyncClient, db_session: AsyncSession):
    # Test filtering by channel and search keyword
    list_all = await test_client.get("/api/v1/contacts")
    assert list_all.status_code == 200

    list_ch = await test_client.get("/api/v1/contacts?channel=whatsapp")
    assert list_ch.status_code == 200

    list_search = await test_client.get("/api/v1/contacts?search=Alice")
    assert list_search.status_code == 200
