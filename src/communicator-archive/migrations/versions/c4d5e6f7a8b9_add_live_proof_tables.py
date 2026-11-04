"""Add LiveProof return and fraud prevention session tables

Revision ID: c4d5e6f7a8b9
Revises: b3e5f6a7c8d9
Create Date: 2026-10-22 17:50:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision: str = "c4d5e6f7a8b9"
down_revision: str | Sequence[str] | None = "b3e5f6a7c8d9"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Create live_proof_sessions table
    op.create_table(
        "live_proof_sessions",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("tenant_id", sa.String(length=64), nullable=False),
        sa.Column("organization_id", sa.String(length=64), nullable=True),
        sa.Column("conversation_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("contact_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("order_id", sa.String(length=128), nullable=True),
        sa.Column("customer_identifier", sa.String(length=255), nullable=False, server_default=""),
        sa.Column("claim_type", sa.String(length=64), nullable=False, server_default="refund"),
        sa.Column("session_token", sa.String(length=128), nullable=False),
        sa.Column(
            "status",
            sa.Enum(
                "PENDING",
                "IN_PROGRESS",
                "CAPTURED",
                "VERIFIED",
                "REJECTED",
                "EXPIRED",
                name="liveproofstatus",
            ),
            nullable=False,
            server_default="PENDING",
        ),
        sa.Column("anti_spoof_nonce", sa.String(length=64), nullable=False),
        sa.Column("max_video_seconds", sa.Integer(), nullable=False, server_default="45"),
        sa.Column("require_video", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("require_photos", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("expires_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("completed_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("rejection_reason", sa.Text(), nullable=True),
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
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("session_token"),
    )
    op.create_index(op.f("ix_live_proof_sessions_tenant_id"), "live_proof_sessions", ["tenant_id"])
    op.create_index(
        op.f("ix_live_proof_sessions_organization_id"), "live_proof_sessions", ["organization_id"]
    )
    op.create_index(
        op.f("ix_live_proof_sessions_conversation_id"), "live_proof_sessions", ["conversation_id"]
    )
    op.create_index(
        op.f("ix_live_proof_sessions_contact_id"), "live_proof_sessions", ["contact_id"]
    )
    op.create_index(op.f("ix_live_proof_sessions_order_id"), "live_proof_sessions", ["order_id"])
    op.create_index(
        op.f("ix_live_proof_sessions_session_token"), "live_proof_sessions", ["session_token"]
    )
    op.create_index(op.f("ix_live_proof_sessions_status"), "live_proof_sessions", ["status"])

    # 2. Create live_proof_evidences table
    op.create_table(
        "live_proof_evidences",
        sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("session_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("tenant_id", sa.String(length=64), nullable=False),
        sa.Column(
            "media_type",
            sa.Enum("LIVE_PHOTO", "LIVE_VIDEO", name="proofmediatype"),
            nullable=False,
        ),
        sa.Column("file_path", sa.String(length=512), nullable=False),
        sa.Column("file_size_bytes", sa.Integer(), nullable=False),
        sa.Column("mime_type", sa.String(length=64), nullable=False),
        sa.Column("sha256_hash", sa.String(length=64), nullable=False),
        sa.Column("duration_seconds", sa.Float(), nullable=True),
        sa.Column("resolution", sa.String(length=32), nullable=True),
        sa.Column(
            "device_fingerprint",
            postgresql.JSONB(astext_type=sa.Text()),
            nullable=True,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column(
            "watermark_verified", sa.Boolean(), nullable=False, server_default=sa.text("true")
        ),
        sa.Column("verification_score", sa.Float(), nullable=False, server_default="1.0"),
        sa.Column(
            "captured_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("TIMEZONE('utc', NOW())"),
            nullable=False,
        ),
        sa.ForeignKeyConstraint(["session_id"], ["live_proof_sessions.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_live_proof_evidences_session_id"), "live_proof_evidences", ["session_id"]
    )
    op.create_index(
        op.f("ix_live_proof_evidences_tenant_id"), "live_proof_evidences", ["tenant_id"]
    )
    op.create_index(
        op.f("ix_live_proof_evidences_sha256_hash"), "live_proof_evidences", ["sha256_hash"]
    )


def downgrade() -> None:
    op.drop_table("live_proof_evidences")
    op.drop_table("live_proof_sessions")
    op.execute("DROP TYPE IF EXISTS proofmediatype")
    op.execute("DROP TYPE IF EXISTS liveproofstatus")
