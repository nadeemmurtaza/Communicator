import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.models.ticket import (
    ActivityType,
    Ticket,
    TicketDepartment,
    TicketPriority,
    TicketStatus,
    TicketType,
)
from communicator.services.ticket_service import TicketService


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_create_customer_and_back_office_tickets(db_session):
    lead = Lead(harvester_institution_id="test_lead_tickets_1", name="Jane Doe")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        person_name="Jane Doe",
        value="+15551234567",
        channel=ChannelType.WHATSAPP,
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(
        contact_id=contact.id,
        state=ConversationState.ENGAGED,
    )
    db_session.add(conv)
    await db_session.commit()

    # 1. Create Customer Issue Ticket
    cust_ticket = await TicketService.create_ticket(
        db=db_session,
        title="Payment authorization failed on order #9021",
        description="Customer was double charged but order didn't complete.",
        ticket_type=TicketType.CUSTOMER_ISSUE.value,
        department=TicketDepartment.SUPPORT.value,
        priority=TicketPriority.HIGH.value,
        conversation_id=conv.id,
        contact_id=contact.id,
        author_name="Sarah (Support)",
    )

    assert cust_ticket.id is not None
    assert cust_ticket.ticket_number.startswith("TCK-")
    assert cust_ticket.ticket_type == TicketType.CUSTOMER_ISSUE.value
    assert cust_ticket.status == TicketStatus.OPEN.value
    assert cust_ticket.conversation_id == conv.id

    # 2. Create Back-Office Collaboration Ticket
    bo_ticket = await TicketService.create_ticket(
        db=db_session,
        title="Finance Reversal: Void authorization hold on order #9021",
        description="Support escalated to Finance to release pre-auth hold within 2 hours.",
        ticket_type=TicketType.BACK_OFFICE.value,
        department=TicketDepartment.FINANCE.value,
        priority=TicketPriority.URGENT.value,
        author_name="Sarah (Support)",
    )

    assert bo_ticket.id is not None
    assert bo_ticket.ticket_type == TicketType.BACK_OFFICE.value
    assert bo_ticket.department == TicketDepartment.FINANCE.value


@pytest.mark.asyncio
async def test_back_office_collaboration_notes_and_updates(db_session):
    ticket = await TicketService.create_ticket(
        db=db_session,
        title="Warehouse Stock Discrepancy - SKU 8891",
        description="Physical count differs from ERP by 14 units.",
        ticket_type=TicketType.BACK_OFFICE.value,
        department=TicketDepartment.WAREHOUSE.value,
        author_name="Marcus",
    )

    # Add back-office collaboration notes
    note = await TicketService.add_activity(
        db=db_session,
        ticket_id=ticket.id,
        message="Aisle 4 shelf 2 recounted: 10 units found in overflow staging bin.",
        activity_type=ActivityType.BACK_OFFICE_NOTE.value,
        author_name="Dave (Warehouse Lead)",
    )
    assert note.id is not None
    assert "Aisle 4" in note.message

    # Update ticket status and priority
    updated = await TicketService.update_ticket(
        db=db_session,
        ticket_id=ticket.id,
        status=TicketStatus.IN_PROGRESS.value,
        priority=TicketPriority.HIGH.value,
        author_name="Dave (Warehouse Lead)",
    )
    assert updated.status == TicketStatus.IN_PROGRESS.value
    assert updated.priority == TicketPriority.HIGH.value

    # Verify activities contain both note and status changes
    detail = await TicketService.get_ticket(db_session, ticket.id)
    assert detail is not None
    assert len(detail["activities"]) >= 3
    activity_messages = [a["message"] for a in detail["activities"]]
    assert any("Aisle 4" in m for m in activity_messages)
    assert any("Status changed" in m for m in activity_messages)


