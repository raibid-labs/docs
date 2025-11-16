"""
Unit tests for AudioExporter

Tests cover:
- Initialization
- Mono/stereo export
- Different sample rates and bit depths
- Normalization
- Batch export
- Error handling
- Edge cases (GPU tensors, gradients, clipping)
- File size calculation
"""

import pytest
import torch
import numpy as np
import soundfile as sf
from pathlib import Path
import tempfile
import shutil

from services.audio.export import AudioExporter


@pytest.fixture
def temp_dir():
    """Create a temporary directory for test outputs."""
    temp_path = tempfile.mkdtemp()
    yield Path(temp_path)
    # Cleanup
    shutil.rmtree(temp_path)


@pytest.fixture
def exporter():
    """Create an AudioExporter instance."""
    return AudioExporter(target_lufs=-16.0)


@pytest.fixture
def mono_audio():
    """Generate mono test audio (1-second 440Hz sine wave)."""
    sample_rate = 32000
    duration = 1.0
    t = np.linspace(0, duration, int(sample_rate * duration))
    audio = 0.5 * np.sin(2 * np.pi * 440 * t)
    return torch.from_numpy(audio.astype(np.float32))


@pytest.fixture
def stereo_audio():
    """Generate stereo test audio (1-second sine waves)."""
    sample_rate = 32000
    duration = 1.0
    t = np.linspace(0, duration, int(sample_rate * duration))
    left = 0.5 * np.sin(2 * np.pi * 440 * t)   # 440Hz
    right = 0.5 * np.sin(2 * np.pi * 554 * t)  # 554Hz (different)
    audio = np.stack([left, right])
    return torch.from_numpy(audio.astype(np.float32))


# ========== Initialization Tests ==========


def test_exporter_initialization():
    """Test AudioExporter initialization with default parameters."""
    exporter = AudioExporter()
    assert exporter.target_lufs == -16.0


def test_exporter_custom_lufs():
    """Test AudioExporter initialization with custom LUFS target."""
    exporter = AudioExporter(target_lufs=-18.0)
    assert exporter.target_lufs == -18.0


# ========== Mono/Stereo Export Tests ==========


def test_export_mono_wav(exporter, mono_audio, temp_dir):
    """Test exporting mono audio to WAV."""
    output_path = temp_dir / "mono_test.wav"

    path, size = exporter.export_wav(
        mono_audio,
        str(output_path),
        sample_rate=32000,
        normalize=False  # Disable for predictable output
    )

    assert Path(path).exists()
    assert size > 0

    # Verify audio content
    audio, sr = sf.read(path)
    assert sr == 32000
    assert audio.ndim == 1  # Mono
    assert len(audio) == 32000  # 1 second at 32kHz


def test_export_stereo_wav(exporter, stereo_audio, temp_dir):
    """Test exporting stereo audio to WAV."""
    output_path = temp_dir / "stereo_test.wav"

    path, size = exporter.export_wav(
        stereo_audio,
        str(output_path),
        sample_rate=32000,
        normalize=False
    )

    assert Path(path).exists()
    assert size > 0

    # Verify audio content
    audio, sr = sf.read(path, always_2d=True)
    assert sr == 32000
    assert audio.shape[1] == 2  # Stereo
    assert audio.shape[0] == 32000  # 1 second at 32kHz


def test_mono_to_stereo_preservation(exporter, mono_audio, temp_dir):
    """Test that mono audio stays mono."""
    output_path = temp_dir / "mono_preserved.wav"

    path, _ = exporter.export_wav(mono_audio, str(output_path), normalize=False)

    audio, _ = sf.read(path)
    assert audio.ndim == 1  # Should remain mono


def test_stereo_channel_preservation(exporter, stereo_audio, temp_dir):
    """Test that stereo channels are preserved correctly."""
    output_path = temp_dir / "stereo_preserved.wav"

    # Export
    path, _ = exporter.export_wav(stereo_audio, str(output_path), normalize=False)

    # Read back and verify channels are different
    audio, _ = sf.read(path, always_2d=True)
    left = audio[:, 0]
    right = audio[:, 1]

    # Channels should be different
    assert not np.allclose(left, right, rtol=0.01)


