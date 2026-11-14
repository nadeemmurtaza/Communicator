"""Add AI learning, contact memory, campaigns and autonomous resolution

Revision ID: b2c3d4e5f6a7
Revises: 9a1b2c3d4e5f
Create Date: 2026-10-21 10:00:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "b2c3d4e5f6a7"
down_revision: str | Sequence[str] | None = "9a1b2c3d4e5f"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create contact_memories table
    op.create_table(
        "contact_memories",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("contact_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("fact", sa.Text(), nullable=False),
        sa.Column("category", sa.String(length=50), nullable=False, server_default="preference"),
        sa.Column("confidence", sa.Float(), nullable=False, server_default="1.0"),
        sa.Column("source_message_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("noted_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["source_message_id"], ["messages.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_contact_memories_contact_id"), "contact_memories", ["contact_id"], unique=False
    )

    # 2. Add columns to knowledge_base_entries
    op.add_column(
        "knowledge_base_entries", sa.Column("source_url", sa.String(length=1024), nullable=True)
    )
    op.add_column(
        "knowledge_base_entries", sa.Column("file_name", sa.String(length=255), nullable=True)
    )

    # 3. Create knowledge_gaps table
    op.create_table(
        "knowledge_gaps",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("question", sa.String(length=500), nullable=False),
        sa.Column("suggested_answer", sa.Text(), nullable=False, server_default=""),
        sa.Column("source_query", sa.Text(), nullable=False, server_default=""),
        sa.Column("frequency", sa.Integer(), nullable=False, server_default="1"),
        sa.Column("status", sa.String(length=30), nullable=False, server_default="suggested"),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )

    # 4. Add campaign / broadcast fields
    op.add_column(
        "broadcasts", sa.Column("audience_count", sa.Integer(), nullable=False, server_default="0")
    )
    op.add_column(
        "broadcasts", sa.Column("skipped_count", sa.Integer(), nullable=False, server_default="0")
    )
    op.add_column(
        "broadcasts",
        sa.Column(
            "template_params",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=False,
            server_default="{}",
        ),
    )

    op.add_column(
        "broadcast_recipients", sa.Column("sent_at", sa.DateTime(timezone=True), nullable=True)
    )
    op.add_column(
        "broadcast_recipients", sa.Column("delivered_at", sa.DateTime(timezone=True), nullable=True)
    )
    op.add_column(
        "broadcast_recipients", sa.Column("read_at", sa.DateTime(timezone=True), nullable=True)
    )

    # 5. Add resolution_type and broadcast_id to conversations
    op.add_column(
        "conversations", sa.Column("resolution_type", sa.String(length=50), nullable=True)
    )
    op.add_column(
        "conversations", sa.Column("broadcast_id", postgresql.UUID(as_uuid=True), nullable=True)
    )
    op.create_foreign_key(
        "fk_conversations_broadcast_id",
        "conversations",
        "broadcasts",
        ["broadcast_id"],
        ["id"],
        ondelete="SET NULL",
    )


def downgrade() -> None:
    op.drop_constraint("fk_conversations_broadcast_id", "conversations", type_="foreignkey")
    op.drop_column("conversations", "broadcast_id")
    op.drop_column("conversations", "resolution_type")
    op.drop_column("broadcast_recipients", "read_at")
    op.drop_column("broadcast_recipients", "delivered_at")
    op.drop_column("broadcast_recipients", "sent_at")
    op.drop_column("broadcasts", "template_params")
    op.drop_column("broadcasts", "skipped_count")
    op.drop_column("broadcasts", "audience_count")
    op.drop_table("knowledge_gaps")
    op.drop_column("knowledge_base_entries", "file_name")
    op.drop_column("knowledge_base_entries", "source_url")
    op.drop_index(op.f("ix_contact_memories_contact_id"), table_name="contact_memories")
    op.drop_table("contact_memories")
