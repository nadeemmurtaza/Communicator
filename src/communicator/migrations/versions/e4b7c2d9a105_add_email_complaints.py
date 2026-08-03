"""Add append-only email complaint evidence

Revision ID: e4b7c2d9a105
Revises: dbf02de26b3f
Create Date: 2026-08-04 01:20:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "e4b7c2d9a105"
down_revision: str | Sequence[str] | None = "dbf02de26b3f"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "email_complaints",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("contact_id", sa.UUID(), nullable=True),
        sa.Column("message_id", sa.UUID(), nullable=True),
        sa.Column("source_event_id", sa.String(length=255), nullable=False),
        sa.Column(
            "complaint_type",
            sa.Enum(
                "RECIPIENT_SPAM",
                "FEEDBACK_LOOP",
                "DMARC_FORENSIC",
                "AUTHENTICATION_FAILURE",
                "OTHER",
                name="emailcomplainttype",
            ),
            nullable=False,
        ),
        sa.Column("provider", sa.String(length=64), nullable=False),
        sa.Column("provider_message_id", sa.String(length=255), nullable=False),
        sa.Column("recipient_email", sa.String(length=320), nullable=False),
        sa.Column("dmarc_result", sa.String(length=32), nullable=False),
        sa.Column("spf_aligned", sa.Boolean(), nullable=True),
        sa.Column("dkim_aligned", sa.Boolean(), nullable=True),
        sa.Column("source_ip", sa.String(length=64), nullable=False),
        sa.Column(
            "evidence_metadata", postgresql.JSONB(astext_type=sa.Text()), nullable=False
        ),
        sa.Column("suppression_applied", sa.Boolean(), nullable=False),
        sa.Column("reported_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("received_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["message_id"], ["messages.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("source_event_id"),
    )
    op.create_index(
        op.f("ix_email_complaints_contact_id"), "email_complaints", ["contact_id"], unique=False
    )
    op.create_index(
        op.f("ix_email_complaints_message_id"), "email_complaints", ["message_id"], unique=False
    )
    op.create_index(
        op.f("ix_email_complaints_recipient_email"),
        "email_complaints",
        ["recipient_email"],
        unique=False,
    )
    op.create_index(
        "ix_email_complaints_recipient_reported",
        "email_complaints",
        ["recipient_email", "reported_at"],
        unique=False,
    )
    op.create_index(
        "ix_email_complaints_provider_reported",
        "email_complaints",
        ["provider", "reported_at"],
        unique=False,
    )


def downgrade() -> None:
    op.drop_index("ix_email_complaints_provider_reported", table_name="email_complaints")
    op.drop_index("ix_email_complaints_recipient_reported", table_name="email_complaints")
    op.drop_index(op.f("ix_email_complaints_recipient_email"), table_name="email_complaints")
    op.drop_index(op.f("ix_email_complaints_message_id"), table_name="email_complaints")
    op.drop_index(op.f("ix_email_complaints_contact_id"), table_name="email_complaints")
    op.drop_table("email_complaints")
    op.execute("DROP TYPE IF EXISTS emailcomplainttype")
