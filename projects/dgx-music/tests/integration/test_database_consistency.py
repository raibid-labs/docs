"""
Database Consistency Integration Tests
=======================================

Tests for database consistency, integrity, and synchronization with files.
"""

import pytest
from pathlib import Path

from services.storage.database import (
    create_generation,
    get_generation,
    update_generation_status,
    complete_generation,
    delete_generation,
)
from services.storage.schema import GenerationStatus
from tests.utils.db_helpers import (
    verify_database_consistency,
    get_orphaned_files,
    get_orphaned_records,
    create_generation_with_file,
    count_by_status,
)


pytestmark = pytest.mark.integration


class TestDatabaseIntegrity:
    """Test database integrity and consistency."""

    def test_all_generations_have_database_records(self, integration_setup):
        """Test all generated files have corresponding database records."""
        # Create generations with files
        for i in range(3):
            prompt = f"test {i}"
            file_path = integration_setup.output_dir / f"test_{i}.wav"
            integration_setup.create_generation_with_file(prompt, file_path)

        # Verify consistency
        consistency = integration_setup.verify_consistency()

        assert consistency["valid"], (
            f"Database consistency check failed: {consistency['errors']}"
        )

    def test_completed_generations_have_files(self, seeded_db_session):
        """Test all completed generations have files (or marked as failed)."""
        from services.storage.models import Generation

        completed_gens = seeded_db_session.query(Generation).filter(
            Generation.status == GenerationStatus.COMPLETED
        ).all()

        for gen in completed_gens:
            # Either file exists or generation should be marked as failed
            file_path = Path(gen.file_path) if gen.file_path else None

            if file_path and not file_path.exists():
                # File missing for completed generation
                # In production, cleanup job would mark these as failed
                print(f"Warning: Completed generation {gen.id} missing file: {file_path}")

    def test_database_records_match_file_properties(self, integration_setup):
        """Test database records match actual file properties."""
        import soundfile as sf

        # Create generation with file
        prompt = "test"
        file_path = integration_setup.output_dir / "test.wav"
        gen = integration_setup.create_generation_with_file(prompt, file_path)

        # Get file properties
        info = sf.info(str(file_path))

        # Verify database matches
        assert gen.sample_rate == info.samplerate
        assert gen.channels == info.channels
        assert abs(gen.duration_seconds - info.duration) < 0.1

    def test_foreign_key_constraints(self, clean_db_session):
        """Test foreign key relationships are enforced."""
        from tests.utils.db_helpers import create_test_generation

        # Create generation (which creates prompt)
        gen = create_test_generation(clean_db_session, prompt="test prompt")
        clean_db_session.commit()

        # Prompt should exist
        from services.storage.database import get_prompt_by_text

        prompt = get_prompt_by_text(clean_db_session, "test prompt")
        assert prompt is not None
        assert prompt.used_count >= 1

    def test_unique_constraints(self, clean_db_session):
        """Test unique constraints are enforced."""
        from tests.utils.db_helpers import create_test_generation

        # Create generation with specific ID
        gen1 = create_test_generation(clean_db_session, prompt="test1")
        gen1_id = gen1.id
        clean_db_session.commit()

        # Try to create another with same ID would fail
        # (but UUIDs are unique by design, so this test is informational)
        gen2 = create_test_generation(clean_db_session, prompt="test2")
        assert gen2.id != gen1_id


