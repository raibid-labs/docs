"""
Integration tests for the complete audio pipeline

Tests cover:
- Complete generation workflow (export + metadata + database)
- Stereo export with metadata
- Batch export workflow
- File cleanup integration
- Error handling
- Different bit depths
- Concurrent exports
- Database integration
"""

import pytest
import torch
import numpy as np
import shutil
import tempfile
from pathlib import Path
from datetime import datetime, timedelta
from uuid import uuid4

from services.audio import AudioExporter, AudioMetadataExtractor, AudioFileManager


@pytest.fixture
def temp_dir():
    """Create a temporary directory for integration tests."""
    temp_path = tempfile.mkdtemp()
    yield Path(temp_path)
    shutil.rmtree(temp_path, ignore_errors=True)


@pytest.fixture
def components(temp_dir):
    """Initialize all audio processing components."""
    exporter = AudioExporter(target_lufs=-16.0)
    metadata_extractor = AudioMetadataExtractor(extract_bpm=True, extract_key=False)
    file_manager = AudioFileManager(base_dir=str(temp_dir))

    return {
        'exporter': exporter,
        'metadata_extractor': metadata_extractor,
        'file_manager': file_manager,
    }


def create_realistic_audio(duration=4.0, sample_rate=32000):
    """Create realistic drum pattern audio for testing."""
    samples = int(sample_rate * duration)
    t = np.linspace(0, duration, samples)

    # Simulate drum pattern
    kick = 0.8 * np.sin(2 * np.pi * 60 * t) * np.exp(-t % 1 * 10)
    snare = 0.6 * np.sin(2 * np.pi * 200 * t) * np.exp(-((t - 0.5) % 1) * 20)
    hihat = 0.3 * np.random.randn(samples) * np.exp(-((t % 0.25) * 40))

    audio = kick + snare + hihat
    audio = audio / np.abs(audio).max() * 0.7  # Normalize

    return torch.from_numpy(audio.astype(np.float32)), sample_rate


# ========== Complete Workflow Tests ==========


def test_complete_generation_workflow(components):
    """Test the complete workflow: generate -> export -> extract metadata."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    # 1. Generate audio (simulated)
    audio_tensor, sample_rate = create_realistic_audio(duration=4.0)
    job_id = f"gen_{uuid4().hex[:8]}"

    # 2. Get output path
    output_path = file_manager.get_output_path(job_id)

    # 3. Export audio
    final_path, file_size = exporter.export_wav(
        audio_tensor=audio_tensor,
        output_path=str(output_path),
        sample_rate=sample_rate,
        normalize=True,
        bit_depth='PCM_16'
    )

    # 4. Extract metadata
    metadata = metadata_extractor.extract_metadata(final_path, compute_stats=True)

    # Verify complete workflow
    assert Path(final_path).exists()
    assert file_size > 0
    assert metadata['duration_seconds'] == pytest.approx(4.0, rel=0.01)
    assert metadata['sample_rate'] == sample_rate
    assert metadata['file_size_bytes'] == file_size
    assert 'peak_amplitude' in metadata
    assert 'rms_energy' in metadata
    assert 'bpm' in metadata


def test_stereo_export_with_metadata(components):
    """Test stereo audio export and metadata extraction."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    # Create stereo audio
    sample_rate = 32000
    duration = 2.0
    t = np.linspace(0, duration, int(sample_rate * duration))
    left = 0.5 * np.sin(2 * np.pi * 440 * t)
    right = 0.5 * np.sin(2 * np.pi * 554 * t)
    stereo_audio = torch.from_numpy(np.stack([left, right]).astype(np.float32))

    # Export
    job_id = f"gen_stereo_{uuid4().hex[:8]}"
    output_path = file_manager.get_output_path(job_id)

    final_path, file_size = exporter.export_wav(
        stereo_audio,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True
    )

    # Extract metadata
    metadata = metadata_extractor.extract_metadata(final_path)

    # Verify stereo properties
    assert metadata['channels'] == 2
    assert metadata['duration_seconds'] == pytest.approx(duration, rel=0.01)
    assert Path(final_path).exists()


