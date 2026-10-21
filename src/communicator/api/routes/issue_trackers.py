from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import issue_tracker_service

router = APIRouter(prefix="/integrations/trackers", tags=["issue-trackers-and-slack"])


class UpdateTrackerConfigRequest(BaseModel):
    account_name: str | None = None
    is_enabled: bool | None = None
    test_mode: bool | None = None
    credentials: dict[str, Any] = Field(default_factory=dict)
    config: dict[str, Any] = Field(default_factory=dict)


class CreateIssueRequest(BaseModel):
    provider: str = Field(..., description="linear | github | jira")
    title: str = Field(..., description="Issue title")
    description: str = Field("", description="Issue description or customer transcript context")
    priority: str = Field("medium", description="urgent | high | medium | low")
    issue_type: str = Field("bug", description="bug | feature | task")
    conversation_id: uuid.UUID | None = None
    ticket_id: uuid.UUID | None = None
    actor_email: str = Field("agent@newax.com", description="Agent creating the issue")
    assigned_to: str = Field("Riley Brooks", description="Assignee name")


class SlackSyncRequest(BaseModel):
    conversation_id: uuid.UUID = Field(..., description="Conversation UUID to sync with Slack")
    channel_name: str | None = Field(None, description="Slack channel e.g. #customer-escalations")
    note: str | None = Field(None, description="Optional note or context to post to Slack thread")
    author: str = Field("Riley Brooks", description="Author name")


@router.get("/configs")
async def list_configs(session: AsyncSession = Depends(get_db_session)) -> list[dict[str, Any]]:
    """List configurations for Linear, GitHub, Jira, and Slack."""
    cfgs = await issue_tracker_service.list_tracker_configs(session)
    return [issue_tracker_service.format_tracker_config(c) for c in cfgs]


@router.get("/configs/{provider}")
async def get_config(
    provider: str, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Get single provider configuration."""
    cfg = await issue_tracker_service.get_tracker_config(session, provider)
    if not cfg:
        raise HTTPException(status_code=404, detail=f"Provider {provider} not found")
    return issue_tracker_service.format_tracker_config(cfg)


@router.post("/configs/{provider}")
async def update_config(
    provider: str,
    payload: UpdateTrackerConfigRequest,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Update credentials, default repository/project, or enabled state for a tracker."""
    updates = payload.model_dump(exclude_unset=True)
    cfg = await issue_tracker_service.update_tracker_config(session, provider, updates)
    return issue_tracker_service.format_tracker_config(cfg)


@router.post("/test/{provider}")
async def test_connection(
    provider: str, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Test connection to Linear, GitHub, Jira, or Slack."""
    return await issue_tracker_service.test_tracker_connection(session, provider)


@router.get("/linked")
async def list_linked_issues(
    conversation_id: uuid.UUID | None = Query(None),
    ticket_id: uuid.UUID | None = Query(None),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    """Retrieve all linked Linear, GitHub, and Jira issues for a conversation or ticket."""
    issues = await issue_tracker_service.list_linked_issues(
        session, conversation_id=conversation_id, ticket_id=ticket_id
    )
    return [issue_tracker_service.format_linked_issue(i) for i in issues]


@router.post("/create-issue")
async def create_issue(
    payload: CreateIssueRequest, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """1-Click creation and linking of an issue in Linear, GitHub, or Jira."""
    try:
        issue = await issue_tracker_service.create_linked_issue(
            session=session,
            provider=payload.provider,
            title=payload.title,
            description=payload.description,
            priority=payload.priority,
            issue_type=payload.issue_type,
            conversation_id=payload.conversation_id,
            ticket_id=payload.ticket_id,
            actor_email=payload.actor_email,
            assigned_to=payload.assigned_to,
        )
        return issue_tracker_service.format_linked_issue(issue)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.delete("/linked/{issue_id}")
async def delete_linked_issue(
    issue_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Unlink an issue from conversation or ticket."""
    success = await issue_tracker_service.unlink_issue(session, issue_id)
    if not success:
        raise HTTPException(status_code=404, detail="Linked issue not found")
    return {"success": True, "deleted_id": str(issue_id)}


@router.post("/linked/{issue_id}/sync")
async def sync_issue_status(
    issue_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Refresh issue status from provider."""
    issue = await issue_tracker_service.sync_issue_status(session, issue_id)
    if not issue:
        raise HTTPException(status_code=404, detail="Linked issue not found")
    return issue_tracker_service.format_linked_issue(issue)


@router.post("/slack/sync")
async def sync_slack_thread(
    payload: SlackSyncRequest, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Bridge conversation to a Slack channel and thread."""
    try:
        return await issue_tracker_service.sync_conversation_to_slack(
            session=session,
            conversation_id=payload.conversation_id,
            channel_name=payload.channel_name,
            note=payload.note,
            author=payload.author,
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/slack/status/{conversation_id}")
async def get_slack_sync_status(
    conversation_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)
) -> dict[str, Any]:
    """Get active Slack thread sync info for conversation."""
    status = await issue_tracker_service.get_slack_sync_status(session, conversation_id)
    return status or {"is_synced": False, "conversation_id": str(conversation_id)}
