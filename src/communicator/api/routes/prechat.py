from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import prechat_service

router = APIRouter(prefix="/inboxes/channels/live-chat/prechat", tags=["Pre-Chat Intake Forms"])


class PreChatFieldIn(BaseModel):
    id: str | None = None
    name: str = Field(..., description="Unique field identifier slug, e.g. companySize")
    label: str = Field(..., description="Human-readable field label")
    type: str = Field("text", description="text | email | phone | select | textarea | number | checkbox")
    placeholder: str = ""
    required: bool = False
    options: list[str] = Field(default_factory=list, description="Dropdown options for select type")
    order: int = 1


class PreChatRoutingRuleIn(BaseModel):
    id: str | None = None
    field: str = Field(..., description="Field name key to evaluate, e.g. inquiryType")
    operator: str = Field("equals", description="equals | contains | not_equals")
    value: str = Field(..., description="Expected value to match")
    assign_priority: str = Field("medium", description="low | medium | high | urgent")
    assign_tag: str = Field("", description="Tag to attach to conversation")
    team_target: str = Field("", description="Target team or department")


class UpdatePreChatConfigIn(BaseModel):
    enabled: bool | None = None
    title: str | None = None
    description: str | None = None
    submit_button_text: str | None = None
    require_email: bool | None = None
    fields: list[PreChatFieldIn] | None = None
    routing_rules: list[PreChatRoutingRuleIn] | None = None


class TestIntakePayload(BaseModel):
    answers: dict[str, Any] = Field(..., description="Key-value pairs corresponding to field names")


@router.get("")
async def get_prechat_form_config(
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Retrieves live chat pre-chat intake configuration, fields, and conditional routing rules."""
    return await prechat_service.get_prechat_config(session)


@router.post("")
async def update_prechat_form_config(
    payload: UpdatePreChatConfigIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Updates pre-chat form structure, dynamic fields, validation requirements, and routing rules."""
    updates = payload.model_dump(exclude_unset=True)
    return await prechat_service.update_prechat_config(session, updates)


@router.post("/reset")
async def reset_prechat_form_config(
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Resets pre-chat intake form fields and rules back to factory defaults."""
    return await prechat_service.reset_prechat_defaults(session)


@router.post("/test")
async def test_prechat_intake_rules(
    payload: TestIntakePayload,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Simulates validation and evaluates conditional routing rules on provided intake answers."""
    cfg = await prechat_service.get_prechat_config(session)
    fields = cfg.get("fields", [])
    rules = cfg.get("routing_rules", [])

    errors = prechat_service.validate_submission_answers(
        fields, payload.answers, require_email=cfg.get("require_email", True)
    )
    routing = prechat_service.evaluate_routing_rules(rules, payload.answers)

    return {
        "valid": len(errors) == 0,
        "errors": errors,
        "routing_result": routing,
        "evaluated_answers": payload.answers,
    }