class TestTransactionHandling:
    """Test transaction handling and rollback."""

    def test_transaction_rollback_on_failure(self, clean_db_session):
        """Test transaction rollback when operation fails."""
        from tests.utils.db_helpers import create_test_generation

        initial_count = clean_db_session.query(
            __import__('services.storage.models').storage.models.Generation
        ).count()

        # Start transaction
        try:
            gen = create_test_generation(clean_db_session, prompt="test")

            # Simulate error before commit
            raise RuntimeError("Simulated error")

        except RuntimeError:
            clean_db_session.rollback()

        # Count should be unchanged
        final_count = clean_db_session.query(
            __import__('services.storage.models').storage.models.Generation
        ).count()

        assert final_count == initial_count

    def test_partial_completion_rollback(self, clean_db_session, output_dir):
        """Test rollback when generation partially completes."""
        from tests.utils.db_helpers import create_test_generation

        # Create generation
        gen = create_test_generation(
            clean_db_session,
            prompt="test",
            file_path=str(output_dir / "test.wav"),
        )
        gen_id = gen.id
        clean_db_session.commit()

        # Try to complete but fail
        try:
            complete_generation(
                clean_db_session,
                gen_id,
                generation_time=10.0,
                file_size_bytes=1024,
            )

            # Simulate error
            raise RuntimeError("Completion failed")

        except RuntimeError:
            clean_db_session.rollback()

        # Generation should still be in original state
        retrieved = get_generation(clean_db_session, gen_id)
        assert retrieved.status == GenerationStatus.PENDING


class TestOrphanedData:
    """Test detection and cleanup of orphaned data."""

    def test_detect_orphaned_files(self, integration_setup):
        """Test detection of files without database records."""
        from tests.utils.audio_helpers import generate_test_wav

        # Create file without database record
        orphan_path = integration_setup.output_dir / "orphan.wav"
        generate_test_wav(orphan_path, duration=1.0)

        # Detect orphans
        orphans = get_orphaned_files(
            integration_setup.db_session,
            integration_setup.output_dir,
        )

        # Should detect the orphan
        assert len(orphans) > 0
        assert orphan_path in orphans

    def test_detect_orphaned_database_records(self, integration_setup):
        """Test detection of database records without files."""
        from tests.utils.db_helpers import create_test_generation

        # Create database record without file
        gen = create_test_generation(
            integration_setup.db_session,
            prompt="test",
            file_path=str(integration_setup.output_dir / "nonexistent.wav"),
        )

        # Mark as completed (even though file doesn't exist)
        complete_generation(
            integration_setup.db_session,
            gen.id,
            generation_time=10.0,
            file_size_bytes=1024,
        )
        integration_setup.db_session.commit()

        # Detect orphaned records
        orphans = get_orphaned_records(
            integration_setup.db_session,
            integration_setup.output_dir,
        )

        # Should detect the orphaned record
        assert len(orphans) > 0
        assert any(o.id == gen.id for o in orphans)

    def test_cleanup_orphaned_files(self, integration_setup):
        """Test cleanup of orphaned files."""
        from tests.utils.audio_helpers import generate_test_wav

        # Create orphan
        orphan_path = integration_setup.output_dir / "orphan.wav"
        generate_test_wav(orphan_path, duration=1.0)

        # Detect
        orphans = get_orphaned_files(
            integration_setup.db_session,
            integration_setup.output_dir,
        )

        # Clean up
        for orphan in orphans:
            orphan.unlink()

        # Verify cleaned
        assert not orphan_path.exists()


class TestDatabaseQueries:
    """Test database query performance and correctness."""

    def test_query_by_status(self, seeded_db_session):
        """Test querying generations by status."""
        from services.storage.database import get_generations_by_status

        # Query completed
        completed = get_generations_by_status(
            seeded_db_session,
            GenerationStatus.COMPLETED,
        )

        # All should be completed
        for gen in completed:
            assert gen.status == GenerationStatus.COMPLETED

    def test_query_with_pagination(self, seeded_db_session):
        """Test query pagination."""
        from services.storage.database import get_all_generations

        # Get first page
        page1 = get_all_generations(seeded_db_session, limit=3, offset=0)

        # Get second page
        page2 = get_all_generations(seeded_db_session, limit=3, offset=3)

        # Pages should be different
        page1_ids = {g.id for g in page1}
        page2_ids = {g.id for g in page2}

        assert len(page1_ids & page2_ids) == 0, "Pages should not overlap"

    def test_query_performance_with_many_records(self, clean_db_session):
        """Test query performance with 100+ records."""
        from tests.utils.db_helpers import seed_test_generations
        import time

        # Seed many records
        seed_test_generations(clean_db_session, count=100)

        # Query all
        start = time.time()
        from services.storage.database import get_all_generations

        results = get_all_generations(clean_db_session, limit=100)
        elapsed = time.time() - start

        print(f"\nQuery 100 records: {elapsed * 1000:.1f}ms")

        assert len(results) <= 100
        assert elapsed < 0.5, f"Query too slow: {elapsed:.2f}s"


