"""
Audio Testing Helpers
======================

Utilities for validating audio files in integration tests.
Includes WAV validation, loudness measurement, and quality checks.
"""

import logging
from pathlib import Path
from typing import Dict, Any, Optional, Tuple

import numpy as np
import soundfile as sf
import torch

try:
    import pyloudnorm as pyln
    PYLOUDNORM_AVAILABLE = True
except ImportError:
    PYLOUDNORM_AVAILABLE = False

try:
    import librosa
    LIBROSA_AVAILABLE = True
except ImportError:
    LIBROSA_AVAILABLE = False


logger = logging.getLogger(__name__)


def validate_wav_file(
    file_path: Path,
    expected_sample_rate: int = 32000,
    expected_channels: int = 2,
    expected_bit_depth: str = 'PCM_16',
) -> Dict[str, Any]:
    """
    Validate a WAV file meets basic requirements.

    Args:
        file_path: Path to WAV file
        expected_sample_rate: Expected sample rate in Hz
        expected_channels: Expected number of channels (1=mono, 2=stereo)
        expected_bit_depth: Expected bit depth (PCM_16, PCM_24, PCM_32, FLOAT)

    Returns:
        Dictionary with validation results and metadata

    Raises:
        AssertionError: If validation fails
        FileNotFoundError: If file doesn't exist
    """
    assert file_path.exists(), f"File not found: {file_path}"
    assert file_path.suffix == '.wav', f"Not a WAV file: {file_path}"

    # Get file info
    info = sf.info(str(file_path))

    # Validate properties
    assert info.samplerate == expected_sample_rate, (
        f"Sample rate mismatch: {info.samplerate} != {expected_sample_rate}"
    )
    assert info.channels == expected_channels, (
        f"Channels mismatch: {info.channels} != {expected_channels}"
    )

    # Check bit depth (subtype)
    if hasattr(info, 'subtype'):
        assert info.subtype == expected_bit_depth, (
            f"Bit depth mismatch: {info.subtype} != {expected_bit_depth}"
        )

    # Load audio for additional checks
    audio, sr = sf.read(str(file_path))

    # Check audio is not empty
    assert len(audio) > 0, "Audio file is empty"

    # Check audio is not silent
    assert np.abs(audio).max() > 0.0, "Audio file is silent"

    return {
        "valid": True,
        "sample_rate": info.samplerate,
        "channels": info.channels,
        "duration": info.duration,
        "samples": len(audio),
        "bit_depth": getattr(info, 'subtype', None),
        "file_size_bytes": file_path.stat().st_size,
    }


def measure_loudness(file_path: Path) -> Optional[float]:
    """
    Measure integrated loudness in LUFS.

    Args:
        file_path: Path to audio file

    Returns:
        Loudness in LUFS, or None if measurement fails
    """
    if not PYLOUDNORM_AVAILABLE:
        logger.warning("pyloudnorm not available, skipping loudness measurement")
        return None

    try:
        audio, sr = sf.read(str(file_path))
        meter = pyln.Meter(sr)
        loudness = meter.integrated_loudness(audio)
        return float(loudness)
    except Exception as e:
        logger.error(f"Loudness measurement failed: {e}")
        return None


def check_no_clipping(file_path: Path, threshold: float = 0.99) -> Tuple[bool, float]:
    """
    Check if audio has clipping (peaks near or at 1.0).

    Args:
        file_path: Path to audio file
        threshold: Peak threshold (default 0.99)

    Returns:
        Tuple of (has_clipping, peak_value)
    """
    audio, _ = sf.read(str(file_path))
    peak = float(np.abs(audio).max())
    has_clipping = peak >= threshold
    return has_clipping, peak


def verify_audio_quality(
    file_path: Path,
    target_lufs: float = -16.0,
    lufs_tolerance: float = 1.0,
    clip_threshold: float = 0.99,
) -> Dict[str, Any]:
    """
    Comprehensive audio quality verification.

    Args:
        file_path: Path to audio file
        target_lufs: Target loudness in LUFS
        lufs_tolerance: Acceptable deviation from target
        clip_threshold: Peak threshold for clipping detection

    Returns:
        Dictionary with quality metrics and validation results
    """
    results = {
        "file_path": str(file_path),
        "valid": True,
        "errors": [],
    }

    # Basic validation
    try:
        basic = validate_wav_file(file_path)
        results.update(basic)
    except AssertionError as e:
        results["valid"] = False
        results["errors"].append(f"Validation error: {e}")
        return results

    # Loudness check
    if PYLOUDNORM_AVAILABLE:
        loudness = measure_loudness(file_path)
        results["loudness_lufs"] = loudness

        if loudness is not None:
            deviation = abs(loudness - target_lufs)
            results["loudness_deviation"] = deviation

            if deviation > lufs_tolerance:
                results["valid"] = False
                results["errors"].append(
                    f"Loudness outside tolerance: {loudness:.1f} LUFS "
                    f"(target: {target_lufs} ±{lufs_tolerance})"
                )

    # Clipping check
    has_clipping, peak = check_no_clipping(file_path, clip_threshold)
    results["peak_amplitude"] = peak
    results["has_clipping"] = has_clipping

    if has_clipping:
        results["valid"] = False
        results["errors"].append(f"Audio clipping detected: peak={peak:.3f}")

    return results


