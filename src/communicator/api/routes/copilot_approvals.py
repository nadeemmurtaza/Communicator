from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services.copilot_approval_service import (
    approve_request,
    create_approval_request,
    evaluate_and_trigger_refund_approval,
    get_approval_request,
    list_approval_requests,
    provide_teammate_info,
    reject_request,
)

router = APIRouter(prefix="/copilot/approvals", tags=["Copilot Teammate Approvals"])


class CreateApprovalIn(BaseModel):
    conversation_id: uuid.UUID
    title: str
    proposed_reply: str
    request_type: str = "refund"
    action_data: dict[str, Any] = {}
    internal_note: str = ""
    question_to_teammate: str | None = None
    assigned_to_id: uuid.UUID | None = None


class ApproveIn(BaseModel):
    reviewer_name: str = "Teammate"
    amended_reply: str | None = None
    execute_send: bool = True


class RejectIn(BaseModel):
    reviewer_name: str = "Teammate"
    feedback: str = ""


class ProvideInfoIn(BaseModel):
    reviewer_name: str = "Teammate"
    teammate_answer: str


class EvaluateTriggerIn(BaseModel):
    conversation_id: uuid.UUID
    customer_message: str


@router.get("")
async def list_approvals(
    conversation_id: uuid.UUID | None = Query(default=None),
    status: str | None = Query(default=None),
    request_type: str | None = Query(default=None),
    session: AsyncSession = Depends(get_db_session),
):
    """
    Returns pending and historical approval requests across conversations.
    """
    return await list_approval_requests(
        session=session,
        conversation_id=conversation_id,
        status=status,
        request_type=request_type,
    )


@router.post("")
async def create_approval(
    payload: CreateApprovalIn, session: AsyncSession = Depends(get_db_session)
):
    """
    AI Agent creates an approval request (e.g. asking teammate to approve a refund reply)
    or asks for specific information.
    """
    try:
        return await create_approval_request(
            session=session,
            conversation_id=payload.conversation_id,
            title=payload.title,
            proposed_reply=payload.proposed_reply,
            request_type=payload.request_type,
            action_data=payload.action_data,
            internal_note=payload.internal_note,
            question_to_teammate=payload.question_to_teammate,
            assigned_to_id=payload.assigned_to_id,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.get("/{id}")
async def get_approval(id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    try:
        return await get_approval_request(session=session, approval_id=id)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/{id}/approve")
async def approve_approval_request(
    id: uuid.UUID,
    payload: ApproveIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Teammate approves the proposed action/reply.
    The AI Agent dispatches the message to the customer and finishes the conversation itself.
    """
    try:
        return await approve_request(
            session=session,
            approval_id=id,
            reviewer_name=payload.reviewer_name,
            amended_reply=payload.amended_reply,
            execute_send=payload.execute_send,
        )
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/{id}/reject")
async def reject_approval_request(
    id: uuid.UUID,
    payload: RejectIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Teammate rejects the approval request with feedback.
    """
    try:
        return await reject_request(
            session=session,
            approval_id=id,
            reviewer_name=payload.reviewer_name,
            feedback=payload.feedback,
        )
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/{id}/provide-info")
async def answer_info_request(
    id: uuid.UUID,
    payload: ProvideInfoIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Teammate provides the requested piece of information.
    The AI Agent synthesizes the answer and finishes the conversation itself.
    """
    try:
        return await provide_teammate_info(
            session=session,
            approval_id=id,
            reviewer_name=payload.reviewer_name,
            teammate_answer=payload.teammate_answer,
        )
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/evaluate-trigger")
async def evaluate_trigger(
    payload: EvaluateTriggerIn,
    session: AsyncSession = Depends(get_db_session),
):
    """
    Evaluates incoming customer text. If refund is detected, triggers a pending approval request.
    """
    res = await evaluate_and_trigger_refund_approval(
        session=session,
        conversation_id=payload.conversation_id,
        customer_message=payload.customer_message,
    )
    return {"triggered": res is not None, "approval": res}