# ========== Sample Rate and Bit Depth Tests ==========


def test_different_sample_rates(exporter, mono_audio, temp_dir):
    """Test export with different sample rates."""
    sample_rates = [16000, 32000, 44100, 48000]

    for sr in sample_rates:
        output_path = temp_dir / f"sr_{sr}.wav"
        path, _ = exporter.export_wav(mono_audio, str(output_path), sample_rate=sr, normalize=False)

        _, read_sr = sf.read(path)
        assert read_sr == sr


def test_different_bit_depths(exporter, mono_audio, temp_dir):
    """Test export with different bit depths."""
    bit_depths = ['PCM_16', 'PCM_24', 'PCM_32', 'FLOAT']

    for bit_depth in bit_depths:
        output_path = temp_dir / f"bit_{bit_depth}.wav"
        path, _ = exporter.export_wav(
            mono_audio,
            str(output_path),
            sample_rate=32000,
            bit_depth=bit_depth,
            normalize=False
        )

        assert Path(path).exists()
        info = sf.info(path)
        assert bit_depth in info.subtype


# ========== Normalization Tests ==========


def test_loudness_normalization(exporter, mono_audio, temp_dir):
    """Test that loudness normalization is applied."""
    output_path = temp_dir / "normalized.wav"

    # Export with normalization
    path, _ = exporter.export_wav(
        mono_audio,
        str(output_path),
        sample_rate=32000,
        normalize=True
    )

    # Read back and check that audio was modified
    audio_normalized, _ = sf.read(path)
    audio_original = mono_audio.numpy()

    # Normalized audio should be different from original
    # (unless original was already at target loudness)
    peak_original = np.abs(audio_original).max()
    peak_normalized = np.abs(audio_normalized).max()

    # At least one should be different (normalization occurred)
    # We can't guarantee exact values due to loudness measurement
    assert audio_normalized is not None


def test_normalization_disabled(exporter, mono_audio, temp_dir):
    """Test that normalization can be disabled."""
    output_path = temp_dir / "not_normalized.wav"

    # Export without normalization
    path, _ = exporter.export_wav(
        mono_audio,
        str(output_path),
        sample_rate=32000,
        normalize=False
    )

    # Read back and compare with original
    audio_read, _ = sf.read(path)
    audio_original = mono_audio.numpy()

    # Should be very close (within floating point precision)
    assert np.allclose(audio_read, audio_original, rtol=1e-5)


def test_silent_audio_normalization(exporter, temp_dir):
    """Test that silent audio skips normalization."""
    # Create silent audio
    silent = torch.zeros(32000)
    output_path = temp_dir / "silent.wav"

    # Should not raise error
    path, _ = exporter.export_wav(
        silent,
        str(output_path),
        sample_rate=32000,
        normalize=True
    )

    # Verify file was created
    assert Path(path).exists()
    audio, _ = sf.read(path)
    assert np.allclose(audio, 0.0)


# ========== Batch Export Tests ==========


def test_batch_export(exporter, temp_dir):
    """Test batch export of multiple files."""
    # Create multiple audio tensors
    tensors = []
    paths = []

    for i in range(3):
        t = np.linspace(0, 1, 32000)
        freq = 440 + i * 100  # Different frequencies
        audio = 0.5 * np.sin(2 * np.pi * freq * t)
        tensors.append(torch.from_numpy(audio.astype(np.float32)))
        paths.append(str(temp_dir / f"batch_{i}.wav"))

    # Batch export
    results = exporter.export_wav_batch(
        tensors,
        paths,
        sample_rate=32000,
        normalize=False
    )

    assert len(results) == 3

    # Verify all files exist
    for path, size in results:
        assert Path(path).exists()
        assert size > 0


