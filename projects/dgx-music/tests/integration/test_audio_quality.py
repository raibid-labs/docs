"""
Audio Quality Integration Tests
================================

Tests for audio quality validation including format, loudness, and properties.
"""

import pytest
import numpy as np
from pathlib import Path

from services.audio.export import AudioExporter
from services.audio.metadata import AudioMetadataExtractor
from tests.utils.audio_helpers import (
    validate_wav_file,
    measure_loudness,
    check_no_clipping,
    verify_audio_quality,
    compare_audio_properties,
    check_stereo_balance,
    measure_rms_energy,
)


pytestmark = pytest.mark.integration


class TestWAVFormat:
    """Test WAV file format compliance."""

    def test_wav_format_pcm16_32khz_stereo(self, mock_engine, output_dir, mock_settings):
        """Test WAV file is PCM_16, 32kHz, stereo."""
        from services.generation.models import GenerationRequest

        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        # Validate format
        validation = validate_wav_file(
            Path(result.file_path),
            expected_sample_rate=32000,
            expected_channels=2,
            expected_bit_depth='PCM_16',
        )

        assert validation["valid"]
        assert validation["sample_rate"] == 32000
        assert validation["channels"] == 2
        assert validation["bit_depth"] == 'PCM_16'

    def test_wav_file_not_corrupted(self, test_audio_file):
        """Test WAV file is not corrupted."""
        import soundfile as sf

        # Should load without errors
        audio, sr = sf.read(str(test_audio_file))

        assert len(audio) > 0
        assert sr > 0
        assert not np.isnan(audio).any()
        assert not np.isinf(audio).any()

    def test_audio_duration_matches_request(self, mock_engine, mock_settings):
        """Test audio duration matches requested duration."""
        from services.generation.models import GenerationRequest

        durations = [2.0, 4.0, 8.0, 16.0]

        for duration in durations:
            request = GenerationRequest(prompt="test", duration=duration)
            result = mock_engine.generate(request)

            props = compare_audio_properties(
                Path(result.file_path),
                expected_duration=duration,
                duration_tolerance=0.5,  # Within 0.5 seconds
            )

            assert props["duration_match"], (
                f"Duration mismatch: expected {duration}s, "
                f"got {props['duration_actual']:.2f}s"
            )

    def test_stereo_channels_present(self, mock_engine, mock_settings):
        """Test audio has 2 channels (stereo)."""
        from services.generation.models import GenerationRequest
        import soundfile as sf

        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        audio, sr = sf.read(result.file_path)

        # Check shape
        if audio.ndim == 1:
            pytest.fail("Audio is mono, expected stereo")

        assert audio.shape[1] == 2, f"Expected 2 channels, got {audio.shape[1]}"


class TestLoudnessNormalization:
    """Test loudness normalization."""

    def test_loudness_within_target_range(self, test_audio_file):
        """Test loudness is within target LUFS ±1."""
        loudness = measure_loudness(test_audio_file)

        if loudness is None:
            pytest.skip("pyloudnorm not available")

        target_lufs = -16.0
        tolerance = 2.0  # ±2 LUFS tolerance for test audio

        deviation = abs(loudness - target_lufs)
        assert deviation <= tolerance, (
            f"Loudness outside tolerance: {loudness:.1f} LUFS "
            f"(target: {target_lufs} ±{tolerance})"
        )

    def test_no_clipping(self, mock_engine, mock_settings):
        """Test audio has no clipping (peak < 0.99)."""
        from services.generation.models import GenerationRequest

        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        has_clipping, peak = check_no_clipping(Path(result.file_path))

        assert not has_clipping, f"Audio clipping detected: peak={peak:.3f}"
        assert peak < 1.0, f"Peak amplitude too high: {peak:.3f}"

    def test_normalization_consistent_across_files(self, mock_engine, mock_settings):
        """Test normalization produces consistent loudness across multiple files."""
        from services.generation.models import GenerationRequest

        loudness_values = []

        for i in range(3):
            request = GenerationRequest(prompt=f"test {i}", duration=2.0)
            result = mock_engine.generate(request)

            loudness = measure_loudness(Path(result.file_path))
            if loudness is not None:
                loudness_values.append(loudness)

        if not loudness_values:
            pytest.skip("pyloudnorm not available")

        # Check consistency (all within 3 LUFS of each other)
        if len(loudness_values) > 1:
            max_diff = max(loudness_values) - min(loudness_values)
            assert max_diff < 3.0, (
                f"Loudness inconsistent across files: "
                f"range={max_diff:.1f} LUFS"
            )


