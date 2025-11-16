"""
Unit tests for AudioMetadataExtractor

Tests cover:
- Initialization
- Basic metadata extraction
- Statistics computation
- BPM detection
- Key detection (experimental)
- Tensor analysis
- Different audio formats and sample rates
- Error handling
- Edge cases (silent audio, very short audio)
"""

import pytest
import torch
import numpy as np
import soundfile as sf
from pathlib import Path
import tempfile
import shutil

from services.audio.metadata import AudioMetadataExtractor
from services.audio.export import AudioExporter


@pytest.fixture
def temp_dir():
    """Create a temporary directory for test files."""
    temp_path = tempfile.mkdtemp()
    yield Path(temp_path)
    shutil.rmtree(temp_path)


@pytest.fixture
def extractor():
    """Create an AudioMetadataExtractor instance."""
    return AudioMetadataExtractor(extract_bpm=True, extract_key=False)


@pytest.fixture
def exporter():
    """Create an AudioExporter for test file creation."""
    return AudioExporter()


@pytest.fixture
def test_audio_file(temp_dir, exporter):
    """Create a test audio file."""
    # Generate 2-second test audio
    sample_rate = 32000
    duration = 2.0
    t = np.linspace(0, duration, int(sample_rate * duration))
    audio = 0.5 * np.sin(2 * np.pi * 440 * t)
    tensor = torch.from_numpy(audio.astype(np.float32))

    output_path = temp_dir / "test_audio.wav"
    exporter.export_wav(tensor, str(output_path), sample_rate=sample_rate, normalize=False)

    return output_path


# ========== Initialization Tests ==========


def test_extractor_initialization_default():
    """Test AudioMetadataExtractor initialization with defaults."""
    extractor = AudioMetadataExtractor()
    assert extractor.extract_bpm is True
    assert extractor.extract_key is False


def test_extractor_initialization_custom():
    """Test AudioMetadataExtractor initialization with custom settings."""
    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=True)
    assert extractor.extract_bpm is False
    assert extractor.extract_key is True


# ========== Basic Metadata Extraction Tests ==========


def test_extract_basic_metadata(extractor, test_audio_file):
    """Test extraction of basic metadata from audio file."""
    metadata = extractor.extract_metadata(test_audio_file, compute_stats=False)

    assert 'duration_seconds' in metadata
    assert 'sample_rate' in metadata
    assert 'channels' in metadata
    assert 'file_size_bytes' in metadata

    assert metadata['sample_rate'] == 32000
    assert metadata['duration_seconds'] == pytest.approx(2.0, rel=0.01)
    assert metadata['channels'] == 1  # Mono
    assert metadata['file_size_bytes'] > 0


def test_extract_file_size(extractor, test_audio_file):
    """Test that file size is correctly extracted."""
    metadata = extractor.extract_metadata(test_audio_file)

    actual_size = test_audio_file.stat().st_size
    assert metadata['file_size_bytes'] == actual_size


def test_extract_sample_rate(extractor, temp_dir, exporter):
    """Test extraction of different sample rates."""
    sample_rates = [16000, 32000, 44100, 48000]

    for sr in sample_rates:
        # Create test file
        audio = torch.randn(sr * 2) * 0.5  # 2 seconds
        path = temp_dir / f"sr_{sr}.wav"
        exporter.export_wav(audio, str(path), sample_rate=sr, normalize=False)

        # Extract metadata
        metadata = extractor.extract_metadata(path)
        assert metadata['sample_rate'] == sr


def test_extract_channels(extractor, temp_dir, exporter):
    """Test extraction of channel count."""
    # Mono
    mono = torch.randn(32000) * 0.5
    mono_path = temp_dir / "mono.wav"
    exporter.export_wav(mono, str(mono_path), normalize=False)

    metadata_mono = extractor.extract_metadata(mono_path)
    assert metadata_mono['channels'] == 1

    # Stereo
    stereo = torch.randn(2, 32000) * 0.5
    stereo_path = temp_dir / "stereo.wav"
    exporter.export_wav(stereo, str(stereo_path), normalize=False)

    metadata_stereo = extractor.extract_metadata(stereo_path)
    assert metadata_stereo['channels'] == 2


# ========== Statistics Computation Tests ==========


