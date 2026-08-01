"""Add language column to contacts

Revision ID: f3a9c1d7e2b4
Revises: dbf02de26b3f
Create Date: 2026-08-01 00:10:00.000000

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = 'f3a9c1d7e2b4'
down_revision: Union[str, Sequence[str], None] = 'dbf02de26b3f'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column(
        'contacts', sa.Column('language', sa.String(length=10), nullable=False, server_default='en')
    )
    op.alter_column('contacts', 'language', server_default=None)


def downgrade() -> None:
    op.drop_column('contacts', 'language')
