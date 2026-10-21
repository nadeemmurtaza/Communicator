"""Add AI model connections and custom policies for AI jobs

Revision ID: d0e1f2a3b4c5
Revises: c9d0e1f2a3b4
Create Date: 2026-10-22 02:05:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "d0e1f2a3b4c5"
down_revision: str | Sequence[str] | None = "c9d0e1f2a3b4"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create ai_connections table
    op.create_table(
        "ai_connections",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("provider", sa.String(length=50), nullable=False, server_default="anthropic"),
        sa.Column("model_id", sa.String(length=100), nullable=False),
        sa.Column("api_key", sa.Text(), nullable=False, server_default=""),
        sa.Column("base_url", sa.String(length=255), nullable=True),
        sa.Column("temperature", sa.Float(), nullable=False, server_default="0.7"),
        sa.Column("max_tokens", sa.Integer(), nullable=False, server_default="1024"),
        sa.Column("is_active", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("assigned_jobs", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("last_tested_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("test_status", sa.String(length=30), nullable=False, server_default="untested"),
        sa.Column("latency_ms", sa.Integer(), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_ai_connections_provider"), "ai_connections", ["provider"], unique=False)

    # 2. Create ai_custom_policies table
    op.create_table(
        "ai_custom_policies",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("job_type", sa.String(length=50), nullable=False),
        sa.Column("tone", sa.String(length=50), nullable=False, server_default="empathetic"),
        sa.Column("guidelines", sa.Text(), nullable=False, server_default=""),
        sa.Column("rules", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("guardrails", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="[]"),
        sa.Column("auto_approval_mode", sa.String(length=50), nullable=False, server_default="require_manager_review"),
        sa.Column("min_csat_learning_threshold", sa.Float(), nullable=False, server_default="4.0"),
        sa.Column("max_autonomous_refund", sa.Float(), nullable=False, server_default="50.0"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_ai_custom_policies_job_type"), "ai_custom_policies", ["job_type"], unique=True)


def downgrade() -> None:
    op.drop_index(op.f("ix_ai_custom_policies_job_type"), table_name="ai_custom_policies")
    op.drop_table("ai_custom_policies")
    op.drop_index(op.f("ix_ai_connections_provider"), table_name="ai_connections")
    op.drop_table("ai_connections")