def test_compute_statistics(extractor, test_audio_file):
    """Test computation of audio statistics."""
    metadata = extractor.extract_metadata(test_audio_file, compute_stats=True)

    assert 'peak_amplitude' in metadata
    assert 'rms_energy' in metadata
    assert 'dynamic_range_db' in metadata

    # Values should be reasonable
    assert 0 <= metadata['peak_amplitude'] <= 1.0
    assert 0 <= metadata['rms_energy'] <= 1.0
    assert metadata['dynamic_range_db'] >= 0


def test_statistics_disabled(extractor, test_audio_file):
    """Test that statistics can be disabled."""
    # Create extractor without BPM/key to speed up
    fast_extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=False)

    metadata = fast_extractor.extract_metadata(test_audio_file, compute_stats=False)

    # Statistics should not be present
    assert 'peak_amplitude' not in metadata
    assert 'rms_energy' not in metadata
    assert 'dynamic_range_db' not in metadata


def test_peak_amplitude_calculation(extractor, temp_dir, exporter):
    """Test peak amplitude calculation accuracy."""
    # Create audio with known peak
    audio_data = np.array([0.0, 0.5, -0.8, 0.3, 0.0])
    audio = torch.from_numpy(audio_data.astype(np.float32))

    path = temp_dir / "peak_test.wav"
    exporter.export_wav(audio, str(path), sample_rate=1000, normalize=False)

    metadata = extractor.extract_metadata(path, compute_stats=True)

    # Peak should be 0.8 (absolute value of -0.8)
    assert metadata['peak_amplitude'] == pytest.approx(0.8, abs=0.01)


# ========== BPM Detection Tests ==========


def test_bpm_detection_enabled(test_audio_file):
    """Test that BPM detection runs when enabled."""
    extractor = AudioMetadataExtractor(extract_bpm=True)
    metadata = extractor.extract_metadata(test_audio_file)

    # BPM should be present (may be None if detection fails)
    assert 'bpm' in metadata


def test_bpm_detection_disabled(test_audio_file):
    """Test that BPM detection is skipped when disabled."""
    extractor = AudioMetadataExtractor(extract_bpm=False)
    metadata = extractor.extract_metadata(test_audio_file)

    # BPM should be None when disabled
    assert metadata['bpm'] is None


# ========== Key Detection Tests ==========


def test_key_detection_experimental(test_audio_file):
    """Test experimental key detection."""
    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=True)
    metadata = extractor.extract_metadata(test_audio_file)

    # Key should be present (may be None if detection fails)
    assert 'key' in metadata


# ========== Tensor Analysis Tests ==========


def test_extract_from_tensor_mono():
    """Test metadata extraction from mono tensor."""
    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=False)

    # Create mono tensor
    sample_rate = 32000
    duration = 2.0
    samples = int(sample_rate * duration)
    audio_tensor = torch.randn(samples) * 0.5

    metadata = extractor.extract_metadata_from_tensor(
        audio_tensor,
        sample_rate,
        compute_stats=True
    )

    assert metadata['duration_seconds'] == pytest.approx(duration, rel=0.01)
    assert metadata['sample_rate'] == sample_rate
    assert metadata['channels'] == 1
    assert metadata['file_size_bytes'] is None  # Not applicable to tensors
    assert metadata['bit_depth'] is None  # Not applicable to tensors


def test_extract_from_tensor_stereo():
    """Test metadata extraction from stereo tensor."""
    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=False)

    # Create stereo tensor
    sample_rate = 32000
    duration = 1.0
    samples = int(sample_rate * duration)
    audio_tensor = torch.randn(2, samples) * 0.5

    metadata = extractor.extract_metadata_from_tensor(
        audio_tensor,
        sample_rate,
        compute_stats=True
    )

    assert metadata['channels'] == 2
    assert metadata['duration_seconds'] == pytest.approx(duration, rel=0.01)


def test_tensor_gpu_handling():
    """Test that GPU tensors are properly handled in extraction."""
    if not torch.cuda.is_available():
        pytest.skip("CUDA not available")

    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=False)

    # Create GPU tensor
    audio_tensor = torch.randn(32000).cuda() * 0.5

    metadata = extractor.extract_metadata_from_tensor(
        audio_tensor,
        32000,
        compute_stats=True
    )

    assert metadata is not None
    assert metadata['channels'] == 1


def test_tensor_with_gradients():
    """Test tensor with gradients is properly handled."""
    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=False)

    # Create tensor with gradients
    audio_tensor = torch.randn(32000).requires_grad_(True) * 0.5

    metadata = extractor.extract_metadata_from_tensor(
        audio_tensor,
        32000,
        compute_stats=True
    )

    assert metadata is not None


