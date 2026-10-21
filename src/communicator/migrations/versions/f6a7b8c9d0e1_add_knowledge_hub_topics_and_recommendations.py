"""Add knowledge hub topics, touchpoints, and recommendations

Revision ID: f6a7b8c9d0e1
Revises: e5f6a7b8c9d0
Create Date: 2026-10-22 00:50:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "f6a7b8c9d0e1"
down_revision: str | Sequence[str] | None = "e5f6a7b8c9d0"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Add topic, touchpoints, and performance metrics to knowledge_base_entries
    op.add_column("knowledge_base_entries", sa.Column("topic", sa.String(length=100), nullable=False, server_default="General"))
    op.add_column("knowledge_base_entries", sa.Column("target_touchpoints", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default='["agent", "copilot", "help_center"]'))
    op.add_column("knowledge_base_entries", sa.Column("status", sa.String(length=30), nullable=False, server_default="published"))
    op.add_column("knowledge_base_entries", sa.Column("resolution_rate", sa.Float(), nullable=False, server_default="85.0"))
    op.add_column("knowledge_base_entries", sa.Column("usage_count", sa.Integer(), nullable=False, server_default="0"))
    op.add_column("knowledge_base_entries", sa.Column("helpful_count", sa.Integer(), nullable=False, server_default="0"))
    op.add_column("knowledge_base_entries", sa.Column("unhelpful_count", sa.Integer(), nullable=False, server_default="0"))
    op.add_column("knowledge_base_entries", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()))

    # 2. Create knowledge_recommendations table for Content, Customer Data, and Action Gaps
    op.create_table(
        "knowledge_recommendations",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("gap_type", sa.String(length=50), nullable=False, server_default="content_gap"),
        sa.Column("topic", sa.String(length=100), nullable=False, server_default="General"),
        sa.Column("title", sa.String(length=255), nullable=False),
        sa.Column("description", sa.Text(), nullable=False, server_default=""),
        sa.Column("impact_on_resolution", sa.Float(), nullable=False, server_default="3.5"),
        sa.Column("suggested_action", sa.Text(), nullable=False, server_default=""),
        sa.Column("suggested_content", sa.Text(), nullable=False, server_default=""),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="open"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("resolved_at", sa.DateTime(timezone=True), nullable=True),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_knowledge_recommendations_topic"), "knowledge_recommendations", ["topic"], unique=False)
    op.create_index(op.f("ix_knowledge_recommendations_gap_type"), "knowledge_recommendations", ["gap_type"], unique=False)
    op.create_index(op.f("ix_knowledge_recommendations_status"), "knowledge_recommendations", ["status"], unique=False)


def downgrade() -> None:
    op.drop_index(op.f("ix_knowledge_recommendations_status"), table_name="knowledge_recommendations")
    op.drop_index(op.f("ix_knowledge_recommendations_gap_type"), table_name="knowledge_recommendations")
    op.drop_index(op.f("ix_knowledge_recommendations_topic"), table_name="knowledge_recommendations")
    op.drop_table("knowledge_recommendations")

    op.drop_column("knowledge_base_entries", "updated_at")
    op.drop_column("knowledge_base_entries", "unhelpful_count")
    op.drop_column("knowledge_base_entries", "helpful_count")
    op.drop_column("knowledge_base_entries", "usage_count")
    op.drop_column("knowledge_base_entries", "resolution_rate")
    op.drop_column("knowledge_base_entries", "status")
    op.drop_column("knowledge_base_entries", "target_touchpoints")
    op.drop_column("knowledge_base_entries", "topic")
