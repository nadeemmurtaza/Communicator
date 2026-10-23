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

Each ADD VALUE statement below is a fully-literal string -- no f-string, no
concatenation, no interpolation of any kind -- specifically so static
analysis never has to reason about whether the value is safe: there is no
dynamic value here to reason about. ALTER TYPE ... ADD VALUE also does not
accept a bind parameter for the value (confirmed against this project's own
Postgres: asyncpg rejects "ALTER TYPE channeltype ADD VALUE IF NOT EXISTS
$1" with a syntax error), so a parameterized statement was never an option
for this one.
"""

from collections.abc import Sequence

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "354b7c787957"
down_revision: str | Sequence[str] | None = "c41400fdcea1"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    """Upgrade schema."""
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'website'")
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'facebook'")
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'instagram'")
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'telegram'")
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'line'")
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'voice'")
    op.execute("ALTER TYPE channeltype ADD VALUE IF NOT EXISTS 'tiktok'")


def downgrade() -> None:
    """Downgrade schema."""
    # Postgres does not support removing enum values; no-op.
