"""
Database Testing Helpers
=========================

Utilities for database testing including seeding, cleanup, and validation.
"""

import logging
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Optional, Dict, Any
from uuid import uuid4

from sqlalchemy.orm import Session

from services.storage.database import (
    init_db,
    get_session,
    create_generation,
    get_generation,
    track_prompt_usage,
)
from services.storage.models import Generation, Prompt
from services.storage.schema import GenerationStatus


logger = logging.getLogger(__name__)


def create_test_database(db_path: Optional[Path] = None) -> str:
    """
    Create a temporary test database.

    Args:
        db_path: Optional path for database. If None, uses temp directory.

    Returns:
        Database URL
    """
    if db_path is None:
        import tempfile
        temp_dir = Path(tempfile.mkdtemp())
        db_path = temp_dir / "test.db"

    db_url = f"sqlite:///{db_path}"
    init_db(db_url)
    logger.info(f"Created test database: {db_path}")

    return db_url


def seed_test_generations(
    session: Session,
    count: int = 10,
    status_distribution: Optional[Dict[str, int]] = None,
) -> List[Generation]:
    """
    Seed database with test generation records.

    Args:
        session: Database session
        count: Number of generations to create
        status_distribution: Optional status distribution
                           e.g., {"completed": 7, "pending": 2, "failed": 1}

    Returns:
        List of created Generation objects
    """
    if status_distribution is None:
        status_distribution = {
            GenerationStatus.COMPLETED: int(count * 0.7),
            GenerationStatus.PENDING: int(count * 0.2),
            GenerationStatus.FAILED: int(count * 0.1),
        }

    generations = []
    test_prompts = [
        "hip hop beat 90 BPM",
        "ambient music with piano",
        "electronic dance music",
        "jazz with saxophone",
        "rock guitar riff",
        "trap beat with 808",
        "lo-fi chill music",
        "dubstep drop 140 BPM",
        "classical piano piece",
        "synthwave 80s style",
    ]

    gen_index = 0
    for status, status_count in status_distribution.items():
        for i in range(status_count):
            prompt = test_prompts[gen_index % len(test_prompts)]

            gen = create_generation(
                session=session,
                prompt=prompt,
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path=f"outputs/test_{uuid4().hex[:8]}.wav",
                metadata={"test": True, "index": gen_index},
            )

            # Set status
            gen.status = status

            if status == GenerationStatus.COMPLETED:
                gen.mark_completed(generation_time=20.0 + i)
                gen.file_size_bytes = 1024 * 1024 * (2 + i)  # ~2MB each
            elif status == GenerationStatus.FAILED:
                gen.mark_failed(f"Test error {i}")

            generations.append(gen)
            gen_index += 1

    session.commit()
    logger.info(f"Seeded {len(generations)} test generations")

    return generations


def seed_test_prompts(session: Session, count: int = 5) -> List[Prompt]:
    """
    Seed database with test prompt records.

    Args:
        session: Database session
        count: Number of unique prompts to create

    Returns:
        List of created Prompt objects
    """
    test_prompts = [
        "hip hop beat 90 BPM",
        "ambient music with piano",
        "electronic dance music",
        "jazz with saxophone",
        "rock guitar riff",
    ]

    prompts = []
    for i in range(count):
        prompt_text = test_prompts[i % len(test_prompts)]

        # Track usage multiple times for some prompts
        for _ in range(i + 1):
            track_prompt_usage(session, prompt_text)

        prompt = session.query(Prompt).filter(Prompt.text == prompt_text).first()
        prompts.append(prompt)

    session.commit()
    logger.info(f"Seeded {len(prompts)} test prompts")

    return prompts


def cleanup_test_files(generations: List[Generation]) -> int:
    """
    Clean up test audio files.

    Args:
        generations: List of Generation objects

    Returns:
        Number of files deleted
    """
    deleted = 0
    for gen in generations:
        if gen.file_path:
            file_path = Path(gen.file_path)
            if file_path.exists():
                file_path.unlink()
                deleted += 1

    logger.info(f"Cleaned up {deleted} test files")
    return deleted


def verify_database_consistency(session: Session) -> Dict[str, Any]:
    """
    Verify database consistency.

    Checks:
    - All generations have valid status
    - Completed generations have file paths
    - Failed generations have error messages
    - Timestamps are valid
    - Foreign key relationships are intact

    Args:
        session: Database session

    Returns:
        Dictionary with consistency check results
    """
    results = {
        "valid": True,
        "errors": [],
        "warnings": [],
    }

    # Check all generations
    generations = session.query(Generation).all()

    for gen in generations:
        # Check status is valid
        valid_statuses = [
            GenerationStatus.PENDING,
            GenerationStatus.PROCESSING,
            GenerationStatus.COMPLETED,
            GenerationStatus.FAILED,
        ]
        if gen.status not in valid_statuses:
            results["valid"] = False
            results["errors"].append(f"Invalid status for generation {gen.id}: {gen.status}")

        # Check completed generations have required fields
        if gen.status == GenerationStatus.COMPLETED:
            if not gen.file_path:
                results["valid"] = False
                results["errors"].append(f"Completed generation {gen.id} missing file_path")

            if gen.generation_time_seconds is None:
                results["warnings"].append(
                    f"Completed generation {gen.id} missing generation_time"
                )

            if gen.completed_at is None:
                results["warnings"].append(
                    f"Completed generation {gen.id} missing completed_at"
                )

        # Check failed generations have error messages
        if gen.status == GenerationStatus.FAILED:
            if not gen.error_message:
                results["warnings"].append(
                    f"Failed generation {gen.id} missing error_message"
                )

        # Check timestamps
        if gen.created_at is None:
            results["valid"] = False
            results["errors"].append(f"Generation {gen.id} missing created_at")

        if gen.completed_at and gen.created_at:
            if gen.completed_at < gen.created_at:
                results["valid"] = False
                results["errors"].append(
                    f"Generation {gen.id} has completed_at before created_at"
                )

    results["total_generations"] = len(generations)
    results["error_count"] = len(results["errors"])
    results["warning_count"] = len(results["warnings"])

    return results