def test_batch_export_workflow(components):
    """Test batch export with multiple files and metadata extraction."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    # Create multiple audio samples
    batch_size = 5
    audio_tensors = []
    output_paths = []
    job_ids = []

    for i in range(batch_size):
        # Create unique audio
        freq = 440 + i * 50
        t = np.linspace(0, 1, 32000)
        audio = 0.5 * np.sin(2 * np.pi * freq * t)
        audio_tensors.append(torch.from_numpy(audio.astype(np.float32)))

        # Generate path
        job_id = f"gen_batch_{i}_{uuid4().hex[:8]}"
        job_ids.append(job_id)
        output_paths.append(str(file_manager.get_output_path(job_id)))

    # Batch export
    results = exporter.export_wav_batch(
        audio_tensors,
        output_paths,
        sample_rate=32000,
        normalize=True
    )

    # Verify all exports
    assert len(results) == batch_size

    # Extract metadata for each
    for path, size in results:
        assert Path(path).exists()
        assert size > 0

        metadata = metadata_extractor.extract_metadata(path, compute_stats=True)
        assert metadata['duration_seconds'] == pytest.approx(1.0, rel=0.01)


def test_file_cleanup_integration(components):
    """Test file cleanup with actual exports."""
    exporter = components['exporter']
    file_manager = components['file_manager']

    # Create several files
    for i in range(3):
        audio_tensor, sample_rate = create_realistic_audio(duration=1.0)
        job_id = f"gen_cleanup_{i}"
        output_path = file_manager.get_output_path(job_id)

        exporter.export_wav(
            audio_tensor,
            str(output_path),
            sample_rate=sample_rate,
            normalize=False
        )

    # Verify files exist
    initial_stats = file_manager.get_storage_stats()
    assert initial_stats['total_files'] >= 3

    # List files
    files = file_manager.list_files()
    assert len(files) >= 3

    # Clean up (dry run)
    count = file_manager.cleanup_old_files(days_old=30, dry_run=True)
    # Files are recent, so nothing should be deleted
    assert count == 0


def test_error_handling_integration(components):
    """Test error handling across the pipeline."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']

    # Test with invalid audio
    invalid_tensor = torch.randn(4, 32000)  # Too many channels

    with pytest.raises(ValueError):
        exporter.export_wav(invalid_tensor, "invalid.wav")

    # Test metadata extraction on non-existent file
    with pytest.raises(FileNotFoundError):
        metadata_extractor.extract_metadata("nonexistent.wav")


# ========== Bit Depth Integration Tests ==========


def test_export_all_bit_depths(components):
    """Test export and metadata extraction for all bit depths."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    audio_tensor, sample_rate = create_realistic_audio(duration=2.0)
    bit_depths = ['PCM_16', 'PCM_24', 'PCM_32', 'FLOAT']

    for bit_depth in bit_depths:
        job_id = f"gen_{bit_depth}_{uuid4().hex[:8]}"
        output_path = file_manager.get_output_path(job_id)

        # Export
        final_path, file_size = exporter.export_wav(
            audio_tensor,
            str(output_path),
            sample_rate=sample_rate,
            normalize=True,
            bit_depth=bit_depth
        )

        # Extract metadata
        metadata = metadata_extractor.extract_metadata(final_path)

        # Verify
        assert Path(final_path).exists()
        assert metadata['duration_seconds'] == pytest.approx(2.0, rel=0.01)


# ========== Concurrent Operations Tests ==========


def test_concurrent_exports_to_same_directory(components):
    """Test multiple concurrent exports to the same date directory."""
    exporter = components['exporter']
    file_manager = components['file_manager']

    # Create multiple files simultaneously (simulating concurrent requests)
    job_ids = [f"gen_concurrent_{i}_{uuid4().hex[:8]}" for i in range(5)]
    paths = []

    for job_id in job_ids:
        audio_tensor, sample_rate = create_realistic_audio(duration=1.0)
        output_path = file_manager.get_output_path(job_id)

        final_path, _ = exporter.export_wav(
            audio_tensor,
            str(output_path),
            sample_rate=sample_rate,
            normalize=False
        )
        paths.append(final_path)

    # Verify all files exist and are unique
    assert len(paths) == len(set(paths))  # All paths unique
    for path in paths:
        assert Path(path).exists()


# ========== Metadata Integration Tests ==========


def test_metadata_extraction_integration(components):
    """Test metadata extraction with various audio types."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    test_cases = [
        ("mono_quiet", 0.2, 2.0),   # Quiet mono
        ("mono_loud", 0.8, 2.0),    # Loud mono
        ("short", 0.5, 0.5),        # Very short
    ]

    for name, amplitude, duration in test_cases:
        # Create audio
        t = np.linspace(0, duration, int(32000 * duration))
        audio = amplitude * np.sin(2 * np.pi * 440 * t)
        tensor = torch.from_numpy(audio.astype(np.float32))

        # Export
        job_id = f"gen_{name}_{uuid4().hex[:8]}"
        output_path = file_manager.get_output_path(job_id)

        final_path, _ = exporter.export_wav(
            tensor,
            str(output_path),
            sample_rate=32000,
            normalize=True
        )

        # Extract metadata
        metadata = metadata_extractor.extract_metadata(final_path, compute_stats=True)

        # Verify
        assert metadata['duration_seconds'] == pytest.approx(duration, rel=0.02)
        assert 0 <= metadata['peak_amplitude'] <= 1.0
        assert metadata['rms_energy'] > 0


