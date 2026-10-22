import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.models.ticket import (
    ActivityType,
    Ticket,
    TicketActivity,
    TicketDepartment,
    TicketPriority,
    TicketStatus,
    TicketType,
)


class TicketService:
    @staticmethod
    async def generate_ticket_number(db: AsyncSession) -> str:
        """Generates a sequential human-readable ticket number (e.g. TCK-1001)."""
        result = await db.execute(select(func.count(Ticket.id)))
        count = result.scalar_one() or 0
        number_candidate = f"TCK-{1001 + count}"

        # Ensure collision safety
        existing = await db.execute(
            select(Ticket.id).where(Ticket.ticket_number == number_candidate)
        )
        if existing.scalar_one_or_none():
            unique_suffix = uuid.uuid4().hex[:4].upper()
            return f"TCK-{1001 + count}-{unique_suffix}"
        return number_candidate

    @staticmethod
    async def create_ticket(
        db: AsyncSession,
        title: str,
        description: str,
        ticket_type: str = TicketType.CUSTOMER_ISSUE.value,
        department: str = TicketDepartment.SUPPORT.value,
        priority: str = TicketPriority.MEDIUM.value,
        status: str = TicketStatus.OPEN.value,
        conversation_id: uuid.UUID | None = None,
        contact_id: uuid.UUID | None = None,
        tracker_id: uuid.UUID | None = None,
        assigned_to_id: uuid.UUID | None = None,
        author_name: str = "Agent",
    ) -> Ticket:
        """Creates a customer issue, back-office task, or tracker ticket."""
        ticket_number = await TicketService.generate_ticket_number(db)

        # If conversation_id is provided but not contact_id, look up conversation's contact
        if conversation_id and not contact_id:
            conv = await db.get(Conversation, conversation_id)
            if conv:
                contact_id = conv.contact_id

        ticket = Ticket(
            ticket_number=ticket_number,
            title=title,
            description=description,
            ticket_type=ticket_type,
            department=department,
            priority=priority,
            status=status,
            conversation_id=conversation_id,
            contact_id=contact_id,
            tracker_id=tracker_id,
            assigned_to_id=assigned_to_id,
        )
        db.add(ticket)
        await db.flush()

        # Log initial creation activity
        if ticket_type == TicketType.TRACKER.value:
            init_msg = (
                "Tracker Ticket initialized to coordinate widespread issue across customer threads."
            )
            act_type = ActivityType.STATUS_CHANGE.value
        elif ticket_type == TicketType.BACK_OFFICE.value:
            init_msg = (
                f"Back-office collaboration ticket dispatched to {department.upper()} department."
            )
            act_type = ActivityType.BACK_OFFICE_NOTE.value
        else:
            init_msg = f"Customer issue ticket opened and assigned to {department.upper()}."
            act_type = ActivityType.STATUS_CHANGE.value

        initial_activity = TicketActivity(
            ticket_id=ticket.id,
            author_name=author_name,
            activity_type=act_type,
            message=init_msg,
            activity_metadata={"status": status, "department": department},
        )
        db.add(initial_activity)

        # If child ticket linked to tracker, record activity on tracker
        if tracker_id:
            tracker_ticket = await db.get(Ticket, tracker_id)
            if tracker_ticket:
                tracker_activity = TicketActivity(
                    ticket_id=tracker_id,
                    author_name=author_name,
                    activity_type=ActivityType.LINKED_TRACKER.value,
                    message=f"Linked child issue {ticket.ticket_number}: '{ticket.title}'",
                    activity_metadata={
                        "child_ticket_id": str(ticket.id),
                        "ticket_number": ticket.ticket_number,
                    },
                )
                db.add(tracker_activity)

        await db.commit()
        await db.refresh(ticket)
        return ticket

    @staticmethod
    async def list_tickets(
        db: AsyncSession,
        ticket_type: str | None = None,
        status: str | None = None,
        department: str | None = None,
        tracker_id: uuid.UUID | None = None,
        conversation_id: uuid.UUID | None = None,
        search: str | None = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[dict[str, Any]]:
        """Lists tickets with filter parameters and includes child counts for tracker tickets."""
        stmt = (
            select(Ticket)
            .options(
                selectinload(Ticket.child_tickets),
                selectinload(Ticket.tracker),
                selectinload(Ticket.contact),
                selectinload(Ticket.conversation),
                selectinload(Ticket.assigned_to),
            )
            .order_by(Ticket.created_at.desc())
        )

        if ticket_type:
            stmt = stmt.where(Ticket.ticket_type == ticket_type)
        if status:
            stmt = stmt.where(Ticket.status == status)
        if department:
            stmt = stmt.where(Ticket.department == department)
        if tracker_id:
            stmt = stmt.where(Ticket.tracker_id == tracker_id)
        if conversation_id:
            stmt = stmt.where(Ticket.conversation_id == conversation_id)
        if search:
            search_filter = f"%{search.strip()}%"
            stmt = stmt.where(
                (Ticket.title.ilike(search_filter))
                | (Ticket.description.ilike(search_filter))
                | (Ticket.ticket_number.ilike(search_filter))
            )

        stmt = stmt.limit(limit).offset(offset)
        result = await db.execute(stmt)
        tickets = result.scalars().all()

        output = []
        for t in tickets:
            item = {
                "id": str(t.id),
                "ticket_number": t.ticket_number,
                "title": t.title,
                "description": t.description,
                "ticket_type": t.ticket_type,
                "status": t.status,
                "priority": t.priority,
                "department": t.department,
                "conversation_id": str(t.conversation_id) if t.conversation_id else None,
                "contact_id": str(t.contact_id) if t.contact_id else None,
                "contact_name": t.contact.person_name if t.contact else None,
                "contact_identifier": t.contact.value if t.contact else None,
                "tracker_id": str(t.tracker_id) if t.tracker_id else None,
                "tracker_number": t.tracker.ticket_number if t.tracker else None,
                "tracker_title": t.tracker.title if t.tracker else None,
                "assigned_to_id": str(t.assigned_to_id) if t.assigned_to_id else None,
                "assigned_to_name": t.assigned_to.name if t.assigned_to else None,
                "resolution_summary": t.resolution_summary,
                "customer_broadcast_sent": t.customer_broadcast_sent,
                "resolved_at": t.resolved_at.isoformat() if t.resolved_at else None,
                "created_at": t.created_at.isoformat(),
                "updated_at": t.updated_at.isoformat(),
                "child_count": len(t.child_tickets) if t.child_tickets else 0,
            }
            if t.ticket_type == TicketType.TRACKER.value and t.child_tickets:
                resolved_children = sum(
                    1 for c in t.child_tickets if c.status == TicketStatus.RESOLVED.value
                )
                item["resolved_child_count"] = resolved_children
            output.append(item)

        return output

    @staticmethod
    async def get_ticket(db: AsyncSession, ticket_id: uuid.UUID) -> dict[str, Any] | None:
        """Retrieves ticket with full activity log and child tickets."""
        stmt = (
            select(Ticket)
            .options(
                selectinload(Ticket.activities),
                selectinload(Ticket.child_tickets).selectinload(Ticket.contact),
                selectinload(Ticket.child_tickets).selectinload(Ticket.conversation),
                selectinload(Ticket.tracker),
                selectinload(Ticket.contact),
                selectinload(Ticket.conversation),
                selectinload(Ticket.assigned_to),
            )
            .where(Ticket.id == ticket_id)
        )
        result = await db.execute(stmt)
        t = result.scalar_one_or_none()
        if not t:
            return None

        activities = [
            {
                "id": str(a.id),
                "author_id": str(a.author_id) if a.author_id else None,
                "author_name": a.author_name,
                "activity_type": a.activity_type,
                "message": a.message,
                "metadata": a.activity_metadata,
                "created_at": a.created_at.isoformat(),
            }
            for a in t.activities
        ]

        children = [
            {
                "id": str(c.id),
                "ticket_number": c.ticket_number,
                "title": c.title,
                "status": c.status,
                "priority": c.priority,
                "department": c.department,
                "contact_id": str(c.contact_id) if c.contact_id else None,
                "contact_name": c.contact.person_name if c.contact else None,
                "conversation_id": str(c.conversation_id) if c.conversation_id else None,
                "created_at": c.created_at.isoformat(),
            }
            for c in t.child_tickets
        ]

        return {
            "id": str(t.id),
            "ticket_number": t.ticket_number,
            "title": t.title,
            "description": t.description,
            "ticket_type": t.ticket_type,
            "status": t.status,
            "priority": t.priority,
            "department": t.department,
            "conversation_id": str(t.conversation_id) if t.conversation_id else None,
            "contact_id": str(t.contact_id) if t.contact_id else None,
            "contact_name": t.contact.person_name if t.contact else None,
            "contact_identifier": t.contact.value if t.contact else None,
            "tracker_id": str(t.tracker_id) if t.tracker_id else None,
            "tracker_number": t.tracker.ticket_number if t.tracker else None,
            "tracker_title": t.tracker.title if t.tracker else None,
            "assigned_to_id": str(t.assigned_to_id) if t.assigned_to_id else None,
            "assigned_to_name": t.assigned_to.name if t.assigned_to else None,
            "resolution_summary": t.resolution_summary,
            "customer_broadcast_sent": t.customer_broadcast_sent,
            "resolved_at": t.resolved_at.isoformat() if t.resolved_at else None,
            "created_at": t.created_at.isoformat(),
            "updated_at": t.updated_at.isoformat(),
            "activities": activities,
            "child_tickets": children,
            "child_count": len(children),
        }

    @staticmethod
    async def update_ticket(
        db: AsyncSession,
        ticket_id: uuid.UUID,
        title: str | None = None,
        description: str | None = None,
        status: str | None = None,
        priority: str | None = None,
        department: str | None = None,
        assigned_to_id: uuid.UUID | None = None,
        resolution_summary: str | None = None,
        author_name: str = "Agent",
    ) -> Ticket | None:
        """Updates ticket fields and records audit activities."""
        ticket = await db.get(Ticket, ticket_id)
        if not ticket:
            return None

        # Track modifications for activity log
        changes: list[str] = []

        if title is not None and title != ticket.title:
            ticket.title = title
            changes.append(f"Title updated to '{title}'")

        if description is not None and description != ticket.description:
            ticket.description = description
            changes.append("Description updated")

        if priority is not None and priority != ticket.priority:
            changes.append(f"Priority changed from {ticket.priority.upper()} to {priority.upper()}")
            ticket.priority = priority

        if department is not None and department != ticket.department:
            changes.append(
                f"Department re-routed from {ticket.department.upper()} to {department.upper()}"
            )
            ticket.department = department

        if assigned_to_id is not None and assigned_to_id != ticket.assigned_to_id:
            ticket.assigned_to_id = assigned_to_id
            changes.append("Assigned to team member")

        if status is not None and status != ticket.status:
            old_status = ticket.status
            ticket.status = status
            changes.append(f"Status changed from {old_status.upper()} to {status.upper()}")
            if status == TicketStatus.RESOLVED.value:
                ticket.resolved_at = datetime.now(UTC)
            elif old_status == TicketStatus.RESOLVED.value:
                ticket.resolved_at = None

        if resolution_summary is not None:
            ticket.resolution_summary = resolution_summary
            changes.append(f"Resolution notes added: '{resolution_summary}'")

        ticket.updated_at = datetime.now(UTC)

        # Log change activities
        for change in changes:
            act = TicketActivity(
                ticket_id=ticket.id,
                author_name=author_name,
                activity_type=ActivityType.STATUS_CHANGE.value
                if "Status" in change
                else ActivityType.COMMENT.value,
                message=change,
            )
            db.add(act)

        await db.commit()
        await db.refresh(ticket)
        return ticket

    @staticmethod
    async def add_activity(
        db: AsyncSession,
        ticket_id: uuid.UUID,
        message: str,
        activity_type: str = ActivityType.BACK_OFFICE_NOTE.value,
        author_id: uuid.UUID | None = None,
        author_name: str = "Agent",
        metadata: dict[str, Any] | None = None,
    ) -> TicketActivity | None:
        """Adds an internal note or collaboration update to the ticket thread."""
        ticket = await db.get(Ticket, ticket_id)
        if not ticket:
            return None

        act = TicketActivity(
            ticket_id=ticket.id,
            author_id=author_id,
            author_name=author_name,
            activity_type=activity_type,
            message=message,
            activity_metadata=metadata or {},
        )
        ticket.updated_at = datetime.now(UTC)
        db.add(act)
        await db.commit()
        await db.refresh(act)
        return act

    @staticmethod
    async def link_to_tracker(
        db: AsyncSession,
        ticket_id: uuid.UUID,
        tracker_id: uuid.UUID,
        author_name: str = "Agent",
    ) -> Ticket | None:
        """Links a customer issue or back-office ticket to a parent tracker ticket."""
        ticket = await db.get(Ticket, ticket_id)
        tracker = await db.get(Ticket, tracker_id)
        if not ticket or not tracker:
            return None

        ticket.tracker_id = tracker_id
        ticket.updated_at = datetime.now(UTC)

        # Record activity on both
        act_child = TicketActivity(
            ticket_id=ticket.id,
            author_name=author_name,
            activity_type=ActivityType.LINKED_TRACKER.value,
            message=f"Linked to Tracker Incident {tracker.ticket_number}: '{tracker.title}'",
            activity_metadata={
                "tracker_id": str(tracker_id),
                "tracker_number": tracker.ticket_number,
            },
        )
        act_tracker = TicketActivity(
            ticket_id=tracker.id,
            author_name=author_name,
            activity_type=ActivityType.LINKED_TRACKER.value,
            message=f"Linked customer issue {ticket.ticket_number}: '{ticket.title}'",
            activity_metadata={
                "child_ticket_id": str(ticket.id),
                "ticket_number": ticket.ticket_number,
            },
        )
        db.add(act_child)
        db.add(act_tracker)

        await db.commit()
        await db.refresh(ticket)
        return ticket

    @staticmethod
    async def broadcast_tracker_update(
        db: AsyncSession,
        tracker_id: uuid.UUID,
        broadcast_message: str,
        resolve_all_children: bool = False,
        send_customer_message: bool = True,
        author_name: str = "Incident Manager",
    ) -> dict[str, Any]:
        """
        Widespread Problem Resolution & Broadcast:
        Dispatches updates across all child tickets and customer conversations linked
        to this tracker ticket, optionally resolving them all in one click.
        """
        tracker = await db.get(Ticket, tracker_id)
        if not tracker:
            return {"error": "Tracker ticket not found"}

        # Fetch child tickets with their conversations
        stmt = (
            select(Ticket)
            .options(
                selectinload(Ticket.conversation),
                selectinload(Ticket.contact),
            )
            .where(Ticket.tracker_id == tracker_id)
        )
        result = await db.execute(stmt)
        children = result.scalars().all()

        impacted_tickets_count = len(children)
        messages_sent_count = 0
        resolved_count = 0

        # Activity on the tracker ticket
        tracker_act = TicketActivity(
            ticket_id=tracker.id,
            author_name=author_name,
            activity_type=ActivityType.BROADCAST.value,
            message=f"Broadcast dispatched to {impacted_tickets_count} linked customer tickets: '{broadcast_message}'",
            activity_metadata={
                "impacted_count": impacted_tickets_count,
                "resolve_all_children": resolve_all_children,
            },
        )
        db.add(tracker_act)

        for child in children:
            # 1. Add broadcast activity to child ticket
            child_act = TicketActivity(
                ticket_id=child.id,
                author_name=author_name,
                activity_type=ActivityType.BROADCAST.value,
                message=f"[Tracker Broadcast - {tracker.ticket_number}] {broadcast_message}",
                activity_metadata={"tracker_id": str(tracker.id)},
            )
            db.add(child_act)

            # 2. If customer message requested and conversation exists, post into customer chat
            if send_customer_message and child.conversation_id:
                msg_channel = (
                    child.contact.channel
                    if (child.contact and child.contact.channel)
                    else ChannelType.EMAIL
                )
                outbound_msg = Message(
                    conversation_id=child.conversation_id,
                    direction=MessageDirection.OUTBOUND,
                    channel=msg_channel,
                    sender_type="agent",
                    body=broadcast_message,
                    status=MessageStatus.SENT,
                )
                db.add(outbound_msg)
                child.customer_broadcast_sent = True
                messages_sent_count += 1

            # 3. If resolving all children
            if resolve_all_children:
                child.status = TicketStatus.RESOLVED.value
                child.resolved_at = datetime.now(UTC)
                child.resolution_summary = (
                    f"Resolved via Tracker {tracker.ticket_number}: {broadcast_message}"
                )
                resolved_count += 1

                res_act = TicketActivity(
                    ticket_id=child.id,
                    author_name=author_name,
                    activity_type=ActivityType.RESOLUTION.value,
                    message="Automatically resolved following tracker incident resolution.",
                )
                db.add(res_act)

        # Update tracker ticket status if requested
        if resolve_all_children:
            tracker.status = TicketStatus.RESOLVED.value
            tracker.resolved_at = datetime.now(UTC)
            tracker.resolution_summary = broadcast_message
            tracker_res_act = TicketActivity(
                ticket_id=tracker.id,
                author_name=author_name,
                activity_type=ActivityType.RESOLUTION.value,
                message="Tracker Incident resolved and closed across all linked issues.",
            )
            db.add(tracker_res_act)

        tracker.updated_at = datetime.now(UTC)
        await db.commit()

        return {
            "tracker_id": str(tracker_id),
            "tracker_number": tracker.ticket_number,
            "impacted_tickets": impacted_tickets_count,
            "messages_sent": messages_sent_count,
            "resolved_tickets": resolved_count,
            "tracker_resolved": resolve_all_children,
        }

    @staticmethod
    async def get_stats_summary(db: AsyncSession) -> dict[str, Any]:
        """Aggregates ticket stats, back-office collaboration load, and tracker impact."""
        # Total counts
        total_q = await db.execute(select(func.count(Ticket.id)))
        total_tickets = total_q.scalar_one() or 0

        open_q = await db.execute(
            select(func.count(Ticket.id)).where(
                Ticket.status.in_(
                    [
                        TicketStatus.OPEN.value,
                        TicketStatus.IN_PROGRESS.value,
                        TicketStatus.WAITING_ON_BACK_OFFICE.value,
                    ]
                )
            )
        )
        open_tickets = open_q.scalar_one() or 0

        cust_q = await db.execute(
            select(func.count(Ticket.id)).where(
                Ticket.ticket_type == TicketType.CUSTOMER_ISSUE.value
            )
        )
        customer_issues = cust_q.scalar_one() or 0

        bo_q = await db.execute(
            select(func.count(Ticket.id)).where(Ticket.ticket_type == TicketType.BACK_OFFICE.value)
        )
        back_office_tickets = bo_q.scalar_one() or 0

        tracker_q = await db.execute(
            select(func.count(Ticket.id)).where(Ticket.ticket_type == TicketType.TRACKER.value)
        )
        tracker_tickets = tracker_q.scalar_one() or 0

        resolved_q = await db.execute(
            select(func.count(Ticket.id)).where(Ticket.status == TicketStatus.RESOLVED.value)
        )
        resolved_tickets = resolved_q.scalar_one() or 0

        # Department breakdown
        dept_q = await db.execute(
            select(Ticket.department, func.count(Ticket.id)).group_by(Ticket.department)
        )
        dept_breakdown = {dept: count for dept, count in dept_q.all()}

        # Distinct impacted contacts attached to child tickets of trackers
        impacted_q = await db.execute(
            select(func.count(func.distinct(Ticket.contact_id))).where(
                Ticket.tracker_id.isnot(None)
            )
        )
        impacted_customers = impacted_q.scalar_one() or 0

        return {
            "total_tickets": total_tickets,
            "open_tickets": open_tickets,
            "customer_issues": customer_issues,
            "back_office_tickets": back_office_tickets,
            "tracker_tickets": tracker_tickets,
            "resolved_tickets": resolved_tickets,
            "department_breakdown": dept_breakdown,
            "impacted_customers_count": impacted_customers,
        }

    @staticmethod
    async def seed_demo_tickets_if_empty(db: AsyncSession) -> None:
        """Seeds representative tracker, back-office, and customer tickets if none exist."""
        count_q = await db.execute(select(func.count(Ticket.id)))
        if (count_q.scalar_one() or 0) > 0:
            return

        # Check if we have contacts and conversations to attach to
        conv_q = await db.execute(select(Conversation).limit(2))
        conversations = conv_q.scalars().all()
        conv_1_id = conversations[0].id if len(conversations) > 0 else None
        conv_2_id = conversations[1].id if len(conversations) > 1 else None

        contact_q = await db.execute(select(Contact).limit(2))
        contacts = contact_q.scalars().all()
        cont_1_id = contacts[0].id if len(contacts) > 0 else None
        cont_2_id = contacts[1].id if len(contacts) > 1 else None

        # 1. Tracker Ticket: Widespread Payment Gateway Issue
        tracker_1 = Ticket(
            ticket_number="TCK-1001",
            title="Widespread Incident: US-East Stripe Payment 504 Gateway Timeouts",
            description="Customers checking out between 14:00 - 15:30 UTC receive 504 timeouts while card is occasionally pre-authorized. Engineering investigating gateway retry spike.",
            ticket_type=TicketType.TRACKER.value,
            status=TicketStatus.IN_PROGRESS.value,
            priority=TicketPriority.URGENT.value,
            department=TicketDepartment.ENGINEERING.value,
        )
        db.add(tracker_1)
        await db.flush()

        act_t1 = TicketActivity(
            ticket_id=tracker_1.id,
            author_name="Captain AI Incident Sentinel",
            activity_type=ActivityType.STATUS_CHANGE.value,
            message="Tracker Ticket created after detecting 8 checkout timeout complaints in 15 minutes.",
        )
        db.add(act_t1)

        # 2. Child Customer Issue 1
        child_1 = Ticket(
            ticket_number="TCK-1002",
            title="Card charged $89.00 but order showed checkout error",
            description="Customer states their credit card shows a pending $89 charge but no order confirmation was sent or received.",
            ticket_type=TicketType.CUSTOMER_ISSUE.value,
            status=TicketStatus.OPEN.value,
            priority=TicketPriority.HIGH.value,
            department=TicketDepartment.SUPPORT.value,
            conversation_id=conv_1_id,
            contact_id=cont_1_id,
            tracker_id=tracker_1.id,
        )
        db.add(child_1)
        await db.flush()

        act_c1 = TicketActivity(
            ticket_id=child_1.id,
            author_name="Sarah (Support)",
            activity_type=ActivityType.COMMENT.value,
            message="Customer shared screenshot of bank app pending charge. Linked to widespread payment incident tracker.",
        )
        db.add(act_c1)

        # 3. Child Customer Issue 2
        child_2 = Ticket(
            ticket_number="TCK-1003",
            title="Cart emptied on pay button press without confirmation",
            description="User was purchasing Annual VIP subscription. Cart cleared and returned to home page without order receipt.",
            ticket_type=TicketType.CUSTOMER_ISSUE.value,
            status=TicketStatus.OPEN.value,
            priority=TicketPriority.HIGH.value,
            department=TicketDepartment.SUPPORT.value,
            conversation_id=conv_2_id,
            contact_id=cont_2_id,
            tracker_id=tracker_1.id,
        )
        db.add(child_2)
        await db.flush()

        # 4. Back-Office Collaboration: Finance Investigation
        bo_1 = Ticket(
            ticket_number="TCK-1004",
            title="Finance Back-Office: Audit duplicated authorizations for incident TCK-1001",
            description="Cross-departmental collaboration: Finance team must review settlement batch #8821 and release authorization holds for impacted customers.",
            ticket_type=TicketType.BACK_OFFICE.value,
            status=TicketStatus.IN_PROGRESS.value,
            priority=TicketPriority.HIGH.value,
            department=TicketDepartment.FINANCE.value,
            tracker_id=tracker_1.id,
        )
        db.add(bo_1)
        await db.flush()

        act_bo1 = TicketActivity(
            ticket_id=bo_1.id,
            author_name="David (Finance)",
            activity_type=ActivityType.BACK_OFFICE_NOTE.value,
            message="Batch #8821 holds identified. 14 authorization holds will void automatically by 18:00 UTC if settlement not captured.",
        )
        db.add(act_bo1)

        # 5. Back-Office Collaboration: Logistics inspection
        bo_2 = Ticket(
            ticket_number="TCK-1005",
            title="Logistics & Warehouse: Batch #412 Glass Packaging Reinforcement",
            description="Warehouse floor reported 3 cracked bottles during pallet strapping. Logistics team to inspect reinforced bubble padding before dispatch.",
            ticket_type=TicketType.BACK_OFFICE.value,
            status=TicketStatus.WAITING_ON_BACK_OFFICE.value,
            priority=TicketPriority.MEDIUM.value,
            department=TicketDepartment.LOGISTICS.value,
        )
        db.add(bo_2)
        await db.flush()

        act_bo2 = TicketActivity(
            ticket_id=bo_2.id,
            author_name="Marcus (Warehouse Lead)",
            activity_type=ActivityType.BACK_OFFICE_NOTE.value,
            message="Inspection in progress on packing line 3. Sample 25 boxes tested; added double-wall outer carton.",
        )
        db.add(act_bo2)

        # 6. Tracker Ticket: Carrier Weather Delay
        tracker_2 = Ticket(
            ticket_number="TCK-1006",
            title="Carrier Incident: Winter Blizzard Delay - Midwest Regional Hubs",
            description="FedEx and UPS have declared weather exceptions for Illinois and Ohio hubs. Outbound shipments delayed by 48-72 hours.",
            ticket_type=TicketType.TRACKER.value,
            status=TicketStatus.OPEN.value,
            priority=TicketPriority.MEDIUM.value,
            department=TicketDepartment.LOGISTICS.value,
        )
        db.add(tracker_2)

        await db.commit()
