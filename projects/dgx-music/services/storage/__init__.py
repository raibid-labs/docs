"""
Storage service for DGX Music.

This module provides database operations for tracking music generations,
prompts, and metadata using SQLite and SQLAlchemy ORM.

Main exports:
- Models: Generation, Prompt
- Database functions: init_db, get_session
- CRUD operations: create_generation, get_generation, etc.
- Status constants: GenerationStatus
"""

from .models import Generation, Prompt, Base
from .schema import GenerationStatus, validate_status, get_schema_version
from .database import (
    init_db,
    get_session,
    get_engine,
    reset_database,
    get_database_stats,
    # Generation CRUD
    create_generation,
    get_generation,
    get_all_generations,
    update_generation_status,
    complete_generation,
    delete_generation,
    get_generations_by_status,
    get_pending_generations,
    count_generations,
    # Prompt CRUD
    track_prompt_usage,
    get_prompt,
    get_prompt_by_text,
    get_all_prompts,
    get_most_used_prompts,
)

__all__ = [
    # Models
    "Generation",
    "Prompt",
    "Base",
    # Status
    "GenerationStatus",
    "validate_status",
    "get_schema_version",
    # Database management
    "init_db",
    "get_session",
    "get_engine",
    "reset_database",
    "get_database_stats",
    # Generation CRUD
    "create_generation",
    "get_generation",
    "get_all_generations",
    "update_generation_status",
    "complete_generation",
    "delete_generation",
    "get_generations_by_status",
    "get_pending_generations",
    "count_generations",
    # Prompt CRUD
    "track_prompt_usage",
    "get_prompt",
    "get_prompt_by_text",
    "get_all_prompts",
    "get_most_used_prompts",
]
