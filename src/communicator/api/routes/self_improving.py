import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services.self_improving_service import SelfImprovingService

router = APIRouter(prefix="/self-improving", tags=["self-improving"])


class ApplyRetrainingRequest(BaseModel):
    author_name: str = Field(default="Manager")


@router.get("/insights", response_model=list[dict[str, Any]])
async def list_insights(
    category: str | None = Query(None),
    status: str | None = Query(None),
    db: AsyncSession = Depends(get_db_session),
):
    """Lists AI insights, human rep winning patterns, and emerging trends."""
    await SelfImprovingService.seed_default_insights_and_retraining(db)
    return await SelfImprovingService.list_insights(db, category=category, status=status)


@router.post("/insights/{insight_id}/apply", response_model=dict[str, Any])
async def apply_insight_retraining(
    insight_id: uuid.UUID,
    body: ApplyRetrainingRequest = ApplyRetrainingRequest(),
    db: AsyncSession = Depends(get_db_session),
):
    """
    1-Click Continuous Retraining:
    Applies the human rep exemplar or trend suggestion to our Agent's training knowledge,
    bumps model version, and logs performance lift.
    """
    await SelfImprovingService.seed_default_insights_and_retraining(db)
    result = await SelfImprovingService.apply_insight_retraining(
        db, insight_id=insight_id, author_name=body.author_name
    )
    if "error" in result:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail=result["error"])
    return result


@router.post("/insights/{insight_id}/dismiss", response_model=dict[str, Any])
async def dismiss_insight(
    insight_id: uuid.UUID,
    db: AsyncSession = Depends(get_db_session),
):
    """Dismisses an insight suggestion."""
    await SelfImprovingService.seed_default_insights_and_retraining(db)
    success = await SelfImprovingService.dismiss_insight(db, insight_id)
    if not success:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Insight not found")
    return {"status": "dismissed", "insight_id": str(insight_id)}


@router.get("/retraining-history", response_model=list[dict[str, Any]])
async def list_retraining_history(db: AsyncSession = Depends(get_db_session)):
    """Returns chronological timeline of agent retraining iterations and performance lifts."""
    await SelfImprovingService.seed_default_insights_and_retraining(db)
    return await SelfImprovingService.list_retraining_history(db)


@router.post("/scan", response_model=dict[str, Any])
async def trigger_auto_learning_scan(db: AsyncSession = Depends(get_db_session)):
    """Scans high-CSAT live customer interactions to surface new winning human rep patterns."""
    await SelfImprovingService.seed_default_insights_and_retraining(db)
    return await SelfImprovingService.trigger_auto_learning_scan(db)


@router.get("/stats", response_model=dict[str, Any])
async def get_self_improving_stats(db: AsyncSession = Depends(get_db_session)):
    """Summary metrics of continuous improvement, resolution lifts, and learning stats."""
    await SelfImprovingService.seed_default_insights_and_retraining(db)
    return await SelfImprovingService.get_self_improving_stats(db)
