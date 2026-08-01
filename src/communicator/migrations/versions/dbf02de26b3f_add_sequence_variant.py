"""Add A/B variant support to sequence steps and conversations

Revision ID: dbf02de26b3f
Revises: 8cf14977a3a6
Create Date: 2026-08-01 00:00:00.000000

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = 'dbf02de26b3f'
down_revision: Union[str, Sequence[str], None] = '8cf14977a3a6'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column(
        'sequence_steps',
        sa.Column('variant', sa.String(length=50), nullable=False, server_default='control'),
    )
    op.alter_column('sequence_steps', 'variant', server_default=None)

    op.add_column('conversations', sa.Column('sequence_variant', sa.String(length=50), nullable=True))


def downgrade() -> None:
    op.drop_column('conversations', 'sequence_variant')
    op.drop_column('sequence_steps', 'variant')
