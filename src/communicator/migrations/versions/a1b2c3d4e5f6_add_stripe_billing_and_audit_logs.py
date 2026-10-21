"""Add Stripe billing integrations and audit logs

Revision ID: a1b2c3d4e5f6
Revises: f1a2b3c4d5e6
Create Date: 2026-10-22 03:50:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "a1b2c3d4e5f6"
down_revision: str | Sequence[str] | None = "f1a2b3c4d5e6"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create stripe_configs table
    op.create_table(
        "stripe_configs",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("account_name", sa.String(length=255), nullable=False, server_default="Primary Stripe Account"),
        sa.Column("api_key", sa.Text(), nullable=False, server_default=""),
        sa.Column("publishable_key", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("webhook_secret", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("is_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("test_mode", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("default_currency", sa.String(length=10), nullable=False, server_default="usd"),
        sa.Column("last_tested_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("last_test_status", sa.String(length=50), nullable=False, server_default="untested"),
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

    # 2. Create stripe_billing_action_logs table
    op.create_table(
        "stripe_billing_action_logs",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("contact_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("customer_identifier", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("action_type", sa.String(length=50), nullable=False),
        sa.Column("amount_cents", sa.Integer(), nullable=False, server_default="0"),
        sa.Column("currency", sa.String(length=10), nullable=False, server_default="usd"),
        sa.Column("status", sa.String(length=50), nullable=False, server_default="succeeded"),
        sa.Column("stripe_id", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("actor_email", sa.String(length=255), nullable=False, server_default="agent@newax.com"),
        sa.Column("reason", sa.Text(), nullable=False, server_default=""),
        sa.Column(
            "details",
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
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index("ix_stripe_billing_action_logs_contact_id", "stripe_billing_action_logs", ["contact_id"])
    op.create_index(
        "ix_stripe_billing_action_logs_customer_identifier",
        "stripe_billing_action_logs",
        ["customer_identifier"],
    )


def downgrade() -> None:
    op.drop_index("ix_stripe_billing_action_logs_customer_identifier", table_name="stripe_billing_action_logs")
    op.drop_index("ix_stripe_billing_action_logs_contact_id", table_name="stripe_billing_action_logs")
    op.drop_table("stripe_billing_action_logs")
    op.drop_table("stripe_configs")