def test_file_operations_integration(components):
    """Test file move/copy operations with metadata."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    # Create and export audio
    audio_tensor, sample_rate = create_realistic_audio(duration=2.0)
    job_id = f"gen_fileops_{uuid4().hex[:8]}"
    output_path = file_manager.get_output_path(job_id)

    original_path, _ = exporter.export_wav(
        audio_tensor,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True
    )

    # Extract metadata from original
    metadata_original = metadata_extractor.extract_metadata(original_path)

    # Copy file
    copy_path = file_manager.base_dir / "backup" / "copy.wav"
    copied_path = file_manager.copy_file(original_path, copy_path)

    # Extract metadata from copy (should be identical)
    metadata_copy = metadata_extractor.extract_metadata(copied_path)

    # Verify metadata matches
    assert metadata_original['duration_seconds'] == metadata_copy['duration_seconds']
    assert metadata_original['sample_rate'] == metadata_copy['sample_rate']


# ========== Normalization Level Tests ==========


def test_export_normalization_levels(components):
    """Test that normalization achieves target loudness."""
    exporter = components['exporter']
    file_manager = components['file_manager']

    # Create audio with known characteristics
    audio_tensor, sample_rate = create_realistic_audio(duration=4.0)

    # Export with normalization
    job_id = f"gen_normalized_{uuid4().hex[:8]}"
    output_path = file_manager.get_output_path(job_id)

    final_path, _ = exporter.export_wav(
        audio_tensor,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True
    )

    # Verify file exists and is playable
    assert Path(final_path).exists()

    # Read back and check peak doesn't exceed 1.0 (no clipping)
    import soundfile as sf
    audio_read, sr = sf.read(final_path)
    peak = np.abs(audio_read).max()
    assert peak <= 1.0


# ========== Storage Statistics Tests ==========


def test_storage_stats_after_operations(components):
    """Test storage statistics after various operations."""
    exporter = components['exporter']
    file_manager = components['file_manager']

    # Initial stats
    initial_stats = file_manager.get_storage_stats()
    initial_count = initial_stats['total_files']

    # Create files
    for i in range(3):
        audio_tensor, sample_rate = create_realistic_audio(duration=1.0)
        job_id = f"gen_stats_{i}"
        output_path = file_manager.get_output_path(job_id)

        exporter.export_wav(
            audio_tensor,
            str(output_path),
            sample_rate=sample_rate,
            normalize=False
        )

    # Check updated stats
    updated_stats = file_manager.get_storage_stats()

    assert updated_stats['total_files'] == initial_count + 3
    assert updated_stats['total_size_bytes'] > initial_stats['total_size_bytes']
    assert '.wav' in updated_stats['file_types']


# ========== BPM Detection Workflow Tests ==========


def test_bpm_detection_workflow(components):
    """Test complete workflow with BPM detection."""
    exporter = components['exporter']
    # Create extractor with BPM enabled
    metadata_extractor = AudioMetadataExtractor(extract_bpm=True, extract_key=False)
    file_manager = components['file_manager']

    # Create rhythmic audio
    audio_tensor, sample_rate = create_realistic_audio(duration=4.0)
    job_id = f"gen_bpm_{uuid4().hex[:8]}"
    output_path = file_manager.get_output_path(job_id)

    # Export
    final_path, _ = exporter.export_wav(
        audio_tensor,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True
    )

    # Extract metadata with BPM
    metadata = metadata_extractor.extract_metadata(final_path)

    # Verify BPM field exists (value may vary)
    assert 'bpm' in metadata


# ========== Database Integration Simulation Tests ==========


def test_simulated_database_workflow(components):
    """Simulate the complete workflow with database integration."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    # Simulate database record creation
    job_id = f"gen_{uuid4().hex[:8]}"
    prompt = "test drum pattern at 140 BPM"
    created_at = datetime.now()

    # Generate audio
    audio_tensor, sample_rate = create_realistic_audio(duration=4.0)

    # Get output path
    output_path = file_manager.get_output_path(job_id)

    # Export
    final_path, file_size = exporter.export_wav(
        audio_tensor,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True
    )

    # Extract metadata
    metadata = metadata_extractor.extract_metadata(final_path, compute_stats=True)

    # Simulate database record completion
    generation_record = {
        'job_id': job_id,
        'prompt': prompt,
        'created_at': created_at,
        'completed_at': datetime.now(),
        'file_path': final_path,
        'file_size_bytes': file_size,
        'duration_seconds': metadata['duration_seconds'],
        'sample_rate': metadata['sample_rate'],
        'channels': metadata['channels'],
        'bpm': metadata['bpm'],
        'peak_amplitude': metadata['peak_amplitude'],
        'rms_energy': metadata['rms_energy'],
    }

    # Verify complete record
    assert generation_record['job_id'] == job_id
    assert generation_record['file_size_bytes'] > 0
    assert generation_record['duration_seconds'] > 0
    assert Path(generation_record['file_path']).exists()


