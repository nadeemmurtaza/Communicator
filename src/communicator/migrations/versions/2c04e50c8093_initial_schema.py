"""Initial schema: leads, contacts, consent, conversations, messages, services, sequences, drafts, audit log

Revision ID: 2c04e50c8093
Revises:
Create Date: 2026-07-31 00:00:00.000000

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision: str = "2c04e50c8093"
down_revision: str | Sequence[str] | None = None
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None

# create_type=False: this type is shared by three tables (contacts,
# sequence_steps, messages) below. It's created explicitly, once, at the
# top of upgrade() instead of letting the first CREATE TABLE that
# references it auto-create it -- otherwise a raw SQL dump of this
# migration (`alembic upgrade head --sql`) emits a duplicate `CREATE TYPE
# channeltype` when the type is later referenced verbatim, which fails
# outside of Alembic's own checkfirst-guarded execution.
channel_type_enum = postgresql.ENUM(
    "EMAIL", "SMS", "WHATSAPP", name="channeltype", create_type=False
)


def upgrade() -> None:
    """Upgrade schema."""
    channel_type_enum.create(op.get_bind(), checkfirst=True)

    op.create_table(
        "leads",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("harvester_institution_id", sa.String(length=64), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("institution_type", sa.String(length=100), nullable=False),
        sa.Column("city", sa.String(length=255), nullable=False),
        sa.Column("qualification_rationale", sa.Text(), nullable=False),
        sa.Column("synced_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_leads_harvester_institution_id"),
        "leads",
        ["harvester_institution_id"],
        unique=True,
    )

    op.create_table(
        "contacts",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("lead_id", sa.UUID(), nullable=False),
        sa.Column("harvester_contact_point_id", sa.String(length=64), nullable=False),
        sa.Column("channel", channel_type_enum, nullable=False),
        sa.Column("value", sa.String(length=255), nullable=False),
        sa.Column("person_name", sa.String(length=255), nullable=False),
        sa.Column("role", sa.String(length=255), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["lead_id"], ["leads.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_contacts_lead_id"), "contacts", ["lead_id"], unique=False)
    op.create_index(
        op.f("ix_contacts_harvester_contact_point_id"),
        "contacts",
        ["harvester_contact_point_id"],
        unique=False,
    )
    op.create_index(op.f("ix_contacts_value"), "contacts", ["value"], unique=False)

    op.create_table(
        "consent_records",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("contact_id", sa.UUID(), nullable=False),
        sa.Column(
            "status",
            sa.Enum("UNKNOWN", "OPTED_IN", "OPTED_OUT", name="consentstatus"),
            nullable=False,
        ),
        sa.Column("source", sa.String(length=100), nullable=False),
        sa.Column("note", sa.Text(), nullable=False),
        sa.Column("recorded_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_consent_records_contact_id"), "consent_records", ["contact_id"], unique=False
    )

    op.create_table(
        "follow_up_sequences",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("description", sa.Text(), nullable=False),
        sa.Column("is_active", sa.Boolean(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )

    op.create_table(
        "sequence_steps",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("sequence_id", sa.UUID(), nullable=False),
        sa.Column("step_number", sa.Integer(), nullable=False),
        sa.Column("delay_days", sa.Integer(), nullable=False),
        sa.Column("channel", channel_type_enum, nullable=False),
        sa.Column("template", sa.Text(), nullable=False),
        sa.ForeignKeyConstraint(["sequence_id"], ["follow_up_sequences.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_sequence_steps_sequence_id"), "sequence_steps", ["sequence_id"], unique=False
    )

    op.create_table(
        "conversations",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("contact_id", sa.UUID(), nullable=False),
        sa.Column(
            "state",
            sa.Enum(
                "NEW",
                "AWAITING_REPLY",
                "ENGAGED",
                "HOT_LEAD",
                "HANDED_OFF",
                "OPTED_OUT",
                "CLOSED_WON",
                "CLOSED_LOST",
                name="conversationstate",
            ),
            nullable=False,
        ),
        sa.Column("sequence_id", sa.UUID(), nullable=True),
        sa.Column("sequence_step", sa.Integer(), nullable=False),
        sa.Column("next_followup_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["contact_id"], ["contacts.id"]),
        sa.ForeignKeyConstraint(["sequence_id"], ["follow_up_sequences.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_conversations_contact_id"), "conversations", ["contact_id"], unique=False
    )

    op.create_table(
        "messages",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("conversation_id", sa.UUID(), nullable=False),
        sa.Column(
            "direction", sa.Enum("INBOUND", "OUTBOUND", name="messagedirection"), nullable=False
        ),
        sa.Column("channel", channel_type_enum, nullable=False),
        sa.Column("body", sa.Text(), nullable=False),
        sa.Column("provider", sa.String(length=50), nullable=False),
        sa.Column("provider_message_id", sa.String(length=255), nullable=False),
        sa.Column(
            "status",
            sa.Enum("QUEUED", "SENT", "DELIVERED", "FAILED", "RECEIVED", name="messagestatus"),
            nullable=False,
        ),
        sa.Column("provider_metadata", postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column("sent_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        op.f("ix_messages_conversation_id"), "messages", ["conversation_id"], unique=False
    )
    op.create_index(
        op.f("ix_messages_provider_message_id"), "messages", ["provider_message_id"], unique=False
    )

    op.create_table(
        "services",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("summary", sa.Text(), nullable=False),
        sa.Column("details", sa.Text(), nullable=False),
        sa.Column(
            "target_institution_types", postgresql.JSONB(astext_type=sa.Text()), nullable=False
        ),
        sa.Column("faq", postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column("pricing_notes", sa.Text(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )

    op.create_table(
        "drafts",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("conversation_id", sa.UUID(), nullable=False),
        sa.Column("body", sa.Text(), nullable=False),
        sa.Column("confidence", sa.Float(), nullable=False),
        sa.Column("citations", postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column(
            "status",
            sa.Enum(
                "PENDING_REVIEW",
                "APPROVED",
                "AUTO_APPROVED",
                "REJECTED",
                "SENT",
                name="draftstatus",
            ),
            nullable=False,
        ),
        sa.Column("reviewer", sa.String(length=255), nullable=False),
        sa.Column("reviewer_feedback", sa.Text(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_drafts_conversation_id"), "drafts", ["conversation_id"], unique=False)

    op.create_table(
        "audit_logs",
        sa.Column("id", sa.UUID(), nullable=False),
        sa.Column("entity_type", sa.String(length=50), nullable=False),
        sa.Column("entity_id", sa.UUID(), nullable=False),
        sa.Column("action", sa.String(length=100), nullable=False),
        sa.Column("actor", sa.String(length=100), nullable=False),
        sa.Column("details", postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_audit_logs_entity_id"), "audit_logs", ["entity_id"], unique=False)


def downgrade() -> None:
    """Downgrade schema."""
    op.drop_index(op.f("ix_audit_logs_entity_id"), table_name="audit_logs")
    op.drop_table("audit_logs")

    op.drop_index(op.f("ix_drafts_conversation_id"), table_name="drafts")
    op.drop_table("drafts")

    op.drop_table("services")

    op.drop_index(op.f("ix_messages_provider_message_id"), table_name="messages")
    op.drop_index(op.f("ix_messages_conversation_id"), table_name="messages")
    op.drop_table("messages")

    op.drop_index(op.f("ix_conversations_contact_id"), table_name="conversations")
    op.drop_table("conversations")

    op.drop_index(op.f("ix_sequence_steps_sequence_id"), table_name="sequence_steps")
    op.drop_table("sequence_steps")

    op.drop_table("follow_up_sequences")

    op.drop_index(op.f("ix_consent_records_contact_id"), table_name="consent_records")
    op.drop_table("consent_records")

    op.drop_index(op.f("ix_contacts_value"), table_name="contacts")
    op.drop_index(op.f("ix_contacts_harvester_contact_point_id"), table_name="contacts")
    op.drop_index(op.f("ix_contacts_lead_id"), table_name="contacts")
    op.drop_table("contacts")

    op.drop_index(op.f("ix_leads_harvester_institution_id"), table_name="leads")
    op.drop_table("leads")

    sa.Enum(name="draftstatus").drop(op.get_bind(), checkfirst=True)
    sa.Enum(name="messagestatus").drop(op.get_bind(), checkfirst=True)
    sa.Enum(name="messagedirection").drop(op.get_bind(), checkfirst=True)
    sa.Enum(name="conversationstate").drop(op.get_bind(), checkfirst=True)
    sa.Enum(name="consentstatus").drop(op.get_bind(), checkfirst=True)
    channel_type_enum.drop(op.get_bind(), checkfirst=True)