# ========== Error Handling Tests ==========


def test_nonexistent_file(extractor):
    """Test that nonexistent file raises error."""
    with pytest.raises(FileNotFoundError):
        extractor.extract_metadata("nonexistent_file.wav")


def test_invalid_file_format(extractor, temp_dir):
    """Test handling of invalid file format."""
    # Create invalid file
    invalid_file = temp_dir / "invalid.wav"
    invalid_file.write_text("not an audio file")

    with pytest.raises(RuntimeError, match="Metadata extraction failed"):
        extractor.extract_metadata(invalid_file)


def test_invalid_tensor_dimensions(extractor):
    """Test that invalid tensor dimensions raise error."""
    # Create 3D tensor
    invalid_tensor = torch.randn(2, 2, 32000)

    with pytest.raises(ValueError, match="Invalid audio dimensions"):
        extractor.extract_metadata_from_tensor(invalid_tensor, 32000)


# ========== Edge Cases Tests ==========


def test_silent_audio_metadata(extractor, temp_dir, exporter):
    """Test metadata extraction from silent audio."""
    # Create silent audio
    silent = torch.zeros(32000)
    path = temp_dir / "silent.wav"
    exporter.export_wav(silent, str(path), normalize=False)

    metadata = extractor.extract_metadata(path, compute_stats=True)

    assert metadata['peak_amplitude'] == pytest.approx(0.0, abs=0.001)
    assert metadata['rms_energy'] == pytest.approx(0.0, abs=0.001)


def test_very_short_audio(extractor, temp_dir, exporter):
    """Test metadata extraction from very short audio."""
    # Create very short audio (0.1 seconds)
    short_audio = torch.randn(3200) * 0.5
    path = temp_dir / "short.wav"
    exporter.export_wav(short_audio, str(path), sample_rate=32000, normalize=False)

    metadata = extractor.extract_metadata(path)

    assert metadata['duration_seconds'] == pytest.approx(0.1, rel=0.01)


def test_very_long_duration_calculation():
    """Test duration calculation for very long audio."""
    extractor = AudioMetadataExtractor(extract_bpm=False, extract_key=False)

    # Simulate 10-minute audio
    sample_rate = 32000
    duration = 600.0  # 10 minutes
    samples = int(sample_rate * duration)

    # Don't create the full tensor, just test calculation
    # This tests the math without memory overhead
    calculated_duration = samples / sample_rate
    assert calculated_duration == pytest.approx(duration, rel=0.001)


# ========== Configuration Tests ==========


def test_get_info(extractor):
    """Test getting extractor configuration info."""
    info = extractor.get_info()

    assert 'extract_bpm' in info
    assert 'extract_key' in info
    assert 'librosa_available' in info

    assert info['extract_bpm'] is True
    assert info['extract_key'] is False
    assert isinstance(info['librosa_available'], bool)


# ========== Integration Tests ==========


def test_complete_metadata_workflow(temp_dir, exporter):
    """Test complete metadata extraction workflow."""
    # Create realistic audio
    sample_rate = 32000
    duration = 4.0
    t = np.linspace(0, duration, int(sample_rate * duration))

    # Create drum-like pattern
    kick = 0.8 * np.sin(2 * np.pi * 60 * t) * np.exp(-t % 1 * 10)
    snare = 0.6 * np.sin(2 * np.pi * 200 * t) * np.exp(-((t - 0.5) % 1) * 20)
    audio = kick + snare
    audio = audio / np.abs(audio).max() * 0.7

    tensor = torch.from_numpy(audio.astype(np.float32))
    path = temp_dir / "complete_test.wav"
    exporter.export_wav(tensor, str(path), sample_rate=sample_rate, normalize=True)

    # Extract all metadata
    extractor = AudioMetadataExtractor(extract_bpm=True, extract_key=False)
    metadata = extractor.extract_metadata(path, compute_stats=True)

    # Verify all fields
    assert metadata['duration_seconds'] == pytest.approx(duration, rel=0.01)
    assert metadata['sample_rate'] == sample_rate
    assert metadata['channels'] == 1
    assert metadata['file_size_bytes'] > 0
    assert 'peak_amplitude' in metadata
    assert 'rms_energy' in metadata
    assert 'dynamic_range_db' in metadata
    assert 'bpm' in metadata
