"""Add intent column to messages

Revision ID: 8cf14977a3a6
Revises: 2c04e50c8093
Create Date: 2026-07-31 21:00:00.000000

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = '8cf14977a3a6'
down_revision: Union[str, Sequence[str], None] = '2c04e50c8093'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column('messages', sa.Column('intent', sa.String(length=50), nullable=False, server_default=''))
    op.alter_column('messages', 'intent', server_default=None)


def downgrade() -> None:
    op.drop_column('messages', 'intent')