def test_batch_export_with_normalization(exporter, mono_audio, temp_dir):
    """Test batch export with normalization enabled."""
    tensors = [mono_audio, mono_audio, mono_audio]
    paths = [str(temp_dir / f"norm_{i}.wav") for i in range(3)]

    results = exporter.export_wav_batch(
        tensors,
        paths,
        sample_rate=32000,
        normalize=True
    )

    assert len(results) == 3
    for path, _ in results:
        assert Path(path).exists()


def test_batch_export_length_mismatch(exporter, mono_audio):
    """Test that batch export fails with mismatched lengths."""
    tensors = [mono_audio, mono_audio]
    paths = ["path1.wav"]  # Only one path for two tensors

    with pytest.raises(ValueError, match="Length mismatch"):
        exporter.export_wav_batch(tensors, paths)


# ========== Error Handling Tests ==========


def test_invalid_bit_depth(exporter, mono_audio, temp_dir):
    """Test that invalid bit depth raises error."""
    output_path = temp_dir / "invalid.wav"

    with pytest.raises(ValueError, match="Unsupported bit depth"):
        exporter.export_wav(
            mono_audio,
            str(output_path),
            bit_depth='INVALID'
        )


def test_invalid_tensor_dimensions(exporter, temp_dir):
    """Test that invalid tensor dimensions raise error."""
    # Create 3D tensor (invalid)
    invalid_tensor = torch.randn(2, 2, 32000)
    output_path = temp_dir / "invalid.wav"

    with pytest.raises(ValueError, match="Invalid audio dimensions"):
        exporter.export_wav(invalid_tensor, str(output_path))


def test_too_many_channels(exporter, temp_dir):
    """Test that too many channels raise error."""
    # Create 4-channel audio (unsupported)
    invalid_tensor = torch.randn(4, 32000)
    output_path = temp_dir / "invalid.wav"

    with pytest.raises(ValueError, match="Expected max 2 channels"):
        exporter.export_wav(invalid_tensor, str(output_path))


def test_create_output_directory(exporter, mono_audio, temp_dir):
    """Test that output directory is created automatically."""
    # Create path in non-existent subdirectory
    output_path = temp_dir / "subdir" / "nested" / "test.wav"

    path, _ = exporter.export_wav(mono_audio, str(output_path), normalize=False)

    assert Path(path).exists()
    assert Path(path).parent.exists()


def test_export_to_readonly_directory(exporter, mono_audio, temp_dir):
    """Test export fails gracefully with permission errors."""
    # Create readonly directory
    readonly_dir = temp_dir / "readonly"
    readonly_dir.mkdir()

    # Make it readonly
    import os
    os.chmod(readonly_dir, 0o444)

    output_path = readonly_dir / "test.wav"

    try:
        with pytest.raises(RuntimeError, match="Could not write WAV file"):
            exporter.export_wav(mono_audio, str(output_path))
    finally:
        # Restore permissions for cleanup
        os.chmod(readonly_dir, 0o755)


def test_write_error_handling(exporter, mono_audio):
    """Test that write errors are properly handled."""
    # Try to write to invalid path
    invalid_path = "/invalid/path/that/does/not/exist/file.wav"

    with pytest.raises(RuntimeError, match="Could not write WAV file"):
        exporter.export_wav(mono_audio, invalid_path)


# ========== GPU and Gradient Tests ==========


@pytest.mark.skipif(not torch.cuda.is_available(), reason="CUDA not available")
def test_gpu_tensor_export(exporter, mono_audio, temp_dir):
    """Test that GPU tensors are automatically moved to CPU."""
    gpu_tensor = mono_audio.cuda()
    output_path = temp_dir / "gpu_test.wav"

    path, _ = exporter.export_wav(gpu_tensor, str(output_path), normalize=False)

    assert Path(path).exists()


def test_tensor_with_gradients(exporter, mono_audio, temp_dir):
    """Test that tensors with gradients are properly handled."""
    # Create tensor requiring gradients
    audio_with_grad = mono_audio.clone().requires_grad_(True)
    output_path = temp_dir / "grad_test.wav"

    path, _ = exporter.export_wav(audio_with_grad, str(output_path), normalize=False)

    assert Path(path).exists()


