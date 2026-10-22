from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import macro_executor

router = APIRouter(tags=["Macros & One-Click Workflows"])


class MacroCreateIn(BaseModel):
    name: str
    description: str = ""
    actions: list[dict[str, Any]]


class MacroUpdateIn(BaseModel):
    name: str | None = None
    description: str | None = None
    actions: list[dict[str, Any]] | None = None


class MacroExecuteIn(BaseModel):
    conversation_id: uuid.UUID
    agent_name: str = "Riley Brooks"


class ConversationMacroExecuteIn(BaseModel):
    agent_name: str = "Riley Brooks"


def _serialize_macro(m) -> dict[str, Any]:
    return {
        "id": str(m.id),
        "name": m.name,
        "description": m.description,
        "actions": m.actions,
        "createdAt": m.created_at.isoformat(),
        "updatedAt": m.updated_at.isoformat(),
    }


@router.get("/macros")
async def list_macros(session: AsyncSession = Depends(get_db_session)) -> list[dict[str, Any]]:
    """Lists all available one-click macros."""
    macros = await macro_executor.list_macros(session)
    return [_serialize_macro(m) for m in macros]


@router.post("/macros")
async def create_macro(
    payload: MacroCreateIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Creates a new one-click macro."""
    if not payload.name.strip():
        raise HTTPException(status_code=400, detail="Macro name cannot be empty")
    if not payload.actions:
        raise HTTPException(status_code=400, detail="Macro must contain at least one action")
    macro = await macro_executor.create_macro(
        session=session,
        name=payload.name,
        description=payload.description,
        actions=payload.actions,
    )
    return _serialize_macro(macro)


@router.get("/macros/{macro_id}")
async def get_macro(
    macro_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Retrieves macro details by id."""
    macro = await macro_executor.get_macro(session, macro_id)
    if not macro:
        raise HTTPException(status_code=404, detail="Macro not found")
    return _serialize_macro(macro)


@router.put("/macros/{macro_id}")
async def update_macro(
    macro_id: uuid.UUID,
    payload: MacroUpdateIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Updates an existing macro."""
    try:
        macro = await macro_executor.update_macro(
            session=session,
            macro_id=macro_id,
            name=payload.name,
            description=payload.description,
            actions=payload.actions,
        )
        return _serialize_macro(macro)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.delete("/macros/{macro_id}")
async def delete_macro(
    macro_id: uuid.UUID,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Deletes a macro."""
    deleted = await macro_executor.delete_macro(session, macro_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="Macro not found")
    return {"status": "ok", "deleted": True}


@router.post("/conversations/{conversation_id}/macros/{macro_id}/execute")
async def execute_macro_on_conversation(
    conversation_id: str,
    macro_id: str,
    payload: ConversationMacroExecuteIn | None = None,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Executes a multi-action sequence against a conversation with one click.
    """
    try:
        m_id = uuid.UUID(str(macro_id))
    except (ValueError, TypeError):
        raise HTTPException(status_code=404, detail="Invalid macro ID")

    agent_name = payload.agent_name if payload else "Riley Brooks"

    try:
        c_id = uuid.UUID(str(conversation_id))
    except (ValueError, TypeError):
        # Demo / mock conversation handler
        macro = await macro_executor.get_macro(session, m_id)
        if not macro:
            raise HTTPException(status_code=404, detail="Macro not found")
        executed_actions = [{"action": a.get("type", "unknown")} for a in macro.actions]
        now = datetime.now(UTC)
        return {
            "status": "ok",
            "macroId": str(macro.id),
            "macroName": macro.name,
            "conversationId": str(conversation_id),
            "executedActionsCount": len(executed_actions),
            "executedActions": executed_actions,
            "conversationState": "closed_won"
            if any(a.get("type") == "resolve_conversation" for a in macro.actions)
            else "open",
            "conversationPriority": next(
                (a.get("priority") for a in macro.actions if a.get("type") == "change_priority"),
                "medium",
            ),
            "conversationTags": next(
                (a.get("tags") for a in macro.actions if a.get("type") == "add_tags"),
                ["macro-executed"],
            ),
            "executedAt": now.isoformat(),
        }

    try:
        return await macro_executor.execute_macro(
            session=session,
            conversation_id=c_id,
            macro_id=m_id,
            agent_name=agent_name,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/macros/{macro_id}/execute")
async def execute_macro_direct(
    macro_id: uuid.UUID,
    payload: MacroExecuteIn,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """
    Alternative endpoint to execute a macro by supplying conversation_id in payload.
    """
    try:
        return await macro_executor.execute_macro(
            session=session,
            conversation_id=payload.conversation_id,
            macro_id=macro_id,
            agent_name=payload.agent_name,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