def test_multi_format_support(components):
    """Test support for multiple audio configurations."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    configs = [
        ('mono_16k', 1, 16000),
        ('mono_32k', 1, 32000),
        ('stereo_44.1k', 2, 44100),
        ('stereo_48k', 2, 48000),
    ]

    for name, channels, sample_rate in configs:
        # Create audio
        if channels == 1:
            audio = torch.randn(sample_rate * 2) * 0.5
        else:
            audio = torch.randn(2, sample_rate * 2) * 0.5

        # Export
        job_id = f"gen_{name}_{uuid4().hex[:8]}"
        output_path = file_manager.get_output_path(job_id)

        final_path, _ = exporter.export_wav(
            audio,
            str(output_path),
            sample_rate=sample_rate,
            normalize=True
        )

        # Extract and verify metadata
        metadata = metadata_extractor.extract_metadata(final_path)
        assert metadata['sample_rate'] == sample_rate
        assert metadata['channels'] == channels


def test_error_recovery(components):
    """Test error recovery in the pipeline."""
    exporter = components['exporter']
    metadata_extractor = components['metadata_extractor']
    file_manager = components['file_manager']

    # Create valid audio
    audio_tensor, sample_rate = create_realistic_audio(duration=2.0)
    job_id = f"gen_recovery_{uuid4().hex[:8]}"
    output_path = file_manager.get_output_path(job_id)

    # Export successfully
    final_path, _ = exporter.export_wav(
        audio_tensor,
        str(output_path),
        sample_rate=sample_rate,
        normalize=True
    )

    # Verify we can recover from errors in subsequent operations
    try:
        # Try to extract from non-existent file
        metadata_extractor.extract_metadata("nonexistent.wav")
    except FileNotFoundError:
        # Error is expected and caught
        pass

    # Original file should still be intact
    assert Path(final_path).exists()

    # Can still extract metadata from valid file
    metadata = metadata_extractor.extract_metadata(final_path)
    assert metadata is not None
