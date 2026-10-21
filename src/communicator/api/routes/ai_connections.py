import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.ai_connection import AIJobType, AIProvider
from communicator.services.ai_connection_service import AIConnectionService

router = APIRouter(prefix="/ai", tags=["ai-connections"])


# Pydantic Schemas
class AIConnectionCreateRequest(BaseModel):
    name: str = Field(..., min_length=2, max_length=255)
    provider: str = Field(default=AIProvider.ANTHROPIC.value)
    model_id: str = Field(..., min_length=2, max_length=100)
    api_key: str = Field(default="")
    base_url: str | None = None
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    max_tokens: int = Field(default=1024, ge=100, le=32000)
    is_active: bool = True
    assigned_jobs: list[str] = Field(default_factory=list)


class JobAssignmentRequest(BaseModel):
    job_type: str = Field(...)
    connection_id: uuid.UUID = Field(...)


class CustomPolicyUpdateRequest(BaseModel):
    tone: str | None = None
    guidelines: str | None = None
    rules: list[dict[str, Any]] | None = None
    guardrails: list[str] | None = None
    auto_approval_mode: str | None = None
    min_csat_learning_threshold: float | None = None
    max_autonomous_refund: float | None = None


@router.get("/connections", response_model=list[dict[str, Any]])
async def list_ai_connections(db: AsyncSession = Depends(get_db_session)):
    """Lists all connected AI models with latency and assigned jobs."""
    await AIConnectionService.seed_default_connections_and_policies(db)
    return await AIConnectionService.list_connections(db)


@router.post("/connections", status_code=status.HTTP_201_CREATED, response_model=dict[str, Any])
async def create_or_update_connection(
    body: AIConnectionCreateRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Connects any AI model (Claude, GPT-4o, Gemini, DeepSeek, Groq, Ollama, custom endpoint)."""
    conn = await AIConnectionService.create_or_update_connection(
        db=db,
        name=body.name,
        provider=body.provider,
        model_id=body.model_id,
        api_key=body.api_key,
        base_url=body.base_url,
        temperature=body.temperature,
        max_tokens=body.max_tokens,
        is_active=body.is_active,
        assigned_jobs=body.assigned_jobs,
    )
    return {
        "id": str(conn.id),
        "name": conn.name,
        "provider": conn.provider,
        "model_id": conn.model_id,
        "test_status": conn.test_status,
        "is_active": conn.is_active,
    }


@router.post("/connections/{connection_id}/test", response_model=dict[str, Any])
async def test_ai_connection(
    connection_id: uuid.UUID,
    db: AsyncSession = Depends(get_db_session),
):
    """Tests live connectivity and latency for a configured AI model."""
    result = await AIConnectionService.test_connection(db, connection_id)
    if "error" in result:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail=result["error"])
    return result


@router.delete("/connections/{connection_id}", response_model=dict[str, Any])
async def delete_ai_connection(
    connection_id: uuid.UUID,
    db: AsyncSession = Depends(get_db_session),
):
    """Deletes an AI model connection."""
    success = await AIConnectionService.delete_connection(db, connection_id)
    if not success:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Connection not found")
    return {"status": "deleted", "connection_id": str(connection_id)}


@router.get("/jobs/mapping", response_model=dict[str, Any])
async def get_job_mapping(db: AsyncSession = Depends(get_db_session)):
    """Returns which model is actively routed to each AI job."""
    await AIConnectionService.seed_default_connections_and_policies(db)
    return await AIConnectionService.get_job_model_mapping(db)


@router.post("/jobs/mapping", response_model=dict[str, Any])
async def assign_model_to_job(
    body: JobAssignmentRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Assigns an AI model connection to a specific AI job."""
    result = await AIConnectionService.assign_model_to_job(
        db=db, job_type=body.job_type, connection_id=body.connection_id
    )
    if "error" in result:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail=result["error"])
    return result


@router.get("/policies/{job_type}", response_model=dict[str, Any])
async def get_custom_policy(
    job_type: str,
    db: AsyncSession = Depends(get_db_session),
):
    """Retrieves custom policies, dynamic rules, and guardrails for a specific AI job."""
    await AIConnectionService.seed_default_connections_and_policies(db)
    policy = await AIConnectionService.get_custom_policy(db, job_type)
    if not policy:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail=f"Policy for {job_type} not found")
    return policy


@router.put("/policies/{job_type}", response_model=dict[str, Any])
async def update_custom_policy(
    job_type: str,
    body: CustomPolicyUpdateRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Updates custom policies, tone, guidelines, rules, and guardrails for an AI job."""
    return await AIConnectionService.update_custom_policy(
        db=db,
        job_type=job_type,
        tone=body.tone,
        guidelines=body.guidelines,
        rules=body.rules,
        guardrails=body.guardrails,
        auto_approval_mode=body.auto_approval_mode,
        min_csat_learning_threshold=body.min_csat_learning_threshold,
        max_autonomous_refund=body.max_autonomous_refund,
    )


@router.get("/prompts/{job_type}", response_model=dict[str, str])
async def get_effective_prompt(
    job_type: str,
    db: AsyncSession = Depends(get_db_session),
):
    """Builds and returns the dynamically compiled system prompt for that job."""
    await AIConnectionService.seed_default_connections_and_policies(db)
    prompt = await AIConnectionService.get_effective_prompt(db, job_type)
    return {"job_type": job_type, "prompt": prompt}
