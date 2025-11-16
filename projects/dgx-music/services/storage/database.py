"""
Database connection management and CRUD operations for DGX Music.

This module provides:
- Database initialization and connection management
- CRUD operations for Generation and Prompt models
- Transaction handling
- Query utilities

Usage:
    from services.storage.database import init_db, get_session, create_generation

    # Initialize database
    init_db()

    # Use context manager for automatic session cleanup
    with get_session() as session:
        gen = create_generation(
            session=session,
            prompt="hip hop beat",
            model_name="musicgen-small",
            duration_seconds=16.0,
            sample_rate=32000,
            channels=2,
            file_path="outputs/gen_123.wav"
        )
"""

import os
from contextlib import contextmanager
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Generator, Dict, Any

from sqlalchemy import create_engine, desc
from sqlalchemy.orm import sessionmaker, Session
from sqlalchemy.exc import IntegrityError, SQLAlchemyError

from .models import Base, Generation, Prompt
from .schema import GenerationStatus


# Database configuration
DEFAULT_DATABASE_URL = "sqlite:///data/generations.db"
DATABASE_URL = os.getenv("DATABASE_URL", DEFAULT_DATABASE_URL)

# Create engine (will be initialized by init_db)
engine = None
SessionLocal = None


def get_database_path() -> Path:
    """
    Get the path to the SQLite database file.

    Returns:
        Path object for the database file
    """
    # Extract path from DATABASE_URL (assumes sqlite:/// format)
    db_path = DATABASE_URL.replace("sqlite:///", "")
    return Path(db_path)


def init_db(database_url: Optional[str] = None) -> None:
    """
    Initialize the database engine and create all tables.

    This should be called once at application startup.

    Args:
        database_url: Optional database URL override
    """
    global engine, SessionLocal

    url = database_url or DATABASE_URL

    # Ensure data directory exists
    db_path = get_database_path()
    db_path.parent.mkdir(parents=True, exist_ok=True)

    # Create engine
    engine = create_engine(
        url,
        echo=False,  # Set to True for SQL debugging
        connect_args={"check_same_thread": False},  # Needed for SQLite
    )

    # Create session factory
    SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

    # Create all tables
    Base.metadata.create_all(bind=engine)


def get_engine():
    """
    Get the database engine.

    Returns:
        SQLAlchemy engine instance
    """
    if engine is None:
        raise RuntimeError("Database not initialized. Call init_db() first.")
    return engine


@contextmanager
def get_session() -> Generator[Session, None, None]:
    """
    Context manager for database sessions.

    Provides automatic session cleanup and transaction handling.

    Usage:
        with get_session() as session:
            gen = create_generation(session, ...)
            session.commit()

    Yields:
        SQLAlchemy session
    """
    if SessionLocal is None:
        raise RuntimeError("Database not initialized. Call init_db() first.")

    session = SessionLocal()
    try:
        yield session
        session.commit()
    except Exception:
        session.rollback()
        raise
    finally:
        session.close()


# ========== Generation CRUD Operations ==========


def create_generation(
    session: Session,
    prompt: str,
    model_name: str,
    duration_seconds: float,
    sample_rate: int,
    channels: int,
    file_path: str,
    model_version: Optional[str] = None,
    metadata: Optional[Dict[str, Any]] = None,
) -> Generation:
    """
    Create a new generation record.

    Args:
        session: Database session
        prompt: User's text prompt
        model_name: AI model name
        duration_seconds: Target audio duration
        sample_rate: Audio sample rate (Hz)
        channels: Number of audio channels
        file_path: Path to output WAV file
        model_version: Optional model version
        metadata: Optional metadata dictionary

    Returns:
        Created Generation object
    """
    generation = Generation(
        prompt=prompt,
        model_name=model_name,
        model_version=model_version,
        duration_seconds=duration_seconds,
        sample_rate=sample_rate,
        channels=channels,
        file_path=file_path,
        status=GenerationStatus.PENDING,
    )

    if metadata:
        generation.set_metadata(metadata)

    session.add(generation)
    session.flush()  # Get the ID without committing

    # Track prompt usage
    track_prompt_usage(session, prompt)

    return generation


def get_generation(session: Session, generation_id: str) -> Optional[Generation]:
    """
    Get a generation by ID.

    Args:
        session: Database session
        generation_id: Generation UUID

    Returns:
        Generation object or None if not found
    """
    return session.query(Generation).filter(Generation.id == generation_id).first()


def get_all_generations(
    session: Session,
    limit: int = 100,
    offset: int = 0,
    status: Optional[str] = None,
) -> List[Generation]:
    """
    Get all generations with optional filtering.

    Args:
        session: Database session
        limit: Maximum number of results
        offset: Number of results to skip
        status: Optional status filter

    Returns:
        List of Generation objects
    """
    query = session.query(Generation)

    if status:
        query = query.filter(Generation.status == status)

    query = query.order_by(desc(Generation.created_at))
    query = query.limit(limit).offset(offset)

    return query.all()


def update_generation_status(
    session: Session,
    generation_id: str,
    status: str,
    error_message: Optional[str] = None,
) -> Optional[Generation]:
    """
    Update the status of a generation.

    Args:
        session: Database session
        generation_id: Generation UUID
        status: New status
        error_message: Optional error message if status is failed

    Returns:
        Updated Generation object or None if not found
    """
    generation = get_generation(session, generation_id)
    if not generation:
        return None

    generation.status = status

    if status == GenerationStatus.FAILED and error_message:
        generation.mark_failed(error_message)

    session.flush()
    return generation


