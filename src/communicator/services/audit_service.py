from __future__ import annotations

import csv
import io
import json
import logging
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import desc, func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.audit import AuditLog

logger = logging.getLogger(__name__)

NIL_UUID = uuid.UUID("00000000-0000-0000-0000-000000000000")


def _normalize_uuid(entity_id: uuid.UUID | str | None) -> uuid.UUID:
    if isinstance(entity_id, uuid.UUID):
        return entity_id
    if isinstance(entity_id, str) and entity_id.strip():
        try:
            return uuid.UUID(entity_id.strip())
        except ValueError:
            return uuid.uuid5(uuid.NAMESPACE_DNS, entity_id.strip())
    return NIL_UUID


async def record_audit_event(
    session: AsyncSession,
    action: str,
    entity_type: str,
    entity_id: uuid.UUID | str | None = None,
    actor: str = "system",
    actor_email: str | None = None,
    actor_name: str | None = None,
    actor_role: str = "admin",
    severity: str = "info",
    compliance_category: str = "configuration_change",
    details: dict[str, Any] | None = None,
    ip_address: str | None = None,
    user_agent: str | None = None,
    description: str | None = None,
) -> AuditLog:
    """
    Creates and records a SOC 2 compliant immutable audit trail event.
    """
    safe_entity_id = _normalize_uuid(entity_id)

    full_details = dict(details or {})
    if actor_email:
        full_details["actor_email"] = actor_email
    if actor_name:
        full_details["actor_name"] = actor_name
    if actor_role:
        full_details["actor_role"] = actor_role
    if ip_address:
        full_details["ip_address"] = ip_address
    if user_agent:
        full_details["user_agent"] = user_agent
    if severity:
        full_details["severity"] = severity
    if compliance_category:
        full_details["compliance_category"] = compliance_category
    if description:
        full_details["description"] = description
    else:
        full_details["description"] = f"{actor} performed {action} on {entity_type}"

    log = AuditLog(
        entity_type=entity_type,
        entity_id=safe_entity_id,
        action=action,
        actor=actor,
        details=full_details,
        created_at=datetime.now(UTC),
    )
    session.add(log)
    await session.flush()
    return log


