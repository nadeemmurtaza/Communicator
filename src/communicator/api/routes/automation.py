from __future__ import annotations

import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.automation_rule import AutomationRule, RuleActionType, RuleMatchType

router = APIRouter(prefix="/automation-rules", tags=["Automation"])


class RuleIn(BaseModel):
    name: str
    trigger_keyword: str
    match_type: RuleMatchType = RuleMatchType.CONTAINS
    action_type: RuleActionType
    action_value: str = ""
    priority: int = 0
    is_active: bool = True


def _serialize(rule: AutomationRule) -> dict:
    return {
        "id": str(rule.id),
        "name": rule.name,
        "triggerKeyword": rule.trigger_keyword,
        "matchType": rule.match_type.value,
        "actionType": rule.action_type.value,
        "actionValue": rule.action_value,
        "priority": rule.priority,
        "isActive": rule.is_active,
    }


@router.get("")
async def list_rules(session: AsyncSession = Depends(get_db_session)):
    rules = (
        (await session.execute(select(AutomationRule).order_by(AutomationRule.priority.desc())))
        .scalars()
        .all()
    )
    return [_serialize(r) for r in rules]


@router.post("")
async def create_rule(payload: RuleIn, session: AsyncSession = Depends(get_db_session)):
    rule = AutomationRule(**payload.model_dump())
    session.add(rule)
    await session.flush()
    return _serialize(rule)


@router.put("/{rule_id}")
async def update_rule(
    rule_id: uuid.UUID, payload: RuleIn, session: AsyncSession = Depends(get_db_session)
):
    rule = await session.get(AutomationRule, rule_id)
    if rule is None:
        raise HTTPException(status_code=404, detail="Rule not found")
    for key, value in payload.model_dump().items():
        setattr(rule, key, value)
    await session.flush()
    return _serialize(rule)


@router.delete("/{rule_id}")
async def delete_rule(rule_id: uuid.UUID, session: AsyncSession = Depends(get_db_session)):
    rule = await session.get(AutomationRule, rule_id)
    if rule is None:
        raise HTTPException(status_code=404, detail="Rule not found")
    await session.delete(rule)
    await session.flush()
    return {"status": "deleted"}
