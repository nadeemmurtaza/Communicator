"""Add inboxes, teams, contact notes, and canned responses

Revision ID: c3d4e5f6a7b8
Revises: b2c3d4e5f6a7
Create Date: 2026-10-21 14:00:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "c3d4e5f6a7b8"
down_revision: str | Sequence[str] | None = "b2c3d4e5f6a7"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create inboxes table
    op.create_table(
        "inboxes",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column(
            "channel_type",
            postgresql.ENUM("email", "sms", "whatsapp", name="channeltype", create_type=False),
            nullable=False,
        ),
        sa.Column("phone_number", sa.String(length=50), nullable=False, server_default=""),
        sa.Column("avatar_url", sa.String(length=500), nullable=False, server_default=""),
        sa.Column("is_active", sa.Boolean(), nullable=False, server_default="true"),
        sa.Column(
            "credentials",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="{}",
        ),
        sa.Column("greeting_enabled", sa.Boolean(), nullable=False, server_default="true"),
        sa.Column("greeting_message", sa.Text(), nullable=False, server_default=""),
        sa.Column("out_of_office_enabled", sa.Boolean(), nullable=False, server_default="false"),
        sa.Column("out_of_office_message", sa.Text(), nullable=False, server_default=""),
        sa.Column("csat_enabled", sa.Boolean(), nullable=False, server_default="true"),
        sa.Column("auto_read_receipts", sa.Boolean(), nullable=False, server_default="true"),
        sa.Column("working_hours_enabled", sa.Boolean(), nullable=False, server_default="false"),
        sa.Column("working_hours_start", sa.Integer(), nullable=False, server_default="9"),
        sa.Column("working_hours_end", sa.Integer(), nullable=False, server_default="18"),
        sa.Column("timezone", sa.String(length=50), nullable=False, server_default="UTC"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_inboxes_channel_type"), "inboxes", ["channel_type"], unique=False)

    # 2. Create teams table
    op.create_table(
        "teams",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("name", sa.String(length=100), nullable=False),
        sa.Column("description", sa.Text(), nullable=False, server_default=""),
        sa.Column("auto_assign", sa.Boolean(), nullable=False, server_default="true"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("name"),
    )

    # 3. Create team_memberships table
    op.create_table(
        "team_memberships",
        sa.Column("team_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("team_member_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.ForeignKeyConstraint(["team_id"], ["teams.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["team_member_id"], ["team_members.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("team_id", "team_member_id"),
    )

    # 4. Create canned_responses table
    op.create_table(
        "canned_responses",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("short_code", sa.String(length=50), nullable=False),
        sa.Column("content", sa.Text(), nullable=False),
        sa.Column("category", sa.String(length=50), nullable=False, server_default="general"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_canned_responses_short_code"), "canned_responses", ["short_code"], unique=True
    )

    # 5. Create contact_notes table
    op.create_table(
        "contact_notes",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("contact_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("author_name", sa.String(length=100), nullable=False, server_default="Agent"),
        sa.Column("body", sa.Text(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_contact_notes_contact_id"), "contact_notes", ["contact_id"], unique=False
    )

    # 6. Add columns to conversations
    op.add_column(
        "conversations", sa.Column("inbox_id", postgresql.UUID(as_uuid=True), nullable=True)
    )
    op.add_column(
        "conversations",
        sa.Column("priority", sa.String(length=20), nullable=False, server_default="medium"),
    )
    op.add_column(
        "conversations", sa.Column("team_id", postgresql.UUID(as_uuid=True), nullable=True)
    )
    op.add_column(
        "conversations", sa.Column("snoozed_until", sa.DateTime(timezone=True), nullable=True)
    )
    op.add_column(
        "conversations",
        sa.Column(
            "custom_attributes",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="{}",
        ),
    )
    op.create_foreign_key(
        "fk_conversations_inbox_id",
        "conversations",
        "inboxes",
        ["inbox_id"],
        ["id"],
        ondelete="SET NULL",
    )
    op.create_foreign_key(
        "fk_conversations_team_id",
        "conversations",
        "teams",
        ["team_id"],
        ["id"],
        ondelete="SET NULL",
    )
    op.create_index(op.f("ix_conversations_inbox_id"), "conversations", ["inbox_id"], unique=False)
    op.create_index(op.f("ix_conversations_team_id"), "conversations", ["team_id"], unique=False)

    # 7. Add columns to messages
    op.add_column(
        "messages", sa.Column("is_private", sa.Boolean(), nullable=False, server_default="false")
    )
    op.add_column(
        "messages",
        sa.Column("sender_type", sa.String(length=20), nullable=False, server_default="customer"),
    )
    op.add_column(
        "messages",
        sa.Column("sender_name", sa.String(length=100), nullable=False, server_default=""),
    )
    op.add_column(
        "messages",
        sa.Column(
            "attachments",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="[]",
        ),
    )


def downgrade() -> None:
    op.drop_column("messages", "attachments")
    op.drop_column("messages", "sender_name")
    op.drop_column("messages", "sender_type")
    op.drop_column("messages", "is_private")

    op.drop_index(op.f("ix_conversations_team_id"), table_name="conversations")
    op.drop_index(op.f("ix_conversations_inbox_id"), table_name="conversations")
    op.drop_constraint("fk_conversations_team_id", "conversations", type_="foreignkey")
    op.drop_constraint("fk_conversations_inbox_id", "conversations", type_="foreignkey")
    op.drop_column("conversations", "custom_attributes")
    op.drop_column("conversations", "snoozed_until")
    op.drop_column("conversations", "team_id")
    op.drop_column("conversations", "priority")
    op.drop_column("conversations", "inbox_id")

    op.drop_index(op.f("ix_contact_notes_contact_id"), table_name="contact_notes")
    op.drop_table("contact_notes")
    op.drop_index(op.f("ix_canned_responses_short_code"), table_name="canned_responses")
    op.drop_table("canned_responses")
    op.drop_table("team_memberships")
    op.drop_table("teams")
    op.drop_index(op.f("ix_inboxes_channel_type"), table_name="inboxes")
    op.drop_table("inboxes")