def compare_audio_properties(
    file_path: Path,
    expected_duration: float,
    duration_tolerance: float = 1.0,
) -> Dict[str, Any]:
    """
    Compare actual audio properties against expected values.

    Args:
        file_path: Path to audio file
        expected_duration: Expected duration in seconds
        duration_tolerance: Acceptable duration deviation in seconds

    Returns:
        Dictionary with comparison results
    """
    info = sf.info(str(file_path))

    duration_diff = abs(info.duration - expected_duration)
    duration_match = duration_diff <= duration_tolerance

    return {
        "duration_expected": expected_duration,
        "duration_actual": info.duration,
        "duration_diff": duration_diff,
        "duration_match": duration_match,
        "sample_rate": info.samplerate,
        "channels": info.channels,
    }


def create_test_audio_tensor(
    duration: float = 1.0,
    sample_rate: int = 32000,
    channels: int = 2,
    frequency: float = 440.0,
) -> torch.Tensor:
    """
    Create a test audio tensor (sine wave).

    Args:
        duration: Duration in seconds
        sample_rate: Sample rate in Hz
        channels: Number of channels
        frequency: Sine wave frequency in Hz

    Returns:
        PyTorch tensor with shape (channels, samples)
    """
    samples = int(duration * sample_rate)
    t = np.linspace(0, duration, samples)
    audio = 0.5 * np.sin(2 * np.pi * frequency * t)

    if channels == 2:
        # Create stereo by duplicating
        audio = np.stack([audio, audio])
    else:
        audio = audio.reshape(1, -1)

    return torch.from_numpy(audio.astype(np.float32))


def extract_bpm(file_path: Path) -> Optional[float]:
    """
    Extract BPM from audio file.

    Args:
        file_path: Path to audio file

    Returns:
        Detected BPM or None
    """
    if not LIBROSA_AVAILABLE:
        return None

    try:
        audio, sr = librosa.load(str(file_path), sr=None)
        tempo, _ = librosa.beat.beat_track(y=audio, sr=sr)
        return float(tempo)
    except Exception as e:
        logger.error(f"BPM extraction failed: {e}")
        return None


def measure_rms_energy(file_path: Path) -> float:
    """
    Measure RMS energy of audio.

    Args:
        file_path: Path to audio file

    Returns:
        RMS energy
    """
    audio, _ = sf.read(str(file_path))
    return float(np.sqrt(np.mean(audio ** 2)))


def check_stereo_balance(file_path: Path, tolerance: float = 0.1) -> Tuple[bool, float]:
    """
    Check if stereo channels are balanced.

    Args:
        file_path: Path to audio file
        tolerance: Maximum acceptable RMS difference

    Returns:
        Tuple of (is_balanced, rms_difference)
    """
    audio, _ = sf.read(str(file_path))

    if audio.ndim != 2 or audio.shape[1] != 2:
        # Not stereo
        return True, 0.0

    left = audio[:, 0]
    right = audio[:, 1]

    rms_left = np.sqrt(np.mean(left ** 2))
    rms_right = np.sqrt(np.mean(right ** 2))

    diff = abs(rms_left - rms_right)
    is_balanced = diff <= tolerance

    return is_balanced, float(diff)


def generate_test_wav(
    output_path: Path,
    duration: float = 1.0,
    sample_rate: int = 32000,
    channels: int = 2,
) -> Path:
    """
    Generate a test WAV file for testing.

    Args:
        output_path: Where to save the file
        duration: Duration in seconds
        sample_rate: Sample rate in Hz
        channels: Number of channels

    Returns:
        Path to created file
    """
    output_path.parent.mkdir(parents=True, exist_ok=True)

    samples = int(duration * sample_rate)
    t = np.linspace(0, duration, samples)

    # Create a simple test tone (440 Hz A4)
    audio = 0.5 * np.sin(2 * np.pi * 440.0 * t)

    if channels == 2:
        audio = np.stack([audio, audio], axis=-1)

    sf.write(str(output_path), audio, sample_rate, subtype='PCM_16')

    return output_path
