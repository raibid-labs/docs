"""
Mock Helpers for Testing
=========================

Mock implementations for testing without GPU/model dependencies.
"""

import logging
import time
from pathlib import Path
from typing import Tuple, Optional
from uuid import uuid4

import numpy as np
import torch

from services.generation.models import (
    GenerationRequest,
    GenerationResult,
    GenerationStatus,
    AudioMetadata,
)


logger = logging.getLogger(__name__)


class MockMusicGenerationEngine:
    """
    Mock generation engine for fast testing without GPU.

    This mock engine generates simple sine wave audio instead of
    using the actual MusicGen model. It's useful for testing
    workflows without requiring GPU hardware.
    """

    def __init__(
        self,
        model_name: str = "small",
        use_gpu: bool = False,
        enable_caching: bool = True,
        generation_delay: float = 0.1,
    ):
        """
        Initialize mock engine.

        Args:
            model_name: Model name (stored but not used)
            use_gpu: GPU flag (stored but not used)
            enable_caching: Caching flag (stored but not used)
            generation_delay: Artificial delay to simulate generation time
        """
        self.model_name = model_name
        self.use_gpu = use_gpu
        self.enable_caching = enable_caching
        self.generation_delay = generation_delay
        self.model = "mock_model"
        self.device = "mock_device"
        self.sample_rate = 32000
        self._generation_count = 0
        self._total_generation_time = 0.0

        logger.info(f"MockMusicGenerationEngine initialized (delay={generation_delay}s)")

    def load_model(self) -> None:
        """Mock model loading."""
        logger.debug("Mock model load (no-op)")
        time.sleep(0.01)  # Simulate minimal loading time

    def unload_model(self) -> None:
        """Mock model unloading."""
        logger.debug("Mock model unload (no-op)")

    def set_generation_params(self, **kwargs) -> None:
        """Mock parameter setting."""
        logger.debug(f"Mock set params: {kwargs}")

    def generate_audio(
        self,
        prompt: str,
        duration: float = 16.0,
        temperature: float = 1.0,
        top_k: int = 250,
        top_p: float = 0.0,
        cfg_coef: float = 3.0,
    ) -> Tuple[np.ndarray, int]:
        """
        Generate mock audio (sine wave).

        Args:
            prompt: Ignored (but logged)
            duration: Audio duration in seconds
            temperature: Ignored
            top_k: Ignored
            top_p: Ignored
            cfg_coef: Ignored

        Returns:
            Tuple of (audio_array, sample_rate)
        """
        logger.info(f"Mock generating audio: prompt='{prompt[:50]}...' duration={duration}s")

        # Simulate generation time
        time.sleep(self.generation_delay)

        # Generate simple test audio (stereo sine wave)
        samples = int(duration * self.sample_rate)
        t = np.linspace(0, duration, samples)

        # Use prompt length to vary frequency slightly (just for variety)
        base_freq = 440.0
        freq = base_freq + (len(prompt) % 100)

        # Generate stereo
        left = 0.5 * np.sin(2 * np.pi * freq * t)
        right = 0.5 * np.sin(2 * np.pi * freq * t * 1.01)  # Slight detune
        audio = np.stack([left, right])  # Shape: (2, samples)

        # Update stats
        self._generation_count += 1
        self._total_generation_time += self.generation_delay

        return audio.astype(np.float32), self.sample_rate

    def save_audio(
        self,
        audio: np.ndarray,
        sample_rate: int,
        output_path: Path,
        normalize: bool = True,
    ) -> AudioMetadata:
        """
        Save mock audio to file.

        Args:
            audio: Audio array (channels, samples)
            sample_rate: Sample rate in Hz
            output_path: Output file path
            normalize: Whether to normalize (ignored in mock)

        Returns:
            Audio metadata
        """
        import soundfile as sf

        logger.debug(f"Mock saving audio to {output_path}")

        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Transpose for soundfile (samples, channels)
        audio_t = audio.T

        # Save as WAV
        sf.write(
            str(output_path),
            audio_t,
            sample_rate,
            subtype='PCM_16'
        )

        # Get file metadata
        file_size = output_path.stat().st_size
        duration = audio.shape[1] / sample_rate
        channels = audio.shape[0]

        metadata = AudioMetadata(
            duration=duration,
            sample_rate=sample_rate,
            channels=channels,
            file_size_bytes=file_size,
            file_size_mb=file_size / (1024 * 1024),
            format="wav",
        )

        return metadata

    def generate(self, request: GenerationRequest) -> GenerationResult:
        """
        High-level mock generation function.

        Args:
            request: Generation request

        Returns:
            Generation result
        """
        job_id = f"gen_{uuid4().hex[:8]}"
        start_time = time.time()

        logger.info(f"Mock starting generation job: {job_id}")

        try:
            # Generate audio
            audio, sample_rate = self.generate_audio(
                prompt=request.prompt,
                duration=request.duration,
                temperature=request.temperature,
                top_k=request.top_k,
                top_p=request.top_p,
                cfg_coef=request.cfg_coef,
            )

            # Save to file
            from services.generation import engine as engine_module
            output_path = engine_module.settings.get_output_path(job_id)

            metadata = self.save_audio(
                audio=audio,
                sample_rate=sample_rate,
                output_path=output_path,
                normalize=True,
            )

            # Calculate total time
            total_time = time.time() - start_time

            # Build result
            result = GenerationResult(
                job_id=job_id,
                status=GenerationStatus.COMPLETED,
                prompt=request.prompt,
                model=f"mock-musicgen-{self.model_name}",
                file_url=f"/api/v1/files/{output_path.name}",
                file_path=str(output_path),
                metadata=metadata,
                generation_time_seconds=total_time,
                created_at=start_time,
                completed_at=time.time(),
            )

            return result

        except Exception as e:
            logger.error(f"Mock job failed: {job_id} - {e}")

            result = GenerationResult(
                job_id=job_id,
                status=GenerationStatus.FAILED,
                prompt=request.prompt,
                model=f"mock-musicgen-{self.model_name}",
                error_message=str(e),
                created_at=start_time,
            )

            return result

    def get_stats(self) -> dict:
        """Get mock engine statistics."""
        return {
            "model_name": f"mock-musicgen-{self.model_name}",
            "device": "mock",
            "model_loaded": True,
            "generation_count": self._generation_count,
            "total_generation_time": self._total_generation_time,
            "average_generation_time": (
                self._total_generation_time / self._generation_count
                if self._generation_count > 0
                else 0.0
            ),
        }


