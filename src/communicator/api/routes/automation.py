from __future__ import annotations

import json
import uuid

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.automation_rule import AutomationRule, RuleActionType, RuleMatchType
from communicator.models.conversation import Conversation
from communicator.services.whatsapp_automations import (
    match_advanced_rule,
    run_inactivity_nudges,
    seed_default_whatsapp_automations,
)

router = APIRouter(prefix="/automation-rules", tags=["Automation"])


class RuleIn(BaseModel):
    name: str
    description: str = ""
    trigger_keyword: str
    match_type: RuleMatchType = RuleMatchType.CONTAINS
    action_type: RuleActionType
    action_value: str = ""
    priority: int = 0
    is_active: bool = True


class TestRuleIn(BaseModel):
    text: str = ""
    payload_id: str | None = None
    intent: str | None = None


def _serialize(rule: AutomationRule) -> dict:
    return {
        "id": str(rule.id),
        "name": rule.name,
        "description": rule.description,
        "triggerKeyword": rule.trigger_keyword,
        "matchType": rule.match_type.value,
        "actionType": rule.action_type.value,
        "actionValue": rule.action_value,
        "priority": rule.priority,
        "isActive": rule.is_active,
        "executionCount": rule.execution_count,
        "lastTriggeredAt": rule.last_triggered_at.isoformat() if rule.last_triggered_at else None,
        "createdAt": rule.created_at.isoformat(),
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


@router.post("/seed-defaults")
async def seed_defaults(session: AsyncSession = Depends(get_db_session)):
    """Installs all 11 battle-tested WhatsApp automations in one click."""
    count = await seed_default_whatsapp_automations(session)
    return {
        "status": "ok",
        "installedCount": count,
        "message": f"Successfully installed {count} default WhatsApp automations.",
    }


@router.post("/test")
async def test_rule(payload: TestRuleIn, session: AsyncSession = Depends(get_db_session)):
    """
    Simulates an incoming WhatsApp message or button click and tests which rule matches
    without sending any real messages.
    """
    rule = await match_advanced_rule(
        session, payload.text, payload_id=payload.payload_id, intent=payload.intent
    )
    if rule is None:
        return {
            "matched": False,
            "message": "No automation rule matched. AI reply engine will handle this message.",
        }

    preview = {}
    try:
        if rule.action_value.strip().startswith(("{", "[")):
            preview = json.loads(rule.action_value)
    except Exception:  # noqa: BLE001, S110
        pass

    return {
        "matched": True,
        "rule": _serialize(rule),
        "actionType": rule.action_type.value,
        "actionValue": rule.action_value,
        "preview": preview,
    }


@router.post("/run-nudges")
async def trigger_inactivity_nudges(session: AsyncSession = Depends(get_db_session)):
    """Advances 24-hour inactivity re-engagement checks for WhatsApp leads."""
    nudged = await run_inactivity_nudges(session)
    return {
        "status": "ok",
        "nudgedCount": len(nudged),
        "conversationIds": [str(c_id) for c_id in nudged],
    }


@router.get("/stats")
async def get_automation_stats(session: AsyncSession = Depends(get_db_session)):
    """Returns analytics on automation rule execution and CSAT performance."""
    total_rules = (
        await session.execute(select(func.count()).select_from(AutomationRule))
    ).scalar_one()
    active_rules = (
        await session.execute(
            select(func.count())
            .select_from(AutomationRule)
            .where(AutomationRule.is_active.is_(True))
        )
    ).scalar_one()
    total_executions = (
        await session.execute(select(func.coalesce(func.sum(AutomationRule.execution_count), 0)))
    ).scalar_one()

    # CSAT stats
    csat_avg = (
        await session.execute(
            select(func.avg(Conversation.csat_score)).where(Conversation.csat_score.is_not(None))
        )
    ).scalar_one()
    csat_count = (
        await session.execute(
            select(func.count())
            .select_from(Conversation)
            .where(Conversation.csat_score.is_not(None))
        )
    ).scalar_one()

    return {
        "totalRules": total_rules,
        "activeRules": active_rules,
        "totalExecutions": int(total_executions),
        "csatAverage": round(float(csat_avg), 2) if csat_avg else None,
        "csatResponsesCount": csat_count,
    }


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