async def search_audit_logs(
    session: AsyncSession,
    action: str | None = None,
    actor: str | None = None,
    entity_type: str | None = None,
    severity: str | None = None,
    compliance_category: str | None = None,
    search: str | None = None,
    start_time: datetime | None = None,
    end_time: datetime | None = None,
    page: int = 1,
    page_size: int = 50,
) -> dict[str, Any]:
    """
    Queries audit logs with enterprise multi-dimensional filters, search, and pagination.
    """
    stmt = select(AuditLog)

    if action:
        stmt = stmt.where(AuditLog.action.ilike(f"%{action.strip()}%"))
    if actor:
        stmt = stmt.where(AuditLog.actor.ilike(f"%{actor.strip()}%"))
    if entity_type:
        stmt = stmt.where(AuditLog.entity_type == entity_type.strip())
    if start_time:
        stmt = stmt.where(AuditLog.created_at >= start_time)
    if end_time:
        stmt = stmt.where(AuditLog.created_at <= end_time)

    # Count query
    count_stmt = select(func.count()).select_from(stmt.subquery())
    total_count = (await session.execute(count_stmt)).scalar() or 0

    # Sort newest first & paginate
    stmt = stmt.order_by(desc(AuditLog.created_at))
    offset = (max(page, 1) - 1) * page_size
    stmt = stmt.offset(offset).limit(page_size)

    rows = (await session.execute(stmt)).scalars().all()

    items = []
    for r in rows:
        d = dict(r.details or {})
        row_severity = d.get("severity", "info")
        row_category = d.get("compliance_category", "general")

        # Client-side filtering on JSONB details if filtered
        if severity and row_severity.lower() != severity.lower():
            continue
        if compliance_category and row_category.lower() != compliance_category.lower():
            continue
        if search:
            q = search.lower()
            haystack = f"{r.action} {r.actor} {r.entity_type} {d.get('description', '')} {d.get('ip_address', '')} {d.get('actor_email', '')}".lower()
            if q not in haystack:
                continue

        items.append({
            "id": str(r.id),
            "created_at": r.created_at.isoformat() if r.created_at else "",
            "action": r.action,
            "actor": r.actor,
            "actor_email": d.get("actor_email", ""),
            "actor_name": d.get("actor_name", r.actor),
            "actor_role": d.get("actor_role", "admin"),
            "entity_type": r.entity_type,
            "entity_id": str(r.entity_id),
            "severity": row_severity,
            "compliance_category": row_category,
            "description": d.get("description", f"{r.actor} executed {r.action}"),
            "ip_address": d.get("ip_address", "127.0.0.1"),
            "user_agent": d.get("user_agent", ""),
            "details": d,
        })

    total_pages = max(1, (total_count + page_size - 1) // page_size)

    return {
        "items": items,
        "total": total_count,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages,
    }


async def get_audit_statistics(session: AsyncSession) -> dict[str, Any]:
    """
    Computes SOC 2 and ISO 27001 summary metrics across recorded audit logs.
    """
    # Fetch recent rows for analytics
    stmt = select(AuditLog).order_by(desc(AuditLog.created_at)).limit(500)
    rows = (await session.execute(stmt)).scalars().all()

    total_count_stmt = select(func.count(AuditLog.id))
    total_events = (await session.execute(total_count_stmt)).scalar() or 0

    severity_counts = {"info": 0, "notice": 0, "warning": 0, "critical": 0}
    category_counts: dict[str, int] = {}
    actor_counts: dict[str, int] = {}
    action_counts: dict[str, int] = {}

    for r in rows:
        d = dict(r.details or {})
        sev = str(d.get("severity", "info")).lower()
        severity_counts[sev] = severity_counts.get(sev, 0) + 1

        cat = str(d.get("compliance_category", "configuration_change")).lower()
        category_counts[cat] = category_counts.get(cat, 0) + 1

        actor_counts[r.actor] = actor_counts.get(r.actor, 0) + 1
        action_counts[r.action] = action_counts.get(r.action, 0) + 1

    top_actors = sorted(
        [{"actor": k, "count": v} for k, v in actor_counts.items()],
        key=lambda x: x["count"],
        reverse=True,
    )[:5]

    top_actions = sorted(
        [{"action": k, "count": v} for k, v in action_counts.items()],
        key=lambda x: x["count"],
        reverse=True,
    )[:5]

    return {
        "total_events": total_events,
        "sample_analyzed": len(rows),
        "severity_distribution": severity_counts,
        "category_distribution": category_counts,
        "top_actors": top_actors,
        "top_actions": top_actions,
        "soc2_compliance_status": "MONITORED_AND_AUDITED",
        "last_audit_event_at": rows[0].created_at.isoformat() if rows and rows[0].created_at else None,
    }


async def export_audit_trail_csv(session: AsyncSession, limit: int = 2000) -> str:
    """
    Generates a RFC 4180 CSV export of the compliance audit trail.
    """
    stmt = select(AuditLog).order_by(desc(AuditLog.created_at)).limit(limit)
    rows = (await session.execute(stmt)).scalars().all()

    output = io.StringIO()
    writer = csv.writer(output)

    writer.writerow([
        "Event ID",
        "Timestamp (UTC)",
        "Action",
        "Severity",
        "Compliance Category",
        "Actor",
        "Actor Email",
        "Actor Role",
        "Entity Type",
        "Entity ID",
        "IP Address",
        "Description",
        "Details JSON",
    ])

    for r in rows:
        d = dict(r.details or {})
        writer.writerow([
            str(r.id),
            r.created_at.isoformat() if r.created_at else "",
            r.action,
            d.get("severity", "info"),
            d.get("compliance_category", "general"),
            r.actor,
            d.get("actor_email", ""),
            d.get("actor_role", "admin"),
            r.entity_type,
            str(r.entity_id),
            d.get("ip_address", ""),
            d.get("description", ""),
            json.dumps(d),
        ])

    return output.getvalue()


async def export_audit_trail_json(session: AsyncSession, limit: int = 2000) -> dict[str, Any]:
    """
    Generates a structured SOC 2 JSON compliance audit envelope.
    """
    stmt = select(AuditLog).order_by(desc(AuditLog.created_at)).limit(limit)
    rows = (await session.execute(stmt)).scalars().all()

    items = []
    for r in rows:
        d = dict(r.details or {})
        items.append({
            "id": str(r.id),
            "timestamp_utc": r.created_at.isoformat() if r.created_at else "",
            "action": r.action,
            "severity": d.get("severity", "info"),
            "compliance_category": d.get("compliance_category", "general"),
            "actor": {
                "identifier": r.actor,
                "email": d.get("actor_email", ""),
                "name": d.get("actor_name", r.actor),
                "role": d.get("actor_role", "admin"),
            },
            "target": {
                "entity_type": r.entity_type,
                "entity_id": str(r.entity_id),
            },
            "network": {
                "ip_address": d.get("ip_address", "127.0.0.1"),
                "user_agent": d.get("user_agent", ""),
            },
            "description": d.get("description", ""),
            "metadata": d,
        })

    return {
        "export_metadata": {
            "exported_at": datetime.now(UTC).isoformat(),
            "standard": "SOC 2 Type II / ISO 27001 Activity Trail",
            "total_records": len(items),
            "generated_by": "Captain AI Enterprise Compliance Engine",
        },
        "records": items,
    }
