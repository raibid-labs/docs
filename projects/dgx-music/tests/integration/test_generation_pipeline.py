"""
Integration Tests for Music Generation Pipeline
===============================================

Tests the complete generation pipeline with actual model (if GPU available).
These tests may be skipped if CUDA is not available.
"""

import pytest
import torch
from pathlib import Path
import time

from services.generation.engine import MusicGenerationEngine, get_engine
from services.generation.models import GenerationRequest, GenerationStatus, ModelName


# Skip tests if CUDA not available
pytestmark = pytest.mark.skipif(
    not torch.cuda.is_available(),
    reason="CUDA not available - integration tests require GPU"
)


@pytest.fixture(scope="module")
def engine():
    """Create engine instance for testing."""
    return MusicGenerationEngine(
        model_name="small",
        use_gpu=True,
        enable_caching=True,
    )


@pytest.fixture
def output_dir(tmp_path):
    """Create temporary output directory."""
    output_dir = tmp_path / "outputs"
    output_dir.mkdir(exist_ok=True)
    return output_dir


class TestGenerationPipeline:
    """Integration tests for complete generation pipeline."""

    def test_model_loads(self, engine):
        """Test model loads successfully."""
        engine.load_model()
        assert engine.model is not None
        assert engine.model_name == "small"

    def test_basic_generation(self, engine):
        """Test basic audio generation."""
        prompt = "electronic dance music with synth melody"
        duration = 8.0

        audio, sample_rate = engine.generate_audio(
            prompt=prompt,
            duration=duration,
        )

        assert audio is not None
        assert audio.ndim == 2  # (channels, samples)
        assert audio.shape[0] == 2  # Stereo
        assert sample_rate > 0

        # Check duration is approximately correct
        actual_duration = audio.shape[1] / sample_rate
        assert abs(actual_duration - duration) < 1.0  # Within 1 second

    def test_generation_performance(self, engine):
        """Test generation meets performance targets."""
        prompt = "trap beat with 808 bass"
        duration = 16.0

        start_time = time.time()
        audio, sample_rate = engine.generate_audio(prompt=prompt, duration=duration)
        generation_time = time.time() - start_time

        # Performance target: <30s for 16s audio (MVP acceptable)
        # Target: <60s (MVP blocker threshold)
        assert generation_time < 60.0, (
            f"Generation too slow: {generation_time:.1f}s > 60s - BLOCKER"
        )

        # Log performance
        rtf = generation_time / duration
        print(f"\nPerformance: {generation_time:.2f}s for {duration}s audio (RTF: {rtf:.2f}x)")

        if generation_time < 30.0:
            print("✅ EXCELLENT: Within 30s target")
        else:
            print("⚠️  ACCEPTABLE: Within 60s, but consider optimization")

    def test_different_prompts(self, engine):
        """Test generation with various prompts."""
        test_prompts = [
            "hip hop beat 90 BPM",
            "ambient music with piano",
            "dubstep drop 140 BPM",
            "jazz with saxophone",
            "rock guitar riff",
        ]

        for prompt in test_prompts:
            audio, sample_rate = engine.generate_audio(
                prompt=prompt,
                duration=4.0,  # Short duration for faster testing
            )

            assert audio is not None
            assert audio.shape[0] == 2  # Stereo
            print(f"✅ Generated: {prompt}")

    def test_different_durations(self, engine):
        """Test generation with different durations."""
        prompt = "electronic music"
        durations = [4.0, 8.0, 16.0, 30.0]

        for duration in durations:
            audio, sample_rate = engine.generate_audio(
                prompt=prompt,
                duration=duration,
            )

            actual_duration = audio.shape[1] / sample_rate
            assert abs(actual_duration - duration) < 1.0
            print(f"✅ Generated {duration}s audio (actual: {actual_duration:.1f}s)")

    def test_generation_parameters(self, engine):
        """Test different generation parameters."""
        prompt = "upbeat electronic music"

        # Test different temperatures
        for temp in [0.5, 1.0, 1.5]:
            audio, _ = engine.generate_audio(
                prompt=prompt,
                duration=4.0,
                temperature=temp,
            )
            assert audio is not None
            print(f"✅ Generated with temperature={temp}")

        # Test different top_k
        for top_k in [100, 250, 500]:
            audio, _ = engine.generate_audio(
                prompt=prompt,
                duration=4.0,
                top_k=top_k,
            )
            assert audio is not None
            print(f"✅ Generated with top_k={top_k}")

    def test_save_audio(self, engine, output_dir):
        """Test audio saving."""
        prompt = "test audio"
        audio, sample_rate = engine.generate_audio(prompt=prompt, duration=4.0)

        output_path = output_dir / "test_output.wav"
        metadata = engine.save_audio(
            audio=audio,
            sample_rate=sample_rate,
            output_path=output_path,
            normalize=True,
        )

        assert output_path.exists()
        assert metadata.duration > 0
        assert metadata.sample_rate == sample_rate
        assert metadata.channels == 2
        assert metadata.file_size_bytes > 0
        print(f"✅ Saved audio: {metadata.file_size_mb:.2f}MB")

    def test_complete_workflow(self, engine, output_dir):
        """Test complete generation workflow."""
        request = GenerationRequest(
            prompt="hip hop beat with 808 bass",
            duration=8.0,
            temperature=1.0,
            top_k=250,
        )

        # Mock settings for output path
        from services.generation import engine as engine_module
        original_settings = engine_module.settings

        class MockSettings:
            normalize_audio = True
            def get_output_path(self, job_id):
                return output_dir / f"{job_id}.wav"

        engine_module.settings = MockSettings()

        try:
            result = engine.generate(request)

            assert result.status == GenerationStatus.COMPLETED
            assert result.job_id is not None
            assert result.prompt == request.prompt
            assert result.metadata is not None
            assert result.generation_time_seconds > 0
            assert result.file_path is not None

            # Check file exists
            file_path = Path(result.file_path)
            assert file_path.exists()

            print(f"✅ Complete workflow: {result.generation_time_seconds:.2f}s")

        finally:
            engine_module.settings = original_settings

    def test_memory_usage(self, engine):
        """Test memory usage stays within budget."""
        prompt = "electronic music"

        # Reset peak memory stats
        torch.cuda.reset_peak_memory_stats()

        # Generate
        audio, _ = engine.generate_audio(prompt=prompt, duration=16.0)

        # Check peak memory
        peak_memory_mb = torch.cuda.max_memory_allocated() / (1024 * 1024)
        memory_budget_mb = 30 * 1024  # 30GB budget

        print(f"\nPeak GPU Memory: {peak_memory_mb:.2f}MB")

        assert peak_memory_mb < memory_budget_mb, (
            f"Memory usage {peak_memory_mb:.0f}MB exceeds budget {memory_budget_mb:.0f}MB"
        )

        if peak_memory_mb < 10 * 1024:  # <10GB
            print("✅ EXCELLENT: Memory usage well within budget")
        else:
            print("✅ ACCEPTABLE: Memory usage within budget")

    def test_benchmark(self, engine):
        """Test performance benchmark."""
        benchmark = engine.benchmark(duration=16.0)

        assert benchmark.model_name == "musicgen-small"
        assert benchmark.duration == 16.0
        assert benchmark.generation_time > 0
        assert benchmark.real_time_factor > 0
        assert benchmark.sample_rate > 0
        assert benchmark.cuda_available is True

        print(f"\nBenchmark Results:")
        print(f"  Duration: {benchmark.duration}s")
        print(f"  Generation Time: {benchmark.generation_time:.2f}s")
        print(f"  Real-Time Factor: {benchmark.real_time_factor:.2f}x")
        print(f"  Peak Memory: {benchmark.peak_memory_mb:.1f}MB")
        print(f"  Sample Rate: {benchmark.sample_rate}Hz")

        # Check performance targets
        if benchmark.generation_time < 30.0:
            print("✅ Performance: EXCELLENT (<30s target)")
        elif benchmark.generation_time < 60.0:
            print("✅ Performance: ACCEPTABLE (<60s)")
        else:
            print("⚠️  Performance: SLOW (>60s) - Consider alternatives")

    def test_error_handling(self, engine):
        """Test error handling for invalid inputs."""
        # Empty prompt should be caught by Pydantic
        with pytest.raises(Exception):
            request = GenerationRequest(prompt="", duration=16.0)

        # Invalid duration
        with pytest.raises(Exception):
            request = GenerationRequest(prompt="test", duration=100.0)


class TestEngineSingleton:
    """Test global engine instance."""

    def test_get_engine(self):
        """Test global engine getter."""
        engine1 = get_engine()
        engine2 = get_engine()

        # Should return same instance
        assert engine1 is engine2

    def test_engine_stats(self):
        """Test engine statistics."""
        engine = get_engine()
        stats = engine.get_stats()

        assert "model_name" in stats
        assert "device" in stats
        assert "model_loaded" in stats
        assert "generation_count" in stats


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
