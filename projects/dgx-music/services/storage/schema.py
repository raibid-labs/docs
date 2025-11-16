"""
Database schema definitions for DGX Music.

This module contains the SQL schema for the SQLite database used in the MVP.
The schema tracks music generation jobs, prompts, and metadata.

Schema Design:
- generations: Core table tracking music generation jobs and results
- prompts: Tracks unique prompts and usage statistics

For the full schema diagram, see docs/database-schema.md
"""

# Raw SQL schema for reference and documentation
# The ORM models in models.py provide the actual implementation

SCHEMA_SQL = """
-- generations table
-- Tracks all music generation jobs from creation through completion
CREATE TABLE IF NOT EXISTS generations (
    id TEXT PRIMARY KEY,                    -- UUID v4 identifier
    prompt TEXT NOT NULL,                   -- User's text prompt describing desired music
    model_name TEXT NOT NULL,               -- AI model used (e.g., "musicgen-small")
    model_version TEXT,                     -- Specific model version/checkpoint
    duration_seconds REAL NOT NULL,         -- Target audio duration in seconds
    sample_rate INTEGER NOT NULL,           -- Audio sample rate (e.g., 32000 Hz)
    channels INTEGER NOT NULL,              -- Audio channels (1=mono, 2=stereo)
    file_path TEXT NOT NULL,                -- Relative path to generated WAV file
    file_size_bytes INTEGER,                -- File size in bytes (NULL until completed)
    status TEXT NOT NULL,                   -- Job status: pending/processing/completed/failed
    created_at TIMESTAMP NOT NULL,          -- When the job was created
    completed_at TIMESTAMP,                 -- When the job finished (NULL if not complete)
    generation_time_seconds REAL,           -- Time taken to generate (NULL until completed)
    error_message TEXT,                     -- Error details if status=failed
    metadata JSON                           -- Additional metadata (BPM, key, tempo, etc.)
);

-- prompts table
-- Tracks unique prompts for analytics and prompt history
CREATE TABLE IF NOT EXISTS prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,   -- Auto-incrementing ID
    text TEXT NOT NULL UNIQUE,              -- Unique prompt text
    used_count INTEGER DEFAULT 1,           -- Number of times this prompt was used
    first_used_at TIMESTAMP NOT NULL,       -- First time this prompt was used
    last_used_at TIMESTAMP NOT NULL         -- Most recent use of this prompt
);

-- Performance indices
CREATE INDEX IF NOT EXISTS idx_generations_status ON generations(status);
CREATE INDEX IF NOT EXISTS idx_generations_created_at ON generations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_prompts_text ON prompts(text);
CREATE INDEX IF NOT EXISTS idx_generations_model_name ON generations(model_name);
CREATE INDEX IF NOT EXISTS idx_generations_completed_at ON generations(completed_at DESC);
"""

# Status constants for generations.status field
class GenerationStatus:
    """Valid values for the generations.status field."""

    PENDING = "pending"         # Job created, waiting to be processed
    PROCESSING = "processing"   # Job is currently being generated
    COMPLETED = "completed"     # Job finished successfully
    FAILED = "failed"          # Job failed with error


def get_schema_version() -> str:
    """
    Return the current schema version.

    This should match the latest Alembic migration version.
    """
    return "1.0.0"


def validate_status(status: str) -> bool:
    """
    Validate that a status value is one of the allowed values.

    Args:
        status: The status string to validate

    Returns:
        True if valid, False otherwise
    """
    valid_statuses = {
        GenerationStatus.PENDING,
        GenerationStatus.PROCESSING,
        GenerationStatus.COMPLETED,
        GenerationStatus.FAILED,
    }
    return status in valid_statuses
