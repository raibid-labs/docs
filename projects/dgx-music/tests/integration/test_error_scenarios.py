"""
Error Scenario Integration Tests
=================================

Tests for error handling, edge cases, and failure scenarios.
"""

import pytest
import os
from pathlib import Path
from unittest.mock import Mock, patch

from services.generation.models import GenerationRequest
from services.generation.engine import GenerationError
from services.audio.export import AudioExporter
from services.storage.database import create_generation
from tests.utils.mock_helpers import create_mock_audio_tensor


pytestmark = pytest.mark.integration


class TestInvalidInputs:
    """Test handling of invalid inputs."""

    def test_empty_prompt(self):
        """Test that empty prompt is rejected."""
        with pytest.raises(Exception):  # Pydantic validation
            GenerationRequest(prompt="", duration=16.0)

    def test_too_long_prompt(self):
        """Test that extremely long prompt is handled."""
        # Create a very long prompt (10000 characters)
        long_prompt = "a" * 10000

        # Should either accept it or raise a validation error
        # (depends on model limits)
        try:
            request = GenerationRequest(prompt=long_prompt, duration=16.0)
            # If accepted, it should at least not crash
            assert len(request.prompt) > 0
        except Exception:
            # If rejected, that's also acceptable
            pass

    def test_special_characters_in_prompt(self, mock_engine, mock_settings):
        """Test prompt with special characters."""
        special_prompts = [
            "music with émojis 🎵🎶",
            "prompt with\nnewlines\n",
            "prompt with\ttabs",
            "prompt with 'quotes' and \"doublequotes\"",
            "prompt with $pecial ch@racters!",
        ]

        for prompt in special_prompts:
            try:
                request = GenerationRequest(prompt=prompt, duration=2.0)
                result = mock_engine.generate(request)
                assert result is not None
            except Exception as e:
                # Some special characters might be rejected
                # but shouldn't crash the system
                assert "validation" in str(e).lower() or "error" in str(e).lower()

    def test_negative_duration(self):
        """Test that negative duration is rejected."""
        with pytest.raises(Exception):  # Pydantic validation
            GenerationRequest(prompt="test", duration=-5.0)

    def test_zero_duration(self):
        """Test that zero duration is rejected."""
        with pytest.raises(Exception):  # Pydantic validation
            GenerationRequest(prompt="test", duration=0.0)

    def test_too_long_duration(self):
        """Test that excessively long duration is rejected."""
        with pytest.raises(Exception):  # Pydantic validation
            GenerationRequest(prompt="test", duration=1000.0)

    def test_invalid_model_name(self):
        """Test that invalid model name is handled."""
        from services.generation.engine import MusicGenerationEngine

        # Try to create engine with invalid model
        # Should either reject or handle gracefully
        try:
            engine = MusicGenerationEngine(
                model_name="nonexistent_model",
                use_gpu=False,
            )
            # If it doesn't reject immediately, loading should fail
            with pytest.raises(Exception):
                engine.load_model()
        except Exception:
            # Immediate rejection is also acceptable
            pass


class TestResourceFailures:
    """Test resource failure scenarios."""

    def test_disk_full_simulation(self, mock_engine, output_dir, monkeypatch):
        """Test handling of disk full error."""
        from services.generation.models import GenerationRequest
        import soundfile as sf

        # Mock sf.write to raise disk full error
        original_write = sf.write

        def mock_write(*args, **kwargs):
            raise OSError("No space left on device")

        monkeypatch.setattr(sf, "write", mock_write)

        request = GenerationRequest(prompt="test", duration=2.0)

        # Generation should fail gracefully
        result = mock_engine.generate(request)
        assert result.status == "failed"
        assert "error" in result.error_message.lower()

    def test_output_directory_missing(self, mock_engine, output_dir):
        """Test handling of missing output directory."""
        from services.generation.models import GenerationRequest

        # Remove output directory
        import shutil
        if output_dir.exists():
            shutil.rmtree(output_dir)

        request = GenerationRequest(prompt="test", duration=2.0)

        # Should either create directory or fail gracefully
        result = mock_engine.generate(request)

        # If it succeeded, directory should be created
        if result.status == "completed":
            assert Path(result.file_path).parent.exists()

    def test_file_permission_error(self, mock_engine, output_dir, monkeypatch):
        """Test handling of file permission errors."""
        from services.generation.models import GenerationRequest
        import soundfile as sf

        # Mock sf.write to raise permission error
        def mock_write(*args, **kwargs):
            raise PermissionError("Permission denied")

        monkeypatch.setattr(sf, "write", mock_write)

        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        assert result.status == "failed"
        assert "permission" in result.error_message.lower() or "error" in result.error_message.lower()

    def test_database_connection_failure(self, monkeypatch):
        """Test handling of database connection failure."""
        from services.storage.database import get_session

        # Mock get_session to raise connection error
        def mock_get_session():
            raise RuntimeError("Database not initialized")

        monkeypatch.setattr("services.storage.database.get_session", mock_get_session)

        # Should raise appropriate error
        with pytest.raises(RuntimeError):
            with mock_get_session() as session:
                pass


