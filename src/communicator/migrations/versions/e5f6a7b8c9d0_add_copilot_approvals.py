"""Add copilot_approvals table for teammate approval workflows

Revision ID: e5f6a7b8c9d0
Revises: d4e5f6a7b8c9
Create Date: 2026-10-21 23:45:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "e5f6a7b8c9d0"
down_revision: str | Sequence[str] | None = "d4e5f6a7b8c9"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "copilot_approvals",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("conversation_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("contact_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("request_type", sa.String(length=50), nullable=False, server_default="refund"),
        sa.Column("status", sa.String(length=50), nullable=False, server_default="pending"),
        sa.Column("title", sa.String(length=255), nullable=False),
        sa.Column("proposed_reply", sa.Text(), nullable=False, server_default=""),
        sa.Column("amended_reply", sa.Text(), nullable=True),
        sa.Column("action_data", postgresql.JSONB(astext_type=sa.Text()), nullable=False, server_default="{}"),
        sa.Column("internal_note", sa.Text(), nullable=False, server_default=""),
        sa.Column("question_to_teammate", sa.Text(), nullable=True),
        sa.Column("teammate_answer", sa.Text(), nullable=True),
        sa.Column("assigned_to_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("reviewer_name", sa.String(length=120), nullable=True),
        sa.Column("reviewer_feedback", sa.Text(), nullable=True),
        sa.Column("dispatched_message_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("resolved_at", sa.DateTime(timezone=True), nullable=True),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["assigned_to_id"], ["team_members.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["dispatched_message_id"], ["messages.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_copilot_approvals_conversation_id"), "copilot_approvals", ["conversation_id"], unique=False)
    op.create_index(op.f("ix_copilot_approvals_contact_id"), "copilot_approvals", ["contact_id"], unique=False)
    op.create_index(op.f("ix_copilot_approvals_request_type"), "copilot_approvals", ["request_type"], unique=False)
    op.create_index(op.f("ix_copilot_approvals_status"), "copilot_approvals", ["status"], unique=False)


def downgrade() -> None:
    op.drop_index(op.f("ix_copilot_approvals_status"), table_name="copilot_approvals")
    op.drop_index(op.f("ix_copilot_approvals_request_type"), table_name="copilot_approvals")
    op.drop_index(op.f("ix_copilot_approvals_contact_id"), table_name="copilot_approvals")
    op.drop_index(op.f("ix_copilot_approvals_conversation_id"), table_name="copilot_approvals")
    op.drop_table("copilot_approvals")