def get_orphaned_files(
    session: Session,
    outputs_dir: Path,
) -> List[Path]:
    """
    Find audio files without database records.

    Args:
        session: Database session
        outputs_dir: Directory containing output files

    Returns:
        List of orphaned file paths
    """
    # Get all file paths from database
    db_files = set()
    generations = session.query(Generation).all()
    for gen in generations:
        if gen.file_path:
            db_files.add(Path(gen.file_path).name)

    # Get all files in outputs directory
    if not outputs_dir.exists():
        return []

    disk_files = set(f.name for f in outputs_dir.glob("*.wav"))

    # Find orphans
    orphaned = disk_files - db_files
    orphaned_paths = [outputs_dir / f for f in orphaned]

    logger.info(f"Found {len(orphaned_paths)} orphaned files")
    return orphaned_paths


def get_orphaned_records(
    session: Session,
    outputs_dir: Path,
) -> List[Generation]:
    """
    Find database records without corresponding files.

    Args:
        session: Database session
        outputs_dir: Directory containing output files

    Returns:
        List of Generation objects with missing files
    """
    orphaned = []
    generations = session.query(Generation).filter(
        Generation.status == GenerationStatus.COMPLETED
    ).all()

    for gen in generations:
        if gen.file_path:
            file_path = Path(gen.file_path)
            if not file_path.exists():
                orphaned.append(gen)

    logger.info(f"Found {len(orphaned)} orphaned records")
    return orphaned


def create_generation_with_file(
    session: Session,
    prompt: str,
    file_path: Path,
    status: str = GenerationStatus.COMPLETED,
) -> Generation:
    """
    Create a generation record and its corresponding file.

    Args:
        session: Database session
        prompt: Generation prompt
        file_path: Path for output file
        status: Generation status

    Returns:
        Created Generation object
    """
    from tests.utils.audio_helpers import generate_test_wav

    # Create file
    generate_test_wav(file_path, duration=2.0)

    # Create database record
    gen = create_generation(
        session=session,
        prompt=prompt,
        model_name="musicgen-small",
        duration_seconds=2.0,
        sample_rate=32000,
        channels=2,
        file_path=str(file_path),
    )

    gen.status = status
    if status == GenerationStatus.COMPLETED:
        gen.mark_completed(generation_time=5.0)
        gen.file_size_bytes = file_path.stat().st_size

    session.commit()

    return gen


def assert_generation_valid(gen: Generation):
    """
    Assert that a generation object is valid.

    Args:
        gen: Generation object to validate

    Raises:
        AssertionError: If validation fails
    """
    assert gen is not None, "Generation is None"
    assert gen.id is not None, "Generation missing ID"
    assert gen.prompt, "Generation missing prompt"
    assert gen.model_name, "Generation missing model_name"
    assert gen.duration_seconds > 0, "Generation has invalid duration"
    assert gen.sample_rate > 0, "Generation has invalid sample_rate"
    assert gen.channels > 0, "Generation has invalid channels"
    assert gen.status, "Generation missing status"
    assert gen.created_at, "Generation missing created_at"


def count_by_status(session: Session) -> Dict[str, int]:
    """
    Count generations by status.

    Args:
        session: Database session

    Returns:
        Dictionary mapping status to count
    """
    counts = {}
    for status in [
        GenerationStatus.PENDING,
        GenerationStatus.PROCESSING,
        GenerationStatus.COMPLETED,
        GenerationStatus.FAILED,
    ]:
        count = session.query(Generation).filter(Generation.status == status).count()
        counts[status] = count

    return counts


def create_test_generation(
    session: Session,
    **overrides
) -> Generation:
    """
    Create a test generation with default values.

    Args:
        session: Database session
        **overrides: Override default values

    Returns:
        Created Generation object
    """
    defaults = {
        "prompt": "test prompt",
        "model_name": "musicgen-small",
        "duration_seconds": 16.0,
        "sample_rate": 32000,
        "channels": 2,
        "file_path": f"outputs/test_{uuid4().hex[:8]}.wav",
    }
    defaults.update(overrides)

    return create_generation(session=session, **defaults)
