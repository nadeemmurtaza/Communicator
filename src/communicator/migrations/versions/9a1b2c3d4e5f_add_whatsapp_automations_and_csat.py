"""Add WhatsApp automations fields and CSAT tracking

Revision ID: 9a1b2c3d4e5f
Revises: 36fb10c15827
Create Date: 2026-10-21 08:30:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

# revision identifiers, used by Alembic.
revision: str = "9a1b2c3d4e5f"
down_revision: str | Sequence[str] | None = "36fb10c15827"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Expand rulematchtype and ruleactiontype enums if using Postgres native enums
    op.execute("ALTER TYPE rulematchtype ADD VALUE IF NOT EXISTS 'STARTS_WITH'")
    op.execute("ALTER TYPE rulematchtype ADD VALUE IF NOT EXISTS 'REGEX'")
    op.execute("ALTER TYPE rulematchtype ADD VALUE IF NOT EXISTS 'BUTTON_CLICK'")
    op.execute("ALTER TYPE rulematchtype ADD VALUE IF NOT EXISTS 'LIST_SELECT'")
    op.execute("ALTER TYPE rulematchtype ADD VALUE IF NOT EXISTS 'INTENT'")

    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'INTERACTIVE_BUTTONS'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'INTERACTIVE_LIST'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'SEND_MEDIA'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'SEND_CTA_URL'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'SEND_TEMPLATE'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'REMOVE_TAG'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'ASSIGN_TEAM'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'ENROLL_SEQUENCE'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'SEND_CSAT'")
    op.execute("ALTER TYPE ruleactiontype ADD VALUE IF NOT EXISTS 'TRIGGER_WEBHOOK'")

    # 2. Add description, execution_count, and last_triggered_at to automation_rules
    op.add_column(
        "automation_rules",
        sa.Column("description", sa.String(length=255), nullable=False, server_default=""),
    )
    op.alter_column("automation_rules", "description", server_default=None)

    op.add_column(
        "automation_rules",
        sa.Column("execution_count", sa.Integer(), nullable=False, server_default="0"),
    )
    op.alter_column("automation_rules", "execution_count", server_default=None)

    op.add_column(
        "automation_rules",
        sa.Column("last_triggered_at", sa.DateTime(timezone=True), nullable=True),
    )

    # 3. Add CSAT and session window tracking to conversations
    op.add_column("conversations", sa.Column("csat_score", sa.Integer(), nullable=True))
    op.add_column("conversations", sa.Column("csat_comment", sa.Text(), nullable=True))
    op.add_column(
        "conversations", sa.Column("last_inbound_at", sa.DateTime(timezone=True), nullable=True)
    )
    op.add_column(
        "conversations", sa.Column("last_outbound_at", sa.DateTime(timezone=True), nullable=True)
    )


def downgrade() -> None:
    op.drop_column("conversations", "last_outbound_at")
    op.drop_column("conversations", "last_inbound_at")
    op.drop_column("conversations", "csat_comment")
    op.drop_column("conversations", "csat_score")
    op.drop_column("automation_rules", "last_triggered_at")
    op.drop_column("automation_rules", "execution_count")
    op.drop_column("automation_rules", "description")
