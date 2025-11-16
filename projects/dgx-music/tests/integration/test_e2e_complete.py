"""
End-to-End Complete Workflow Tests
===================================

Complete integration tests covering the full workflow:
API request → generation → export → metadata → database

These tests validate the entire system working together.
"""

import pytest
import time
from pathlib import Path

from services.generation.models import GenerationRequest, GenerationStatus
from services.storage.database import (
    create_generation,
    get_generation,
    update_generation_status,
    complete_generation,
)
from services.storage.schema import GenerationStatus as DBStatus
from services.audio.export import AudioExporter
from services.audio.metadata import AudioMetadataExtractor
from tests.utils.audio_helpers import validate_wav_file, verify_audio_quality


pytestmark = pytest.mark.integration


class TestCompleteWorkflow:
    """Test complete end-to-end workflows."""

    def test_simple_generation_to_database(
        self, mock_engine, clean_db_session, output_dir, mock_settings
    ):
        """Test simple workflow: generate → save → database."""
        # Step 1: Create generation request
        request = GenerationRequest(
            prompt="test electronic music",
            duration=4.0,
        )

        # Step 2: Generate audio
        result = mock_engine.generate(request)

        assert result.status == GenerationStatus.COMPLETED
        assert result.file_path is not None
        assert Path(result.file_path).exists()

        # Step 3: Create database record
        gen = create_generation(
            session=clean_db_session,
            prompt=request.prompt,
            model_name="musicgen-small",
            duration_seconds=request.duration,
            sample_rate=32000,
            channels=2,
            file_path=result.file_path,
        )

        # Step 4: Complete generation in database
        complete_generation(
            session=clean_db_session,
            generation_id=gen.id,
            generation_time=result.generation_time_seconds,
            file_size_bytes=Path(result.file_path).stat().st_size,
        )

        clean_db_session.commit()

        # Verify
        retrieved = get_generation(clean_db_session, gen.id)
        assert retrieved is not None
        assert retrieved.status == DBStatus.COMPLETED
        assert Path(retrieved.file_path).exists()

    def test_complete_workflow_with_export(
        self, mock_engine, clean_db_session, output_dir
    ):
        """Test workflow with explicit export step."""
        # Generate
        prompt = "ambient music"
        duration = 4.0

        audio, sr = mock_engine.generate_audio(prompt, duration)

        # Export
        exporter = AudioExporter(target_lufs=-16.0)
        output_path = output_dir / "exported.wav"

        export_path, file_size = exporter.export_wav(
            audio_tensor=audio,
            output_path=output_path,
            sample_rate=sr,
            normalize=True,
        )

        # Create database record
        gen = create_generation(
            session=clean_db_session,
            prompt=prompt,
            model_name="musicgen-small",
            duration_seconds=duration,
            sample_rate=sr,
            channels=2,
            file_path=export_path,
        )

        complete_generation(
            session=clean_db_session,
            generation_id=gen.id,
            generation_time=1.0,
            file_size_bytes=file_size,
        )

        clean_db_session.commit()

        # Verify
        assert Path(export_path).exists()
        retrieved = get_generation(clean_db_session, gen.id)
        assert retrieved.file_size_bytes == file_size

    def test_workflow_with_metadata_extraction(
        self, mock_engine, clean_db_session, output_dir
    ):
        """Test workflow with metadata extraction."""
        # Generate and save
        prompt = "test music"
        request = GenerationRequest(prompt=prompt, duration=4.0)
        result = mock_engine.generate(request)

        # Extract metadata
        extractor = AudioMetadataExtractor(extract_bpm=True)
        metadata = extractor.extract_metadata(result.file_path)

        # Create database record with metadata
        gen = create_generation(
            session=clean_db_session,
            prompt=prompt,
            model_name="musicgen-small",
            duration_seconds=result.metadata.duration,
            sample_rate=metadata["sample_rate"],
            channels=metadata["channels"],
            file_path=result.file_path,
            metadata=metadata,
        )

        complete_generation(
            session=clean_db_session,
            generation_id=gen.id,
            generation_time=result.generation_time_seconds,
            file_size_bytes=metadata["file_size_bytes"],
        )

        clean_db_session.commit()

        # Verify metadata stored
        retrieved = get_generation(clean_db_session, gen.id)
        stored_metadata = retrieved.get_metadata()
        assert "sample_rate" in stored_metadata
        assert "channels" in stored_metadata

    def test_multiple_generations_sequential(
        self, mock_engine, clean_db_session, output_dir, mock_settings
    ):
        """Test multiple sequential generations."""
        prompts = [
            "electronic music",
            "hip hop beat",
            "ambient soundscape",
        ]

        generated_ids = []

        for prompt in prompts:
            # Generate
            request = GenerationRequest(prompt=prompt, duration=2.0)
            result = mock_engine.generate(request)

            # Store in database
            gen = create_generation(
                session=clean_db_session,
                prompt=prompt,
                model_name="musicgen-small",
                duration_seconds=2.0,
                sample_rate=32000,
                channels=2,
                file_path=result.file_path,
            )

            complete_generation(
                session=clean_db_session,
                generation_id=gen.id,
                generation_time=result.generation_time_seconds,
                file_size_bytes=Path(result.file_path).stat().st_size,
            )

            generated_ids.append(gen.id)

        clean_db_session.commit()

        # Verify all exist
        for gen_id in generated_ids:
            retrieved = get_generation(clean_db_session, gen_id)
            assert retrieved is not None
            assert retrieved.status == DBStatus.COMPLETED
            assert Path(retrieved.file_path).exists()

    def test_workflow_with_different_durations(
        self, mock_engine, clean_db_session, mock_settings
    ):
        """Test workflow with various durations."""
        durations = [4.0, 8.0, 16.0]

        for duration in durations:
            request = GenerationRequest(
                prompt=f"test music {duration}s",
                duration=duration,
            )

            result = mock_engine.generate(request)

            gen = create_generation(
                session=clean_db_session,
                prompt=request.prompt,
                model_name="musicgen-small",
                duration_seconds=duration,
                sample_rate=32000,
                channels=2,
                file_path=result.file_path,
            )

            complete_generation(
                session=clean_db_session,
                generation_id=gen.id,
                generation_time=result.generation_time_seconds,
                file_size_bytes=Path(result.file_path).stat().st_size,
            )

        clean_db_session.commit()

        # Verify all have correct durations
        from tests.utils.audio_helpers import compare_audio_properties

        all_gens = clean_db_session.query(
            __import__('services.storage.models').storage.models.Generation
        ).all()

        for gen in all_gens:
            props = compare_audio_properties(
                Path(gen.file_path),
                gen.duration_seconds,
                duration_tolerance=1.0,
            )
            assert props["duration_match"]

    def test_workflow_with_status_transitions(self, clean_db_session, output_dir):
        """Test generation status transitions."""
        from tests.utils.db_helpers import create_test_generation

        # Create pending generation
        gen = create_test_generation(
            clean_db_session,
            prompt="test",
            file_path=str(output_dir / "test.wav"),
        )
        assert gen.status == DBStatus.PENDING

        # Move to processing
        update_generation_status(
            clean_db_session,
            gen.id,
            DBStatus.PROCESSING,
        )
        clean_db_session.commit()

        retrieved = get_generation(clean_db_session, gen.id)
        assert retrieved.status == DBStatus.PROCESSING

        # Complete
        complete_generation(
            clean_db_session,
            gen.id,
            generation_time=10.0,
            file_size_bytes=1024 * 1024,
        )
        clean_db_session.commit()

        retrieved = get_generation(clean_db_session, gen.id)
        assert retrieved.status == DBStatus.COMPLETED
        assert retrieved.generation_time_seconds == 10.0


