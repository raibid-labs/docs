"""
Audio Export Module
====================

Convert PyTorch tensors to WAV files with professional loudness normalization.

This module provides the AudioExporter class which handles:
- PyTorch tensor to WAV conversion
- EBU R128 loudness normalization to -16 LUFS
- Mono/stereo support with multiple bit depths
- Automatic clipping prevention
- Batch export capabilities

Usage:
    from services.audio import AudioExporter

    exporter = AudioExporter(target_lufs=-16.0)
    output_path, file_size = exporter.export_wav(
        audio_tensor=tensor,
        output_path="output.wav",
        sample_rate=32000,
        normalize=True
    )
"""

import logging
from pathlib import Path
from typing import Tuple, List, Union, Optional

import numpy as np
import soundfile as sf
import torch

try:
    import pyloudnorm as pyln
    PYLOUDNORM_AVAILABLE = True
except ImportError:
    PYLOUDNORM_AVAILABLE = False
    logging.warning("pyloudnorm not available - loudness normalization will be disabled")


logger = logging.getLogger(__name__)


class AudioExporter:
    """
    Export PyTorch tensors to WAV files with professional audio processing.

    This class handles all aspects of exporting generated audio to disk:
    - Tensor format conversion (PyTorch → NumPy → WAV)
    - Loudness normalization using EBU R128 standard
    - Multiple bit depth support
    - GPU tensor handling (automatic CPU transfer)
    - Gradient cleanup (automatic detach)
    - Clipping prevention with fallback normalization

    Attributes:
        target_lufs (float): Target loudness in LUFS (default: -16.0)
    """

    SUPPORTED_BIT_DEPTHS = {
        'PCM_16': 16,
        'PCM_24': 24,
        'PCM_32': 32,
        'FLOAT': 32,
    }

    def __init__(self, target_lufs: float = -16.0):
        """
        Initialize the AudioExporter.

        Args:
            target_lufs: Target loudness in LUFS for normalization.
                        -16.0 is the standard for streaming platforms
                        (Spotify, YouTube, Apple Music).
        """
        self.target_lufs = target_lufs
        logger.info(f"AudioExporter initialized with target_lufs={target_lufs}")

        if not PYLOUDNORM_AVAILABLE:
            logger.warning(
                "pyloudnorm not available - loudness normalization disabled. "
                "Install with: pip install pyloudnorm"
            )

    def export_wav(
        self,
        audio_tensor: torch.Tensor,
        output_path: Union[str, Path],
        sample_rate: int = 32000,
        normalize: bool = True,
        bit_depth: str = 'PCM_16',
    ) -> Tuple[str, int]:
        """
        Export a PyTorch tensor to a WAV file.

        Args:
            audio_tensor: PyTorch tensor with shape (channels, samples) or (samples,)
            output_path: Path where WAV file will be saved
            sample_rate: Audio sample rate in Hz (default: 32000)
            normalize: Apply loudness normalization (default: True)
            bit_depth: Output bit depth - PCM_16, PCM_24, PCM_32, or FLOAT
                      (default: PCM_16 for maximum compatibility)

        Returns:
            Tuple of (output_path, file_size_bytes)

        Raises:
            ValueError: If tensor shape is invalid or bit_depth unsupported
            RuntimeError: If export fails
        """
        logger.debug(
            f"Exporting audio: shape={audio_tensor.shape}, "
            f"sample_rate={sample_rate}, normalize={normalize}, bit_depth={bit_depth}"
        )

        # Validate bit depth
        if bit_depth not in self.SUPPORTED_BIT_DEPTHS:
            raise ValueError(
                f"Unsupported bit depth: {bit_depth}. "
                f"Supported: {list(self.SUPPORTED_BIT_DEPTHS.keys())}"
            )

        # Convert tensor to numpy
        audio_np = self._tensor_to_numpy(audio_tensor)

        # Validate and reshape audio
        audio_np = self._validate_and_reshape(audio_np)

        # Normalize loudness if requested
        if normalize:
            audio_np = self._normalize_loudness(audio_np, sample_rate)

        # Ensure output directory exists
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Write WAV file
        try:
            sf.write(
                str(output_path),
                audio_np,
                sample_rate,
                subtype=bit_depth
            )
            logger.debug(f"WAV file written: {output_path}")
        except Exception as e:
            logger.error(f"Failed to write WAV file: {e}")
            raise RuntimeError(f"Could not write WAV file to {output_path}: {e}")

        # Get file size
        file_size = output_path.stat().st_size

        logger.info(
            f"Exported audio: {output_path.name} "
            f"({file_size / 1024:.1f} KB, {audio_np.shape[0] / sample_rate:.2f}s)"
        )

        return str(output_path), file_size

    def export_wav_batch(
        self,
        audio_tensors: List[torch.Tensor],
        output_paths: List[Union[str, Path]],
        sample_rate: int = 32000,
        normalize: bool = True,
        bit_depth: str = 'PCM_16',
    ) -> List[Tuple[str, int]]:
        """
        Export multiple audio tensors to WAV files efficiently.

        Args:
            audio_tensors: List of PyTorch tensors
            output_paths: List of output paths (same length as audio_tensors)
            sample_rate: Audio sample rate in Hz
            normalize: Apply loudness normalization to each file
            bit_depth: Output bit depth

        Returns:
            List of tuples (output_path, file_size_bytes)

        Raises:
            ValueError: If lengths don't match or inputs invalid
        """
        if len(audio_tensors) != len(output_paths):
            raise ValueError(
                f"Length mismatch: {len(audio_tensors)} tensors "
                f"but {len(output_paths)} paths"
            )

        logger.info(f"Batch exporting {len(audio_tensors)} audio files")

        results = []
        for i, (tensor, path) in enumerate(zip(audio_tensors, output_paths)):
            try:
                result = self.export_wav(
                    audio_tensor=tensor,
                    output_path=path,
                    sample_rate=sample_rate,
                    normalize=normalize,
                    bit_depth=bit_depth,
                )
                results.append(result)
                logger.debug(f"Batch export {i+1}/{len(audio_tensors)} complete")
            except Exception as e:
                logger.error(f"Failed to export file {i+1}/{len(audio_tensors)}: {e}")
                raise

        logger.info(f"Batch export complete: {len(results)} files")
        return results

    def _tensor_to_numpy(self, tensor: torch.Tensor) -> np.ndarray:
        """
        Convert PyTorch tensor to NumPy array.

        Handles:
        - GPU tensors (automatically moves to CPU)
        - Tensors with gradients (automatically detaches)
        - Type conversion to float32

        Args:
            tensor: PyTorch tensor

        Returns:
            NumPy array (float32)
        """
        # Move to CPU if on GPU
        if tensor.is_cuda:
            tensor = tensor.cpu()
            logger.debug("Moved tensor from GPU to CPU")

        # Detach from computation graph if needed
        if tensor.requires_grad:
            tensor = tensor.detach()
            logger.debug("Detached tensor from computation graph")

        # Convert to numpy
        audio_np = tensor.numpy()

        # Ensure float32
        if audio_np.dtype != np.float32:
            audio_np = audio_np.astype(np.float32)

        return audio_np

    def _validate_and_reshape(self, audio: np.ndarray) -> np.ndarray:
        """
        Validate audio array shape and reshape for soundfile.

        soundfile expects audio in format (samples, channels), but PyTorch
        generates (channels, samples). This method handles the conversion.

        Args:
            audio: NumPy array from PyTorch

        Returns:
            Reshaped array (samples, channels) or (samples,) for mono

        Raises:
            ValueError: If audio shape is invalid
        """
        if audio.ndim == 1:
            # Mono audio - already correct shape
            logger.debug(f"Mono audio: {len(audio)} samples")
            return audio

        elif audio.ndim == 2:
            # Could be (channels, samples) or (samples, channels)
            # We assume PyTorch format: (channels, samples)
            channels, samples = audio.shape

            if channels > 2:
                # If first dimension > 2, likely incorrect shape
                # Try to detect: if second dim is much larger, it's (samples, channels)
                if samples > channels * 10:
                    # Already in (samples, channels) format
                    logger.debug(f"Audio already in correct format: {audio.shape}")
                    return audio
                else:
                    raise ValueError(
                        f"Invalid audio shape: {audio.shape}. "
                        f"Expected max 2 channels, got {channels}"
                    )

            # Transpose from (channels, samples) to (samples, channels)
            audio_t = audio.T
            logger.debug(
                f"{'Stereo' if channels == 2 else 'Mono'} audio: "
                f"{samples} samples, {channels} channel(s)"
            )
            return audio_t

        else:
            raise ValueError(
                f"Invalid audio dimensions: {audio.ndim}. "
                f"Expected 1D (mono) or 2D (stereo)"
            )

    def _normalize_loudness(
        self,
        audio: np.ndarray,
        sample_rate: int,
    ) -> np.ndarray:
        """
        Normalize audio loudness to target LUFS using EBU R128 standard.

        This method:
        1. Measures integrated loudness using pyloudnorm
        2. Applies gain to reach target loudness
        3. Falls back to peak normalization if clipping would occur
        4. Handles silent audio gracefully

        Args:
            audio: NumPy array (samples,) or (samples, channels)
            sample_rate: Sample rate in Hz

        Returns:
            Normalized audio array

        Notes:
            If pyloudnorm is not available, returns original audio unchanged.
        """
        if not PYLOUDNORM_AVAILABLE:
            logger.debug("Skipping normalization (pyloudnorm not available)")
            return audio

        try:
            # Create loudness meter
            meter = pyln.Meter(sample_rate)

            # Measure loudness
            loudness = meter.integrated_loudness(audio)

            # Check if audio is silent or nearly silent
            if loudness < -70.0 or np.isinf(loudness):
                logger.warning(
                    f"Audio is silent or very quiet (loudness: {loudness:.1f} LUFS), "
                    f"skipping normalization"
                )
                return audio

            # Calculate gain needed
            gain_db = self.target_lufs - loudness

            # Normalize
            normalized = pyln.normalize.loudness(audio, loudness, self.target_lufs)

            # Check for clipping
            peak = np.abs(normalized).max()
            if peak > 1.0:
                logger.warning(
                    f"Normalization would cause clipping (peak: {peak:.2f}), "
                    f"falling back to peak normalization"
                )
                # Fall back to peak normalization
                normalized = audio / np.abs(audio).max() * 0.95
            else:
                logger.debug(
                    f"Normalized loudness: {loudness:.1f} → {self.target_lufs:.1f} LUFS "
                    f"(gain: {gain_db:+.1f} dB, peak: {peak:.2f})"
                )

            return normalized

        except Exception as e:
            logger.warning(f"Normalization failed: {e}, using original audio")
            return audio

    def get_info(self) -> dict:
        """
        Get exporter configuration information.

        Returns:
            Dictionary with exporter settings
        """
        return {
            "target_lufs": self.target_lufs,
            "normalization_available": PYLOUDNORM_AVAILABLE,
            "supported_bit_depths": list(self.SUPPORTED_BIT_DEPTHS.keys()),
        }


if __name__ == "__main__":
    # Quick test
    print("AudioExporter module")
    print("=" * 50)

    exporter = AudioExporter()
    print(f"Configuration: {exporter.get_info()}")

    # Test with dummy audio
    print("\nTesting with 1-second sine wave...")
    sample_rate = 32000
    duration = 1.0
    freq = 440.0  # A4 note

    t = np.linspace(0, duration, int(sample_rate * duration))
    audio = 0.5 * np.sin(2 * np.pi * freq * t)
    tensor = torch.from_numpy(audio.astype(np.float32))

    output_path = Path("test_output.wav")
    path, size = exporter.export_wav(tensor, output_path, sample_rate)
    print(f"Exported: {path} ({size} bytes)")

    if output_path.exists():
        output_path.unlink()
        print("Test file cleaned up")