def complete_generation(
    session: Session,
    generation_id: str,
    generation_time: float,
    file_size_bytes: int,
    metadata: Optional[Dict[str, Any]] = None,
) -> Optional[Generation]:
    """
    Mark a generation as completed and update metadata.

    Args:
        session: Database session
        generation_id: Generation UUID
        generation_time: Time taken to generate (seconds)
        file_size_bytes: Size of generated file
        metadata: Optional metadata to add/update

    Returns:
        Updated Generation object or None if not found
    """
    generation = get_generation(session, generation_id)
    if not generation:
        return None

    generation.mark_completed(generation_time)
    generation.file_size_bytes = file_size_bytes

    if metadata:
        existing = generation.get_metadata()
        existing.update(metadata)
        generation.set_metadata(existing)

    session.flush()
    return generation


def delete_generation(session: Session, generation_id: str) -> bool:
    """
    Delete a generation record.

    Args:
        session: Database session
        generation_id: Generation UUID

    Returns:
        True if deleted, False if not found
    """
    generation = get_generation(session, generation_id)
    if not generation:
        return False

    session.delete(generation)
    session.flush()
    return True


def get_generations_by_status(
    session: Session, status: str, limit: int = 100
) -> List[Generation]:
    """
    Get all generations with a specific status.

    Args:
        session: Database session
        status: Status to filter by
        limit: Maximum number of results

    Returns:
        List of Generation objects
    """
    return (
        session.query(Generation)
        .filter(Generation.status == status)
        .order_by(desc(Generation.created_at))
        .limit(limit)
        .all()
    )


def get_pending_generations(session: Session, limit: int = 100) -> List[Generation]:
    """
    Get all pending generations.

    Args:
        session: Database session
        limit: Maximum number of results

    Returns:
        List of pending Generation objects
    """
    return get_generations_by_status(session, GenerationStatus.PENDING, limit)


def count_generations(session: Session, status: Optional[str] = None) -> int:
    """
    Count generations with optional status filter.

    Args:
        session: Database session
        status: Optional status filter

    Returns:
        Count of generations
    """
    query = session.query(Generation)
    if status:
        query = query.filter(Generation.status == status)
    return query.count()


# ========== Prompt CRUD Operations ==========


def track_prompt_usage(session: Session, prompt_text: str) -> Prompt:
    """
    Track prompt usage, creating new record or incrementing existing.

    Args:
        session: Database session
        prompt_text: The prompt text

    Returns:
        Prompt object
    """
    # Try to find existing prompt
    prompt = session.query(Prompt).filter(Prompt.text == prompt_text).first()

    if prompt:
        # Update existing
        prompt.increment_usage()
    else:
        # Create new
        prompt = Prompt(text=prompt_text)
        session.add(prompt)

    session.flush()
    return prompt


def get_prompt(session: Session, prompt_id: int) -> Optional[Prompt]:
    """
    Get a prompt by ID.

    Args:
        session: Database session
        prompt_id: Prompt ID

    Returns:
        Prompt object or None if not found
    """
    return session.query(Prompt).filter(Prompt.id == prompt_id).first()


def get_prompt_by_text(session: Session, text: str) -> Optional[Prompt]:
    """
    Get a prompt by text.

    Args:
        session: Database session
        text: Prompt text

    Returns:
        Prompt object or None if not found
    """
    return session.query(Prompt).filter(Prompt.text == text).first()


def get_all_prompts(
    session: Session, limit: int = 100, offset: int = 0
) -> List[Prompt]:
    """
    Get all prompts ordered by usage count.

    Args:
        session: Database session
        limit: Maximum number of results
        offset: Number of results to skip

    Returns:
        List of Prompt objects
    """
    return (
        session.query(Prompt)
        .order_by(desc(Prompt.used_count))
        .limit(limit)
        .offset(offset)
        .all()
    )


def get_most_used_prompts(session: Session, limit: int = 10) -> List[Prompt]:
    """
    Get the most frequently used prompts.

    Args:
        session: Database session
        limit: Number of prompts to return

    Returns:
        List of Prompt objects ordered by usage count
    """
    return (
        session.query(Prompt)
        .order_by(desc(Prompt.used_count))
        .limit(limit)
        .all()
    )


# ========== Utility Functions ==========


def reset_database() -> None:
    """
    Reset the database by dropping and recreating all tables.

    WARNING: This will delete all data!
    """
    if engine is None:
        raise RuntimeError("Database not initialized. Call init_db() first.")

    Base.metadata.drop_all(bind=engine)
    Base.metadata.create_all(bind=engine)


def get_database_stats(session: Session) -> Dict[str, Any]:
    """
    Get database statistics.

    Args:
        session: Database session

    Returns:
        Dictionary of statistics
    """
    return {
        "total_generations": count_generations(session),
        "pending_generations": count_generations(session, GenerationStatus.PENDING),
        "processing_generations": count_generations(session, GenerationStatus.PROCESSING),
        "completed_generations": count_generations(session, GenerationStatus.COMPLETED),
        "failed_generations": count_generations(session, GenerationStatus.FAILED),
        "total_prompts": session.query(Prompt).count(),
    }
