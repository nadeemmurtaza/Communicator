"""Add Slack sync and issue tracker integrations (Linear, GitHub, Jira)

Revision ID: b3e5f6a7c8d9
Revises: a1b2c3d4e5f6
Create Date: 2026-10-22 04:45:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "b3e5f6a7c8d9"
down_revision: str | Sequence[str] | None = "a1b2c3d4e5f6"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create issue_tracker_configs table
    op.create_table(
        "issue_tracker_configs",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("provider", sa.String(length=50), nullable=False),
        sa.Column("account_name", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("is_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("test_mode", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column(
            "credentials",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column(
            "config",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column("last_tested_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column(
            "last_test_status", sa.String(length=50), nullable=False, server_default="untested"
        ),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.Column(
            "updated_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        "ix_issue_tracker_configs_provider", "issue_tracker_configs", ["provider"], unique=True
    )

    # 2. Create linked_issues table
    op.create_table(
        "linked_issues",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("conversation_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("ticket_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("provider", sa.String(length=50), nullable=False),
        sa.Column("external_issue_id", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("issue_key", sa.String(length=100), nullable=False),
        sa.Column("issue_url", sa.String(length=1024), nullable=False),
        sa.Column("title", sa.String(length=512), nullable=False),
        sa.Column("status", sa.String(length=100), nullable=False, server_default="Open"),
        sa.Column("priority", sa.String(length=50), nullable=False, server_default="medium"),
        sa.Column("assigned_to", sa.String(length=255), nullable=False, server_default=""),
        sa.Column(
            "created_by_email",
            sa.String(length=255),
            nullable=False,
            server_default="agent@newax.com",
        ),
        sa.Column("sync_status", sa.String(length=50), nullable=False, server_default="synced"),
        sa.Column(
            "metadata_json",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.Column(
            "updated_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["ticket_id"], ["tickets.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index("ix_linked_issues_conversation_id", "linked_issues", ["conversation_id"])
    op.create_index("ix_linked_issues_ticket_id", "linked_issues", ["ticket_id"])
    op.create_index("ix_linked_issues_issue_key", "linked_issues", ["issue_key"])

    # 3. Create slack_conversation_syncs table
    op.create_table(
        "slack_conversation_syncs",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("conversation_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("slack_channel_id", sa.String(length=100), nullable=False),
        sa.Column(
            "slack_channel_name",
            sa.String(length=255),
            nullable=False,
            server_default="#customer-support",
        ),
        sa.Column("slack_thread_ts", sa.String(length=100), nullable=False),
        sa.Column("is_synced", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column(
            "last_synced_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        "ix_slack_conversation_syncs_conversation_id",
        "slack_conversation_syncs",
        ["conversation_id"],
        unique=True,
    )


def downgrade() -> None:
    op.drop_index(
        "ix_slack_conversation_syncs_conversation_id", table_name="slack_conversation_syncs"
    )
    op.drop_table("slack_conversation_syncs")
    op.drop_index("ix_linked_issues_issue_key", table_name="linked_issues")
    op.drop_index("ix_linked_issues_ticket_id", table_name="linked_issues")
    op.drop_index("ix_linked_issues_conversation_id", table_name="linked_issues")
    op.drop_table("linked_issues")
    op.drop_index("ix_issue_tracker_configs_provider", table_name="issue_tracker_configs")
    op.drop_table("issue_tracker_configs")