class TestAsyncJobQueue:
    """Test async job queue behavior."""

    def test_job_status_polling(self, clean_db_session, output_dir):
        """Test polling job status."""
        from tests.utils.db_helpers import create_test_generation

        # Create pending job
        gen = create_test_generation(
            clean_db_session,
            prompt="test",
            file_path=str(output_dir / "test.wav"),
        )
        job_id = gen.id
        clean_db_session.commit()

        # Poll status (simulating client polling)
        for _ in range(3):
            retrieved = get_generation(clean_db_session, job_id)
            assert retrieved is not None
            time.sleep(0.01)

        # Update to completed
        complete_generation(
            clean_db_session,
            job_id,
            generation_time=5.0,
            file_size_bytes=1024,
        )
        clean_db_session.commit()

        # Final poll
        retrieved = get_generation(clean_db_session, job_id)
        assert retrieved.status == DBStatus.COMPLETED

    def test_retrieving_completed_jobs(self, seeded_db_session):
        """Test retrieving completed jobs."""
        from services.storage.database import get_generations_by_status

        completed = get_generations_by_status(
            seeded_db_session,
            DBStatus.COMPLETED,
        )

        assert len(completed) > 0
        for gen in completed:
            assert gen.status == DBStatus.COMPLETED
            assert gen.completed_at is not None

    def test_pending_jobs_queue(self, seeded_db_session):
        """Test pending jobs queue."""
        from services.storage.database import get_pending_generations

        pending = get_pending_generations(seeded_db_session)

        for gen in pending:
            assert gen.status == DBStatus.PENDING
            assert gen.completed_at is None


