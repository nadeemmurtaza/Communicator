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

import re
from collections.abc import Sequence

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "354b7c787957"
down_revision: str | Sequence[str] | None = "c41400fdcea1"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None

_NEW_VALUES = ("website", "facebook", "instagram", "telegram", "line", "voice", "tiktok")
_SAFE_ENUM_VALUE = re.compile(r"^[a-z_]+$")


def upgrade() -> None:
    """Upgrade schema."""
    # ALTER TYPE ... ADD VALUE does not accept a bind parameter for the value
    # (confirmed against this project's own Postgres: asyncpg rejects
    # "ALTER TYPE channeltype ADD VALUE IF NOT EXISTS $1" with a syntax
    # error -- DDL in Postgres takes a literal here, not a parameter slot),
    # so Semgrep's raw-SQL-concatenation rule can't be satisfied by
    # parameterizing it the normal way. The value is additionally validated
    # against a strict allowlist regex before interpolation, and every
    # entry comes from the fixed tuple above, never request input, so this
    # is not the injection the rule is written to catch.
    for value in _NEW_VALUES:
        if not _SAFE_ENUM_VALUE.match(value):
            raise ValueError(f"Unsafe enum value in migration: {value!r}")
        # nosemgrep: python.sqlalchemy.security.sqlalchemy-execute-raw-query.sqlalchemy-execute-raw-query
        # nosemgrep: python.lang.security.audit.formatted-sql-query.formatted-sql-query
        op.execute(f"ALTER TYPE channeltype ADD VALUE IF NOT EXISTS '{value}'")


def downgrade() -> None:
    """Downgrade schema."""
    # Postgres does not support removing enum values; no-op.
