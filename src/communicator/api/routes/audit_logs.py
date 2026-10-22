from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, Query, Request, Response
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import audit_service

router = APIRouter(prefix="/audit-logs", tags=["Audit Logs & SOC 2 Compliance"])


class CreateAuditEventIn(BaseModel):
    action: str = Field(..., description="Action name, e.g. conversation.exported")
    entity_type: str = Field(..., description="Entity type, e.g. conversation")
    entity_id: str | None = Field(None, description="UUID of the affected entity")
    actor: str = Field("system", description="Username or service name")
    actor_email: str | None = None
    actor_name: str | None = None
    actor_role: str = "admin"
    severity: str = Field("info", description="info | notice | warning | critical")
    compliance_category: str = Field(
        "configuration_change",
        description="access_control | data_modification | export_activity | ai_governance | security_event",
    )
    description: str | None = None
    details: dict[str, Any] | None = None


@router.get("")
async def list_audit_logs(
    action: str | None = Query(None),
    actor: str | None = Query(None),
    entity_type: str | None = Query(None),
    severity: str | None = Query(None),
    compliance_category: str | None = Query(None),
    search: str | None = Query(None),
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=200),
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Retrieves paginated audit logs with search, actor, entity, and severity filters."""
    return await audit_service.search_audit_logs(
        session,
        action=action,
        actor=actor,
        entity_type=entity_type,
        severity=severity,
        compliance_category=compliance_category,
        search=search,
        page=page,
        page_size=page_size,
    )


@router.get("/stats")
async def get_audit_statistics(
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Returns SOC 2 and ISO 27001 compliance summary metrics and event distributions."""
    return await audit_service.get_audit_statistics(session)


@router.get("/export")
async def export_audit_logs(
    format: str = Query("csv", description="csv | json"),
    limit: int = Query(1000, ge=1, le=5000),
    session: AsyncSession = Depends(get_db_session),
):
    """Exports audit trail for compliance verification in CSV or JSON format."""
    if format.lower() == "json":
        data = await audit_service.export_audit_trail_json(session, limit=limit)
        return data

    csv_data = await audit_service.export_audit_trail_csv(session, limit=limit)
    return Response(
        content=csv_data,
        media_type="text/csv",
        headers={"Content-Disposition": 'attachment; filename="audit-trail-soc2.csv"'},
    )


@router.post("")
async def create_audit_event(
    payload: CreateAuditEventIn,
    request: Request,
    session: AsyncSession = Depends(get_db_session),
) -> dict[str, Any]:
    """Records an explicit audit event into the SOC 2 compliance ledger."""
    client_ip = request.client.host if request.client else "127.0.0.1"
    user_agent = request.headers.get("user-agent", "")

    log = await audit_service.record_audit_event(
        session,
        action=payload.action,
        entity_type=payload.entity_type,
        entity_id=payload.entity_id,
        actor=payload.actor,
        actor_email=payload.actor_email,
        actor_name=payload.actor_name,
        actor_role=payload.actor_role,
        severity=payload.severity,
        compliance_category=payload.compliance_category,
        details=payload.details,
        ip_address=client_ip,
        user_agent=user_agent,
        description=payload.description,
    )
    return {
        "success": True,
        "id": str(log.id),
        "created_at": log.created_at.isoformat() if log.created_at else "",
        "action": log.action,
    }