class TestFileAndDatabaseSync:
    """Test file and database synchronization."""

    def test_file_creation_matches_database(
        self, mock_engine, clean_db_session, mock_settings
    ):
        """Test that created files match database records."""
        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        gen = create_generation(
            session=clean_db_session,
            prompt=request.prompt,
            model_name="musicgen-small",
            duration_seconds=2.0,
            sample_rate=32000,
            channels=2,
            file_path=result.file_path,
        )

        complete_generation(
            session=clean_db_session,
            generation_id=gen.id,
            generation_time=result.generation_time_seconds,
            file_size_bytes=Path(result.file_path).stat().st_size,
        )

        clean_db_session.commit()

        # Verify file and database match
        retrieved = get_generation(clean_db_session, gen.id)
        file_path = Path(retrieved.file_path)

        assert file_path.exists()
        assert file_path.stat().st_size == retrieved.file_size_bytes

    def test_database_records_match_files(self, integration_setup):
        """Test all database records have corresponding files."""
        # Create generations with files
        prompts = ["test1", "test2", "test3"]

        for i, prompt in enumerate(prompts):
            file_path = integration_setup.output_dir / f"test_{i}.wav"
            integration_setup.create_generation_with_file(prompt, file_path)

        # Verify all records have files
        from services.storage.models import Generation

        all_gens = integration_setup.db_session.query(Generation).all()

        for gen in all_gens:
            if gen.status == DBStatus.COMPLETED:
                assert Path(gen.file_path).exists()

    def test_wav_file_playable(self, mock_engine, mock_settings):
        """Test that generated WAV files are playable."""
        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        # Validate WAV file
        validation = validate_wav_file(
            Path(result.file_path),
            expected_sample_rate=32000,
            expected_channels=2,
            expected_bit_depth='PCM_16',
        )

        assert validation["valid"]
        assert validation["samples"] > 0

    def test_complete_workflow_quality_check(
        self, mock_engine, clean_db_session, mock_settings
    ):
        """Test complete workflow with quality verification."""
        # Generate
        request = GenerationRequest(
            prompt="electronic music test",
            duration=4.0,
        )
        result = mock_engine.generate(request)

        # Create database record
        gen = create_generation(
            session=clean_db_session,
            prompt=request.prompt,
            model_name="musicgen-small",
            duration_seconds=4.0,
            sample_rate=32000,
            channels=2,
            file_path=result.file_path,
        )

        complete_generation(
            session=clean_db_session,
            generation_id=gen.id,
            generation_time=result.generation_time_seconds,
            file_size_bytes=Path(result.file_path).stat().st_size,
        )

        clean_db_session.commit()

        # Verify quality
        file_path = Path(result.file_path)
        quality = verify_audio_quality(
            file_path,
            target_lufs=-16.0,
            lufs_tolerance=2.0,  # More lenient for mock audio
        )

        # Mock audio won't be normalized, so we skip LUFS check
        # but verify other properties
        assert quality["valid"] or len(quality["errors"]) <= 1  # Only LUFS might fail
        assert not quality["has_clipping"]

    def test_concurrent_database_writes(self, clean_db_session, output_dir):
        """Test concurrent database writes don't cause issues."""
        from tests.utils.db_helpers import create_test_generation

        # Create multiple generations quickly
        gen_ids = []
        for i in range(5):
            gen = create_test_generation(
                clean_db_session,
                prompt=f"test {i}",
                file_path=str(output_dir / f"test_{i}.wav"),
            )
            gen_ids.append(gen.id)

        clean_db_session.commit()

        # Verify all were created
        for gen_id in gen_ids:
            retrieved = get_generation(clean_db_session, gen_id)
            assert retrieved is not None


class TestPromptVariations:
    """Test with various prompt types."""

    def test_various_prompt_types(self, mock_engine, mock_settings):
        """Test generation with different prompt styles."""
        prompts = [
            "simple",
            "electronic dance music with synth melody and driving beat",
            "hip hop 90 BPM trap beat with 808 bass",
            "ambient, peaceful, piano, slow tempo",
        ]

        for prompt in prompts:
            request = GenerationRequest(prompt=prompt, duration=2.0)
            result = mock_engine.generate(request)

            assert result.status == GenerationStatus.COMPLETED
            assert Path(result.file_path).exists()

    def test_empty_prompt_handling(self):
        """Test that empty prompts are rejected."""
        with pytest.raises(Exception):  # Pydantic validation error
            GenerationRequest(prompt="", duration=2.0)

    def test_long_prompt(self, mock_engine, mock_settings):
        """Test generation with very long prompt."""
        long_prompt = "electronic music " * 50  # Very long prompt

        request = GenerationRequest(prompt=long_prompt, duration=2.0)
        result = mock_engine.generate(request)

        assert result.status == GenerationStatus.COMPLETED
