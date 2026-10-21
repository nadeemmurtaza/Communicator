"""Add agent policies, training SOPs, and AI conversation simulations

Revision ID: b8c9d0e1f2a3
Revises: a7b8c9d0e1f2
Create Date: 2026-10-22 01:45:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "b8c9d0e1f2a3"
down_revision: str | Sequence[str] | None = "a7b8c9d0e1f2"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create agent_policies table
    op.create_table(
        "agent_policies",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("category", sa.String(length=50), nullable=False, server_default="general"),
        sa.Column("description", sa.Text(), nullable=False, server_default=""),
        sa.Column("version", sa.String(length=20), nullable=False, server_default="1.0"),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="active"),
        sa.Column("process_steps", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("rules", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="{}"),
        sa.Column("guardrails", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_agent_policies_category"), "agent_policies", ["category"], unique=False)
    op.create_index(op.f("ix_agent_policies_status"), "agent_policies", ["status"], unique=False)

    # 2. Create simulation_scenarios table
    op.create_table(
        "simulation_scenarios",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("description", sa.Text(), nullable=False, server_default=""),
        sa.Column("category", sa.String(length=50), nullable=False, server_default="general"),
        sa.Column("policy_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("customer_persona", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="{}"),
        sa.Column("expected_outcome", sa.String(length=50), nullable=False, server_default="resolve_autonomous"),
        sa.Column("customer_messages", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.ForeignKeyConstraint(["policy_id"], ["agent_policies.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_simulation_scenarios_category"), "simulation_scenarios", ["category"], unique=False)
    op.create_index(op.f("ix_simulation_scenarios_policy_id"), "simulation_scenarios", ["policy_id"], unique=False)

    # 3. Create simulation_runs table
    op.create_table(
        "simulation_runs",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("scenario_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("policy_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="completed"),
        sa.Column("verdict", sa.String(length=30), nullable=False, server_default="passed"),
        sa.Column("policy_adherence_score", sa.Float(), nullable=False, server_default="95.0"),
        sa.Column("empathy_score", sa.Float(), nullable=False, server_default="90.0"),
        sa.Column("accuracy_score", sa.Float(), nullable=False, server_default="92.0"),
        sa.Column("overall_score", sa.Float(), nullable=False, server_default="92.5"),
        sa.Column("turn_count", sa.Integer(), nullable=False, server_default="2"),
        sa.Column("transcript", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("evaluation_summary", sa.Text(), nullable=False, server_default=""),
        sa.Column("recommendations", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.ForeignKeyConstraint(["policy_id"], ["agent_policies.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["scenario_id"], ["simulation_scenarios.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_simulation_runs_scenario_id"), "simulation_runs", ["scenario_id"], unique=False)
    op.create_index(op.f("ix_simulation_runs_verdict"), "simulation_runs", ["verdict"], unique=False)


def downgrade() -> None:
    op.drop_index(op.f("ix_simulation_runs_verdict"), table_name="simulation_runs")
    op.drop_index(op.f("ix_simulation_runs_scenario_id"), table_name="simulation_runs")
    op.drop_table("simulation_runs")

    op.drop_index(op.f("ix_simulation_scenarios_policy_id"), table_name="simulation_scenarios")
    op.drop_index(op.f("ix_simulation_scenarios_category"), table_name="simulation_scenarios")
    op.drop_table("simulation_scenarios")

    op.drop_index(op.f("ix_agent_policies_status"), table_name="agent_policies")
    op.drop_index(op.f("ix_agent_policies_category"), table_name="agent_policies")
    op.drop_table("agent_policies")
