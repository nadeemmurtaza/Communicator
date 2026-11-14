from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.knowledge_base import KnowledgeBaseEntry, KnowledgeRecommendation

TOPIC_ICONS: dict[str, str] = {
    "Billing & Refunds": "CreditCard",
    "Shipping & Delivery": "Truck",
    "Orders & Returns": "Package",
    "Product & Catalog": "Sparkles",
    "Account & VIP Access": "Crown",
    "General": "BookOpen",
}


def serialize_hub_entry(e: KnowledgeBaseEntry) -> dict[str, Any]:
    return {
        "id": str(e.id),
        "title": e.title,
        "content": e.content,
        "topic": e.topic or "General",
        "sourceType": e.source_type,
        "source_type": e.source_type,
        "sourceUrl": e.source_url,
        "source_url": e.source_url,
        "fileName": e.file_name,
        "file_name": e.file_name,
        "targetTouchpoints": e.target_touchpoints or ["agent", "copilot", "help_center"],
        "target_touchpoints": e.target_touchpoints or ["agent", "copilot", "help_center"],
        "status": e.status,
        "resolutionRate": round(float(e.resolution_rate or 85.0), 1),
        "resolution_rate": round(float(e.resolution_rate or 85.0), 1),
        "usageCount": int(e.usage_count or 0),
        "usage_count": int(e.usage_count or 0),
        "helpfulCount": int(e.helpful_count or 0),
        "helpful_count": int(e.helpful_count or 0),
        "unhelpfulCount": int(e.unhelpful_count or 0),
        "unhelpful_count": int(e.unhelpful_count or 0),
        "createdAt": e.created_at.isoformat(),
        "created_at": e.created_at.isoformat(),
        "updatedAt": e.updated_at.isoformat() if e.updated_at else e.created_at.isoformat(),
        "updated_at": e.updated_at.isoformat() if e.updated_at else e.created_at.isoformat(),
    }


def serialize_recommendation(r: KnowledgeRecommendation) -> dict[str, Any]:
    return {
        "id": str(r.id),
        "gapType": r.gap_type,
        "gap_type": r.gap_type,
        "topic": r.topic,
        "title": r.title,
        "description": r.description,
        "impactOnResolution": round(float(r.impact_on_resolution or 3.5), 1),
        "impact_on_resolution": round(float(r.impact_on_resolution or 3.5), 1),
        "suggestedAction": r.suggested_action,
        "suggested_action": r.suggested_action,
        "suggestedContent": r.suggested_content,
        "suggested_content": r.suggested_content,
        "status": r.status,
        "createdAt": r.created_at.isoformat(),
        "created_at": r.created_at.isoformat(),
        "resolvedAt": r.resolved_at.isoformat() if r.resolved_at else None,
        "resolved_at": r.resolved_at.isoformat() if r.resolved_at else None,
    }


