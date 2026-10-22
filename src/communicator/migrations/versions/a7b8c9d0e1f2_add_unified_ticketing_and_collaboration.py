"""Add unified ticketing, back-office collaboration, and tracker tickets

Revision ID: a7b8c9d0e1f2
Revises: f6a7b8c9d0e1
Create Date: 2026-10-22 01:30:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "a7b8c9d0e1f2"
down_revision: str | Sequence[str] | None = "f6a7b8c9d0e1"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create tickets table
    op.create_table(
        "tickets",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("ticket_number", sa.String(length=32), nullable=False),
        sa.Column("title", sa.String(length=255), nullable=False),
        sa.Column("description", sa.Text(), nullable=False),
        sa.Column(
            "ticket_type", sa.String(length=50), nullable=False, server_default="customer_issue"
        ),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="open"),
        sa.Column("priority", sa.String(length=20), nullable=False, server_default="medium"),
        sa.Column("department", sa.String(length=50), nullable=False, server_default="support"),
        sa.Column("conversation_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("contact_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("tracker_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("assigned_to_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("resolution_summary", sa.Text(), nullable=True),
        sa.Column(
            "customer_broadcast_sent", sa.Boolean(), nullable=False, server_default=sa.text("false")
        ),
        sa.Column("resolved_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column(
            "created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()
        ),
        sa.Column(
            "updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()
        ),
        sa.ForeignKeyConstraint(["assigned_to_id"], ["team_members.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["tracker_id"], ["tickets.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_tickets_ticket_number"), "tickets", ["ticket_number"], unique=True)
    op.create_index(op.f("ix_tickets_ticket_type"), "tickets", ["ticket_type"], unique=False)
    op.create_index(op.f("ix_tickets_status"), "tickets", ["status"], unique=False)
    op.create_index(op.f("ix_tickets_department"), "tickets", ["department"], unique=False)
    op.create_index(
        op.f("ix_tickets_conversation_id"), "tickets", ["conversation_id"], unique=False
    )
    op.create_index(op.f("ix_tickets_contact_id"), "tickets", ["contact_id"], unique=False)
    op.create_index(op.f("ix_tickets_tracker_id"), "tickets", ["tracker_id"], unique=False)
    op.create_index(op.f("ix_tickets_assigned_to_id"), "tickets", ["assigned_to_id"], unique=False)

    # 2. Create ticket_activities table
    op.create_table(
        "ticket_activities",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("ticket_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("author_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("author_name", sa.String(length=100), nullable=False, server_default="Agent"),
        sa.Column("activity_type", sa.String(length=50), nullable=False, server_default="comment"),
        sa.Column("message", sa.Text(), nullable=False),
        sa.Column(
            "activity_metadata",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="{}",
        ),
        sa.Column(
            "created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.func.now()
        ),
        sa.ForeignKeyConstraint(["author_id"], ["team_members.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["ticket_id"], ["tickets.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_ticket_activities_ticket_id"), "ticket_activities", ["ticket_id"], unique=False
    )


def downgrade() -> None:
    op.drop_index(op.f("ix_ticket_activities_ticket_id"), table_name="ticket_activities")
    op.drop_table("ticket_activities")

    op.drop_index(op.f("ix_tickets_assigned_to_id"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_tracker_id"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_contact_id"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_conversation_id"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_department"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_status"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_ticket_type"), table_name="tickets")
    op.drop_index(op.f("ix_tickets_ticket_number"), table_name="tickets")
    op.drop_table("tickets")
