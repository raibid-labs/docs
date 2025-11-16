"""
Audio Metadata Extraction Module
==================================

Extract comprehensive metadata from audio files and tensors.

This module provides the AudioMetadataExtractor class which handles:
- Basic metadata: duration, sample rate, channels, file size
- Optional BPM detection using librosa beat tracking
- Optional musical key detection (experimental)
- Audio statistics: peak amplitude, RMS energy, dynamic range
- Direct tensor analysis (without file I/O)

Usage:
    from services.audio import AudioMetadataExtractor

    extractor = AudioMetadataExtractor(extract_bpm=True)
    metadata = extractor.extract_metadata("audio.wav")
    print(f"BPM: {metadata['bpm']}, Duration: {metadata['duration_seconds']}s")
"""

import logging
from pathlib import Path
from typing import Dict, Any, Optional, Union

import numpy as np
import soundfile as sf
import torch

try:
    import librosa
    LIBROSA_AVAILABLE = True
except ImportError:
    LIBROSA_AVAILABLE = False
    logging.warning("librosa not available - BPM and key detection will be disabled")


logger = logging.getLogger(__name__)


class AudioMetadataExtractor:
    """
    Extract comprehensive metadata from audio files and tensors.

    This class provides detailed audio analysis including:
    - Basic properties (duration, sample rate, channels, file size)
    - Audio statistics (peak amplitude, RMS energy, dynamic range)
    - Optional BPM detection (tempo estimation)
    - Optional key detection (musical key, experimental)

    Attributes:
        extract_bpm (bool): Enable BPM detection (slower, ~3s per file)
        extract_key (bool): Enable key detection (slower, experimental)
    """

    def __init__(
        self,
        extract_bpm: bool = True,
        extract_key: bool = False,
    ):
        """
        Initialize the AudioMetadataExtractor.

        Args:
            extract_bpm: Enable BPM (tempo) detection. This adds ~3s per file
                        but provides valuable tempo information.
            extract_key: Enable musical key detection (experimental).
                        This is slower and accuracy varies.
        """
        self.extract_bpm = extract_bpm
        self.extract_key = extract_key

        logger.info(
            f"AudioMetadataExtractor initialized: "
            f"bpm={extract_bpm}, key={extract_key}"
        )

        if (extract_bpm or extract_key) and not LIBROSA_AVAILABLE:
            logger.warning(
                "librosa not available - BPM and key detection disabled. "
                "Install with: pip install librosa"
            )

    def extract_metadata(
        self,
        audio_path: Union[str, Path],
        compute_stats: bool = True,
    ) -> Dict[str, Any]:
        """
        Extract metadata from an audio file.

        Args:
            audio_path: Path to audio file (WAV, FLAC, MP3, etc.)
            compute_stats: Compute audio statistics (peak, RMS, dynamic range)

        Returns:
            Dictionary with metadata fields:
                - duration_seconds: Audio duration
                - sample_rate: Sample rate in Hz
                - channels: Number of channels
                - file_size_bytes: File size
                - bit_depth: Bit depth (if available)
                - bpm: Detected tempo (if enabled)
                - key: Musical key (if enabled)
                - peak_amplitude: Maximum absolute value (if compute_stats)
                - rms_energy: RMS energy (if compute_stats)
                - dynamic_range_db: Dynamic range in dB (if compute_stats)

        Raises:
            FileNotFoundError: If audio file doesn't exist
            RuntimeError: If metadata extraction fails
        """
        audio_path = Path(audio_path)

        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        logger.debug(f"Extracting metadata from: {audio_path}")

        try:
            # Get basic file info using soundfile (fast)
            info = sf.info(str(audio_path))

            metadata = {
                "duration_seconds": info.duration,
                "sample_rate": info.samplerate,
                "channels": info.channels,
                "file_size_bytes": audio_path.stat().st_size,
                "bit_depth": getattr(info, 'subtype_info', None),
            }

            # Load audio for advanced analysis
            if compute_stats or self.extract_bpm or self.extract_key:
                audio, sr = sf.read(str(audio_path), always_2d=False)

                # Compute audio statistics
                if compute_stats:
                    stats = self._compute_statistics(audio)
                    metadata.update(stats)

                # Extract BPM
                if self.extract_bpm and LIBROSA_AVAILABLE:
                    bpm = self._extract_bpm(audio, sr)
                    metadata["bpm"] = bpm
                else:
                    metadata["bpm"] = None

                # Extract key
                if self.extract_key and LIBROSA_AVAILABLE:
                    key = self._extract_key(audio, sr)
                    metadata["key"] = key
                else:
                    metadata["key"] = None

            logger.debug(f"Metadata extracted: {metadata}")
            return metadata

        except Exception as e:
            logger.error(f"Failed to extract metadata from {audio_path}: {e}")
            raise RuntimeError(f"Metadata extraction failed: {e}")

    def extract_metadata_from_tensor(
        self,
        audio_tensor: torch.Tensor,
        sample_rate: int,
        compute_stats: bool = True,
    ) -> Dict[str, Any]:
        """
        Extract metadata directly from a PyTorch tensor (no file I/O).

        This is useful for analyzing generated audio before saving to disk.

        Args:
            audio_tensor: PyTorch tensor (channels, samples) or (samples,)
            sample_rate: Sample rate in Hz
            compute_stats: Compute audio statistics

        Returns:
            Dictionary with metadata (no file_size_bytes or bit_depth)
        """
        logger.debug(f"Extracting metadata from tensor: shape={audio_tensor.shape}")

        # Convert to numpy
        if audio_tensor.is_cuda:
            audio_tensor = audio_tensor.cpu()
        if audio_tensor.requires_grad:
            audio_tensor = audio_tensor.detach()

        audio = audio_tensor.numpy().astype(np.float32)

        # Determine shape
        if audio.ndim == 1:
            channels = 1
            samples = len(audio)
        elif audio.ndim == 2:
            channels, samples = audio.shape
            # Convert to mono for analysis if stereo
            if channels == 2:
                audio = audio.mean(axis=0)
        else:
            raise ValueError(f"Invalid audio dimensions: {audio.ndim}")

        duration = samples / sample_rate

        metadata = {
            "duration_seconds": duration,
            "sample_rate": sample_rate,
            "channels": channels,
            "file_size_bytes": None,  # Not applicable for tensors
            "bit_depth": None,  # Not applicable for tensors
        }

        # Compute statistics
        if compute_stats:
            stats = self._compute_statistics(audio)
            metadata.update(stats)

        # Extract BPM
        if self.extract_bpm and LIBROSA_AVAILABLE:
            bpm = self._extract_bpm(audio, sample_rate)
            metadata["bpm"] = bpm
        else:
            metadata["bpm"] = None

        # Extract key
        if self.extract_key and LIBROSA_AVAILABLE:
            key = self._extract_key(audio, sample_rate)
            metadata["key"] = key
        else:
            metadata["key"] = None

        return metadata

    def _compute_statistics(self, audio: np.ndarray) -> Dict[str, float]:
        """
        Compute audio statistics.

        Args:
            audio: Audio array (mono or will be converted to mono)

        Returns:
            Dictionary with:
                - peak_amplitude: Maximum absolute value
                - rms_energy: Root mean square energy
                - dynamic_range_db: Dynamic range in decibels
        """
        # Convert to mono if needed
        if audio.ndim == 2:
            audio = audio.mean(axis=0)

        # Peak amplitude
        peak = float(np.abs(audio).max())

        # RMS energy
        rms = float(np.sqrt(np.mean(audio ** 2)))

        # Dynamic range (peak to RMS ratio in dB)
        if rms > 0:
            dynamic_range_db = float(20 * np.log10(peak / rms))
        else:
            dynamic_range_db = 0.0

        return {
            "peak_amplitude": peak,
            "rms_energy": rms,
            "dynamic_range_db": dynamic_range_db,
        }

    def _extract_bpm(
        self,
        audio: np.ndarray,
        sample_rate: int,
    ) -> Optional[float]:
        """
        Extract BPM (tempo) from audio using librosa beat tracking.

        This uses onset strength and tempo estimation. Accuracy varies
        depending on the music style.

        Args:
            audio: Audio array (mono recommended)
            sample_rate: Sample rate in Hz

        Returns:
            Estimated BPM as float, or None if detection fails
        """
        if not LIBROSA_AVAILABLE:
            return None

        try:
            # Convert to mono if needed
            if audio.ndim == 2:
                audio = audio.mean(axis=0)

            # Estimate tempo
            tempo, _ = librosa.beat.beat_track(y=audio, sr=sample_rate)

            # librosa returns tempo as numpy scalar
            bpm = float(tempo)

            logger.debug(f"Detected BPM: {bpm:.1f}")
            return bpm

        except Exception as e:
            logger.warning(f"BPM detection failed: {e}")
            return None

    def _extract_key(
        self,
        audio: np.ndarray,
        sample_rate: int,
    ) -> Optional[str]:
        """
        Extract musical key from audio (experimental).

        This uses chromagram analysis to estimate the key. Accuracy is
        limited and depends heavily on the music content.

        Args:
            audio: Audio array (mono recommended)
            sample_rate: Sample rate in Hz

        Returns:
            Estimated key as string (e.g., "C major", "A minor"), or None
        """
        if not LIBROSA_AVAILABLE:
            return None

        try:
            # Convert to mono if needed
            if audio.ndim == 2:
                audio = audio.mean(axis=0)

            # Compute chromagram
            chroma = librosa.feature.chroma_cqt(y=audio, sr=sample_rate)

            # Average over time to get key profile
            chroma_mean = chroma.mean(axis=1)

            # Find dominant pitch class
            dominant_pitch = int(np.argmax(chroma_mean))

            # Map to key names
            pitch_classes = ['C', 'C#', 'D', 'D#', 'E', 'F',
                           'F#', 'G', 'G#', 'A', 'A#', 'B']

            # Simple major/minor detection based on intervals
            # This is very basic and experimental
            major_profile = np.array([1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1])
            minor_profile = np.array([1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0])

            # Roll profiles to match dominant pitch
            major_corr = np.correlate(
                chroma_mean,
                np.roll(major_profile, dominant_pitch),
                mode='valid'
            )
            minor_corr = np.correlate(
                chroma_mean,
                np.roll(minor_profile, dominant_pitch),
                mode='valid'
            )

            mode = "major" if major_corr > minor_corr else "minor"
            key = f"{pitch_classes[dominant_pitch]} {mode}"

            logger.debug(f"Detected key: {key} (experimental)")
            return key

        except Exception as e:
            logger.warning(f"Key detection failed: {e}")
            return None

    def get_info(self) -> dict:
        """
        Get extractor configuration information.

        Returns:
            Dictionary with extractor settings
        """
        return {
            "extract_bpm": self.extract_bpm,
            "extract_key": self.extract_key,
            "librosa_available": LIBROSA_AVAILABLE,
        }


if __name__ == "__main__":
    # Quick test
    print("AudioMetadataExtractor module")
    print("=" * 50)

    extractor = AudioMetadataExtractor(extract_bpm=True, extract_key=False)
    print(f"Configuration: {extractor.get_info()}")

    # Test with generated audio
    print("\nTesting with generated sine wave...")
    sample_rate = 32000
    duration = 2.0
    freq = 440.0

    t = np.linspace(0, duration, int(sample_rate * duration))
    audio = 0.5 * np.sin(2 * np.pi * freq * t)
    tensor = torch.from_numpy(audio.astype(np.float32))

    metadata = extractor.extract_metadata_from_tensor(tensor, sample_rate)
    print("\nMetadata:")
    for key, value in metadata.items():
        if isinstance(value, float):
            print(f"  {key}: {value:.3f}")
        else:
            print(f"  {key}: {value}")
