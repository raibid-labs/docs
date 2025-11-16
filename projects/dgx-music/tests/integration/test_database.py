"""
Integration tests for database operations.

These tests require a real database and test CRUD operations,
transactions, and session management.
"""

import os
import tempfile
from pathlib import Path
import pytest

from services.storage import (
    init_db,
    get_session,
    create_generation,
    get_generation,
    get_all_generations,
    update_generation_status,
    complete_generation,
    delete_generation,
    get_pending_generations,
    count_generations,
    track_prompt_usage,
    get_prompt_by_text,
    get_most_used_prompts,
    get_database_stats,
    GenerationStatus,
)


@pytest.fixture
def test_db():
    """Create a temporary test database."""
    # Create temporary database file
    temp_fd, temp_path = tempfile.mkstemp(suffix=".db")
    os.close(temp_fd)

    # Initialize database with temp path
    init_db(f"sqlite:///{temp_path}")

    yield temp_path

    # Cleanup
    try:
        os.unlink(temp_path)
    except OSError:
        pass


class TestDatabaseInitialization:
    """Tests for database initialization."""

    def test_init_db(self, test_db):
        """Test that init_db creates the database file."""
        assert Path(test_db).exists()

    def test_get_session(self, test_db):
        """Test getting a database session."""
        with get_session() as session:
            assert session is not None


class TestGenerationCRUD:
    """Tests for Generation CRUD operations."""

    def test_create_generation(self, test_db):
        """Test creating a generation."""
        with get_session() as session:
            gen = create_generation(
                session=session,
                prompt="test hip hop beat",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="outputs/test.wav",
            )

            assert gen.id is not None
            assert gen.prompt == "test hip hop beat"
            assert gen.model_name == "musicgen-small"
            assert gen.status == GenerationStatus.PENDING

    def test_create_generation_with_metadata(self, test_db):
        """Test creating a generation with metadata."""
        with get_session() as session:
            metadata = {"bpm": 140, "key": "C"}
            gen = create_generation(
                session=session,
                prompt="test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
                metadata=metadata,
            )

            assert gen.get_metadata() == metadata

    def test_get_generation(self, test_db):
        """Test retrieving a generation by ID."""
        with get_session() as session:
            # Create
            gen = create_generation(
                session=session,
                prompt="test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
            )
            gen_id = gen.id
            session.commit()

        # Retrieve in new session
        with get_session() as session:
            retrieved = get_generation(session, gen_id)
            assert retrieved is not None
            assert retrieved.id == gen_id
            assert retrieved.prompt == "test"

    def test_get_generation_not_found(self, test_db):
        """Test retrieving a non-existent generation."""
        with get_session() as session:
            result = get_generation(session, "nonexistent-id")
            assert result is None

    def test_get_all_generations(self, test_db):
        """Test retrieving all generations."""
        with get_session() as session:
            # Create multiple generations
            for i in range(5):
                create_generation(
                    session=session,
                    prompt=f"test {i}",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test_{i}.wav",
                )
            session.commit()

        with get_session() as session:
            all_gens = get_all_generations(session)
            assert len(all_gens) == 5

    def test_get_all_generations_with_limit(self, test_db):
        """Test retrieving generations with limit."""
        with get_session() as session:
            for i in range(10):
                create_generation(
                    session=session,
                    prompt=f"test {i}",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test_{i}.wav",
                )
            session.commit()

        with get_session() as session:
            limited = get_all_generations(session, limit=5)
            assert len(limited) == 5

    def test_get_all_generations_with_status_filter(self, test_db):
        """Test retrieving generations filtered by status."""
        with get_session() as session:
            # Create some pending
            for i in range(3):
                create_generation(
                    session=session,
                    prompt=f"test {i}",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test_{i}.wav",
                )

            # Create some with different status
            gen = create_generation(
                session=session,
                prompt="completed test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="completed.wav",
            )
            gen.mark_completed(18.0)

            session.commit()

        with get_session() as session:
            pending = get_all_generations(session, status=GenerationStatus.PENDING)
            assert len(pending) == 3

            completed = get_all_generations(session, status=GenerationStatus.COMPLETED)
            assert len(completed) == 1

    def test_update_generation_status(self, test_db):
        """Test updating generation status."""
        with get_session() as session:
            gen = create_generation(
                session=session,
                prompt="test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
            )
            gen_id = gen.id
            session.commit()

        with get_session() as session:
            updated = update_generation_status(
                session, gen_id, GenerationStatus.PROCESSING
            )
            assert updated is not None
            assert updated.status == GenerationStatus.PROCESSING
            session.commit()

        with get_session() as session:
            gen = get_generation(session, gen_id)
            assert gen.status == GenerationStatus.PROCESSING

    def test_complete_generation(self, test_db):
        """Test marking a generation as completed."""
        with get_session() as session:
            gen = create_generation(
                session=session,
                prompt="test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
            )
            gen_id = gen.id
            session.commit()

        with get_session() as session:
            updated = complete_generation(
                session,
                gen_id,
                generation_time=18.5,
                file_size_bytes=5000000,
                metadata={"bpm": 120},
            )
            assert updated is not None
            assert updated.status == GenerationStatus.COMPLETED
            assert updated.generation_time_seconds == 18.5
            assert updated.file_size_bytes == 5000000
            assert updated.get_metadata()["bpm"] == 120
            session.commit()

    def test_delete_generation(self, test_db):
        """Test deleting a generation."""
        with get_session() as session:
            gen = create_generation(
                session=session,
                prompt="test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
            )
            gen_id = gen.id
            session.commit()

        with get_session() as session:
            result = delete_generation(session, gen_id)
            assert result is True
            session.commit()

        with get_session() as session:
            gen = get_generation(session, gen_id)
            assert gen is None

    def test_delete_generation_not_found(self, test_db):
        """Test deleting a non-existent generation."""
        with get_session() as session:
            result = delete_generation(session, "nonexistent-id")
            assert result is False

    def test_get_pending_generations(self, test_db):
        """Test retrieving pending generations."""
        with get_session() as session:
            # Create pending
            for i in range(3):
                create_generation(
                    session=session,
                    prompt=f"test {i}",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test_{i}.wav",
                )

            # Create completed
            gen = create_generation(
                session=session,
                prompt="completed",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="completed.wav",
            )
            gen.mark_completed(18.0)
            session.commit()

        with get_session() as session:
            pending = get_pending_generations(session)
            assert len(pending) == 3
            assert all(g.is_pending for g in pending)

    def test_count_generations(self, test_db):
        """Test counting generations."""
        with get_session() as session:
            for i in range(5):
                create_generation(
                    session=session,
                    prompt=f"test {i}",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test_{i}.wav",
                )
            session.commit()

        with get_session() as session:
            count = count_generations(session)
            assert count == 5


