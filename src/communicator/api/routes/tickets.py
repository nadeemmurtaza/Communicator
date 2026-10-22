import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.ticket import (
    ActivityType,
    TicketDepartment,
    TicketPriority,
    TicketStatus,
    TicketType,
)
from communicator.services.ticket_service import TicketService

router = APIRouter(prefix="/tickets", tags=["tickets"])


# Pydantic Schemas
class TicketCreateRequest(BaseModel):
    title: str = Field(..., min_length=3, max_length=255)
    description: str = Field(..., min_length=3)
    ticket_type: str = Field(default=TicketType.CUSTOMER_ISSUE.value)
    department: str = Field(default=TicketDepartment.SUPPORT.value)
    priority: str = Field(default=TicketPriority.MEDIUM.value)
    status: str = Field(default=TicketStatus.OPEN.value)
    conversation_id: uuid.UUID | None = None
    contact_id: uuid.UUID | None = None
    tracker_id: uuid.UUID | None = None
    assigned_to_id: uuid.UUID | None = None
    author_name: str = "Agent"


class TicketUpdateRequest(BaseModel):
    title: str | None = None
    description: str | None = None
    status: str | None = None
    priority: str | None = None
    department: str | None = None
    assigned_to_id: uuid.UUID | None = None
    resolution_summary: str | None = None
    author_name: str = "Agent"


class TicketActivityCreateRequest(BaseModel):
    message: str = Field(..., min_length=1)
    activity_type: str = Field(default=ActivityType.BACK_OFFICE_NOTE.value)
    author_id: uuid.UUID | None = None
    author_name: str = "Agent"
    metadata: dict[str, Any] = Field(default_factory=dict)


class LinkTrackerRequest(BaseModel):
    tracker_id: uuid.UUID
    author_name: str = "Agent"


class BroadcastTrackerRequest(BaseModel):
    broadcast_message: str = Field(..., min_length=3)
    resolve_all_children: bool = False
    send_customer_message: bool = True
    author_name: str = "Incident Manager"


@router.get("", response_model=list[dict[str, Any]])
async def list_tickets(
    ticket_type: str | None = Query(None),
    status: str | None = Query(None),
    department: str | None = Query(None),
    tracker_id: uuid.UUID | None = Query(None),
    conversation_id: uuid.UUID | None = Query(None),
    search: str | None = Query(None),
    limit: int = Query(100, ge=1, le=500),
    offset: int = Query(0, ge=0),
    db: AsyncSession = Depends(get_db_session),
):
    """Lists tickets across Customer Issues, Back-Office Collaboration, and Tracker Incidents."""
    await TicketService.seed_demo_tickets_if_empty(db)
    return await TicketService.list_tickets(
        db,
        ticket_type=ticket_type,
        status=status,
        department=department,
        tracker_id=tracker_id,
        conversation_id=conversation_id,
        search=search,
        limit=limit,
        offset=offset,
    )


@router.get("/stats", response_model=dict[str, Any])
async def get_ticket_stats(db: AsyncSession = Depends(get_db_session)):
    """Summary metrics of active tickets, back-office collaboration load, and tracker impact."""
    await TicketService.seed_demo_tickets_if_empty(db)
    return await TicketService.get_stats_summary(db)


@router.get("/trackers", response_model=list[dict[str, Any]])
async def list_trackers(db: AsyncSession = Depends(get_db_session)):
    """Lists active and open tracker tickets for linking dropdowns."""
    await TicketService.seed_demo_tickets_if_empty(db)
    return await TicketService.list_tickets(db, ticket_type=TicketType.TRACKER.value)


@router.post("", status_code=status.HTTP_201_CREATED, response_model=dict[str, Any])
async def create_ticket(
    body: TicketCreateRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Creates a new Customer Issue, Back-Office task, or Tracker Ticket."""
    ticket = await TicketService.create_ticket(
        db=db,
        title=body.title,
        description=body.description,
        ticket_type=body.ticket_type,
        department=body.department,
        priority=body.priority,
        status=body.status,
        conversation_id=body.conversation_id,
        contact_id=body.contact_id,
        tracker_id=body.tracker_id,
        assigned_to_id=body.assigned_to_id,
        author_name=body.author_name,
    )
    detail = await TicketService.get_ticket(db, ticket.id)
    return detail or {"id": str(ticket.id), "ticket_number": ticket.ticket_number}


@router.get("/{ticket_id}", response_model=dict[str, Any])
async def get_ticket(
    ticket_id: uuid.UUID,
    db: AsyncSession = Depends(get_db_session),
):
    """Fetches complete ticket details, back-office collaboration thread, and linked child issues."""
    detail = await TicketService.get_ticket(db, ticket_id)
    if not detail:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Ticket {ticket_id} not found",
        )
    return detail


@router.patch("/{ticket_id}", response_model=dict[str, Any])
async def update_ticket(
    ticket_id: uuid.UUID,
    body: TicketUpdateRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Updates ticket status, priority, department, or resolution summary with audit activities."""
    updated = await TicketService.update_ticket(
        db=db,
        ticket_id=ticket_id,
        title=body.title,
        description=body.description,
        status=body.status,
        priority=body.priority,
        department=body.department,
        assigned_to_id=body.assigned_to_id,
        resolution_summary=body.resolution_summary,
        author_name=body.author_name,
    )
    if not updated:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Ticket {ticket_id} not found",
        )
    detail = await TicketService.get_ticket(db, ticket_id)
    return detail or {"id": str(ticket_id)}


@router.post(
    "/{ticket_id}/activities", status_code=status.HTTP_201_CREATED, response_model=dict[str, Any]
)
async def add_ticket_activity(
    ticket_id: uuid.UUID,
    body: TicketActivityCreateRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Adds an internal collaboration note or status update to the ticket thread."""
    act = await TicketService.add_activity(
        db=db,
        ticket_id=ticket_id,
        message=body.message,
        activity_type=body.activity_type,
        author_id=body.author_id,
        author_name=body.author_name,
        metadata=body.metadata,
    )
    if not act:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Ticket {ticket_id} not found",
        )
    return {
        "id": str(act.id),
        "ticket_id": str(act.ticket_id),
        "author_name": act.author_name,
        "activity_type": act.activity_type,
        "message": act.message,
        "metadata": act.activity_metadata,
        "created_at": act.created_at.isoformat(),
    }


@router.post("/{ticket_id}/link-tracker", response_model=dict[str, Any])
async def link_to_tracker(
    ticket_id: uuid.UUID,
    body: LinkTrackerRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Links a customer issue or back-office ticket to a parent tracker ticket."""
    linked = await TicketService.link_to_tracker(
        db=db,
        ticket_id=ticket_id,
        tracker_id=body.tracker_id,
        author_name=body.author_name,
    )
    if not linked:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Ticket or Tracker Ticket not found",
        )
    detail = await TicketService.get_ticket(db, ticket_id)
    return detail or {"id": str(ticket_id)}


@router.post("/{ticket_id}/broadcast", response_model=dict[str, Any])
async def broadcast_tracker_update(
    ticket_id: uuid.UUID,
    body: BroadcastTrackerRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Dispatches widespread broadcast to all linked child tickets and customer conversations,
    optionally resolving the entire widespread incident in one click.
    """
    result = await TicketService.broadcast_tracker_update(
        db=db,
        tracker_id=ticket_id,
        broadcast_message=body.broadcast_message,
        resolve_all_children=body.resolve_all_children,
        send_customer_message=body.send_customer_message,
        author_name=body.author_name,
    )
    if "error" in result:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=result["error"],
        )
    return result
