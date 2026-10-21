import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.simulation import PolicyStatus
from communicator.services.simulation_service import SimulationService

router = APIRouter(prefix="/simulations", tags=["simulations"])


# Pydantic Schemas
class PolicyCreateRequest(BaseModel):
    name: str = Field(..., min_length=3, max_length=255)
    category: str = Field(default="general")
    description: str = Field(default="")
    version: str = Field(default="1.0")
    status: str = Field(default=PolicyStatus.ACTIVE.value)
    process_steps: list[dict[str, Any]] = Field(default_factory=list)
    rules: dict[str, Any] = Field(default_factory=dict)
    guardrails: list[str] = Field(default_factory=list)


class ScenarioCreateRequest(BaseModel):
    name: str = Field(..., min_length=3, max_length=255)
    description: str = Field(default="")
    category: str = Field(default="general")
    policy_id: uuid.UUID | None = None
    customer_persona: dict[str, Any] = Field(default_factory=dict)
    expected_outcome: str = Field(default="resolve_autonomous")
    customer_messages: list[str] = Field(default_factory=list)


class RunSimulationRequest(BaseModel):
    scenario_id: uuid.UUID
    policy_id: uuid.UUID | None = None


class SandboxChatRequest(BaseModel):
    message: str = Field(..., min_length=1)
    history: list[dict[str, str]] = Field(default_factory=list)
    policy_id: uuid.UUID | None = None


@router.get("/policies", response_model=list[dict[str, Any]])
async def list_policies(
    category: str | None = Query(None),
    db: AsyncSession = Depends(get_db_session),
):
    """Lists all agent training policies and SOP guidelines."""
    await SimulationService.seed_default_policies_and_scenarios(db)
    policies = await SimulationService.list_policies(db, category=category)
    return [
        {
            "id": str(p.id),
            "name": p.name,
            "category": p.category,
            "description": p.description,
            "version": p.version,
            "status": p.status,
            "process_steps": p.process_steps,
            "rules": p.rules,
            "guardrails": p.guardrails,
            "created_at": p.created_at.isoformat(),
            "updated_at": p.updated_at.isoformat(),
        }
        for p in policies
    ]


@router.post("/policies", status_code=status.HTTP_201_CREATED, response_model=dict[str, Any])
async def create_policy(
    body: PolicyCreateRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Creates a new agent policy or process SOP."""
    p = await SimulationService.create_policy(
        db=db,
        name=body.name,
        category=body.category,
        description=body.description,
        version=body.version,
        status=body.status,
        process_steps=body.process_steps,
        rules=body.rules,
        guardrails=body.guardrails,
    )
    return {
        "id": str(p.id),
        "name": p.name,
        "category": p.category,
        "status": p.status,
        "version": p.version,
    }


@router.get("/scenarios", response_model=list[dict[str, Any]])
async def list_scenarios(
    category: str | None = Query(None),
    db: AsyncSession = Depends(get_db_session),
):
    """Lists pre-configured customer conversation scenarios for agent testing."""
    await SimulationService.seed_default_policies_and_scenarios(db)
    return await SimulationService.list_scenarios(db, category=category)


@router.post("/run", response_model=dict[str, Any])
async def run_simulation(
    body: RunSimulationRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Executes a simulated multi-turn conversation between an AI Customer Persona and our Agent.
    Evaluates step-by-step adherence to policies and guardrails.
    """
    await SimulationService.seed_default_policies_and_scenarios(db)
    try:
        run = await SimulationService.run_simulation(
            db,
            scenario_id=body.scenario_id,
            policy_id=body.policy_id,
        )
        return {
            "id": str(run.id),
            "scenario_id": str(run.scenario_id),
            "status": run.status,
            "verdict": run.verdict,
            "policy_adherence_score": run.policy_adherence_score,
            "empathy_score": run.empathy_score,
            "accuracy_score": run.accuracy_score,
            "overall_score": run.overall_score,
            "turn_count": run.turn_count,
            "transcript": run.transcript,
            "evaluation_summary": run.evaluation_summary,
            "recommendations": run.recommendations,
            "created_at": run.created_at.isoformat(),
        }
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail=str(e))


@router.post("/batch", response_model=dict[str, Any])
async def run_batch_simulations(db: AsyncSession = Depends(get_db_session)):
    """Runs all scenarios in batch mode and generates overall suite pass rate and scores."""
    await SimulationService.seed_default_policies_and_scenarios(db)
    return await SimulationService.run_batch_simulations(db)


@router.post("/sandbox/chat", response_model=dict[str, Any])
async def sandbox_chat(
    body: SandboxChatRequest,
    db: AsyncSession = Depends(get_db_session),
):
    """Interactive sandbox test: test any customer message and view real-time policy evaluation."""
    await SimulationService.seed_default_policies_and_scenarios(db)
    return await SimulationService.interactive_sandbox_turn(
        db=db,
        message=body.message,
        history=body.history,
        policy_id=body.policy_id,
    )


@router.get("/stats", response_model=dict[str, Any])
async def get_simulation_stats(db: AsyncSession = Depends(get_db_session)):
    """Summary metrics of test suite pass rate, active scenarios, and average scores."""
    await SimulationService.seed_default_policies_and_scenarios(db)
    return await SimulationService.get_stats_summary(db)