class TestPromptTracking:
    """Tests for prompt tracking functionality."""

    def test_track_prompt_usage_new(self, test_db):
        """Test tracking a new prompt."""
        with get_session() as session:
            prompt = track_prompt_usage(session, "test prompt")
            assert prompt.text == "test prompt"
            assert prompt.used_count == 1
            session.commit()

    def test_track_prompt_usage_existing(self, test_db):
        """Test tracking an existing prompt."""
        with get_session() as session:
            # First use
            prompt1 = track_prompt_usage(session, "test prompt")
            session.commit()

        with get_session() as session:
            # Second use
            prompt2 = track_prompt_usage(session, "test prompt")
            assert prompt2.text == "test prompt"
            assert prompt2.used_count == 2
            session.commit()

    def test_prompt_tracking_with_generation(self, test_db):
        """Test that creating a generation tracks the prompt."""
        with get_session() as session:
            create_generation(
                session=session,
                prompt="hip hop beat",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
            )
            session.commit()

        with get_session() as session:
            prompt = get_prompt_by_text(session, "hip hop beat")
            assert prompt is not None
            assert prompt.used_count == 1

    def test_get_most_used_prompts(self, test_db):
        """Test retrieving most used prompts."""
        with get_session() as session:
            # Create generations with different prompts
            for i in range(5):
                create_generation(
                    session=session,
                    prompt="prompt 1",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test_{i}.wav",
                )

            for i in range(3):
                create_generation(
                    session=session,
                    prompt="prompt 2",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"test2_{i}.wav",
                )

            create_generation(
                session=session,
                prompt="prompt 3",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test3.wav",
            )
            session.commit()

        with get_session() as session:
            most_used = get_most_used_prompts(session, limit=3)
            assert len(most_used) == 3
            assert most_used[0].text == "prompt 1"
            assert most_used[0].used_count == 5
            assert most_used[1].text == "prompt 2"
            assert most_used[1].used_count == 3


class TestDatabaseStats:
    """Tests for database statistics."""

    def test_get_database_stats(self, test_db):
        """Test retrieving database statistics."""
        with get_session() as session:
            # Create some generations with different statuses
            for i in range(3):
                create_generation(
                    session=session,
                    prompt=f"pending {i}",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path=f"pending_{i}.wav",
                )

            gen = create_generation(
                session=session,
                prompt="completed",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="completed.wav",
            )
            gen.mark_completed(18.0)

            gen = create_generation(
                session=session,
                prompt="failed",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="failed.wav",
            )
            gen.mark_failed("Test error")

            session.commit()

        with get_session() as session:
            stats = get_database_stats(session)
            assert stats["total_generations"] == 5
            assert stats["pending_generations"] == 3
            assert stats["completed_generations"] == 1
            assert stats["failed_generations"] == 1
            assert stats["total_prompts"] == 5


class TestTransactionHandling:
    """Tests for transaction and session management."""

    def test_session_commit_on_success(self, test_db):
        """Test that sessions commit on success."""
        with get_session() as session:
            gen = create_generation(
                session=session,
                prompt="test",
                model_name="musicgen-small",
                duration_seconds=16.0,
                sample_rate=32000,
                channels=2,
                file_path="test.wav",
            )
            gen_id = gen.id

        # Verify in new session
        with get_session() as session:
            gen = get_generation(session, gen_id)
            assert gen is not None

    def test_session_rollback_on_error(self, test_db):
        """Test that sessions rollback on error."""
        gen_id = None

        try:
            with get_session() as session:
                gen = create_generation(
                    session=session,
                    prompt="test",
                    model_name="musicgen-small",
                    duration_seconds=16.0,
                    sample_rate=32000,
                    channels=2,
                    file_path="test.wav",
                )
                gen_id = gen.id
                # Force an error
                raise Exception("Test error")
        except Exception:
            pass

        # Verify rollback
        with get_session() as session:
            gen = get_generation(session, gen_id)
            # Should be None because transaction was rolled back
            assert gen is None