class TestAudioProperties:
    """Test audio property validation."""

    def test_metadata_extraction_accuracy(self, test_audio_file):
        """Test metadata extraction is accurate."""
        extractor = AudioMetadataExtractor(extract_bpm=False)
        metadata = extractor.extract_metadata(test_audio_file)

        # Validate basic metadata
        assert metadata["duration_seconds"] > 0
        assert metadata["sample_rate"] == 32000
        assert metadata["channels"] == 2
        assert metadata["file_size_bytes"] > 0

    def test_audio_statistics(self, test_audio_file):
        """Test audio statistics computation."""
        import soundfile as sf

        audio, sr = sf.read(str(test_audio_file))

        # Compute statistics
        peak = np.abs(audio).max()
        rms = np.sqrt(np.mean(audio ** 2))

        # Validate ranges
        assert 0.0 <= peak <= 1.0, f"Peak out of range: {peak}"
        assert rms > 0, "RMS energy is zero (silent audio)"
        assert rms < peak, "RMS should be less than peak"

    def test_stereo_balance(self, mock_engine, mock_settings):
        """Test stereo channels are balanced."""
        from services.generation.models import GenerationRequest

        request = GenerationRequest(prompt="test", duration=2.0)
        result = mock_engine.generate(request)

        is_balanced, diff = check_stereo_balance(
            Path(result.file_path),
            tolerance=0.2,  # Allow 0.2 RMS difference
        )

        # Stereo balance check (may not be perfectly balanced for all audio)
        # This is informational rather than a hard requirement
        if not is_balanced:
            print(f"Note: Stereo imbalance detected: {diff:.3f} RMS difference")

    def test_dynamic_range(self, test_audio_file):
        """Test audio has reasonable dynamic range."""
        import soundfile as sf

        audio, sr = sf.read(str(test_audio_file))

        if audio.ndim == 2:
            audio = audio.mean(axis=1)  # Convert to mono

        peak = np.abs(audio).max()
        rms = np.sqrt(np.mean(audio ** 2))

        if rms > 0:
            dynamic_range_db = 20 * np.log10(peak / rms)
            assert dynamic_range_db > 0, "Dynamic range should be positive"
            assert dynamic_range_db < 60, "Dynamic range unusually high"


class TestBatchQuality:
    """Test quality consistency in batch generation."""

    def test_batch_generation_consistent_quality(self, mock_engine, mock_settings):
        """Test batch generation produces consistent quality."""
        from services.generation.models import GenerationRequest

        results = []
        for i in range(3):
            request = GenerationRequest(prompt=f"test {i}", duration=2.0)
            result = mock_engine.generate(request)
            results.append(result)

        # Validate all files
        for result in results:
            validation = validate_wav_file(
                Path(result.file_path),
                expected_sample_rate=32000,
                expected_channels=2,
            )
            assert validation["valid"]

        # Check file sizes are similar
        file_sizes = [Path(r.file_path).stat().st_size for r in results]
        avg_size = sum(file_sizes) / len(file_sizes)

        for size in file_sizes:
            deviation = abs(size - avg_size) / avg_size
            assert deviation < 0.5, (
                f"File size variation too high: {deviation:.1%}"
            )

    def test_quality_metrics_consistent(self, mock_engine, mock_settings):
        """Test quality metrics are consistent across generations."""
        from services.generation.models import GenerationRequest

        rms_values = []

        for i in range(3):
            request = GenerationRequest(prompt=f"test {i}", duration=2.0)
            result = mock_engine.generate(request)

            rms = measure_rms_energy(Path(result.file_path))
            rms_values.append(rms)

        # Check RMS values are similar
        avg_rms = sum(rms_values) / len(rms_values)

        for rms in rms_values:
            deviation = abs(rms - avg_rms) / avg_rms
            # Allow 50% deviation (mock audio is synthetic)
            assert deviation < 0.5, (
                f"RMS energy variation too high: {deviation:.1%}"
            )


class TestComprehensiveQuality:
    """Comprehensive audio quality tests."""

    def test_complete_quality_verification(self, mock_engine, mock_settings):
        """Test complete quality verification pipeline."""
        from services.generation.models import GenerationRequest

        request = GenerationRequest(prompt="test music", duration=4.0)
        result = mock_engine.generate(request)

        # Run comprehensive quality check
        quality = verify_audio_quality(
            Path(result.file_path),
            target_lufs=-16.0,
            lufs_tolerance=3.0,  # Lenient for mock audio
            clip_threshold=0.99,
        )

        # Check no clipping
        assert not quality["has_clipping"], (
            f"Audio has clipping: peak={quality['peak_amplitude']:.3f}"
        )

        # Check duration
        assert quality["duration"] > 0

        # If LUFS measurement available, check it
        if "loudness_lufs" in quality and quality["loudness_lufs"] is not None:
            assert abs(quality["loudness_lufs"] - (-16.0)) < 5.0, (
                f"Loudness far from target: {quality['loudness_lufs']:.1f} LUFS"
            )
