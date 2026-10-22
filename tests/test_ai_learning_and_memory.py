import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.knowledge_base import KnowledgeGap
from communicator.models.lead import Lead
from communicator.services.knowledge_gaps import (
    approve_knowledge_gap,
    detect_knowledge_gaps,
    dismiss_knowledge_gap,
)
from communicator.services.knowledge_ingest import ingest_document_bytes
from communicator.services.memory import (
    extract_contact_memories,
    get_contact_memories,
    render_contact_memories_context,
)


@pytest.mark.asyncio
async def test_contact_memory_extraction_and_grounding(db_session: AsyncSession):
    lead = Lead(
        harvester_institution_id="test:inst-1", name="Maya Corp", institution_type="hospital"
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155559988",
        person_name="Maya Chen",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    # Inbound message with channel preference & SLA requirement
    inbound_text = "Hi, our team prefers email over calls. Also please note that we require 99.99% uptime guarantee."
    memories = await extract_contact_memories(db_session, contact, conv, inbound_text)

    assert len(memories) >= 2
    facts = [m.fact for m in memories]
    assert any("email over calls" in f.lower() for f in facts)
    assert any("99.99% uptime" in f.lower() for f in facts)

    # Verify persistent storage
    loaded = await get_contact_memories(db_session, contact.id)
    assert len(loaded) == len(memories)

    # Verify context rendering for Claude prompt
    prompt_context = render_contact_memories_context(loaded)
    assert "CUSTOMER MEMORY & REMEMBERED PREFERENCES (REMEMBERS)" in prompt_context
    assert "email over calls" in prompt_context


@pytest.mark.asyncio
async def test_document_ingestion(db_session: AsyncSession):
    content = b"""Refund and Cancellation Policy
Customers may request a full refund within 30 days of purchase.
All refunds are credited back to the original payment method within 3 to 5 business days."""

    entry = ingest_document_bytes(db_session, "refund-policy.txt", content)
    await db_session.flush()

    assert entry.title == "Refund Policy"
    assert "within 30 days" in entry.content
    assert entry.source_type == "document"
    assert entry.file_name == "refund-policy.txt"


@pytest.mark.asyncio
async def test_knowledge_gap_detection_and_approval(db_session: AsyncSession):
    # Detect starter / suggested gaps
    gaps = await detect_knowledge_gaps(db_session)
    assert len(gaps) > 0

    first_gap = gaps[0]
    assert first_gap.status == "suggested"
    gap_id = first_gap.id

    # Approve gap -> turns into KnowledgeBaseEntry
    approved_entry = await approve_knowledge_gap(
        db_session, gap_id, custom_answer="Standard refunds take 3-5 business days."
    )
    assert approved_entry.title == first_gap.question
    assert approved_entry.content == "Standard refunds take 3-5 business days."
    assert approved_entry.source_type == "suggested_faq"

    # Gap status updated
    updated_gap = await db_session.get(KnowledgeGap, gap_id)
    assert updated_gap.status == "approved"

    # Dismiss another gap
    if len(gaps) > 1:
        await dismiss_knowledge_gap(db_session, gaps[1].id)
        dismissed_gap = await db_session.get(KnowledgeGap, gaps[1].id)
        assert dismissed_gap.status == "dismissed"


@pytest.mark.asyncio
async def test_knowledge_base_and_memory_api_endpoints(
    test_client: AsyncClient, db_session: AsyncSession
):
    # 1. Test Ingest Document upload endpoint
    files = {
        "file": (
            "return-policy.txt",
            b"Items must be returned in original packaging.",
            "text/plain",
        )
    }
    resp = await test_client.post("/api/v1/knowledge-base/upload", files=files)
    assert resp.status_code == 200
    data = resp.json()
    assert data["title"] == "Return Policy"
    assert data["sourceType"] == "document"

    # 2. Test Knowledge Gaps endpoints
    resp_gaps = await test_client.get("/api/v1/knowledge-base/gaps")
    assert resp_gaps.status_code == 200

    resp_scan = await test_client.post("/api/v1/knowledge-base/gaps/scan")
    assert resp_scan.status_code == 200
    assert resp_scan.json()["status"] == "success"

    # 3. Test Contact Memories endpoints
    lead = Lead(
        harvester_institution_id="test:inst-2", name="Beta Health", institution_type="clinic"
    )
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+14155557766",
        person_name="Daniel Okafor",
    )
    db_session.add(contact)
    await db_session.flush()

    # Create memory via API
    resp_create_mem = await test_client.post(
        f"/api/v1/contacts/{contact.id}/memories",
        json={"fact": "Prefers WhatsApp notifications over SMS", "category": "preference"},
    )
    assert resp_create_mem.status_code == 200
    mem_id = resp_create_mem.json()["id"]

    # List memories
    resp_list_mem = await test_client.get(f"/api/v1/contacts/{contact.id}/memories")
    assert resp_list_mem.status_code == 200
    assert len(resp_list_mem.json()) == 1

    # Delete memory
    resp_del_mem = await test_client.delete(f"/api/v1/contacts/{contact.id}/memories/{mem_id}")
    assert resp_del_mem.status_code == 200
