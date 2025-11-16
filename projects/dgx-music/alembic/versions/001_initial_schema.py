"""Initial schema for DGX Music

Revision ID: 001
Revises:
Create Date: 2025-11-07 00:00:00.000000

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision: str = '001'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """Create initial tables for generations and prompts."""

    # Create generations table
    op.create_table(
        'generations',
        sa.Column('id', sa.String(), nullable=False),
        sa.Column('prompt', sa.Text(), nullable=False),
        sa.Column('model_name', sa.String(), nullable=False),
        sa.Column('model_version', sa.String(), nullable=True),
        sa.Column('duration_seconds', sa.Float(), nullable=False),
        sa.Column('sample_rate', sa.Integer(), nullable=False),
        sa.Column('channels', sa.Integer(), nullable=False),
        sa.Column('file_path', sa.String(), nullable=False),
        sa.Column('file_size_bytes', sa.Integer(), nullable=True),
        sa.Column('status', sa.String(), nullable=False),
        sa.Column('created_at', sa.DateTime(), nullable=False),
        sa.Column('completed_at', sa.DateTime(), nullable=True),
        sa.Column('generation_time_seconds', sa.Float(), nullable=True),
        sa.Column('error_message', sa.Text(), nullable=True),
        sa.Column('metadata', sa.Text(), nullable=True),
        sa.PrimaryKeyConstraint('id')
    )

    # Create indices for generations table
    op.create_index('idx_generations_status', 'generations', ['status'])
    op.create_index('idx_generations_created_at', 'generations', ['created_at'])
    op.create_index('idx_generations_model_name', 'generations', ['model_name'])
    op.create_index('idx_generations_completed_at', 'generations', ['completed_at'])

    # Create prompts table
    op.create_table(
        'prompts',
        sa.Column('id', sa.Integer(), nullable=False),
        sa.Column('text', sa.Text(), nullable=False),
        sa.Column('used_count', sa.Integer(), nullable=False),
        sa.Column('first_used_at', sa.DateTime(), nullable=False),
        sa.Column('last_used_at', sa.DateTime(), nullable=False),
        sa.PrimaryKeyConstraint('id'),
        sa.UniqueConstraint('text')
    )

    # Create index for prompts table
    op.create_index('idx_prompts_text', 'prompts', ['text'])


def downgrade() -> None:
    """Drop all tables."""

    # Drop indices
    op.drop_index('idx_prompts_text', table_name='prompts')
    op.drop_index('idx_generations_completed_at', table_name='generations')
    op.drop_index('idx_generations_model_name', table_name='generations')
    op.drop_index('idx_generations_created_at', table_name='generations')
    op.drop_index('idx_generations_status', table_name='generations')

    # Drop tables
    op.drop_table('prompts')
    op.drop_table('generations')
