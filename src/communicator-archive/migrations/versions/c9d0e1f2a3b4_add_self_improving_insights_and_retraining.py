"""Add self-improving AI insights, human exemplar learning, and continuous retraining events

Revision ID: c9d0e1f2a3b4
Revises: b8c9d0e1f2a3
Create Date: 2026-10-22 02:00:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "c9d0e1f2a3b4"
down_revision: str | Sequence[str] | None = "b8c9d0e1f2a3"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create agent_insights table
    op.create_table(
        "agent_insights",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("title", sa.String(length=255), nullable=False),
        sa.Column("category", sa.String(length=50), nullable=False, server_default="trend"),
        sa.Column("description", sa.Text(), nullable=False, server_default=""),
        sa.Column("human_rep_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("human_rep_name", sa.String(length=100), nullable=True),
        sa.Column("exemplar_content", sa.Text(), nullable=True),
        sa.Column("impact_score", sa.Float(), nullable=False, server_default="8.0"),
        sa.Column("estimated_resolution_lift", sa.Float(), nullable=False, server_default="3.5"),
        sa.Column(
            "suggested_action", sa.String(length=50), nullable=False, server_default="retrain_agent"
        ),
        sa.Column(
            "action_payload",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="{}",
        ),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="open"),
        sa.Column("applied_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column(
            "created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()
        ),
        sa.Column(
            "updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()
        ),
        sa.ForeignKeyConstraint(["human_rep_id"], ["team_members.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_agent_insights_category"), "agent_insights", ["category"], unique=False
    )
    op.create_index(op.f("ix_agent_insights_status"), "agent_insights", ["status"], unique=False)

    # 2. Create agent_retraining_events table
    op.create_table(
        "agent_retraining_events",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("title", sa.String(length=255), nullable=False),
        sa.Column(
            "trigger_type",
            sa.String(length=50),
            nullable=False,
            server_default="human_exemplar_sync",
        ),
        sa.Column("version_from", sa.String(length=20), nullable=False, server_default="2.1"),
        sa.Column("version_to", sa.String(length=20), nullable=False, server_default="2.2"),
        sa.Column("insight_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column(
            "changes_applied",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="[]",
        ),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="applied"),
        sa.Column("baseline_resolution_rate", sa.Float(), nullable=False, server_default="86.4"),
        sa.Column(
            "post_training_resolution_rate", sa.Float(), nullable=False, server_default="91.2"
        ),
        sa.Column("resolution_lift", sa.Float(), nullable=False, server_default="4.8"),
        sa.Column(
            "created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()
        ),
        sa.ForeignKeyConstraint(["insight_id"], ["agent_insights.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )


def downgrade() -> None:
    op.drop_table("agent_retraining_events")
    op.drop_index(op.f("ix_agent_insights_status"), table_name="agent_insights")
    op.drop_index(op.f("ix_agent_insights_category"), table_name="agent_insights")
    op.drop_table("agent_insights")