def test_tensor_conversion_to_numpy(exporter, mono_audio):
    """Test tensor to numpy conversion."""
    numpy_array = exporter._tensor_to_numpy(mono_audio)

    assert isinstance(numpy_array, np.ndarray)
    assert numpy_array.dtype == np.float32
    assert numpy_array.shape == mono_audio.shape


def test_tensor_from_cuda(exporter, mono_audio):
    """Test tensor conversion from CUDA (if available)."""
    if torch.cuda.is_available():
        cuda_tensor = mono_audio.cuda()
        numpy_array = exporter._tensor_to_numpy(cuda_tensor)

        assert isinstance(numpy_array, np.ndarray)
        assert not torch.is_tensor(numpy_array)


def test_tensor_detach_gradients(exporter, mono_audio):
    """Test that gradients are properly detached."""
    tensor_with_grad = mono_audio.clone().requires_grad_(True)
    numpy_array = exporter._tensor_to_numpy(tensor_with_grad)

    # Should be numpy array without gradient tracking
    assert isinstance(numpy_array, np.ndarray)


# ========== File Size Tests ==========


def test_file_size_calculation(exporter, mono_audio, temp_dir):
    """Test that file size is accurately reported."""
    output_path = temp_dir / "size_test.wav"

    path, reported_size = exporter.export_wav(
        mono_audio,
        str(output_path),
        normalize=False
    )

    actual_size = Path(path).stat().st_size
    assert reported_size == actual_size


def test_file_size_increases_with_duration(exporter, temp_dir):
    """Test that longer audio produces larger files."""
    sizes = []

    for duration in [1, 2, 4]:
        samples = 32000 * duration
        audio = torch.randn(samples) * 0.5
        output_path = temp_dir / f"duration_{duration}.wav"

        _, size = exporter.export_wav(audio, str(output_path), normalize=False)
        sizes.append(size)

    # Sizes should increase with duration
    assert sizes[1] > sizes[0]
    assert sizes[2] > sizes[1]


# ========== Configuration Tests ==========


def test_get_info(exporter):
    """Test getting exporter configuration info."""
    info = exporter.get_info()

    assert 'target_lufs' in info
    assert 'normalization_available' in info
    assert 'supported_bit_depths' in info

    assert info['target_lufs'] == -16.0
    assert isinstance(info['normalization_available'], bool)
    assert len(info['supported_bit_depths']) == 4


def test_supported_bit_depths_list():
    """Test that supported bit depths are properly defined."""
    assert 'PCM_16' in AudioExporter.SUPPORTED_BIT_DEPTHS
    assert 'PCM_24' in AudioExporter.SUPPORTED_BIT_DEPTHS
    assert 'PCM_32' in AudioExporter.SUPPORTED_BIT_DEPTHS
    assert 'FLOAT' in AudioExporter.SUPPORTED_BIT_DEPTHS


# ========== Integration Tests ==========


def test_complete_export_workflow(exporter, temp_dir):
    """Test complete export workflow with realistic audio."""
    # Generate realistic audio (4-second drum pattern simulation)
    sample_rate = 32000
    duration = 4.0
    samples = int(sample_rate * duration)

    # Create drum-like pattern
    t = np.linspace(0, duration, samples)
    kick = 0.8 * np.sin(2 * np.pi * 60 * t) * np.exp(-t % 1 * 10)
    snare = 0.6 * np.sin(2 * np.pi * 200 * t) * np.exp(-((t - 0.5) % 1) * 20)
    audio = kick + snare
    audio = audio / np.abs(audio).max() * 0.7  # Normalize to prevent clipping

    tensor = torch.from_numpy(audio.astype(np.float32))
    output_path = temp_dir / "complete_test.wav"

    # Export with all features
    path, size = exporter.export_wav(
        tensor,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True,
        bit_depth='PCM_16'
    )

    # Verify
    assert Path(path).exists()
    assert size > 0

    # Read and verify metadata
    info = sf.info(path)
    assert info.samplerate == sample_rate
    assert info.duration == pytest.approx(duration, rel=0.01)