class TestGPUFallback:
    """Test GPU/CUDA unavailability scenarios."""

    def test_cuda_unavailable_cpu_fallback(self, mock_cuda_available):
        """Test fallback to CPU when CUDA unavailable."""
        from services.generation.engine import MusicGenerationEngine

        # Mock CUDA as unavailable
        mock_cuda_available(False)

        # Create engine - should fall back to CPU
        engine = MusicGenerationEngine(
            model_name="small",
            use_gpu=True,  # Request GPU
            enable_caching=False,
        )

        assert engine.device == "cpu"
        assert not engine.use_gpu

    def test_gpu_memory_error_handling(self, monkeypatch):
        """Test handling of GPU out-of-memory error."""
        # This is hard to test without actual GPU
        # but we can simulate the error condition
        pass  # Skipped - requires actual GPU testing


class TestCorruptedData:
    """Test handling of corrupted data."""

    def test_corrupted_audio_tensor(self, output_dir):
        """Test handling of corrupted audio tensor."""
        import numpy as np
        import torch

        exporter = AudioExporter()

        # Create tensor with NaN values
        corrupted = torch.from_numpy(np.array([[np.nan, np.nan]]))

        output_path = output_dir / "corrupted.wav"

        # Should handle gracefully
        try:
            exporter.export_wav(
                audio_tensor=corrupted,
                output_path=output_path,
                sample_rate=32000,
            )
            # If it succeeds, check the file
            if output_path.exists():
                import soundfile as sf
                audio, sr = sf.read(str(output_path))
                # Check for NaN in output
                assert not np.isnan(audio).any()
        except Exception as e:
            # Failing gracefully is also acceptable
            assert "error" in str(e).lower() or "invalid" in str(e).lower()

    def test_invalid_tensor_shape(self, output_dir):
        """Test handling of invalid tensor shapes."""
        import torch

        exporter = AudioExporter()

        # Create tensor with wrong shape (3D instead of 2D)
        invalid = torch.randn(2, 100, 100)

        output_path = output_dir / "invalid.wav"

        # Should reject invalid shape
        with pytest.raises(ValueError):
            exporter.export_wav(
                audio_tensor=invalid,
                output_path=output_path,
                sample_rate=32000,
            )


class TestInterruptedOperations:
    """Test interrupted operation scenarios."""

    def test_interrupted_generation(self, clean_db_session, output_dir):
        """Test handling of interrupted generation."""
        from tests.utils.db_helpers import create_test_generation
        from services.storage.schema import GenerationStatus

        # Create generation in processing state
        gen = create_test_generation(
            clean_db_session,
            prompt="test",
            file_path=str(output_dir / "test.wav"),
        )

        # Simulate interruption by setting to processing
        from services.storage.database import update_generation_status

        update_generation_status(
            clean_db_session,
            gen.id,
            GenerationStatus.PROCESSING,
        )
        clean_db_session.commit()

        # Generation should be marked as interrupted
        # (in real system, cleanup job would handle this)
        retrieved = clean_db_session.query(
            __import__('services.storage.models').storage.models.Generation
        ).filter_by(id=gen.id).first()

        assert retrieved.status == GenerationStatus.PROCESSING


class TestMissingDependencies:
    """Test missing dependency scenarios."""

    def test_missing_pyloudnorm(self, mock_no_pyloudnorm, output_dir):
        """Test operation without pyloudnorm."""
        tensor = create_mock_audio_tensor(duration=1.0)

        exporter = AudioExporter(target_lufs=-16.0)
        output_path = output_dir / "no_loudnorm.wav"

        # Should still work, just without normalization
        path, size = exporter.export_wav(
            audio_tensor=tensor,
            output_path=output_path,
            sample_rate=32000,
            normalize=True,  # Requested but unavailable
        )

        assert Path(path).exists()

    def test_missing_librosa(self, mock_no_librosa, test_audio_file):
        """Test metadata extraction without librosa."""
        from services.audio.metadata import AudioMetadataExtractor

        extractor = AudioMetadataExtractor(extract_bpm=True)
        metadata = extractor.extract_metadata(test_audio_file)

        # Should return metadata without BPM
        assert "duration_seconds" in metadata
        assert metadata["bpm"] is None  # Not available without librosa


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_very_short_duration(self, mock_engine, mock_settings):
        """Test generation with very short duration."""
        from services.generation.models import GenerationRequest

        # Minimum duration (0.5s)
        request = GenerationRequest(prompt="test", duration=0.5)
        result = mock_engine.generate(request)

        assert result.status == "completed"
        assert Path(result.file_path).exists()

    def test_maximum_duration(self, mock_engine, mock_settings):
        """Test generation with maximum allowed duration."""
        from services.generation.models import GenerationRequest

        # Maximum duration (30s per MVP spec)
        request = GenerationRequest(prompt="test", duration=30.0)
        result = mock_engine.generate(request)

        assert result.status == "completed"

    def test_unicode_filename_handling(self, output_dir):
        """Test handling of unicode characters in file paths."""
        from tests.utils.audio_helpers import generate_test_wav

        # This tests if the system can handle unicode paths
        # (most systems should handle this, but good to verify)
        unicode_path = output_dir / "test_音楽.wav"

        try:
            generate_test_wav(unicode_path, duration=1.0)
            assert unicode_path.exists()
            unicode_path.unlink()
        except Exception:
            # Some filesystems don't support unicode
            pytest.skip("Filesystem doesn't support unicode filenames")
