"""add missing channeltype enum values

Revision ID: 354b7c787957
Revises: c41400fdcea1
Create Date: 2026-10-22 14:30:00.000000

The ChannelType Python enum (communicator.models.contact) has long included
facebook/instagram/telegram/line/voice/tiktok, but the Postgres "channeltype"
enum was only ever created with email/sms/whatsapp -- no migration backfilled
the rest, so any query filtering by one of those channels (e.g. the
Instagram webhook verify endpoint) fails with
"invalid input value for enum channeltype".
"""

from collections.abc import Sequence

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "354b7c787957"
down_revision: str | Sequence[str] | None = "c41400fdcea1"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None

_NEW_VALUES = ("website", "facebook", "instagram", "telegram", "line", "voice", "tiktok")


def upgrade() -> None:
    """Upgrade schema."""
    for value in _NEW_VALUES:
        op.execute(f"ALTER TYPE channeltype ADD VALUE IF NOT EXISTS '{value}'")


def downgrade() -> None:
    """Downgrade schema."""
    # Postgres does not support removing enum values; no-op.
    pass
