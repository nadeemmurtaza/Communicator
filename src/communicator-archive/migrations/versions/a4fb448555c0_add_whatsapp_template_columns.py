"""Add WhatsApp template columns: sequence_steps.template_sid, drafts.content_sid

Revision ID: a4fb448555c0
Revises: 8cf14977a3a6
Create Date: 2026-07-31 23:00:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

revision: str = "a4fb448555c0"
down_revision: str | Sequence[str] | None = "8cf14977a3a6"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column(
        "sequence_steps",
        sa.Column("template_sid", sa.String(length=255), nullable=False, server_default=""),
    )
    op.alter_column("sequence_steps", "template_sid", server_default=None)

    op.add_column(
        "drafts",
        sa.Column("content_sid", sa.String(length=255), nullable=False, server_default=""),
    )
    op.alter_column("drafts", "content_sid", server_default=None)


def downgrade() -> None:
    op.drop_column("drafts", "content_sid")
    op.drop_column("sequence_steps", "template_sid")