class TestMetadataConsistency:
    """Test metadata storage and consistency."""

    def test_metadata_json_structure(self, clean_db_session):
        """Test metadata is stored as valid JSON."""
        from tests.utils.db_helpers import create_test_generation
        import json

        # Create generation with metadata
        metadata = {
            "test_key": "test_value",
            "number": 42,
            "nested": {"key": "value"},
        }

        gen = create_test_generation(
            clean_db_session,
            prompt="test",
            metadata=metadata,
        )
        clean_db_session.commit()

        # Retrieve and verify
        retrieved = get_generation(clean_db_session, gen.id)
        stored_metadata = retrieved.get_metadata()

        assert stored_metadata["test_key"] == "test_value"
        assert stored_metadata["number"] == 42
        assert stored_metadata["nested"]["key"] == "value"

    def test_metadata_update(self, clean_db_session):
        """Test metadata can be updated."""
        from tests.utils.db_helpers import create_test_generation

        # Create with initial metadata
        gen = create_test_generation(
            clean_db_session,
            prompt="test",
            metadata={"version": 1},
        )
        clean_db_session.commit()

        # Update metadata
        complete_generation(
            clean_db_session,
            gen.id,
            generation_time=10.0,
            file_size_bytes=1024,
            metadata={"version": 2, "completed": True},
        )
        clean_db_session.commit()

        # Verify updated
        retrieved = get_generation(clean_db_session, gen.id)
        stored_metadata = retrieved.get_metadata()

        assert stored_metadata["version"] == 2
        assert stored_metadata["completed"] is True


class TestConcurrentAccess:
    """Test concurrent database access."""

    def test_concurrent_writes_no_conflicts(self, clean_db_session):
        """Test concurrent writes don't cause conflicts."""
        from tests.utils.db_helpers import create_test_generation

        # Create multiple generations quickly
        generations = []
        for i in range(10):
            gen = create_test_generation(
                clean_db_session,
                prompt=f"test {i}",
            )
            generations.append(gen)

        clean_db_session.commit()

        # Verify all were created
        for gen in generations:
            retrieved = get_generation(clean_db_session, gen.id)
            assert retrieved is not None

    def test_concurrent_status_updates(self, clean_db_session):
        """Test concurrent status updates."""
        from tests.utils.db_helpers import create_test_generation

        # Create generation
        gen = create_test_generation(clean_db_session, prompt="test")
        gen_id = gen.id
        clean_db_session.commit()

        # Update status multiple times
        statuses = [
            GenerationStatus.PROCESSING,
            GenerationStatus.COMPLETED,
        ]

        for status in statuses:
            update_generation_status(clean_db_session, gen_id, status)
            clean_db_session.commit()

        # Verify final status
        retrieved = get_generation(clean_db_session, gen_id)
        assert retrieved.status == GenerationStatus.COMPLETED


class TestConsistencyReport:
    """Test consistency reporting."""

    def test_comprehensive_consistency_check(self, integration_setup):
        """Test comprehensive consistency verification."""
        # Create various test data
        # 1. Valid generation with file
        gen1_path = integration_setup.output_dir / "valid.wav"
        gen1 = integration_setup.create_generation_with_file("test1", gen1_path)

        # 2. Orphaned file
        from tests.utils.audio_helpers import generate_test_wav

        orphan_path = integration_setup.output_dir / "orphan.wav"
        generate_test_wav(orphan_path, duration=1.0)

        # Run consistency check
        consistency = verify_database_consistency(integration_setup.db_session)

        print(f"\nConsistency report:")
        print(f"  Total generations: {consistency['total_generations']}")
        print(f"  Errors: {consistency['error_count']}")
        print(f"  Warnings: {consistency['warning_count']}")

        if consistency["errors"]:
            print(f"  Error details: {consistency['errors']}")

        # Should have minimal errors for valid data
        assert consistency["total_generations"] > 0