@pytest.mark.asyncio
async def test_tracker_ticket_widespread_linking_and_broadcast_resolution(db_session):
    lead1 = Lead(harvester_institution_id="test_lead_tracker_1", name="Alice")
    lead2 = Lead(harvester_institution_id="test_lead_tracker_2", name="Bob")
    db_session.add_all([lead1, lead2])
    await db_session.flush()

    contact1 = Contact(lead_id=lead1.id, person_name="Alice", value="+15551111111", channel=ChannelType.SMS)
    contact2 = Contact(lead_id=lead2.id, person_name="Bob", value="+15552222222", channel=ChannelType.SMS)
    db_session.add_all([contact1, contact2])
    await db_session.flush()

    conv1 = Conversation(contact_id=contact1.id, state=ConversationState.ENGAGED)
    conv2 = Conversation(contact_id=contact2.id, state=ConversationState.ENGAGED)
    db_session.add_all([conv1, conv2])
    await db_session.commit()

    # 1. Create widespread Tracker ticket
    tracker = await TicketService.create_ticket(
        db=db_session,
        title="Outage: US-East Carrier SMS Delivery Delay",
        description="SMS gateway throttling carrier messages across Midwest.",
        ticket_type=TicketType.TRACKER.value,
        department=TicketDepartment.ENGINEERING.value,
        priority=TicketPriority.URGENT.value,
        author_name="Captain AI Incident Monitor",
    )

    # 2. Create customer tickets and link to tracker
    child1 = await TicketService.create_ticket(
        db=db_session,
        title="SMS 2FA code not received",
        description="Customer cannot log in due to missing verification SMS.",
        ticket_type=TicketType.CUSTOMER_ISSUE.value,
        conversation_id=conv1.id,
        contact_id=contact1.id,
        tracker_id=tracker.id,
    )

    child2 = await TicketService.create_ticket(
        db=db_session,
        title="Shipping confirmation SMS delayed",
        description="Customer placed order 4 hours ago, no text notification.",
        ticket_type=TicketType.CUSTOMER_ISSUE.value,
        conversation_id=conv2.id,
        contact_id=contact2.id,
    )
    # Link child2 using explicit link_to_tracker
    await TicketService.link_to_tracker(db_session, child2.id, tracker.id, author_name="Agent")

    # Check tracker details
    tracker_detail = await TicketService.get_ticket(db_session, tracker.id)
    assert tracker_detail["child_count"] == 2
    assert len(tracker_detail["child_tickets"]) == 2

    # 3. Broadcast Widespread Resolution & Customer Notification
    broadcast_result = await TicketService.broadcast_tracker_update(
        db=db_session,
        tracker_id=tracker.id,
        broadcast_message="Carrier gateway queue has recovered; all delayed SMS messages have been delivered.",
        resolve_all_children=True,
        send_customer_message=True,
        author_name="Incident Response Team",
    )

    assert broadcast_result["impacted_tickets"] == 2
    assert broadcast_result["messages_sent"] == 2
    assert broadcast_result["resolved_tickets"] == 2
    assert broadcast_result["tracker_resolved"] is True

    # Verify both children and tracker are resolved in DB
    refreshed_c1 = await TicketService.get_ticket(db_session, child1.id)
    refreshed_c2 = await TicketService.get_ticket(db_session, child2.id)
    refreshed_tracker = await TicketService.get_ticket(db_session, tracker.id)

    assert refreshed_c1["status"] == TicketStatus.RESOLVED.value
    assert refreshed_c2["status"] == TicketStatus.RESOLVED.value
    assert refreshed_tracker["status"] == TicketStatus.RESOLVED.value
    assert refreshed_c1["customer_broadcast_sent"] is True


@pytest.mark.asyncio
async def test_ticket_list_filtering_and_stats(db_session):
    # Create tickets across different types and departments
    await TicketService.create_ticket(
        db=db_session,
        title="Customer Delivery Missing",
        description="Package tracking stuck in transit.",
        ticket_type=TicketType.CUSTOMER_ISSUE.value,
        department=TicketDepartment.SUPPORT.value,
    )
    await TicketService.create_ticket(
        db=db_session,
        title="Legal Compliance Review for Terms v3",
        description="Review privacy clause for California CCPA.",
        ticket_type=TicketType.BACK_OFFICE.value,
        department=TicketDepartment.LEGAL.value,
    )

    # Test filtering by type
    back_office_list = await TicketService.list_tickets(
        db_session, ticket_type=TicketType.BACK_OFFICE.value
    )
    assert any(t["department"] == "legal" for t in back_office_list)

    # Test stats summary
    stats = await TicketService.get_stats_summary(db_session)
    assert stats["total_tickets"] >= 2
    assert stats["back_office_tickets"] >= 1
    assert "legal" in stats["department_breakdown"]


@pytest.mark.asyncio
async def test_tickets_api_routes(override_database_dependency, db_session):
    async with AsyncClient(
        transport=ASGITransport(app=app),
        base_url="http://test",
    ) as client:
        # 1. Create ticket via API
        resp = await client.post(
            "/api/v1/tickets",
            json={
                "title": "API Test Customer Issue",
                "description": "Customer needs assistance with password reset link.",
                "ticket_type": "customer_issue",
                "department": "support",
                "priority": "medium",
            },
        )
        assert resp.status_code == 201
        data = resp.json()
        ticket_id = data["id"]
        assert data["title"] == "API Test Customer Issue"

        # 2. Add activity note via API
        resp_act = await client.post(
            f"/api/v1/tickets/{ticket_id}/activities",
            json={
                "message": "Sent password reset link to customer email.",
                "author_name": "Support Bot",
                "activity_type": "comment",
            },
        )
        assert resp_act.status_code == 201
        assert resp_act.json()["message"] == "Sent password reset link to customer email."

        # 3. List tickets via API
        list_resp = await client.get("/api/v1/tickets")
        assert list_resp.status_code == 200
        assert len(list_resp.json()) > 0

        # 4. Get stats via API
        stats_resp = await client.get("/api/v1/tickets/stats")
        assert stats_resp.status_code == 200
        stats_data = stats_resp.json()
        assert "total_tickets" in stats_data
        assert "customer_issues" in stats_data