async def get_knowledge_hub_overview(
    session: AsyncSession,
    search_query: str | None = None,
    topic_filter: str | None = None,
    touchpoint_filter: str | None = None,
) -> dict[str, Any]:

    stmt = select(KnowledgeBaseEntry).order_by(desc(KnowledgeBaseEntry.created_at))
    entries = list((await session.execute(stmt)).scalars().all())

    # Filter
    if search_query and search_query.strip():
        q = search_query.strip().lower()
        entries = [
            e
            for e in entries
            if q in e.title.lower() or q in e.content.lower() or q in (e.topic or "").lower()
        ]

    if topic_filter and topic_filter.strip() and topic_filter.lower() != "all":
        t = topic_filter.strip().lower()
        entries = [e for e in entries if (e.topic or "").lower() == t]

    if touchpoint_filter and touchpoint_filter.strip() and touchpoint_filter.lower() != "all":
        tp = touchpoint_filter.strip().lower()
        entries = [e for e in entries if tp in [x.lower() for x in (e.target_touchpoints or [])]]

    # Group by topic
    topics_map: dict[str, list[dict[str, Any]]] = {}
    for e in entries:
        top = e.topic or "General"
        if top not in topics_map:
            topics_map[top] = []
        topics_map[top].append(serialize_hub_entry(e))

    grouped_topics = []
    for top_name, items in topics_map.items():
        avg_res = sum(it["resolutionRate"] for it in items) / len(items) if items else 0.0
        grouped_topics.append(
            {
                "name": top_name,
                "icon": TOPIC_ICONS.get(top_name, "Folder"),
                "sourcesCount": len(items),
                "avgResolutionRate": round(avg_res, 1),
                "sources": items,
            }
        )

    # Sort topics alphabetically
    grouped_topics.sort(key=lambda x: x["name"])

    # Aggregate performance metrics
    all_entries_stmt = select(KnowledgeBaseEntry)
    all_entries = list((await session.execute(all_entries_stmt)).scalars().all())
    total_sources = len(all_entries)
    overall_res_rate = (
        sum(float(e.resolution_rate or 85.0) for e in all_entries) / total_sources
        if total_sources > 0
        else 85.0
    )

    touchpoint_breakdown = {
        "agent": {
            "title": "Fin Autonomous Agent",
            "sourcesDeployed": len(
                [e for e in all_entries if "agent" in (e.target_touchpoints or [])]
            ),
            "resolutionRate": round(overall_res_rate - 1.2, 1),
            "deflectionsCount": sum(int(e.usage_count or 0) for e in all_entries),
            "status": "Active & Grounded",
        },
        "copilot": {
            "title": "Inbox Copilot Assistant",
            "sourcesDeployed": len(
                [e for e in all_entries if "copilot" in (e.target_touchpoints or [])]
            ),
            "resolutionRate": round(overall_res_rate + 4.5, 1),
            "insertionsCount": int(sum(int(e.usage_count or 0) for e in all_entries) * 0.72),
            "status": "Assisting Teammates",
        },
        "help_center": {
            "title": "Customer Help Center",
            "sourcesDeployed": len(
                [e for e in all_entries if "help_center" in (e.target_touchpoints or [])]
            ),
            "resolutionRate": round(overall_res_rate + 2.0, 1),
            "satisfactionPct": 93.4,
            "status": "Public Self-Serve",
        },
    }

    return {
        "totalSources": total_sources,
        "overallResolutionRate": round(overall_res_rate, 1),
        "topics": grouped_topics,
        "touchpointBreakdown": touchpoint_breakdown,
    }


async def list_recommendations(
    session: AsyncSession,
    status: str | None = "open",
    gap_type: str | None = None,
) -> list[dict[str, Any]]:
    stmt = select(KnowledgeRecommendation).order_by(
        desc(KnowledgeRecommendation.impact_on_resolution)
    )

    if status and status.lower() != "all":
        stmt = stmt.where(KnowledgeRecommendation.status == status.lower())
    if gap_type and gap_type.lower() != "all":
        stmt = stmt.where(KnowledgeRecommendation.gap_type == gap_type.lower())

    results = list((await session.execute(stmt)).scalars().all())
    return [serialize_recommendation(r) for r in results]


async def apply_recommendation(
    session: AsyncSession,
    recommendation_id: uuid.UUID,
    custom_content: str | None = None,
) -> dict[str, Any]:
    rec = await session.get(KnowledgeRecommendation, recommendation_id)
    if not rec:
        raise ValueError(f"Recommendation {recommendation_id} not found")

    content_to_use = (
        custom_content.strip()
        if custom_content and custom_content.strip()
        else rec.suggested_content
    )

    # Create new published Knowledge Base article
    new_entry = KnowledgeBaseEntry(
        title=rec.title,
        content=content_to_use,
        topic=rec.topic or "General",
        source_type="article",
        target_touchpoints=["agent", "copilot", "help_center"],
        status="published",
        resolution_rate=round(85.0 + float(rec.impact_on_resolution), 1),
        usage_count=1,
        helpful_count=1,
    )
    session.add(new_entry)

    # Mark recommendation as applied
    rec.status = "applied"
    rec.resolved_at = datetime.now(UTC)
    await session.flush()

    return {
        "status": "applied",
        "recommendation": serialize_recommendation(rec),
        "createdEntry": serialize_hub_entry(new_entry),
    }


async def dismiss_recommendation(
    session: AsyncSession,
    recommendation_id: uuid.UUID,
) -> dict[str, Any]:
    rec = await session.get(KnowledgeRecommendation, recommendation_id)
    if not rec:
        raise ValueError(f"Recommendation {recommendation_id} not found")

    rec.status = "dismissed"
    rec.resolved_at = datetime.now(UTC)
    await session.flush()
    return {"status": "dismissed", "recommendation": serialize_recommendation(rec)}