def create_mock_audio_tensor(
    duration: float = 1.0,
    sample_rate: int = 32000,
    channels: int = 2,
    frequency: float = 440.0,
) -> torch.Tensor:
    """
    Create a mock audio tensor for testing.

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
        # Create stereo with slight detune
        left = audio
        right = 0.5 * np.sin(2 * np.pi * frequency * 1.01 * t)
        audio = np.stack([left, right])
    else:
        audio = audio.reshape(1, -1)

    return torch.from_numpy(audio.astype(np.float32))


def mock_generation_failure(error_message: str = "Mock error") -> GenerationResult:
    """
    Create a mock failed generation result.

    Args:
        error_message: Error message

    Returns:
        Failed GenerationResult
    """
    return GenerationResult(
        job_id=f"gen_{uuid4().hex[:8]}",
        status=GenerationStatus.FAILED,
        prompt="test prompt",
        model="mock-musicgen-small",
        error_message=error_message,
        created_at=time.time(),
    )


def mock_generation_success(
    prompt: str = "test prompt",
    duration: float = 16.0,
    file_path: Optional[str] = None,
) -> GenerationResult:
    """
    Create a mock successful generation result.

    Args:
        prompt: Generation prompt
        duration: Audio duration
        file_path: Optional file path

    Returns:
        Successful GenerationResult
    """
    job_id = f"gen_{uuid4().hex[:8]}"

    if file_path is None:
        file_path = f"/tmp/{job_id}.wav"

    metadata = AudioMetadata(
        duration=duration,
        sample_rate=32000,
        channels=2,
        file_size_bytes=1024 * 1024 * 2,  # ~2MB
        file_size_mb=2.0,
        format="wav",
    )

    return GenerationResult(
        job_id=job_id,
        status=GenerationStatus.COMPLETED,
        prompt=prompt,
        model="mock-musicgen-small",
        file_url=f"/api/v1/files/{job_id}.wav",
        file_path=file_path,
        metadata=metadata,
        generation_time_seconds=0.1,
        created_at=time.time() - 1.0,
        completed_at=time.time(),
    )


class MockDatabaseSession:
    """Mock database session for testing without database."""

    def __init__(self):
        self.committed = False
        self.rolled_back = False
        self.closed = False
        self.objects = []

    def add(self, obj):
        self.objects.append(obj)

    def commit(self):
        self.committed = True

    def rollback(self):
        self.rolled_back = True

    def close(self):
        self.closed = True

    def query(self, *args):
        return self

    def filter(self, *args):
        return self

    def first(self):
        return None

    def all(self):
        return []

    def count(self):
        return 0


def mock_cuda_available(available: bool = False):
    """
    Mock torch.cuda.is_available() for testing.

    Args:
        available: Whether CUDA should be available

    Returns:
        Mock function
    """
    def _mock():
        return available

    return _mock
